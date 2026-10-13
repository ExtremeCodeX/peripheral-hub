# Peripheral Hub — Instruções mestras de implementação para o OpenCode

> **Destinatário:** agente de programação executado dentro do OpenCode, na raiz do repositório.
>
> **Objetivo:** transformar o repositório existente em um projeto open source funcional e extensível para controlar periféricos (teclados, mouses e, futuramente, outros dispositivos) e permitir que a comunidade adicione suporte a hardware com ajuda de agentes de IA e do **Device Research Assistant (DRA)**.
>
> **Importante:** os diretórios e diversos arquivos **já foram criados pelo mantenedor**. Não recrie a árvore indiscriminadamente, não apague conteúdo existente e não presuma que os arquivos estejam vazios. Inspecione primeiro, complete depois.

## 0. Sua missão e seu modo de execução

Você está autorizado a **implementar código, testes, configurações, scripts e documentação neste repositório**, e não apenas escrever um planejamento. Trabalhe de forma incremental até entregar tudo o que puder ser corretamente implementado no ambiente disponível. Se uma fase exigir periférico real, sistema operacional específico, executável proprietário, permissão elevada ou ferramenta ausente, **não fabrique resultados**: implemente a infraestrutura e os testes simulados, documente precisamente o bloqueio e continue com o restante que seja independente dele.

Procedimento obrigatório:

1. Inspecione a árvore, os arquivos existentes, o estado do Git, as ferramentas instaladas e a versão de cada runtime. Identifique implementações já existentes para não duplicá-las.
2. Leia o `AGENTS.md`, os `SKILL.md` e as decisões arquiteturais existentes. Se estiverem vazios, produza o conteúdo descrito neste documento.
3. Registre um plano de execução com tarefas verificáveis em `docs/architecture/implementation-status.md`. Atualize esse arquivo ao final de cada fase, com estados **concluído / parcial / bloqueado**, evidências e comandos executados.
4. Implemente de fato as fases abaixo. Prefira uma pequena implementação real, testada, a arquivos vazios, pseudocódigo ou demonstrações que fingem controlar hardware.
5. Execute formatação, checagens, testes e builds. Corrija os problemas encontrados. Se algum comando falhar por ambiente ou ferramenta ausente, registre a causa exata.
6. Preserve alterações pré-existentes do usuário. Não execute `git reset --hard`, `git clean -fd`, comandos destrutivos nem sobrescreva arquivos significativos sem primeiro analisar e integrar seu conteúdo.
7. Não publique releases, não faça push, não abra PRs, não envie dados para serviços externos e não rode binários proprietários sem autorização explícita.
8. Não interrompa o trabalho por decisões triviais: escolha padrões técnicos sólidos e documente-os como ADRs. Pergunte somente quando for indispensável autorização humana, hardware ou uma decisão com impacto irreversível.
9. Ao terminar, informe os arquivos implementados, o que realmente funciona, os testes realizados, limitações e os passos exatos para executar o projeto.

**Definição de sucesso:** a aplicação inicia com dados simulados, o daemon pode executar e responder à interface via IPC, o SDK oferece contratos testados, existe descoberta HID real quando o ambiente permite, há ao menos um driver de exemplo seguro, o DRA possui fluxo de investigação reproduzível com fixtures e as skills/documentações estão prontas para orientar novos contribuidores. **Não afirme compatibilidade física com dispositivos não testados.**

---

## 1. Visão do produto

O Peripheral Hub é uma aplicação desktop de código aberto para reunir funções normalmente restritas a programas proprietários dos fabricantes:

- Detecção e identificação de teclados, mouses e, futuramente, outros periféricos.
- Controle RGB (cores, brilho, zonas e efeitos conforme capacidade do dispositivo).
- Controle de displays integrados em periféricos compatíveis.
- Remapeamento de botões e macros, quando tecnicamente suportados.
- Perfis persistentes e restauração de configurações.
- Catálogo comunitário de dispositivos suportados.
- Ferramentas de pesquisa e documentação para permitir a contribuição de novos drivers.

**Princípio de produto:** o usuário vê somente as funções realmente disponíveis para seu periférico. Suporte parcial é válido; capacidades devem ser declaradas e validadas individualmente.

**Cenário comunitário prioritário:** um contribuidor possui um teclado ainda incompatível, oferece o instalador/`.exe` original obtido legalmente e conecta o periférico ao próprio computador. Seu agente, orientado pelas skills, analisa o programa com ferramentas opcionais como REA, obtém capturas de tráfego USB/HID por meio do DRA, documenta as evidências, implementa um driver e prepara os artefatos para revisão humana. O `.exe` **não basta necessariamente** para descobrir todo o protocolo.

