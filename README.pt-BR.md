# Peripheral Hub

**Um projeto comunitário para controle aberto de periféricos, orientado às capacidades de cada dispositivo.**

[English](README.md) · [Como contribuir](docs/pt-BR/contributors/contributing.md) · [Segurança](docs/pt-BR/contributors/security.md) · [Status técnico](docs/architecture/implementation-status.md)

## Estado do projeto

O Peripheral Hub está na **etapa de fundação documental e organizacional**. Há uma estrutura de diretórios planejada, orientações de contribuição, regras arquiteturais e procedimentos de skills de IA. Os arquivos de código, manifests, schemas e CI continuam como placeholders vazios. Ainda não existem aplicativo desktop, daemon, Device Research Assistant (DRA) ou driver validado executáveis.

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
- DRA independente para sessões locais de pesquisa, importação de capturas, comparação de pacotes, relatórios sanitizados e scaffolds baseados em evidências.

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

Windows 10/11 é a prioridade. Linux e macOS são alvos futuros com backends separados, sem suporte ou testes declarados nesta etapa. Python é permitido somente para ferramentas auxiliares justificadas de laboratório.

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

Leia o [guia de contribuição](docs/pt-BR/contributors/contributing.md), o [AGENTS.md](AGENTS.md) e o registro de status. Já é possível contribuir com documentação e pesquisa local autorizada de protocolos.

**Ainda não há comandos funcionais de instalação, execução, build ou testes**: manifests e lockfiles Cargo/pnpm estão vazios. Versões dos runtimes, dependências de plataforma e comandos exatos do daemon, desktop e DRA serão documentados quando houver workspaces reais. Os comandos futuros de validação estão identificados como tal no guia de contribuição.

## Participação comunitária

São bem-vindas contribuições de documentação, acessibilidade, revisão arquitetural, investigação segura de protocolos e, futuramente, drivers e funcionalidades. Um novo dispositivo precisará de identificação de interfaces, evidências, fixtures sanitizadas, testes de falhas e matriz de validação física. Um instalador do fabricante, sozinho, pode não revelar todo o protocolo.

O uso de IA é opcional. As [sete skills](.agents/skills/README.md) orientam o processo, com autorização e revisão humanas. Não envie executáveis proprietários, capturas USB brutas, dados pessoais ou credenciais para issues, PRs ou serviços externos de IA.

Consulte [como adicionar dispositivos](docs/pt-BR/contributors/adding-a-device.md) e a [política de segurança](docs/pt-BR/contributors/security.md).

## Licença e governança

Apache-2.0 é uma **proposta pendente de confirmação do mantenedor**. O `LICENSE` existente está vazio e não concede uma licença. Não apresente o projeto como já licenciado nem publique uma release oficial antes dessa decisão.

Identidades dos responsáveis, contato de segurança, aplicação do código de conduta e atribuições de propriedade do repositório também estão pendentes e registradas no status técnico.
