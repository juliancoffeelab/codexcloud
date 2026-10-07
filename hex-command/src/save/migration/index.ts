import { currentSave, isRecord, SAVE_VERSION, type BattleSave } from "../schema";
import { migrateV1ToV2 } from "./v1-to-v2";
import { migrateV2ToV3 } from "./v2-to-v3";

export interface MigrationResult {
  save: BattleSave;
  migrated: boolean;
}
/** Pure, sequential conversion. Storage is rewritten only after native restoration. */
export function migrateSave(input: unknown): MigrationResult {
  if (!isRecord(input)) throw new Error("Invalid saved battle");
  let value = input;
  // The first native envelope placed version one inside its checkpoint.
  if (value.version === undefined && value.engine === "rust-wasm") value = { ...value, version: 1 };
  if (
    !Number.isSafeInteger(value.version) ||
    Number(value.version) < 1 ||
    Number(value.version) > SAVE_VERSION
  )
    throw new Error(`Unsupported save version ${String(value.version)}`);
  const originalVersion = value.version;
  while (value.version !== SAVE_VERSION) {
    switch (value.version) {
      case 1:
        value = migrateV1ToV2(value) as unknown as Record<string, unknown>;
        break;
      case 2:
        value = migrateV2ToV3(value) as unknown as Record<string, unknown>;
        break;
      default:
        throw new Error(`No migration from save version ${String(value.version)}`);
    }
  }
  return { save: currentSave(value), migrated: originalVersion !== SAVE_VERSION };
}
