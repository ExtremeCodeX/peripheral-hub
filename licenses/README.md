# Licenças e atribuições — escopo efetivo do candidato Linux

`INDEX.json` associa pacote/versão, origem, licença declarada, escopo e textos
preservados em `texts/<sha256>.txt`. Textos idênticos compartilham arquivo; as
atribuições/origens continuam separadas por pacote. O núcleo próprio usa
Apache-2.0 conforme `../LICENSE`; as dependências preservam suas licenças.

## Evidência e limites

- Cargo metadata offline, locked, `--filter-platform x86_64-unknown-linux-gnu`,
  features padrão mais `custom-protocol` para embedding Tauri de produção.
  Arquivos `.crate` selecionados conferidos pelo SHA-256
  do lock; manifestos e textos reais lidos sem executar código ou build scripts.
- npm: manifestos instalados e tarballs em cache conferidos pelo SRI do lock.
  Textos instalados comparados aos arquivos do tarball. Build e test são closures
  distintas dos seeds de ferramentas; runtime-bundle vem dos módulos observados
  numa compilação Vite de produção com `write:false`. Não houve instalação.
- O grafo conservador não substitui análise do ELF/bundle final nem prova que
  cada pacote inteiro seja redistribuído. Build generators podem emitir expressão
  no resultado; seus avisos foram conservadoramente preservados também.
- `inactiveCargoLockEntries` e escopo `inactive-optional-platform` não são
  dependências runtime desse candidato. Falta de cache delas não é falta de
  licença; rever seleção ao trocar plataforma, features ou formato de pacote.
- `externalSystemRuntime` contém metadados versionados do host, não uma auditoria
  arquivo a arquivo de GTK/WebKit ou promessa de suporte a essa versão mínima.
  Não há runtimes de sistema copiados nesta fonte.

## Obrigações a levar ao artefato concreto

| Família | Tratamento necessário, conforme texto efetivo |
|---|---|
| MIT/ISC/BSD | Preservar permissões, copyrights, disclaimers e condições; não endossar com nomes de titulares. Verificar a variante exata. |
| Apache-2.0 | Preservar licença e NOTICE aplicável, avisos e modificações; observar patentes/marcas. Não converter AND em OR nem relicenciar terceiros. |
| MPL-2.0 | Preservar avisos de arquivos cobertos, disponibilizar Source Code Form conforme a licença quando distribuir executáveis, incluindo alterações se existirem. Não implica licença global MPL. |
| Unicode-3.0 | Preservar licença e aviso de copyright dos dados/código fornecidos. Cláusula Unicode adicional não desaparece ao optar por MIT/Apache onde há AND. |
| Zlib/0BSD/Unlicense/CC0/MIT-0 | Seguir as condições específicas e a expressão composta registrada, sem presumir um único conjunto de termos para todos os pacotes. |
| OFL-1.1 (Outfit) | Preservar aviso/licença; observar reserved font names/modificações e condições de redistribuição. Fonte incluída continua com OFL própria. |
| LGPL/outros componentes externos de sistema | Respeitar os componentes efetivamente ligados/redistribuídos; se embutir bibliotecas/runtimes, rever fontes, relink/substituição e avisos conforme cada licença e forma de ligação. Não aplicar licenças de utilitários do pacote a todo o aplicativo por associação. |

Fontes oficiais dos crates são referenciadas por versão/URL/hash. Isso permite
identificar as fontes, mas **não afirma que só um link já cumpra todas as obrigações
de uma futura distribuição**. Antes de distribuir binários, conferir acesso real
às fontes cobertas, forma de oferta e inclusão destes avisos no pacote escolhido.
`bundle.active` continua false; nenhum pacote final foi gerado nesta revisão.

As permissões de terceiros não são transferidas pela licença do Hub ou
starter. A pendência de texto completo de stackback é test-only e deve ser
resolvida se a toolchain de testes for redistribuída; não é prova de ausência de
licença nem motivo para afirmar que o bundle runtime a contém. R1 segue excluído.
