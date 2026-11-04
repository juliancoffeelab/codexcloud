import { lightPosition, type BattlefieldLighting } from "./battlefield-lighting";

// A fragment visits only lights whose influence intersects its cell. Keep this
// independent of terrain: death marks are ordinary light sources, never artwork.
export const lightCellSize = 64;
export function lightGrid(
  lighting: BattlefieldLighting,
  width: number,
  height: number,
  scale: number,
) {
  const cols = Math.ceil(width / lightCellSize),
    rows = Math.ceil(height / lightCellSize);
  const cells: number[][] = Array.from({ length: cols * rows }, () => []);
  for (const [index, light] of (lighting.lights ?? []).entries()) {
    const point = lightPosition(light);
    const x = point.x * scale,
      y = point.y * scale,
      radius = light.radius * scale;
    for (
      let row = Math.max(0, Math.floor((y - radius / 1.12) / lightCellSize));
      row <= Math.min(rows - 1, Math.floor((y + radius / 1.12) / lightCellSize));
      row++
    ) {
      for (
        let col = Math.max(0, Math.floor((x - radius) / lightCellSize));
        col <= Math.min(cols - 1, Math.floor((x + radius) / lightCellSize));
        col++
      ) {
        const dx = Math.max(col * lightCellSize - x, 0, x - (col + 1) * lightCellSize);
        const dy = Math.max(row * lightCellSize - y, 0, y - (row + 1) * lightCellSize) * 1.12;
        if (dx * dx + dy * dy <= radius * radius) cells[row * cols + col]!.push(index);
      }
    }
  }
  const ranges = new Uint32Array(cols * rows * 2),
    indices: number[] = [];
  cells.forEach((cell, index) => {
    ranges[index * 2] = indices.length;
    ranges[index * 2 + 1] = cell.length;
    indices.push(...cell);
  });
  return { cols, rows, ranges, indices: new Uint32Array(indices.length ? indices : [0]) };
}
