# Licenças e atribuições

O mantenedor definiu **Apache-2.0** para o código próprio do núcleo Hub e seu
starter. O texto está em `LICENSE`. Essa decisão não transfere direitos de
terceiros nem se aplica ao port/companion R1 GPL-3.0-or-later, excluídos daqui.

- **Outfit**: Copyright 2021 The Outfit Project Authors
  (https://github.com/Outfitio/Outfit-Fonts), SIL OFL-1.1.
  Licença integral preservada em `apps/desktop/src/ui/fonts/OFL.txt`.
- Ilustrações fictícias e ícones de UI são vetores próprios do projeto.
  O ícone de build é geométrico genérico, não artwork de fabricante.
- React, React DOM e scheduler: MIT. Tauri API: Apache-2.0 OR MIT,
  com aviso próprio Microsoft para o subconjunto tslib incluído.
- Os avisos versionados npm/Cargo e respectivas origens são preservados em
  [licenses/INDEX.json](licenses/INDEX.json) e `licenses/texts/`.
  Não foram alteradas as licenças dessas dependências.

GTK/WebKitGTK, Python e bibliotecas do sistema não são distribuídos nesta fonte.
O inventário corresponde aos lockfiles Linux usados nesta preparação; não é uma
promessa de que todas as dependências sejam MIT ou de que todo pacote esteja no
binário final. Consulte [obrigações de distribuição](licenses/README.md).

Este repositório distribui fontes, não node_modules, crates ou toolchains de
teste. O inventário registra uma lacuna do texto MIT de stackback 0.0.2, test-only:
obter o aviso exato antes de redistribuir essa ferramenta. Isso não altera a
licença declarada nem inclui a dependência no bundle de produção.
