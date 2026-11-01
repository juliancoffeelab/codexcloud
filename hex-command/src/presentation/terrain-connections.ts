import castleBells from "../../pixel/art/terrain/castle-bell/variants.json";
import castleWalls from "../../pixel/art/terrain/castle-wall/variants.json";
import roadDeadEnds from "../../pixel/art/terrain/road/dead-ends.json";
import type { HexDirection, TerrainType } from "../game/types";

export const TERRAIN_DIRECTIONS: readonly HexDirection[] = ["e", "se", "sw", "w", "nw", "ne"];
type ConnectedTile = {
  terrain: TerrainType;
  connections?: readonly HexDirection[];
  deadEnds?: readonly HexDirection[];
  walls?: readonly HexDirection[];
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
  "castle-bell",
];
const directionMask = (directions: readonly HexDirection[]) =>
  directions.reduce((mask, direction) => mask | (1 << TERRAIN_DIRECTIONS.indexOf(direction)), 0);

/** Compact source filename for explicit wall directions. */
export function castleVariantId(walls: readonly HexDirection[]): string {
  return String(directionMask(walls));
}
const castleVariants = {
  "castle-wall": castleWalls,
  "castle-bell": castleBells,
};

/** Finished capped roads are explicitly registered alongside their source matrices. */
export function validTerrainConnections(tile: ConnectedTile): boolean {
  if (Object.hasOwn(tile, "wallCorners") || Object.hasOwn(tile, "gates")) return false;
  const castle = tile.terrain === "castle-wall" || tile.terrain === "castle-bell";
  const validEdges = (edges: readonly HexDirection[] | undefined) =>
    edges !== undefined &&
    edges.every((edge) => TERRAIN_DIRECTIONS.includes(edge)) &&
    new Set(edges).size === edges.length;
  if (castle) {
    if (tile.connections !== undefined || tile.deadEnds !== undefined || !validEdges(tile.walls))
      return false;
    const key = castleVariantId(tile.walls!);
    return castleVariants[tile.terrain as keyof typeof castleVariants].some(
      (variant) => variant.id === key,
    );
  }
  if (tile.walls !== undefined) return false;
  if (tile.connections === undefined)
    return !CONNECTED_TERRAINS.includes(tile.terrain) && tile.deadEnds === undefined;
  if (!tile.connections.every((direction) => TERRAIN_DIRECTIONS.includes(direction))) return false;
  const mask = directionMask(tile.connections);
  if (
    (!mask && tile.terrain !== "village") ||
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
  if (tile.terrain === "castle-wall" || tile.terrain === "castle-bell")
    return [`${tile.terrain}_${castleVariantId(tile.walls!)}`];
  if (!CONNECTED_TERRAINS.includes(tile.terrain)) return [tile.terrain];
  const mask = directionMask(tile.connections!);
  if (tile.terrain === "road" && tile.deadEnds?.length)
    return [`road_${mask}-end-${directionMask(tile.deadEnds)}`];
  return [`${tile.terrain}_${mask}`];
}
