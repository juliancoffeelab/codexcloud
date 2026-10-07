import type {
  TerrainType,
  TerrainDefinition,
  RaceDefinition,
  UnitDefinition,
} from "../../../game/types";

/** Version-one TS snapshots rejected units standing on impassable terrain. */
export function movementCost(
  terrain: TerrainDefinition,
  type: TerrainType,
  race: RaceDefinition,
  unit: Pick<UnitDefinition, "terrainModifiers">,
): number {
  const inherited = race.terrainModifiers[type];
  const individual = unit.terrainModifiers?.[type];
  const passable = individual?.passable ?? inherited?.passable ?? terrain.passable;
  return passable
    ? Math.max(1, terrain.cost + (inherited?.cost ?? 0) + (individual?.cost ?? 0))
    : Infinity;
}
