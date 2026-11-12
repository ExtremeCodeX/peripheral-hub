#!/usr/bin/env python3
"""Run Rust's offline suite with a fresh, separate DEV snapshot and no R1."""
import importlib.util
import os
from pathlib import Path
import subprocess
import sys
import tempfile

root = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('installer', root / 'tools/install-local.py')
installer = importlib.util.module_from_spec(spec)
spec.loader.exec_module(installer)
with tempfile.TemporaryDirectory(prefix='hub-rust-') as temporary:
    installer.install('starter', root / 'community/integration-starter', Path(temporary))
    source = Path(temporary) / 'source'
    source.mkdir()
    text = (root / 'community/integration-starter/plugin.py').read_text().replace('dev.starter', 'dev.test-mouse').replace('sample_value', 'gain')
    (source / 'plugin.py').write_text(text)
    installer.install('starter', source, Path(temporary), 'dev.test-mouse')
    result = subprocess.run(['cargo', 'test', '--locked', '--manifest-path', str(root / 'apps/desktop/src-tauri/Cargo.toml'), *sys.argv[1:]],
                            env={**os.environ, 'XDG_DATA_HOME': temporary}, cwd=root)
    sys.exit(result.returncode)
