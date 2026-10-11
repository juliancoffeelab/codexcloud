import type { Coordinates, TerrainType } from "../game/types";
export function hexPoints(x: number, y: number) {
  return Array.from({ length: 6 }, (_, i) => {
    const angle = ((60 * i - 30) * Math.PI) / 180;
    return `${x + 35 * Math.cos(angle)},${y + 35 * Math.sin(angle)}`;
  }).join(" ");
}
export function tileCenter(tile: Coordinates) {
  return {
    x: 43 + 61 * (tile.col + (tile.row % 2) / 2),
    y: 42 + 53 * tile.row,
  };
}
export const previewColors: Record<TerrainType, string> = {
  marsh: "#809678",
  ruins: "#a2a18c",
  fort: "#b4a277",
  grove: "#4d884e",
  plains: "#8d9f4f",
  forest: "#536e3d",
  hill: "#85866c",
  water: "#407899",
  bridge: "#b3ab8d",
  ford: "#78a4a6",
  village: "#d4b476",
  keep: "#b9b597",
};
