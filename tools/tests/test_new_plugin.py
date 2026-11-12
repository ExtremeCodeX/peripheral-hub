import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

spec = importlib.util.spec_from_file_location('new_plugin', Path(__file__).parents[1] / 'new-plugin.py')
generator = importlib.util.module_from_spec(spec)
spec.loader.exec_module(generator)


class GeneratorTests(unittest.TestCase):
    def test_independent_source_tests_license_and_incomplete_submission(self):
        with tempfile.TemporaryDirectory() as tmp:
            destination = generator.create(Path(tmp) / 'mouse', 'dev.another-mouse')
            self.assertIn("PLUGIN_ID = 'dev.another-mouse'", (destination / 'plugin.py').read_text())
            self.assertNotIn('dev.starter', (destination / 'tests/test_plugin.py').read_text())
            self.assertIn('Apache License', (destination / 'LICENSE').read_text())
            self.assertTrue(json.loads((destination / 'publication.json').read_text())['pending'])
            with self.assertRaises(ValueError):
                generator.create(destination, 'dev.another-mouse')

    def test_invalid_identity_never_creates_destination(self):
        with tempfile.TemporaryDirectory() as tmp:
            destination = Path(tmp) / 'mouse'
            for identity in ['dev.fake', 'dev.starter', '../mouse', 'dev.x/y', 'dev.x..y', 'dev.x.']:
                with self.subTest(identity=identity), self.assertRaises(ValueError):
                    generator.create(destination, identity)
            self.assertFalse(destination.exists())
