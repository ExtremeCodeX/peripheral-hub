# Criar uma integração local

## 1. Gerar e testar

Na raiz do Hub:

```sh
python3 -B tools/new-plugin.py ../my-mouse --id dev.my-mouse
python3 -B -m unittest discover -s ../my-mouse/tests -v
```

A cópia tem identidade própria e transporte autocontido. Edite `plugin.py`:
`SETTINGS` declara controles e `INITIAL_VALUES` o estado inicial. O starter
valida escalares booleanos, enums e intervalos numéricos; `device` implementa
as operações. Não declare funções que seu backend não implementa.

O transporte JSON-RPC é independente da linguagem. O carregador DEV desta versão
carrega snapshots Python com biblioteca padrão; binários de outras linguagens e
dependências externas não têm instalador/launcher genérico nesta versão. Isso não
obriga futuros plugins a usar Python, mas limita o fluxo executável aqui entregue.
O modo genérico entregue é simulado: backend físico exige implementação própria,
consentimento específico e validação real, não simplesmente mudar um rótulo.

## 2. Registrar localmente

```sh
python3 -I -B tools/install-local.py starter --source "$(realpath ../my-mouse)" --id dev.my-mouse --accept-local-code
npm run dev
```

O comando copia **somente** `plugin.py` para
`${XDG_DATA_HOME:-$HOME/.local/share}/peripheral-hub/dev-plugins/dev.my-mouse/`.
Não executa código, não sobrescreve uma instalação existente e não baixa nada.
IDs aceitos são nomes DEV limitados, sem barras, traversal, URLs ou shell.
Diretórios/arquivos com symlinks, permissões inseguras ou filesystem remoto
conhecido são recusados; os snapshots são limitados a 64 KiB.

Para usar o exemplo original, omita `--id` e use
`--source "$PWD/community/integration-starter"`: destino `dev-starter/plugin.py`,
identidade `dev.starter`. `dev.fake` é reservado.

## 3. Abrir, editar e reiniciar

Menu → Registrar starter local (DEV) → cartão Starter local — simulado.
Informe `dev.my-mouse`, clique Selecionar integração e Iniciar processo.
Cada início exige confirmação GTK host-local contendo a identidade e SHA-256
do snapshot em memória. Sem provedor nativo ou se cancelar, nada é iniciado.
O processo roda como usuário comum, **sem sandbox**: revise a fonte.

Os controles são renderizados das capacidades: enum, intervalo, booleano e
editor JSON para estruturas. A validação autoritativa está no host e no plugin.
Resultados são sempre lidos do processo; rascunhos não são sucesso de hardware.

Edite **a cópia XDG registrada**, pare e inicie: o loader relê a fonte e pede novo
consentimento, sem recompilar o Hub. Editar a fonte original não atualiza a cópia.
Mantenha a identidade negociada igual ao ID registrado. O registro visual dura
a sessão do Hub; os arquivos XDG permanecem. Para remover uma cópia, pare o Hub
e remova somente seu diretório de integração após conferir o caminho.

## Contrato 0.1

Um objeto JSON-RPC 2.0 por linha UTF-8, stdout exclusivo e flush por resposta;
logs em stderr. Nenhuma porta HTTP, frontend remoto ou shell por RPC.

1. `plugin.hello {apiVersions:["0.1"]}` → `apiVersion`, `pluginId`, `mode:"fake"`, `hardwareAccess:false`.
2. `plugin.get_capabilities {}` → `mode`, `hardwareAccess`, `settings` (1–32).
3. Cada setting contém `key`, `access:["read","write"]`, `schema`, `effects:"simulation_only"`.
4. `device.open {candidateId:"fake-device"}` → sessão opaca de 32 hex e identificação `simulated`.
5. `device.get_state {session}` → `values`, `baselineToken`, `outcome:"simulated"`, sessão/modo/acesso.
6. `device.set_setting {session,baselineToken,key,value}` → estado e novo baseline.
7. `device.close {session}`; `plugin.shutdown {}` encerra o processo.

Um baseline antigo é conflito; bool/float não passam por inteiro. Notificações de
device/shutdown não mutam estado. Batch serial 1–8, sem transação. Starter: linha
até 16 KiB, profundidade 32, 256 frames por processo; host: 256 KiB por frame e
timeout normal de 3 s. JSON duplicado, não finito, UTF-8 inválido, resposta
malformada, crash e timeout são recusados. Falhas invalidam a geração: não há
replay, retry de escrita ou restart automático. Parar encerra/recolhe o processo.

Schema do host: enums escalares, booleanos, intervalos com minimum/maximum/
multipleOf, arrays limitados e objetos de propriedades obrigatórias. Sem refs,
validadores executáveis ou coerção. O starter implementa somente os escalares;
estruturas exigem validação correspondente no seu backend.

## Submissão

Siga [CONTRIBUTING](../../CONTRIBUTING.md) e o
[guia do catálogo](CATALOG.md). O manifesto de publicação não é um manifesto de
execução. Testes simulados não qualificam uma integração física para aprovação.
