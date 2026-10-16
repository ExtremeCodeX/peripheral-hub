# Interface desktop (Tauri 2 + React/TypeScript)

A [referência técnica em inglês](../../architecture/desktop.md) detalha arquitetura, telas e limitações. O frontend e a ponte Tauri existem como código; a ponte não é compilada nem executada no ambiente atual (faltam dependências de webview). Veja o [ADR 0005](../../architecture/decisions/0005-desktop-ui.md) e o [registro de evidências](../../architecture/implementation-status.md).

## O que já existe

- Frontend React/TypeScript em `apps/desktop/src`, sem acesso direto a USB/HID nem ao socket: ele envia `Operation` tipadas para a única ponte Tauri (`daemon_call`), que fala o IPC local do daemon (versão 2).
- Telas de dispositivos, detalhes, iluminação, perfis, macros, displays e diagnóstico, com estados de vazio, carregando, desconectado, permissão negada, erro e recurso não suportado.
- Perfis usam as operações reais do daemon (criar/renomear/editar/salvar/aplicar/excluir) mais import/export JSON versionado.
- Macros apenas editam, validam, persistem e geram um **trace de simulação descritivo**; nada é injetado no sistema.
- Displays mostram o contrato declarado e um preview; nenhum frame é transmitido.

## Compilação e testes do frontend

Instale Node 20+ e pnpm. Na raiz:

```sh
pnpm install
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

- A ponte Tauri (`apps/desktop/src-tauri`) tem manifesto próprio e depende de webview do sistema (webkit2gtk/GTK no Linux, WebView2 no Windows). Neste ambiente ela **não** foi compilada nem iniciada; não há instalador, runtime desktop nem evidência ponta a ponta.
- Windows permanece apenas com evidência estática de compilação.
- Não há DRA, schemas de manifesto, CI, empacotamento, releases assinadas, drivers físicos ou aplicação automática de um perfil a um dispositivo. Não trate esta camada como comprovação de controle físico.
