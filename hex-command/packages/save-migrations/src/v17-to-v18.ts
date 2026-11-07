import type { Save } from "./formats/history";
import { readSave } from "./formats/read";
import { isRecord } from "./formats/envelope-v39";

// Frozen version-seventeen geometry. No current catalog or gameplay dependencies.
const directions = ["e", "se", "sw", "w", "nw", "ne"];
const castles = new Set(["castle-wall", "castle-gate", "castle-bell"]);
const validEdges = (value: unknown): value is string[] =>
  Array.isArray(value) &&
  value.every((edge) => typeof edge === "string" && directions.includes(edge)) &&
  new Set(value).size === value.length;

/** Retire gate terrain and apertures while preserving every battle outcome. */
export function migrateV17ToV18(input: unknown): Save<18> {
  const value = readSave(input, 17);
  if (value.version !== 17 || !isRecord(value.checkpoint) || !isRecord(value.checkpoint.state))
    throw new Error("Invalid version-seventeen save");
  const state = value.checkpoint.state;
  if (!Array.isArray(state.tiles) || !state.tiles.every(isRecord))
    throw new Error("Invalid historical tile grid");
  const tiles = state.tiles.map((tile) => {
    if (typeof tile.terrain !== "string" || !castles.has(tile.terrain)) return tile;
    const walls = tile.walls,
      gates = tile.gates;
    if (!validEdges(walls) || !validEdges(gates) || walls.some((edge) => gates.includes(edge)))
      throw new Error("Invalid historical castle geometry");
    if (gates.length !== (tile.terrain === "castle-gate" ? 1 : 0))
      throw new Error("Invalid historical gate count");
    const { gates: _retired, ...rest } = tile;
    return { ...rest, terrain: tile.terrain === "castle-gate" ? "castle-wall" : tile.terrain };
  });
  return { ...value, version: 18, checkpoint: { ...value.checkpoint, state: { ...state, tiles } } };
}
