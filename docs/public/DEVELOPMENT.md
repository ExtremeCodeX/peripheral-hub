# Desenvolvimento Linux

Requer Node >=22.12, npm >=10, Rust estável/Cargo, Python >=3.11 em
`/usr/bin/python3`, GTK 3 e WebKitGTK 4.1 de desenvolvimento.
As dependências exatas variam por distribuição; veja os
[pré-requisitos Tauri 2](https://tauri.app/start/prerequisites/).

Em Debian/Ubuntu, instale pelos meios administrativos da distribuição:
`build-essential`, `pkg-config`, `libwebkit2gtk-4.1-dev`, `libgtk-3-dev`,
`libxdo-dev`, `libssl-dev`, `librsvg2-dev`, `curl`, `wget`, `file`, `python3`.
Essa lista não executa sudo ou altera o sistema automaticamente.

```sh
npm ci
npm run dev
```

`dev:web` é apenas frontend; não executa plugins. Não execute o Hub como root.
O R1 não é dependência de build ou teste do núcleo.

## Testes

```sh
npm test -- --maxWorkers=1
npm run typecheck
npm run test:python
npm run test:catalog
npm run test:rust
npm run fmt:rust
npm run build:web
npm run build
```

`test:rust` prepara um XDG temporário com fixtures DEV independentes e executa
todos os targets Rust. Pode-se acrescentar `-- --offline --bin peripheral-hub`
se as dependências já estiverem em cache. Não depende do XDG pessoal ou do R1.
Sem cache, npm/Cargo precisam de rede para dependências fixadas nos lockfiles.

Os testes usam simulação e peers locais. Não abrem HID físico nem comprovam
mudança em mouse real. A confirmação de teste injetada em Rust não substitui o
consentimento GTK do aplicativo. Para isolamento adicional em Linux, execute
as suítes sob bwrap com `/dev` privado, `/sys` vazio e rede isolada; isso é um
ambiente de ensaio, não uma sandbox fornecida pelo produto.

O release fica em `apps/desktop/src-tauri/target/release/peripheral-hub`.
Na configuração padrão, `bundle.active` é false; o AppImage usa a configuração
separada abaixo. O release não registra os comandos DEV. A distribuição pública
do X6 é fixada; o catálogo comunitário remoto contém somente metadados.

## AppImage Linux x86_64

O empacotamento AppImage é opcional e usa uma configuração separada:

```sh
npm ci
npm run build:appimage
```

O arquivo é gerado em `apps/desktop/src-tauri/target/release/bundle/appimage/`.
O build padrão `npm run build` continua gerando somente o executável.
Para distribuir, compile em Ubuntu 22.04 ou outra base antiga compatível com
WebKitGTK 4.1; compilar em uma distribuição mais recente pode exigir uma glibc
mais nova no computador do usuário. São necessários os pré-requisitos de build
do Tauri, `patchelf` e as ferramentas AppImage que o Tauri baixa ao empacotar.
Em um contêiner sem FUSE, use `APPIMAGE_EXTRACT_AND_RUN=1` durante o build.

O AppImage inclui os recursos de licença do Hub e da fonte. Não inclui Python:
os plugins continuam exigindo Python 3.11+ em `/usr/bin/python3`. O formato não
concede acesso hidraw, não instala regras udev e não é uma sandbox.

No artefato publicado, os copyrights de `/usr/share/doc/*/copyright`, os textos
de `/usr/share/common-licenses` e um inventário `dpkg-query` com versões e nomes
dos pacotes-fonte do ambiente Ubuntu 22.04 também foram preservados sob
`licenses/system-runtime-ubuntu-22.04/`, dentro dos recursos do AppImage.
É um inventário conservador do ambiente, não uma afirmação de que todos os
pacotes listados sejam redistribuídos. Preserve os avisos dos componentes
efetivamente empacotados ao gerar um novo artefato em outro ambiente.
