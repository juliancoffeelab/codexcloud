// Explicit current-fixture coordinates; historical fixtures keep their own formats.
export const cellKey = (value) => (typeof value === "string" ? value : `${value.col},${value.row}`);
export function hex(value) {
  if (typeof value !== "string") return { col: value.col, row: value.row };
  if (!/^-?\d+,-?\d+$/.test(value)) throw new Error(`Invalid fixture coordinates: ${value}`);
  const [col, row] = value.split(",").map(Number);
  return { col, row };
}
