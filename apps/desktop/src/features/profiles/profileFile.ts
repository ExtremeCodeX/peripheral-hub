// Versioned profile JSON mirroring `crates/profiles` `ProfileFile` and its
// validation. The wire `save_profile` transfers the bare `Profile`; this module
// provides the versioned import/export envelope used by the UI.

import type { Profile } from "../../types/ipc";

export const PROFILE_SCHEMA_VERSION = 1;

const TOKEN = /^[A-Za-z0-9_.:-]{1,128}$/;

export interface ProfileFile {
  profile_schema_version: number;
  profile: Profile;
}

export function validateProfile(profile: Profile): string[] {
  const problems: string[] = [];
  if (!TOKEN.test(profile.id)) {
    problems.push("Profile id must be a 1..128 character token.");
  }
  if (profile.name.length === 0 || profile.name.length > 64) {
    problems.push("Name must be 1..64 characters.");
  }
  if ([...profile.name].some((char) => char.charCodeAt(0) < 32 || char.charCodeAt(0) === 127)) {
    problems.push("Name must not contain control characters.");
  }
  if (profile.lighting) {
    if (!TOKEN.test(profile.lighting.zone)) {
      problems.push("Lighting zone must be a valid token.");
    }
    const brightness = profile.lighting.brightness;
    if (brightness !== null && (brightness < 0 || brightness > 100)) {
      problems.push("Brightness must be 0..100.");
    }
  }
  return problems;
}

export function serializeProfile(profile: Profile): string {
  const problems = validateProfile(profile);
  if (problems.length > 0) {
    throw new Error(problems.join(" "));
  }
  const file: ProfileFile = { profile_schema_version: PROFILE_SCHEMA_VERSION, profile };
  return JSON.stringify(file, null, 2);
}

export function parseProfile(text: string): Profile {
  let parsed: unknown;
  try {
    parsed = JSON.parse(text);
  } catch {
    throw new Error("The profile file is not valid JSON.");
  }
  if (typeof parsed !== "object" || parsed === null) {
    throw new Error("The profile file must be a JSON object.");
  }
  const record = parsed as Partial<ProfileFile>;
  if (record.profile_schema_version !== PROFILE_SCHEMA_VERSION) {
    throw new Error("Unsupported profile schema version.");
  }
  const profile = record.profile;
  if (typeof profile !== "object" || profile === null) {
    throw new Error("The profile payload is missing.");
  }
  const problems = validateProfile(profile);
  if (problems.length > 0) {
    throw new Error(problems.join(" "));
  }
  return profile;
}

export function emptyProfile(): Profile {
  return { id: "new-profile", name: "New profile", device_model: null, lighting: null };
}
