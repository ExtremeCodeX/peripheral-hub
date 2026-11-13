import { useEffect, useRef, useState } from 'react';
import type { LocalSoftware } from '../../app/local-software';
import { SettingsPanel } from '../../ui/SettingsPanel';
import './preview.css';
import { starterClient, starterError, type Snapshot } from './client';

export function StarterPreview({ onBack }: { onBack: () => void }) {
  const title = useRef<HTMLHeadingElement>(null);
  const [state, setState] = useState<Snapshot | null>(null);
  const [value, setValue] = useState<Record<string, unknown>>({ sample_value: 50 });
  const [pluginId, setPluginId] = useState('dev.starter');
  const [editingId, setEditingId] = useState('dev.starter');
  const [hardware, setHardware] = useState(false);
  const [identityConfirmed, setIdentityConfirmed] = useState(false);
  const [appliedKey, setAppliedKey] = useState<string | null>(null);
  const [invalid, setInvalid] = useState<Record<string, boolean>>({});
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');
  const generation = useRef(0);
  useEffect(() => {
    title.current?.focus();
    return () => { generation.current++; void starterClient.stop().catch(() => undefined); };
  }, []);
  async function run(task: () => Promise<Snapshot | null>, key: string | null = null) {
    const current = ++generation.current;
    setBusy(true); setError('');
    try {
      const result = await task();
      if (current !== generation.current) return;
      setState(result); setValue(result?.values ?? { sample_value: 50 }); setInvalid({});
      setAppliedKey(key);
    } catch (failure) {
      if (current !== generation.current) return;
      setState(null); setError(starterError(failure));
      await starterClient.stop().catch(() => undefined);
    } finally { if (current === generation.current) setBusy(false); }
  }
  return <section className="starter-preview" aria-labelledby="starter-title">
    <button className="text-button" type="button" onClick={onBack}>Voltar</button>
    <h2 id="starter-title" ref={title} tabIndex={-1}>{hardware ? 'Integração local — hardware' : 'Starter local — simulado'}</h2>
    <p>{hardware ? 'DEV · Configuração física · Sem fallback para simulação.' : 'DEV · Processo Python JSON-RPC real · Estado simulado, sem hardware.'}</p>
    <form onSubmit={(event) => { event.preventDefault(); setPluginId(editingId); }}>
      <label htmlFor="local-plugin-id">Integração local registrada</label>{' '}
      <input id="local-plugin-id" value={editingId} disabled={busy || !!state} maxLength={64} pattern="dev\.[a-z0-9.-]+" onChange={(event) => setEditingId(event.target.value)} />
      <button className="text-button" type="submit" disabled={busy || !!state || editingId === pluginId}>Selecionar integração</button>
      <p>Selecionada: {pluginId}. Registre a fonte com tools/install-local.py antes de iniciar.</p>
    </form>
    <label><input type="checkbox" checked={hardware} disabled={busy || !!state} onChange={(e) => { setHardware(e.target.checked); setIdentityConfirmed(false); }} /> Backend físico (snapshot hardware necessário)</label>
    {hardware && <label><input type="checkbox" checked={identityConfirmed} disabled={busy || !!state} onChange={(e) => setIdentityConfirmed(e.target.checked)} /> Confirmei o modelo físico e a conexão do periférico selecionado. VID/PID pode ser compartilhado.</label>}
    <SettingsPanel id="starter-setting-title" title={hardware ? 'Configurações do dispositivo' : 'Valor de exemplo'} status={busy ? 'Aguardando processo…' : state ? hardware ? 'Conectado · leitura física validada' : 'Sessão simulada aberta' : 'Processo parado'}>
      <p>{hardware ? 'Abrir, ler e aplicar exigem confirmação GTK por operação. Aplicar escreve no periférico e exige readback; backup é responsabilidade do plugin.' : 'O starter fornecido altera somente memória.'} Iniciar exige confirmação nativa: código local modificado executa com suas permissões, sem sandbox.</p>
      {(state?.settings ?? (hardware ? [] : [{ key: 'sample_value', access: ['read', 'write'], effects: 'simulation_only', schema: { type: 'integer', enum: [0, 25, 50, 75, 100] } }])).map((setting) => {
        const id = `setting-${setting.key}`;
        const disabled = !state || busy || !setting.access.includes('write');
        const draft = value[setting.key];
        const update = (next: unknown) => setValue((values) => ({ ...values, [setting.key]: next }));
        const numeric = setting.schema.type === 'integer' || setting.schema.type === 'number';
        const options = setting.schema.enum;
        const regular = numeric && options && Number(options[1]) > Number(options[0]) && options.every((v, i) => typeof v === 'number' && (!i || v === Number(options[0]) + i * (Number(options[1]) - Number(options[0]))));
        return <div className="starter-preview__field" key={setting.key}>
          <label htmlFor={id}>{setting.key === 'sample_value' ? 'Rascunho simulado' : setting.key} <output htmlFor={id}>{typeof draft === 'object' ? JSON.stringify(draft) : String(draft ?? '')}</output></label>
          {regular && options && options.length > 1 ? <input id={id} type="range" min={Number(options[0])} max={Number(options.at(-1))} step={Number(options[1]) - Number(options[0])} value={Number(draft)} disabled={disabled} onChange={(e) => update(Number(e.target.value))} />
            : options ? <select id={id} value={JSON.stringify(draft)} disabled={disabled} onChange={(e) => update(JSON.parse(e.target.value))}>{options.map((v) => <option key={JSON.stringify(v)} value={JSON.stringify(v)}>{v}</option>)}</select>
            : setting.schema.type === 'boolean' ? <input id={id} type="checkbox" checked={draft === true} disabled={disabled} onChange={(e) => update(e.target.checked)} />
            : numeric ? <input id={id} type="number" min={setting.schema.minimum} max={setting.schema.maximum} step={setting.schema.multipleOf} value={Number(draft)} disabled={disabled} onChange={(e) => update(e.target.value === '' ? null : Number(e.target.value))} />
            : <textarea id={id} key={state?.baselineToken ?? 'stopped'} defaultValue={JSON.stringify(draft, null, 2)} disabled={disabled} aria-invalid={invalid[setting.key] || undefined} aria-describedby={invalid[setting.key] ? `${id}-error` : undefined} onChange={(e) => { try { update(JSON.parse(e.target.value)); setInvalid((errors) => ({ ...errors, [setting.key]: false })); } catch { setInvalid((errors) => ({ ...errors, [setting.key]: true })); } }} />}
          {invalid[setting.key] && <p id={`${id}-error`} role="alert">JSON inválido. Corrija o rascunho antes de aplicar.</p>}
          <button className="text-button" type="button" disabled={disabled || invalid[setting.key] || JSON.stringify(draft) === JSON.stringify(state?.values[setting.key])} onClick={() => state && void run(() => starterClient.set(state, draft, setting.key), setting.key)}>{hardware ? 'Aplicar no dispositivo' : 'Aplicar no simulador'}</button>
          <p role="status">{state ? `Lido do processo: ${typeof state.values[setting.key] === 'object' ? JSON.stringify(state.values[setting.key]) : String(state.values[setting.key] ?? 'não disponível')} · ${state.hardwareAccess ? state.outcome === 'confirmed_by_readback' && appliedKey === setting.key ? 'escrita confirmada por readback físico' : state.outcome === 'unchanged' && appliedKey === setting.key ? 'sem alteração física' : 'lido do dispositivo físico' : 'resultado simulado'}.` : 'Inicie o processo para ler seu estado. Parar ou sair encerra a sessão.'}</p>
        </div>;
      })}
      <div className="starter-preview__actions">
        <button className="text-button" type="button" disabled={busy || (!state && hardware && !identityConfirmed)} onClick={() => void run(state ? async () => { await starterClient.stop(); return null; } : () => starterClient.start(pluginId, hardware))}>{state ? 'Parar processo' : 'Iniciar processo'}</button>
        <button className="text-button" type="button" disabled={!state || busy} onClick={() => state && void run(() => starterClient.read(state.session))}>Ler estado</button>
      </div>
      {error && <p role="alert">{error}</p>}
    </SettingsPanel>
    <p>Registro local DEV, não aprovação de catálogo. Prepare o snapshot com tools/install-local.py; edite a cópia local e reinicie o processo, sem recompilar. Caminhos e URLs não são aceitos pelo IPC.</p>
  </section>;
}

export const starterPreviewIntegration: LocalSoftware = {
  id: 'dev.starter', name: 'Starter local — simulado', manufacturer: 'Desenvolvimento local',
  status: 'JSON-RPC · Simulação sem hardware', available: true, category: 'other', View: StarterPreview,
};
