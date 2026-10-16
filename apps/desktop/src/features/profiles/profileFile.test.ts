import { describe, expect, it } from "vitest";

import type { Profile } from "../../types/ipc";
import { emptyProfile, parseProfile, serializeProfile, validateProfile } from "./profileFile";

const profile: Profile = {
  id: "gaming",
  name: "Gaming",
  device_model: "simulation:peripheral",
  lighting: {
    zone: "whole-device",
    color: { red: 10, green: 20, blue: 30 },
    brightness: 100,
    effect: "static",
  },
};

describe("profileFile", () => {
  it("round-trips through the versioned envelope", () => {
    const text = serializeProfile(profile);
    expect(JSON.parse(text).profile_schema_version).toBe(1);
    expect(parseProfile(text)).toEqual(profile);
  });

  it("rejects an unsupported schema version", () => {
    expect(() => parseProfile(JSON.stringify({ profile_schema_version: 99, profile }))).toThrow(
      /schema version/,
    );
  });

  it("rejects malformed JSON", () => {
    expect(() => parseProfile("{nope")).toThrow(/valid JSON/);
  });

  it("rejects an out-of-range brightness", () => {
    const invalid: Profile = { ...profile, lighting: { ...profile.lighting!, brightness: 101 } };
    expect(validateProfile(invalid)).toContain("Brightness must be 0..100.");
    expect(() => serializeProfile(invalid)).toThrow(/Brightness/);
  });

  it("exposes an empty draft with no device model", () => {
    expect(emptyProfile().device_model).toBeNull();
  });
});
