# Adicionando um dispositivo, com ou sem IA

[English](../../en/contributors/adding-a-device.md)

**Etapa atual:** [SDK 1.0, Registry e Core](../../architecture/sdk.md) são preservados; a [Fase 2](../users/daemon.md) implementa descoberta/transporte HID real, IPC protegido, daemon e simulação explícita, e o [sistema de drivers](../../architecture/driver-system.md) adiciona um interpretador declarativo limitado (`serve --manifests DIR`, `schemas/device-manifest.schema.json`) que cobre apenas iluminação volátil. DRA, scaffold e drivers físicos proprietários continuam pendentes. Um driver real exige escopo autorizado, política/protocolo comprovados e validação física por capacidade; enumeração bem-sucedida não basta.

## Identidade e objetivo limitado

Registre modelo real, sistema operacional, firmware/revisão, transporte, VID/PID, interfaces, usage page/usage e evidências adicionais de identificação. VID/PID, sozinhos, não definem um protocolo. Não exponha seriais identificáveis ou caminhos privados. Escolha uma capacidade específica, como uma mudança volátil de cor comprovadamente segura, sem prometer suporte completo.

Verifique entradas existentes. `devices/manufacturer/device-model/` é um placeholder vazio, não um exemplo suportado.

## Evidências e segurança

Leia a [política de segurança](security.md) e o [procedimento técnico de pesquisa](../../reverse-engineering/research-workflow.md). Obtenha consentimento e registre origem autorizada, versão e hash do software original. Guarde binários/capturas brutos em armazenamento local ignorado pelo Git. Análise estática opcional com REA orienta hipóteses, mas capturas/testes físicos podem ser necessários.

Execução dinâmica e elevação para captura precisam de autorização explícita e laboratório adequado. Correlacione baseline e ações individuais seguras na interface do fabricante com o tráfego da interface alvo. Não reproduza pacotes desconhecidos nem tente firmware, bootloader ou escritas persistentes. Sanitize e revise qualquer exportação.

## Documente antes de implementar

Use o [formato de protocolo](../../protocols/specification-format.md) para relatórios comprovados, inicialização, sequenciamento, ACKs, checksums, limites, incertezas e diferenças entre revisões. Hipóteses usam `confirmed-on-hardware`, `inferred` ou `unverified`. Explicite operações não suportadas e inseguras.

Quando SDK/validadores existirem e o código estiver autorizado, escolha operações declarativas limitadas para protocolos simples já modelados ou um driver Rust compilado e revisado para casos complexos. Declare apenas capacidades comprovadas, erros reais e testes significativos com transporte fake. Sem plugins nativos arbitrários ou pacotes enviados pelo frontend.

## Artefatos esperados futuramente

Em `devices/<manufacturer>/<model>/`: `manifest.toml`, `protocol.md`, `validation.md`, `tests/` e `fixtures/` textuais sintéticas/sanitizadas autorizadas. Inclua `Cargo.toml` e `src/` somente para driver nativo com código real. Campos e versões devem vir dos modelos implementados, não ser inventados. Exemplos fictícios pertencem a `examples/`, fora do catálogo validado.

Em `validation.md`, registre SO, firmware/revisão, revisão do driver, capacidade/interface testada, resultado real e limitações. Diferencie `simulated`, `inferred`, `tested` e `validated-on-hardware`; testes de software não provam sucesso físico. Hardware exige adesão explícita e não participa de CI automática.

## Com IA

Leia [AGENTS.md](../../../AGENTS.md) e use [add-device](../../../.agents/skills/add-device/SKILL.md) e as skills correspondentes. Elas orientam, mas não autorizam execução, operações físicas ou publicação. Não envie evidências proprietárias a serviços externos de IA. Verifique comandos em ferramentas instaladas; os comandos DRA do documento mestre ainda são propostas.

## Sem IA

Siga manualmente identificação → evidências → implementação → validação → revisão usando os guias vinculados. Registre ferramentas ausentes e capacidades não testadas. OpenCode e REA não são obrigatórios para contribuir com documentação ou revisar evidências.

## Revisão

Prepare uma proposta de PR focada com artefatos sanitizados, origem, verificações/resultados exatos e lacunas por capacidade. Siga o [guia de contribuição](contributing.md) para o repositório canônico existente. Confirmação explícita da licença, canal privado de segurança, responsáveis e CI ativa dependem do mantenedor. Agentes só publicam mediante autorização explícita.
