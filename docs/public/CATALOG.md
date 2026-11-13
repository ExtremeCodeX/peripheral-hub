# Configuração do catálogo

## Versões públicas instaláveis (Hub 0.2.0)

O catálogo agora apresenta também as versões públicas explicitamente fixadas em
`catalog/installable.json`, começando por Attack Shark X6 0.6.0, estado **Parcial**.
Esse registro de distribuição acompanha o build; não depende de uma pasta DEV,
URL configurada no ambiente ou branch remota móvel. Metadados de versão, manifesto,
licença, tamanho e SHA-256 são revisados juntos no código do Hub.

Em **Explorar softwares**, clicar no card instala a release HTTPS do GitHub.
O worker isolado não executa o pacote e não acessa HID: verifica manifesto/artefato/
licença por hashes fixados, escreve em diretório temporário privado e renomeia
atomicamente para `$XDG_DATA_HOME/peripheral-hub/plugins/<id>` (fallback padrão XDG).
Não há extração de arquivos, URLs/caminhos fornecidos pelo renderer ou atualizações
automáticas. Redirecionamentos são limitados aos hosts de assets do GitHub.

**Meus softwares** mostra uma única entrada pública quando instalada; a fonte DEV
anterior e backups permanecem intactos. Abertura do painel não inicia o plugin.
Conectar exige consentimento GTK para código Python sem sandbox e cada operação
física. O loader revalida hash e permissões antes de executar o snapshot em memória,
inclusive no build release. DEV continua restrito ao debug.

Novas versões exigem novo registro/revisão e build do Hub. Não há instalação
genérica de plugins de terceiros nem qualificação por vídeo presumida. A autorização
do mantenedor para esta primeira distribuição não certifica todas as teclas ou
efeitos: limites e prova parcial constam no manifesto e nas notas da release.
O fluxo comunitário de metadados abaixo mantém suas regras, sem evidência inventada.

## Catálogo remoto de metadados comunitários

O catálogo contém somente metadados. O índice inicial está vazio; não representa
integração aprovada, instalação ou autorização para executar código remoto.

Somente depois de confirmar com o mantenedor um endpoint HTTPS publicado e as
aprovações necessárias, defina `PERIPHERAL_HUB_CATALOG_URL` no ambiente de
compilação Rust e recompile o Hub. Não invente um endereço nem use uma branch
móvel como autorização de versão. Sem configuração, o aplicativo informa que o
catálogo não está configurado. O preview web não oferece transporte nativo.

Verificação local do índice, sem publicar:

```sh
python3 -B tools/build-catalog.py --check
```

As entradas usam o schema de publicação comunitária e o validador existentes.
Referências documentais de uma entrada devem acompanhar a proposta; a seleção
para revisão não deve importar relatórios internos para satisfazer essas referências.
Validação sintática e checksums não aprovam licença, autoria ou hardware.

## Preparar uma submissão

Copie [community/templates/publication.json](../../community/templates/publication.json)
para seu repositório e substitua os campos de rascunho por fatos verificáveis.
`null` e `pending` são honestos durante desenvolvimento, mas bloqueiam uma
submissão incompleta. `images` é opcional; não adicione imagens sem direitos.
VID/PID só devem ser informados com proveniência real. Fonte deve apontar para
commit específico, não `main`/HEAD; artefatos declarados exigem SHA-256.

```sh
python3 -B tools/validate-publication.py --root /caminho/absoluto/do/plugin --submission publication.json
python3 -B tools/build-catalog.py --check
```

O primeiro comando verifica completude, não aprova. A inclusão em
`catalog/software/<id>.json` e regeneração do índice só ocorrem depois de revisão
humana, vídeo físico e resolução de direitos. Registre versão, revisores, data,
decisão e limitações no PR; novas versões externas não são aprovadas automaticamente.

O workflow valida PRs sem publicar. Pages só é habilitado se o mantenedor
configurar `CATALOG_PUBLISH_BRANCH`, proteção da branch e aprovação do ambiente
`github-pages`. Sem isso, não há deploy. Este repositório não configura essas
proteções silenciosamente nem instala software a partir do índice.

Veja [desenvolvimento](DEVELOPMENT.md) e [contribuições](../../CONTRIBUTING.md).
