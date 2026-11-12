import type { DemoMouse } from './demo-mice';
import { coverageLabel, type DemoSoftware } from './demo-software';

export function buildReviewPrompt(mouse: DemoMouse, software: DemoSoftware, colorLabel?: string) {
  const fileSuffix = colorLabel ? `-${colorLabel.normalize('NFD').replace(/[\u0300-\u036f]/g, '').toLowerCase().replace(/[^a-z0-9]+/g, '-')}` : '';
  return `Você vai me entrevistar para preparar uma avaliação funcional de uma integração de periférico Linux para o Peripheral Hub. Responda em português brasileiro.

CONTEXTO DESTA CÓPIA
- Modelo: ${mouse.name}.
- Fabricante fictício: ${mouse.manufacturer}.${colorLabel ? `\n- Cor / variante: ${colorLabel}. Neste exemplo ela muda o software; não generalize isso para outros modelos.` : ''}
- Versão: ${software.version}.
- Cobertura declarada: ${coverageLabel(software)}.
- Validação: ${software.validation === 'beta' ? 'BETA' : 'revisada no exemplo'}; ${software.approvals}/2 aprovações no cenário fictício.
- Repositório: não informado; este modelo é uma demonstração local.
- Funções declaradas no exemplo:
${software.features.map((item) => `  - ${item.name}: ${item.available ? 'declarada disponível' : 'não implementada'} — ${item.note}`).join('\n')}
- Limitações: ${software.limitation}

REGRA INICIAL OBRIGATÓRIA
Os dados acima são fictícios e não comprovam software, vídeo ou hardware. Antes de perguntar se algo funciona, confirme comigo qual é o dispositivo REAL, o link do repositório da integração e a versão/commit exatos que testei. Se eu estiver apenas explorando esta demonstração, limite a conversa à experiência da interface e marque o resultado como DEMONSTRAÇÃO — NÃO É AVALIAÇÃO DE HARDWARE. Nunca converta as funções fictícias acima em fatos de um dispositivo real.

COMO CONDUZIR A ENTREVISTA
1. Faça poucas perguntas por vez e aguarde minhas respostas. Pergunte se tenho o dispositivo físico e se executei pessoalmente a versão declarada.
2. Confirme distribuição/versão Linux, arquitetura, forma de conexão e modo de instalação, sem pedir dados pessoais, serial, tokens, senhas, hostname ou logs completos. Só peça VID/PID se necessário e conhecido; nunca invente. Se houver diferença documentada entre cores/revisões, confirme a variante efetivamente testada e a versão correta do software; um exemplar não comprova equivalência entre todas as cores. Não presuma incompatibilidade somente pela cor.
3. Pergunte “Tudo funciona corretamente dentro do escopo declarado?” e depois examine cada função REAL documentada no repositório confirmado: o que tentei, passos, resultado esperado, resultado observado e como verifiquei o efeito no hardware. Não solicite firmware, resets irreversíveis, comandos desconhecidos, root ou alteração de udev como teste casual.
4. Separe: confirmado com hardware; falhou; não testado; não implementado; não aplicável. Ausência de teste não é sucesso. Simulação ou resposta de sucesso do backend não comprovam alteração física.
5. Pergunte sobre desconexão/reconexão, mensagens de permissão e recuperação de erro somente quando eu puder testar com segurança. Não transforme casos não executados em resultados.
6. Pergunte por vídeo com hardware físico visível demonstrando as funções declaradas e, quando necessário, como o efeito foi verificado. Se eu não tiver evidência, registre “não fornecida”. Não invente URLs nem diga que assistiu a um vídeo sem realmente poder acessá-lo.
7. Peça reprodução das falhas, limitações, testes pendentes e evidências sanitizadas. Avise que eu devo revisar/remover dados pessoais e segredos antes de compartilhar informações com IA ou publicamente.
8. Não considere esta conversa uma aprovação. Uma avaliação submetida não é automaticamente um voto válido; duas aprovações humanas e a decisão final do mantenedor pertencem ao processo do projeto. Vídeo ou duas aprovações não garantem segurança, ausência de vulnerabilidades ou de bugs.
9. Antes de concluir, mostre um resumo e pergunte “Você já descreveu tudo ou quer acrescentar/corrigir algo?”. Gere o relatório final somente depois que eu confirmar que terminei. Não pressione por conclusão positiva.

ENTREGA FINAL
Gere um Markdown pronto para revisão humana e publicação manual no repositório correto, seguindo CONTRIBUTING e o template de Issues/Discussions desse projeto quando existirem. Não afirme ter publicado. Se puder criar arquivos, ofereça o mesmo conteúdo como avaliacao-${mouse.id}${fileSuffix}.md; caso contrário, entregue o Markdown em um único bloco de código, pronto para copiar. Sem integração automática com GitHub.

Use esta estrutura, preenchendo somente o que confirmei e marcando lacunas como “não informado”:
# Avaliação funcional — [modelo real confirmado ou DEMONSTRAÇÃO]
## Natureza do relato
[hardware físico / somente interface de demonstração; não é auditoria de segurança]
## Integração e versão
[repositório confirmado, versão/tag/commit, modelo, conexão; sem serial]
## Ambiente Linux
[distribuição, versão, arquitetura e instalação]
## Escopo e limitações documentadas
[origem do escopo real; não importar automaticamente fixtures]
## Funções avaliadas
| Função | Estado | Como testei | Esperado | Observado | Evidência |
| --- | --- | --- | --- | --- | --- |
## Falhas e reprodução
[passos, frequência e recuperação; ou não observado/não testado]
## Vídeo e outras evidências
[links fornecidos ou não fornecidos; não inventar]
## Testes não executados e limitações
[pendências explícitas]
## Conclusão do usuário
[posição do usuário e ressalvas, sem autoaprovação do projeto]
## Revisão humana pendente
[este relato não registra voto nem remove BETA; decisão do mantenedor]
## Revisão de privacidade
[confirmar remoção de dados pessoais/segredos antes da publicação]
`;
}
