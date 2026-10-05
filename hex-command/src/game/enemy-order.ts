import type { Tile, Unit } from "./types";

const axial = (tile: Pick<Tile, "col" | "row">) => ({
  q: tile.col - Math.floor(tile.row / 2),
  r: tile.row,
});

/** Resolve equal choices relative to home, never screen coordinates or array order. */
export function enemyTileOrder(
  tiles: readonly Tile[],
  units: readonly Unit[],
  unit: Unit,
  preferHomeVillages = false,
) {
  const origin = tiles.find((tile) => tile.id === unit.tile)!;
  const center = (locations: readonly Tile[]) => {
    const points = locations.map(axial);
    return {
      q: points.reduce((sum, point) => sum + point.q, 0) / points.length,
      r: points.reduce((sum, point) => sum + point.r, 0) / points.length,
    };
  };
  const keeps = tiles.filter((tile) => tile.terrain === "keep");
  const homeKeeps = keeps.filter((tile) => tile.owner === unit.side);
  const home = center(homeKeeps.length ? homeKeeps : [origin]);
  const opposingKeeps = keeps.filter((tile) => tile.owner && tile.owner !== unit.side);
  const enemies = units
    .filter((other) => other.side !== unit.side)
    .map((other) => tiles.find((tile) => tile.id === other.tile)!);
  const target = center(opposingKeeps.length ? opposingKeeps : enemies.length ? enemies : tiles);
  let dq = target.q - home.q,
    dr = target.r - home.r;
  if (dq === 0 && dr === 0) {
    const position = axial(origin);
    dq = position.q - home.q;
    dr = position.r - home.r;
  }
  // A central unit without directional landmarks has no distinguishable homeward axis.
  if (dq === 0 && dr === 0) dq = unit.side === "player" ? 1 : -1;
  const homeDistance = (point: ReturnType<typeof axial>) =>
    (Math.abs(point.q - home.q) +
      Math.abs(point.r - home.r) +
      Math.abs(point.q + point.r - home.q - home.r)) /
    2;
  return (first: Tile, second: Tile): number => {
    const a = axial(first),
      b = axial(second);
    // Equally useful village captures start near our keep, whichever side owns it.
    const aVillage = preferHomeVillages && first.terrain === "village",
      bVillage = preferHomeVillages && second.terrain === "village";
    return (
      Number(bVillage) - Number(aVillage) ||
      (aVillage && bVillage ? homeDistance(a) - homeDistance(b) : 0) ||
      (b.q - a.q) * dq + (b.r - a.r) * dr ||
      (a.q - b.q) * dr - (a.r - b.r) * dq
    );
  };
}
