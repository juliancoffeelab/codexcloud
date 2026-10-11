/** Pure hex geometry implemented in OCaml. */
export interface Point { readonly x: number; readonly y: number }
export interface Dimensions { readonly width: number; readonly height: number }
export const sprite_size: number;
export const native_pixel_scale: number;
export const column_spacing: number;
export const row_spacing: number;
export function center(col: number, row: number): Point;
export function hex_points(x: number, y: number): string;
export function dimensions(cols: number, rows: number): Dimensions;
export function preview_dimensions(cols: number, rows: number): Dimensions;
export function preview_color(terrain: string): string;
