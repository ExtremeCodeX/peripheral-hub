import { useEffect, useRef, useState } from 'react';
import type { LocalSoftware } from '../../app/local-software';
import { starterClient, starterError, type Snapshot } from '../starter/client';
import './control.css';
import catalogData from './key-catalog.json';
import { Lighting } from './Lighting';
import { MouseArt, X6CardArt } from './MouseArt';

const BUTTONS = ['Clique esquerdo', 'Clique direito', 'Clique da roda', 'Lateral dianteiro', 'Lateral traseiro', 'Botão superior 1 (frente)', 'Botão superior 2 (trás)'];
type CatalogEntry = { id: string; label: string; category: string; disabled?: boolean };
const CATALOG: CatalogEntry[] = catalogData;
const ACTIONS: Record<string, string> = Object.fromEntries(CATALOG.map((entry) => [entry.id, entry.label]));
ACTIONS.unsupported = 'Ação não decodificada — preservada';
const normalize = (value: string) => value.normalize('NFD').replace(/\p{Diacritic}/gu, '').toLowerCase();
const POINTS = [[36, 22], [67, 22], [51, 25], [15, 42], [15, 54], [52, 36], [52, 44]];

/** Original schematic, authored for this integration; no R1 or third-party asset. */
function MouseDiagram({ selected, select }: { selected: number; select: (n: number) => void }) {
  return <div className="x6-mouse" aria-label="Seleção visual de botões">
    <MouseArt/>
    {POINTS.map(([left, top], i) => <button type="button" key={i} className={`x6-hotspot${i >= 5 ? ' x6-hotspot--top' : ''}${selected === i + 1 ? ' is-selected' : ''}`} style={{ left: `${left}%`, top: `${top}%` }} aria-label={`Selecionar ${BUTTONS[i]} no mouse`} aria-pressed={selected === i + 1} onClick={() => select(i + 1)}>{i + 1}</button>)}
  </div>;
}

