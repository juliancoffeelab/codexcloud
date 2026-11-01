/** Read-only browser metadata projected from the canonical game documents. */
import { immutable } from "./immutable";
import { AUTHORED_UNITS, AUTHORED_TERRAIN } from "./authored-content";
import type {
  UnitType,
  Unit,
  UnitDefinition,
  Weapon,
  TerrainType,
  TerrainDefinition,
  Coordinates,
} from "./types";
export { RACES } from "./races";
export type * from "./types";
export const DAMAGE_TYPES = Object.freeze({
  blade: "Blade",
  pierce: "Pierce",
  impact: "Impact",
});
export const TYPES = immutable<Record<UnitType, UnitDefinition>>(AUTHORED_UNITS);
/** Rust-owned equipment contributes attack options to the individual unit. */
export function unitWeapons(unit: {
  type: UnitType;
  items?: readonly { attacks: readonly Weapon[] }[];
}): readonly Weapon[] {
  return [...TYPES[unit.type].attacks, ...(unit.items?.flatMap((item) => item.attacks) ?? [])];
}
/** Display leadership follows the instance role supplied by Rust. */
export function isLeader(unit: Pick<Unit, "role">): boolean {
  return unit.role === "leader";
}
export const TERRAIN = immutable<Record<TerrainType, TerrainDefinition>>(AUTHORED_TERRAIN);
export function distance(first: Coordinates, second: Coordinates) {
  const q = first.col - Math.floor(first.row / 2) - second.col + Math.floor(second.row / 2);
  const r = first.row - second.row;
  return (Math.abs(q) + Math.abs(r) + Math.abs(q + r)) / 2;
}
