"""Offline contract 0.1 fixture. No R1 imports, discovery, files or hardware I/O."""
import json
import math
import secrets
import sys

MAX_LINE = 262144
MAX_BATCH = 8
MAX_DEPTH = 32


def offline_guard(event, args):
    # A regression tripwire, NOT a sandbox against malicious Python code.
    if event in ('open', 'os.listdir', 'os.scandir') or event.startswith(
        ('socket.', 'subprocess.', 'ctypes.', 'fcntl.')
    ):
        raise PermissionError('OFFLINE_ONLY')


# Hardware entrypoint explicitly supplies OFFLINE=False; offline fixtures and
# the simulator retain the original tripwire with no hardware fallback.
if globals().get('OFFLINE', True):
    sys.addaudithook(offline_guard)


def error(code, message, ident=None, kind=None):
    detail = {'code': code, 'message': message}
    if kind:
        detail['data'] = {'kind': kind, 'configurationOutcome': 'not_sent'}
    return {'jsonrpc': '2.0', 'id': ident, 'error': detail}


class Fault(Exception):
    def __init__(self, code, kind):
        self.code, self.kind = code, kind


def fields(value, expected):
    if not isinstance(value, dict) or set(value) != set(expected):
        raise Fault(-32602, 'INVALID_PARAMS')


def no_duplicates(pairs):
    obj = {}
    for key, value in pairs:
        if key in obj:
            raise ValueError('duplicate key')
        obj[key] = value
    return obj


def finite_float(text):
    value = float(text)
    if not math.isfinite(value):
        raise ValueError('nonfinite number')
    return value


def depth_ok(text):
    depth = 0
    quoted = escaped = False
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
                return False
        elif char in ']}':
            depth -= 1
    return True


