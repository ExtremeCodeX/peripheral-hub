# Contribuir

Use o [guia de desenvolvimento](docs/public/DEVELOPMENT.md) para instalar as
dependências e executar testes. Mudanças devem preservar Linux-first, a interface
existente, acessibilidade e independência entre o Hub e os periféricos.

## Plugins

Comece pelo [starter e contrato](docs/public/STARTER.md). Mantenha sua integração
em repositório independente, com licença que permita alteração e redistribuição,
origem e atribuições claras. Não copie material público sem permissão de uso.
Não inclua dados privados, dumps, fotos ou ícones de direitos desconhecidos.

Para uma submissão ao catálogo:

1. Documente modelo/revisão, funções implementadas, suporte parcial e limitações.
2. Fixe repositório e commit de uma versão revisada; declare licença/autores.
3. Registre testes simulados separadamente de testes físicos.
4. Forneça vídeo com hardware físico visível demonstrando as funções declaradas.
5. Preencha o [manifesto de publicação](community/templates/publication.json),
   documentos e hashes de artefatos quando houver.
6. Valide com `python3 -B tools/validate-publication.py --submission caminho/publication.json`.
7. Abra PR no repositório oficial. Apenas o mantenedor humano aprova o catálogo.

Validação de schema, vídeo e revisão funcional não constituem sandbox ou auditoria
de segurança. Não adicione exemplos fictícios a `catalog/software/`.

## Segurança e licenças

Não execute a UI como root, não faça alterações udev/sudo/firmware implicitamente
e não teste hardware sem consentimento. Nunca transforme URL ou manifesto em
comando de shell ou carregamento de frontend remoto.

Contribuições ao núcleo são sob Apache-2.0, sem relicenciar código de terceiros.
R1 permanece separado sob GPL-3.0-or-later. Preserve avisos de dependências e
declare origem de trechos reutilizados. Não publique pesquisa privada.
