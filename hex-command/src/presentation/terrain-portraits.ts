import type { TerrainType } from "../game/types";

// UI portraits are explicit presentation assets, never map geometry defaults.
// Every orientation, floor and junction configuration of a terrain shares this image.
export const terrainPortraits = {
  plains: ["plains_layout_0", "green-plain"],
  road: ["road_9", "green-plain"],
  forest: ["forest_layout_0", "green-forest"],
  hill: ["hill_layout_0", "green-plain"],
  water: ["water", "dark-water"],
  bridge: ["bridge_9", "dark-water"],
  ford: ["ford_9_layout_0", "shallow-water"],
  village: ["village_0", "green-plain"],
  keep: ["keep", "green-plain"],
  marsh: ["marsh", "marsh"],
  ruins: ["ruins", "green-plain"],
  fort: ["fort", "green-plain"],
  grove: ["grove", "green-forest"],
  landing: ["landing_1_layout_0", "shallow-water"],
  ferry: ["ferry_1_layout_0", "dark-water"],
  house: ["house_layout_0", "green-plain"],
  mountain: ["mountain_layout_0", "mountain-rock"],
  cave: ["cave_layout_0", "mountain-rock"],
  "burned-village": ["burned-village_layout_0", "green-plain"],
  signpost: ["signpost_9_layout_0", "green-plain"],
  "castle-wall": ["castle-wall_63", "castle-paving"],
  "castle-bell": ["castle-bell_63", "castle-paving"],
  monastery: ["monastery_layout_0", "green-plain"],
  church: ["church_layout_0", "green-plain"],
  graveyard: ["graveyard_layout_0", "green-plain"],
  "ritual-star": ["ritual-star_layout_0", "mountain-rock"],
  "ritual-spent": ["ritual-spent_layout_0", "mountain-rock"],
} as const satisfies Record<TerrainType, readonly [string, string]>;

export function terrainPortraitKey(terrain: TerrainType) {
  const [name, floor] = terrainPortraits[terrain];
  return `terrain_${name === "grove" ? "sacred_grove" : name}__${floor}`;
}
