import type { DemoMouseId } from './demo-mice';

// Matriz de referência local para desenvolvedores e agentes; não é catálogo.
export const demoExamples: Record<DemoMouseId, { title: string; description: string }> = {
  aura: { title: 'Completo · revisado', description: 'Todas as funções do escopo, sem seleção de cor.' },
  vector: { title: 'Parcial · revisado', description: 'Funções ausentes permanecem explícitas após validação.' },
  nova: { title: 'Completo · BETA', description: 'Cobertura completa e nenhuma aprovação humana no exemplo.' },
  forma: { title: 'Parcial · BETA', description: 'Cobertura parcial e uma de duas aprovações no exemplo.' },
  pulse: { title: 'Escopo reduzido', description: 'Poucas funções declaradas, todas disponíveis.' },
  arc: { title: 'Software diferente por cor', description: 'Areia e Grafite mudam versão, funções e validação nos detalhes.' },
};
