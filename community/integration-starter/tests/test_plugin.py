"""Offline tests using the copied starter only; no Hub or device imports."""
import json
import select
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
PYTHON = '/usr/bin/python3'


def request(method, params=None, identifier=1):
    return {'jsonrpc': '2.0', 'id': identifier, 'method': method,
            'params': {} if params is None else params}


HELLO = request('plugin.hello', {'apiVersions': ['0.1']})


class StarterTests(unittest.TestCase):
    def run_peer(self, data, root=ROOT):
        return subprocess.run([PYTHON, '-I', '-S', '-B', str(root / 'plugin.py')],
                              input=data, capture_output=True, timeout=3,
                              cwd='/', env={})

    def exchange(self, *requests, root=ROOT):
        data = b''.join(json.dumps(item).encode() + b'\n' for item in requests)
        result = self.run_peer(data, root)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stderr, b'')
        return [json.loads(line) for line in result.stdout.splitlines()]

    def test_standalone_copy_without_workspace_or_port(self):
        with tempfile.TemporaryDirectory(prefix='starter-', dir='/tmp/opencode') as directory:
            root = Path(directory)
            shutil.copyfile(ROOT / 'plugin.py', root / 'plugin.py')
            replies = self.exchange(HELLO, request('plugin.get_capabilities', identifier=2),
                                    request('plugin.shutdown', identifier=3), root=root)
        self.assertEqual(replies[0]['result']['pluginId'], 'dev.starter')
        self.assertFalse(replies[0]['result']['hardwareAccess'])
        self.assertEqual(replies[1]['result']['settings'][0]['key'], 'sample_value')
        self.assertEqual(replies[1]['result']['settings'][0]['effects'], 'simulation_only')
        self.assertTrue(replies[2]['result']['stopped'])

    def test_handshake_required_and_incompatible(self):
        replies = self.exchange(request('plugin.get_capabilities'),
                                request('plugin.hello', {'apiVersions': ['9']}), HELLO)
        self.assertEqual(replies[0]['error']['message'], 'HANDSHAKE_REQUIRED')
        self.assertEqual(replies[1]['error']['message'], 'API_INCOMPATIBLE')
        self.assertIn('result', replies[2])

    def test_no_device_or_arbitrary_execution(self):
        for method in ['device.discover', 'shell', '/bin/sh']:
            with self.subTest(method=method):
                reply = self.exchange(HELLO, request(method, {'path': '/dev/hidraw0'}))[1]
                self.assertEqual(reply['error']['code'], -32601)
        for method in ['device.open', 'device.get_state', 'device.set_setting']:
            self.assertEqual(self.exchange(HELLO, request(method, {'path': '/dev/hidraw0'}))[1]['error']['code'], -32602)

    def test_real_pipe_state_baseline_session_notifications_and_restart(self):
        for _ in range(2):
            peer = subprocess.Popen([PYTHON, '-I', '-S', '-B', str(ROOT / 'plugin.py')],
                                    stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                    cwd='/', env={})
            def exchange(value):
                peer.stdin.write(json.dumps(value).encode() + b'\n')
                peer.stdin.flush()
                self.assertTrue(select.select([peer.stdout], [], [], 3)[0], 'RPC timed out')
                return json.loads(peer.stdout.readline())
            try:
                exchange(HELLO)
                session = exchange(request('device.open', {'candidateId': 'fake-device'}))['result']['session']
                state = exchange(request('device.get_state', {'session': session}))['result']
                self.assertEqual(state['values']['sample_value'], 50)
                params = {'session': session, 'baselineToken': state['baselineToken'], 'key': 'sample_value', 'value': 75}
                notify = request('device.set_setting', params)
                del notify['id']
                peer.stdin.write(json.dumps(notify).encode() + b'\n'); peer.stdin.flush()
                # A notification cannot mutate state or consume the baseline.
                changed = exchange(request('device.set_setting', params))['result']
                self.assertEqual(changed['values']['sample_value'], 75)
                self.assertFalse(changed['hardwareAccess'])
                self.assertEqual(exchange(request('device.set_setting', params))['error']['message'], 'CONFLICT')
                self.assertEqual(exchange(request('device.get_state', {'session': 'invalid'}))['error']['message'], 'INVALID_SESSION')
                self.assertEqual(exchange(request('device.get_state', {'session': session}))['result']['values']['sample_value'], 75)
                exchange(request('plugin.shutdown'))
                self.assertEqual(peer.wait(timeout=3), 0)
                self.assertEqual(peer.stderr.read(), b'')
            finally:
                if peer.poll() is None: peer.kill(); peer.wait()
                for pipe in [peer.stdin, peer.stdout, peer.stderr]: pipe.close()

    def test_params_and_extra_envelope_fields_rejected(self):
        extra = dict(HELLO, source='arbitrary')
        replies = self.exchange(HELLO, request('plugin.get_capabilities', {'args': []}),
                                request('plugin.shutdown', []), extra)
        self.assertEqual([r['error']['code'] for r in replies[1:]],
                         [-32602, -32602, -32600])

    def test_invalid_ids_and_requests(self):
        for value in [True, 1.5, [], {}, 'x' * 129]:
            with self.subTest(identifier=repr(value)[:20]):
                reply = self.exchange(dict(HELLO, id=value))[0]
                self.assertEqual(reply['error']['code'], -32600)
        for value in [None, 1, 'text', {'jsonrpc': '2.0', 'id': 1, 'result': {}}]:
            self.assertEqual(self.exchange(value)[0]['error']['code'], -32600)

    def test_notifications_and_serial_shutdown(self):
        notify = request('plugin.shutdown')
        del notify['id']
        batch = [notify, request('plugin.get_capabilities', identifier='caps'),
                 request('plugin.shutdown', identifier=None),
                 request('plugin.get_capabilities', identifier=4)]
        replies = self.exchange(HELLO, batch)[1]
        self.assertEqual(len(replies), 3)
        self.assertEqual(replies[0]['id'], 'caps')
        self.assertIsNone(replies[1]['id'])
        self.assertTrue(replies[1]['result']['stopped'])
        self.assertEqual(replies[2]['error']['message'], 'STOPPING')

    def test_only_notifications_have_no_output(self):
        notify = dict(HELLO)
        del notify['id']
        self.assertEqual(self.exchange([notify, notify]), [])

    def test_batch_bounds_and_invalid_element(self):
        replies = self.exchange([], [HELLO] * 9, request('plugin.get_capabilities'))
        self.assertEqual([r['error']['code'] for r in replies], [-32600, -32600, -32000])
        mixed = self.exchange([False, HELLO])[0]
        self.assertEqual(mixed[0]['error']['code'], -32600)
        self.assertIn('result', mixed[1])

    def test_malformed_duplicate_nonfinite_utf8_depth(self):
        for frame in [b'{', b'\xff', b'{"id":1,"id":2}', b'NaN', b'Infinity',
                      b'1e999', b'[' * 33 + b'0' + b']' * 33]:
            with self.subTest(frame=frame[:30]):
                result = self.run_peer(frame + b'\n')
                self.assertEqual(result.returncode, 0)
                self.assertEqual(json.loads(result.stdout)['error']['code'], -32700)

    def test_oversize_and_truncated_frame_exit(self):
        for frame in [b'x' * 16385 + b'\n', json.dumps(HELLO).encode()]:
            result = self.run_peer(frame)
            self.assertEqual(result.returncode, 2)
            self.assertEqual(result.stdout, b'')

    def test_frame_budget(self):
        result = self.run_peer((json.dumps(HELLO).encode() + b'\n') * 257)
        self.assertEqual(result.returncode, 2)
        self.assertEqual(len(result.stdout.splitlines()), 256)

    def test_dispatch_under_offline_io_tripwire(self):
        # Exercise the implementation under a regression detector, not a sandbox.
        code = '''import runpy, sys
module = runpy.run_path(sys.argv[1], run_name='starter_test')
def guard(event, args):
    if event == 'open' or event.startswith(('socket.', 'subprocess.', 'fcntl.')):
        raise RuntimeError('OFFLINE_ONLY')
sys.addaudithook(guard)
peer = module['Starter']()
assert 'result' in peer.one({'jsonrpc':'2.0','id':1,'method':'plugin.hello','params':{'apiVersions':['0.1']}})
opened=peer.one({'jsonrpc':'2.0','id':2,'method':'device.open','params':{'candidateId':'fake-device'}})['result']
state=peer.one({'jsonrpc':'2.0','id':3,'method':'device.get_state','params':{'session':opened['session']}})['result']
changed=peer.one({'jsonrpc':'2.0','id':4,'method':'device.set_setting','params':{'session':opened['session'],'baselineToken':state['baselineToken'],'key':'sample_value','value':75}})['result']
assert changed['values']['sample_value']==75 and changed['hardwareAccess'] is False
for name in ('device.discover', 'shell'):
    assert peer.one({'jsonrpc':'2.0','id':5,'method':name})['error']['code'] == -32601
'''
        result = subprocess.run([PYTHON, '-I', '-S', '-B', '-c', code, str(ROOT / 'plugin.py')],
                                capture_output=True, timeout=3, cwd='/', env={})
        self.assertEqual(result.returncode, 0, result.stderr)


if __name__ == '__main__':
    unittest.main()
