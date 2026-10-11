/** Public calls into emitted OCaml; contains no runtime implementation. */
export interface Frame { readonly url: string; readonly width: number; readonly height: number }
export interface Artwork { readonly _artwork: unique symbol }
export function current(): Artwork;
export function frames(artwork: Artwork): Readonly<Record<string, Frame>>;
export function find(artwork: Artwork, key: string): Frame | undefined;
export function frame_key(kind: string, name: string, side?: string, floor?: string, unit?: undefined): string;
export function frame(artwork: Artwork, kind: string, name: string, side?: string, floor?: string, unit?: undefined): Frame | undefined;
export function unit_frame(artwork: Artwork, side: string, name: string): Frame | undefined;
export function terrain_frame(artwork: Artwork, floor: string | undefined, name: string): Frame | undefined;
export function portrait_frame(artwork: Artwork, name: string): Frame | undefined;
export function icon_markup(name: string): string;
