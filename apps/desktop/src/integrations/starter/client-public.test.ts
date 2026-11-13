import { expect, it, vi } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { starterClient } from './client';
vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn(), isTauri: () => true }));

it('routes the installed public X6 through the release host, retaining per-operation authorization', async () => {
  vi.mocked(invoke).mockImplementation(async (command, args) => {
    expect(command).toBe('plugin_public');
    const request = (args as { request: { action: string; development?: boolean; authorization?: boolean } }).request;
    if (request.action === 'start_hardware') expect(request.development).toBe(false);
    if (request.action === 'capabilities') return { settings: [{ key: 'gain', access: ['read', 'write'], effects: 'active_read_and_configuration_write', schema: { type: 'integer' } }] };
    if (request.action === 'discover') return { candidates: [{ candidateId: 'c'.repeat(32) }] };
    if (request.action === 'open_candidate') { expect(request.authorization).toBe(true); return { session: 'a'.repeat(32) }; }
    if (request.action === 'read_active' || request.action === 'apply') {
      expect(request.authorization).toBe(true);
      return { mode: 'hardware', hardwareAccess: true, outcome: request.action === 'apply' ? 'confirmed_by_readback' : 'read_from_device', session: 'a'.repeat(32), baselineToken: 'b'.repeat(32), values: { gain: 1 } };
    }
    return {};
  });
  const state = await starterClient.start('io.github.extremecodex.attack-shark-x6', true);
  await starterClient.set(state, 2, 'gain');
  await starterClient.stop();
  expect(invoke).toHaveBeenCalledWith('plugin_public', { request: { action: 'start_hardware', plugin_id: 'io.github.extremecodex.attack-shark-x6', development: false } });
});
