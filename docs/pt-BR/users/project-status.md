# Guia do usuário: disponibilidade atual

[English](../../en/users/project-status.md)

O Peripheral Hub possui daemon/CLI Rust executáveis, enumeração/transporte HID real implementado, simulação explícita, persistência de perfis em SQLite com IPC de perfis tipado e import/export JSON, persistência de macros em SQLite com IPC de macros tipado (edição/validação/import/export, **sem execução ou injeção de entrada**), contrato de display com tela de validação/pré-visualização local (**sem transmissão de frames**), e um frontend desktop React/TypeScript com ponte Tauri restrita. Ainda não há driver físico, shell desktop compilado/instalável neste ambiente, injeção de entrada ou transmissão de displays. Um MVP de DRA independente oferece sessões JSON locais, importação limitada de capturas textuais, análise determinística, relatórios sanitizados e scaffold comentado; ele não lê pcap/pcapng, não executa ferramentas do fabricante, não reproduz pacotes e não acessa hardware. Evidências de execução/software no Linux e checagens estáticas Windows não validam controle físico nem runtime Windows. Consulte o [guia do daemon](daemon.md), a [referência do desktop](../../architecture/desktop.md) e a [referência do DRA](../../architecture/dra.md).

## Instalação e uso

Não há instalador desktop ou formato de perfil distribuído. O comando real do daemon é `cargo run -p peripheral-hub-daemon -- serve --simulate`; consulte com `health` e `list` em outro terminal. Sem `--simulate`, há descoberta real, mas nenhum driver de controle físico registrado.

O frontend desktop passa typecheck, testes unitários e build com pnpm (`pnpm install`, depois `pnpm --filter @peripheral-hub/desktop typecheck|test|build`) e espera um daemon em execução. A ponte Tauri em `apps/desktop/src-tauri` é código real, mas não é compilada nem iniciada neste ambiente por falta de dependências de webview, então ainda não há aplicativo desktop instalável. Arquivos de schema/configuração que continuam vazios são placeholders, não pacotes funcionais ou entradas de compatibilidade.

Quando existirem releases reais, este guia deverá conter instalação testada, inicialização separada de desktop/daemon, importação/exportação de perfis, diagnóstico e matriz de compatibilidade comprovada. Windows 10/11 é a prioridade pretendida, não uma garantia atual.

A CLI de pesquisa é independente do daemon e do desktop. Execute `cargo run -p device-research-assistant -- --help`; sessões exigem `--consent`, ficam fora do repositório no diretório de dados do usuário e aceitam apenas o formato JSON documentado. Não há backend de captura ao vivo, parser pcap, replay de pacotes ou caminho de escrita no hardware.

## Diagnóstico e pedidos de suporte

Para solicitar suporte a um dispositivo, registre modelo, SO, firmware quando conhecido e capacidades desejadas, sem publicar serial, caminho privado, executável ou captura bruta. Consulte [como adicionar um dispositivo](../contributors/adding-a-device.md). Templates GitHub e contato privado de segurança ainda não estão configurados; não envie evidências sensíveis a issues públicas.

O [README](../../../README.pt-BR.md) e o [status técnico](../../architecture/implementation-status.md) mostram o progresso real. Bibliotecas e daemon possuem build/testes; consulte o [guia do daemon](daemon.md) e a [referência do desktop](../../architecture/desktop.md) para diagnóstico. Não há GUI desktop compilada para inicializar neste ambiente.
