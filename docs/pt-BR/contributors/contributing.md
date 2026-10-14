# Como contribuir

O [CONTRIBUTING.md em inglês](../../../CONTRIBUTING.md) é a referência. Esta tradução cobre as orientações públicas prioritárias da etapa de fundação.

## Escopo e ambiente

A Fase 2 preserva SDK Rust, Registry e Core e acrescenta descoberta/transporte HID real, IPC protegido, daemon e simulação explícita. Leia os [contratos do SDK](../../architecture/sdk.md) e o [guia do daemon](../users/daemon.md). Desktop, drivers físicos, perfis, DRA, validadores de manifestos, schemas e CI continuam pendentes. Esta execução termina antes da Fase 3.

A stack prevista é Rust estável/Cargo, Node.js/pnpm e Tauri 2 + React/TypeScript/Vite. Rust 1.99.0 está fixado com rustfmt/Clippy, edition 2024 e `Cargo.lock`; instale rustup e linker nativo. Há seis pacotes reais no workspace. `cargo run -p peripheral-hub-daemon -- serve --simulate` inicia desenvolvimento seguro; `health` e `list` consultam o daemon em outro terminal. Manifests frontend/DRA continuam vazios. Windows 10/11 é prioritário; testes de software Linux e análise estática Windows não comprovam runtime Windows ou hardware.

O repositório canônico informado pelo mantenedor é [ExtremeCodeX/peripheral-hub](https://github.com/ExtremeCodeX/peripheral-hub), com branch padrão `main`. Faça fork, clone e crie uma branch focada. O checkout local agora possui histórico Git e remoto `origin`; a auditoria documental original ocorreu antes dessa configuração. Mantenha lockfiles reais, sem dependências instaladas ou produtos de build no Git.

O repositório existente inclui o texto Apache-2.0 em `LICENSE`, que deve ser preservado. A confirmação explícita exigida pelo documento mestre continua pendente antes de distribuição oficial. Contato privado de segurança e política de conduta/aplicação também precisam ser definidos. Mantenha interações respeitosas e construtivas, sem expor dados pessoais.

## Dispositivos e funcionalidades

Para hardware, siga [como adicionar um dispositivo](adding-a-device.md): identidade e interfaces → evidências autorizadas → driver quando transporte/validadores e escopo estiverem disponíveis → testes → documentação e revisão. VID/PID não bastam. Binários e capturas brutos ficam localmente em diretório ignorado; somente evidências necessárias, autorizadas e sanitizadas entram na contribuição.

Para funcionalidades, defina escopo e critérios verificáveis, identifique a camada responsável e registre decisões relevantes em ADRs. Preserve frontend → Tauri restrito → IPC tipado/versionado → daemon na sessão do usuário → SDK/drivers → transporte. DRA é independente; REA é opcional. Sem Electron, runtime Python, plugins nativos arbitrários, daemon elevado ou listener público.

Use erros e capacidades explícitos; operações ausentes retornam `NotSupported`. Cubra estados de carregamento, vazio, desconexão, permissão negada, erro, simulação e não suportado, com acessibilidade. Macros iniciais abrangem edição/validação/persistência/simulação, não injeção global. Persistência exige migrações e versões.

## Verificações

Nas bibliotecas Rust implementadas, execute na raiz os comandos abaixo. Revise também conteúdo, links relativos, frontmatter, traduções e afirmações de status. Não execute manifests vazios dos demais componentes como testes.

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo check --workspace
cargo test --workspace
cargo build --workspace --locked
```

Frontend futuro requer `pnpm install --frozen-lockfile` e somente scripts definidos; validação de manifestos/schemas requer validadores reais. Esses componentes ainda são placeholders. Registre resultados e bloqueios exatos. Testes significativos devem cobrir contratos, ambiguidades, limites, dados inválidos, desconexão, timeout, permissão e `NotSupported`; IPC, perfis e sanitização DRA serão testados quando implementados. Hardware é opt-in, fora da CI automática, com SO/firmware/capacidade registrados.

## IA e pull requests

Leia [AGENTS.md](../../../AGENTS.md) e as [skills](../../../.agents/skills/README.md). O fluxo vale também para pessoas sem IA. Não envie amostras proprietárias a provedores externos. REA/MCP exige instalação e comandos de inicialização verificados; o exemplo vazio não é utilizável. Reinicie o OpenCode após alterações nas skills/configuração.

Quando houver Git remoto, prepare uma PR focada: explique problema/escopo, impacto arquitetural, critérios, origem dos artefatos e dependências, verificações executadas e pendências. Revise o diff e inclua somente arquivos pretendidos. Atualize documentação inglesa, guias pt-BR prioritários e status; registre traduções incompletas e limites de firmware/plataforma.

Solicite revisão humana. Findings sensíveis seguem a [política de segurança](security.md), não uma PR pública. Templates e workflows GitHub ainda são placeholders. Agentes não podem abrir PR, fazer push ou publicar sem autorização explícita.
