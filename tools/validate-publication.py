#!/usr/bin/env python3
"""Validate local metadata only; no network, imports of plugins or execution."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import stat
import struct
import sys
import zlib

ROOT = Path(__file__).resolve().parent.parent
SCHEMA = ROOT / 'schemas/community-publication-v2.schema.json'
KEYWORDS = {'$schema', 'title', 'definitions', '$ref', 'type', 'const', 'enum', 'anyOf', 'required', 'properties', 'additionalProperties', 'items', 'minItems', 'uniqueItems', 'minLength', 'maxLength', 'pattern', 'minimum', 'maximum'}


def load_json(path):
    def pairs(entries):
        result = {}
        for key, value in entries:
            if key in result:
                raise ValueError(f'duplicate JSON key: {key}')
            result[key] = value
        return result
    def reject(value):
        raise ValueError(f'non-finite JSON: {value}')
    with path.open(encoding='utf-8') as stream:
        return json.load(stream, object_pairs_hook=pairs, parse_constant=reject)


def check_schema(schema):
    """Fail closed when the shipped schema exceeds this explicit subset."""
    if set(schema) - KEYWORDS:
        raise ValueError('unsupported schema keyword')
    if '$ref' in schema and not schema['$ref'].startswith('#/definitions/'):
        raise ValueError('only local schema references supported')
    for key in ('properties', 'definitions'):
        for child in schema.get(key, {}).values():
            check_schema(child)
    for child in schema.get('anyOf', []):
        check_schema(child)
    if 'items' in schema:
        check_schema(schema['items'])


def matches(value, schema, root):
    if '$ref' in schema:
        return matches(value, root['definitions'][schema['$ref'].split('/')[-1]], root)
    if 'anyOf' in schema and not any(matches(value, s, root) for s in schema['anyOf']):
        return False
    types = {'null': lambda v: v is None, 'string': lambda v: isinstance(v, str), 'array': lambda v: isinstance(v, list), 'object': lambda v: isinstance(v, dict), 'boolean': lambda v: type(v) is bool,
             'integer': lambda v: type(v) is int or (type(v) is float and v.is_integer())}
    if 'type' in schema and not types[schema['type']](value):
        return False
    def same(a, b):
        if type(a) in (int, float) and type(b) in (int, float):
            return a == b
        return type(a) is type(b) and a == b
    if 'const' in schema and not same(value, schema['const']):
        return False
    if 'enum' in schema and not any(same(value, v) for v in schema['enum']):
        return False
    if type(value) in (int, float):
        if value < schema.get('minimum', -float('inf')) or value > schema.get('maximum', float('inf')):
            return False
    if isinstance(value, str):
        if not schema.get('minLength', 0) <= len(value) <= schema.get('maxLength', float('inf')):
            return False
        if 'pattern' in schema and not re.search(schema['pattern'], value, flags=re.ASCII):
            return False
    if isinstance(value, list):
        if len(value) < schema.get('minItems', 0):
            return False
        if schema.get('uniqueItems') and len({json.dumps(v, sort_keys=True) for v in value}) != len(value):
            return False
        if 'items' in schema and not all(matches(v, schema['items'], root) for v in value):
            return False
    if isinstance(value, dict):
        if not set(schema.get('required', [])).issubset(value):
            return False
        props = schema.get('properties', {})
        if schema.get('additionalProperties') is False and set(value) - set(props):
            return False
        if not all(matches(v, props[k], root) for k, v in value.items() if k in props):
            return False
    return True


def local_file(root, relative):
    root = Path(root).absolute()
    for ancestor in (*reversed(root.parents), root):
        if ancestor.is_symlink():
            raise ValueError('symlink root/ancestor forbidden')
    if not re.fullmatch(r'[A-Za-z0-9_-][A-Za-z0-9_.-]*(?:/[A-Za-z0-9_-][A-Za-z0-9_.-]*)*', relative, flags=re.ASCII):
        raise ValueError(f'unsafe path: {relative}')
    current = root
    for part in relative.split('/'):
        current = current / part
        if current.is_symlink():
            raise ValueError(f'symlink forbidden: {relative}')
    if not stat.S_ISREG(current.stat().st_mode):
        raise ValueError(f'not a regular file: {relative}')
    return current


def reference_file(root, reference):
    # The schema has already checked every reference: never fetch HTTPS.
    if reference and not reference.startswith('https://'):
        local_file(root, reference)


def image_metadata(path):
    """Inspect raster framing/metadata, not a decoder or security scanner.

    No project-defined pixel/file-size cap. PNG CRCs/framing and JPEG segments
    are checked, but compressed pixels are never decompressed/rendered.
    """
    raw = path.read_bytes()
    if raw.startswith(b'\x89PNG\r\n\x1a\n'):
        offset, dimensions, ended, has_data = 8, None, False, False
        while offset < len(raw):
            if offset + 12 > len(raw):
                raise ValueError('truncated PNG chunk')
            length = int.from_bytes(raw[offset:offset + 4], 'big')
            kind = raw[offset + 4:offset + 8]
            end = offset + 12 + length
            if end > len(raw):
                raise ValueError('truncated PNG payload')
            payload = raw[offset + 8:end - 4]
            if zlib.crc32(kind + payload) != int.from_bytes(raw[end - 4:end], 'big'):
                raise ValueError('PNG CRC mismatch')
            if dimensions is None and kind != b'IHDR':
                raise ValueError('PNG must start with IHDR')
            if kind == b'IHDR':
                if dimensions is not None or length != 13:
                    raise ValueError('invalid PNG IHDR')
                width, height, depth, color, compression, filtering, interlace = struct.unpack('>IIBBBBB', payload)
                valid_depths = {0: (1, 2, 4, 8, 16), 2: (8, 16), 3: (1, 2, 4, 8), 4: (8, 16), 6: (8, 16)}
                if not (0 < width < 2**31 and 0 < height < 2**31) or depth not in valid_depths.get(color, ()) or compression != 0 or filtering != 0 or interlace not in (0, 1):
                    raise ValueError('invalid PNG dimensions/header')
                dimensions = (width, height)
            if kind == b'IDAT':
                has_data = True
            if kind == b'IEND':
                if length or end != len(raw):
                    raise ValueError('invalid PNG ending')
                ended = True
                break
            offset = end
        if not ended or not has_data:
            raise ValueError('incomplete PNG')
        return 'image/png', *dimensions, len(raw)
    if raw.startswith(b'\xff\xd8') and raw.endswith(b'\xff\xd9'):
        offset, dimensions = 2, None
        while offset < len(raw) - 2:
            if raw[offset] != 255:
                raise ValueError('invalid JPEG marker')
            while offset < len(raw) and raw[offset] == 255:
                offset += 1
            if offset >= len(raw):
                break
            marker = raw[offset]
            offset += 1
            if marker == 0xDA:  # SOS: do not decode the entropy-coded pixels.
                if offset + 2 > len(raw) - 2:
                    break
                length = int.from_bytes(raw[offset:offset + 2], 'big')
                if dimensions and length >= 2 and offset + length <= len(raw) - 2:
                    return 'image/jpeg', *dimensions, len(raw)
                break
            if marker == 0x01 or 0xD0 <= marker <= 0xD7:
                continue
            if offset + 2 > len(raw) - 2:
                break
            length = int.from_bytes(raw[offset:offset + 2], 'big')
            end = offset + length
            if length < 2 or end > len(raw) - 2:
                break
            if marker in (0xC0, 0xC1, 0xC2, 0xC3, 0xC5, 0xC6, 0xC7, 0xC9, 0xCA, 0xCB, 0xCD, 0xCE, 0xCF):
                if length < 8:
                    break
                height, width = struct.unpack('>HH', raw[offset + 3:offset + 7])
                if not width or not height:
                    break
                dimensions = (width, height)
            offset = end
        raise ValueError('invalid/incomplete JPEG metadata')
    raise ValueError('unsupported image signature (PNG/JPEG required by this validator)')


def declared_images(data):
    images = data.get('images') or {}
    return [images[key] for key in ('main', 'thumbnail') if images.get(key)] + images.get('screenshots', [])


def check_submission(data, schema):
    # Apply this overlay only AFTER the closed publication schema. Keeping the
    # policy machine-readable avoids a second, divergent set of field rules.
    if not matches(data, schema['definitions']['submission'], schema):
        raise ValueError('submission incomplete: metadata, documentation, image rights or pending (see definitions/submission)')


def validate(root, manifest, submission=False):
    schema = load_json(SCHEMA)
    check_schema(schema)
    data = load_json(local_file(root, manifest))
    if not matches(data, schema, schema):
        raise ValueError('manifest does not match community-publication-v2.schema.json')
    for document in data['documents']:
        local_file(root, document)
    docs = data.get('documentation') or {}
    for reference in (docs.get('readme'), docs.get('changelog'), *docs.get('guides', [])):
        reference_file(root, reference)
    for device in data['devices']:
        for usb in device.get('usb') or []:
            reference_file(root, usb['provenance'])
    if data['version'] and data['version'].startswith('git-'):
        if not data['source'] or data['version'][4:] != data['source']['commit']:
            raise ValueError('git version must equal source.commit')
    for image in declared_images(data):
        path = local_file(root, image['path'])
        actual = image_metadata(path)
        expected_extensions = {'image/png': ('.png',), 'image/jpeg': ('.jpg', '.jpeg')}
        if image['mediaType'] != actual[0] or path.suffix.lower() not in expected_extensions[actual[0]]:
            raise ValueError(f'image format/extension mismatch: {image["path"]}')
        for key, value in zip(('width', 'height', 'byteLength'), actual[1:]):
            if image.get(key) is not None and image[key] != value:
                raise ValueError(f'image {key} mismatch: {image["path"]}')
        for reference in (image.get('rights') or {}).get('evidence', []):
            reference_file(root, reference)
    for artifact in data['artifacts']:
        with local_file(root, artifact['path']).open('rb') as stream:
            digest = hashlib.file_digest(stream, 'sha256').hexdigest()
        if digest != artifact['sha256']:
            raise ValueError(f'hash mismatch: {artifact["path"]}')
    if submission:
        check_submission(data, schema)
    return data


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('manifest', help='relative to --root')
    parser.add_argument('--root', default=str(ROOT))
    parser.add_argument('--submission', action='store_true', help='completeness gate, never approval')
    args = parser.parse_args()
    try:
        validate(args.root, args.manifest, args.submission)
    except (OSError, ValueError, KeyError, RecursionError) as error:
        print(f'INVALID: {error}', file=sys.stderr)
        return 1
    print('VALID local metadata only — not approval, installation or physical verification')
    return 0


if __name__ == '__main__':
    sys.exit(main())
