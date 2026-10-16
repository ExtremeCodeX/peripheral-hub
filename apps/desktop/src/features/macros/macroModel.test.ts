import { describe, expect, it } from "vitest";

import { emptyMacro, fromMacroFile, simulateMacro, toMacroFile, validateMacro } from "./macroModel";

describe("macroModel", () => {
  it("accepts the empty draft", () => {
    expect(validateMacro(emptyMacro())).toEqual([]);
  });

  it("flags a macro with no actions", () => {
    expect(validateMacro({ id: "m", name: "M", actions: [] })).toContain(
      "A macro needs at least one action.",
    );
  });

  it("rejects an out-of-range delay", () => {
    const problems = validateMacro({
      id: "m",
      name: "M",
      actions: [{ kind: "delay", ms: 0 }],
    });
    expect(problems.some((problem) => problem.includes("delay must be"))).toBe(true);
  });

  it("simulates a trace without injecting input", () => {
    const result = simulateMacro({
      id: "m",
      name: "M",
      actions: [
        { kind: "key", key: "f13", press: "tap" },
        { kind: "delay", ms: 25 },
      ],
    });
    expect(result.simulated).toBe(true);
    expect(result.durationMs).toBe(25);
    expect(result.steps.map((step) => step.label)).toEqual(["tap f13", "Wait 25 ms"]);
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
