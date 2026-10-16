import { useState } from "react";

import { ErrorNotice } from "../../components/StateNotice";
import type { Macro, MacroAction } from "./macroModel";
import { emptyMacro, fromMacroFile, simulateMacro, toMacroFile, validateMacro } from "./macroModel";

export function MacrosScreen() {
  const [macro, setMacro] = useState<Macro>(emptyMacro());
  const [io, setIo] = useState("");
  const [trace, setTrace] = useState<string[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [message, setMessage] = useState<string | null>(null);

  const problems = validateMacro(macro);

  function setAction(index: number, action: MacroAction): void {
    setMacro({
      ...macro,
      actions: macro.actions.map((candidate, position) => (position === index ? action : candidate)),
    });
  }

  function addAction(): void {
    setMacro({ ...macro, actions: [...macro.actions, { kind: "delay", ms: 50 }] });
  }

  function removeAction(index: number): void {
    setMacro({ ...macro, actions: macro.actions.filter((_, position) => position !== index) });
  }

  function simulate(): void {
    setError(null);
    setMessage(null);
    try {
      const result = simulateMacro(macro);
      setTrace(result.steps.map((step) => `${step.index + 1}. ${step.label}`));
      setMessage(`Simulated dry run (${result.durationMs} ms total). No input was injected.`);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Simulation failed.");
    }
  }

  function exportJson(): void {
    setError(null);
    try {
      setIo(toMacroFile(macro));
      setMessage("Serialized the macro to versioned JSON.");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Export failed.");
    }
  }

  function importJson(): void {
    setError(null);
    setMessage(null);
    try {
      setMacro(fromMacroFile(io));
      setMessage("Imported a versioned macro.");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Import failed.");
    }
  }

  return (
    <section>
      <div className="section-head">
        <h2>Macros</h2>
        <span className="notice-hint">Editing, validation and safe simulation only.</span>
      </div>

      {error ? <ErrorNotice message={error} /> : null}
      {message ? (
        <p className="notice" role="status">
          {message}
        </p>
      ) : null}

      <p className="notice-hint">
        The MVP does not inject keystrokes or hook global input. Simulation below is a descriptive
        trace and does not claim a remapping was applied to hardware.
      </p>

      <form className="form" onSubmit={(event) => event.preventDefault()}>
        <label className="field">
          <span>Id</span>
          <input value={macro.id} onChange={(event) => setMacro({ ...macro, id: event.target.value })} />
        </label>
        <label className="field">
          <span>Name</span>
          <input value={macro.name} onChange={(event) => setMacro({ ...macro, name: event.target.value })} />
        </label>
      </form>

      <h3>Actions</h3>
      <ul className="macro-actions">
        {macro.actions.map((action, index) => (
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
                    setAction(index, { ...action, press: event.target.value as "down" | "up" | "tap" })
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
                  setAction(index, { kind: "delay", ms: Number.parseInt(event.target.value, 10) || 0 })
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
        <button type="button" onClick={addAction}>
          Add action
        </button>
        <button type="button" disabled={problems.length > 0} onClick={simulate}>
          Simulate
        </button>
        <button type="button" onClick={exportJson}>
          Export JSON
        </button>
      </div>

      {problems.length > 0 ? (
        <div className="notice notice-error" role="alert">
          <p className="notice-title">Fix the following before running</p>
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
        <button type="button" onClick={importJson}>
          Import
        </button>
      </div>
    </section>
  );
}
