# Daemon e diagnóstico — Fase 2

A [referência técnica em inglês](../../architecture/daemon.md) detalha os contratos e limitações. O desktop da Fase 3 ainda não foi implementado.

## Preparação e uso simulado

Instale Rust via rustup e um linker nativo. A toolchain fixa Rust estável 1.99.0. Na raiz:

```sh
cargo build --workspace --locked
cargo run -p peripheral-hub-daemon -- --help
```

Em um terminal:

```sh
cargo run -p peripheral-hub-daemon -- serve --simulate
```

Em outro:

```sh
cargo run -p peripheral-hub-daemon -- health
cargo run -p peripheral-hub-daemon -- list
cargo run -p peripheral-hub-daemon -- events
```

Copie um `interface_id` real da saída de `list` e substitua `INTERFACE_ID` nos exemplos abaixo. Esse texto é um placeholder, não um identificador fixo de hardware:

```sh
cargo run -p peripheral-hub-daemon -- connect INTERFACE_ID
cargo run -p peripheral-hub-daemon -- capabilities INTERFACE_ID
cargo run -p peripheral-hub-daemon -- lighting INTERFACE_ID 255 0 0
cargo run -p peripheral-hub-daemon -- state INTERFACE_ID
cargo run -p peripheral-hub-daemon -- disconnect INTERFACE_ID
cargo run -p peripheral-hub-daemon -- simulate-absent
cargo run -p peripheral-hub-daemon -- simulate-present
```

O modelo é fictício e todos os resultados são rotulados `simulated`. Relatórios JSON permanecem em memória; não são pacotes de fabricante nem capturas físicas. A simulação nunca é habilitada automaticamente após uma falha HID.

## Descoberta física

```sh
cargo run -p peripheral-hub-daemon -- enumerate
cargo run -p peripheral-hub-daemon -- serve
```

`enumerate` realiza inventário HID não destrutivo. O daemon sem `--simulate` consulta dispositivos reais, mas o registry físico está vazio: interfaces desconhecidas não são abertas nem recebem relatórios. Não há controle RGB físico validado. Caminhos privados e seriais são omitidos do IPC/diagnóstico.

O polling usa intervalo padrão de um segundo. Remoção, mudança de descritor e reconexão invalidam handles; falhas de enumeração são explícitas em `health` e não viram uma lista vazia bem-sucedida. IDs incluem épocas novas por sessão e não devem ser reaproveitados após reinicialização; clientes devem redefinir cursores de eventos quando `Health.session_id` mudar. Agrupamento físico entre caminhos de interfaces compostas ainda é conservador/incompleto, e transições entre polls podem não ser observadas.

## IPC e encerramento

IPC é local e versionado, sem listener de rede ou elevação automática. Unix exige diretório real do usuário com modo 0700, socket 0600 e UID remoto igual; named pipes Windows têm DACL do SID atual, rejeitam conexões remotas e verificam o proprietário dos processos. O endpoint padrão é compartilhado pelo daemon/cliente; `--endpoint PATH` seleciona uma instância de desenvolvimento explícita e mantém essas validações.

Ctrl+C e SIGTERM no Unix encerram o daemon e fecham conexões. Instância duplicada no mesmo endpoint é recusada. Mensagens, clientes, fila, taxa, eventos e deadlines são limitados. Operações que excedem o deadline não são repetidas automaticamente: uma escrita nativa já iniciada pode não ser cancelável pelo `hidapi`, e seu resultado é desconhecido, nunca sucesso presumido.

Os testes Linux usam sockets reais e simulação; a enumeração física foi verificada separadamente. Verificações estáticas Windows não comprovam DACL/runtime ou hardware Windows. Não foram executados relatórios físicos, firmware, bootloader, escrita persistente ou captura USB. Perfis/SQLite/DRA/desktop/CI continuam pendentes. Consulte o [registro de evidências](../../architecture/implementation-status.md).
