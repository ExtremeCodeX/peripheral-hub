#!/usr/bin/python3
"""Explicit local-only preparation. Never downloads, imports or executes a plugin.

Private R1 snapshot != redistributable artifact or catalog approval.
"""
import argparse
import hashlib
import json
import os
import re
from pathlib import Path
import shutil
import stat
import tempfile

PINS = {
    'r1_adapter.py': 'b7b772afb9ce8c111d0159bf98c9242cff89f130e75b15648209e60b81a052b4',
    'r1_hardware.py': '4c3e646915d02ac950ea2e27f894d29778bff3dbddcbcd072ef8c2f1abde2429',
    'r1control/protocol.py': '86055dc6f8fe859ae15cd868f5fe42e4add14419092bc9a8eab3f670f012f694',
    'r1control/service.py': '7e9eac6722eaece6497747079eb9e64b8e887e43359dbdf4cba31bc02a0b794c',
    'r1control/transport.py': '9784f3bfdea3ca9ce6db8aa693f558ada04203670d64f95eb0e20288b3a2dc5c',
    'r1control/storage.py': '2ad9da4c8ba6f7f1b534d07b35b4d9425b8c32faeaa79147ad57364c53858e51',
}


def local_path(value):
    text = str(value)
    if not text.startswith('/') or text.startswith('//') or '://' in text or '..' in Path(text).parts:
        raise ValueError('absolute local path required; URLs/UNC/traversal refused')
    # Linux mount table: conservative rejection of network and FUSE sources.
    mounts = []
    for line in Path('/proc/self/mountinfo').read_text().splitlines():
        left, right = line.split(' - ', 1)
        mount = left.split()[4].replace('\\040', ' ').replace('\\134', '\\')
        if text == mount or text.startswith(mount.rstrip('/') + '/'):
            mounts.append((len(mount), right.split()[0]))
    kind = max(mounts)[1]
    if kind.startswith(('nfs', 'cifs', 'smb', 'fuse', '9p', 'afs', 'ceph')):
        raise ValueError('remote/FUSE filesystem refused')
    return Path(text)


def trusted(fd):
    info = os.fstat(fd)
    if info.st_uid not in (0, os.geteuid()) or info.st_mode & 0o022:
        raise ValueError('unsafe owner/permissions')


def directory(path, create=False):
    path = local_path(path)
    fd = os.open('/', os.O_RDONLY | os.O_DIRECTORY)
    try:
        for part in path.parts[1:]:
            if create:
                try:
                    os.mkdir(part, 0o700, dir_fd=fd)
                except FileExistsError:
                    pass
            new = os.open(part, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW, dir_fd=fd)
            os.close(fd)
            fd = new
        trusted(fd)
        return fd
    except BaseException:
        os.close(fd)
        raise


def read_source(root, name):
    path = local_path(str(root / name))
    fd = directory(path.parent)
    try:
        leaf = os.open(path.name, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK, dir_fd=fd)
        with os.fdopen(leaf, 'rb') as stream:
            trusted(stream.fileno())
            if not stat.S_ISREG(os.fstat(stream.fileno()).st_mode):
                raise ValueError('regular file required')
            data = stream.read(65537)
            if len(data) > 65536:
                raise ValueError('source too large')
            data.decode('utf-8')
            return data
    finally:
        os.close(fd)


def install(kind, source, data_home, plugin_id=None):
    source = local_path(source)
    if plugin_id is not None and (kind != 'starter' or not re.fullmatch(r'dev\.[a-z0-9][a-z0-9.-]{0,59}', plugin_id)
                                  or '..' in plugin_id or plugin_id.endswith('.') or plugin_id in ('dev.fake', 'dev.starter')):
        raise ValueError('invalid or reserved development identity')
    files = dict(PINS) if kind == 'r1' else {'plugin.py': None}
    if kind == 'r1':
        files.update({'LICENSE': None, 'COPYING': None})
    snapshot = {}
    for name, pin in files.items():
        data = read_source(source, name)
        if pin and hashlib.sha256(data).hexdigest() != pin:
            raise ValueError('source integrity mismatch: ' + name)
        snapshot[name] = data
    base = local_path(data_home) / 'peripheral-hub'
    parent = base / 'integrations' if kind == 'r1' else base / 'dev-plugins' if plugin_id else base
    leaf = 'attack-shark-r1-linux' if kind == 'r1' else plugin_id or 'dev-starter'
    parent_fd = directory(parent, create=True)
    # Keep the verified parent descriptor throughout staging and rename. No
    # reopening a writable path between verification and installation.
    anchored = Path('/proc/self/fd') / str(parent_fd)
    temp = None
    try:
        if os.path.lexists(anchored / leaf):
            raise ValueError('destination exists; no overwrite or automatic update')
        temp = Path(tempfile.mkdtemp(prefix='.local-staging-', dir=anchored))
        for name, data in snapshot.items():
            target = temp / name
            target.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
            target.write_bytes(data)
            target.chmod(0o600)
        (temp / 'LOCAL_ONLY.json').write_text(json.dumps({
            'approval': 'not-approved', 'distribution': 'blocked-pending-rights',
            'sha256': {n: hashlib.sha256(d).hexdigest() for n, d in snapshot.items()},
        }, indent=2) + '\n')
        # Linux renameat2 RENAME_NOREPLACE: atomic and cannot replace even a
        # concurrently created empty directory. Fail closed if unavailable.
        import ctypes
        libc = ctypes.CDLL(None, use_errno=True)
        result = libc.renameat2(parent_fd, os.fsencode(temp.name), parent_fd, os.fsencode(leaf), 1)
        if result != 0:
            raise OSError(ctypes.get_errno(), 'atomic no-replace installation refused')
        temp = None
    finally:
        if temp is not None:
            shutil.rmtree(temp)
        os.close(parent_fd)
    return parent / leaf


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('kind', choices=('r1', 'starter'))
    parser.add_argument('--source', required=True, help='absolute local source directory')
    parser.add_argument('--id', help='registered dev.name for an independent starter copy')
    parser.add_argument('--accept-local-code', action='store_true', required=True,
                        help='consent to preparing local code; NOT redistribution rights')
    args = parser.parse_args()
    home = os.environ.get('XDG_DATA_HOME', str(Path.home() / '.local/share'))
    try:
        install(args.kind, args.source, home, args.id)
    except (OSError, ValueError) as error:
        parser.exit(1, 'Local preparation refused: ' + str(error) + '\n')
    print('Prepared local snapshot only. No execution, hardware or publication. Restart requires host consent for DEV code.')


if __name__ == '__main__':
    main()
