import registry from '../../../../catalog/installable.json';
export type PublicRelease = typeof registry[number]['package'] & { installed: boolean };
// Same reviewed version/hash registry embedded in the native installer/loader.
export const publicReleases: PublicRelease[] = registry.map((entry) => ({ ...entry.package, installed: false }));
