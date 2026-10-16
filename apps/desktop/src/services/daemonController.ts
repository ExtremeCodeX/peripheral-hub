import type { DaemonStore } from "../stores/daemonStore";
import type {
  Capabilities,
  CommandOutcome,
  DeviceState,
  DeviceSummary,
  LightingCommand,
  LightingScene,
  Profile,
} from "../types/ipc";
import { BridgeError, DaemonFaultError } from "./errors";
import type { IpcClient } from "./ipcClient";

export function errorMessage(error: unknown): string {
  if (error instanceof DaemonFaultError || error instanceof BridgeError) {
    return error.message;
  }
  if (error instanceof Error) {
    return error.message;
  }
  return "Unexpected error.";
}

/**
 * Orchestrates the daemon client and pushes results into the observable store.
 *
 * Device/profile mutations are surfaced as thrown errors so a screen can show a
 * precise message; list refreshes update the store. Nothing here retries a
 * mutation automatically after a timeout or disconnect.
 */
export class DaemonController {
  constructor(
    private readonly client: IpcClient,
    private readonly store: DaemonStore,
  ) {}

  async initialize(): Promise<void> {
    this.store.dispatch({ type: "connecting" });
    try {
      const health = await this.client.health();
      this.store.dispatch({ type: "health", health });
      this.store.dispatch({ type: "connected" });
    } catch (error) {
      this.store.dispatch({ type: "connectionFailed", message: errorMessage(error) });
      return;
    }
    await this.refreshDevices();
    await this.refreshProfiles();
  }

  async refreshHealth(): Promise<void> {
    const health = await this.client.health();
    this.store.dispatch({ type: "health", health });
  }

  async refreshDevices(): Promise<void> {
    try {
      const devices = await this.client.listDevices();
      this.store.dispatch({ type: "devices", devices });
    } catch (error) {
      this.store.dispatch({ type: "devicesError", message: errorMessage(error) });
    }
  }

  async refreshProfiles(): Promise<void> {
    try {
      const profiles = await this.client.listProfiles();
      this.store.dispatch({ type: "profiles", profiles });
    } catch (error) {
      this.store.dispatch({ type: "profilesError", message: errorMessage(error) });
    }
  }

  async connectDevice(interfaceId: string): Promise<DeviceSummary> {
    const device = await this.client.connect(interfaceId);
    await this.refreshDevices();
    return device;
  }

  async disconnectDevice(interfaceId: string): Promise<void> {
    await this.client.disconnect(interfaceId);
    await this.refreshDevices();
  }

  getDevice(interfaceId: string): Promise<DeviceSummary> {
    return this.client.getDevice(interfaceId);
  }

  async setSimulatedPresence(present: boolean): Promise<void> {
    await this.client.simulatePresence(present);
    await this.refreshDevices();
  }

  getCapabilities(interfaceId: string): Promise<Capabilities> {
    return this.client.getCapabilities(interfaceId);
  }

  readState(interfaceId: string): Promise<DeviceState> {
    return this.client.readState(interfaceId);
  }

  setLighting(interfaceId: string, command: LightingCommand): Promise<CommandOutcome> {
    return this.client.setLighting(interfaceId, command);
  }

  setLightingScene(interfaceId: string, scene: LightingScene): Promise<CommandOutcome> {
    return this.client.setLightingScene(interfaceId, scene);
  }

  getProfile(profileId: string): Promise<Profile> {
    return this.client.getProfile(profileId);
  }

  async saveProfile(profile: Profile): Promise<Profile> {
    const saved = await this.client.saveProfile(profile);
    await this.refreshProfiles();
    return saved;
  }

  async deleteProfile(profileId: string): Promise<void> {
    await this.client.deleteProfile(profileId);
    await this.refreshProfiles();
  }

  applyProfile(interfaceId: string, profileId: string): Promise<CommandOutcome> {
    return this.client.applyProfile(interfaceId, profileId);
  }
}
