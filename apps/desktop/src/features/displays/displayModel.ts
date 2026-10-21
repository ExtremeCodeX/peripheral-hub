// Display contract/preview helpers. This module is deliberately presentation-only:
// it validates a prospective frame against the declared contract and builds a
// local CSS preview. There is no encoder, transport call or frame transmission
// here, and the SDK has no display command to send one.

import type { DisplayCapabilities, PixelFormat, RgbColor } from "../../types/ipc";

export const DISPLAY_PATTERNS = ["solid", "checker", "gradient"] as const;

export type DisplayPattern = (typeof DISPLAY_PATTERNS)[number];

export interface FrameSelection {
  format: PixelFormat;
  pattern: DisplayPattern;
  color: RgbColor;
}

export interface FramePreview {
  format: PixelFormat;
  bytes: number;
  withinLimit: boolean;
  /** CSS background for a local-only preview; never transmitted. */
  background: string;
  label: string;
}

/** Uncompressed bytes per pixel for the SDK's generic formats. */
export function bytesPerPixel(format: PixelFormat): number {
  return format === "rgb888" ? 3 : 2;
}

/** Uncompressed frame size in bytes for the declared dimensions. */
export function frameBytes(display: DisplayCapabilities, format: PixelFormat): number {
  return display.width * display.height * bytesPerPixel(format);
}

/** Return human-readable problems; empty means the frame fits the contract. */
export function validateFrame(display: DisplayCapabilities, format: PixelFormat): string[] {
  const problems: string[] = [];
  if (!display.formats.includes(format)) {
    problems.push(`Format ${format} is not declared by this display.`);
    return problems;
  }
  const bytes = frameBytes(display, format);
  if (bytes > display.max_frame_bytes) {
    problems.push(
      `A ${display.width}×${display.height} ${format} frame needs ${bytes} bytes, above the declared ${display.max_frame_bytes}.`,
    );
  }
  return problems;
}

function cssColor(color: RgbColor): string {
  return `rgb(${color.red}, ${color.green}, ${color.blue})`;
}

function backgroundFor(selection: FrameSelection): string {
  const color = cssColor(selection.color);
  switch (selection.pattern) {
    case "checker":
      return `repeating-conic-gradient(${color} 0% 25%, #101014 0% 50%) 50% / 16px 16px`;
    case "gradient":
      return `linear-gradient(90deg, ${color}, #101014)`;
    case "solid":
      return color;
  }
}

/**
 * Build a descriptive preview of one frame. The returned `background` is only
 * applied to a local DOM element; nothing is serialized or sent to a device.
 */
export function previewFrame(
  display: DisplayCapabilities,
  selection: FrameSelection,
): FramePreview {
  const bytes = frameBytes(display, selection.format);
  const problems = validateFrame(display, selection.format);
  return {
    format: selection.format,
    bytes,
    withinLimit: problems.length === 0,
    background: backgroundFor(selection),
    label: `Local ${selection.pattern} preview at ${display.width}×${display.height}, ${selection.format}, not transmitted`,
  };
}

/** Default selection: the first declared format, a solid colour and the whole area. */
export function defaultSelection(display: DisplayCapabilities): FrameSelection {
  return {
    format: display.formats[0] ?? "rgb888",
    pattern: "solid",
    color: { red: 64, green: 128, blue: 255 },
  };
}
