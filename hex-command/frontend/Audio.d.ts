/** Opaque per-session audio owner implemented in OCaml. */
export interface Audio { readonly _audio: unique symbol }
export function create(): Audio;
export function enabled(audio: Audio): boolean;
export function set_enabled(audio: Audio, enabled: boolean): void;
export function cancel(audio: Audio): void;
export function unlock(audio: Audio, trusted: boolean): Promise<boolean>;
export function play(audio: Audio, family: string): Promise<boolean>;
export function interaction_family(attack: string | undefined, unitType: string | undefined, terrain: string | undefined, unit?: undefined): string;
export function play_interaction(audio: Audio, attack: string | undefined, unitType: string | undefined, terrain: string | undefined, unit?: undefined): Promise<boolean>;
