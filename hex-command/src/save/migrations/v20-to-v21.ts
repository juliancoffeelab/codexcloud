import { isRecord } from "../schema";
import frozen from "./legacy/data/faction-economy-v20.json";

/** Preserve existing wealth and ownership; new faction treasuries start empty. */
export function migrateV20ToV21(value: Record<string, unknown>): Record<string, unknown> {
  if (value.version !== 20 || !isRecord(value.checkpoint) || !isRecord(value.checkpoint.state))
    throw new Error("Invalid version-twenty save");
  const old = value.checkpoint.state;
  if (!isRecord(old.gold) || !Array.isArray(old.factions) || !old.factions.every(isRecord))
    throw new Error("Invalid historical faction economy");
  const gold = { ...old.gold };
  let factions = old.factions.map((faction) => {
    if (typeof faction.id !== "string") throw new Error("Invalid historical faction");
    if (!(faction.id in gold)) gold[faction.id] = 0;
    return { ...faction };
  });
  let state: Record<string, unknown> = { ...old, gold, factions };
  if (old.mapId === frozen.mapId && old.mapRevision === frozen.fromRevision) {
    if (
      !Array.isArray(old.tiles) ||
      !old.tiles.every(isRecord) ||
      !Array.isArray(old.units) ||
      !old.units.every(isRecord)
    )
      throw new Error("Invalid historical roadside battle");
    const units = old.units.map((unit) =>
      unit.name === frozen.leader.name && unit.faction === frozen.leader.faction
        ? { ...unit, role: "leader", aiGroup: frozen.leader.aiGroup }
        : unit,
    );
    const tiles = old.tiles.map((tile) => {
      const change = frozen.tiles.find((patch) => patch.id === tile.id);
      if (!change) return tile;
      const occupant = units.find((unit) => unit.tile === tile.id);
      const owner = tile.owner ?? occupant?.faction ?? occupant?.side ?? change.owner;
      const { connections: _oldConnections, ...rest } = tile;
      return { ...rest, ...change, owner };
    });
    factions = factions.map((faction) =>
      faction.id === frozen.faction.id ? { ...faction, ...frozen.faction } : faction,
    );
    state = { ...state, tiles, units, factions, mapRevision: frozen.toRevision };
  }
  return { ...value, version: 21, checkpoint: { ...value.checkpoint, state } };
}
