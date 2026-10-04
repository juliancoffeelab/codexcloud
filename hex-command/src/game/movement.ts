import type {
  TerrainType,
  TerrainModifier,
  TerrainDefinition,
  RaceDefinition,
  UnitDefinition,
} from "./types";

export const HEAVY_MOVEMENT = Object.freeze({
  ford: { cost: 1 },
  marsh: { cost: 1 },
} satisfies Partial<Record<TerrainType, TerrainModifier>>);

export const WOODLAND_MOVEMENT = Object.freeze({
  forest: { cost: -1 },
  grove: { cost: -1 },
} satisfies Partial<Record<TerrainType, TerrainModifier>>);

export const PATHFINDER_MOVEMENT = Object.freeze({
  ...WOODLAND_MOVEMENT,
  hill: { cost: -1 },
} satisfies Partial<Record<TerrainType, TerrainModifier>>);

/** All callers resolve the same access and cost; modifiers never make water land. */
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
