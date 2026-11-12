import importlib.util
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

SPEC = importlib.util.spec_from_file_location('local_install', Path(__file__).parents[1] / 'install-local.py')
installer = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(installer)


class LocalInstallTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(dir='/tmp/opencode' if Path('/tmp/opencode').is_dir() else None)
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.source = self.root / 'source'
        self.source.mkdir(mode=0o700)
        (self.source / 'plugin.py').write_text("print('not executed')\n")
        self.data = self.root / 'data'

    def test_starter_snapshot_is_local_editable_and_no_execution(self):
        target = installer.install('starter', self.source, self.data)
        self.assertEqual((target / 'plugin.py').read_bytes(), (self.source / 'plugin.py').read_bytes())
        (target / 'plugin.py').write_text('# edited without rebuilding\n')
        self.assertIn('edited', (target / 'plugin.py').read_text())
        self.assertEqual(sorted(p.name for p in target.iterdir()), ['LOCAL_ONLY.json', 'plugin.py'])

    def test_no_overwrite_and_existing_install_preserved(self):
        target = installer.install('starter', self.source, self.data)
        (self.source / 'plugin.py').write_text('changed')
        with self.assertRaises(ValueError):
            installer.install('starter', self.source, self.data)
        self.assertEqual((target / 'plugin.py').read_text(), "print('not executed')\n")

    def test_r1_incomplete_and_wrong_pin_never_installs(self):
        with self.assertRaises(OSError):
            installer.install('r1', self.source, self.data)
        (self.source / 'r1_adapter.py').write_text('bad')
        with self.assertRaisesRegex(ValueError, 'integrity'):
            installer.install('r1', self.source, self.data)
        self.assertFalse(self.data.exists())

    def test_url_unc_relative_traversal_refused(self):
        for path in ['https://example.org/plugin', 'file:///tmp/plugin', '//server/plugin', 'relative', '/tmp/../etc']:
            with self.subTest(path=path), self.assertRaises(ValueError):
                installer.local_path(path)

    def test_known_remote_and_fuse_mounts_are_refused_without_connecting(self):
        for kind in ('nfs4', 'cifs', 'fuse.sshfs', 'fuse.gvfsd-fuse', '9p', 'ceph'):
            table = f'1 0 0:1 / / rw - ext4 local rw\n2 1 0:2 / /mounted rw - {kind} remote rw\n'
            with patch.object(Path, 'read_text', return_value=table):
                with self.subTest(kind=kind), self.assertRaisesRegex(ValueError, 'remote/FUSE'):
                    installer.local_path('/mounted/plugin.py')

    def test_symlink_leaf_and_ancestor_refused(self):
        original = self.source / 'original.py'
        (self.source / 'plugin.py').rename(original)
        (self.source / 'plugin.py').symlink_to(original)
        with self.assertRaises(OSError):
            installer.install('starter', self.source, self.data)
        link = self.root / 'link'
        link.symlink_to(self.source, target_is_directory=True)
        with self.assertRaises(OSError):
            installer.install('starter', link, self.data)

    def test_writable_source_refused(self):
        (self.source / 'plugin.py').chmod(0o666)
        with self.assertRaisesRegex(ValueError, 'permissions'):
            installer.install('starter', self.source, self.data)

    def test_oversize_source_refused(self):
        (self.source / 'plugin.py').write_bytes(b'#' * 65537)
        with self.assertRaisesRegex(ValueError, 'large'):
            installer.install('starter', self.source, self.data)

    def test_destination_symlink_refused(self):
        self.data.symlink_to(self.source, target_is_directory=True)
        with self.assertRaises(OSError):
            installer.install('starter', self.source, self.data)

    def test_r1_is_not_loaded_by_apache_core(self):
        runtime = (Path(__file__).parents[2] / 'apps/desktop/src-tauri/src/local_runtime.rs').read_text()
        for digest in installer.PINS.values():
            self.assertNotIn(digest, runtime)

    def test_named_registration_and_unsafe_id_rejected(self):
        target = installer.install('starter', self.source, self.data, 'dev.other-mouse')
        self.assertEqual(target, self.data / 'peripheral-hub/dev-plugins/dev.other-mouse')
        for identity in ('../../bin/sh', 'dev../x', 'dev.foo/bar', 'dev.fake', 'dev.starter', 'dev.foo.', 'https://example.org'):
            with self.subTest(identity=identity), self.assertRaises(ValueError):
                installer.install('starter', self.source, self.data, identity)


if __name__ == '__main__':
    unittest.main()
