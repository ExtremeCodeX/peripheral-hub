"""Standalone JSON-RPC simulator. Memory only; no physical device or R1 code.

SPDX-License-Identifier: Apache-2.0
The Hub accepts this reviewed source only through its explicit development gate.
"""
import json
import secrets
import sys

MAX_FRAME = 16384
MAX_BATCH = 8
MAX_DEPTH = 32
MAX_FRAMES = 256
PLUGIN_ID = 'dev.starter'
# Replace these declarations and corresponding validation for your simulator.
SETTINGS = [{'key': 'sample_value', 'access': ['read', 'write'],
             'schema': {'type': 'integer', 'enum': [0, 25, 50, 75, 100]},
             'effects': 'simulation_only'}]
INITIAL_VALUES = {'sample_value': 50}


def valid_value(schema, value):
    kind = schema['type']
    if kind == 'boolean':
        return type(value) is bool
    if kind == 'integer' and type(value) is not int:
        return False
    if kind == 'number' and type(value) not in (int, float):
        return False
    if kind == 'string' and not isinstance(value, str):
        return False
    if 'enum' in schema:
        return value in schema['enum']
    if kind in ('integer', 'number'):
        return (schema['minimum'] <= value <= schema['maximum']
                and abs(value / schema['multipleOf'] - round(value / schema['multipleOf'])) < 1e-9)
    return False


def offline_guard(event, args):
    # Regression detector, not a sandbox against malicious Python code.
    if event in ('open', 'os.listdir', 'os.scandir') or event.startswith(
            ('socket.', 'subprocess.', 'ctypes.', 'fcntl.')):
        raise PermissionError('OFFLINE_ONLY')


def error(identifier, code, message):
    return {'jsonrpc': '2.0', 'id': identifier,
            'error': {'code': code, 'message': message}}


