import type { TerrainType } from "./types";

export const HEAVY_MOVEMENT = Object.freeze({
  ford: 1,
  marsh: 1,
} satisfies Partial<Record<TerrainType, number>>);

export const WOODLAND_MOVEMENT = Object.freeze({
  forest: -1,
  grove: -1,
} satisfies Partial<Record<TerrainType, number>>);

export const PATHFINDER_MOVEMENT = Object.freeze({
  ...WOODLAND_MOVEMENT,
  hill: -1,
} satisfies Partial<Record<TerrainType, number>>);
