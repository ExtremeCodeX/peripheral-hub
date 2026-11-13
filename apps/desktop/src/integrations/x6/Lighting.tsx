import { useEffect, useState } from 'react';
import type { Snapshot } from '../starter/client';

const MODES: Record<string, string> = { off: 'Desligado', fixed: 'Cor fixa', breathing: 'Respiração — cor própria', neon: 'Neon — alternância', color_breathing: 'Respiração multicolorida', static_dpi: 'Cor do DPI — fixa', breathing_dpi: 'Cor do DPI — respiração' };
const PRESETS: Record<string, string> = { Roxo: '#A855F7', Azul: '#0000FF', Verde: '#00FF00', Vermelho: '#FF0000', Amarelo: '#FFFF00', Branco: '#FFFFFF' };
export const toHex = (value: unknown) => typeof value === 'number' && Number.isInteger(value) && value >= 0 && value <= 0xffffff ? `#${value.toString(16).padStart(6, '0').toUpperCase()}` : '';
export const fromHex = (value: string) => /^#[0-9a-f]{6}$/i.test(value) ? parseInt(value.slice(1), 16) : null;
type Apply = (key: string, value: unknown, label: string) => void;

function ColorControl({ name, settingKey, state, original, disabled, apply }: { name: string; settingKey: string; state: Snapshot | null; original: Record<string, unknown>; disabled: boolean; apply: Apply }) {
  const current = toHex(state?.values[settingKey]);
  const [hex, setHex] = useState(current);
  useEffect(() => setHex(current), [current, state?.session, settingKey]);
  const value = fromHex(hex);
  const writable = !disabled && !!state?.settings.some((s) => s.key === settingKey && s.access.includes('write'));
  return <div className="x6-light-color">
    <h4>{name}</h4><p>Cor lida: {current || 'Sem leitura'}</p>
    <label>Seletor RGB<input aria-label={`Seletor RGB — ${name}`} type="color" value={value === null ? current || '#000000' : hex} disabled={!writable} onChange={(e) => setHex(e.target.value.toUpperCase())}/></label>
    <label>Hexadecimal<input aria-label={`Hexadecimal — ${name}`} value={hex} placeholder="#A855F7" maxLength={7} disabled={!writable} aria-invalid={value === null} onChange={(e) => setHex(e.target.value)}/></label>
    <span className="x6-color-preview" role="img" aria-label={`Prévia — ${name}: ${value === null ? 'cor inválida' : hex}`} style={{ backgroundColor: value === null ? 'transparent' : hex }}/>
    <div className="x6-color-presets">{Object.entries(PRESETS).map(([label, color]) => <button type="button" key={label} disabled={!writable} aria-label={`${label} — ${name}`} onClick={() => setHex(color)}><span style={{ backgroundColor: color }}/>{label}</button>)}</div>
    <button type="button" className="x6-primary" disabled={!writable || value === null || value === state?.values[settingKey]} onClick={() => apply(settingKey, value, `Aplicar cor — ${name}`)}>Aplicar cor — {name}</button>
    <button type="button" disabled={!writable || !toHex(original[settingKey]) || original[settingKey] === state?.values[settingKey]} onClick={() => apply(settingKey, original[settingKey], `Restaurar cor — ${name}`)}>Restaurar cor original — {name}</button>
  </div>;
}

