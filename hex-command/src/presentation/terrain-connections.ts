import castleBells from "../../pixel/art/terrain/castle-bell/variants.json";
import castleWalls from "../../pixel/art/terrain/castle-wall/variants.json";
import castleGates from "../../pixel/art/terrain/castle-gate/variants.json";
import roadDeadEnds from "../../pixel/art/terrain/road/dead-ends.json";
import type { HexDirection, TerrainType } from "../game/types";

export const TERRAIN_DIRECTIONS: readonly HexDirection[] = ["e", "se", "sw", "w", "nw", "ne"];
type ConnectedTile = {
  terrain: TerrainType;
  connections?: readonly HexDirection[];
  deadEnds?: readonly HexDirection[];
  wallCorners?: readonly number[];
};
export const CONNECTED_TERRAINS: readonly TerrainType[] = [
  "signpost",
  "road",
  "village",
  "bridge",
  "ford",
  "landing",
  "ferry",
  "castle-wall",
  "castle-gate",
  "castle-bell",
];
const directionMask = (directions: readonly HexDirection[]) =>
  directions.reduce((mask, direction) => mask | (1 << TERRAIN_DIRECTIONS.indexOf(direction)), 0);

/** A pure key encoding of explicit authored geometry; no neighbor inspection. */
export function castleVariantId(mask: number, corners: readonly number[]): string {
  return corners
    .reduce((id, code, i) => id | (BigInt(code) << BigInt(6 + 9 * i)), BigInt(mask))
    .toString();
}
const castleVariants = {
  "castle-wall": castleWalls,
  "castle-gate": castleGates,
  "castle-bell": castleBells,
};

/** Finished capped roads are explicitly registered alongside their source matrices. */
export function validTerrainConnections(tile: ConnectedTile): boolean {
  const castle =
    tile.terrain === "castle-wall" ||
    tile.terrain === "castle-gate" ||
    tile.terrain === "castle-bell";
  if (tile.wallCorners !== undefined && !castle) return false;
  if (
    castle &&
    (!tile.wallCorners ||
      tile.wallCorners.length !== 6 ||
      !tile.wallCorners.every((code) => Number.isInteger(code) && code >= 0 && code < 512))
  )
    return false;
  if (tile.connections === undefined)
    return !CONNECTED_TERRAINS.includes(tile.terrain) && tile.deadEnds === undefined;
  if (!tile.connections.every((direction) => TERRAIN_DIRECTIONS.includes(direction))) return false;
  const mask = directionMask(tile.connections);
  if (
    (!mask &&
      tile.terrain !== "castle-wall" &&
      tile.terrain !== "castle-bell" &&
      tile.terrain !== "village") ||
    new Set(tile.connections).size !== tile.connections.length
  )
    return false;
  if (tile.deadEnds !== undefined) {
    if (
      !tile.deadEnds.every((direction) => TERRAIN_DIRECTIONS.includes(direction)) ||
      new Set(tile.deadEnds).size !== tile.deadEnds.length
    )
      return false;
    const capped = directionMask(tile.deadEnds);
    return (
      tile.terrain === "road" &&
      roadDeadEnds.some(([exits, ends]) => exits === mask && ends === capped)
    );
  }
  switch (tile.terrain) {
    case "castle-wall":
    case "castle-gate":
    case "castle-bell":
      return castleVariants[tile.terrain].some(
        (variant) =>
          variant.mask === mask &&
          variant.wallCorners.every((code, i) => code === tile.wallCorners![i]),
      );
    case "road":
      return true;
    case "signpost":
      return mask === 9;
    case "village":
      return tile.connections.length <= 3;
    case "bridge":
    case "ford":
      return [9, 18, 36].includes(mask);
    case "landing":
    case "ferry":
      return [1, 8].includes(mask);
    default:
      return false;
  }
}

/** Connections select native frames; they do not change movement or protection. */
export function terrainArtNames(tile: ConnectedTile): string[] {
  if (!validTerrainConnections(tile))
    throw new Error(`Invalid or missing terrain connections: ${tile.terrain}`);
  if (!CONNECTED_TERRAINS.includes(tile.terrain)) return [tile.terrain];
  const mask = directionMask(tile.connections!);
  if (
    tile.terrain === "castle-wall" ||
    tile.terrain === "castle-gate" ||
    tile.terrain === "castle-bell"
  )
    return [`${tile.terrain}_${castleVariantId(mask, tile.wallCorners!)}`];
  if (tile.terrain === "road" && tile.deadEnds?.length)
    return [`road_${mask}-end-${directionMask(tile.deadEnds)}`];
  return [`${tile.terrain}_${mask}`];
}
