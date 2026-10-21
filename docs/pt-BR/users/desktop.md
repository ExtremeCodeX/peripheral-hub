# Interface desktop (Tauri 2 + React/TypeScript)

A [referência técnica em inglês](../../architecture/desktop.md) detalha arquitetura, telas e limitações. O frontend, a ponte Tauri e o shell nativo existem como código; o shell compila e roda em uma máquina com as dependências de webview instaladas (não no sandbox de validação automática). Veja o [ADR 0005](../../architecture/decisions/0005-desktop-ui.md) e o [registro de evidências](../../architecture/implementation-status.md).

## O que já existe

- Frontend React/TypeScript em `apps/desktop/src`, sem acesso direto a USB/HID nem ao socket: ele envia `Operation` tipadas para a única ponte Tauri (`daemon_call`), que fala o IPC local do daemon (versão 2).
- Telas de dispositivos, detalhes, iluminação, perfis, macros, displays e diagnóstico, com estados de vazio, carregando, desconectado, permissão negada, erro e recurso não suportado.
- Perfis usam as operações reais do daemon (criar/renomear/editar/salvar/aplicar/excluir) mais import/export JSON versionado.
- Macros apenas editam, validam, persistem e geram um **trace de simulação descritivo**; nada é injetado no sistema.
- Displays mostram o contrato declarado e um preview; nenhum frame é transmitido.

## Dependências nativas

O frontend React só precisa de Node 20+ e pnpm. Para compilar e abrir o shell Tauri, instale também o webview do sistema e um compilador C:

- **Arch Linux:** `sudo pacman -S --needed base-devel pkgconf gtk3 webkit2gtk-4.1` (`librsvg` só ao empacotar ícones).
- **Debian/Ubuntu:** `build-essential pkg-config libgtk-3-dev libwebkit2gtk-4.1-dev libssl-dev`.
- **Windows:** ferramentas de build C++ (MSVC) e o runtime WebView2.

Sem esses pacotes, a compilação falha dentro de `gtk-sys`/`webkit2gtk-sys` (falta o pacote `pkg-config` `gtk+-3.0`/`webkit2gtk-4.1`), não no código deste projeto.

## Compilação e testes do frontend

Instale Node 20+ e pnpm. Na raiz:

```sh
pnpm install --frozen-lockfile
pnpm --filter @peripheral-hub/desktop typecheck
pnpm --filter @peripheral-hub/desktop test
pnpm --filter @peripheral-hub/desktop build
```

O desktop precisa de um daemon acessível: inicie antes `cargo run -p peripheral-hub-daemon -- serve --simulate`.

O shell Tauri é aberto pelo seu próprio manifesto e não define `devUrl`: debug e release embutem `frontendDist` (`../dist`), então não precisa de servidor de dev. Compile o frontend uma vez e abra o shell:

```sh
pnpm --filter @peripheral-hub/desktop build
cargo run --manifest-path apps/desktop/src-tauri/Cargo.toml
```

Para desenvolvimento com recarregamento do Vite, adicione `"devUrl": "http://localhost:5173"` e `"beforeDevCommand": "pnpm dev"` ao bloco `build` e rode `pnpm dev` antes do shell.

## Limitações

- A ponte Tauri (`apps/desktop/src-tauri`) tem manifesto próprio e depende de webview do sistema (webkit2gtk/GTK no Linux, WebView2 no Windows). No sandbox de validação ela **não** foi compilada (faltam `pkg-config`/GTK/WebKit); em uma máquina com essas dependências ela compila e roda, e, contra um daemon `serve --simulate`, lista o dispositivo simulado, aplica iluminação simulada e persiste um perfil após reiniciar o daemon.
- Não há instalador, instância única nem auto-start do daemon: os dois processos são iniciados manualmente.
- Windows permanece apenas com evidência estática de compilação.
- O DRA é uma CLI independente, fora desta interface; os schemas de captura/relatório de pesquisa existem, mas o DRA não reproduz pacotes nem acessa hardware. Não há CI, empacotamento, releases assinadas ou drivers físicos. Não trate esta camada como comprovação de controle físico.
