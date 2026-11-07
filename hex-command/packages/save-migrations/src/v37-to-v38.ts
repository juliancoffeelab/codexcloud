import type { Save, State, Condition, Rule, Light } from "./formats/history";
import { readSave } from "./formats/read";
import { convertStateAndUndo } from "./legacy/undo-data";
import frozen from "./legacy/data/locations-v37.json";

const locations: Record<string, Record<string, string>> = frozen;
const events = new Map([
  ["sealed-hex-0941-278b487380", "northern-seal-closed"],
  ["sealed-hex-0957-c94afbbfa1", "southern-seal-closed"],
]);

/** Convert declared references only: prose, item data and save identities are opaque. */
export function migrateV37ToV38(input: unknown): Save<38> {
  const value = readSave(input, 37);
  function state(input: unknown): State<38> {
    const old = readSave({ ...value, checkpoint: { ...value.checkpoint, state: input } }, 37)
      .checkpoint.state;
    const names = locations[old.mapId] ?? {};
    const addresses = new Map<string, string>();
    const occupied = new Set<string>();
    for (const tile of old.tiles) {
      if (!Number.isSafeInteger(tile.col) || !Number.isSafeInteger(tile.row))
        throw new Error("Invalid historical cell");
      const address = `${tile.col},${tile.row}`;
      if (addresses.has(tile.id) || occupied.has(address))
        throw new Error("Duplicate historical cell");
      addresses.set(tile.id, address);
      occupied.add(address);
    }
    function tile(id: string): string {
      const address = addresses.get(id);
      if (address === undefined) throw new Error("Unknown historical cell reference");
      return address;
    }
    // These two renamed event/objective/flag IDs belong to the Ashen campaign.
    const event = (id: string) => (old.campaign?.id === "ashen-oath" ? (events.get(id) ?? id) : id);
    function condition(c: Condition<37>): Condition<38> {
      switch (c.kind) {
        case "all":
        case "any":
          return { ...c, conditions: c.conditions.map(condition) };
        case "owner":
        case "proximity":
          return { ...c, ...(c.tile === undefined ? {} : { tile: tile(c.tile) }) };
        case "event":
          return { ...c, event: event(c.event) };
        case "flag":
          return { ...c, flag: event(c.flag) };
        case "choice":
          return { ...c, choice: event(c.choice) };
        default:
          return c;
      }
    }
    const rule = (r: Rule<37>): Rule<38> => ({ ...r, condition: condition(r.condition) });
    const light = (l: Light<37>): Light<38> => ({
      ...l,
      ...(l.when ? { when: rule(l.when) } : {}),
    });
    const unit = <T extends { tile: string }>(u: T): T => ({ ...u, tile: tile(u.tile) });
    function memory(key: string): string {
      for (const prefix of ["fog:", "light:"])
        if (key.startsWith(prefix)) return prefix + tile(key.slice(prefix.length));
      return key;
    }
    function marker(key: string): string {
      if (key.startsWith("event:")) return "event:" + event(key.slice(6));
      if (key.startsWith("sealed:")) return "sealed:" + tile(key.slice(7));
      return key;
    }
    function keys<T>(
      values: Record<string, T>,
      rename: (key: string) => string,
    ): Record<string, T> {
      const entries = Object.entries(values).map(([id, value]) => [rename(id), value] as const);
      if (new Set(entries.map(([id]) => id)).size !== entries.length)
        throw new Error("Ambiguous historical reference rename");
      return Object.fromEntries(entries);
    }
    const campaign = old.campaign;
    const scripts = campaign?.scripts;
    return {
      ...old,
      tiles: old.tiles.map((t) => ({
        ...t,
        id: tile(t.id),
        ...(names[t.id] ? { location: names[t.id] } : {}),
        ...(t.fog ? { fog: rule(t.fog) } : {}),
        ...(t.light ? { light: light(t.light) } : {}),
      })),
      units: old.units.map(unit),
      groundItems: old.groundItems.map(unit),
      factions: old.factions.map((f) => ({
        ...f,
        ...(f.transfer ? { transfer: { ...f.transfer, when: rule(f.transfer.when) } } : {}),
      })),
      ...(old.conditionMemory ? { conditionMemory: old.conditionMemory.map(memory) } : {}),
      ...(campaign
        ? {
            campaign: {
              ...campaign,
              flags: keys(campaign.flags, event),
              marks: campaign.marks.map(marker),
              ...(campaign.holds
                ? {
                    holds: Object.fromEntries(
                      Object.entries(campaign.holds).map(([id, counter]) => [tile(id), counter]),
                    ),
                  }
                : {}),
              ...(campaign.evacuated
                ? {
                    evacuated: campaign.evacuated.map((e) => ({
                      ...e,
                      event: event(e.event),
                      unit: unit(e.unit),
                    })),
                  }
                : {}),
              ...(scripts
                ? {
                    scripts: {
                      ...scripts,
                      active: scripts.active.map(event),
                      completed: scripts.completed.map(event),
                      revealed: scripts.revealed.map(event),
                      ...(scripts.failed ? { failed: scripts.failed.map(event) } : {}),
                      holds: scripts.holds.map((h) => ({ ...h, event: event(h.event) })),
                      marks: scripts.marks.map((m) => ({
                        ...m,
                        tile: tile(m.tile),
                        ...(m.light ? { light: light(m.light) } : {}),
                      })),
                      pending: scripts.pending.map((p) => ({ ...p, area: p.area.map(tile) })),
                    },
                  }
                : {}),
            },
          }
        : {}),
    };
  }
  const converted = convertStateAndUndo(value.checkpoint.state, value.undo, state);
  return {
    ...value,
    version: 38,
    checkpoint: { ...value.checkpoint, state: converted.state },
    ...(converted.undo === undefined ? {} : { undo: converted.undo }),
  };
}
