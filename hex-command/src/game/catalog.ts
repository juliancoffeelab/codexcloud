/** Read-only browser metadata projected from the canonical game documents. */
import { immutable } from "./immutable";
import { AUTHORED_UNITS, AUTHORED_TERRAIN } from "./authored-content";
import type {
  UnitType,
  UnitDefinition,
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
export const TERRAIN = immutable<Record<TerrainType, TerrainDefinition>>(AUTHORED_TERRAIN);
export function axial(tile: Coordinates) {
  return { q: tile.col - Math.floor(tile.row / 2), r: tile.row };
}
export function distance(first: Coordinates, second: Coordinates) {
  const q = first.col - Math.floor(first.row / 2) - second.col + Math.floor(second.row / 2);
  const r = first.row - second.row;
  return (Math.abs(q) + Math.abs(r) + Math.abs(q + r)) / 2;
}
