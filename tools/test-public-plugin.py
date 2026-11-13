#!/usr/bin/env python3
"""Real release loader/host with a pinned package; handshake only, NO HID calls."""
import argparse,hashlib,importlib.util,json,os,subprocess,tempfile
from pathlib import Path

ROOT=Path(__file__).resolve().parents[1]
p=argparse.ArgumentParser(description=__doc__);p.add_argument('--package',type=Path,required=True)
a=p.parse_args();entry=json.loads((ROOT/'catalog/installable.json').read_text())[0]
manifest=(a.package/'manifest.json').read_bytes()
artifact=a.package/Path(entry['package']['artifact']).name
source=(artifact if artifact.exists() else a.package/'plugin.py').read_bytes()
assert hashlib.sha256(manifest).hexdigest()==entry['manifestSha256']
assert hashlib.sha256(source).hexdigest()==entry['package']['sha256']
spec=importlib.util.spec_from_file_location('distribution',ROOT/'apps/desktop/src-tauri/src/distribution_worker.py')
worker=importlib.util.module_from_spec(spec);spec.loader.exec_module(worker)
with tempfile.TemporaryDirectory(prefix='hub-public-') as tmp:
    work=Path(tmp);base=work/'data';base.mkdir()
    fd=worker.root(str(base))
    try:worker.install(fd,entry,lambda url,limit: (a.package/'LICENSE').read_bytes() if url.endswith('LICENSE') else source if url.endswith('.py') else manifest)
    finally:os.close(fd)
    (work/'src').mkdir()
    (work/'Cargo.toml').write_text('''[package]
name="hub-public-contract"
version="0.1.0"
edition="2021"
[dependencies]
serde={version="1",features=["derive"]}
serde_json="1"
sha2="=0.10.9"
libc="0.2"
''')
    host=ROOT/'apps/desktop/src-tauri/src/plugin_host.rs'
    (work/'src/main.rs').write_text(f'#[path={json.dumps(str(host))}] mod plugin_host;\n'+'''
use plugin_host::{DevAction as A,DevHost};
fn main(){
 let id="io.github.extremecodex.attack-shark-x6";
 let mut host=DevHost::default();
 let start=||A::StartHardware{plugin_id:id.into(),development:false};
 assert_eq!(host.execute(start()).unwrap_err(),"NATIVE_CONFIRMATION_REQUIRED");
 let hello=host.execute_with_confirmation(start(),|c|{assert_eq!(c.method,"process.start.public");Ok(())}).unwrap();
 assert_eq!(hello["pluginId"],id);assert_eq!(hello["mode"],"hardware");
 let caps=host.execute(A::Capabilities{}).unwrap();assert_eq!(caps["settings"].as_array().unwrap().len(),30);
 host.execute(A::Stop{}).unwrap();
 println!("PASS: pinned public package installed, release runtime handshake/capabilities/stop; NO discovery, NO HID I/O.");
}
''')
    subprocess.run(['cargo','run','--offline','--release','--manifest-path',str(work/'Cargo.toml')],env={**os.environ,'XDG_DATA_HOME':str(base),'CARGO_TARGET_DIR':'/tmp/opencode/hub-public-contract-target'},check=True)
