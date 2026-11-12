import type { ComponentType } from 'react';

// Registro explícito de interfaces locais incluídas no build. Não é instalador,
// detecção de hardware nem contrato de plugins externos.
export interface LocalSoftware {
  id: string;
  name: string;
  manufacturer: string;
  status: string;
  coverage?: 'partial' | 'complete';
  available: boolean;
  category?: 'mouse' | 'keyboard' | 'headset' | 'other';
  // Presentation only; hiding the label must not discard classification.
  showCategoryOnCard?: boolean;
  workspace?: boolean;
  // Reviewed, build-local components, like View; never remote code or paths.
  Visual?: ComponentType;
  Presence?: ComponentType;
  View: ComponentType<{ onBack: () => void }>;
}

// Adicione somente interfaces locais revisadas e realmente incluídas no build.
// The real local/dev entry is not a catalog installation. Simulation stays in
// its separate menu entry and is never a fallback for unavailable hardware.
export const registeredSoftware: readonly LocalSoftware[] = [];

export const categoryLabels = { mouse: 'Mouse', keyboard: 'Teclado', headset: 'Headset', other: 'Outro periférico' };

export function filterSoftware(software: readonly LocalSoftware[], query: string) {
  const normalize = (value: string) => value.normalize('NFD').replace(/\p{Diacritic}/gu, '').toLowerCase().trim();
  const term = normalize(query);
  return software.filter((entry) => normalize(`${entry.name} ${entry.manufacturer}`).includes(term));
}
