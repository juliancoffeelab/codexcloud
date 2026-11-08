import type { Coordinates, TerrainType } from "../game/types";
export const battlefieldSpriteSize = 70;
// Native hexes tessellate on a 56×48 pixel lattice (half-column: 28).
// Fractional native offsets shift connected artwork by a pixel at tile seams.
export const hexColumnSpacing = (battlefieldSpriteSize * 56) / 64;
export const hexRowSpacing = (battlefieldSpriteSize * 3) / 4;

export function mapDimensions(map: { cols: number; rows: number }) {
  return {
    width: 758 + hexColumnSpacing * (map.cols - 11),
    height: 452 + hexRowSpacing * (map.rows - 7),
  };
}

export function hexPoints(x: number, y: number) {
  return Array.from({ length: 6 }, (_, i) => {
    const angle = ((60 * i - 30) * Math.PI) / 180;
    return `${x + (hexColumnSpacing / Math.sqrt(3)) * Math.cos(angle)},${y + (battlefieldSpriteSize / 2) * Math.sin(angle)}`;
  }).join(" ");
}
// Keep the origin on native pixel (39, 38), not only the spacing between tiles.
export function tileCenter(tile: Coordinates) {
  return {
    x: ((39 + 56 * (tile.col + (tile.row % 2) / 2)) * battlefieldSpriteSize) / 64,
    y: ((38 + 48 * tile.row) * battlefieldSpriteSize) / 64,
  };
}
export const previewColors: Record<TerrainType, string> = {
  monastery: "#b9b597",
  church: "#b9b597",
  graveyard: "#8d9f4f",
  "ritual-star": "#8d9f4f",
  "ritual-spent": "#8d9f4f",
  marsh: "#809678",
  ruins: "#a2a18c",
  fort: "#b4a277",
  "castle-wall": "#b9b597",
  "castle-bell": "#b9b597",
  grove: "#4d884e",
  plains: "#8d9f4f",
  road: "#8fb337",
  signpost: "#8fb337",
  forest: "#536e3d",
  hill: "#85866c",
  water: "#407899",
  bridge: "#b3ab8d",
  ford: "#78a4a6",
  landing: "#78a4a6",
  ferry: "#78a4a6",
  house: "#d4b476",
  mountain: "#727063",
  cave: "#727063",
  "burned-village": "#8d9f4f",
  village: "#d4b476",
  keep: "#b9b597",
};
