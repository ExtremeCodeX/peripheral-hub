"""Pinned release installer. No extraction, execution, HID, or renderer URLs."""
import hashlib
import json
import os
from pathlib import Path
import re
import secrets
import stat
import urllib.request
import urllib.parse


def trusted(file):
    info=os.fstat(file)
    if info.st_uid!=os.geteuid() or info.st_mode&0o022:
        raise ValueError('UNSAFE_INSTALLATION')


def root(base):
    path=Path(base)
    if not path.is_absolute() or '..' in path.parts:raise ValueError('UNSAFE_INSTALLATION')
    fd=os.open('/',os.O_RDONLY|os.O_DIRECTORY)
    try:
        for part in (*path.parts[1:],'peripheral-hub','plugins'):
            try:os.mkdir(part,0o700,dir_fd=fd)
            except FileExistsError:pass
            child=os.open(part,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW,dir_fd=fd)
            os.close(fd);fd=child
        trusted(fd)
        return fd
    except BaseException:
        os.close(fd);raise


def read_file(directory,name,limit):
    fd=os.open(name,os.O_RDONLY|os.O_NOFOLLOW|os.O_NONBLOCK,dir_fd=directory)
    try:
        trusted(fd)
        if not stat.S_ISREG(os.fstat(fd).st_mode):raise ValueError('UNSAFE_INSTALLATION')
        with os.fdopen(os.dup(fd),'rb') as f:data=f.read(limit+1)
        if len(data)>limit:raise ValueError('PACKAGE_TOO_LARGE')
        return data
    finally:os.close(fd)


def installed(directory,entry):
    package=entry['package']
    try:fd=os.open(package['id'],os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW,dir_fd=directory)
    except FileNotFoundError:return False
    try:
        trusted(fd)
        source=read_file(fd,'plugin.py',65536)
        manifest=read_file(fd,'manifest.json',16384)
        license_text=read_file(fd,'LICENSE',16384)
        if hashlib.sha256(source).hexdigest()!=package['sha256'] or len(source)!=package['byteLength'] or hashlib.sha256(manifest).hexdigest()!=entry['manifestSha256']:
            raise ValueError('INSTALLED_PACKAGE_INTEGRITY')
        if hashlib.sha256(license_text).hexdigest()!=package['licenseSha256']:raise ValueError('INSTALLED_LICENSE_INTEGRITY')
        return True
    finally:os.close(fd)


def safe_url(url):
    p=urllib.parse.urlsplit(url)
    if p.scheme!='https' or p.hostname not in ('github.com','release-assets.githubusercontent.com','objects.githubusercontent.com') or p.username or p.password or p.port not in (None,443):
        raise ValueError('UNSAFE_PACKAGE_URL')
    return url


class Redirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self,req,fp,code,msg,headers,newurl):
        return super().redirect_request(req,fp,code,msg,headers,safe_url(newurl))


def download(url,limit):
    opener=urllib.request.build_opener(urllib.request.ProxyHandler({}),Redirect())
    with opener.open(urllib.request.Request(safe_url(url),headers={'User-Agent':'Peripheral-Hub/0.2.0','Accept-Encoding':'identity'}),timeout=12) as response:
        if response.status!=200:raise ValueError('PACKAGE_HTTP_ERROR')
        data=response.read(limit+1)
    if len(data)>limit:raise ValueError('PACKAGE_TOO_LARGE')
    return data


def install(directory,entry,fetch=download):
    package=entry['package']
    if installed(directory,entry):return
    manifest=fetch(entry['manifestUrl'],16384)
    if hashlib.sha256(manifest).hexdigest()!=entry['manifestSha256'] or json.loads(manifest)!=package:
        raise ValueError('MANIFEST_INTEGRITY')
    source=fetch(package['artifact'],65536)
    if len(source)!=package['byteLength'] or hashlib.sha256(source).hexdigest()!=package['sha256']:
        raise ValueError('PACKAGE_INTEGRITY')
    source.decode('utf-8',errors='strict')
    license_text=fetch(package['licenseUrl'],16384)
    if hashlib.sha256(license_text).hexdigest()!=package['licenseSha256']:raise ValueError('LICENSE_INTEGRITY')
    temporary='.install-'+secrets.token_hex(16)
    os.mkdir(temporary,0o700,dir_fd=directory)
    fd=os.open(temporary,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW,dir_fd=directory)
    try:
        for name,data in (('plugin.py',source),('manifest.json',manifest),('LICENSE',license_text)):
            out=os.open(name,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600,dir_fd=fd)
            with os.fdopen(out,'wb') as f:f.write(data);f.flush();os.fsync(f.fileno())
        os.fsync(fd)
        # Never replace a populated installation or touch the DEV directory.
        os.rename(temporary,package['id'],src_dir_fd=directory,dst_dir_fd=directory)
        os.fsync(directory)
    finally:os.close(fd)


def run(request):
    entries=request['registry']
    for entry in entries:
        if not re.fullmatch(r'[a-z0-9]+(?:[.-][a-z0-9]+)+',entry['package']['id']):raise ValueError('INVALID_ID')
    fd=root(request['base'])
    try:
        if request['action']=='install':
            entry=next((e for e in entries if e['package']['id']==request['id']),None)
            if entry is None:raise ValueError('UNKNOWN_PACKAGE')
            install(fd,entry)
        elif request['action']!='list':raise ValueError('INVALID_ACTION')
        return [dict(e['package'],installed=installed(fd,e)) for e in entries]
    finally:os.close(fd)


def worker_main():
    import sys
    try:print(json.dumps({'software':run(json.load(sys.stdin))}))
    except Exception as error:
        message=str(error)
        print(json.dumps({'error':message if isinstance(error,ValueError) else 'INSTALLATION_FAILED'}))