def unique_pairs(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError('duplicate key')
        result[key] = value
    return result


def reject_constant(value):
    raise ValueError('non-finite number')


def decode(frame):
    # Bound nesting before json.loads, including deeply nested invalid JSON.
    text = frame.decode('utf-8')
    depth, quoted, escaped = 0, False, False
    for char in text:
        if quoted:
            if escaped:
                escaped = False
            elif char == '\\':
                escaped = True
            elif char == '"':
                quoted = False
        elif char == '"':
            quoted = True
        elif char in '[{':
            depth += 1
            if depth > MAX_DEPTH:
                raise ValueError('nesting limit')
        elif char in ']}':
            depth -= 1
    # Reject non-finite floats,
    # including overflow notation, without trusting the default float parser.
    def finite_float(value):
        number = float(value)
        if number in (float('inf'), -float('inf')):
            raise ValueError('numeric overflow')
        return number
    return json.loads(text, object_pairs_hook=unique_pairs,
                      parse_constant=reject_constant, parse_float=finite_float)


class Starter:
    def __init__(self):
        self.ready = False
        self.stopping = False
        self.session = None
        self.baseline = None
        self.values = dict(INITIAL_VALUES)

    def snapshot(self):
        self.baseline = secrets.token_hex(16)
        return {'mode': 'fake', 'hardwareAccess': False, 'session': self.session,
                'baselineToken': self.baseline, 'values': dict(self.values),
                'outcome': 'simulated'}

    def device(self, method, params, identifier):
        fields = {
            'device.open': {'candidateId'}, 'device.close': {'session'},
            'device.get_state': {'session'},
            'device.set_setting': {'session', 'baselineToken', 'key', 'value'},
        }
        if method not in fields:
            return error(identifier, -32601, 'Method not found')
        if not isinstance(params, dict) or set(params) != fields[method]:
            return error(identifier, -32602, 'Invalid params')
        if method == 'device.open':
            if params['candidateId'] != 'fake-device' or self.session is not None:
                return error(identifier, -32602, 'Invalid simulated candidate or already open')
            self.session = secrets.token_hex(16)
            return {'mode': 'fake', 'hardwareAccess': False, 'session': self.session,
                    'identification': 'simulated'}
        if self.session is None or params['session'] != self.session:
            return error(identifier, -32002, 'INVALID_SESSION')
        if method == 'device.close':
            self.session = self.baseline = None
            return {'mode': 'fake', 'closed': True}
        if method == 'device.set_setting':
            if self.baseline is None or params['baselineToken'] != self.baseline:
                return error(identifier, -32003, 'CONFLICT')
            setting = next((s for s in SETTINGS if s['key'] == params['key']), None)
            if (setting is None or 'write' not in setting['access']
                    or not valid_value(setting['schema'], params['value'])):
                return error(identifier, -32602, 'Invalid simulated setting')
            self.values[params['key']] = params['value']
        return self.snapshot()

    def one(self, request):
        if not isinstance(request, dict):
            return error(None, -32600, 'Invalid Request')
        identifier = request.get('id')
        valid_id = (identifier is None or type(identifier) is int
                    or (isinstance(identifier, str) and len(identifier) <= 128))
        if (set(request) - {'jsonrpc', 'id', 'method', 'params'}
                or request.get('jsonrpc') != '2.0'
                or not isinstance(request.get('method'), str) or not valid_id
                or not isinstance(request.get('params', {}), (dict, list))):
            return error(None, -32600, 'Invalid Request')
        notification = 'id' not in request
        method, params = request['method'], request.get('params', {})
        response = None
        if self.stopping:
            response = error(identifier, -32000, 'STOPPING')
        elif method == 'plugin.hello':
            if not isinstance(params, dict) or set(params) != {'apiVersions'}:
                response = error(identifier, -32602, 'Invalid params')
            elif params['apiVersions'] != ['0.1']:
                response = error(identifier, -32001, 'API_INCOMPATIBLE')
            else:
                self.ready = True
                response = {'apiVersion': '0.1', 'pluginId': PLUGIN_ID,
                            'mode': 'fake', 'hardwareAccess': False}
        elif not self.ready:
            response = error(identifier, -32000, 'HANDSHAKE_REQUIRED')
        elif method in ('plugin.get_capabilities', 'plugin.shutdown'):
            if params != {}:
                response = error(identifier, -32602, 'Invalid params')
            elif method == 'plugin.get_capabilities':
                response = {'mode': 'fake', 'hardwareAccess': False,
                            'settings': SETTINGS}
            elif not notification:
                self.stopping = True
                self.session = self.baseline = None
                response = {'mode': 'fake', 'stopped': True}
        elif method.startswith('device.'):
            # Fire-and-forget never mutates state or rotates baselines.
            if notification:
                return None
            response = self.device(method, params, identifier)
        else:
            # Unknown methods, shell and paths never become execution or
            # fabricated configuration success.
            response = error(identifier, -32601, 'Method not found')
        if notification:
            return None
        if isinstance(response, dict) and 'jsonrpc' in response:
            return response
        return {'jsonrpc': '2.0', 'id': identifier, 'result': response}

    def dispatch(self, request):
        if not isinstance(request, list):
            return self.one(request)
        if not 1 <= len(request) <= MAX_BATCH:
            return error(None, -32600, 'Invalid batch')
        responses = [self.one(item) for item in request]
        return [item for item in responses if item is not None] or None


def main():
    sys.addaudithook(offline_guard)
    plugin = Starter()
    for _ in range(MAX_FRAMES):
        frame = sys.stdin.buffer.readline(MAX_FRAME + 2)
        if not frame:
            return 0
        if not frame.endswith(b'\n') or len(frame) > MAX_FRAME + 1:
            # Do not resynchronize a truncated or oversized frame.
            return 2
        try:
            request = decode(frame[:-1])
        except (ValueError, RecursionError):
            response = error(None, -32700, 'Parse error')
        else:
            response = plugin.dispatch(request)
        if response is not None:
            print(json.dumps(response, separators=(',', ':'), allow_nan=False), flush=True)
        if plugin.stopping:
            return 0
    return 2


if __name__ == '__main__':
    sys.exit(main())
