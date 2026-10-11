/** Pure hex geometry implemented in OCaml. */
export interface Point { readonly x: number; readonly y: number }
export interface Dimensions { readonly width: number; readonly height: number }
export function center(col: number, row: number): Point;
export function hex_points(x: number, y: number): string;
export function dimensions(cols: number, rows: number): Dimensions;
export function color(terrain: string): string;
export function preview_color(terrain: string): string;
