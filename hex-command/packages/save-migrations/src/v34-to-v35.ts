import { entryInfo } from "./formats/history-info";
import type { Save, Entry, State, Setup } from "./formats/history";
import { readSave } from "./formats/read";
import { isRecord } from "./formats/envelope-v39";

/** Remove archived-position variants and resolve all metadata before runtime use. */
export function migrateV34ToV35(input: unknown): Save<35> {
  const value = readSave(input, 34);
  if (value.version !== 34 || !isRecord(value.checkpoint) || !isRecord(value.checkpoint.state))
    throw new Error("Invalid version-thirty-four save");
  const checkpoint = structuredClone(value.checkpoint);
  const state = checkpoint.state;
  if (state.combatRng === undefined) state.combatRng = 0x6d2b79f5;
  if (!Array.isArray(state.units)) throw new Error("Invalid historical units");
  for (const unit of state.units) {
    if (!isRecord(unit)) throw new Error("Invalid historical unit");
    if (unit.experience === undefined) unit.experience = 0;
  }
  const setup: Setup<35> =
    value.setup === undefined
      ? { first: "player", enemy: { policy: "classic", diplomacy: "diplomatic" } }
      : value.setup;
  let entry: Entry<35>;
  if (value.progress === undefined) {
    // Stable data-only identity for an old snapshot that had no save identities.
    const text = JSON.stringify(value.checkpoint);
    const hashes = [2166136261, 2246822519, 3266489917, 668265263];
    for (let i = 0; i < text.length; i++)
      for (let j = 0; j < hashes.length; j++)
        hashes[j] = Math.imul(hashes[j]! ^ text.charCodeAt(i), 16777619);
    const id = `migrated-${hashes.map((h) => (h >>> 0).toString(16).padStart(8, "0")).join("")}`;
    const campaign = state.campaign;
    const mission = isRecord(campaign)
      ? `Mission ${Number(campaign.mission) + 1} · ${String(state.mapId)}`
      : String(state.mapId);
    entry = {
      id: `${id}-branch`,
      runId: id,
      branchId: `${id}-branch`,
      checkpointId: `${id}-point`,
      ordinal: 0,
      savedAt: 0,
      run: isRecord(campaign) ? campaign.id : state.mapId,
      mission,
      round: state.round,
      side: state.turn,
      kind: "recovery",
      missionStart: false,
    };
  } else {
    if (!isRecord(value.progress) || !isRecord(value.progress.entry))
      throw new Error("Invalid historical progress identity");
    const old = value.progress.entry;
    if (!["turn", "recovery", "resume"].includes(String(old.kind)))
      throw new Error("Invalid historical checkpoint kind");
    entry = {
      ...entryInfo(old),
      kind: old.kind === "turn" ? "turn" : "recovery",
      ...(old.parentId === undefined ? {} : { parentId: old.parentId }),
    };
    if (entry.kind !== "turn") {
      entry.kind = "recovery";
      entry.id = entry.branchId;
      entry.missionStart = false;
      delete entry.parentId;
    }
  }
  const { undo: _undo, ...rest } = value;
  return {
    ...rest,
    version: 35,
    setup,
    checkpoint: {
      ...checkpoint,
      state: { ...checkpoint.state, combatRng: Number(state.combatRng) },
    },
    progress: { entry },
    ...(entry.kind === "recovery" ? { undo: [] } : {}),
  };
}
