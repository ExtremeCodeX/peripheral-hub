# Peripheral Hub 0.2.1 — detalhes do catálogo em tela própria

- Explorar softwares mostra somente o card do X6, com SVG original e selo Parcial.
- Clicar no card abre uma tela dedicada com descrição, compatibilidade, versão,
  estado de instalação, limitações e links do repositório/release.
- Instalar/Abrir ficam nessa tela; links abrem no navegador pelo mecanismo nativo.
- Voltar retorna ao catálogo e restaura o foco no card.

O plugin Attack Shark X6 permanece em 0.6.0. Não há mudanças no protocolo HID,
nos recursos físicos ou nas configurações do dispositivo.

## Linux x86_64

Extraia o arquivo e execute `./peripheral-hub`. Mantenha os arquivos de licença
junto à distribuição. Requer as bibliotecas Tauri/WebKitGTK do sistema e Python
3.11+ em `/usr/bin/python3`. Não é AppImage universal; não execute como root.

Fluxo: **Explorar softwares → card Attack Shark X6 → detalhes → Instalar/Abrir**.
Instalação pública existente e backups são preservados. Não há atualização
automática do plugin. Hardware continua exigindo consentimento GTK.

Limitações do X6 permanecem: suporte parcial, receptor USB 2,4 GHz, sem macros,
perfis, firmware, reset, Bluetooth, USB com fio ou RGB da base; teclas/efeitos com
validação física parcial. Nenhuma configuração HID enviada nesta publicação.
