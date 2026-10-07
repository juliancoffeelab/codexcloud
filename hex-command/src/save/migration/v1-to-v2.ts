import type { SavedGame } from "../../../rust/pkg/hex_command";
import { MAPS } from "../../game/maps";
import { importLegacySnapshot } from "./legacy/snapshot";
import { isRecord, type BattleSave } from "../schema";

/** Both original browser formats were version one; the engine tag distinguishes them. */
export function migrateV1ToV2(value: Record<string, unknown>): BattleSave {
  if (value.engine === "rust-wasm") {
    const saved = value.saved;
    if (!isRecord(saved) || saved.version !== 1 || typeof value.pendingEnemy !== "boolean")
      throw new Error("Invalid version-one native save");
    const { version: _version, ...checkpoint } = saved;
    return {
      version: 2,
      checkpoint: checkpoint as unknown as SavedGame,
      pendingEnemy: value.pendingEnemy,
    };
  }
  if (value.engine !== undefined) throw new Error("Unknown historical engine");
  const { state, pendingEnemy } = importLegacySnapshot(value);
  // Only river retained a separately published layout. Other TS revision-one
  // snapshots used the current layout and must name its supported revision.
  if (state.mapId !== "river") state.mapRevision = MAPS[state.mapId].revision ?? 1;
  return {
    version: 2,
    checkpoint: { seed: 17, state: state as unknown as SavedGame["state"] },
    pendingEnemy,
  };
}
