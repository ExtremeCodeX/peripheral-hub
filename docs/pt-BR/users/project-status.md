# Guia do usuário: disponibilidade atual

[English](../../en/users/project-status.md)

O Peripheral Hub possui daemon/CLI Rust da Fase 2 executáveis, enumeração/transporte HID real implementado e simulação explícita. Ainda não há GUI desktop instalável, driver físico, perfis, macros, displays ou DRA. Evidências de execução/software no Linux e checagens estáticas Windows não validam controle físico nem runtime Windows. Consulte o [guia do daemon](daemon.md).

## Instalação e uso

Não há instalador desktop ou formato de perfil. O comando real do daemon é `cargo run -p peripheral-hub-daemon -- serve --simulate`; consulte com `health` e `list` em outro terminal. Sem `--simulate`, há descoberta real, mas nenhum driver de controle físico registrado. Arquivos restantes de desktop/DRA/configurações/schemas são placeholders, não pacotes funcionais ou entradas de compatibilidade.

Quando existirem releases reais, este guia deverá conter instalação testada, inicialização separada de desktop/daemon, importação/exportação de perfis, diagnóstico e matriz de compatibilidade comprovada. Windows 10/11 é a prioridade pretendida, não uma garantia atual.

## Diagnóstico e pedidos de suporte

Para solicitar suporte a um dispositivo, registre modelo, SO, firmware quando conhecido e capacidades desejadas, sem publicar serial, caminho privado, executável ou captura bruta. Consulte [como adicionar um dispositivo](../contributors/adding-a-device.md). Templates GitHub e contato privado de segurança ainda não estão configurados; não envie evidências sensíveis a issues públicas.

O [README](../../../README.pt-BR.md) e o [status técnico](../../architecture/implementation-status.md) mostram o progresso real. Bibliotecas e daemon possuem build/testes; consulte o [guia do daemon](daemon.md) para diagnóstico. Ainda não há GUI desktop para inicializar.
