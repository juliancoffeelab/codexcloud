/** Public calls into emitted OCaml; contains no runtime implementation. */
export type Style = 0 | 1 | 2;
export interface Frame {
  readonly url: string;
  readonly width: number;
  readonly height: number;
  readonly pixel: boolean;
}
export interface Artwork { readonly _artwork: unique symbol }
export const styles: readonly (readonly [Style, string])[];
export function style_name(style: Style): string;
export function style_of_string(name: string): Style | undefined;
export function saved_style(): Style;
export function save_style(style: Style): void;
export function artwork(style: Style): Artwork;
export function style(artwork: Artwork): Style;
export function load(style: Style): Promise<boolean>;
export function find(artwork: Artwork, key: string): Frame | undefined;
export function unit_frame(artwork: Artwork, side: string, name: string): Frame | undefined;
export function terrain_frame(artwork: Artwork, name: string): Frame | undefined;
export function portrait_frame(artwork: Artwork, name: string): Frame | undefined;
export function color(artwork: Artwork, terrain: string): string;
export function sprite(artwork: Artwork, kind: string, name: string, x: number, y: number, width: number, height: number, className?: string, side?: string, unit?: undefined): string;
export function portrait(artwork: Artwork, name: string, x: number, y: number, width: number, height: number): string;
export function icon_markup(className: string | undefined, name: string): string;
