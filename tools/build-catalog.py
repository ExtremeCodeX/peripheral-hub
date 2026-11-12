#!/usr/bin/env python3
"""Build reviewed metadata; never fetch or execute integrations."""
import argparse
import importlib.util
import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[1]


def module(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    result = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result


publication = module('publication', ROOT / 'tools/validate-publication.py')
catalog = module('catalog', ROOT / 'apps/desktop/src-tauri/src/catalog_worker.py')


def build(root):
    root = Path(root)
    directory = root / 'catalog/software'
    if directory.is_symlink() or any(p.is_symlink() for p in directory.absolute().parents):
        raise ValueError('symlink catalog forbidden')
    software = []
    for entry in sorted(directory.iterdir()):
        if entry.name == 'README.md':
            continue
        if entry.suffix != '.json':
            raise ValueError('only JSON manifests allowed in catalog/software')
        data = publication.validate(root, entry.relative_to(root).as_posix(), True)
        if entry.stem != data['id']:
            raise ValueError('manifest filename must equal id')
        software.append(data)
    index = {'schemaVersion': 1, 'revision': catalog.revision(software), 'software': software}
    catalog.validate_index(index, publication, publication.load_json(publication.SCHEMA))
    raw = (json.dumps(index, indent=2, ensure_ascii=True) + '\n').encode()
    if len(raw) > catalog.MAX_BYTES:
        raise ValueError('catalog exceeds transport size limit')
    return raw


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, default=ROOT)
    parser.add_argument('--check', action='store_true')
    parser.add_argument('--output', type=Path)
    args = parser.parse_args()
    try:
        raw = build(args.root)
        output = args.output or args.root / 'catalog/v1/index.json'
        if args.check:
            if publication.local_file(args.root, 'catalog/v1/index.json').read_bytes() != raw:
                raise ValueError('index out of date; run python3 tools/build-catalog.py')
        else:
            if output.is_symlink() or any(p.is_symlink() for p in output.absolute().parents):
                raise ValueError('symlink output forbidden')
            output.parent.mkdir(parents=True, exist_ok=True)
            output.write_bytes(raw)
    except (OSError, ValueError, catalog.CatalogError) as error:
        print(f'INVALID: {error}', file=sys.stderr)
        return 1
    return 0


if __name__ == '__main__':
    sys.exit(main())
