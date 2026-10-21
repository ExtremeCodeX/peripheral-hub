# Peripheral Hub

**Um projeto comunitário para controle aberto de periféricos, orientado às capacidades de cada dispositivo.**

[English](README.md) · [Como contribuir](docs/pt-BR/contributors/contributing.md) · [Segurança](docs/pt-BR/contributors/security.md) · [Status técnico](docs/architecture/implementation-status.md)

## Estado do projeto

O Peripheral Hub possui a **fundação Rust e o daemon implementados**: SDK 1.0, Registry, Core, descoberta/transporte de relatórios reais via `hidapi`, IPC local versionado, gerenciamento de conexões, simulação explícita, cenas de iluminação genéricas multi-zona, persistência de perfis e macros em SQLite com IPC tipado e import/export, aplicação de perfil vinculada ao modelo em um dispositivo conectado, contrato/prévia local de display sem transmissão de frames, um interpretador declarativo limitado (`serve --manifests DIR`, `schemas/device-manifest.schema.json`) e um frontend desktop React/Tauri sobre o daemon. O daemon e seu cliente de diagnóstico executam sem GUI. Um MVP de DRA independente oferece sessões locais de pesquisa, importação JSON estável de capturas, análise determinística de offsets com rótulos inferred/unverified, relatórios Markdown/JSON sanitizados e um scaffold totalmente comentado. Importação pcap/pcapng, integração REA e validação física de protocolos continuam pendentes, assim como drivers físicos proprietários, os demais schemas, empacotamento e CI; nenhuma capacidade de controle físico foi validada.

**Nenhuma compatibilidade física ou funcionalidade de hardware é declarada.** `devices/manufacturer/device-model/` é apenas um placeholder estrutural. A referência do projeto é [OPENCODE_IMPLEMENTATION_MASTER.md](OPENCODE_IMPLEMENTATION_MASTER.md); o [registro de status](docs/architecture/implementation-status.md) diferencia a documentação entregue dos marcos futuros de software.

## Objetivos

A configuração de periféricos costuma depender de aplicativos proprietários separados. O projeto pretende reunir teclados, mouses e futuramente outros periféricos em uma experiência desktop extensível, com protocolos documentados pela comunidade e níveis de suporte transparentes.

O princípio central é **mostrar apenas as capacidades realmente suportadas e validadas individualmente para o dispositivo conectado**. Suporte parcial é válido; um protocolo inferido não comprova funcionamento físico.

## Funcionalidades planejadas

- Descoberta e identificação HID, incluindo dispositivos compostos e múltiplas interfaces.
- Cores, brilho, zonas e efeitos RGB conforme o suporte efetivo.
- Controle de displays integrados com protocolo validado.
- Remapeamento e macros quando tecnicamente suportados; inicialmente, edição, validação, persistência e simulação segura de macros.
- Perfis persistentes com importação/exportação JSON versionada.
- Catálogo comunitário com evidências e validação por capacidade.

São itens do roadmap, não recursos disponíveis. Gravação de firmware, bootloader, escritas persistentes desconhecidas e carregamento arbitrário de plugins nativos estão fora do MVP.

## Tecnologias planejadas

| Área | Direção definida na especificação |
| --- | --- |
| Aplicação e workspaces | Rust estável, Cargo workspace; pnpm para pacotes JavaScript |
| Desktop | Tauri 2, React, TypeScript e Vite |
| Hardware | `hidapi` encapsulada em uma abstração de transporte própria |
| Processamento assíncrono | Tokio onde houver benefício real |
| Comunicação local | IPC versionado e restrito ao usuário; named pipes no Windows e sockets Unix protegidos |
| Persistência | SQLite para estado; JSON versionado para perfis |
| Drivers | TOML declarativo validado ou crates Rust revisadas e compiladas com a aplicação |
| Validação | Modelos tipados, regras semânticas e JSON Schema |
| Pesquisa | DRA em Rust; REA, USBPcap e Wireshark/TShark opcionais |

Windows 10/11 é a prioridade do produto. As bibliotecas da Fase 1 foram verificadas/testadas em um host Linux de laboratório; isso não estabelece suporte desktop ou físico no Linux. Builds Windows/macOS e backends futuros continuam sem validação. Python é permitido somente para ferramentas auxiliares justificadas de laboratório.

