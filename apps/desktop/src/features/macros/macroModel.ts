// Versioned macro model mirroring `crates/macros`: validation, versioned
// import/export and *safe* dry-run simulation. There is deliberately no global
// input injection or hooking here; simulation produces a descriptive trace,
// never keystrokes. The wire `save_macro` transfers the bare `Macro`; this
// module provides the versioned envelope used by the UI and the local checks.

import type { KeyPress, Macro, MacroAction, MacroRepeat } from "../../types/ipc";

export type { KeyPress, Macro, MacroAction, MacroRepeat };

export const MACRO_SCHEMA_VERSION = 1;

const TOKEN = /^[A-Za-z0-9_.:-]{1,128}$/;

export interface MacroFile {
  macro_schema_version: number;
  macro: Macro;
}

export interface MacroLimits {
  maxActions: number;
  maxDelayMs: number;
  maxRepeat: number;
  maxIntervalMs: number;
  maxIterationMs: number;
  maxTotalMs: number;
}

export const DEFAULT_MACRO_LIMITS: MacroLimits = {
  maxActions: 4096,
  maxDelayMs: 60000,
  maxRepeat: 1000,
  maxIntervalMs: 60000,
  maxIterationMs: 600000,
  maxTotalMs: 3600000,
};

function iterationMs(macro: Macro): number {
  return macro.actions.reduce(
    (total, action) => total + (action.kind === "delay" ? action.ms : 0),
    0,
  );
}

/** Return a list of human-readable problems; empty means valid. */
export function validateMacro(macro: Macro, limits: MacroLimits = DEFAULT_MACRO_LIMITS): string[] {
  const problems: string[] = [];
  if (!TOKEN.test(macro.id)) {
    problems.push("Macro id must be a 1..128 character token.");
  }
  if (typeof macro.name !== "string" || macro.name.length === 0 || macro.name.length > 64) {
    problems.push("Name must be 1..64 characters.");
  } else if (
    [...macro.name].some((char) => char.charCodeAt(0) < 32 || char.charCodeAt(0) === 127)
  ) {
    problems.push("Name must not contain control characters.");
  }
  if (macro.profile !== null && !TOKEN.test(macro.profile)) {
    problems.push("Profile link must be a valid token or empty.");
  }
  if (!Array.isArray(macro.actions) || macro.actions.length === 0) {
    problems.push("A macro needs at least one action.");
    return problems;
  }
  if (macro.actions.length > limits.maxActions) {
    problems.push(`At most ${limits.maxActions} actions are allowed.`);
  }
  macro.actions.forEach((action, index) => {
    if (action.kind === "delay") {
      if (!Number.isInteger(action.ms) || action.ms < 1 || action.ms > limits.maxDelayMs) {
        problems.push(`Action ${index + 1}: delay must be 1..${limits.maxDelayMs} ms.`);
      }
    } else if (!TOKEN.test(action.key)) {
      problems.push(`Action ${index + 1}: key name must be a valid token.`);
    }
  });

  const repeat = macro.repeat ?? { times: 1, interval_ms: 0 };
  if (!Number.isInteger(repeat.times) || repeat.times < 1 || repeat.times > limits.maxRepeat) {
    problems.push(`Repeat count must be 1..${limits.maxRepeat}.`);
  }
  if (
    !Number.isInteger(repeat.interval_ms) ||
    repeat.interval_ms < 0 ||
    repeat.interval_ms > limits.maxIntervalMs
  ) {
    problems.push(`Repeat pause must be 0..${limits.maxIntervalMs} ms.`);
  }

  const perIteration = iterationMs(macro);
  if (perIteration > limits.maxIterationMs) {
    problems.push(`A single iteration may wait at most ${limits.maxIterationMs} ms.`);
  }
  const total =
    perIteration * repeat.times + repeat.interval_ms * Math.max(0, repeat.times - 1);
  if (total > limits.maxTotalMs) {
    problems.push(`Total macro duration may be at most ${limits.maxTotalMs} ms.`);
  }
  return problems;
}

export function toMacroFile(macro: Macro, limits: MacroLimits = DEFAULT_MACRO_LIMITS): string {
  const problems = validateMacro(macro, limits);
  if (problems.length > 0) {
    throw new Error(problems.join(" "));
  }
  const file: MacroFile = { macro_schema_version: MACRO_SCHEMA_VERSION, macro };
  return JSON.stringify(file, null, 2);
}

export function fromMacroFile(text: string, limits: MacroLimits = DEFAULT_MACRO_LIMITS): Macro {
  let parsed: unknown;
  try {
    parsed = JSON.parse(text);
  } catch {
    throw new Error("The macro file is not valid JSON.");
  }
  if (typeof parsed !== "object" || parsed === null) {
    throw new Error("The macro file must be a JSON object.");
  }
  const record = parsed as Partial<MacroFile>;
  if (record.macro_schema_version !== MACRO_SCHEMA_VERSION) {
    throw new Error("Unsupported macro schema version.");
  }
  const macro = record.macro;
  if (typeof macro !== "object" || macro === null) {
    throw new Error("The macro payload is missing.");
  }
  const problems = validateMacro(macro, limits);
  if (problems.length > 0) {
    throw new Error(problems.join(" "));
  }
  return macro;
}

export interface SimulationStep {
  index: number;
  label: string;
}

export interface MacroSimulation {
  simulated: true;
  steps: SimulationStep[];
  durationMs: number;
}

/**
 * Dry-run the macro into a descriptive trace. This performs no input injection
 * and does not claim that a remapping was applied to hardware. The trace covers
 * one iteration; the reported duration accounts for finite repetition.
 */
export function simulateMacro(
  macro: Macro,
  limits: MacroLimits = DEFAULT_MACRO_LIMITS,
): MacroSimulation {
  const problems = validateMacro(macro, limits);
  if (problems.length > 0) {
    throw new Error(problems.join(" "));
  }
  const steps: SimulationStep[] = [];
  let perIteration = 0;
  macro.actions.forEach((action, index) => {
    if (action.kind === "delay") {
      perIteration += action.ms;
      steps.push({ index, label: `Wait ${action.ms} ms` });
    } else {
      steps.push({ index, label: `${action.press} ${action.key}` });
    }
  });
  const repeat = macro.repeat ?? { times: 1, interval_ms: 0 };
  if (repeat.times > 1) {
    steps.push({
      index: macro.actions.length,
      label: `Repeat ${repeat.times}\u00d7 with ${repeat.interval_ms} ms pause`,
    });
  }
  const durationMs =
    perIteration * repeat.times + repeat.interval_ms * Math.max(0, repeat.times - 1);
  return { simulated: true, steps, durationMs };
}

export function emptyMacro(): Macro {
  return {
    id: "new-macro",
    name: "New macro",
    actions: [{ kind: "key", key: "f13", press: "tap" }],
    repeat: { times: 1, interval_ms: 0 },
    profile: null,
  };
}
