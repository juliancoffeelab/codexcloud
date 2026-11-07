import { isRecord } from "../schema";
import frozen from "./legacy/data/locations-v37.json";

const locations = frozen as Record<string, Record<string, string>>;
const events = new Map(
  Object.entries({
    "sealed-hex-0941-278b487380": "northern-seal-closed",
    "sealed-hex-0957-c94afbbfa1": "southern-seal-closed",
  }),
);

/** Separate historical cell identities from authored landmark names, including undo. */
export function migrateV37ToV38(value: Record<string, unknown>): Record<string, unknown> {
  if (value.version !== 37 || !isRecord(value.checkpoint) || !isRecord(value.checkpoint.state))
    throw new Error("Invalid version-thirty-seven save");
  const state = value.checkpoint.state;
  if (typeof state.mapId !== "string" || !Array.isArray(state.tiles))
    throw new Error("Invalid historical map");
  const names = locations[state.mapId] ?? {};
  const addresses = new Map<string, string>();
  const occupied = new Set<string>();
  for (const tile of state.tiles) {
    if (
      !isRecord(tile) ||
      typeof tile.id !== "string" ||
      !Number.isSafeInteger(tile.col) ||
      !Number.isSafeInteger(tile.row)
    )
      throw new Error("Invalid historical cell");
    const address = `${tile.col},${tile.row}`;
    if (addresses.has(tile.id) || occupied.has(address))
      throw new Error("Duplicate historical cell");
    addresses.set(tile.id, address);
    occupied.add(address);
  }
  function reference(text: string): string {
    const exact = addresses.get(text) ?? events.get(text);
    if (exact !== undefined) return exact;
    for (const prefix of ["fog:", "light:"]) {
      if (text.startsWith(prefix)) {
        const address = addresses.get(text.slice(prefix.length));
        if (address !== undefined) return prefix + address;
      }
    }
    return text;
  }
  function convert(input: unknown): unknown {
    if (typeof input === "string") return reference(input);
    if (Array.isArray(input)) return input.map(convert);
    if (!isRecord(input)) return input;
    const result: Record<string, unknown> = Object.fromEntries(
      Object.entries(input).map(([key, entry]) => [events.get(key) ?? key, convert(entry)]),
    );
    // Full tile records can also appear in backward changes. Preserve all surface facts.
    if (typeof input.id === "string" && "col" in input && "row" in input && "terrain" in input) {
      if (!addresses.has(input.id)) throw new Error("Unknown historical undo cell");
      const location = names[input.id];
      if (location !== undefined) result.location = location;
    }
    return result;
  }
  return { ...(convert(value) as Record<string, unknown>), version: 38 };
}
