# Peripheral Hub 0.2.2 — interface X6 simplificada

Inclui as melhorias de detalhes do catálogo da versão 0.2.1 e os novos ajustes:

- X6 / CONTROL com botão **Conectar 2.4 GHz**, sem checkbox adicional.
- Campo de teste local e avisos repetitivos removidos da tela de configuração.
- Biblioteca sem o texto explicativo do R1; Explorar softwares sem subtítulos
  técnicos desnecessários.
- Tela exclusiva de detalhes do X6 reorganizada: identidade e selo Parcial,
  plataforma/versão/instalação, compatibilidade, limitações, repositório e release.
- Layout dos detalhes conferido em 360 e 900 px, sem rolagem horizontal.
- Instalar/Abrir e links nativos preservados; voltar restaura o foco no card.

O plugin X6 permanece em 0.6.0. Consentimento GTK, backups, verificação por hashes,
ACK e readback permanecem. Nenhuma configuração HID foi enviada nesta publicação.

## Linux x86_64

Extraia o arquivo e execute `./peripheral-hub`. Mantenha LICENSE,
THIRD_PARTY_NOTICES e licenças de fontes junto ao aplicativo. Requer bibliotecas
Linux do Tauri/WebKitGTK e Python 3.11+ em `/usr/bin/python3`.
Não é AppImage universal; não execute como root.

Fluxo: **Explorar softwares → card Attack Shark X6 → detalhes → Instalar/Abrir**.
Instalação pública existente, fontes DEV e backups são preservados. Sem atualização
automática do plugin. O consentimento nativo continua obrigatório para iniciar
código e operar hardware; remover o checkbox não remove essa proteção.

Limitações do X6 permanecem: receptor USB 2,4 GHz; recursos e validação física
parciais; sem macros, perfis, firmware, reset, Bluetooth, USB com fio ou RGB da base.
Consulte a [release do X6](https://github.com/ExtremeCodeX/attack-shark-x6-plugin/releases/tag/v0.6.0).
