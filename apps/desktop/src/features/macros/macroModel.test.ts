import { describe, expect, it } from "vitest";

import {
  emptyMacro,
  fromMacroFile,
  simulateMacro,
  toMacroFile,
  validateMacro,
} from "./macroModel";

describe("macroModel", () => {
  it("accepts the empty draft", () => {
    expect(validateMacro(emptyMacro())).toEqual([]);
  });

  it("flags a macro with no actions", () => {
    expect(validateMacro({ ...emptyMacro(), actions: [] })).toContain(
      "A macro needs at least one action.",
    );
  });

  it("rejects an out-of-range delay", () => {
    const problems = validateMacro({
      ...emptyMacro(),
      actions: [{ kind: "delay", ms: 0 }],
    });
    expect(problems.some((problem) => problem.includes("delay must be"))).toBe(true);
  });

  it("rejects an out-of-range repeat count and pause", () => {
    const problems = validateMacro({
      ...emptyMacro(),
      repeat: { times: 1001, interval_ms: 60001 },
    });
    expect(problems.some((problem) => problem.includes("Repeat count must be"))).toBe(true);
    expect(problems.some((problem) => problem.includes("Repeat pause must be"))).toBe(true);
  });

  it("rejects a macro whose repeated duration exceeds the limit", () => {
    const problems = validateMacro({
      ...emptyMacro(),
      actions: [{ kind: "delay", ms: 60000 }],
      repeat: { times: 100, interval_ms: 0 },
    });
    expect(problems.some((problem) => problem.includes("Total macro duration"))).toBe(true);
  });

  it("simulates a trace without injecting input", () => {
    const result = simulateMacro({
      ...emptyMacro(),
      actions: [
        { kind: "key", key: "f13", press: "tap" },
        { kind: "delay", ms: 25 },
      ],
    });
    expect(result.simulated).toBe(true);
    expect(result.durationMs).toBe(25);
    expect(result.steps.map((step) => step.label)).toEqual(["tap f13", "Wait 25 ms"]);
  });

  it("accounts for finite repetition in the simulated duration", () => {
    const result = simulateMacro({
      ...emptyMacro(),
      actions: [{ kind: "delay", ms: 10 }],
      repeat: { times: 3, interval_ms: 5 },
    });
    // 3 * 10 ms of delays plus 2 * 5 ms of pauses between iterations.
    expect(result.durationMs).toBe(40);
  });

  it("round-trips through the versioned macro file", () => {
    const macro = emptyMacro();
    expect(fromMacroFile(toMacroFile(macro))).toEqual(macro);
  });

  it("rejects an unsupported macro schema version", () => {
    expect(() =>
      fromMacroFile(JSON.stringify({ macro_schema_version: 2, macro: emptyMacro() })),
    ).toThrow(/schema version/);
  });
});
