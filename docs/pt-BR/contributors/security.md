# Segurança e pesquisa responsável

O [SECURITY.md em inglês](../../../SECURITY.md) é a referência. Não há versões publicadas suportadas nem controles de segurança implementados/auditados nesta etapa.

## Reporte privado

Contato privado, repositório GitHub canônico e responsáveis por resposta ainda não foram configurados. O mantenedor deverá definir e verificar esse canal antes de aceitar relatórios sensíveis ou publicar releases. Não há endereço nem prazo de resposta inventado.

Quando o reporte privado do GitHub estiver habilitado no repositório oficial, use **Security → Report a vulnerability**. Caso contrário, solicite um canal privado ao mantenedor sem expor detalhes técnicos. Até a verificação do canal, mantenha informações sensíveis localmente. Não publique exploit, credenciais, binários, capturas completas ou dados identificáveis em issues/PRs.

Um reporte privado deve indicar revisão/componente, impacto, condições, reprodução sanitizada, plataforma/firmware e permissões ou escritas envolvidas. Compartilhe somente evidência mínima pelo canal verificado e coordene a divulgação com os responsáveis.

## Limites da aplicação e do hardware

- Daemon na sessão do usuário, sem elevação automática; IPC local versionado e restrito/autenticado ao mesmo usuário, sem listener público.
- Tauri com privilégio mínimo; frontend sem acesso direto a HID/USB nem pacotes arbitrários.
- Manifestos e respostas são entradas não confiáveis: valide identidade, versões, tipos, tamanhos, parâmetros, sequência, timeout e taxa.
- Drivers Rust revisados/compilados, sem DLLs de terceiros carregadas automaticamente.
- Sem firmware, bootloader, EEPROM, escrita persistente não documentada, comandos destrutivos ou reprodução de pacotes desconhecidos. Enumeração é não destrutiva.
- Macros iniciais limitadas a edição/validação/persistência/simulação. Pare diante de segurança incerta, interface ambígua ou comportamento inesperado.

## Consentimento e laboratório

Analise apenas software/hardware próprios ou autorizados, respeitando leis e licenças. Registre origem, autorização, objetivo, identidade e consentimento localmente. Nunca execute um instalador/binário automaticamente. Análise dinâmica e captura elevada precisam de autorização explícita e laboratório adequado; uma VM pode ter limitações de passthrough USB.

REA é ferramenta opcional de desenvolvimento. Verifique instalação/inicialização antes de configurar MCP e nunca inclua tokens/endpoints presumidos. Não envie evidências a serviços externos ou ferramentas de IA sem autorização explícita e revisão de dados.

## Privacidade e artefatos

Guarde material bruto em `local-data/` na raiz ou outro local comprovadamente ignorado. Pode organizar localmente subdiretórios `vendor/`, `captures/`, `sessions/` e `reports/`; esta etapa não os cria.

Capturas USB podem incluir outros dispositivos e teclas digitadas. Filtre a interface alvo, reduza duração e não digite informações pessoais durante coleta. Antes de exportar, remova tráfego alheio, seriais, caminhos privados, nomes, chaves e dados de entrada. Logs não devem incluir bytes sensíveis por padrão.

Não redistribua instaladores, executáveis, DLLs, firmware, chaves, capturas completas ou material protegido. Somente fixtures necessárias, autorizadas, sintéticas/sanitizadas e revisadas podem entrar no Git. Preserve origem e incerteza não sensíveis, rotule dados sintéticos e revise arquivos preparados para commit; `.gitignore` não protege arquivos já rastreados nem inclusão forçada.

Não guarde evidências sensíveis em SQLite, pacotes, CI ou relatórios públicos. Combine retenção/descarte local com o pesquisador e evite telemetria não solicitada.

## Validação e publicação

Separe simulação, inferência, testes de software e validação física por capacidade/SO/firmware. Testes de hardware são manuais/opt-in, nunca CI automática. Antes de releases, revise dependências, permissões, IPC, logs, parsers, manifestos e sanitização. Contatos, certificados, credenciais e licença estão pendentes; publicação automática ou compartilhamento público exige autorização explícita.
