export type DemoMouseId = 'aura' | 'vector' | 'nova' | 'forma' | 'pulse' | 'arc';

export type DemoMouse = {
  id: DemoMouseId;
  name: string;
  manufacturer: string;
  variant: 'classic' | 'sport' | 'ergo';
  colors: [string, string, string];
  accent: string;
};

// Fixtures visuais originais. Não representam dispositivos detectados ou plugins.
export const demoMice: DemoMouse[] = [
  { id: 'aura', name: 'AURA M1', manufacturer: 'Aster', variant: 'classic', colors: ['#f0f0ec', '#c4c7c7', '#929999'], accent: '#697370' },
  { id: 'vector', name: 'VECTOR S', manufacturer: 'Aster', variant: 'sport', colors: ['#62686c', '#343a40', '#1b2026'], accent: '#b6d8c7' },
  { id: 'nova', name: 'NOVA AIR', manufacturer: 'Nimbo', variant: 'classic', colors: ['#ddd4ed', '#afa0c9', '#7c6c9d'], accent: '#65517e' },
  { id: 'forma', name: 'FORMA ERGO', manufacturer: 'Órbita', variant: 'ergo', colors: ['#c9dace', '#99b3a5', '#658478'], accent: '#466657' },
  { id: 'pulse', name: 'PULSE MINI', manufacturer: 'Pulso', variant: 'sport', colors: ['#e2b8ab', '#c78976', '#925e50'], accent: '#725044' },
  { id: 'arc', name: 'ARC PRO', manufacturer: 'Órbita', variant: 'ergo', colors: ['#e4dfcf', '#b9b19a', '#8b826d'], accent: '#746b58' },
];

export function normalizeSearch(value: string) {
  return value.normalize('NFD').replace(/[\u0300-\u036f]/g, '').toLocaleLowerCase('pt-BR').trim();
}

export function filterMice(query: string) {
  const normalized = normalizeSearch(query);
  return demoMice.filter((mouse) => normalizeSearch(`${mouse.name} ${mouse.manufacturer}`).includes(normalized));
}