export function Lighting({ state, busy, original, apply }: { state: Snapshot | null; busy: boolean; original: Record<string, unknown>; apply: Apply }) {
  const [stage, setStage] = useState(4);
  const [mode, setMode] = useState('');
  const [levels, setLevels] = useState<Record<string, number>>({});
  useEffect(() => { setStage(Number(state?.values.dpi_stage) || 4); }, [state?.session]);
  useEffect(() => { setMode(String(state?.values.lighting_mode ?? '')); setLevels({}); }, [state]);
  const disabled = busy || !state?.hardwareAccess;
  const supports = (key: string) => !!state?.settings.some((s) => s.key === key && s.access.includes('write'));
  const currentMode = String(state?.values.lighting_mode ?? '');
  const knownMode = currentMode in MODES;
  const customColor = ['fixed', 'breathing'].includes(currentMode);
  return <div className="x6-lighting">
    <section className="x6-panel"><h3>Cores dos estágios de DPI</h3>
      <label className="x6-action">Estágio de DPI<select aria-label="Estágio de DPI para cor" value={stage} disabled={disabled} onChange={(e) => setStage(Number(e.target.value))}>{[1, 2, 3, 4, 5, 6].map((n) => <option key={n} value={n}>Estágio {n} — {state?.values[`dpi_stage_${n}`] === undefined ? 'sem leitura' : `${state.values[`dpi_stage_${n}`]} DPI`}{state?.values.dpi_stage === n ? ' — ativo' : ''}</option>)}</select></label>
      <p>Editar a cor não ativa o estágio nem muda seu DPI. A cor permanece no LED nos modos “Cor do DPI”; em outros modos pode aparecer apenas na indicação temporária.</p>
      <ColorControl key={stage} name={`Estágio ${stage}`} settingKey={`dpi_color_${stage}`} state={state} original={original} disabled={disabled} apply={apply}/>
      <small>Oito cores são lidas; slots 7/8 reservados são preservados e não editáveis. RGB por estágio: validação física descrita na documentação, não calibração de cor.</small>
    </section>
    <section className="x6-panel"><h3>Efeito do indicador do mouse</h3>
      <p>Modo lido: {MODES[currentMode] ?? 'Não disponível / desconhecido — preservado'}</p>
      <label className="x6-action">Modo<select aria-label="Modo de iluminação" disabled={disabled || !supports('lighting_mode') || !knownMode} value={mode} onChange={(e) => setMode(e.target.value)}><option value="" disabled>Sem leitura</option>{Object.entries(MODES).map(([key, label]) => <option key={key} value={key}>{label}</option>)}</select></label>
      <button type="button" className="x6-primary" disabled={disabled || !supports('lighting_mode') || !knownMode || !(mode in MODES) || mode === currentMode} onClick={() => apply('lighting_mode', mode, 'Aplicar modo de iluminação')}>Aplicar efeito</button>
      <button type="button" disabled={disabled || !supports('lighting_mode') || !knownMode || !(String(original.lighting_mode) in MODES) || original.lighting_mode === currentMode} onClick={() => apply('lighting_mode', original.lighting_mode, 'Restaurar modo de iluminação')}>Restaurar efeito original</button>
      <p>Cor própria somente em Cor fixa/Respiração. Aplique primeiro esse modo; não há alteração automática de modo ao escolher uma cor.</p>
      <ColorControl name="Efeito" settingKey="lighting_color" state={state} original={original} disabled={disabled || !customColor} apply={apply}/>
      {([['lighting_brightness', 'Brilho', 8], ['lighting_speed', 'Velocidade (código firmware)', 5]] as const).map(([key, label, max]) => <div className="x6-light-level" key={key}><label>{label}<input aria-label={label} type="number" min={1} max={max} step={1} value={levels[key] ?? (typeof state?.values[key] === 'number' ? Number(state.values[key]) : '')} disabled={disabled || !supports(key) || !knownMode} onChange={(e) => setLevels((old) => ({ ...old, [key]: e.target.valueAsNumber }))}/></label><button type="button" disabled={disabled || !supports(key) || !knownMode || !Number.isInteger(levels[key]) || levels[key] < 1 || levels[key] > max || levels[key] === state?.values[key]} onClick={() => apply(key, levels[key], `Aplicar ${label}`)}>Aplicar {label}</button><button type="button" disabled={disabled || !supports(key) || !knownMode || typeof original[key] !== 'number' || Number(original[key]) < 1 || Number(original[key]) > max || original[key] === state?.values[key]} onClick={() => apply(key, original[key], `Restaurar ${label}`)}>Restaurar {label}</button></div>)}
      <small>Sete modos, brilho 1–8 e velocidade 1–5 documentados nas referências; efeitos/níveis ainda sem validação visual individual nesta unidade. Não alteramos suspensão ou debounce. Restaurar recupera cada campo do início da sessão, não padrões de fábrica.</small>
    </section>
    <section className="x6-panel"><h3>Base de carregamento</h3><p>RGB da base é separado do indicador do mouse. Não foi identificado protocolo seguro de controle por software nas referências; use o controle físico da base. Nenhum comando é enviado à base.</p></section>
  </div>;
}
