import type { Coordinates, TerrainType } from "../game/types";
export const battlefieldSpriteSize = 70;
export const hexColumnSpacing = (battlefieldSpriteSize * Math.sqrt(3)) / 2;
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
    return `${x + (battlefieldSpriteSize / 2) * Math.cos(angle)},${y + (battlefieldSpriteSize / 2) * Math.sin(angle)}`;
  }).join(" ");
}
export function tileCenter(tile: Coordinates) {
  return {
    x: 43 + hexColumnSpacing * (tile.col + (tile.row % 2) / 2),
    y: 42 + hexRowSpacing * tile.row,
  };
}
export const previewColors: Record<TerrainType, string> = {
  marsh: "#809678",
  ruins: "#a2a18c",
  fort: "#b4a277",
  grove: "#4d884e",
  plains: "#8d9f4f",
  road: "#8fb337",
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
  village: "#d4b476",
  keep: "#b9b597",
};