---

## 2. Stack e decisões obrigatórias

| Área | Escolha |
|---|---|
| Linguagem principal | **Rust estável** |
| Desktop | **Tauri 2 + React + TypeScript + Vite** |
| Pacotes JS | **pnpm** |
| Workspace Rust | **Cargo workspace** |
| Plataforma prioritária | **Windows 10/11**, sem acoplamento desnecessário às APIs do Windows |
| Futuro | Linux e macOS, com implementações de transporte separadas |
| Comunicação com HID | **hidapi** com abstração própria de transporte |
| Processamento assíncrono | Tokio onde agregar valor; evitar async artificial |
| IPC desktop ↔ daemon | Protocolo local, autenticado/restrito ao usuário, com versão explícita |
| Persistência | SQLite para estado da aplicação; JSON versionado para exportação/importação de perfis |
| Manifestos de dispositivos | TOML validado contra modelos tipados e verificações semânticas |
| Schemas de intercâmbio | JSON Schema para manifestos, perfis e relatórios de pesquisa |
| Drivers | Declarativos para casos simples; crates Rust compiladas para casos complexos |
| Engenharia reversa | REA opcional; DRA obrigatório como ferramenta do projeto |
| IA | `AGENTS.md` + `.agents/skills/<nome>/SKILL.md` |
| Documentação | Inglês como referência técnica; português brasileiro para guias públicos prioritários |
| Licença | **Apache-2.0 proposta**, a confirmar com o mantenedor antes de publicar oficialmente |

**Regras adicionais:**

- Não introduza Electron, Python como runtime da aplicação, nem múltiplos frameworks frontend. Python é permitido apenas em ferramentas auxiliares de laboratório se houver benefício real.
- Não implemente plugins nativos arbitrários carregados em runtime no MVP. Drivers Rust entram no build do projeto e passam por revisão.
- Drivers declarativos só executam operações previamente modeladas, sob políticas de segurança e limites explícitos.
- O daemon deve funcionar na sessão do usuário; não assumir serviço privilegiado do sistema. Instalação de drivers do sistema, captura que requer elevação e funcionalidades avançadas devem ser opcionais e bem delimitadas.
- Não tente fazer *firmware flashing*, escrita em memória permanente não documentada, desbloqueio ou comandos destrutivos.

---

## 3. Estrutura-alvo de repositório

**Os caminhos abaixo definem responsabilidades.** Antes de criar qualquer coisa, respeite o que já existe e adapte os manifests para que os diretórios resultem em um workspace válido. Diretórios sem implementação imediata podem permanecer, mas não crie código fictício apenas para preencher a árvore.

```text
peripheral-hub/
├── .github/
│   ├── workflows/
│   │   ├── ci-rust.yml
│   │   ├── ci-desktop.yml
│   │   ├── validate-devices.yml
│   │   ├── security.yml
│   │   └── release.yml
│   ├── ISSUE_TEMPLATE/
│   │   ├── bug-report.yml
│   │   ├── device-request.yml
│   │   └── feature-request.yml
│   ├── PULL_REQUEST_TEMPLATE.md
│   └── CODEOWNERS
├── .agents/
│   └── skills/
│       ├── add-device/SKILL.md
│       ├── reverse-engineer/SKILL.md
│       ├── capture-protocol/SKILL.md
│       ├── develop-driver/SKILL.md
│       ├── validate-device/SKILL.md
│       ├── develop-feature/SKILL.md
│       └── review-contribution/SKILL.md
├── apps/
│   ├── desktop/
│   │   ├── src/
│   │   │   ├── components/
│   │   │   ├── features/{devices,lighting,displays,macros,profiles}/
│   │   │   ├── hooks/
│   │   │   ├── services/
│   │   │   ├── stores/
│   │   │   └── types/
│   │   ├── src-tauri/
│   │   │   ├── src/
│   │   │   ├── capabilities/
│   │   │   ├── Cargo.toml
│   │   │   └── tauri.conf.json
│   │   └── package.json
│   ├── daemon/{src,Cargo.toml}
│   └── device-research-assistant/{src,Cargo.toml}
├── crates/
│   ├── core/
│   ├── device-sdk/
│   ├── device-registry/
│   ├── driver-declarative/
│   ├── transport-hid/
│   ├── transport-usb/
│   ├── lighting/
│   ├── displays/
│   ├── macros/
│   ├── profiles/
│   ├── ipc/
│   ├── storage/
│   └── research/
│       ├── device-discovery/
│       ├── capture/
│       ├── packet-analysis/
│       ├── experiment-runner/
│       └── report-generator/
├── devices/
│   └── <fabricante>/<modelo>/
│       ├── manifest.toml
│       ├── protocol.md
│       ├── validation.md
│       ├── Cargo.toml             # apenas driver nativo
│       ├── src/                   # apenas driver nativo
│       ├── tests/
│       └── fixtures/
├── schemas/
│   ├── device-manifest.schema.json
│   ├── profile.schema.json
│   └── research-report.schema.json
├── docs/
│   ├── en/{users,contributors}/
│   ├── pt-BR/{users,contributors}/
│   ├── architecture/
│   │   ├── overview.md
│   │   ├── device-lifecycle.md
│   │   ├── driver-system.md
│   │   ├── ipc.md
│   │   ├── security.md
│   │   ├── implementation-status.md
│   │   └── decisions/
│   ├── reverse-engineering/
│   └── protocols/
├── examples/{declarative-driver,native-driver,research-session}/
├── tools/{scaffold-device,validate-driver,generate-compatibility}/
├── config/
│   ├── agents/rea-mcp.example.json
│   └── development/
├── tests/{integration,hardware,fixtures}/
├── assets/{icons,branding}/
├── AGENTS.md
├── README.md
├── README.pt-BR.md
├── CONTRIBUTING.md
├── CODE_OF_CONDUCT.md
├── SECURITY.md
├── CHANGELOG.md
├── LICENSE                       # só após confirmar licença
├── Cargo.toml
├── Cargo.lock
├── package.json
├── pnpm-workspace.yaml
├── pnpm-lock.yaml
├── rust-toolchain.toml
├── .editorconfig
├── .gitignore
└── .gitattributes
```

