# Política de engenharia reversa responsável

**Status: política vinculante.** O DRA, o REA, ferramentas de captura e todo trabalho de engenharia reversa neste projeto existem com uma única finalidade: **estudar protocolos de periféricos, interoperabilidade e desenvolvimento independente de drivers em ambientes autorizados**. Consulte a [referência técnica em inglês](../../reverse-engineering/responsible-research.md). Este documento é a tradução prioritária.

Nada aqui concede permissão para examinar, capturar, conectar ou modificar qualquer dispositivo ou software. Ele define os limites que se aplicam por cima do escopo explícito do usuário e do [SECURITY.md](../../../SECURITY.md).

## Finalidade e limites

A engenharia reversa é feita apenas para entender como um dispositivo se comunica e permitir um driver independente e interoperável. Ela **não** serve para contornar proteções, obter segredos, desenvolver ataques ou copiar código do fabricante. Quando a finalidade ou a autorização de um pedido for incerta, ou o pedido conflitar com estas regras, suspenda esse ramo e solicite revisão humana.

## Regras obrigatórias

1. Investigue apenas hardware próprio ou para o qual você tenha autorização adequada.
2. Use executáveis e artefatos de origem legítima; respeite direitos autorais, licenças e a lei aplicável.
3. Priorize análise estática e observação passiva USB/HID antes de qualquer ação ativa.
4. Nunca implemente bypass de autenticação, DRM, licenciamento ou mecanismos de segurança.
5. Nunca extraia credenciais, chaves privadas ou informações confidenciais do fabricante.
6. Nunca desenvolva exploração de vulnerabilidades nem acesso não autorizado a sistemas.
7. Nunca reproduza código-fonte proprietário no repositório, incluindo trechos decompilados; implemente drivers de forma independente a partir de fatos de protocolo permitidos.
8. Mantenha executáveis originais e capturas brutas em armazenamento local ignorado e nunca os envie automaticamente a modelos ou serviços externos; compartilhe apenas achados de protocolo autorizados, revisados e sanitizados.
9. Exija autorização humana explícita para operações físicas ativas, identificando dispositivo, ação e limites; propriedade, descoberta, uma skill ou uma API disponível não a concedem.
10. Suspenda operações com finalidade ou autorização desconhecidas e solicite revisão humana antes de retomar; a aprovação de um experimento ativo nunca dispensa as regras 1–8.

## Aplicação no fluxo

- [Fluxo de pesquisa](../../reverse-engineering/research-workflow.md): autorização/escopo, estático primeiro, captura controlada, sanitização e handoff referenciam estas regras.
- [SECURITY.md](../../../SECURITY.md) e [arquitetura de segurança](../../architecture/security.md): isolamento de laboratório e proibição de upload automático.
- Skills [`reverse-engineer`](../../../.agents/skills/reverse-engineer/SKILL.md), [`capture-protocol`](../../../.agents/skills/capture-protocol/SKILL.md), [`add-device`](../../../.agents/skills/add-device/SKILL.md), [`develop-driver`](../../../.agents/skills/develop-driver/SKILL.md), [`validate-device`](../../../.agents/skills/validate-device/SKILL.md), [`develop-feature`](../../../.agents/skills/develop-feature/SKILL.md) e [`review-contribution`](../../../.agents/skills/review-contribution/SKILL.md): cada uma deve ler e aplicar esta política e parar diante de operação proibida ou de autorização desconhecida.

## Evidência e proveniência

Achados de protocolo mantêm seus rótulos de evidência (`confirmed-on-hardware`, `inferred`, `unverified`). Exportações sanitizadas não podem conter informações confidenciais nem fonte proprietária. Suspeitas de vulnerabilidade encontradas incidentalmente não autorizam exploração; use a [política de reporte privado](../../../SECURITY.md#current-status-and-reporting).
