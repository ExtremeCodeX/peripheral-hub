# Configuração do catálogo

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
