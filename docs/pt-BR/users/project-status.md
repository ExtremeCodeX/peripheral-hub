# Guia do usuário: disponibilidade atual

[English](../../en/users/project-status.md)

O Peripheral Hub possui daemon/CLI Rust executáveis, enumeração/transporte HID real implementado, simulação explícita, persistência de perfis em SQLite com IPC de perfis tipado, e um frontend desktop React/TypeScript com ponte Tauri restrita. Ainda não há driver físico, shell desktop compilado/instalável neste ambiente, comando de import/export de perfis, injeção de macros, transmissão de displays ou DRA. Evidências de execução/software no Linux e checagens estáticas Windows não validam controle físico nem runtime Windows. Consulte o [guia do daemon](daemon.md) e a [referência do desktop](../../architecture/desktop.md).

## Instalação e uso

Não há instalador desktop ou formato de perfil distribuído. O comando real do daemon é `cargo run -p peripheral-hub-daemon -- serve --simulate`; consulte com `health` e `list` em outro terminal. Sem `--simulate`, há descoberta real, mas nenhum driver de controle físico registrado.

O frontend desktop passa typecheck, testes unitários e build com pnpm (`pnpm install`, depois `pnpm --filter @peripheral-hub/desktop typecheck|test|build`) e espera um daemon em execução. A ponte Tauri em `apps/desktop/src-tauri` é código real, mas não é compilada nem iniciada neste ambiente por falta de dependências de webview, então ainda não há aplicativo desktop instalável. Arquivos restantes de DRA/schemas/configurações são placeholders, não pacotes funcionais ou entradas de compatibilidade.

Quando existirem releases reais, este guia deverá conter instalação testada, inicialização separada de desktop/daemon, importação/exportação de perfis, diagnóstico e matriz de compatibilidade comprovada. Windows 10/11 é a prioridade pretendida, não uma garantia atual.

## Diagnóstico e pedidos de suporte

Para solicitar suporte a um dispositivo, registre modelo, SO, firmware quando conhecido e capacidades desejadas, sem publicar serial, caminho privado, executável ou captura bruta. Consulte [como adicionar um dispositivo](../contributors/adding-a-device.md). Templates GitHub e contato privado de segurança ainda não estão configurados; não envie evidências sensíveis a issues públicas.

O [README](../../../README.pt-BR.md) e o [status técnico](../../architecture/implementation-status.md) mostram o progresso real. Bibliotecas e daemon possuem build/testes; consulte o [guia do daemon](daemon.md) e a [referência do desktop](../../architecture/desktop.md) para diagnóstico. Não há GUI desktop compilada para inicializar neste ambiente.