class Plugin:
    def __init__(self):
        self.ready = self.stopping = False
        self.session = self.baseline = None
        self.value = 500
        self.ids = set()

    def invoke(self, method, params):
        if self.stopping:
            raise Fault(-32000, 'STOPPING')
        if method == 'plugin.hello':
            fields(params, ['apiVersions'])
            if params['apiVersions'] != ['0.1']:
                raise Fault(-32001, 'API_INCOMPATIBLE')
            if self.ready:
                raise Fault(-32000, 'ALREADY_READY')
            self.ready = True
            return {'apiVersion': '0.1', 'pluginId': 'dev.fake', 'mode': 'fake',
                    'hardwareAccess': False}
        if not self.ready:
            raise Fault(-32000, 'HANDSHAKE_REQUIRED')
        if method == 'plugin.get_capabilities':
            fields(params, [])
            return {'mode': 'fake', 'hardwareAccess': False, 'settings': [
                {'key': 'polling_rate', 'access': ['read', 'write'],
                 'schema': {'type': 'integer', 'enum': [125, 250, 500, 1000]},
                 'effects': 'simulation_only'}]}
        if method == 'device.open':
            fields(params, ['candidateId'])
            if params['candidateId'] != 'fake-device':
                raise Fault(-32602, 'UNKNOWN_FAKE_CANDIDATE')
            if self.session:
                raise Fault(-32000, 'ALREADY_OPEN')
            self.session = secrets.token_hex(16)
            self.baseline = None
            return {'session': self.session, 'mode': 'fake', 'hardwareAccess': False,
                    'identification': 'simulated'}
        if method in ('device.get_state', 'device.set_setting', 'device.close'):
            expected = {'device.get_state': ['session'], 'device.close': ['session'],
                        'device.set_setting': ['session', 'key', 'value', 'baselineToken']}[method]
            fields(params, expected)
            if not self.session or params['session'] != self.session:
                raise Fault(-32002, 'INVALID_SESSION')
            if method == 'device.close':
                self.session = self.baseline = None
                return {'mode': 'fake', 'closed': True}
            if method == 'device.set_setting':
                if params['key'] != 'polling_rate' or type(params['value']) is not int or params['value'] not in (125, 250, 500, 1000):
                    raise Fault(-32602, 'INVALID_SETTING')
                if not self.baseline or params['baselineToken'] != self.baseline:
                    raise Fault(-32003, 'CONFLICT')
                self.value = params['value']
            self.baseline = secrets.token_hex(16)
            return {'mode': 'fake', 'hardwareAccess': False, 'session': self.session,
                    'values': {'polling_rate': self.value}, 'baselineToken': self.baseline,
                    'outcome': 'simulated'}
        if method == 'plugin.shutdown':
            fields(params, [])
            self.session = self.baseline = None
            self.stopping = True
            return {'mode': 'fake', 'stopped': True}
        raise Fault(-32601, 'METHOD_NOT_FOUND')

    def one(self, request):
        if not isinstance(request, dict):
            return error(-32600, 'Invalid Request')
        ident = request.get('id')
        has_id = 'id' in request
        valid_id = ident is None or type(ident) in (int, float) or isinstance(ident, str)
        if isinstance(ident, str) and len(ident) > 128:
            valid_id = False
        if (request.get('jsonrpc') != '2.0' or not isinstance(request.get('method'), str)
                or set(request) - {'jsonrpc', 'id', 'method', 'params'} or not valid_id
                or ('params' in request and not isinstance(request['params'], (dict, list)))):
            return error(-32600, 'Invalid Request', ident if valid_id else None)
        # Only hello/capabilities are notification-safe. Other methods require an
        # ID-bearing request as explicit authorization; errors never get replies.
        if not has_id:
            if request['method'] in ('plugin.hello', 'plugin.get_capabilities'):
                try:
                    self.invoke(request['method'], request.get('params', {}))
                except Fault:
                    pass
            return None
        marker = ('number' if type(ident) in (int, float) else type(ident).__name__, ident)
        if marker in self.ids:
            return error(-32004, 'Duplicate ID', ident, 'DUPLICATE_ID')
        if len(self.ids) >= 4096:
            return error(-32005, 'Request limit', ident, 'REQUEST_LIMIT')
        self.ids.add(marker)
        try:
            result = self.invoke(request['method'], request.get('params', {}))
            return {'jsonrpc': '2.0', 'id': ident, 'result': result}
        except Fault as exc:
            return error(exc.code, exc.kind, ident, exc.kind)

    def frame(self, raw):
        if len(raw) > MAX_LINE:
            return error(-32600, 'Message limit', kind='MESSAGE_LIMIT')
        try:
            text = raw.decode('utf-8')
            if not depth_ok(text):
                return error(-32600, 'Depth limit', kind='DEPTH_LIMIT')
            request = json.loads(text, object_pairs_hook=no_duplicates,
                                 parse_float=finite_float,
                                 parse_constant=lambda _: (_ for _ in ()).throw(ValueError()))
        except (ValueError, UnicodeError, RecursionError):
            return error(-32700, 'Parse error')
        if isinstance(request, list):
            if not request or len(request) > MAX_BATCH:
                return error(-32600, 'Invalid batch or batch limit')
            responses = []
            for item in request:
                response = self.one(item)
                if response is not None:
                    responses.append(response)
            return responses or None
        return self.one(request)


def main():
    plugin = Plugin()
    print('offline fake started; hardware disabled' if globals().get('OFFLINE', True)
          else 'local hardware adapter started; explicit actions required', file=sys.stderr, flush=True)
    while not plugin.stopping:
        raw = sys.stdin.buffer.readline(MAX_LINE + 2)
        if not raw:
            break
        if not raw.endswith(b'\n') or len(raw) - 1 > MAX_LINE:
            response = error(-32600, 'Message limit or incomplete frame')
            print(json.dumps(response, separators=(',', ':')), flush=True)
            break
        response = plugin.frame(raw[:-1])
        if response is not None:
            encoded = json.dumps(response, separators=(',', ':'))
            if len(encoded.encode()) > MAX_LINE:
                break
            print(encoded, flush=True)


if __name__ == '__main__':
    main()
