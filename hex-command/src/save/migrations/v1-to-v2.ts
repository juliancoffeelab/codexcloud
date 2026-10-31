import type { SavedGame } from "../../../rust/pkg/hex_command";
import { MAPS } from "./legacy/frozen";
import { importLegacySnapshot } from "./legacy/snapshot";
import { isRecord, type BattleSave } from "../schema";

export type VersionTwoSave = Omit<BattleSave, "version"> & { version: 2 };

/** Both original browser formats were version one; the engine tag distinguishes them. */
export function migrateV1ToV2(value: Record<string, unknown>): VersionTwoSave {
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
  // The imported tiles preserve the original board; name the current catalog revision.
  state.mapRevision = MAPS[state.mapId].revision ?? 1;
  return {
    version: 2,
    checkpoint: { seed: 17, state: state as unknown as SavedGame["state"] },
    pendingEnemy,
  };
}
