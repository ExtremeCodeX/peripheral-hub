import { describe, expect, it } from 'vitest';
import { starterError } from './client';

describe('starter local consent and installation errors', () => {
  it('explains preparation instead of pretending the starter is embedded', () => {
    expect(starterError({ kind: 'LOCAL_INTEGRATION_UNAVAILABLE' })).toContain('tools/install-local.py');
  });
  it('reports native refusal and missing provider without implying execution', () => {
    expect(starterError({ kind: 'NATIVE_CONFIRMATION_DENIED' })).toContain('Nenhum processo foi iniciado');
    expect(starterError({ kind: 'NATIVE_CONFIRMATION_REQUIRED' })).toContain('confirmação nativa');
    expect(starterError({ kind: 'NATIVE_CONFIRMATION_UNAVAILABLE' })).toContain('confirmação nativa');
  });
  it('reports unsafe local source and never guarantees no hardware access for editable code', () => {
    expect(starterError({ kind: 'LOCAL_INTEGRATION_UNSAFE' })).toContain('sem symlinks');
    expect(starterError({ kind: 'PLUGIN_EXITED' })).not.toContain('Nenhum hardware');
  });
});
