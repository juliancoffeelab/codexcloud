import { migrateV25ToV26 } from "./v25-to-v26";
import { migrateV24ToV25 } from "./v24-to-v25";
import { migrateV23ToV24 } from "./v23-to-v24";
import { migrateV22ToV23 } from "./v22-to-v23";
import { migrateV21ToV22 } from "./v21-to-v22";
import { migrateV20ToV21 } from "./v20-to-v21";
import { migrateV19ToV20 } from "./v19-to-v20";
import { migrateV18ToV19 } from "./v18-to-v19";
import { migrateV17ToV18 } from "./v17-to-v18";
import { migrateV16ToV17 } from "./v16-to-v17";
import { migrateV15ToV16 } from "./v15-to-v16";
import { migrateV14ToV15 } from "./v14-to-v15";
import { currentSave, isRecord, SAVE_VERSION, type BattleSave } from "../schema";
import { migrateV1ToV2 } from "./v1-to-v2";
import { migrateV2ToV3 } from "./v2-to-v3";
import { migrateV9ToV10 } from "./v9-to-v10";
import { migrateV10ToV11 } from "./v10-to-v11";
import { migrateV13ToV14 } from "./v13-to-v14";
import { migrateV12ToV13 } from "./v12-to-v13";
import { migrateV11ToV12 } from "./v11-to-v12";
import { migrateV8ToV9 } from "./v8-to-v9";
import { migrateV7ToV8 } from "./v7-to-v8";
import { migrateV6ToV7 } from "./v6-to-v7";
import { migrateV5ToV6 } from "./v5-to-v6";
import { migrateV4ToV5 } from "./v4-to-v5";
import { migrateV3ToV4 } from "./v3-to-v4";

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
      case 25:
        value = migrateV25ToV26(value);
        break;
      case 24:
        value = migrateV24ToV25(value);
        break;
      case 23:
        value = migrateV23ToV24(value);
        break;
      case 22:
        value = migrateV22ToV23(value);
        break;
      case 21:
        value = migrateV21ToV22(value);
        break;
      case 20:
        value = migrateV20ToV21(value);
        break;
      case 19:
        value = migrateV19ToV20(value);
        break;
      case 18:
        value = migrateV18ToV19(value);
        break;
      case 1:
        value = migrateV1ToV2(value) as unknown as Record<string, unknown>;
        break;
      case 2:
        value = migrateV2ToV3(value) as unknown as Record<string, unknown>;
        break;
      case 3:
        value = migrateV3ToV4(value) as unknown as Record<string, unknown>;
        break;
      case 4:
        value = migrateV4ToV5(value) as unknown as Record<string, unknown>;
        break;
      case 5:
        value = migrateV5ToV6(value) as unknown as Record<string, unknown>;
        break;
      case 6:
        value = migrateV6ToV7(value) as unknown as Record<string, unknown>;
        break;
      case 7:
        value = migrateV7ToV8(value);
        break;
      case 8:
        value = migrateV8ToV9(value);
        break;
      case 9:
        value = migrateV9ToV10(value);
        break;
      case 10:
        value = migrateV10ToV11(value);
        break;
      case 11:
        value = migrateV11ToV12(value);
        break;
      case 12:
        value = migrateV12ToV13(value);
        break;
      case 17:
        value = migrateV17ToV18(value);
        break;
      case 16:
        value = migrateV16ToV17(value);
        break;
      case 15:
        value = migrateV15ToV16(value);
        break;
      case 14:
        value = migrateV14ToV15(value);
        break;
      case 13:
        value = migrateV13ToV14(value);
        break;
      default:
        throw new Error(`No migration from save version ${String(value.version)}`);
    }
  }
  return {
    save: currentSave(value),
    migrated: originalVersion !== SAVE_VERSION,
  };
}