**Nota técnica:** a árvore é conceitual. Expressões com `{...}` e `<...>` denotam diretórios, não nomes literais. Para Cargo, use apenas crates efetivamente implementadas como membros do workspace. Crates do Tauri devem ser integradas corretamente. Não adicione um `Cargo.toml` de driver sem código correspondente.

---

## 4. Contratos arquiteturais obrigatórios

### 4.1 Fluxo de execução

```text
Desktop (React/TypeScript)
  └── Tauri (API restrita)
       └── cliente IPC tipado
            └── Daemon Rust (sessão do usuário)
                 ├── Core / estado / perfis
                 ├── Device Registry + Device SDK
                 │    ├── driver declarativo validado
                 │    └── driver Rust compilado
                 └── transportes HID / USB
                      └── periférico real
```

O **DRA é independente** do aplicativo distribuído; não faz parte desse fluxo normal. Ele gera documentação, fixtures e drivers para inclusão no repositório.

### 4.2 Device SDK

Implemente contratos Rust tipados e documentados, pelo menos para:

- `DeviceDescriptor`: identidade observada do hardware (VID, PID, fabricante, produto, serial quando disponível, caminho/interface, usage page, usage, número da interface, transporte e versão de firmware quando conhecida).
- `DeviceId`/`DeviceInstanceId`: diferencia modelo de instância física; não use apenas VID/PID como chave única.
- `Capabilities`: capacidades detalhadas e opcionais de iluminação (zonas, brilho, efeitos), display (resolução/formato), macros/botões, perfis, etc.
- `DeviceDriver` ou interfaces equivalentes: identificação de compatibilidade, abertura/fechamento, consulta de capacidades, leitura de estado e execução de comandos suportados.
- `DeviceError`: erros explícitos como `NotSupported`, `NotConnected`, `PermissionDenied`, `InvalidArgument`, `Timeout`, `TransportError` e `ProtocolError`.
- `Transport`: abstração testável de entrada/saída com operações adequadas a relatórios HID (`input`, `output`, `feature`) e respectivos limites; não exponha acesso USB irrestrito à interface.
- `DriverRegistry`: escolhe driver com base em VID/PID **e** interface, usage, identificadores adicionais e condições de identificação; lida com ambiguidades sem selecionar incorretamente um driver.

Use traits e DTOs que permitam testes sem hardware. Respostas a operações ausentes devem usar `NotSupported`, não sucesso fictício. Declare e documente regras de versionamento dos contratos.

### 4.3 Descoberta e conexões

- Utilize `hidapi` encapsulada em `crates/transport-hid`.
- Separe descoberta de identificação do driver, abertura do dispositivo e execução de comandos.
- Trate interfaces HID múltiplas, hotplug/reconexão, autorização, desconexões e falhas sem derrubar o daemon.
- Evite enviar pacotes arbitrários durante a enumeração: descoberta deve ser essencialmente não destrutiva.
- Defina um transporte fake/in-memory para CI e testes de integração.
- `transport-usb` deve fornecer contratos adequados e, se não houver backend real seguro na fase inicial, documentar claramente a limitação em vez de simulá-lo como funcional.

