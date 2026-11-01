import { isRecord } from "../schema";

// Frozen v16 connection order and gate rule. Never consult the current catalog.
const directions = ["e", "se", "sw", "w", "nw", "ne"];
const families = new Set(["castle-wall", "castle-gate", "castle-bell"]);

/** Preserve the battle while replacing neighbor-dependent castle geometry. */
export function migrateV16ToV17(value: Record<string, unknown>): Record<string, unknown> {
  if (value.version !== 16 || !isRecord(value.checkpoint) || !isRecord(value.checkpoint.state))
    throw new Error("Invalid version-sixteen save");
  const state = value.checkpoint.state;
  if (!Array.isArray(state.tiles) || !state.tiles.every(isRecord))
    throw new Error("Invalid historical tile grid");
  const tiles = state.tiles.map((tile) => {
    if (typeof tile.terrain !== "string" || !families.has(tile.terrain)) return tile;
    const connections = tile.connections;
    if (
      !Array.isArray(connections) ||
      !connections.every((edge) => typeof edge === "string" && directions.includes(edge)) ||
      new Set(connections).size !== connections.length ||
      !Array.isArray(tile.wallCorners) ||
      tile.wallCorners.length !== 6
    )
      throw new Error("Invalid historical castle geometry");
    const boundary = directions.filter((edge) => !connections.includes(edge));
    const gates =
      tile.terrain === "castle-gate"
        ? boundary.filter((edge) => {
            const i = directions.indexOf(edge);
            return (
              boundary.includes(directions[(i + 5) % 6]!) &&
              boundary.includes(directions[(i + 1) % 6]!)
            );
          })
        : [];
    if (tile.terrain === "castle-gate" && gates.length !== 1)
      throw new Error("Invalid historical gate orientation");
    const { connections: _connections, wallCorners: _corners, ...rest } = tile;
    return { ...rest, walls: boundary.filter((edge) => !gates.includes(edge)), gates };
  });
  return { ...value, version: 17, checkpoint: { ...value.checkpoint, state: { ...state, tiles } } };
}
