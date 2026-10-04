import type { MapDefinition, TerrainType, UnitType } from "./types";
import { UNIT_TYPES } from "./types";

// Explicit ordering keeps map data independent of engine definition imports.
const terrainTypes = [
  "plains",
  "forest",
  "hill",
  "water",
  "bridge",
  "ford",
  "village",
  "keep",
  "marsh",
  "ruins",
  "fort",
  "grove",
] as const satisfies readonly TerrainType[];

const unitTypes = UNIT_TYPES;

export const DEVELOPER_TERRAIN_SAMPLES = terrainTypes.map((terrain, index) => ({
  terrain,
  col: index % 6,
  row: Math.floor(index / 6),
}));

export const DEVELOPER_UNIT_SAMPLES = unitTypes.flatMap((type, index) => {
  const col = (index % 2) * 3;
  const row = 3 + Math.floor(index / 2);
  return [
    { type, side: "player" as const, col, row },
    { type, side: "enemy" as const, col: col + 1, row },
  ];
});

export const DEVELOPER_MAP = {
  id: "developer",
  revision: 1,
  name: "Developer map",
  description: "All terrains and both sides of every unit.",
  cols: 6,
  rows: 3 + Math.ceil(unitTypes.length / 2),
  terrain: DEVELOPER_TERRAIN_SAMPLES,
  units: DEVELOPER_UNIT_SAMPLES,
} satisfies MapDefinition;