### 4.4 Drivers nativos e declarativos

- Nativos: crates Rust internas, compiladas com o aplicativo, registradas explicitamente. Nada de DLLs de terceiros carregadas automaticamente.
- Declarativos: manifesto TOML + comandos permitidos pelo interpretador `driver-declarative`; schema, versões, limites de bytes, validação de parâmetros, timeouts e allowlist de tipos de operação.
- Por padrão, **proibir** firmware update, bootloader, gravação persistente desconhecida, execução de código, comandos arbitrários vindos do frontend e escritas não documentadas.
- Não trate dados de um manifesto da comunidade como código confiável. Se o interpretador não for seguro o suficiente, limite o MVP a carregamento de manifestos e pré-visualização, com execução desativada.
- Use um exemplo de fabricante/modelo **explicitamente fictício** em `examples/` para demonstrar o SDK. Identificadores fictícios jamais devem aparecer como dispositivos fisicamente validados.

### 4.5 IPC e daemon

- Crie protocolo local **versionado**, DTOs compartilhados via `crates/ipc` e operações como `health`, `list_devices`, `get_device`, `get_capabilities`, `get_profile`, `save_profile`, `set_lighting` e status/eventos de conexão.
- Escolha um transporte IPC coerente: **named pipe no Windows com acesso limitado ao mesmo usuário**; para desenvolvimento multiplataforma, um backend Unix domain socket ou outra alternativa local explicitamente protegida. Documente a estratégia e não faça bind em rede pública.
- Evite instâncias duplicadas do daemon; trate encerramento e reconexão do cliente, timeouts e erros estruturados.
- Não conceda automaticamente privilégios administrativos ao daemon.
- Permita executar daemon e interface separadamente em desenvolvimento, com instruções reproduzíveis.

### 4.6 Recursos de domínio

- `lighting`: cor, brilho, zona e efeito com validação de suporte por dispositivo; operações aplicadas devem refletir o estado real ou informar falha.
- `displays`: contrato para imagens/frames, resolução, formato, limites de taxa; não enviar conteúdo para tela física sem protocolo validado.
- `macros`: modelo de ações, atalhos, delays, perfis e limites de execução; no MVP, implemente edição/validação/persistência e simulação segura. Injeção de teclas e hooks globais só em etapa explicitamente autorizada e testada.
- `profiles`: persistência local, esquema versionado, importação/exportação JSON e migração de dados.
- `storage`: SQLite com migrations; nunca armazene executáveis do fabricante ou capturas sensíveis no banco público do projeto.

---

## 5. Device Research Assistant (DRA)

### 5.1 Objetivo e limites

Implemente `apps/device-research-assistant` como CLI Rust utilizável por humanos e agentes. O DRA deve ajudar a descobrir protocolos proprietários **sem prometer resultados impossíveis**. Análise estática de `.exe` e captura dinâmica são complementares.

O fluxo é:

```text
Software original (.exe) + dispositivo conectado
       ├── análise estática opcional via REA / outras ferramentas autorizadas
       └── experimentos controlados no software original
               └── USBPcap/Wireshark ou outro backend compatível
                       └── DRA: importar, correlacionar, comparar e classificar
                              └── relatório de evidências
                                     └── skill develop-driver
                                            └── driver + testes + PR
```

### 5.2 Funcionalidades implementáveis sem hardware

Construa primeiro um pipeline reproduzível com arquivos de exemplo **sintéticos**:

1. Enumerar interfaces HID quando o sistema permitir e exportar inventário estruturado.
2. Criar uma `research session` local (identificador, data, dispositivo, firmware se conhecido, software original, hash SHA-256, objetivo e consentimento).
3. Registrar ações guiadas: baseline, mudar RGB para vermelho/verde/azul, alterar brilho etc. Os experimentos precisam ser **descritivos**, não execução automática de comandos de hardware desconhecidos.
4. Importar capturas `.pcapng`/`.pcap` quando houver parser comprovado ou usar exportação estruturada do TShark; documentar dependências. Uma importação JSON própria e estável deve funcionar na CI sem USBPcap.
5. Normalizar eventos: timestamp, direção, transferência/relatório, interface, Report ID, payload, origem e ação correlacionada.
6. Comparar sequências, agrupar pacotes repetidos, destacar offsets que variam e gerar hipóteses com **graus de confiança** (`confirmed-on-hardware`, `inferred`, `unverified`).
7. Exportar relatório Markdown + JSON validado por schema, sem arquivos proprietários e com sanitização.
8. Gerar um scaffold de driver/manifesto/testes a partir de dados verificados, sem declarar capacidades não confirmadas.

