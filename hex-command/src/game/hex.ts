import type { Hex } from "../../rust/pkg/hex_command";
export type { Hex } from "../../rust/pkg/hex_command";

/** Presentation lookup keys; never authored landmark IDs or native command targets. */
export function cellKey(hex: Readonly<Hex> | string): string;
export function cellKey(hex: Readonly<Hex> | string | null | undefined): string | undefined;
export function cellKey(hex: Readonly<Hex> | string | null | undefined): string | undefined {
  if (hex == null) return undefined;
  return typeof hex === "string" ? hex : `${hex.col},${hex.row}`;
}
export function sameHex(first: Readonly<Hex>, second: Readonly<Hex>): boolean {
  return first.col === second.col && first.row === second.row;
}
