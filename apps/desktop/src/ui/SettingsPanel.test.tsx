import { afterEach, expect, it } from 'vitest';
import { cleanup, render, screen, within } from '@testing-library/react';
import { SettingsPanel } from './SettingsPanel';

afterEach(cleanup);

it('agrupa conteúdo de outro periférico sob um título acessível', () => {
  render(<SettingsPanel id="keyboard-lighting" title="Iluminação" status="Indisponível">
    <button disabled>Aplicar iluminação</button>
  </SettingsPanel>);
  const panel = screen.getByRole('region', { name: 'Iluminação' });
  expect(within(panel).getByText('Indisponível')).toBeTruthy();
  expect(within(panel).getByRole('button', { name: 'Aplicar iluminação' }).hasAttribute('disabled')).toBe(true);
});