Proponha CLI coerente (ajuste comandos à implementação real), por exemplo:

```powershell
cargo run -p device-research-assistant -- devices list
cargo run -p device-research-assistant -- session create --name keyboard-rgb
cargo run -p device-research-assistant -- capture import --input capture.json --session <id>
cargo run -p device-research-assistant -- analyze --session <id>
cargo run -p device-research-assistant -- report --session <id>
```

Não apresente comandos ainda não implementados como funcionais. Ofereça `--help` útil, erros claros e documentação testável.

### 5.3 Integração com REA e captura externa

- REA: **https://github.com/morluto/rea**. Use como ferramenta **opcional de desenvolvimento**, não faça fork, não copie código sem necessidade e não o inclua no runtime final.
- Consulte a documentação atual do REA e configure MCP do OpenCode **somente se houver instalação e comando de inicialização confirmados**. Não invente comandos ou URLs de serviços locais.
- Forneça `config/agents/rea-mcp.example.json` como **exemplo comentado/documentado**, desabilitado e com placeholders inequívocos se o método exato depender do ambiente. Nunca inclua tokens.
- No Windows, documente uso do USBPcap + Wireshark/TShark para tráfego USB, inclusive a necessidade eventual de permissões elevadas. O MVP deve ser capaz de **importar** evidências mesmo que não consiga iniciar o capturador automaticamente.
- Não executar `.exe` recebido automaticamente. Para análise dinâmica, exija origem/autorização e recomende máquina/VM de laboratório; conexão física do periférico pode ser necessária, o que limita o uso de VM.
- Não envie pacotes desconhecidos ao dispositivo para “testar” hipóteses. Diferencie comandos voláteis comprovadamente seguros de ações persistentes ou potencialmente destrutivas.
- Todos os dados brutos de investigação devem permanecer fora do Git e dos artefatos de distribuição, em diretório de dados local ignorado pelo Git.

### 5.4 Artefatos de uma contribuição de driver

Uma investigação bem-sucedida deve produzir:

```text
devices/<fabricante>/<modelo>/
├── manifest.toml
├── protocol.md          # tabelas de comandos + evidências e incertezas
├── validation.md        # SO, firmware, hardware, testes e limites
├── tests/
├── fixtures/            # sintéticas/sanitizadas
└── (Cargo.toml + src/...) somente se for driver Rust
```

VID/PID são **identificadores**, não especificação de protocolo. Investigue também interfaces, endpoints, Report IDs, inicializações, checksums, sequenciamento, ACKs, limites e diferenças entre revisões de firmware. Dispositivos compostos e dongles podem exigir tratamento especial.

---

## 6. Interface de desktop — MVP real

Produza uma interface React/Tauri 2 limpa e funcional, com os seguintes estados:

- **Dispositivos:** lista de dispositivos descobertos, status de conexão, fabricante/modelo, funcionalidades declaradas e nível de validação.
- **Detalhes do dispositivo:** capacidades, informações técnicas úteis e avisos de suporte parcial.
- **Iluminação:** seletor de cor/brilho/efeito apenas quando suportado; mensagens de sucesso/erro vindas do daemon. No modo simulado, sinalize claramente que a operação é simulada.
- **Perfis:** criar, renomear, editar, salvar, aplicar, importar e exportar conforme as operações efetivamente implementadas.
- **Macros:** editor e visualização/simulação segura de ações; não simular que um remapeamento foi aplicado ao hardware se não foi.
- **Displays:** painel de informações e contrato/preview; controles efetivos apenas em driver com suporte validado.
- **Diagnóstico:** versão do daemon/API, último erro, estado do transporte, instruções para DRA e como solicitar um novo dispositivo.

Requisitos de UX: estados de loading, vazio, desconectado, permissão negada, erro e recurso não suportado; validação de formulários; navegação acessível; sem requisições arbitrárias de rede. O frontend não conversa diretamente com USB/HID.

---

## 7. OpenCode, AGENTS.md e skills

O OpenCode reconhece instruções de repositório em `AGENTS.md` e skills em `.agents/skills/<nome>/SKILL.md`. Preserve esse diretório existente. Cada skill precisa começar com YAML frontmatter contendo `name` igual ao nome do diretório e `description` específica; outras opções são facultativas. Escreva as skills **de forma executável, com entradas, ferramentas, passos, artefatos e critérios de parada** — nunca apenas uma descrição genérica.

