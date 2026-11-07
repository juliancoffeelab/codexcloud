import type { Save } from "./formats/history";
import { readSave, readFragment } from "./formats/read";
import { currentSave, isRecord } from "./formats/envelope-v39";
import type * as V38 from "./formats/native-v38";
import type * as V39 from "./formats/native-v39";
import { convertStateAndUndo } from "./legacy/undo-data";

function hex(value: string): V39.Hex {
  if (typeof value !== "string") throw new Error("Invalid historical hex address");
  const match = /^(-?\d+),(-?\d+)$/.exec(value);
  if (!match) throw new Error("Invalid historical hex address");
  const col = Number(match[1]),
    row = Number(match[2]);
  if (!Number.isSafeInteger(col) || !Number.isSafeInteger(row))
    throw new Error("Invalid historical hex coordinates");
  return { col, row };
}
function reference(value: string): V39.TileRef {
  if (typeof value !== "string" || !value.length)
    throw new Error("Invalid historical tile reference");
  return /^-?\d+,-?\d+$/.test(value) ? { hex: hex(value) } : { location: value };
}

/** One typed condition conversion for every owner, including faction transfers. */
function condition(value: V38.Condition): V39.Condition {
  if (!isRecord(value)) throw new Error("Invalid historical condition");
  switch (value.kind) {
    case "all":
    case "any":
      if (!Array.isArray(value.conditions)) throw new Error("Invalid historical conditions");
      return { ...value, conditions: value.conditions.map(condition) };
    case "owner":
    case "proximity": {
      const { tile, ...rest } = value;
      return tile === undefined ? rest : { ...rest, tile: reference(tile) };
    }
    case "visibleUnits":
    case "hasUnits":
    case "noUnits":
    case "choice":
    case "flag":
    case "event":
    case "round":
      return value;
    default:
      throw new Error("Unknown historical condition");
  }
}
const rule = (value: V38.ConditionRule): V39.ConditionRule => ({
  ...value,
  condition: condition(value.condition),
});
function light(value: V38.TileLight): V39.TileLight {
  const { when, ...rest } = value;
  return { ...rest, ...(when ? { when: rule(when) } : {}) };
}
function tile(value: V38.Tile): V39.Tile {
  const { fog, light: illumination, ...rest } = value;
  return {
    ...rest,
    ...(fog ? { fog: rule(fog) } : {}),
    ...(illumination ? { light: light(illumination) } : {}),
  };
}
function faction(value: V38.Faction): V39.Faction {
  const { transfer, ...rest } = value;
  return { ...rest, ...(transfer ? { transfer: { ...transfer, when: rule(transfer.when) } } : {}) };
}
function mark(value: V38.MapMark): V39.MapMark {
  const { tile, light: illumination, ...rest } = value;
  return { ...rest, tile: hex(tile), ...(illumination ? { light: light(illumination) } : {}) };
}
function campaign(value: V38.CampaignState): V39.CampaignState {
  const { scripts, evacuated, ...rest } = value;
  return {
    ...rest,
    ...(evacuated
      ? { evacuated: evacuated.map((entry) => ({ ...entry, unit: unit(entry.unit) })) }
      : {}),
    ...(scripts
      ? {
          scripts: {
            ...scripts,
            marks: scripts.marks.map(mark),
            pending: scripts.pending.map((spawn) => ({ ...spawn, area: spawn.area.map(hex) })),
          },
        }
      : {}),
  };
}

function unit(value: V38.PublicUnit): V39.PublicUnit {
  return { ...value, tile: hex(value.tile) };
}

/** Convert complete historical states, never infer a field's meaning from its path. */
function state(input: unknown): V39.State {
  const value = readFragment(input, "nativeState38");
  const { campaign: progress, ...rest } = value;
  return {
    ...rest,
    units: value.units.map(unit),
    groundItems: value.groundItems.map((item) => ({ ...item, tile: hex(item.tile) })),
    tiles: value.tiles.map(tile),
    factions: value.factions.map(faction),
    ...(progress ? { campaign: campaign(progress) } : {}),
  };
}

export function migrateV38ToV39(input: unknown): Save<39> {
  const value = readSave(input, 38);
  if (value.version !== 38 || !isRecord(value.checkpoint) || !isRecord(value.checkpoint.state))
    throw new Error("Invalid version-thirty-eight save");
  const converted = convertStateAndUndo(value.checkpoint.state, value.undo, state);
  return currentSave({
    ...value,
    version: 39,
    checkpoint: { ...value.checkpoint, state: converted.state },
    ...(converted.undo === undefined ? {} : { undo: converted.undo }),
  });
}