## Arquitetura planejada

```text
React/TypeScript → API Tauri restrita → cliente IPC tipado
    → daemon Rust na sessão do usuário → core / registry / SDK / drivers
    → transporte HID ou USB → periférico

DRA independente → evidências / fixtures sanitizadas / relatórios
    → contribuição de dispositivo revisada, fora do runtime normal
```

O frontend não acessará USB/HID diretamente. Operações não suportadas devem retornar erros explícitos; manifestos declarativos são entradas não confiáveis. Consulte a [arquitetura de referência em inglês](docs/architecture/overview.md).

## Como começar

Instale Rust via rustup e um linker nativo do sistema. `rust-toolchain.toml` fixa Rust estável **1.99.0**, rustfmt e Clippy. Na raiz do repositório:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo check --workspace
cargo test --workspace
cargo build --workspace --locked
```

Execute o backend explicitamente simulado em um terminal e consulte em outro:

```sh
cargo run -p peripheral-hub-daemon -- serve --simulate
cargo run -p peripheral-hub-daemon -- health
cargo run -p peripheral-hub-daemon -- list
```

Para inventário HID físico não destrutivo, use `cargo run -p peripheral-hub-daemon -- enumerate`; o daemon físico usa `serve` sem `--simulate`. Dispositivos desconhecidos não são abertos nem recebem relatórios automaticamente.

A CLI de pesquisa é independente e prioriza dados sintéticos:

```sh
cargo run -p device-research-assistant -- --help
cargo run -p device-research-assistant -- session create --name estudo --device "Pad de exemplo" --goal "Mapear bytes RGB" --consent
cargo run -p device-research-assistant -- capture import --session <id> --input capture.json
cargo run -p device-research-assistant -- analyze --session <id>
cargo run -p device-research-assistant -- report --session <id>
cargo run -p device-research-assistant -- scaffold --session <id>
```

Ela não executa binários do fabricante, não envia nem reproduz pacotes e não lê pcap/pcapng. O scaffold é um template comentado, não um driver. Consulte a [referência do DRA](docs/architecture/dra.md).

O frontend desktop (React/TypeScript) é um membro do workspace pnpm. Instale Node 20+/pnpm e rode:

```sh
pnpm install
pnpm --filter @peripheral-hub/desktop typecheck
pnpm --filter @peripheral-hub/desktop test
pnpm --filter @peripheral-hub/desktop build
```

Ele espera um daemon em execução e o acessa somente pela ponte Tauri restrita `daemon_call`. A ponte em `apps/desktop/src-tauri` é código real, mas não é compilada nem iniciada aqui por falta das dependências de webview, então não há instalador desktop. Consulte o [guia do daemon](docs/pt-BR/users/daemon.md), a [referência do desktop](docs/architecture/desktop.md), os [contratos do SDK](docs/architecture/sdk.md) e o [guia de contribuição](docs/pt-BR/contributors/contributing.md). Testes Linux e verificações estáticas para Windows são evidências distintas; execução Windows e controle físico não foram validados.

## Participação comunitária

São bem-vindas contribuições de documentação, acessibilidade, revisão arquitetural, investigação segura de protocolos e, futuramente, drivers e funcionalidades. Um novo dispositivo precisará de identificação de interfaces, evidências, fixtures sanitizadas, testes de falhas e matriz de validação física. Um instalador do fabricante, sozinho, pode não revelar todo o protocolo.

O uso de IA é opcional. As [sete skills](.agents/skills/README.md) orientam o processo, com autorização e revisão humanas. Não envie executáveis proprietários, capturas USB brutas, dados pessoais ou credenciais para issues, PRs ou serviços externos de IA.

Consulte [como adicionar dispositivos](docs/pt-BR/contributors/adding-a-device.md) e a [política de segurança](docs/pt-BR/contributors/security.md).

## Licença e governança

O repositório GitHub existente contém o texto Apache-2.0 em [LICENSE](LICENSE), preservado do commit inicial. O documento mestre ainda exige confirmação explícita do mantenedor antes de tratar essa escolha como definitiva para publicação oficial; essa confirmação permanece pendente.

Identidades dos responsáveis, contato de segurança, aplicação do código de conduta e atribuições de propriedade do repositório também estão pendentes e registradas no status técnico.
