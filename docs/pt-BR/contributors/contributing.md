# Como contribuir

O [CONTRIBUTING.md em inglês](../../../CONTRIBUTING.md) é a referência. Esta tradução cobre as orientações públicas prioritárias da etapa de fundação.

## Escopo e ambiente

Ainda não há aplicação, SDK, DRA, schemas ou CI funcionais. São úteis contribuições de documentação, tradução, discussão arquitetural e pesquisa local autorizada. Os fluxos de drivers e funcionalidades descritos abaixo dependem de uma etapa futura de implementação.

A stack prevista é Rust estável/Cargo, Node.js compatível com as futuras dependências, pnpm e Tauri 2 + React/TypeScript/Vite. Windows 10/11 é prioritário; preserve a portabilidade para Linux/macOS. Versões e dependências específicas de plataforma serão registradas com os workspaces reais. Manifests e lockfiles estão vazios; não há `pnpm dev`, `cargo run` ou comando DRA funcional.

O checkout atual não tem Git inicializado. Quando houver repositório canônico, faça fork, clone e crie uma branch focada; não invente URLs ou responsáveis. Mantenha lockfiles reais, sem dependências instaladas ou produtos de build no Git.

Apache-2.0 é proposta pendente, e `LICENSE` está vazio. Confirme o licenciamento antes de integrar código substancial ou distribuir oficialmente. Contato privado de segurança e política de conduta/aplicação também precisam ser definidos. Mantenha interações respeitosas e construtivas, sem expor dados pessoais.

## Dispositivos e funcionalidades

Para hardware, siga [como adicionar um dispositivo](adding-a-device.md): identidade e interfaces → evidências autorizadas → driver quando o SDK existir → testes → documentação e revisão. VID/PID não bastam. Binários e capturas brutos ficam localmente em diretório ignorado; somente evidências necessárias, autorizadas e sanitizadas entram na contribuição.

Para funcionalidades, defina escopo e critérios verificáveis, identifique a camada responsável e registre decisões relevantes em ADRs. Preserve frontend → Tauri restrito → IPC tipado/versionado → daemon na sessão do usuário → SDK/drivers → transporte. DRA é independente; REA é opcional. Sem Electron, runtime Python, plugins nativos arbitrários, daemon elevado ou listener público.

Use erros e capacidades explícitos; operações ausentes retornam `NotSupported`. Cubra estados de carregamento, vazio, desconexão, permissão negada, erro, simulação e não suportado, com acessibilidade. Macros iniciais abrangem edição/validação/persistência/simulação, não injeção global. Persistência exige migrações e versões.

## Verificações

Nesta etapa, revise conteúdo, links relativos, frontmatter das skills, consistência de traduções e afirmações de status. Não execute manifests vazios como testes.

**Quando existirem workspaces e ferramentas reais**, a ordem será:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
pnpm install --frozen-lockfile
```

Depois, execute somente scripts lint/test/build realmente definidos e validadores implementados de schema/semântica. São verificações futuras, não instruções funcionais atuais. Registre resultados e bloqueios exatos. Testes significativos devem cobrir contratos, ambiguidades, limites, dados inválidos, desconexão, timeout, permissão, `NotSupported`, IPC, perfis e sanitização DRA. Hardware é opt-in, fora da CI automática, com SO/firmware/capacidade registrados.

## IA e pull requests

Leia [AGENTS.md](../../../AGENTS.md) e as [skills](../../../.agents/skills/README.md). O fluxo vale também para pessoas sem IA. Não envie amostras proprietárias a provedores externos. REA/MCP exige instalação e comandos de inicialização verificados; o exemplo vazio não é utilizável. Reinicie o OpenCode após alterações nas skills/configuração.

Quando houver Git remoto, prepare uma PR focada: explique problema/escopo, impacto arquitetural, critérios, origem dos artefatos e dependências, verificações executadas e pendências. Revise o diff e inclua somente arquivos pretendidos. Atualize documentação inglesa, guias pt-BR prioritários e status; registre traduções incompletas e limites de firmware/plataforma.

Solicite revisão humana. Findings sensíveis seguem a [política de segurança](security.md), não uma PR pública. Templates e workflows GitHub ainda são placeholders. Agentes não podem abrir PR, fazer push ou publicar sem autorização explícita.
