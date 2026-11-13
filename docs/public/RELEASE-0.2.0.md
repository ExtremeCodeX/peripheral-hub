# Peripheral Hub 0.2.0 — X6 público Parcial

- Card X6 com a mesma arte SVG do workspace, sem hotspots, e selo **Parcial**.
- Distribuição pública do Attack Shark X6 0.6.0 em **Explorar softwares**.
- Manifesto, fonte e licença baixados da release HTTPS e verificados por hashes
  fixados no build. Instalação atômica, sem execução ou acesso HID.
- X6 / CONTROL disponível no aplicativo empacotado; GTK obrigatório para iniciar
  código e para abrir, ler ou alterar hardware. Python não é sandbox.
- Runtime DEV continua disponível no debug; instalação pública separada e card
  público prioritário, preservando fontes DEV e backups.

## Executar a distribuição Linux x86_64

Extraia o arquivo da release e execute `./peripheral-hub` dentro da pasta extraída.
Mantenha LICENSE/THIRD_PARTY_NOTICES e licenças de fontes junto à distribuição.
O binário requer as bibliotecas Linux do Tauri/WebKitGTK e Python 3.11+ em
`/usr/bin/python3`. Não é AppImage universal nem instala dependências/regras udev.
Veja [dependências](DEVELOPMENT.md); não execute como root.

Depois: **Explorar softwares → Instalar Attack Shark X6 → Meus softwares → Abrir**.
A abertura da tela não inicia o plugin. Confirme modelo/conexão e consinta no GTK
somente ao conectar. Transporte suportado: X6 por receptor USB 2,4 GHz.

## Limites

Distribuição pública inicial fixa uma versão revisada, sem atualizações automáticas
ou instalação arbitrária de terceiros. O catálogo comunitário remoto de metadados
continua separado e pode informar que sua URL não foi configurada; isso não bloqueia
a versão pública X6 incluída no catálogo do build.

RGB por estágio comprovado fisicamente; efeitos/níveis e catálogo de teclas têm
validação parcial. Sem macros, perfis, firmware, reset, Bluetooth, USB com fio ou
RGB da base. Sem vídeo público de qualificação individual. Consulte os limites
da release [X6 0.6.0](https://github.com/ExtremeCodeX/attack-shark-x6-plugin/releases/tag/v0.6.0).

Nenhuma configuração HID foi enviada nos testes de publicação.
