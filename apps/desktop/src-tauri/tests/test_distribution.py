import hashlib
import importlib.util
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

spec=importlib.util.spec_from_file_location('distribution',Path(__file__).resolve().parents[1]/'src/distribution_worker.py')
worker=importlib.util.module_from_spec(spec);spec.loader.exec_module(worker)


class DistributionTests(unittest.TestCase):
    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory(prefix='hub-release-')
        self.base=Path(self.tmp.name)
        self.fd=worker.root(str(self.base))
        self.source=b'raise RuntimeError("installer must never execute this")\n'
        self.license=b'License fixture'
        package=dict(id='io.test.mouse',version='1.0.0',sha256=hashlib.sha256(self.source).hexdigest(),byteLength=len(self.source),artifact='https://github.com/test/mouse/releases/download/v1.0.0/plugin.py',licenseUrl='https://github.com/test/mouse/releases/download/v1.0.0/LICENSE',licenseSha256=hashlib.sha256(self.license).hexdigest())
        self.manifest=json.dumps(package).encode()
        self.entry=dict(package=package,manifestUrl='https://github.com/test/mouse/releases/download/v1.0.0/manifest.json',manifestSha256=hashlib.sha256(self.manifest).hexdigest())
    def tearDown(self):
        os.close(self.fd);self.tmp.cleanup()
    def fetch(self,url,limit):return self.license if url.endswith('LICENSE') else self.source if url.endswith('.py') else self.manifest
    def test_install_is_atomic_idempotent_and_never_executes(self):
        self.assertFalse(worker.installed(self.fd,self.entry))
        worker.install(self.fd,self.entry,self.fetch)
        self.assertTrue(worker.installed(self.fd,self.entry))
        with patch.object(worker,'download',side_effect=AssertionError('must not download twice')):
            worker.install(self.fd,self.entry)
        path=self.base/'peripheral-hub/plugins/io.test.mouse'
        self.assertEqual((path/'plugin.py').read_bytes(),self.source)
        self.assertEqual((path/'plugin.py').stat().st_mode&0o777,0o600)
    def test_tampered_downloads_do_not_install(self):
        for bad in ('manifest','source'):
            def fetch(url,limit):return b'bad' if (url.endswith('.py')==(bad=='source')) else self.fetch(url,limit)
            with self.assertRaises(ValueError):worker.install(self.fd,self.entry,fetch)
            self.assertFalse(worker.installed(self.fd,self.entry))
    def test_symlink_and_local_tampering_refused(self):
        worker.install(self.fd,self.entry,self.fetch)
        path=self.base/'peripheral-hub/plugins/io.test.mouse/plugin.py'
        path.write_bytes(b'changed')
        with self.assertRaisesRegex(ValueError,'INTEGRITY'):worker.installed(self.fd,self.entry)
        path.unlink();path.symlink_to('/etc/passwd')
        with self.assertRaises(OSError):worker.installed(self.fd,self.entry)
    def test_safe_transport_only_and_no_renderer_package_selection(self):
        for url in ('http://github.com/x','https://localhost/x','https://github.com.evil.example/x','https://user:secret@github.com/x','file:///etc/passwd'):
            with self.assertRaises(ValueError):worker.safe_url(url)
        with self.assertRaisesRegex(ValueError,'UNKNOWN_PACKAGE'):
            worker.run(dict(action='install',id='unknown.mouse',base=str(self.base),registry=[self.entry]))
    def test_parent_symlink_refused_and_dev_data_preserved(self):
        (self.base/'redirect').symlink_to(self.base/'peripheral-hub',target_is_directory=True)
        with self.assertRaises(OSError):worker.root(str(self.base/'redirect'))
        dev=self.base/'peripheral-hub/dev-plugins/dev.mouse';dev.mkdir(parents=True)
        (dev/'plugin.py').write_text('personal development source')
        worker.install(self.fd,self.entry,self.fetch)
        self.assertEqual((dev/'plugin.py').read_text(),'personal development source')
