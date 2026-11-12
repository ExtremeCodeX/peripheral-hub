#!/usr/bin/env python3
"""Create an independent simulated plugin from the shipped starter; never execute it."""
import argparse
import json
from pathlib import Path
import re
import shutil


def create(destination, identity):
    if not re.fullmatch(r'dev\.[a-z0-9][a-z0-9.-]{0,59}', identity) or '..' in identity or identity.endswith('.') or identity in ('dev.fake', 'dev.starter'):
        raise ValueError('use a non-reserved dev.name identity')
    destination = Path(destination)
    if destination.exists() or destination.is_symlink():
        raise ValueError('destination must not exist')
    template = Path(__file__).resolve().parents[1] / 'community/integration-starter'
    shutil.copytree(template, destination, ignore=shutil.ignore_patterns('__pycache__', '.pytest_cache'))
    for file in [destination / 'plugin.py', destination / 'tests/test_plugin.py']:
        file.write_text(file.read_text().replace('dev.starter', identity))
    shutil.copyfile(template.parents[1] / 'LICENSE', destination / 'LICENSE')
    manifest = json.loads((template.parent / 'templates/publication.json').read_text())
    manifest['id'] = identity
    (destination / 'publication.json').write_text(json.dumps(manifest, indent=2, ensure_ascii=False) + '\n')
    (destination / 'README.md').write_text(f'# {identity}\n\nSimulador JSON-RPC independente gerado do starter Peripheral Hub.\n'
        'Não é suporte físico aprovado. Código inicial Apache-2.0; preserve licenças de qualquer código reutilizado.\n\n'
        'Teste: `python3 -B -m unittest discover -s tests -v`.\n'
        'Edite `SETTINGS`, `INITIAL_VALUES` e a validação em `plugin.py`.\n'
        'Registre a fonte pelo tools/install-local.py do Hub e selecione esta identidade no menu DEV.\n'
        'Edite a cópia XDG e pare/inicie para recarregar sem recompilar o Hub.\n'
        'Preencha publication.json com fatos reais, origem e vídeo antes de submeter ao catálogo.\n')
    return destination.resolve()


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('destination')
    parser.add_argument('--id', required=True)
    args = parser.parse_args()
    try:
        print(create(args.destination, args.id))
    except (ValueError, OSError) as error:
        parser.exit(1, str(error) + '\n')
