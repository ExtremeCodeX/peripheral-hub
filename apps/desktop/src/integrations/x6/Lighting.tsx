import { useEffect, useState, type CSSProperties } from 'react';
import type { Snapshot } from '../starter/client';
import { MouseArt } from './MouseArt';

const MODES: Record<string, string> = { off: 'Desligado', fixed: 'Cor fixa', breathing: 'Respiração — cor própria', neon: 'Neon — alternância', color_breathing: 'Respiração multicolorida', static_dpi: 'Cor do DPI — fixa', breathing_dpi: 'Cor do DPI — respiração' };
const PRESETS: Record<string, string> = { Roxo: '#A855F7', Azul: '#0000FF', Verde: '#00FF00', Vermelho: '#FF0000', Amarelo: '#FFFF00', Branco: '#FFFFFF' };
export const toHex = (value: unknown) => typeof value === 'number' && Number.isInteger(value) && value >= 0 && value <= 0xffffff ? `#${value.toString(16).padStart(6, '0').toUpperCase()}` : '';
export const fromHex = (value: string) => /^#[0-9a-f]{6}$/i.test(value) ? parseInt(value.slice(1), 16) : null;
type Apply = (key: string, value: unknown, label: string) => void;

function ColorControl({ name, settingKey, state, original, disabled, apply, onPreview }: { name: string; settingKey: string; state: Snapshot | null; original: Record<string, unknown>; disabled: boolean; apply: Apply; onPreview: (color: string) => void }) {
  const current = toHex(state?.values[settingKey]);
  const [hex, setHex] = useState(current);
  useEffect(() => { setHex(current); }, [current, state?.session, settingKey]);
  function chooseColor(color: string) { setHex(color); if (fromHex(color) !== null) onPreview(color); }
  const value = fromHex(hex);
  const writable = !disabled && !!state?.settings.some((s) => s.key === settingKey && s.access.includes('write'));
  const canPreview = !state || writable;
  return <div className="x6-light-color">
    <div className="x6-color-heading"><h4>{name}</h4><span>{current || '—'}</span></div>
    <label className="x6-color-picker" title="Escolher cor">Cor<input aria-label={`Seletor RGB — ${name}`} type="color" value={value === null ? current || '#000000' : hex} disabled={!canPreview} onChange={(e) => chooseColor(e.target.value.toUpperCase())}/></label>
    <label className="x6-hex-field">Hex<input aria-label={`Hexadecimal — ${name}`} value={hex} placeholder="#A855F7" maxLength={7} disabled={!canPreview} aria-invalid={value === null} onChange={(e) => chooseColor(e.target.value)}/></label>
    <span className="x6-color-preview" role="img" aria-label={`Prévia — ${name}: ${value === null ? 'cor inválida' : hex}`} style={{ backgroundColor: value === null ? 'transparent' : hex }}/>
    <div className="x6-color-presets">{Object.entries(PRESETS).map(([label, color]) => <button type="button" key={label} disabled={!canPreview} aria-label={`${label} — ${name}`} title={label} aria-pressed={hex.toUpperCase() === color} onClick={() => chooseColor(color)}><span style={{ backgroundColor: color }}/></button>)}</div>
    <button type="button" className="x6-primary" disabled={!writable || value === null || value === state?.values[settingKey]} onClick={() => apply(settingKey, value, `Aplicar cor — ${name}`)}>Aplicar cor — {name}</button>
    <button type="button" disabled={!writable || !toHex(original[settingKey]) || original[settingKey] === state?.values[settingKey]} onClick={() => apply(settingKey, original[settingKey], `Restaurar cor — ${name}`)}>Restaurar cor original — {name}</button>
  </div>;
}

