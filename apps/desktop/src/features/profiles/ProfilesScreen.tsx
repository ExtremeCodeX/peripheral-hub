import { useState } from "react";

import { EmptyNotice, ErrorNotice } from "../../components/StateNotice";
import { useDaemon } from "../../hooks/useDaemon";
import { errorMessage } from "../../services/daemonController";
import type { LightingEffect, Profile } from "../../types/ipc";
import { emptyProfile, parseProfile, serializeProfile, validateProfile } from "./profileFile";

function clampByte(value: string): number {
  const parsed = Number.parseInt(value, 10);
  if (Number.isNaN(parsed)) {
    return 0;
  }
  return Math.min(255, Math.max(0, parsed));
}

export function ProfilesScreen() {
  const { state, controller } = useDaemon();
  const [draft, setDraft] = useState<Profile>(emptyProfile());
  const [lightingEnabled, setLightingEnabled] = useState(false);
  const [zone, setZone] = useState("whole-device");
  const [red, setRed] = useState("255");
  const [green, setGreen] = useState("0");
  const [blue, setBlue] = useState("0");
  const [effect, setEffect] = useState<LightingEffect>("static");
  const [io, setIo] = useState("");
  const [applyTarget, setApplyTarget] = useState("");
  const [message, setMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const connected = state.devices.filter((device) => device.status === "connected");
  const targetId = connected.some((device) => device.interface_id === applyTarget)
    ? applyTarget
    : (connected[0]?.interface_id ?? "");

  function buildProfile(): Profile {
    const model = draft.device_model?.trim() ?? "";
    return {
      id: draft.id.trim(),
      name: draft.name,
      device_model: model.length > 0 ? model : null,
      lighting: lightingEnabled
        ? {
            zone,
            color: { red: clampByte(red), green: clampByte(green), blue: clampByte(blue) },
            brightness: null,
            effect,
          }
        : null,
    };
  }

  async function save(): Promise<void> {
    setError(null);
    setMessage(null);
    const profile = buildProfile();
    const problems = validateProfile(profile);
    if (problems.length > 0) {
      setError(problems.join(" "));
      return;
    }
    setBusy(true);
    try {
      await controller.saveProfile(profile);
      setMessage(`Saved profile “${profile.name}”.`);
    } catch (cause) {
      setError(errorMessage(cause));
    } finally {
      setBusy(false);
    }
  }

  async function remove(id: string): Promise<void> {
    setError(null);
    setMessage(null);
    try {
      await controller.deleteProfile(id);
      setMessage(`Deleted profile “${id}”.`);
    } catch (cause) {
      setError(errorMessage(cause));
    }
  }

  async function apply(id: string): Promise<void> {
    setError(null);
    setMessage(null);
    if (targetId.length === 0) {
      setError("Connect a device before applying a profile.");
      return;
    }
    setBusy(true);
    try {
      const outcome = await controller.applyProfile(targetId, id);
      setMessage(`Applied profile “${id}” (${outcome.mode}).`);
    } catch (cause) {
      setError(errorMessage(cause));
    } finally {
      setBusy(false);
    }
  }

  function edit(profile: Profile): void {
    setDraft(profile);
    setLightingEnabled(profile.lighting !== null);
    if (profile.lighting) {
      setZone(profile.lighting.zone);
      setRed(String(profile.lighting.color.red));
      setGreen(String(profile.lighting.color.green));
      setBlue(String(profile.lighting.color.blue));
      setEffect(profile.lighting.effect);
    }
  }

  async function editFromList(id: string): Promise<void> {
    setError(null);
    try {
      edit(await controller.getProfile(id));
    } catch (cause) {
      setError(errorMessage(cause));
    }
  }

  async function importFromIo(): Promise<void> {
    setError(null);
    setMessage(null);
    try {
      const profile = parseProfile(io);
      edit(profile);
      await controller.saveProfile(profile);
      setMessage(`Imported profile “${profile.name}”.`);
    } catch (cause) {
      setError(errorMessage(cause));
    }
  }

  function exportToIo(): void {
    setError(null);
    try {
      setIo(serializeProfile(buildProfile()));
      setMessage("Serialized the current form to versioned JSON.");
    } catch (cause) {
      setError(errorMessage(cause));
    }
  }

  return (
    <section>
      <div className="section-head">
        <h2>Profiles</h2>
        <button type="button" onClick={() => void controller.refreshProfiles()}>
          Refresh
        </button>
      </div>

      {state.profilesError ? <ErrorNotice message={state.profilesError} /> : null}
      {error ? <ErrorNotice message={error} /> : null}
      {message ? (
        <p className="notice" role="status">
          {message}
        </p>
      ) : null}

      <div className="apply-target">
        <label className="field">
          <span>Apply to connected device</span>
          <select
            value={targetId}
            onChange={(event) => setApplyTarget(event.target.value)}
            disabled={connected.length === 0}
          >
            {connected.length === 0 ? <option value="">No connected device</option> : null}
            {connected.map((device) => (
              <option key={device.interface_id} value={device.interface_id}>
                {device.product ?? device.interface_id}
              </option>
            ))}
          </select>
        </label>
      </div>

      {state.profiles.length === 0 ? (
        <EmptyNotice title="No profiles yet" hint="Create one with the form below." />
      ) : (
        <ul className="profile-list">
          {state.profiles.map((profile) => (
            <li key={profile.id} className="profile-row">
              <span className="profile-name">{profile.name}</span>
              <span className="profile-id">{profile.id}</span>
              {profile.device_model ? <span className="profile-model">{profile.device_model}</span> : null}
              <span className="profile-actions">
                <button
                  type="button"
                  disabled={busy || connected.length === 0}
                  onClick={() => void apply(profile.id)}
                >
                  Apply
                </button>
                <button type="button" onClick={() => void editFromList(profile.id)}>
                  Edit
                </button>
                <button type="button" onClick={() => void remove(profile.id)}>
                  Delete
                </button>
              </span>
            </li>
          ))}
        </ul>
      )}

      <h3>Create or edit</h3>
      <form
        className="form"
        onSubmit={(event) => {
          event.preventDefault();
          void save();
        }}
      >
        <label className="field">
          <span>Id</span>
          <input value={draft.id} onChange={(event) => setDraft({ ...draft, id: event.target.value })} />
        </label>
        <label className="field">
          <span>Name</span>
          <input value={draft.name} onChange={(event) => setDraft({ ...draft, name: event.target.value })} />
        </label>
        <label className="field">
          <span>Device model (optional)</span>
          <input
            value={draft.device_model ?? ""}
            onChange={(event) => setDraft({ ...draft, device_model: event.target.value })}
          />
        </label>

        <label className="field checkbox">
          <input
            type="checkbox"
            checked={lightingEnabled}
            onChange={(event) => setLightingEnabled(event.target.checked)}
          />
          <span>Include lighting preference</span>
        </label>

        {lightingEnabled ? (
          <fieldset className="rgb">
            <legend>Lighting</legend>
            <label className="field">
              <span>Zone</span>
              <input value={zone} onChange={(event) => setZone(event.target.value)} />
            </label>
            <label className="field">
              <span>Red</span>
              <input type="number" min={0} max={255} value={red} onChange={(event) => setRed(event.target.value)} />
            </label>
            <label className="field">
              <span>Green</span>
              <input type="number" min={0} max={255} value={green} onChange={(event) => setGreen(event.target.value)} />
            </label>
            <label className="field">
              <span>Blue</span>
              <input type="number" min={0} max={255} value={blue} onChange={(event) => setBlue(event.target.value)} />
            </label>
            <label className="field">
              <span>Effect</span>
              <select value={effect} onChange={(event) => setEffect(event.target.value as LightingEffect)}>
                <option value="static">static</option>
                <option value="breathing">breathing</option>
                <option value="color_cycle">color_cycle</option>
              </select>
            </label>
          </fieldset>
        ) : null}

        <div className="form-actions">
          <button type="submit" disabled={busy}>
            Save profile
          </button>
          <button type="button" onClick={() => setDraft(emptyProfile())}>
            New
          </button>
          <button type="button" onClick={exportToIo}>
            Export JSON
          </button>
        </div>
      </form>

      <h3>Import / export (versioned JSON)</h3>
      <textarea
        className="io"
        aria-label="Versioned profile JSON"
        value={io}
        onChange={(event) => setIo(event.target.value)}
        rows={8}
      />
      <div className="form-actions">
        <button type="button" onClick={() => void importFromIo()}>
          Import and save
        </button>
      </div>
    </section>
  );
}
