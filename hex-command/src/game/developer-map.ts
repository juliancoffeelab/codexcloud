import type { MapDefinition, TerrainType, UnitType } from "./types";

// Explicit ordering keeps the gallery independent of engine definition imports.
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

const unitTypes = [
  "commander",
  "soldier",
  "archer",
  "scout",
  "warden",
  "adept",
  "skeleton",
  "bonearcher",
  "wolf",
  "boar",
  "wildkeeper",
  "necromancer",
  "thorncaller",
] as const satisfies readonly UnitType[];

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
  name: "Developer showcase",
  description: "Inspect every terrain and both sides of every unit. No battle actions.",
  cols: 6,
  rows: 10,
  terrain: DEVELOPER_TERRAIN_SAMPLES,
  units: DEVELOPER_UNIT_SAMPLES,
} satisfies MapDefinition;
