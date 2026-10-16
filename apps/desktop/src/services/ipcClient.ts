import {
  IPC_VERSION,
  isReply,
  type Capabilities,
  type CommandOutcome,
  type DeviceState,
  type DeviceSummary,
  type EventBatch,
  type Health,
  type LightingCommand,
  type Operation,
  type Profile,
  type ProfileSummary,
  type Reply,
} from "../types/ipc";
import { DaemonFaultError } from "./errors";
import type { DaemonTransport } from "./transport";

function unexpected(expected: Reply["result"], actual: Reply["result"]): Error {
  return new Error(`unexpected daemon reply: expected ${expected}, received ${actual}`);
}

/**
 * Thin typed wrapper over a `DaemonTransport`.
 *
 * It unwraps the versioned `Response`, verifies the wire version, converts a
 * `Fault` into a typed `DaemonFaultError`, and asserts each reply discriminant.
 * It performs no business logic; screens and the controller consume it.
 */
export class IpcClient {
  private nextRequestId = 0;

  constructor(private readonly transport: DaemonTransport) {}

  private async send(operation: Operation): Promise<Reply> {
    this.nextRequestId += 1;
    const response = await this.transport.call(operation);
    if (response.version !== IPC_VERSION) {
      throw new DaemonFaultError({ code: "incompatible_version" });
    }
    if ("Err" in response.result) {
      throw new DaemonFaultError(response.result.Err);
    }
    return response.result.Ok;
  }

  async health(): Promise<Health> {
    const reply = await this.send({ operation: "health" });
    if (isReply(reply, "health")) {
      return reply.data;
    }
    throw unexpected("health", reply.result);
  }

  async listDevices(): Promise<DeviceSummary[]> {
    const reply = await this.send({ operation: "list_devices" });
    if (isReply(reply, "devices")) {
      return reply.data;
    }
    throw unexpected("devices", reply.result);
  }

  async getDevice(interfaceId: string): Promise<DeviceSummary> {
    const reply = await this.send({
      operation: "get_device",
      parameters: { interface_id: interfaceId },
    });
    if (isReply(reply, "device")) {
      return reply.data;
    }
    throw unexpected("device", reply.result);
  }

  async getCapabilities(interfaceId: string): Promise<Capabilities> {
    const reply = await this.send({
      operation: "get_capabilities",
      parameters: { interface_id: interfaceId },
    });
    if (isReply(reply, "capabilities")) {
      return reply.data;
    }
    throw unexpected("capabilities", reply.result);
  }

  async connect(interfaceId: string): Promise<DeviceSummary> {
    const reply = await this.send({
      operation: "connect",
      parameters: { interface_id: interfaceId },
    });
    if (isReply(reply, "device")) {
      return reply.data;
    }
    throw unexpected("device", reply.result);
  }

  async disconnect(interfaceId: string): Promise<void> {
    const reply = await this.send({
      operation: "disconnect",
      parameters: { interface_id: interfaceId },
    });
    if (isReply(reply, "ack")) {
      return;
    }
    throw unexpected("ack", reply.result);
  }

  async readState(interfaceId: string): Promise<DeviceState> {
    const reply = await this.send({
      operation: "read_state",
      parameters: { interface_id: interfaceId },
    });
    if (isReply(reply, "state")) {
      return reply.data;
    }
    throw unexpected("state", reply.result);
  }

  async setLighting(interfaceId: string, command: LightingCommand): Promise<CommandOutcome> {
    const reply = await this.send({
      operation: "set_lighting",
      parameters: { interface_id: interfaceId, command },
    });
    if (isReply(reply, "outcome")) {
      return reply.data;
    }
    throw unexpected("outcome", reply.result);
  }

  async simulatePresence(present: boolean): Promise<void> {
    const reply = await this.send({ operation: "simulate_presence", parameters: { present } });
    if (isReply(reply, "ack")) {
      return;
    }
    throw unexpected("ack", reply.result);
  }

  async events(after: number): Promise<EventBatch> {
    const reply = await this.send({ operation: "events", parameters: { after } });
    if (isReply(reply, "events")) {
      return reply.data;
    }
    throw unexpected("events", reply.result);
  }

  async listProfiles(): Promise<ProfileSummary[]> {
    const reply = await this.send({ operation: "list_profiles" });
    if (isReply(reply, "profiles")) {
      return reply.data;
    }
    throw unexpected("profiles", reply.result);
  }

  async getProfile(profileId: string): Promise<Profile> {
    const reply = await this.send({
      operation: "get_profile",
      parameters: { profile_id: profileId },
    });
    if (isReply(reply, "profile")) {
      return reply.data;
    }
    throw unexpected("profile", reply.result);
  }

  async saveProfile(profile: Profile): Promise<Profile> {
    const reply = await this.send({
      operation: "save_profile",
      parameters: { profile },
    });
    if (isReply(reply, "profile")) {
      return reply.data;
    }
    throw unexpected("profile", reply.result);
  }

  async deleteProfile(profileId: string): Promise<void> {
    const reply = await this.send({
      operation: "delete_profile",
      parameters: { profile_id: profileId },
    });
    if (isReply(reply, "ack")) {
      return;
    }
    throw unexpected("ack", reply.result);
  }
}