export function X6Control({ onBack, pluginId = 'dev.attack-shark-x6' }: { onBack: () => void; pluginId?: string }) {
  const [tab, setTab] = useState('Mouse');
  const [state, setState] = useState<Snapshot | null>(null);
  const [busy, setBusy] = useState(false);
  const [confirmed, setConfirmed] = useState(false);
  const [error, setError] = useState('');
  const [status, setStatus] = useState('Conecte o X6 para ler configurações reais. Nenhuma alteração automática.');
  const [selected, setSelected] = useState(4);
  const [action, setAction] = useState('');
  const [search, setSearch] = useState('');
  const [category, setCategory] = useState('Todas');
  const [lastKey, setLastKey] = useState('Nenhum evento local');
  const [draft, setDraft] = useState<Record<string, unknown>>({});
  const [dpi, setDpi] = useState(4);
  const [logs, setLogs] = useState<string[]>([]);
  const original = useRef<Record<string, unknown>>({});
  const generation = useRef(0);
  const heading = useRef<HTMLHeadingElement>(null);
  useEffect(() => { heading.current?.focus(); return () => { generation.current++; void starterClient.stop().catch(() => undefined); }; }, []);
  const connected = state?.mode === 'hardware' && state.hardwareAccess;
  const key = `button_${selected}`;
  const topMappingAvailable = !!state?.settings.some((s) => s.key === 'button_7') && state?.values.button_7 !== undefined;
  const setting = state?.settings.find((s) => s.key === key);
  const writable = !!connected && (selected < 6 || topMappingAvailable) && !!setting?.access.includes('write') && state?.values[key] !== 'unsupported';
  const options = setting?.schema.enum?.filter((v): v is string => typeof v === 'string' && v in ACTIONS && v !== 'unsupported') ?? [];
  const categories = [...new Set(CATALOG.map((entry) => entry.category))];
  const filtered = CATALOG.filter((entry) => (options.includes(entry.id) || entry.disabled) && (category === 'Todas' || entry.category === category) && normalize(`${entry.label} ${entry.id}`).includes(normalize(search)));
  const currentAction = selected >= 6 && !topMappingAvailable ? undefined : state?.values[key];
  function select(n: number) { setSelected(n); setAction(''); setSearch(''); setCategory('Todas'); }
  async function run(task: () => Promise<Snapshot | null>, label: string, starting = false) {
    if (busy) return;
    const current = ++generation.current;
    setBusy(true); setError('');
    try {
      const result = await task();
      if (current !== generation.current) return;
      if (result && (result.mode !== 'hardware' || !result.hardwareAccess)) throw new Error('INVALID_HANDSHAKE');
      if (starting && result) { original.current = { ...result.values }; setDpi(Number(result.values.dpi_stage)); }
      setState(result); setDraft(result?.values ?? {}); setAction('');
      if (!result) { setConfirmed(false); original.current = {}; }
      const message = result ? `${label} · ${result.outcome === 'confirmed_by_readback' ? 'Escrita confirmada por readback físico.' : result.outcome === 'unchanged' ? 'Sem alteração.' : 'Leitura física validada.'}` : 'Desconectado · processo encerrado.';
      setStatus(message); setLogs((old) => [message, ...old].slice(0, 16));
    } catch (failure) {
      if (current !== generation.current) return;
      const message = starterError(failure);
      setState(null); setDraft({}); setError(message); setStatus('Sem confirmação — sessão descartada.');
      setConfirmed(false); original.current = {};
      setLogs((old) => [message, ...old].slice(0, 16));
      await starterClient.stop().catch(() => undefined);
    } finally { if (current === generation.current) setBusy(false); }
  }
  function apply(k: string, value: unknown, label: string) { if (state) void run(() => starterClient.set(state, value, k), label); }
  function read(label = 'Ler estado') { if (state) void run(() => starterClient.read(state.session), label); }
  const dpiKey = `dpi_stage_${dpi}`;
  return <section className="x6-control" aria-labelledby="x6-title">
    <header className="x6-heading"><h2 id="x6-title" ref={heading} tabIndex={-1}>X6 / CONTROL</h2><span className={`x6-badge${connected ? ' is-live' : ''}`}>{busy ? 'AGUARDANDO GTK / HID' : connected ? 'HARDWARE · CONECTADO' : error ? 'SEM CONFIRMAÇÃO' : 'DESCONECTADO'}</span><button type="button" className="x6-back" onClick={onBack} disabled={busy}>Voltar ao Hub</button></header>
    <div className="x6-toolbar"><div role="tablist" aria-label="Seções X6">{['Mouse', 'Macros', 'Diagnóstico', 'Iluminação'].map((name, i, tabs) => <button type="button" role="tab" key={name} id={`x6-tab-${i}`} aria-selected={tab === name} aria-controls={`x6-panel-${i}`} tabIndex={tab === name ? 0 : -1} onClick={() => setTab(name)} onKeyDown={(e) => { if (['ArrowLeft', 'ArrowRight', 'Home', 'End'].includes(e.key)) { e.preventDefault(); const next = e.key === 'Home' ? 0 : e.key === 'End' ? tabs.length - 1 : (i + (e.key === 'ArrowRight' ? 1 : tabs.length - 1)) % tabs.length; setTab(tabs[next]); document.getElementById(`x6-tab-${next}`)?.focus(); } }}>{name}</button>)}</div>
      <div className="x6-connection">{!connected && <label><input type="checkbox" checked={confirmed} onChange={(e) => setConfirmed(e.target.checked)} disabled={busy}/> Confirmei X6 · dongle 2,4 GHz</label>}<button type="button" disabled={busy || (!connected && !confirmed)} onClick={() => connected ? void run(async () => { await starterClient.stop(); return null; }, 'Desconectar') : void run(() => starterClient.start(pluginId, true), 'Conectar X6', true)}>{connected ? 'Desconectar' : 'Conectar X6'}</button></div>
    </div>
    <p className={`x6-notice${error ? ' is-error' : ''}`} role={error ? 'alert' : 'status'}>{error || status}</p>
    <div role="tabpanel" id="x6-panel-0" aria-labelledby="x6-tab-0" hidden={tab !== 'Mouse'}>
      <div className="x6-columns">
        <aside className="x6-left">
          <section className="x6-panel"><div className="x6-panel-heading"><h3>Botões</h3><button type="button" disabled={!connected || busy} onClick={() => read('Ler botões')}>Ler botões</button></div>
            <div className="x6-button-list">{BUTTONS.map((name, i) => <button type="button" key={name} aria-pressed={selected === i + 1} onClick={() => select(i + 1)}><span>{i + 1}</span><span>{name}<small>{state ? i >= 5 && !topMappingAvailable ? 'Snapshot antigo — atualize o plugin' : ACTIONS[String(state.values[`button_${i + 1}`])] ?? 'Indisponível — sem leitura' : 'Aguardando leitura…'}</small></span></button>)}</div>
            <p className="x6-muted">Função atual lida: {currentAction === undefined ? 'Sem leitura' : ACTIONS[String(currentAction)] ?? 'Não decodificada — preservada'}</p>
            <label className="x6-action">Pesquisar função<input aria-label="Pesquisar função" type="search" placeholder="F5, Ctrl, Espaço, Volume…" value={search} onChange={(e) => setSearch(e.target.value)}/></label>
            <label className="x6-action">Categoria<select aria-label="Categoria de função" value={category} onChange={(e) => setCategory(e.target.value)}><option>Todas</option>{categories.map((name) => <option key={name}>{name}</option>)}</select></label>
            <label className="x6-action">Nova ação de {BUTTONS[selected - 1]}<select aria-label="Nova ação do botão selecionado" value={action || (typeof currentAction === 'string' ? currentAction : '')} disabled={!writable || busy} onChange={(e) => setAction(e.target.value)}><option value="" disabled>{state ? 'Escolha uma função' : 'Aguardando leitura…'}</option>{typeof currentAction === 'string' && !filtered.some((entry) => entry.id === currentAction) && <option value={currentAction}>{ACTIONS[currentAction] ?? 'Ação não decodificada' } — atual</option>}{action && !filtered.some((entry) => entry.id === action) && action !== currentAction && <option value={action}>{ACTIONS[action]} — rascunho</option>}{categories.map((name) => { const entries = filtered.filter((entry) => entry.category === name); return entries.length ? <optgroup key={name} label={name}>{entries.map((entry) => <option key={entry.id} value={entry.id} disabled={entry.disabled}>{entry.label}</option>)}</optgroup> : null; })}<option disabled>Macro — não implementada</option></select></label>
            <small>Teclas são posições HID: ABNT2/US e o layout do sistema determinam o caractere. Catálogo documentado; validação física parcial. F13–F24/ABNT2 extra pendentes.</small>
            <p className="x6-muted">{selected <= 2 ? 'Cliques primários protegidos — somente leitura.' : selected >= 6 ? `Superior ${selected - 5} · report 0x08, grupo ${selected - 1}. ${writable ? 'Um superior por operação, com backup, ACK e readback.' : 'Snapshot somente leitura; atualize o plugin validado.'}` : selected === 3 ? 'Somente leitura nesta fase de validação.' : 'Um lateral por operação. Backup antes da escrita.'}</p>
            <button type="button" className="x6-primary x6-wide" disabled={!writable || busy || !action || action === currentAction} onClick={() => apply(key, action, `Aplicar ${BUTTONS[selected - 1]}`)}>Aplicar botão selecionado</button>
            <button type="button" className="x6-wide" disabled={!writable || busy || original.current[key] === currentAction || original.current[key] === undefined} onClick={() => apply(key, 'restore_session', `Restaurar ${BUTTONS[selected - 1]}`)}>Restaurar botão</button>
            <small>Restaura a leitura inicial desta sessão, nunca um padrão de fábrica.</small>
          </section>
          <section className="x6-panel"><h3>Perfis</h3><div className="x6-profile-empty">Perfil físico: não implementado.<br/>Nenhum preset é aplicado automaticamente.</div><div className="x6-profile-actions">{['Salvar', 'Carregar', 'Importar', 'Exportar'].map((name) => <button key={name} type="button" disabled title="Presets locais pendentes">{name}</button>)}</div><button type="button" disabled className="x6-wide" title="Restauração arbitrária de arquivo não habilitada">Restaurar backup — pendente</button></section>
        </aside>
        <div className="x6-center"><MouseDiagram selected={selected} select={select}/><span className="x6-diagram-caption">ATTACK SHARK X6 <span>2,4 GHz · esquema original</span></span><label className="x6-test-input">Teste local do botão secundário<input aria-label="Campo de teste de botão secundário" placeholder="Clique aqui e pressione o botão" autoComplete="off" spellCheck={false} onKeyDown={(e) => { setLastKey(`${e.code} · Ctrl=${e.ctrlKey} Shift=${e.shiftKey} Alt=${e.altKey} Super=${e.metaKey}`); if (/^F\d+$/.test(e.code) || e.ctrlKey || e.altKey || e.metaKey) e.preventDefault(); }}/></label><small role="status">{lastKey}</small><small>Somente neste campo. Nenhuma captura global de teclas.</small></div>
        <aside className="x6-right">
          <section className="x6-panel"><h3>Sensibilidade</h3><div className="x6-dpi-stages">{Array.from({ length: 6 }, (_, i) => i + 1).map((n) => <div key={n}><button type="button" aria-label={`Selecionar estágio ${n}`} aria-pressed={dpi === n} className={Number(state?.values.dpi_stage) === n ? 'is-active-stage' : ''} onClick={() => setDpi(n)}>{n}</button><input type="range" aria-label={`DPI estágio ${n} slider`} min="800" max="26000" step="50" disabled={!connected || busy} value={Number(draft[`dpi_stage_${n}`] ?? 800)} onChange={(e) => { setDpi(n); setDraft((d) => ({ ...d, [`dpi_stage_${n}`]: Number(e.target.value) })); }}/><input type="number" aria-label={`DPI estágio ${n}`} min="800" max="26000" step="50" placeholder="—" value={draft[`dpi_stage_${n}`] === undefined ? '' : Number(draft[`dpi_stage_${n}`])} disabled={!connected || busy} onChange={(e) => { setDpi(n); setDraft((d) => ({ ...d, [`dpi_stage_${n}`]: e.target.value === '' ? null : Number(e.target.value) })); }}/></div>)}</div>
            <p className="x6-muted">Ativo no mouse: {connected ? String(state?.values.dpi_stage) : 'sem leitura'} · aplica somente o estágio selecionado.</p>
            <div className="x6-unavailable"><label><input type="checkbox" disabled/> Angle snapping — pendente</label><label><input type="checkbox" disabled/> Ripple control — pendente</label></div>
            <button type="button" className="x6-primary x6-wide" disabled={!connected || busy || typeof draft[dpiKey] !== 'number' || Number(draft[dpiKey]) < 800 || Number(draft[dpiKey]) > 26000 || Number(draft[dpiKey]) % 50 !== 0 || draft[dpiKey] === state?.values[dpiKey]} onClick={() => apply(dpiKey, draft[dpiKey], `Aplicar DPI estágio ${dpi}`)}>Aplicar DPI selecionado</button>
            <button type="button" className="x6-wide" disabled={!connected || busy || state?.values.dpi_stage === dpi} onClick={() => apply('dpi_stage', dpi, `Ativar estágio ${dpi}`)}>Ativar estágio {dpi}</button>
          </section>
          <section className="x6-panel"><div className="x6-panel-heading"><h3>Polling rate</h3><button type="button" disabled={!connected || busy || draft.polling_rate_hz === state?.values.polling_rate_hz} onClick={() => apply('polling_rate_hz', draft.polling_rate_hz, 'Aplicar polling')}>Aplicar polling</button></div><div className="x6-polling">{[125, 250, 500, 1000].map((hz) => <button key={hz} type="button" aria-pressed={draft.polling_rate_hz === hz} disabled={!connected || busy} onClick={() => setDraft((d) => ({ ...d, polling_rate_hz: hz }))}>{hz} Hz</button>)}</div><small>Lido: {connected ? `${state?.values.polling_rate_hz} Hz` : 'aguardando dispositivo'}</small></section>
          <section className="x6-panel"><h3>Resposta e energia</h3><div className="x6-energy"><span>Debounce</span><span>Não disponibilizado</span></div><input aria-label="Debounce indisponível" type="range" disabled/><p className="x6-muted">Suspensão e debounce: escrita não habilitada.</p><div className="x6-energy"><span>Bateria bruta</span><strong>{!connected || state?.values.battery_raw === -1 ? 'Não observada' : String(state?.values.battery_raw)}</strong></div><small>Escala de porcentagem não comprovada.</small><button type="button" className="x6-wide" disabled>Aplicar resposta e energia — pendente</button></section>
        </aside>
      </div>
    </div>
    <section role="tabpanel" id="x6-panel-1" aria-labelledby="x6-tab-1" hidden={tab !== 'Macros'} className="x6-panel x6-tab-content"><h3>Macros</h3><p>Não implementadas. Nenhum report de macro é enviado pelo plugin.</p><button type="button" disabled>Criar macro — indisponível</button></section>
    <section role="tabpanel" id="x6-panel-2" aria-labelledby="x6-tab-2" hidden={tab !== 'Diagnóstico'} className="x6-panel x6-tab-content"><h3>Diagnóstico físico</h3><dl>{Object.entries({ Plugin: 'dev.attack-shark-x6', Backend: connected ? 'Hardware real — sem fallback' : 'Desconectado / não confirmado', USB: state?.device ? `${state.device.vid}:${state.device.pid}` : 'Sem leitura', Interface: state?.device?.interface ?? 'Sem leitura', HID: state?.device?.hidraw ?? 'Sem leitura', Transporte: state?.device?.transport ?? 'Sem leitura', 'Endpoint esperado pelo plugin X6': 'IN 0x83 (validado na descoberta)', 'Report de botões': connected && state?.settings.some((s) => s.key === 'button_4') ? '0x08 · checksum/readback exigidos' : 'Sem leitura', 'Bateria bruta': connected ? state?.values.battery_raw : 'Sem leitura' }).map(([k, v]) => <div key={k}><dt>{k}</dt><dd>{String(v)}</dd></div>)}</dl><button type="button" disabled={!connected || busy} onClick={() => read('Diagnóstico: leitura física')}>Ler estado físico</button><h4>Operações desta sessão</h4><ol>{logs.map((line, i) => <li key={i}>{line}</li>)}</ol><p>Backups duráveis: ~/.local/state/attack-shark-x6-plugin/backups/. Nenhum reset ou firmware.</p></section>
    <section role="tabpanel" id="x6-panel-3" aria-labelledby="x6-tab-3" hidden={tab !== 'Iluminação'}>{tab === 'Iluminação' && <Lighting state={state} busy={busy} original={original.current} apply={apply}/>}</section>
    <footer className="x6-footer"><span>Configuração local · nenhuma alteração automática</span><span>Projeto independente · Linux{pluginId.startsWith('dev.') ? ' · DEV' : ''}</span></footer>
  </section>;
}

export const x6Integration: LocalSoftware = { id: 'dev.attack-shark-x6', name: 'Attack Shark X6', manufacturer: 'Attack Shark', category: 'mouse', available: true, workspace: true, coverage: 'partial', status: 'Parcial', Visual: X6CardArt, View: X6Control };
export const publicX6Integration: LocalSoftware = { ...x6Integration, id: 'io.github.extremecodex.attack-shark-x6', View: ({ onBack }) => <X6Control onBack={onBack} pluginId="io.github.extremecodex.attack-shark-x6"/> };
