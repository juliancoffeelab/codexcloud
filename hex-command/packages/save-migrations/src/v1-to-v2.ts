import type { Save } from "./formats/history";
import { readSave, readVersionOne } from "./formats/read";
import { MAPS } from "./legacy/frozen";
import { importLegacySnapshot } from "./legacy/snapshot";
import { isRecord } from "./formats/envelope-v39";

export type VersionTwoSave = Save<2>;

/** Both original browser formats were version one; the engine tag distinguishes them. */
export function migrateV1ToV2(value: Record<string, unknown>): VersionTwoSave {
  if (value.engine === "rust-wasm") {
    const native = readVersionOne(value, "native");
    const saved = native.saved;
    if (!isRecord(saved) || saved.version !== 1 || typeof value.pendingEnemy !== "boolean")
      throw new Error("Invalid version-one native save");
    const { version: _version, ...checkpoint } = saved;
    return readSave({ version: 2, checkpoint, pendingEnemy: value.pendingEnemy }, 2);
  }
  if (value.engine !== undefined) throw new Error("Unknown historical engine");
  const { state, pendingEnemy } = importLegacySnapshot(readVersionOne(value, "javascript"));
  // The imported tiles preserve the original board; name the current catalog revision.
  state.mapRevision = MAPS[state.mapId].revision ?? 1;
  return readSave({ version: 2, checkpoint: { seed: 17, state }, pendingEnemy }, 2);
}
