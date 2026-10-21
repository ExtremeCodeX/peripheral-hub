import { useState } from "react";

import { EmptyNotice, ErrorNotice } from "../../components/StateNotice";
import { useDaemon } from "../../hooks/useDaemon";
import { errorMessage } from "../../services/daemonController";
import type { KeyPress, Macro, MacroAction } from "./macroModel";
import { emptyMacro, fromMacroFile, simulateMacro, toMacroFile, validateMacro } from "./macroModel";

export function MacrosScreen() {
  const { state, controller } = useDaemon();
  const [draft, setDraft] = useState<Macro>(emptyMacro());
  const [io, setIo] = useState("");
  const [trace, setTrace] = useState<string[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [message, setMessage] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const problems = validateMacro(draft);

  function buildMacro(): Macro {
    const profile = draft.profile?.trim() ?? "";
    return {
      id: draft.id.trim(),
      name: draft.name,
      actions: draft.actions,
      repeat: {
        times: draft.repeat.times,
        interval_ms: draft.repeat.interval_ms,
      },
      profile: profile.length > 0 ? profile : null,
    };
  }

  function setAction(index: number, action: MacroAction): void {
    setDraft({
      ...draft,
      actions: draft.actions.map((candidate, position) => (position === index ? action : candidate)),
    });
  }

  function addAction(): void {
    setDraft({ ...draft, actions: [...draft.actions, { kind: "delay", ms: 50 }] });
  }

  function removeAction(index: number): void {
    setDraft({ ...draft, actions: draft.actions.filter((_, position) => position !== index) });
  }

  function edit(macro: Macro): void {
    setDraft(macro);
  }

  function simulate(): void {
    setError(null);
    setMessage(null);
    try {
      const result = simulateMacro(buildMacro());
      setTrace(result.steps.map((step) => `${step.index + 1}. ${step.label}`));
      setMessage(`Simulated dry run (${result.durationMs} ms total). No input was injected.`);
    } catch (cause) {
      setError(errorMessage(cause));
    }
  }

  async function save(): Promise<void> {
    setError(null);
    setMessage(null);
    const macro = buildMacro();
    const issues = validateMacro(macro);
    if (issues.length > 0) {
      setError(issues.join(" "));
      return;
    }
    setBusy(true);
    try {
      await controller.saveMacro(macro);
      setMessage(`Saved macro “${macro.name}”. Stored macros are never auto-run.`);
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
      await controller.deleteMacro(id);
      setMessage(`Deleted macro “${id}”.`);
    } catch (cause) {
      setError(errorMessage(cause));
    }
  }

  async function loadFromList(id: string): Promise<void> {
    setError(null);
    try {
      edit(await controller.getMacro(id));
    } catch (cause) {
      setError(errorMessage(cause));
    }
  }

  function exportJson(): void {
    setError(null);
    try {
      setIo(toMacroFile(buildMacro()));
      setMessage("Serialized the macro to versioned JSON.");
    } catch (cause) {
      setError(errorMessage(cause));
    }
  }

  async function importFromIo(): Promise<void> {
    setError(null);
    setMessage(null);
    try {
      const macro = fromMacroFile(io);
      edit(macro);
      await controller.saveMacro(macro);
      setMessage(`Imported macro “${macro.name}”. It was not executed.`);
    } catch (cause) {
      setError(errorMessage(cause));
    }
  }

  return (
    <section>
      <div className="section-head">
        <h2>Macros</h2>
        <button type="button" onClick={() => void controller.refreshMacros()}>
          Refresh
        </button>
      </div>

      {state.macrosError ? <ErrorNotice message={state.macrosError} /> : null}
      {error ? <ErrorNotice message={error} /> : null}
      {message ? (
        <p className="notice" role="status">
          {message}
        </p>
      ) : null}

      <p className="notice-hint">
        Macros are stored, edited and simulated only. Execution is not implemented; this screen
        never injects keystrokes or hooks global input.
      </p>

      {state.macros.length === 0 ? (
        <EmptyNotice title="No macros yet" hint="Create one with the form below." />
      ) : (
        <ul className="macro-list">
          {state.macros.map((macro) => (
            <li key={macro.id} className="macro-row">
              <span className="macro-name">{macro.name}</span>
              <span className="macro-id">{macro.id}</span>
              <span className="macro-actions-count">{macro.action_count} actions</span>
              {macro.profile ? <span className="macro-profile">{macro.profile}</span> : null}
              <span className="macro-actions">
                <button type="button" onClick={() => void loadFromList(macro.id)}>
                  Edit
                </button>
                <button type="button" onClick={() => void remove(macro.id)}>
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
          <input
            value={draft.id}
            onChange={(event) => setDraft({ ...draft, id: event.target.value })}
          />
        </label>
        <label className="field">
          <span>Name</span>
          <input
            value={draft.name}
            onChange={(event) => setDraft({ ...draft, name: event.target.value })}
          />
        </label>
        <label className="field">
          <span>Associated profile (optional)</span>
          <select
            value={draft.profile ?? ""}
            onChange={(event) =>
              setDraft({ ...draft, profile: event.target.value.length > 0 ? event.target.value : null })
            }
          >
            <option value="">No profile</option>
            {state.profiles.map((profile) => (
              <option key={profile.id} value={profile.id}>
                {profile.name}
              </option>
            ))}
          </select>
        </label>
        <label className="field">
          <span>Repeat</span>
          <input
            type="number"
            min={1}
            max={1000}
            value={draft.repeat.times}
            onChange={(event) =>
              setDraft({
                ...draft,
                repeat: { ...draft.repeat, times: Number.parseInt(event.target.value, 10) || 0 },
              })
            }
          />
        </label>
        <label className="field">
          <span>Pause between repeats (ms)</span>
          <input
            type="number"
            min={0}
            max={60000}
            value={draft.repeat.interval_ms}
            onChange={(event) =>
              setDraft({
                ...draft,
                repeat: {
                  ...draft.repeat,
                  interval_ms: Number.parseInt(event.target.value, 10) || 0,
                },
              })
            }
          />
        </label>

        <h3>Actions</h3>
        <ul className="macro-action-list">
          {draft.actions.map((action, index) => (
            <li key={index} className="macro-action">
              <select
                value={action.kind}
                onChange={(event) =>
                  setAction(
                    index,
                    event.target.value === "delay"
                      ? { kind: "delay", ms: 50 }
                      : { kind: "key", key: "f13", press: "tap" },
                  )
                }
              >
                <option value="key">key</option>
                <option value="delay">delay</option>
              </select>

              {action.kind === "key" ? (
                <>
                  <input
                    aria-label="Key"
                    value={action.key}
                    onChange={(event) => setAction(index, { ...action, key: event.target.value })}
                  />
                  <select
                    aria-label="Press"
                    value={action.press}
                    onChange={(event) =>
                      setAction(index, { ...action, press: event.target.value as KeyPress })
                    }
                  >
                    <option value="tap">tap</option>
                    <option value="down">down</option>
                    <option value="up">up</option>
                  </select>
                </>
              ) : (
                <input
                  aria-label="Delay milliseconds"
                  type="number"
                  min={1}
                  max={60000}
                  value={action.ms}
                  onChange={(event) =>
                    setAction(index, {
                      kind: "delay",
                      ms: Number.parseInt(event.target.value, 10) || 0,
                    })
                  }
                />
              )}

              <button type="button" onClick={() => removeAction(index)}>
                Remove
              </button>
            </li>
          ))}
        </ul>

        <div className="form-actions">
          <button type="submit" disabled={busy}>
            Save macro
          </button>
          <button type="button" onClick={addAction}>
            Add action
          </button>
          <button type="button" onClick={() => setDraft(emptyMacro())}>
            New
          </button>
          <button type="button" disabled={problems.length > 0} onClick={simulate}>
            Simulate
          </button>
          <button type="button" onClick={exportJson}>
            Export JSON
          </button>
        </div>
      </form>

      {problems.length > 0 ? (
        <div className="notice notice-error" role="alert">
          <p className="notice-title">Fix the following before saving</p>
          <ul>
            {problems.map((problem) => (
              <li key={problem}>{problem}</li>
            ))}
          </ul>
        </div>
      ) : null}

      {trace.length > 0 ? (
        <div className="trace">
          <h3>Simulation trace</h3>
          <ol>
            {trace.map((line) => (
              <li key={line}>{line}</li>
            ))}
          </ol>
        </div>
      ) : null}

      <h3>Import / export (versioned JSON)</h3>
      <textarea
        className="io"
        aria-label="Versioned macro JSON"
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
