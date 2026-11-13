# Peripheral Hub

<!-- impeccable:product-schema 1 -->

## Platform

web

## Users

Usuários Linux que querem descobrir suporte comunitário para seus periféricos e abrir as integrações instaladas. Desenvolvedores têm ferramentas e exemplos separados, disponíveis somente em desenvolvimento.

## Product Purpose

Reunir descoberta, instalação explícita e acesso aos controles dos periféricos. Biblioteca e descoberta têm igual importância. Após rejeitar propostas de redesign, o usuário pediu preservar a UI atual e fazer melhorias pontuais de acesso e limpeza visual.

## Operating Context

Aplicativo desktop Linux-first em Tauri 2, React/TypeScript e Rust. A interface web serve para prévia; operações locais dependem do aplicativo desktop.

## Capabilities and Constraints

- Catálogo comunitário de metadados validado e versões públicas fixadas por manifesto, tamanho e SHA-256.
- Instalação não implica execução, conexão física ou suporte completo.
- Plugins usam JSON-RPC em processos separados; isso não é sandbox. Consentimentos nativos e avisos de segurança devem permanecer.
- O X6 possui suporte parcial real; não prometer configuração universal, SDK estável ou detecção inexistente.
- Preservar o workspace, a arte original X6 e a organização atual. Sem redesenhar a interface.
- O usuário pediu lembrar a autorização de conexão. O opt-in nativo é vinculado à identidade, SHA-256 e modo da integração, inclui execução/conexão/leitura, pode ser revogado e não autoriza escritas.
- Tema claro/escuro com um clique no topo, substituindo o menu. Remover o diagnóstico ‘Verificar backend’ da interface.
- Preservar ferramentas DEV e exemplos fictícios, sem expô-los na produção.

## Brand Commitments

Manter Peripheral Hub, português e a UI atual. O usuário rejeitou todas as propostas de redesign; não substituir a identidade, reorganizar tudo ou iniciar nova rodada estética. Remover ruído e facilitar o acesso ao mouse com mudanças locais. No modo claro, o usuário pediu controles significativamente mais escuros: textos quase pretos, bordas fortes e preenchimentos visíveis, inclusive nos desabilitados. Não voltar a tons apagados ou transparência para indicar indisponibilidade.

## Evidence on Hand

README.md na raiz, docs/public, catálogo e manifestos existentes; apps/desktop/src/app/distribution.ts; integrações locais e testes existentes. Exemplos DEV não são prova de compatibilidade real.

## Product Principles

- Distinguir claramente integrações instaladas e descoberta.
- Priorizar ações e informações úteis sobre decoração.
- Não confundir cobertura da integração, instalação e presença física.
- Não iniciar código ou hardware automaticamente.
- Preservar fluxos funcionais, teclado e retorno de foco.
