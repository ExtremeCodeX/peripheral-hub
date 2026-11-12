import { describe, expect, it } from 'vitest';
import { demoMice } from './demo-mice';
import { demoColorCases, demoSoftware } from './demo-software';
import { buildReviewPrompt } from './review-prompt';

describe('dados de demonstração e prompt', () => {
  it('mantém cobertura consistente com as funções e BETA separado', () => {
    for (const mouse of demoMice) {
      const software = demoSoftware[mouse.id];
      expect(software.creator.name).toBe('Criador de exemplo');
      expect(software.creator.profileUrl).toBeNull();
      expect(software.repositoryUrl).toBeNull();
      expect(software.version.endsWith('-demo')).toBe(true);
      expect(software.features.every((feature) => feature.available)).toBe(software.coverage === 'complete');
      expect(software.approvals < 2).toBe(software.validation === 'beta');
    }
    expect(demoSoftware.nova.coverage).toBe('complete');
    expect(demoSoftware.nova.validation).toBe('beta');
    expect(demoSoftware.vector.coverage).toBe('partial');
    expect(demoSoftware.vector.validation).toBe('reviewed');
  });

  it('gera um roteiro por modelo sem inventar links ou validações', () => {
    for (const mouse of demoMice) {
      const prompt = buildReviewPrompt(mouse, demoSoftware[mouse.id]);
      expect(prompt).toContain(mouse.name);
      expect(prompt).toContain(demoSoftware[mouse.id].version);
      expect(prompt).toContain('DEMONSTRAÇÃO — NÃO É AVALIAÇÃO DE HARDWARE');
      expect(prompt).toContain('confirme comigo qual é o dispositivo REAL');
      expect(prompt).toContain('não testado');
      expect(prompt).toContain('duas aprovações humanas');
      expect(prompt).toContain('decisão final do mantenedor');
      expect(prompt).toContain('Gere o relatório final somente depois');
      expect(prompt).toContain(`avaliacao-${mouse.id}.md`);
      expect(prompt).not.toContain('https://');
    }
  });

  it('contextualiza a exceção por cor sem generalizá-la', () => {
    const arc = demoMice.find((mouse) => mouse.id === 'arc')!;
    const option = demoColorCases.arc!.options[1];
    const prompt = buildReviewPrompt(arc, option.software, option.label);
    expect(prompt).toContain('Cor / variante: Grafite');
    expect(prompt).toContain('0.5.0-demo');
    expect(prompt).toContain('avaliacao-arc-grafite.md');
    expect(prompt).toContain('Não presuma incompatibilidade somente pela cor');
    expect(prompt).not.toContain('Versão: 0.3.0-demo');
  });
});
