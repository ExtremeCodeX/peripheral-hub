# Peripheral Hub

## Attack Shark X6 · Parcial

O Hub 0.2.0 inclui instalação pública do X6 em **Explorar softwares**. A release
é fixada por manifesto, tamanho e SHA-256; instalar e abrir o card não acessam
o mouse. A tela X6 / CONTROL reutiliza a arte original e exige GTK para iniciar
o plugin e operar hardware. Requer Linux, Python 3.11+ e acesso hidraw existente.
Veja [catálogo e instalação](docs/public/CATALOG.md) e a
[release do plugin](https://github.com/ExtremeCodeX/attack-shark-x6-plugin/releases/tag/v0.6.0).


Aplicativo **Linux-first** para desenvolver e descobrir integrações comunitárias
de periféricos. Tauri 2, React/TypeScript e Rust; plugins comunicam por
JSON-RPC 2.0 em stdin/stdout. Núcleo sob **Apache-2.0**.

## Clonar e executar

Pré-requisitos: Linux, Node >=22.12, npm >=10, Rust estável, Python >=3.11 em
`/usr/bin/python3`, GTK 3 e WebKitGTK 4.1 de desenvolvimento.
Veja [dependências Linux e comandos](docs/public/DEVELOPMENT.md).

```sh
git clone https://github.com/ExtremeCodeX/peripheral-hub.git
cd peripheral-hub
npm ci
npm run dev
```

O Hub não exige o R1 instalado. `npm run dev:web` abre somente a interface;
operações locais exigem o aplicativo Tauri. O visual existente é preservado.

## Criar suporte para outro mouse

Comece por um simulador independente, sem inventar protocolo HID:

```sh
python3 -B tools/new-plugin.py ../my-mouse --id dev.my-mouse
python3 -B -m unittest discover -s ../my-mouse/tests -v
python3 -I -B tools/install-local.py starter --source "$(realpath ../my-mouse)" --id dev.my-mouse --accept-local-code
npm run dev
```

No **Menu → Registrar starter local (DEV)**, abra o cartão, informe
`dev.my-mouse`, selecione a integração e clique **Iniciar processo**. Autorize
somente código revisado no diálogo GTK nativo. Edite a cópia XDG instalada e
pare/inicie o processo: **não é necessário recompilar o Hub**.

O [guia de plugins](docs/public/STARTER.md) explica capacidades, estado, erros,
limites do contrato e submissão. O transporte não define protocolo de mouse;
o carregador DEV e o runtime público usam Python. O X6 tem configuração física;
isso não promete configuração universal nem um SDK estável.

## Estado do produto

- Catálogo comunitário remoto de metadados com cache/validação e distribuição
  pública fixada do X6. Nenhum R1 ou exemplo fictício é aprovado por este código.
- Instalação pública explícita pelo catálogo; sem execução ou atualização automática.
- Exemplos fictícios e registro de código local somente em desenvolvimento.
- Controles locais vêm das capacidades do processo; sem frontend remoto.
- Execução DEV exige confirmação nativa por snapshot e validação de caminhos.
  **Processo separado e confirmação não são sandbox.** Código autorizado tem
  as permissões do seu usuário, inclusive arquivos, rede e hardware.
- O port Attack Shark R1 e seu companion são projetos separados GPL-3.0-or-later,
  não incluídos no Git, build ou licença Apache deste núcleo. Consulte
  [integração R1](docs/public/R1.md). Sua publicação permanece retida enquanto
  houver direitos de redistribuição não esclarecidos.

## Verificar e compilar

```sh
npm test
npm run typecheck
npm run test:python
npm run test:catalog
npm run test:rust
npm run fmt:rust
npm run build
```

O build gera `apps/desktop/src-tauri/target/release/peripheral-hub`, sem instalador
ou runtime Python embutido. Nenhum desses comandos é teste físico autorizado.
Veja [contribuições](CONTRIBUTING.md), [catálogo](docs/public/CATALOG.md) e
[atribuições](THIRD_PARTY_NOTICES.md).
