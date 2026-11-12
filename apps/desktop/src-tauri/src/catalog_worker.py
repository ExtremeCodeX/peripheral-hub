"""Metadata only. Embedded by Rust; stdlib, no plugin imports or execution."""
import hashlib
import http.client
import ipaddress
import json
import math
import os
from pathlib import Path
import re
import socket
import ssl
import stat
import sys
import time
from urllib.parse import urlsplit

MAX_BYTES = 2 * 1024 * 1024
TIMEOUT = 12


class CatalogError(Exception):
    pass


def parse(raw, limit=MAX_BYTES):
    if len(raw) > limit:
        raise CatalogError('TOO_LARGE')
    def pairs(items):
        out = {}
        for k, v in items:
            if k in out:
                raise CatalogError('INVALID_JSON')
            out[k] = v
        return out
    def number(text):
        value = float(text)
        if not math.isfinite(value):
            raise CatalogError('INVALID_JSON')
        return value
    try:
        text = raw.decode('utf-8') if isinstance(raw, bytes) else raw
        value = json.loads(text, object_pairs_hook=pairs, parse_float=number,
                           parse_constant=lambda _: (_ for _ in ()).throw(CatalogError('INVALID_JSON')))
        # Reject unpaired surrogate escapes instead of passing invalid Unicode
        # across IPC or persisting an unreadable cache.
        json.dumps(value, ensure_ascii=False).encode('utf-8')
        return value
    except (ValueError, UnicodeError, RecursionError):
        raise CatalogError('INVALID_JSON') from None


def revision(software):
    return hashlib.sha256(json.dumps(software, ensure_ascii=True, sort_keys=True, separators=(',', ':')).encode()).hexdigest()


def validate_index(data, validator, schema):
    try:
        json.dumps(data, ensure_ascii=False).encode('utf-8')
    except (UnicodeError, ValueError, RecursionError):
        raise CatalogError('INVALID_CONTENT') from None
    if not isinstance(data, dict) or set(data) != {'schemaVersion', 'revision', 'software'}:
        raise CatalogError('INVALID_CONTENT')
    if type(data['schemaVersion']) is not int or data['schemaVersion'] != 1:
        raise CatalogError('UNSUPPORTED_VERSION')
    entries = data['software']
    if not isinstance(entries, list) or len(entries) > 1000:
        raise CatalogError('INVALID_CONTENT')
    ids = set()
    for item in entries:
        if not validator.matches(item, schema, schema):
            raise CatalogError('INVALID_CONTENT')
        try:
            validator.check_submission(item, schema)
        except ValueError:
            raise CatalogError('INVALID_CONTENT') from None
        if item['id'] in ids or (item['version'].startswith('git-') and item['version'][4:] != item['source']['commit']):
            raise CatalogError('INVALID_CONTENT')
        ids.add(item['id'])
    if not isinstance(data['revision'], str) or data['revision'] != revision(entries):
        raise CatalogError('INVALID_REVISION')
    if len(json.dumps(data).encode()) > MAX_BYTES:
        raise CatalogError('TOO_LARGE')
    return data


def endpoint(url):
    # Exact build-configured endpoint only; never accept a URL from the UI/index.
    if not isinstance(url, str) or len(url) > 2048 or not re.fullmatch(r'https://[a-z0-9.-]+/[A-Za-z0-9_./-]+', url):
        raise CatalogError('CONFIGURATION')
    p = urlsplit(url)
    if not p.hostname or '.' not in p.hostname or any(x in ('', '.', '..') for x in p.path.split('/')[1:]):
        raise CatalogError('CONFIGURATION')
    try:
        ipaddress.ip_address(p.hostname)
    except ValueError:
        return p
    raise CatalogError('CONFIGURATION')


def valid_etag(value):
    return isinstance(value, str) and len(value) <= 256 and re.fullmatch(r'(?:W/)?"[\x21\x23-\x7e]*"', value) is not None


def public_address(value):
    address = ipaddress.ip_address(value)
    # Exclude IPv6 translation/tunneling ranges as well as non-global routes.
    if address.version == 6 and any(address in ipaddress.ip_network(block) for block in ('::ffff:0:0/96', '64:ff9b::/96', '64:ff9b:1::/48', '2002::/16', '2001::/32')):
        return False
    return address.is_global


