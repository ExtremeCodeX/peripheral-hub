// MVP macro model: editing, validation, versioned persistence and *safe*
// simulation only. There is deliberately no global input injection or hooking
// here; the simulation produces a descriptive trace, never keystrokes.

export const MACRO_SCHEMA_VERSION = 1;

const TOKEN = /^[A-Za-z0-9_.:-]{1,128}$/;

export type MacroAction =
  | { kind: "key"; key: string; press: "down" | "up" | "tap" }
  | { kind: "delay"; ms: number };

export interface Macro {
  id: string;
  name: string;
  actions: MacroAction[];
}

export interface MacroFile {
  macro_schema_version: number;
  macro: Macro;
}

export interface MacroLimits {
  maxActions: number;
  maxDelayMs: number;
}

export const DEFAULT_MACRO_LIMITS: MacroLimits = { maxActions: 4096, maxDelayMs: 60000 };

/** Return a list of human-readable problems; empty means valid. */
export function validateMacro(macro: Macro, limits: MacroLimits = DEFAULT_MACRO_LIMITS): string[] {
  const problems: string[] = [];
  if (!TOKEN.test(macro.id)) {
    problems.push("Macro id must be a 1..128 character token.");
  }
  if (macro.name.length === 0 || macro.name.length > 64) {
    problems.push("Name must be 1..64 characters.");
  }
  if ([...macro.name].some((char) => char.charCodeAt(0) < 32 || char.charCodeAt(0) === 127)) {
    problems.push("Name must not contain control characters.");
  }
  if (macro.actions.length === 0) {
    problems.push("A macro needs at least one action.");
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
 * and does not claim that a remapping was applied to hardware.
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
  let durationMs = 0;
  macro.actions.forEach((action, index) => {
    if (action.kind === "delay") {
      durationMs += action.ms;
      steps.push({ index, label: `Wait ${action.ms} ms` });
    } else {
      steps.push({ index, label: `${action.press} ${action.key}` });
    }
  });
  return { simulated: true, steps, durationMs };
}

export function emptyMacro(): Macro {
  return { id: "new-macro", name: "New macro", actions: [{ kind: "key", key: "f13", press: "tap" }] };
}
