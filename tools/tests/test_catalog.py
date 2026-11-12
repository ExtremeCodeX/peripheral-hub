"""Synthetic metadata/loopback HTTP only; no external requests or hardware."""
import copy
import http.client
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
from pathlib import Path
import socket
import tempfile
import threading
from types import SimpleNamespace
import unittest
from unittest.mock import patch

from test_publication import publication, ROOT, EXAMPLE

import importlib.util
spec = importlib.util.spec_from_file_location('build_catalog', ROOT / 'tools/build-catalog.py')
builder = importlib.util.module_from_spec(spec)
spec.loader.exec_module(builder)
catalog = builder.catalog
URL = 'https://example.org/catalog/v1/index.json'  # syntax fixture, never contacted


class CatalogTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(dir='/tmp/opencode' if Path('/tmp/opencode').is_dir() else None, prefix='catalog-')
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        (self.root / 'catalog/software').mkdir(parents=True)
        (self.root / 'doc.md').write_text('Synthetic only; not evidence or approval.')
        self.cache = self.root / 'cache/index.json'
        self.schema = publication.load_json(publication.SCHEMA)
        self.data = publication.load_json(ROOT / EXAMPLE)
        self.data.update(id='test.synthetic', name='Synthetic metadata fixture', version='0.0.0-fixture',
                         authors=['Synthetic fixture, not a person'], license='Synthetic fixture only',
                         source={'repository': 'https://example.org/source', 'commit': 'a' * 40},
                         videos=['https://example.org/video'], documents=['doc.md'], images=None,
                         documentation={'readme': 'doc.md'}, architectures=['fixture'], artifacts=[], pending=[])

    def index(self, entries=None):
        software = [self.data] if entries is None else entries
        return {'schemaVersion': 1, 'revision': catalog.revision(software), 'software': software}

    def sync(self, fetch, url=URL, refresh=True):
        return catalog.sync(url, self.cache, refresh, publication, self.schema, fetch=fetch)

    def reply(self, index=None, tag='"one"'):
        raw = json.dumps(self.index() if index is None else index).encode()
        return lambda *_: (200, raw, tag)

    def test_generator_add_update_remove_deterministic(self):
        path = self.root / 'catalog/software/test.synthetic.json'
        path.write_text(json.dumps(self.data))
        first = builder.build(self.root)
        self.assertEqual(first, builder.build(self.root))
        self.assertEqual(len(json.loads(first)['software']), 1)
        self.data['version'] = '0.0.1-fixture'
        path.write_text(json.dumps(self.data))
        self.assertNotEqual(first, builder.build(self.root))
        path.unlink()
        self.assertEqual(json.loads(builder.build(self.root))['software'], [])

    def test_generator_rejects_incomplete_duplicate_symlink_traversal(self):
        path = self.root / 'catalog/software/test.synthetic.json'
        path.write_text(json.dumps(self.data))
        other = path.with_name('duplicate.json')
        other.write_text(json.dumps(self.data))
        with self.assertRaises(ValueError): builder.build(self.root)
        other.unlink()
        path.unlink()
        path.symlink_to(self.root / 'doc.md')
        with self.assertRaises(ValueError): builder.build(self.root)
        path.unlink()
        for change in ({'pending': ['not approved']}, {'documents': ['../escape']}):
            data = {**self.data, **change}
            path.write_text(json.dumps(data))
            with self.assertRaises(ValueError): builder.build(self.root)

    def test_invalid_version_schema_content_revision_keep_cache(self):
        self.sync(self.reply())
        before = self.cache.read_bytes()
        mutations = [({'schemaVersion': 2}, 'UNSUPPORTED_VERSION'),
                     ({'schemaVersion': True}, 'UNSUPPORTED_VERSION'),
                     ({'revision': '0' * 64}, 'INVALID_REVISION'),
                     ({'software': [{**self.data, 'entrypoint': 'evil'}]}, 'INVALID_CONTENT'),
                     ({'extra': True}, 'INVALID_CONTENT')]
        for change, expected in mutations:
            result = self.sync(self.reply({**self.index(), **change}))
            self.assertEqual(result['status'], expected)
            self.assertEqual(result['index'], self.index())
            self.assertEqual(before, self.cache.read_bytes())

    def test_json_utf8_duplicates_nonfinite_size_and_no_cache_deletion(self):
        self.sync(self.reply())
        before = self.cache.read_bytes()
        for raw, expected in [(b'{broken', 'INVALID_JSON'), (b'{"a":1,"a":2}', 'INVALID_JSON'),
                              (b'{"a":NaN}', 'INVALID_JSON'), (b'{"a":1e9999}', 'INVALID_JSON'), (b'{"a":"\\ud800"}', 'INVALID_JSON'), (b'{}' .decode().encode('utf-16'), 'INVALID_JSON'),
                              (b' ' * (catalog.MAX_BYTES + 1), 'TOO_LARGE')]:
            result = self.sync(lambda *_: (200, raw, '"invalid-response"'))
            self.assertEqual(result['status'], expected)
            self.assertEqual(before, self.cache.read_bytes())

    def test_cache_304_origin_etag_corruption_and_errors(self):
        self.sync(self.reply())
        calls = []
        def unchanged(url, tag):
            calls.append((url, tag)); return 304, None, None
        self.assertEqual(self.sync(unchanged)['status'], 'NOT_MODIFIED')
        self.assertEqual(calls[-1], (URL, '"one"'))
        changed = self.sync(unchanged, url='https://example.org/other/index.json')
        self.assertEqual(changed['status'], 'UNEXPECTED_304')
        self.assertIsNone(calls[-1][1])
        for failure in ('NETWORK', 'TIMEOUT', 'REDIRECT', 'CONTENT_TYPE', 'HTTP_ERROR'):
            def fail(*_): raise catalog.CatalogError(failure)
            result = self.sync(fail)
            self.assertEqual(result['status'], failure)
            self.assertIsNotNone(result['index'])
        self.cache.write_text('{broken')
        result = self.sync(unchanged)
        self.assertEqual(result['cacheWarning'], 'CACHE_ERROR')
        self.assertIsNone(result['index'])
        self.assertEqual(self.cache.read_text(), '{broken')
        self.sync(self.reply(tag='bad\r\nheader'))
        self.sync(unchanged)
        self.assertIsNone(calls[-1][1])

    def test_cache_symlink_special_write_failure_and_empty_snapshot(self):
        self.cache.parent.mkdir()
        self.cache.symlink_to(self.root / 'doc.md')
        result = self.sync(self.reply())
        self.assertEqual(result['cacheWarning'], 'CACHE_WRITE_ERROR')
        self.assertIsNotNone(result['index'])
        self.cache.unlink()
        self.sync(self.reply())
        result = self.sync(self.reply(self.index([])))
        self.assertEqual(result['index']['software'], [])
        def forbidden(*_): self.fail('network on cache-only/not configured')
        self.assertEqual(self.sync(forbidden, refresh=False)['status'], 'CACHED')
        self.assertEqual(self.sync(forbidden, url='')['status'], 'NOT_CONFIGURED')

    def test_atomic_cache_failure_does_not_replace_index_or_etag(self):
        self.sync(self.reply())
        before = self.cache.read_bytes()
        with patch.object(catalog.os, 'replace', side_effect=OSError('fixture disk failure')):
            result = self.sync(self.reply(self.index([]), tag='"new"'))
        self.assertEqual(result['index']['software'], [])
        self.assertEqual(result['cacheWarning'], 'CACHE_WRITE_ERROR')
        self.assertEqual(before, self.cache.read_bytes())
        self.assertEqual(list(self.cache.parent.glob('.catalog-*')), [])

    def test_urls_dns_ssrf_no_external_connections(self):
        for url in ('http://example.org/index.json', 'https://127.0.0.1/index.json',
                    'https://example.org:443/index.json', 'https://user@example.org/index.json',
                    'https://example.org/../index.json', 'https://example.org/%2e/index.json',
                    'https://example.org/index.json?token=secret', 'https://example.org/index.json#x'):
            with self.subTest(url=url), self.assertRaises(catalog.CatalogError): catalog.endpoint(url)
        for address in ('127.0.0.1', '10.0.0.1', '169.254.169.254', '::1', 'fc00::1', '192.0.2.1', '64:ff9b::a00:1', '2002:0a00:0001::1'):
            with patch.object(socket, 'getaddrinfo', return_value=[(socket.AF_INET, socket.SOCK_STREAM, 6, '', (address, 443))]), patch.object(socket, 'socket', side_effect=AssertionError('must block before socket')):
                with self.assertRaisesRegex(catalog.CatalogError, 'UNSAFE_ADDRESS'): catalog.request(URL, None)

    def test_timeout_network_and_inert_artifact_are_not_execution(self):
        for failure, expected in ((socket.timeout(), 'TIMEOUT'), (OSError(), 'NETWORK')):
            with patch.object(socket, 'getaddrinfo', side_effect=failure):
                with self.assertRaisesRegex(catalog.CatalogError, expected): catalog.request(URL, None)
        script = self.root / 'not-a-plugin.py'
        marker = self.root / 'must-not-exist'
        script.write_text(f'from pathlib import Path; Path({str(marker)!r}).write_text("executed")')
        import hashlib
        self.data['artifacts'] = [{'path': script.name, 'sha256': hashlib.sha256(script.read_bytes()).hexdigest(), 'target': 'Linux synthetic fixture only'}]
        (self.root / 'catalog/software/test.synthetic.json').write_text(json.dumps(self.data))
        builder.build(self.root)
        self.sync(self.reply())
        self.assertFalse(marker.exists())
        # Remote references are validated syntactically, never opened locally.
        with patch.object(publication, 'local_file', side_effect=AssertionError('remote path opened')):
            catalog.validate_index(self.index(), publication, self.schema)

    def test_real_loopback_http_conditional_transport_headers_and_removal(self):
        state = {'index': self.index(), 'tag': '"local-1"', 'status': 200, 'type': 'application/json', 'length': None}
        received = []
        class Handler(BaseHTTPRequestHandler):
            def log_message(self, *_): pass
            def do_GET(self):
                received.append(dict(self.headers))
                if self.headers.get('If-None-Match') == state['tag'] and state['status'] == 200:
                    self.send_response(304); self.end_headers(); return
                self.send_response(state['status'])
                self.send_header('Content-Type', state['type'])
                self.send_header('ETag', state['tag'])
                raw = json.dumps(state['index']).encode()
                self.send_header('Content-Length', str(state['length'] or len(raw)))
                self.end_headers()
                try: self.wfile.write(raw)
                except (BrokenPipeError, ConnectionResetError): pass
        server = ThreadingHTTPServer(('127.0.0.1', 0), Handler)
        thread = threading.Thread(target=server.serve_forever, daemon=True); thread.start()
        self.addCleanup(server.server_close)
        self.addCleanup(server.shutdown)
        # Test-only seam replaces DNS/TLS socket by a local HTTP connection.
        # All response parsing, header/size checks and cache code stay real.
        original_connection = http.client.HTTPConnection
        def connection(*_, **__): return original_connection('127.0.0.1', server.server_port, timeout=2)
        class Socket:
            def settimeout(self, *_): pass
            def connect(self, *_): pass
            def close(self): pass
        class TLS:
            def wrap_socket(self, *_, **__): return None
        public_dns = [(socket.AF_INET, socket.SOCK_STREAM, 6, '', ('93.184.216.34', 443))]
        def local_fetch(url, tag):
            conn = connection()
            class LocalConnection:
                sock = None
                def request(self, *a, **k): return conn.request(*a, **k)
                def getresponse(self): return conn.getresponse()
                def close(self): return conn.close()
            fake_socket = SimpleNamespace(getaddrinfo=lambda *_a, **_k: public_dns, socket=lambda *_: Socket(), SOCK_STREAM=socket.SOCK_STREAM, timeout=socket.timeout)
            with patch.object(catalog, 'socket', fake_socket), patch.object(catalog.ssl, 'create_default_context', return_value=TLS()), patch.object(catalog.http.client, 'HTTPSConnection', return_value=LocalConnection()):
                return catalog.request(url, tag)
        self.assertEqual(self.sync(local_fetch)['status'], 'UPDATED')
        self.assertEqual(self.sync(local_fetch)['status'], 'NOT_MODIFIED')
        self.assertEqual(received[-1]['If-None-Match'], '"local-1"')
        self.assertNotIn('Cookie', received[-1]); self.assertNotIn('Authorization', received[-1])
        state.update(index=self.index([]), tag='"local-2"')
        self.assertEqual(self.sync(local_fetch)['index']['software'], [])
        for status, expected in ((302, 'REDIRECT'), (503, 'HTTP_ERROR')):
            state.update(status=status)
            self.assertEqual(self.sync(local_fetch)['status'], expected)
        state.update(status=200, type='text/html', tag='"different"')
        self.assertEqual(self.sync(local_fetch)['status'], 'CONTENT_TYPE')
        state.update(type='application/json', length=catalog.MAX_BYTES + 1)
        self.assertEqual(self.sync(local_fetch)['status'], 'TOO_LARGE')


if __name__ == '__main__': unittest.main()