def request(url, etag):
    p = endpoint(url)
    try:
        addresses = socket.getaddrinfo(p.hostname, 443, type=socket.SOCK_STREAM)
        if not addresses or any(not public_address(a[4][0]) for a in addresses):
            raise CatalogError('UNSAFE_ADDRESS')
        # Pin checked address, retaining TLS hostname verification. No proxy,
        # DNS re-resolution, redirects, cookies, authorization or device data.
        family, kind, proto, _, address = addresses[0]
        raw = socket.socket(family, kind, proto)
        raw.settimeout(TIMEOUT)
        try:
            raw.connect(address)
            secure = ssl.create_default_context().wrap_socket(raw, server_hostname=p.hostname)
        except BaseException:
            raw.close()
            raise
        conn = http.client.HTTPSConnection(p.hostname, timeout=TIMEOUT)
        conn.sock = secure
        try:
            headers = {'Accept': 'application/json', 'Accept-Encoding': 'identity'}
            if valid_etag(etag):
                headers['If-None-Match'] = etag
            conn.request('GET', p.path, headers=headers)
            response = conn.getresponse()
            if response.status == 304:
                return 304, None, None
            if 300 <= response.status < 400:
                raise CatalogError('REDIRECT')
            if response.status != 200:
                raise CatalogError('HTTP_ERROR')
            if response.getheader('Content-Encoding', 'identity') != 'identity':
                raise CatalogError('CONTENT_TYPE')
            if response.getheader('Content-Type', '').split(';')[0].strip().lower() != 'application/json':
                raise CatalogError('CONTENT_TYPE')
            length = response.getheader('Content-Length')
            if length and (not length.isdecimal() or int(length) > MAX_BYTES):
                raise CatalogError('TOO_LARGE')
            body = response.read(MAX_BYTES + 1)
            if len(body) > MAX_BYTES:
                raise CatalogError('TOO_LARGE')
            tag = response.getheader('ETag')
            return 200, body, tag if valid_etag(tag) else None
        finally:
            conn.close()
    except (socket.timeout, TimeoutError):
        raise CatalogError('TIMEOUT') from None
    except (OSError, ssl.SSLError, http.client.HTTPException):
        raise CatalogError('NETWORK') from None


def safe_cache(path):
    path = Path(path).absolute()
    for p in (*reversed(path.parents), path):
        if p.is_symlink():
            raise CatalogError('CACHE_ERROR')
    if path.exists() and not stat.S_ISREG(path.stat().st_mode):
        raise CatalogError('CACHE_ERROR')
    return path


def sync(url, cache, refresh, validator, schema, fetch=request):
    result = {'index': None, 'status': 'NOT_CONFIGURED', 'cacheWarning': None, 'checkedAt': None}
    if not url:
        return result
    endpoint(url)
    current = None
    try:
        path = safe_cache(cache)
        if path.exists():
            if path.stat().st_size > MAX_BYTES + 4096:
                raise CatalogError('CACHE_ERROR')
            saved = parse(path.read_bytes(), MAX_BYTES + 4096)
            if set(saved) != {'url', 'etag', 'index', 'checkedAt'} or saved['url'] != url:
                raise CatalogError('CACHE_ERROR')
            validate_index(saved['index'], validator, schema)
            if (saved['etag'] is not None and not valid_etag(saved['etag'])) or type(saved['checkedAt']) is not int or saved['checkedAt'] < 0:
                raise CatalogError('CACHE_ERROR')
            if len(json.dumps(saved['index']).encode()) > MAX_BYTES:
                raise CatalogError('CACHE_ERROR')
            current = saved
            result.update(index=saved['index'], status='CACHED', checkedAt=saved['checkedAt'])
    except (OSError, ValueError, KeyError, TypeError, RecursionError, CatalogError):
        result['cacheWarning'] = 'CACHE_ERROR'
    if not refresh:
        return result
    try:
        status, body, etag = fetch(url, current['etag'] if current else None)
        if status == 304:
            if not current:
                raise CatalogError('UNEXPECTED_304')
            result.update(status='NOT_MODIFIED', checkedAt=int(time.time()))
            return result
        index = validate_index(parse(body), validator, schema)
        result.update(index=index, status='UPDATED', checkedAt=int(time.time()))
        saved = {'url': url, 'etag': etag if valid_etag(etag) else None, 'index': index, 'checkedAt': result['checkedAt']}
        import tempfile
        temporary = None
        try:
            path = safe_cache(cache)
            path.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
            safe_cache(path)
            with tempfile.NamedTemporaryFile(dir=path.parent, prefix='.catalog-', delete=False) as stream:
                temporary = stream.name
                stream.write(json.dumps(saved).encode())
                stream.flush()
                os.fsync(stream.fileno())
            os.replace(temporary, path)
        except (OSError, CatalogError):
            result['cacheWarning'] = 'CACHE_WRITE_ERROR'
        finally:
            if temporary and os.path.exists(temporary):
                os.unlink(temporary)
    except CatalogError as error:
        result['status'] = str(error)
    return result


def worker_main(validator):
    try:
        args = json.loads(sys.stdin.buffer.read(MAX_BYTES + 65536))
        result = sync(args['url'], args['cache'], args['refresh'], validator, args['schema'])
    except CatalogError as error:
        result = {'index': None, 'status': str(error), 'cacheWarning': None, 'checkedAt': None}
    except Exception:
        result = {'index': None, 'status': 'WORKER_ERROR', 'cacheWarning': None, 'checkedAt': None}
    print(json.dumps(result))