`AGENTS.md` deverá ser curto e obrigatório para qualquer agente: arquitetura, stack, comandos reais de build/test, convenções, proibição de inventar testes, regras de hardware, privacidade, documentação, uso das skills e ordem de validação.

Implemente as skills:

| Skill | Responsabilidade mínima |
|---|---|
| `add-device` | Orquestrar identificação → evidências → implementação → testes → documentação → proposta de PR. |
| `reverse-engineer` | Inspecionar legalmente `.exe` e dependências, usar REA quando disponível, separar fatos de hipóteses. |
| `capture-protocol` | Conduzir sessões DRA, orientar USBPcap/TShark, correlacionar ações e pacotes. |
| `develop-driver` | Escolher driver declarativo ou Rust, seguir o SDK, preencher manifesto e criar testes. |
| `validate-device` | Validar manifesto/schema, testes de software, hardware físico, firmware e matriz de capacidades. |
| `develop-feature` | Implementar mudanças em crates/desktop/daemon respeitando fronteiras arquiteturais. |
| `review-contribution` | Revisar segurança, compatibilidade, documentação, licenciamento, CI e evidências. |

Exemplo do formato inicial de `SKILL.md` (adapte para cada nome):

```yaml
---
name: add-device
description: Orienta a investigação, implementação, teste e documentação de suporte a um novo periférico no Peripheral Hub.
compatibility: opencode
---
```

As skills podem ter `references/` e `scripts/` apenas quando acrescentarem material útil. Links internos devem resolver corretamente. Não faça uma skill executar comandos de firmware/destrutivos sem aprovação humana específica.

Use `opencode.json` somente quando necessário; **não substitua configurações existentes**. A integração MCP para REA deverá ser opcional e ativada pelo contribuidor, com procedimento de instalação testado e instruções atualizadas.

Referências oficiais:

- OpenCode Skills: https://docs.opencode.ai/docs/skills/
- OpenCode Rules: https://docs.opencode.ai/docs/rules/
- OpenCode MCP: https://docs.opencode.ai/docs/mcp-servers/
- Tauri 2: https://v2.tauri.app/
- REA: https://github.com/morluto/rea

---

## 8. Documentação e padrões comunitários

Crie conteúdo substancial, sem textos genéricos ou links quebrados, para:

- `README.md`: proposta de valor, escopo real, compatibilidade, arquitetura curta, status do MVP, como compilar e contribuir.
- `README.pt-BR.md`: equivalente em pt-BR.
- `CONTRIBUTING.md`: ambiente Rust/Node/pnpm, padrões, drivers, DRA, testes, PR, regras para evidências proprietárias e revisão.
- `SECURITY.md`: reporte privado de vulnerabilidades, tratamento de capturas, isolamento de executáveis, consentimento e operações perigosas.
- `CODE_OF_CONDUCT.md`: política comunitária clara e aplicável.
- `docs/architecture/*`: diagramas textuais, invariantes entre camadas, contrato SDK, ciclo de vida, IPC, segurança e ADRs.
- `docs/reverse-engineering/*`: procedimentos de análise estática, coleta USB/HID, laboratório, sanitização e limitações.
- `docs/en/users/` e `docs/pt-BR/users/`: instalação, uso, perfis, dispositivos compatíveis e troubleshooting.
- `docs/en/contributors/` e `docs/pt-BR/contributors/`: como adicionar um dispositivo com e sem IA.
- `docs/protocols/`: formato de especificações, sem dumping de material proprietário.
- `CHANGELOG.md`: versão inicial e convenção de alterações.

Mantenha documentação técnica de referência em inglês e guias principais em português e inglês. Se uma tradução não for concluída, sinalize esse fato no status, em vez de apresentar arquivos vazios como concluídos.

GitHub: configure modelos de issues (bug, solicitação de suporte a dispositivo, feature), PR template, CODEOWNERS **apenas se os responsáveis reais forem conhecidos**, e workflows de CI para Rust, desktop, validação de manifestos e segurança. Não invente usuários em CODEOWNERS. Releases Windows assinadas e publicação automática ficam preparadas/documentadas, mas **não ative** publicação nem use certificados inexistentes.

Licença: antes de publicar `LICENSE` como definitiva, confirme a escolha Apache-2.0 com o mantenedor. Se não houver confirmação, registre a pendência de forma explícita e **não afirme** que o projeto já está licenciado.

---

## 9. Segurança, privacidade e engenharia reversa responsável

Estas regras prevalecem sobre a conveniência:

