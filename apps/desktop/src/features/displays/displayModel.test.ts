import { describe, expect, it } from "vitest";

import type { DisplayCapabilities } from "../../types/ipc";
import {
  bytesPerPixel,
  defaultSelection,
  frameBytes,
  previewFrame,
  validateFrame,
} from "./displayModel";

function display(overrides: Partial<DisplayCapabilities> = {}): DisplayCapabilities {
  return {
    width: 64,
    height: 32,
    formats: ["rgb888", "rgb565"],
    max_frames_per_second: 30,
    max_frame_bytes: 64 * 32 * 3,
    ...overrides,
  };
}

describe("displayModel", () => {
  it("uses three and two bytes per declared pixel format", () => {
    expect(bytesPerPixel("rgb888")).toBe(3);
    expect(bytesPerPixel("rgb565")).toBe(2);
  });

  it("computes uncompressed frame bytes from the declared dimensions", () => {
    expect(frameBytes(display(), "rgb888")).toBe(6144);
    expect(frameBytes(display(), "rgb565")).toBe(4096);
  });

  it("accepts a declared format that fits the byte limit", () => {
    expect(validateFrame(display(), "rgb888")).toEqual([]);
    expect(validateFrame(display(), "rgb565")).toEqual([]);
  });

  it("rejects a format the display does not declare", () => {
    const problems = validateFrame(display({ formats: ["rgb565"] }), "rgb888");
    expect(problems).toHaveLength(1);
    expect(problems[0]).toContain("not declared");
  });

  it("rejects a frame above the declared byte limit", () => {
    const problems = validateFrame(display({ max_frame_bytes: 1024 }), "rgb888");
    expect(problems).toHaveLength(1);
    expect(problems[0]).toContain("above the declared 1024");
  });

  it("builds a local preview without encoding or transmitting a frame", () => {
    const selection = defaultSelection(display());
    const preview = previewFrame(display(), selection);
    expect(preview.format).toBe("rgb888");
    expect(preview.bytes).toBe(6144);
    expect(preview.withinLimit).toBe(true);
    expect(preview.background).toBe("rgb(64, 128, 255)");
    expect(preview.label).toContain("Local");
    expect(preview.label).toContain("not transmitted");
  });

  it("tints checker and gradient previews with the selected colour", () => {
    const selection = {
      ...defaultSelection(display()),
      pattern: "checker" as const,
      color: { red: 1, green: 2, blue: 3 },
    };
    expect(previewFrame(display(), selection).background).toContain("rgb(1, 2, 3)");
    expect(
      previewFrame(display(), { ...selection, pattern: "gradient" }).background,
    ).toContain("linear-gradient");
  });

  it("flags a frame above a tighter byte limit", () => {
    const preview = previewFrame(display({ max_frame_bytes: 100 }), defaultSelection(display()));
    expect(preview.withinLimit).toBe(false);
  });
});