export function Lighting({ state, busy, original, apply }: { state: Snapshot | null; busy: boolean; original: Record<string, unknown>; apply: Apply }) {
  const [stage, setStage] = useState(4);
  const [mode, setMode] = useState('');
  const [previewColor, setPreviewColor] = useState('#A855F7');
  const [previewTarget, setPreviewTarget] = useState<'stage' | 'effect'>('stage');
  const [levels, setLevels] = useState<Record<string, number>>({});
  useEffect(() => { const active = Number(state?.values.dpi_stage) || 4; setStage(active); const color = toHex(state?.values[`dpi_color_${active}`]); if (color) setPreviewColor(color); }, [state?.session]);
  useEffect(() => { setMode(String(state?.values.lighting_mode ?? '')); setLevels({}); }, [state]);
  const disabled = busy || !state?.hardwareAccess;
  const supports = (key: string) => !!state?.settings.some((s) => s.key === key && s.access.includes('write'));
  const currentMode = String(state?.values.lighting_mode ?? '');
  const knownMode = currentMode in MODES;
  const customColor = ['fixed', 'breathing'].includes(currentMode);
  return <div className="x6-lighting-workspace">
    <div className="x6-lighting-hero"><div className="x6-hero-copy"><h3>Iluminação</h3><p>Escolha uma cor ou efeito para visualizar.</p><span>Prévia ilustrativa · alterações só são enviadas ao aplicar</span></div><div className={`x6-hero-mouse ${mode === 'off' && previewTarget === 'effect' ? 'is-off' : ''} ${['breathing', 'breathing_dpi', 'color_breathing'].includes(mode) && previewTarget === 'effect' ? 'is-breathing' : ''} ${mode === 'neon' && previewTarget === 'effect' ? 'is-neon' : ''}`} style={{ '--preview-color': previewColor } as CSSProperties}><MouseArt/><span className="x6-hero-led"/></div></div>
    <div className="x6-lighting">
    <section className="x6-panel"><h3>Cores dos estágios de DPI</h3>
      <div className="x6-stage-picker" role="group" aria-label="Estágio de DPI para cor">{[1, 2, 3, 4, 5, 6].map((n) => <button key={n} type="button" aria-label={`Estágio ${n}${state?.values[`dpi_stage_${n}`] === undefined ? '' : ` — ${state.values[`dpi_stage_${n}`]} DPI`}${state?.values.dpi_stage === n ? ' — ativo' : ''}`} aria-pressed={stage === n} onClick={() => { setStage(n); setPreviewTarget('stage'); const color = toHex(state?.values[`dpi_color_${n}`]); if (color) setPreviewColor(color); }}><span className="x6-stage-dot" style={{ backgroundColor: toHex(state?.values[`dpi_color_${n}`]) || 'transparent' }}/>{n}</button>)}</div>
      <p className="x6-light-hint">Selecione um estágio e escolha sua cor.</p>
      <div onFocusCapture={() => setPreviewTarget('stage')}><ColorControl key={stage} name={`Estágio ${stage}`} settingKey={`dpi_color_${stage}`} state={state} original={original} disabled={disabled} apply={apply} onPreview={setPreviewColor}/></div>
      
    </section>
    <section className="x6-panel"><h3>Efeito do indicador do mouse</h3>
      <p className="x6-light-hint">Atual: {MODES[currentMode] ?? '—'}</p>
      <div className="x6-mode-picker" role="group" aria-label="Efeito de iluminação">{Object.entries(MODES).map(([key, label]) => <button type="button" key={key} disabled={!!state && (disabled || !supports('lighting_mode') || !knownMode)} aria-pressed={mode === key} onClick={() => { setMode(key); setPreviewTarget('effect'); }}>{label}</button>)}</div>
      <button type="button" className="x6-primary" disabled={disabled || !supports('lighting_mode') || !knownMode || !(mode in MODES) || mode === currentMode} onClick={() => apply('lighting_mode', mode, 'Aplicar modo de iluminação')}>Aplicar efeito</button>
      <button type="button" disabled={disabled || !supports('lighting_mode') || !knownMode || !(String(original.lighting_mode) in MODES) || original.lighting_mode === currentMode} onClick={() => apply('lighting_mode', original.lighting_mode, 'Restaurar modo de iluminação')}>Restaurar efeito original</button>
      <p className="x6-light-hint">{customColor ? 'Cor personalizada disponível' : 'Para escolher uma cor, aplique Cor fixa ou Respiração.'}</p>
      <div onFocusCapture={() => setPreviewTarget('effect')}><ColorControl name="Efeito" settingKey="lighting_color" state={state} original={original} disabled={disabled || !customColor} apply={apply} onPreview={setPreviewColor}/></div>
      {([['lighting_brightness', 'Brilho', 8], ['lighting_speed', 'Velocidade (código firmware)', 5]] as const).map(([key, label, max]) => <div className="x6-light-level" key={key}><label>{label}<input aria-label={label} type="number" min={1} max={max} step={1} value={levels[key] ?? (typeof state?.values[key] === 'number' ? Number(state.values[key]) : '')} disabled={disabled || !supports(key) || !knownMode} onChange={(e) => setLevels((old) => ({ ...old, [key]: e.target.valueAsNumber }))}/></label><button type="button" disabled={disabled || !supports(key) || !knownMode || !Number.isInteger(levels[key]) || levels[key] < 1 || levels[key] > max || levels[key] === state?.values[key]} onClick={() => apply(key, levels[key], `Aplicar ${label}`)}>Aplicar {label}</button><button type="button" disabled={disabled || !supports(key) || !knownMode || typeof original[key] !== 'number' || Number(original[key]) < 1 || Number(original[key]) > max || original[key] === state?.values[key]} onClick={() => apply(key, original[key], `Restaurar ${label}`)}>Restaurar {label}</button></div>)}
      <small className="x6-light-note">Efeitos e níveis ainda não foram validados visualmente nesta unidade. Restaurar recupera o valor inicial da sessão.</small>
    </section>
    </div>
  </div>;
}
