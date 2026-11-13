# Peripheral Hub 0.2.3 — temas e acesso ao mouse

Melhorias pontuais, preservando a organização da interface e os controles do X6:

- Botão claro/escuro no topo, no lugar do menu, também disponível dentro do X6.
  A preferência é salva entre aberturas do aplicativo.
- Tema claro com fundo cinza suave, textos escuros, bordas fortes e controles
  desabilitados legíveis. Ações disponíveis e indisponíveis têm estados distintos.
- Remoção do botão **Verificar backend** e da navegação duplicada do menu.
  Ferramentas DEV ficam recolhidas no rodapé, somente em desenvolvimento.
- Cards com ações explícitas; integrações já instaladas abrem seus controles
  diretamente pelo catálogo. Voltar restaura o foco ao card.
- Biblioteca com orientação mais direta e correção da busca sem resultados
  para uma integração pública instalada.
- Confirmações GTK encerradas corretamente após uma decisão, sem deixar
  diálogos antigos abertos.

## Lembrar minha escolha

Na autorização nativa de início do plugin, a opção **Lembrar minha escolha**
começa desmarcada. Quando marcada e autorizada, permite executar, conectar e
ler a mesma integração nas próximas conexões, inclusive após reiniciar o Hub.

A escolha é vinculada à identidade, SHA-256 do snapshot e modo da integração.
Código modificado pede nova aprovação. Não é uma autorização para um exemplar
físico específico: a descoberta continua exigindo um candidato compatível
inequívoco. Não há conexão automática nem repetição automática de operações.

**Escritas continuam exigindo confirmação nativa.** Validação de caminhos,
hashes, capacidades, sessão, baseline, backup, ACK e readback permanecem.
Processo separado e confirmação não são sandbox: código autorizado tem as
permissões do usuário.

**Esquecer autorizações**, nos controles do mouse, remove todas as escolhas
salvas do Hub, sem encerrar uma conexão já aberta. Se não for possível salvar
a autorização, a operação informa o erro em vez de prosseguir silenciosamente.

## Linux x86_64

### AppImage

Baixe `peripheral-hub-0.2.3-linux-x86_64.AppImage`, permita a execução e abra:

```sh
chmod +x peripheral-hub-0.2.3-linux-x86_64.AppImage
./peripheral-hub-0.2.3-linux-x86_64.AppImage
```

Sem FUSE 2 disponível, use a extração temporária integrada:

```sh
./peripheral-hub-0.2.3-linux-x86_64.AppImage --appimage-extract-and-run
```

O AppImage foi compilado em Ubuntu 22.04 e inclui bibliotecas de interface,
licenças do Hub, da fonte e avisos dos pacotes de runtime do ambiente de build.
Não inclui Python: os plugins ainda exigem Python 3.11+ em `/usr/bin/python3`
e as permissões hidraw existentes. Não instala regras udev e não é uma sandbox.
Não é uma promessa de compatibilidade com toda distribuição Linux.

### Arquivo tar.gz

Extraia `peripheral-hub-0.2.3-linux-x86_64.tar.gz` e execute `./peripheral-hub`
na pasta extraída. Mantenha LICENSE, THIRD_PARTY_NOTICES, a pasta de licenças e
a licença da fonte junto ao aplicativo. Verifique o arquivo com `SHA256SUMS`.

Requer as bibliotecas Linux do Tauri/GTK/WebKitGTK e Python 3.11+ em
`/usr/bin/python3`. Não execute como root em nenhum dos formatos.
Instalações existentes e backups são preservados. Não há atualização automática
do plugin.

O plugin X6 permanece em **0.6.0**, com suporte **Parcial** e receptor USB 2,4 GHz.
Sem novas promessas de macros, perfis, firmware, reset, Bluetooth, USB com fio ou
RGB da base. Nenhuma conexão ou escrita em hardware físico foi realizada para
validar esta release.