1. Analisar apenas software e hardware próprios ou para os quais o pesquisador possua autorização, observando legislação e licenças aplicáveis.
2. Não distribuir instaladores, `.exe`, DLLs proprietárias, firmware, chaves, capturas completas ou conteúdo protegido de terceiros.
3. Manter capturas e binários em armazenamento local ignorado por Git; exigir revisão/sanitização antes de exportar amostras.
4. Não exfiltrar telemetria, dados pessoais, teclas digitadas, serial identificável ou caminhos privados. Logs devem evitar bytes sensíveis por padrão.
5. Não executar amostras desconhecidas automaticamente. Análise dinâmica requer autorização e ambiente adequado.
6. Capturas USB podem conter tráfego de outros dispositivos; orientar filtro por dispositivo/interface e informar riscos de privacidade.
7. Desabilitar escrita de firmware, bootloader, EEPROM, comandos desconhecidos e outras operações perigosas até haver projeto específico e aprovação explícita.
8. Driver ou manifesto externo é entrada não confiável; validar tamanhos, tipos, limites de tempo e conteúdo antes de qualquer operação.
9. Não tratar análise estática como prova de funcionalidade física. Diferenciar `simulated`, `inferred`, `tested`, `validated-on-hardware`.
10. Preferir menor privilégio, IPC restrito ao usuário, ausência de serviços expostos na rede e dependências auditáveis.

---

## 10. Testes e automação exigidos

Implemente um conjunto útil de testes desde o início:

- **Rust unitários:** manifesto, capabilities, match de drivers, parser e serialização, limites e erros, formatos RGB, perfis e IPC.
- **Integração sem hardware:** fake transport → fake driver → registry → daemon → cliente IPC; cenários de desconexão, `NotSupported`, timeout e permissões.
- **DRA:** fixtures sintéticas com RGB vermelho/verde/azul; diferenças de bytes; correlação ação/pacote; exportação de relatório; sanitização.
- **Frontend:** testes das views e de estados reais/simulados, chamadas ao cliente IPC com mock apropriado; build TypeScript.
- **Schemas:** manifestos e relatórios inválidos devem falhar com mensagem clara.
- **Hardware:** testes manuais/opt-in, nunca executados automaticamente em CI, com relatório de plataforma, firmware e dispositivo.
- **CI:** `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` (ajuste explicitamente se alguma target for incompatível), `cargo test --workspace`, `pnpm install --frozen-lockfile`, lint/test/build frontend e validação de schemas, executados nas plataformas compatíveis. Não configure jobs impossíveis sem comentários ou condições de plataforma.

Comandos deverão funcionar no projeto real (ajuste os nomes dos pacotes após inspecionar manifests). Produza scripts consistentes na raiz, como `pnpm dev`, `pnpm build`, `pnpm test`, `pnpm lint` quando forem apropriados. Documente os comandos exatos para iniciar daemon, desktop e DRA.

---

## 11. Ordem de implementação — execute sem pular verificações

### Fase 0 — Auditoria e baseline

- Inspecionar arquivos já existentes, Git, ferramentas e dependências.
- Criar/atualizar `docs/architecture/implementation-status.md` com checklist e observações.
- Não apagar trabalho existente.

**Aceite:** mapa real do projeto, decisões registradas e plano verificável.

### Fase 1 — Fundação e workspaces

- Configurar Cargo workspace e pnpm workspace consistentes.
- Implementar Tauri 2 + React/TypeScript mínimo, com build reproduzível.
- Adicionar formatação, lint, .gitignore seguro, configs e documentação de setup.
- Criar ADRs para processo daemon, IPC, dois tipos de drivers e DRA desacoplado.

**Aceite:** builds mínimos da aplicação e crates existentes; instruções reais para execução.

### Fase 2 — Device SDK, transportes e registry

- Contratos tipados, erros, capacidades, DeviceInstanceId e matcher sem ambiguidade.
- Adaptador HID `hidapi` e fake transport.
- Exemplo nativo fictício ou mock com testes; manifesto declarativo e validator.
- Validação de schema/semântica de identidade, interface e funcionalidades.

**Aceite:** CI testa descoberta simulada, registro, erros e uma ação RGB mockada; enumeração HID real se disponível, sem escrita em hardware.

### Fase 3 — Daemon, IPC e perfis

- Daemon user-mode, protocolo versionado, health, listagem, capacidades e eventos/status.
- SQLite com migrações e perfis JSON com versão.
- Cliente IPC compartilhado com frontend, recuperação de desconexão e mensagens de erro.

**Aceite:** daemon executa, cliente responde, perfis são persistidos e testes de integração passam.

### Fase 4 — Interface de usuário

