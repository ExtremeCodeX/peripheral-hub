import { invoke, isTauri } from '@tauri-apps/api/core';

export type Setting = { key: string; access: ('read' | 'write')[]; effects: 'simulation_only' | 'active_read' | 'active_read_and_configuration_write'; schema: { type: string; enum?: (number | string)[]; minimum?: number; maximum?: number; multipleOf?: number } };
export type Candidate = { candidateId: string; transport: string; vid: string; pid: string; revision: string; interface: string; hidraw: string; identity: string };
export type Snapshot = { mode: 'fake' | 'hardware'; hardwareAccess: boolean; outcome: 'simulated' | 'read_from_device' | 'confirmed_by_readback' | 'unchanged'; session: string; baselineToken: string; baselineTokens?: Record<string,string>; values: Record<string, unknown>; settings: Setting[]; device?: Candidate };
let settings: Setting[] = [];
let hardwareMode = false;
let publicMode = false;
let device: Candidate | undefined;
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
  return invoke<T>(publicMode ? 'plugin_public' : 'starter_local', { request: publicMode && request.action === 'start_hardware' ? { ...request, development: false } : request });
}
function snapshot(value: Snapshot): Snapshot {
  if ((hardwareMode ? value.mode !== 'hardware' || value.hardwareAccess !== true || !['read_from_device','confirmed_by_readback','unchanged'].includes(value.outcome)
      : value.mode !== 'fake' || value.hardwareAccess !== false || value.outcome !== 'simulated')
      || !/^[a-f0-9]{32}$/.test(value.session) || !/^[a-f0-9]{32}$/.test(value.baselineToken)
       || !value.values || typeof value.values !== 'object' || Array.isArray(value.values)) throw new Error('INVALID_RESULT');
   return { ...value, settings, device };
}
export const starterClient = {
  start: (pluginId = 'dev.starter', hardware = false) => transaction(async () => {
    settings = [];
    device = undefined;
    hardwareMode = hardware;
    publicMode = !pluginId.startsWith('dev.');
    await request(hardware ? { action: 'start_hardware', plugin_id: pluginId } : pluginId === 'dev.starter' ? { action: 'start' } : { action: 'start_plugin', plugin_id: pluginId });
    try {
      const capabilities = await request<{ settings: Setting[] }>({ action: 'capabilities' });
      if (!Array.isArray(capabilities.settings) || !capabilities.settings.length || capabilities.settings.length > 32
          || capabilities.settings.some((s) => !s.key || !(hardware ? ['active_read','active_read_and_configuration_write'] : ['simulation_only']).includes(s.effects) || !s.schema || !Array.isArray(s.access))) throw new Error('INVALID_RESULT');
      settings = capabilities.settings;
      let candidateId = '';
      if (hardware) {
        const discovery = await request<{ candidates: Candidate[] }>({ action: 'discover' });
        if (discovery.candidates.length === 0) throw new Error('DEVICE_NOT_FOUND');
        if (discovery.candidates.length !== 1) throw new Error('IDENTITY_UNCONFIRMED');
        candidateId = discovery.candidates[0].candidateId;
        device = discovery.candidates[0];
      }
      const opened = await request<{ session: string }>(hardware ? { action: 'open_candidate', candidate_id: candidateId, authorization: true } : { action: 'open' });
      return snapshot(await request<Snapshot>(hardware ? { action: 'read_active', session: opened.session, authorization: true } : { action: 'read', session: opened.session }));
    } catch (error) { await request({ action: 'stop' }).catch(() => undefined); throw error; }
  }),
  read: (session: string) => transaction(async () => snapshot(await request<Snapshot>(hardwareMode ? { action: 'read_active', session, authorization: true } : { action: 'read', session }))),
  set: (state: Snapshot, value: unknown, key = 'sample_value') => transaction(async () => snapshot(await request<Snapshot>({ action: state.hardwareAccess ? 'apply' : 'set_setting', session: state.session, baseline_token: state.baselineTokens?.[key] ?? state.baselineToken, key, value, ...(state.hardwareAccess ? { authorization: true } : {}) }))),
  stop: () => transaction(() => request({ action: 'stop' })),
};

export function starterError(error: unknown): string {
  const kind = String(error && typeof error === 'object' && 'kind' in error ? error.kind : error);
  if (kind.includes('DEVICE_NOT_FOUND')) return 'Desconectado ou dispositivo não identificado. Confira conexão e modelo; nenhuma simulação será usada.';
  if (kind.includes('IDENTITY_UNCONFIRMED')) return 'Dispositivo não identificado inequivocamente ou múltiplos candidatos. Operação recusada.';
  if (kind.includes('PERMISSION_DENIED')) return 'Erro de permissão HID. Nenhuma regra udev ou permissão foi alterada.';
  if (kind.includes('DEVICE_DISCONNECTED')) return 'Dispositivo desconectado. Sessão descartada; reconecte e inicie novamente, sem replay.';
  if (kind.includes('READ_PREPARATION_FAILED')) return 'Falha de comunicação: grant de leitura recusado. Acorde o mouse e inicie uma nova sessão.';
  if (kind.includes('ACK_FAILED')) return 'ACK ausente ou rejeitado. A escrita pode ter ocorrido; não repita automaticamente. Confira o backup e faça uma nova leitura autorizada.';
  if (/VERIFICATION_FAILED|DEVICE_IO_FAILED|BACKUP_FAILED/.test(kind)) return `Falha física (${kind}). Não repita a escrita automaticamente. Confira o dispositivo e o backup; resultado pode ser desconhecido.`;
  if (kind.includes('INVALID_HANDSHAKE')) return 'Modo do snapshot incompatível. Selecione hardware para fonte física ou instale o snapshot simulado.';
  if (/DESKTOP_REQUIRED|DEV_ONLY|not found|not allowed/i.test(kind)) return 'O starter requer o aplicativo Tauri debug. No navegador e em release não há execução nem mock automático.';
  if (kind.includes('SPAWN_FAILED')) return 'Não foi possível iniciar /usr/bin/python3. Nenhuma dependência foi instalada.';
  if (kind.includes('LOCAL_INTEGRATION_UNAVAILABLE')) return 'Snapshot local ausente. Prepare o starter com tools/install-local.py antes de iniciar.';
  if (/LOCAL_INTEGRATION_UNSAFE|LOCAL_SOURCE_INTEGRITY/.test(kind)) return 'Snapshot local recusado. Confira permissões, tamanho e armazenamento local sem symlinks.';
  if (kind.includes('NATIVE_CONFIRMATION_DENIED')) return 'Operação cancelada na confirmação nativa. Nenhum processo foi iniciado se o cancelamento ocorreu no início; operações seguintes canceladas não são enviadas.';
  if (kind.includes('PERMISSION_STORAGE_FAILED')) return 'Não foi possível salvar a autorização. Confira o diretório de configuração do Hub e tente conectar novamente.';
  if (/NATIVE_CONFIRMATION_REQUIRED|NATIVE_CONFIRMATION_UNAVAILABLE/.test(kind)) return 'A confirmação nativa é obrigatória antes de executar código local.';
  return 'Falha no processo local. Estado descartado; revise o código antes de iniciar uma nova sessão.';
}
