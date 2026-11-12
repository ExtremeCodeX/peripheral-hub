"""Standard-library offline wire tests; run with python3 -B -m unittest discover ..."""
import json
from pathlib import Path
import subprocess
import unittest

SOURCE = (Path(__file__).resolve().parents[1] / 'src' / 'fake_plugin.py').read_text()


def request(method, ident=1, **params):
    return {'jsonrpc': '2.0', 'id': ident, 'method': method, 'params': params}


HELLO = request('plugin.hello', apiVersions=['0.1'])


class FakeTests(unittest.TestCase):
    def wire(self, frames):
        raw = b''.join((json.dumps(f).encode() if not isinstance(f, bytes) else f) + b'\n' for f in frames)
        child = subprocess.run(['/usr/bin/python3', '-I', '-S', '-u', '-c', SOURCE],
                               input=raw, capture_output=True, timeout=5, env={}, cwd='/')
        self.assertEqual(child.returncode, 0, child.stderr)
        self.assertIn(b'hardware disabled', child.stderr)
        return [json.loads(line) for line in child.stdout.splitlines()]

    def inside(self, code):
        # Test the real guard in a separate process, not in this unittest runner.
        program = "ns={'__name__':'fixture'}\nexec(" + repr(SOURCE) + ",ns)\np=ns['Plugin']()\n" + code
        child = subprocess.run(['/usr/bin/python3', '-I', '-S', '-u', '-c', program],
                               capture_output=True, timeout=5, env={}, cwd='/')
        self.assertEqual(child.returncode, 0, child.stderr)

    def test_handshake_valid_invalid_and_required(self):
        self.assertEqual(self.wire([HELLO])[0]['result']['mode'], 'fake')
        for versions in [[], ['1'], '0.1', None, [True]]:
            self.assertEqual(self.wire([request('plugin.hello', apiVersions=versions)])[0]['error']['code'], -32001)
        self.assertEqual(self.wire([request('device.open', candidateId='fake-device')])[0]['error']['data']['kind'], 'HANDSHAKE_REQUIRED')

    def test_invalid_json_utf8_keys_constants_and_depth(self):
        for raw in [b'{', b'\xff', b'{"a":1,"a":2}', b'NaN', b'Infinity', b'1e999', b'{} {}']:
            with self.subTest(raw=raw):
                self.assertEqual(self.wire([raw])[0]['error']['code'], -32700)
        raw = b'[' * 33 + b'0' + b']' * 33
        self.assertEqual(self.wire([raw])[0]['error']['data']['kind'], 'DEPTH_LIMIT')

    def test_invalid_requests_and_response_objects(self):
        for value in [None, 1, 'x', {}, {'jsonrpc': '1.0', 'method': 'x'},
                      {'jsonrpc': '2.0', 'method': 1},
                      {'jsonrpc': '2.0', 'method': 'x', 'id': True},
                      {'jsonrpc': '2.0', 'method': 'x', 'id': []},
                      {'jsonrpc': '2.0', 'method': 'x', 'id': 'x' * 129},
                      {'jsonrpc': '2.0', 'method': 'x', 'params': 1},
                      {'jsonrpc': '2.0', 'id': 1, 'result': {}},
                      {'jsonrpc': '2.0', 'method': 'x', 'shell': 'id'}]:
            with self.subTest(value=value):
                self.assertEqual(self.wire([value])[0]['error']['code'], -32600)

    def test_batch_mixed_invalid_elements_and_notifications(self):
        result = self.wire([[HELLO, {'jsonrpc': '2.0', 'method': 'unknown'}, 7,
                             request('plugin.get_capabilities', 2), [],
                             request('unknown', 'string-id')]])[0]
        self.assertEqual(len(result), 5)
        self.assertEqual([r['id'] for r in result], [1, None, 2, None, 'string-id'])
        self.assertEqual(result[1]['error']['code'], -32600)
        self.assertEqual(result[-1]['error']['code'], -32601)
        self.assertEqual(result[2]['result']['settings'][0]['schema']['enum'], [125, 250, 500, 1000])

    def test_empty_and_excess_batch_are_atomic_rejections(self):
        for batch in [[], [HELLO] * 9]:
            result = self.wire([batch, HELLO])
            self.assertEqual(result[0]['error']['code'], -32600)
            self.assertIn('result', result[1])  # oversize batch did not execute hello

    def test_notification_only_and_safe_notification_execution(self):
        self.assertEqual(self.wire([[{'jsonrpc': '2.0', 'method': 'unknown'}]]), [])
        self.assertEqual(self.wire([{'jsonrpc': '2.0', 'method': 'unknown'}]), [])
        result = self.wire([[{'jsonrpc': '2.0', 'method': 'plugin.hello', 'params': {'apiVersions': ['0.1']}},
                             request('plugin.get_capabilities', 2)]])
        self.assertEqual(len(result[0]), 1)
        self.assertIn('result', result[0][0])

    def test_notification_mutation_and_shutdown_never_authorize(self):
        result = self.wire([HELLO, {'jsonrpc': '2.0', 'method': 'device.open', 'params': {'candidateId': 'fake-device'}},
                            {'jsonrpc': '2.0', 'method': 'plugin.shutdown'},
                            request('device.open', 2, candidateId='fake-device')])
        self.assertEqual(len(result), 2)
        self.assertIn('result', result[1])

    def test_ids_null_string_numeric_duplicate_and_limits(self):
        for ident in [None, 'abc', -4, 0, 1.5]:
            result = self.wire([request('plugin.hello', ident, apiVersions=['0.1']),
                                request('plugin.get_capabilities', ident)])
            self.assertEqual(result[0]['id'], ident)
            self.assertEqual(result[1]['error']['data']['kind'], 'DUPLICATE_ID')
        self.inside("p.ids=set(('number',i) for i in range(4096))\nassert p.one({'jsonrpc':'2.0','id':5000,'method':'x'})['error']['data']['kind']=='REQUEST_LIMIT'\n")

    def test_method_params_strict_types_and_paths(self):
        for method, params in [('plugin.get_capabilities', {'path': '/dev/hidraw0'}),
                               ('device.open', {'candidateId': '/dev/hidraw0'}),
                               ('device.open', {'candidateId': 'fake-device', 'args': []}),
                               ('plugin.get_capabilities', []), ('plugin.shutdown', {'shell': 'id'})]:
            frame = {'jsonrpc': '2.0', 'id': 2, 'method': method, 'params': params}
            self.assertEqual(self.wire([HELLO, frame])[1]['error']['code'], -32602)

    def test_session_baseline_validation_and_simulated_effects(self):
        self.inside("""
p.invoke('plugin.hello',{'apiVersions':['0.1']})
s=p.invoke('device.open',{'candidateId':'fake-device'})['session']
assert p.baseline is None
r=p.invoke('device.get_state',{'session':s})
assert r['mode']=='fake' and r['hardwareAccess'] is False and r['outcome']=='simulated'
b=r['baselineToken']
for value in (True, 1000.0, 42, '1000', None):
    try: p.invoke('device.set_setting',{'session':s,'key':'polling_rate','value':value,'baselineToken':b})
    except ns['Fault'] as e: assert e.code==-32602
    else: raise AssertionError('invalid value accepted')
r=p.invoke('device.set_setting',{'session':s,'key':'polling_rate','value':1000,'baselineToken':b})
assert r['values']['polling_rate']==1000 and r['outcome']=='simulated'
try: p.invoke('device.set_setting',{'session':s,'key':'polling_rate','value':500,'baselineToken':b})
except ns['Fault'] as e: assert e.kind=='CONFLICT'
else: raise AssertionError('stale baseline accepted')
p.invoke('device.close',{'session':s})
new=p.invoke('device.open',{'candidateId':'fake-device'})['session']
assert new!=s and p.baseline is None
try: p.invoke('device.get_state',{'session':s})
except ns['Fault'] as e: assert e.kind=='INVALID_SESSION'
else: raise AssertionError('stale session accepted')
""")

    def test_serial_batch_shutdown_and_remaining_elements(self):
        result = self.wire([[HELLO, request('plugin.shutdown', 2), request('device.open', 3, candidateId='fake-device')]])[0]
        self.assertEqual(result[1]['result']['stopped'], True)
        self.assertEqual(result[2]['error']['data']['kind'], 'STOPPING')

    def test_oversize_and_incomplete_frames(self):
        self.assertEqual(self.wire([b'x' * 262145])[0]['error']['code'], -32600)
        child = subprocess.run(['/usr/bin/python3', '-I', '-S', '-u', '-c', SOURCE],
                               input=b'{}', capture_output=True, timeout=5, env={})
        self.assertEqual(json.loads(child.stdout)['error']['code'], -32600)

    def test_fragmented_pipes_multiple_lines_and_flush(self):
        child = subprocess.Popen(['/usr/bin/python3', '-I', '-S', '-u', '-c', SOURCE],
                                 stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, env={})
        try:
            encoded = json.dumps(HELLO).encode() + b'\n'
            for pos in range(0, len(encoded), 3):
                child.stdin.write(encoded[pos:pos + 3])
                child.stdin.flush()
            # communicate supplies more lines then EOF, with a bounded overall wait.
            out, _ = child.communicate(json.dumps(request('plugin.get_capabilities', 2)).encode() + b'\n', timeout=5)
            self.assertEqual([json.loads(x)['id'] for x in out.splitlines()], [1, 2])
        finally:
            if child.poll() is None:
                child.kill()
                child.wait()

    def test_offline_tripwire_blocks_device_files_discovery_and_network(self):
        self.inside("""
for event,args in [('open',('/dev/hidraw0','r',0)),('open',('/sys/bus/usb/devices','r',0)),('os.listdir',('/sys',)),('os.scandir',('/sys',)),('socket.__new__',()),('subprocess.Popen',()),('fcntl.ioctl',()),('ctypes.dlopen',())]:
    try: ns['offline_guard'](event,args)
    except PermissionError: pass
    else: raise AssertionError(event)
try: open('/dev/hidraw0','rb')
except PermissionError as e: assert str(e)=='OFFLINE_ONLY'
else: raise AssertionError('physical open not blocked')
""")


if __name__ == '__main__':
    unittest.main()