- Telas de dispositivos, detalhes, iluminação, perfis, macros, displays e diagnóstico.
- Conectar efetivamente ao daemon para funções já implementadas.
- Estados sem dispositivo, simulado, indisponível e erro; nunca insinuar controle físico inexistente.

**Aceite:** desktop inicia e opera com driver simulado e daemon real, inclusive perfis.

### Fase 5 — Device Research Assistant

- CLI e modelos de sessões, importação de JSON de captura, parser/normalização, comparação e relatório.
- Fixtures sintéticas determinísticas, workflow guiado e geração de scaffold.
- Documentar USBPcap/Wireshark/TShark e integração opcional REA.
- Se o ambiente permitir, suporte opcional a importação pcapng ou export TShark; não bloquear MVP se dependências externas estiverem ausentes.

**Aceite:** uma sessão sintética produz relatório e scaffold reproducíveis e testados, sem hardware.

### Fase 6 — Skills, documentação e comunidade

- Preencher `AGENTS.md` e as sete skills com frontmatter válido e procedimentos reais.
- Produzir READMEs, CONTRIBUTING, SECURITY, arquitetura, guias e templates GitHub.
- Configurar CI coerente, validar links relativos e documentar licença pendente.

**Aceite:** novo contribuidor consegue seguir a documentação, reproduzir os testes e criar um driver de exemplo.

### Fase 7 — Revisão integrada e preparo de release

- Executar fmt, clippy, testes, lint e build; corrigir falhas.
- Revisar dependências, permissões Tauri, superfície do IPC, logs e arquivos ignorados.
- Atualizar matriz de capacidades: simulado / inferido / testado / validado no hardware.
- Documentar instalação e empacotamento Windows que realmente funcionarem; deixar assinatura/publicação para autorização futura.

**Aceite:** relatório final honesto e verificável, sem claims de compatibilidade física não demonstrada.

---

## 12. Restrições de escopo e definição de pronto

**Não confunda estrutura completa com funcionalidades físicas prontas.** A arquitetura inteira deve estar preparada, mas RGB real, displays e macros dependem de protocolos efetivamente descobertos e de hardware. Um simulador funcional e um driver de exemplo completo são preferíveis a suporte “fake” para dezenas de modelos.

**Proibido como entrega considerada pronta:**

- Apenas gerar diretórios/arquivos vazios.
- Registrar `TODO` como se fosse implementação.
- Fazer interface bonita sem backend e declarar integração concluída.
- Criar `hidapi` wrapper que devolve sucesso sem enviar/receber dados quando diz operar fisicamente.
- Colocar `.exe` ou dumps USB brutos no Git.
- Inventar VID/PID de periféricos reais, comandos e capturas.
- Usar REA como dependência obrigatória do instalador final.
- Executar comandos desconhecidos em dispositivos reais por conta própria.
- Declarar sucesso de testes que não rodaram.

**Entrega mínima objetiva:**

- [ ] Cargo + pnpm workspaces válidos.
- [ ] Tauri 2/React/TS compila e abre quando dependências de GUI estiverem disponíveis.
- [ ] Device SDK e Registry testados.
- [ ] HID enumerável em ambiente compatível + transporte fake para CI.
- [ ] Driver mock e manifesto de exemplo com validação.
- [ ] Daemon Rust e protocolo IPC com integração real ao frontend.
- [ ] Persistência de perfis e importação/exportação versionada.
- [ ] Telas principais, estados de erro e capabilities dinâmicas.
- [ ] DRA CLI processa fixture, compara pacotes, gera relatório e scaffold.
- [ ] Skills OpenCode, `AGENTS.md` e documentação operacional preenchidos.
- [ ] Testes automatizados e CI; status dos comandos registrado.
- [ ] Limitações físicas, plataformas não testadas e licença pendente documentadas.

---

## 13. Formato do relatório final do agente

Ao concluir sua sessão, responda obrigatoriamente com:

1. **Resumo do que foi implementado de verdade** (não apenas arquivos criados).
2. **Arquitetura resultante** e principais contratos criados.
3. **Comandos exatos para instalar, executar, testar e compilar** daemon, desktop e DRA.
4. **Testes executados:** comando, resultado e motivo de qualquer teste não executado.
5. **O que depende de hardware, de captura USB/HID, do REA ou de autorização humana.**
6. **Pendências ordenadas por prioridade**, com indicação explícita do que ficou parcial/bloqueado.
7. **Arquivos relevantes alterados** e decisões arquiteturais registradas.

**Instrução final:** você está no repositório existente do Peripheral Hub. **Comece agora pela Fase 0 e avance na ordem acima. Implemente o máximo possível com qualidade, testes e evidências. Não se limite a explicar como faria.**
