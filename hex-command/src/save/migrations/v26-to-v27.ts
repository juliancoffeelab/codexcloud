import { isRecord } from "../schema";
import boards from "./legacy/data/tiles-v26.json";

/** Rekey historical board references; coordinates remain explicit geometric facts. */
export function migrateV26ToV27(value: Record<string, unknown>): Record<string, unknown> {
  if (value.version !== 26 || !isRecord(value.checkpoint) || !isRecord(value.checkpoint.state))
    throw new Error("Invalid version-twenty-six save");
  const old = value.checkpoint.state;
  const ids = boards[old.mapId as keyof typeof boards] as Record<string, string> | undefined;
  if (!ids) throw new Error("Unknown historical board");
  const tile = (input: unknown): string => {
    if (typeof input !== "string" || !ids[input]) throw new Error("Unknown historical tile");
    return ids[input]!;
  };
  const record = (input: unknown): Record<string, unknown> => {
    if (!isRecord(input)) throw new Error("Invalid historical board reference");
    return input;
  };
  const list = (input: unknown): unknown[] => {
    if (!Array.isArray(input)) throw new Error("Invalid historical board list");
    return input;
  };
  const condition = (input: unknown): unknown => {
    const c = record(input);
    return {
      ...c,
      ...(c.tile != null ? { tile: tile(c.tile) } : {}),
      ...(Array.isArray(c.conditions) ? { conditions: c.conditions.map(condition) } : {}),
    };
  };
  const rule = (input: unknown) => {
    const r = record(input);
    return { ...r, condition: condition(r.condition) };
  };
  const unit = (input: unknown) => {
    const u = record(input);
    return { ...u, tile: tile(u.tile) };
  };
  const marker = (input: unknown) => {
    if (typeof input !== "string") throw new Error("Invalid historical marker");
    return input.replace(/\d+,\d+/g, (coordinate) => tile(coordinate));
  };
  const state: Record<string, unknown> = {
    ...old,
    tiles: list(old.tiles).map((input) => {
      const t = record(input);
      return {
        ...t,
        id: tile(t.id),
        ...(t.fog ? { fog: rule(t.fog) } : {}),
        ...(isRecord(t.light) && t.light.when
          ? { light: { ...t.light, when: rule(t.light.when) } }
          : {}),
      };
    }),
    units: list(old.units).map(unit),
    groundItems: list(old.groundItems).map((input) => {
      const item = record(input);
      return { ...item, tile: tile(item.tile) };
    }),
    factions: list(old.factions).map((input) => {
      const f = record(input);
      return {
        ...f,
        ...(isRecord(f.transfer)
          ? { transfer: { ...f.transfer, when: rule(f.transfer.when) } }
          : {}),
      };
    }),
    ...(Array.isArray(old.conditionMemory)
      ? { conditionMemory: old.conditionMemory.map(marker) }
      : {}),
  };
  if (isRecord(old.campaign)) {
    const c = old.campaign;
    state.campaign = {
      ...c,
      marks: list(c.marks).map(marker),
      ...(isRecord(c.holds)
        ? {
            holds: Object.fromEntries(
              Object.entries(c.holds).map(([key, counter]) => [tile(key), counter]),
            ),
          }
        : {}),
      ...(Array.isArray(c.evacuated)
        ? {
            evacuated: c.evacuated.map((input) => {
              const e = record(input);
              return { ...e, unit: unit(e.unit) };
            }),
          }
        : {}),
    };
  }
  return { ...value, version: 27, checkpoint: { ...value.checkpoint, state } };
}
