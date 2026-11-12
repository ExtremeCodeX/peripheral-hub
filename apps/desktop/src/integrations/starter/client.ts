import { invoke, isTauri } from '@tauri-apps/api/core';

export type Setting = { key: string; access: ('read' | 'write')[]; effects: 'simulation_only'; schema: { type: string; enum?: (number | string)[]; minimum?: number; maximum?: number; multipleOf?: number } };
export type Snapshot = { mode: 'fake'; hardwareAccess: false; outcome: 'simulated'; session: string; baselineToken: string; values: Record<string, unknown>; settings: Setting[] };
let settings: Setting[] = [];
type Request = { action: string; [key: string]: unknown };
// One queue for complete transactions across mounts. Unmount's stop cannot
// interrupt half a Start/Open/Read transaction or overtake a later start.
let tail: Promise<unknown> = Promise.resolve();
function transaction<T>(task: () => Promise<T>): Promise<T> {
  const result = tail.then(task);
  tail = result.catch(() => undefined);
  return result;
}
async function request<T>(request: Request): Promise<T> {
  if (!isTauri()) throw new Error('DESKTOP_REQUIRED');
  return invoke<T>('starter_local', { request });
}
function snapshot(value: Snapshot): Snapshot {
  if (value.mode !== 'fake' || value.hardwareAccess !== false || value.outcome !== 'simulated'
      || !/^[a-f0-9]{32}$/.test(value.session) || !/^[a-f0-9]{32}$/.test(value.baselineToken)
       || !value.values || typeof value.values !== 'object' || Array.isArray(value.values)) throw new Error('INVALID_RESULT');
   return { ...value, settings };
}
export const starterClient = {
  start: (pluginId = 'dev.starter') => transaction(async () => {
    settings = [];
    await request(pluginId === 'dev.starter' ? { action: 'start' } : { action: 'start_plugin', plugin_id: pluginId });
    try {
      const capabilities = await request<{ settings: Setting[] }>({ action: 'capabilities' });
      if (!Array.isArray(capabilities.settings) || !capabilities.settings.length || capabilities.settings.length > 32
          || capabilities.settings.some((s) => !s.key || s.effects !== 'simulation_only' || !s.schema || !Array.isArray(s.access))) throw new Error('INVALID_RESULT');
      settings = capabilities.settings;
      const opened = await request<{ session: string }>({ action: 'open' });
      return snapshot(await request<Snapshot>({ action: 'read', session: opened.session }));
    } catch (error) { await request({ action: 'stop' }).catch(() => undefined); throw error; }
  }),
  read: (session: string) => transaction(async () => snapshot(await request<Snapshot>({ action: 'read', session }))),
  set: (state: Snapshot, value: unknown, key = 'sample_value') => transaction(async () => snapshot(await request<Snapshot>({ action: 'set_setting', session: state.session, baseline_token: state.baselineToken, key, value }))),
  stop: () => transaction(() => request({ action: 'stop' })),
};

export function starterError(error: unknown): string {
  const kind = String(error && typeof error === 'object' && 'kind' in error ? error.kind : error);
  if (/DESKTOP_REQUIRED|DEV_ONLY|not found|not allowed/i.test(kind)) return 'O starter requer o aplicativo Tauri debug. No navegador e em release não há execução nem mock automático.';
  if (kind.includes('SPAWN_FAILED')) return 'Não foi possível iniciar /usr/bin/python3. Nenhuma dependência foi instalada.';
  if (kind.includes('LOCAL_INTEGRATION_UNAVAILABLE')) return 'Snapshot local ausente. Prepare o starter com tools/install-local.py antes de iniciar.';
  if (/LOCAL_INTEGRATION_UNSAFE|LOCAL_SOURCE_INTEGRITY/.test(kind)) return 'Snapshot local recusado. Confira permissões, tamanho e armazenamento local sem symlinks.';
  if (kind.includes('NATIVE_CONFIRMATION_DENIED')) return 'Execução local cancelada. Nenhum processo foi iniciado.';
  if (/NATIVE_CONFIRMATION_REQUIRED|NATIVE_CONFIRMATION_UNAVAILABLE/.test(kind)) return 'A confirmação nativa é obrigatória antes de executar código local.';
  return 'Falha no processo local. Estado descartado; revise o código antes de iniciar uma nova sessão.';
}
