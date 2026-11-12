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
`bundle.active` é false: não se promete AppImage, pacote ou runtime embutido.
O release não registra os comandos DEV; catálogo continua somente consulta.
