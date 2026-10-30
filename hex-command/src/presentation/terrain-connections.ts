import roadDeadEnds from "../../pixel/art/terrain/road/dead-ends.json";
import type { HexDirection, TerrainType } from "../game/types";

export const TERRAIN_DIRECTIONS: readonly HexDirection[] = ["e", "se", "sw", "w", "nw", "ne"];
type ConnectedTile = {
  terrain: TerrainType;
  connections?: readonly HexDirection[];
  deadEnds?: readonly HexDirection[];
};
const directionMask = (directions: readonly HexDirection[]) =>
  directions.reduce((mask, direction) => mask | (1 << TERRAIN_DIRECTIONS.indexOf(direction)), 0);

/** Finished capped roads are explicitly registered alongside their source matrices. */
export function validTerrainConnections(tile: ConnectedTile): boolean {
  if (tile.connections === undefined) return tile.deadEnds === undefined;
  if (!tile.connections.every((direction) => TERRAIN_DIRECTIONS.includes(direction))) return false;
  const mask = directionMask(tile.connections);
  if (!mask || new Set(tile.connections).size !== tile.connections.length) return false;
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
  const mask = directionMask(tile.connections ?? []);
  if ((tile.terrain === "landing" || tile.terrain === "ferry") && mask === 1)
    return [`${tile.terrain}_east`];
  if (tile.terrain === "road" && tile.deadEnds?.length)
    return [`road_${mask}-end-${directionMask(tile.deadEnds)}`];
  if (tile.terrain === "village") return [`village_${mask}`];
  if (tile.terrain === "road") {
    if (mask && mask !== 9) return [`${tile.terrain}_${mask}`];
  }
  if (tile.terrain === "bridge" || tile.terrain === "ford") {
    if (mask === 18) return [`${tile.terrain}_nw-se`];
    if (mask === 36) return [`${tile.terrain}_ne-sw`];
  }
  return [tile.terrain];
}
