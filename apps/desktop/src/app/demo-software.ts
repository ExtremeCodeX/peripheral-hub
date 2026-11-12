import type { DemoMouse, DemoMouseId } from './demo-mice';

export type DemoFeature = { name: string; available: boolean; note: string };
export type DemoSoftware = {
  version: string;
  creator: { name: string; profileUrl: string | null };
  repositoryUrl: string | null;
  coverage: 'complete' | 'partial';
  validation: 'reviewed' | 'beta';
  approvals: 0 | 1 | 2;
  features: DemoFeature[];
  limitation: string;
};

const feature = (name: string, available: boolean, note: string): DemoFeature => ({ name, available, note });
const source = { creator: { name: 'Criador de exemplo', profileUrl: null }, repositoryUrl: null };

// Conteúdo fictício para testar a interface. Não é contrato de plugin nem catálogo.
export const demoSoftware: Record<DemoMouseId, DemoSoftware> = {
  aura: {
    ...source,
    version: '1.0.0-demo', coverage: 'complete', validation: 'reviewed', approvals: 2,
    features: [feature('Sensibilidade (DPI)', true, 'Ajuste de sensibilidade por estágio.'), feature('Taxa de atualização', true, 'Selecionar a frequência de atualização.'), feature('Remapeamento de botões', true, 'Atribuir ações aos botões.'), feature('Perfis', true, 'Salvar e alternar configurações.')],
    limitation: 'Cobertura completa apenas das funções listadas. Macros e iluminação não fazem parte deste escopo.',
  },
  vector: {
    ...source,
    version: '0.6.0-demo', coverage: 'partial', validation: 'reviewed', approvals: 2,
    features: [feature('Sensibilidade (DPI)', true, 'Ajuste de sensibilidade por estágio.'), feature('Taxa de atualização', true, 'Selecionar a frequência de atualização.'), feature('Remapeamento de botões', false, 'Atribuir ações aos botões.'), feature('Iluminação', false, 'Controlar a iluminação do dispositivo.')],
    limitation: 'Remapeamento e iluminação permanecem ausentes. Validação concluída não transforma suporte parcial em completo.',
  },
  nova: {
    ...source,
    version: '0.9.0-demo', coverage: 'complete', validation: 'beta', approvals: 0,
    features: [feature('Sensibilidade (DPI)', true, 'Ajuste de sensibilidade por estágio.'), feature('Taxa de atualização', true, 'Selecionar a frequência de atualização.'), feature('Remapeamento de botões', true, 'Atribuir ações aos botões.'), feature('Perfis', true, 'Salvar e alternar configurações.')],
    limitation: 'Cobertura declarada não equivale a validação independente. Neste cenário fictício ainda faltam duas aprovações humanas.',
  },
  forma: {
    ...source,
    version: '0.4.0-demo', coverage: 'partial', validation: 'beta', approvals: 1,
    features: [feature('Sensibilidade (DPI)', true, 'Ajustar a sensibilidade.'), feature('Remapeamento de botões', true, 'Atribuir ações aos botões.'), feature('Perfis', false, 'Salvar e alternar configurações.'), feature('Taxa de atualização', false, 'Selecionar a frequência de atualização.')],
    limitation: 'Perfis e taxa de atualização não estão implementados. O cenário demonstra uma de duas aprovações humanas, sem votos reais.',
  },
  pulse: {
    ...source,
    version: '1.1.0-demo', coverage: 'complete', validation: 'reviewed', approvals: 2,
    features: [feature('Sensibilidade (DPI)', true, 'Ajustar a sensibilidade.'), feature('Remapeamento de botões', true, 'Atribuir ações aos botões.')],
    limitation: 'Perfis, iluminação e macros não pertencem ao escopo deste exemplo. Completo não significa todos os recursos imagináveis.',
  },
  arc: {
    ...source,
    version: '0.3.0-demo', coverage: 'partial', validation: 'beta', approvals: 0,
    features: [feature('Sensibilidade (DPI)', true, 'Ajustar a sensibilidade.'), feature('Perfis', false, 'Salvar e alternar configurações.'), feature('Remapeamento de botões', false, 'Atribuir ações aos botões.')],
    limitation: 'Apenas sensibilidade é representada como disponível. O exemplo está em BETA e não possui aprovação real ou plugin para instalar.',
  },
};

export type DemoColorOption = {
  id: string;
  label: string;
  colors: DemoMouse['colors'];
  accent: string;
  software: DemoSoftware;
};

// Exceção documentada somente para a galeria de referência; não inferir por cor real.
export const demoColorCases: Partial<Record<DemoMouseId, { reason: string; options: DemoColorOption[] }>> = {
  arc: {
    reason: 'Neste exemplo, Areia e Grafite usam eletrônicas diferentes e precisam de softwares diferentes.',
    options: [
      { id: 'areia', label: 'Areia', colors: ['#e4dfcf', '#b9b19a', '#8b826d'], accent: '#746b58', software: demoSoftware.arc },
      { id: 'grafite', label: 'Grafite', colors: ['#62686c', '#343a40', '#1b2026'], accent: '#b6d8c7', software: {
        ...source, version: '0.5.0-demo', coverage: 'complete', validation: 'beta', approvals: 1,
        features: [feature('Sensibilidade (DPI)', true, 'Ajustes de sensibilidade.'), feature('Perfis', true, 'Salvar e alternar perfis.'), feature('Remapeamento de botões', true, 'Atribuir ações aos botões.')],
        limitation: 'Variante Grafite fictícia: eletrônica e software diferentes da variante Areia. Necessita validação própria.',
      } },
    ],
  },
};

export function coverageLabel(software: DemoSoftware) {
  return software.coverage === 'complete' ? 'Completo' : 'Parcial';
}
