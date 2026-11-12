import copy
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import tempfile
import unittest
import struct
import zlib
from datetime import date

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('publication', ROOT / 'tools/validate-publication.py')
publication = importlib.util.module_from_spec(spec)
spec.loader.exec_module(publication)
EXAMPLE = 'tools/tests/fixtures/publication-base.json'
FIXTURES = ROOT / 'tools/tests/fixtures'


class PublicationTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='publication-', dir='/tmp/opencode' if Path('/tmp/opencode').is_dir() else None)
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        (self.root / 'doc.md').write_text('Synthetic fixture, not physical evidence.\n')
        payload = b'INERT synthetic fixture, not a plugin\n'
        (self.root / 'artifact.txt').write_bytes(payload)
        self.data = publication.load_json(ROOT / EXAMPLE)
        self.data.update(id='test.synthetic', name='Synthetic validation fixture only',
                         version='0.0.0-fixture.1', authors=['Synthetic test fixture, not a person'],
                         source={'repository': 'https://example.org/fixture', 'commit': 'a' * 40},
                         license='Synthetic fixture only, no license assertion',
                         documents=['doc.md'], videos=['https://example.org/video'],
                         artifacts=[{'path': 'artifact.txt', 'sha256': hashlib.sha256(payload).hexdigest(), 'target': 'Linux synthetic fixture only'}], pending=[],
                         minimumHubVersion=None, architectures=['fixture-only'],
                         documentation={'readme': 'doc.md', 'changelog': None, 'homepage': None, 'guides': []},
                         images=None)
        self.schema = publication.load_json(publication.SCHEMA)

    def save(self):
        (self.root / 'publication.json').write_text(json.dumps(self.data), encoding='utf-8')

    def test_valid_incomplete_synthetic_metadata_not_submission(self):
        publication.validate(ROOT, EXAMPLE)
        with self.assertRaises(ValueError):
            publication.validate(ROOT, EXAMPLE, True)

    def test_valid_synthetic_complete_gate_not_approval(self):
        self.save()
        publication.validate(self.root, 'publication.json', True)

    def test_schema_keyword_subset_and_mutations(self):
        publication.check_schema(self.schema)
        with self.assertRaises(ValueError):
            publication.check_schema({'unsupported': True})
        with self.assertRaises(ValueError):
            publication.check_schema({'$ref': 'https://example.org/schema'})
        for case in publication.load_json(FIXTURES / 'invalid-cases.json'):
            with self.subTest(case=case):
                data = copy.deepcopy(self.data)
                target = data
                for part in case['path'][:-1]:
                    target = target[part]
                target[case['path'][-1]] = case['value']
                self.assertFalse(publication.matches(data, self.schema, self.schema))

    def test_required_fields_and_nested_unknowns(self):
        for key in self.schema['required']:
            data = copy.deepcopy(self.data)
            del data[key]
            with self.subTest(key=key):
                self.assertFalse(publication.matches(data, self.schema, self.schema))
        for filename in ('invalid-empty.json', 'invalid-execution.json'):
            self.assertFalse(publication.matches(publication.load_json(FIXTURES / filename), self.schema, self.schema))

    def test_json_numeric_const_semantics(self):
        self.data['schemaVersion'] = 2.0
        self.assertTrue(publication.matches(self.data, self.schema, self.schema))

    def test_types_limits_duplicate_items_and_trailing_controls(self):
        for key in self.data:
            data = copy.deepcopy(self.data)
            data[key] = False
            with self.subTest(key=key):
                self.assertFalse(publication.matches(data, self.schema, self.schema))
        for key, value in [('name', ''), ('name', 'bad\n'),
                           ('id', 'test.id\n'), ('documents', ['doc.md\n']),
                           ('videos', ['https://example.org/video\n']),
                           ('authors', ['duplicate', 'duplicate'])]:
            data = copy.deepcopy(self.data)
            data[key] = value
            with self.subTest(key=key, value=str(value)[:40]):
                self.assertFalse(publication.matches(data, self.schema, self.schema))

    def test_special_file_rejected_without_opening(self):
        os.mkfifo(self.root / 'pipe')
        with self.assertRaises(ValueError):
            publication.local_file(self.root, 'pipe')

    def test_duplicates_nonfinite_malformed(self):
        for text in ('{"x": 1, "x": 2}', '{"x": NaN}', '{"x": Infinity}', '{broken'):
            (self.root / 'bad.json').write_text(text)
            with self.subTest(text=text), self.assertRaises(ValueError):
                publication.load_json(self.root / 'bad.json')

    def test_missing_directory_and_hash(self):
        for path in ('missing.md', 'directory'):
            (self.root / 'directory').mkdir(exist_ok=True)
            self.data['documents'] = [path]
            self.save()
            with self.subTest(path=path), self.assertRaises((OSError, ValueError)):
                publication.validate(self.root, 'publication.json')
        self.data['documents'] = ['doc.md']
        self.data['artifacts'][0]['sha256'] = '0' * 64
        self.save()
        with self.assertRaises(ValueError):
            publication.validate(self.root, 'publication.json')

    def test_symlink_leaf_parent_manifest_root_and_ancestor(self):
        (self.root / 'leaf').symlink_to(self.root / 'doc.md')
        (self.root / 'parent').symlink_to(self.root, target_is_directory=True)
        for path in ('leaf', 'parent/doc.md'):
            with self.subTest(path=path), self.assertRaises(ValueError):
                publication.local_file(self.root, path)
        self.save()
        (self.root / 'manifest-link.json').symlink_to(self.root / 'publication.json')
        with self.assertRaises(ValueError):
            publication.validate(self.root, 'manifest-link.json')
        with self.assertRaises(ValueError):
            publication.validate(self.root / 'parent', 'publication.json')
        (self.root / 'child').mkdir()
        with self.assertRaises(ValueError):
            publication.local_file(self.root / 'parent/child', 'anything')
        self.data['artifacts'][0]['path'] = 'leaf'
        self.save()
        with self.assertRaises(ValueError):
            publication.validate(self.root, 'publication.json')

    def test_submission_completeness(self):
        for key in ('version', 'authors', 'source', 'license', 'videos', 'pending'):
            original = copy.deepcopy(self.data)
            self.data[key] = ['unresolved'] if key == 'pending' else (None if key in ('version', 'source', 'license') else [])
            self.save()
            with self.subTest(key=key), self.assertRaises(ValueError):
                publication.validate(self.root, 'publication.json', True)
            self.data = original

    def test_missing_information_requires_explicit_pending(self):
        data = publication.load_json(ROOT / EXAMPLE)
        data['pending'] = []
        self.assertFalse(publication.matches(data, self.schema, self.schema))
        data['pending'] = ['Missing information is explicitly unresolved, not approved.']
        self.assertTrue(publication.matches(data, self.schema, self.schema))

    def test_object_schemas_are_closed(self):
        def visit(node):
            if isinstance(node, dict):
                if node.get('type') == 'object' and 'properties' in node:
                    self.assertIs(node.get('additionalProperties'), False)
                for value in node.values():
                    visit(value)
            elif isinstance(node, list):
                for value in node:
                    visit(value)
        schema = copy.deepcopy(self.schema)
        # Policy overlays intentionally constrain only a subset; the full
        # publication schema remains closed and is always applied first.
        for key in ('submission', 'submissionImage'):
            del schema['definitions'][key]
        visit(schema)

    def test_submission_policy_schema_and_validator_agree(self):
        cases = [copy.deepcopy(self.data)]
        for key in ('shortDescription', 'fullDescription', 'architectures', 'documentation'):
            data = copy.deepcopy(self.data)
            data[key] = None
            cases.append(data)
        self.with_image()
        cases.append(copy.deepcopy(self.data))
        for rights in (None, {'status': 'unknown'}, {'status': 'permission-confirmed'},
                       {'status': 'open-license', 'statement': 'Fixture only', 'evidence': ['doc.md']}):
            data = copy.deepcopy(self.data)
            data['images']['thumbnail']['rights'] = rights
            cases.append(data)
        for data in cases:
            expected = publication.matches(data, self.schema, self.schema) and publication.matches(data, self.schema['definitions']['submission'], self.schema)
            self.data = data
            self.save()
            try:
                publication.validate(self.root, 'publication.json', True)
                actual = True
            except ValueError:
                actual = False
            self.assertEqual(actual, expected)

    def png(self, width=1, height=1):
        def chunk(kind, payload):
            return struct.pack('>I', len(payload)) + kind + payload + struct.pack('>I', zlib.crc32(kind + payload))
        return (b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', struct.pack('>IIBBBBB', width, height, 8, 2, 0, 0, 0))
                + chunk(b'IDAT', zlib.compress(b'\x00\x00\x00\x00')) + chunk(b'IEND', b''))

    def with_image(self):
        payload = self.png()
        (self.root / 'image.png').write_bytes(payload)
        image = {'path': 'image.png', 'mediaType': 'image/png', 'alt': 'Synthetic pixel fixture only',
                 'width': 1, 'height': 1, 'byteLength': len(payload),
                 'rights': {'status': 'permission-confirmed', 'holder': None, 'license': None,
                            'statement': 'Synthetic fixture evidence, not a real rights assertion.', 'evidence': ['doc.md']}}
        self.data['images'] = {'thumbnail': image, 'screenshots': [copy.deepcopy(image)]}
        return image

    def test_source_only_and_optional_unknown_metadata(self):
        self.data['artifacts'] = []
        self.save()
        publication.validate(self.root, 'publication.json', True)
        for key in ('shortDescription', 'fullDescription', 'minimumHubVersion', 'architectures', 'distributions', 'releaseDate', 'documentation', 'images'):
            data = copy.deepcopy(self.data)
            del data[key]
            self.assertTrue(publication.matches(data, self.schema, self.schema), key)
            data[key] = None
            self.assertTrue(publication.matches(data, self.schema, self.schema), key)
        self.data['source'] = None
        self.data['pending'] = ['Missing source, not an approved source-only submission.']
        self.save()
        with self.assertRaises(ValueError):
            publication.validate(self.root, 'publication.json', True)

    def test_submission_accepts_undefined_minimum_hub_and_changelog(self):
        self.data['artifacts'] = []
        for present in (True, False):
            data = copy.deepcopy(self.data)
            if present:
                data['minimumHubVersion'] = None
                data['documentation']['changelog'] = None
            else:
                data.pop('minimumHubVersion', None)
                data['documentation'].pop('changelog', None)
            self.data = data
            self.save()
            self.assertTrue(publication.matches(data, self.schema['definitions']['submission'], self.schema))
            publication.validate(self.root, 'publication.json', True)
        self.data['minimumHubVersion'] = '0.1.0-alpha.1'
        self.data['documentation']['changelog'] = 'doc.md'
        self.save()
        publication.validate(self.root, 'publication.json', True)
        self.data['minimumHubVersion'] = 'not-defined'
        self.save()
        with self.assertRaises(ValueError):
            publication.validate(self.root, 'publication.json', True)

    def test_main_image_is_optional_and_independently_validated(self):
        thumbnail = self.with_image()
        original = copy.deepcopy(self.data)
        for value in (None, copy.deepcopy(thumbnail)):
            self.data['images']['main'] = value
            self.save()
            publication.validate(self.root, 'publication.json', True)
        # Main-only works; thumbnail and screenshots are not prerequisites.
        self.data['images'] = {'main': copy.deepcopy(thumbnail)}
        self.save()
        publication.validate(self.root, 'publication.json', True)
        for key, value in (('path', '../escape.png'), ('path', 'missing.png'),
                           ('width', 2), ('height', 2), ('byteLength', 1),
                           ('mediaType', 'image/jpeg'), ('rights', None), ('alt', None)):
            self.data = copy.deepcopy(original)
            self.data['images']['main'] = copy.deepcopy(thumbnail)
            self.data['images']['main'][key] = value
            self.save()
            with self.subTest(key=key), self.assertRaises((OSError, ValueError)):
                publication.validate(self.root, 'publication.json', True)
        self.data['images']['main'] = copy.deepcopy(thumbnail)
        self.data['images']['main']['rights']['evidence'] = ['missing.md']
        self.save()
        with self.assertRaises(OSError):
            publication.validate(self.root, 'publication.json')
        (self.root / 'main-link.png').symlink_to(self.root / 'image.png')
        self.data['images']['main'] = copy.deepcopy(thumbnail)
        self.data['images']['main']['path'] = 'main-link.png'
        self.save()
        with self.assertRaises(ValueError):
            publication.validate(self.root, 'publication.json')

    def test_versions_dates_and_git_identity(self):
        for version in ('0.0.0', '12.34.56', '1.0.0-alpha.1+build.001', 'git-' + 'a' * 40, 'git-' + 'a' * 64):
            data = copy.deepcopy(self.data)
            data['version'] = version
            self.assertTrue(publication.matches(data, self.schema, self.schema), version)
        for version in ('main', 'HEAD', 'v1.0.0', '1', '01.0.0', '1.0.0-01', '1.0.0+', 'git-1234', '1.0.0\n'):
            data = copy.deepcopy(self.data)
            data['version'] = version
            self.assertFalse(publication.matches(data, self.schema, self.schema), version)
        self.data['version'] = 'git-' + self.data['source']['commit']
        self.data['releaseDate'] = None
        self.save()
        publication.validate(self.root, 'publication.json', True)
        self.data['version'] = 'git-' + 'b' * 40
        self.save()
        with self.assertRaises(ValueError):
            publication.validate(self.root, 'publication.json')
        rule = self.schema['definitions']['date']
        for year in (1, 4, 100, 400, 1600, 1700, 1900, 2000, 2024, 2100, 2400, 9999):
            for month in range(1, 13):
                for day in range(0, 33):
                    value = f'{year:04}-{month:02}-{day:02}'
                    try:
                        date(year, month, day)
                        expected = True
                    except ValueError:
                        expected = False
                    self.assertEqual(publication.matches(value, rule, self.schema), expected, value)
        for value in ('0000-01-01', '2024-13-01', '2024-2-1', '2024-01-01T00:00:00Z', 'tomorrow', '2024-02-29\n'):
            self.assertFalse(publication.matches(value, rule, self.schema), value)

    def test_metadata_types_and_usb_provenance(self):
        self.data['devices'][0]['usb'] = [{'vid': 'abcd', 'pid': '0123', 'interface': 255, 'usagePage': None, 'usage': '0001', 'provenance': 'doc.md'}]
        self.data['distributions'] = [{'id': 'synthetic-fixture', 'version': None, 'tested': False}]
        self.save()
        publication.validate(self.root, 'publication.json')
        for key, value in [('vid', '0xabcd'), ('pid', 1234), ('interface', -1), ('interface', 256), ('interface', True), ('interface', 1.5), ('usage', 'xyz1'), ('provenance', '../escape')]:
            data = copy.deepcopy(self.data)
            data['devices'][0]['usb'][0][key] = value
            self.assertFalse(publication.matches(data, self.schema, self.schema), (key, value))
        self.data['devices'][0]['usb'][0]['provenance'] = 'missing.md'
        self.save()
        with self.assertRaises(OSError):
            publication.validate(self.root, 'publication.json')

    def test_image_metadata_sizes_and_framing(self):
        self.with_image()
        self.save()
        publication.validate(self.root, 'publication.json', True)
        for key, value in [('width', 2), ('height', 2), ('byteLength', 1), ('mediaType', 'image/jpeg')]:
            original = copy.deepcopy(self.data)
            self.data['images']['thumbnail'][key] = value
            self.save()
            with self.subTest(key=key), self.assertRaises(ValueError):
                publication.validate(self.root, 'publication.json')
            self.data = original
        for payload in (b'not an image', b'<svg/>', self.png()[:-1], self.png()[:-8] + b'corrupt!', self.png(width=0)):
            (self.root / 'image.png').write_bytes(payload)
            with self.subTest(payload=payload[:10]), self.assertRaises(ValueError):
                publication.image_metadata(self.root / 'image.png')
        # Header-only inspection has no product-defined pixel limit; no decode.
        (self.root / 'large.png').write_bytes(self.png(width=100000, height=100000))
        self.assertEqual(publication.image_metadata(self.root / 'large.png')[1:3], (100000, 100000))
        jpeg = (b'\xff\xd8\xff\xc0\x00\x0b\x08\x00\x01\x00\x02\x01\x01\x11\x00'
                b'\xff\xda\x00\x08\x01\x01\x00\x00\x3f\x00\x00\xff\xd9')
        (self.root / 'metadata.jpg').write_bytes(jpeg)
        self.assertEqual(publication.image_metadata(self.root / 'metadata.jpg'), ('image/jpeg', 2, 1, len(jpeg)))

    def test_image_and_document_reference_safety(self):
        image = self.with_image()
        for path in ('../image.png', '/etc/passwd', 'nested/../image.png', 'image.png\n'):
            image['path'] = path
            self.save()
            with self.subTest(path=path), self.assertRaises(ValueError):
                publication.validate(self.root, 'publication.json')
        image['path'] = 'link.png'
        (self.root / 'link.png').symlink_to(self.root / 'image.png')
        self.save()
        with self.assertRaises(ValueError):
            publication.validate(self.root, 'publication.json')
        image['path'] = 'image.png'
        (self.root / 'wrong.jpg').write_bytes(self.png())
        image['path'] = 'wrong.jpg'
        self.save()
        with self.assertRaises(ValueError):
            publication.validate(self.root, 'publication.json')
        image['path'] = 'image.png'
        for container, key in ((self.data['documentation'], 'readme'), (self.data['documentation'], 'changelog')):
            container[key] = 'https://example.org/docs'
            self.save()
            publication.validate(self.root, 'publication.json')
            container[key] = 'missing.md'
            self.save()
            with self.assertRaises(OSError):
                publication.validate(self.root, 'publication.json')
            container[key] = 'doc.md'
        image['rights']['evidence'] = ['link.png']
        self.save()
        with self.assertRaises(ValueError):
            publication.validate(self.root, 'publication.json')

    def test_root_relative_references_not_manifest_directory(self):
        self.with_image()
        self.save()
        (self.root / 'nested').mkdir()
        (self.root / 'nested/publication.json').write_text(json.dumps(self.data))
        publication.validate(self.root, 'nested/publication.json', True)
        with self.assertRaises(OSError):
            publication.validate(self.root / 'nested', 'publication.json')

    def test_screenshot_rights_are_not_inherited_from_thumbnail(self):
        self.with_image()
        self.data['images']['screenshots'][0]['rights'] = None
        self.save()
        publication.validate(self.root, 'publication.json')
        with self.assertRaises(ValueError):
            publication.validate(self.root, 'publication.json', True)

    def test_image_and_required_metadata_submission_policy(self):
        image = self.with_image()
        for key in ('shortDescription', 'fullDescription', 'architectures', 'documentation'):
            original = copy.deepcopy(self.data)
            self.data[key] = None
            self.save()
            with self.subTest(key=key), self.assertRaises(ValueError):
                publication.validate(self.root, 'publication.json', True)
            self.data = original
        for key in ('manufacturer', 'category', 'transports'):
            original = copy.deepcopy(self.data)
            self.data['devices'][0][key] = None
            self.save()
            with self.subTest(key=key), self.assertRaises(ValueError):
                publication.validate(self.root, 'publication.json', True)
            self.data = original
        for rights in (None, {'status': 'unknown'}, {'status': 'permission-confirmed'},
                       {'status': 'open-license', 'statement': 'Fixture only', 'evidence': ['doc.md']}):
            self.data['images']['thumbnail']['rights'] = rights
            self.save()
            publication.validate(self.root, 'publication.json')
            with self.subTest(rights=rights), self.assertRaises(ValueError):
                publication.validate(self.root, 'publication.json', True)
        self.data['images']['thumbnail']['rights'] = {'status': 'open-license', 'license': 'Synthetic terms fixture', 'statement': 'Fixture only', 'evidence': ['doc.md']}
        self.save()
        publication.validate(self.root, 'publication.json', True)

    def test_no_arbitrary_description_or_image_size_caps(self):
        self.data['fullDescription'] = 'Factual description fixture ' * 10000 + 'end'
        self.assertTrue(publication.matches(self.data, self.schema, self.schema))
        image = self.with_image()
        for key in ('width', 'height', 'byteLength'):
            for value in (0, -1, True, 1.5):
                data = copy.deepcopy(self.data)
                data['images']['thumbnail'][key] = value
                self.assertFalse(publication.matches(data, self.schema, self.schema), (key, value))
        image['width'] = 1.0
        self.assertTrue(publication.matches(self.data, self.schema, self.schema))

    def test_local_document_links(self):
        files = [*ROOT.glob('community/integration-starter/**/*.md'), *ROOT.glob('docs/public/*.md')]
        files += [ROOT / name for name in ('.github/PULL_REQUEST_TEMPLATE.md', 'CONTRIBUTING.md', 'README.md', 'THIRD_PARTY_NOTICES.md') if (ROOT / name).exists()]
        checked = 0
        for file in files:
            for target in re.findall(r'\[[^\]]+\]\(([^)]+)\)', file.read_text()):
                if target.startswith(('https://', 'http://', '#')):
                    continue
                self.assertNotIn('#', target, 'fragment checking not implemented')
                resolved = (file.parent / target).resolve()
                self.assertTrue(resolved.is_relative_to(ROOT), f'outside workspace: {file}: {target}')
                self.assertTrue(resolved.exists(), f'broken link {file}: {target}')
                checked += 1
        self.assertGreater(checked, 0)


if __name__ == '__main__':
    unittest.main()
