import { migrateV38ToV39 } from "./v38-to-v39";
import { migrateV37ToV38 } from "./v37-to-v38";
import { migrateV36ToV37 } from "./v36-to-v37";
import { migrateV35ToV36 } from "./v35-to-v36";
import { migrateV34ToV35 } from "./v34-to-v35";
import { migrateV33ToV34 } from "./v33-to-v34";
import { migrateV32ToV33 } from "./v32-to-v33";
import { migrateV31ToV32 } from "./v31-to-v32";
import { migrateV30ToV31 } from "./v30-to-v31";
import { migrateV29ToV30 } from "./v29-to-v30";
import { migrateV28ToV29 } from "./v28-to-v29";
import { migrateV27ToV28 } from "./v27-to-v28";
import { migrateV26ToV27 } from "./v26-to-v27";
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
  const originalVersion = value.version;
  // Route the original native envelope directly through its explicit v1→v2 step.
  if (value.version === undefined && value.engine === "rust-wasm") value = migrateV1ToV2(value);
  if (
    !Number.isSafeInteger(value.version) ||
    Number(value.version) < 1 ||
    Number(value.version) > SAVE_VERSION
  )
    throw new Error(`Unsupported save version ${String(value.version)}`);
  while (value.version !== SAVE_VERSION) {
    switch (value.version) {
      case 38:
        value = migrateV38ToV39(value);
        break;
      case 37:
        value = migrateV37ToV38(value);
        break;
      case 36:
        value = migrateV36ToV37(value);
        break;
      case 35:
        value = migrateV35ToV36(value);
        break;
      case 34:
        value = migrateV34ToV35(value);
        break;
      case 33:
        value = migrateV33ToV34(value);
        break;
      case 32:
        value = migrateV32ToV33(value);
        break;
      case 31:
        value = migrateV31ToV32(value);
        break;
      case 30:
        value = migrateV30ToV31(value);
        break;
      case 29:
        value = migrateV29ToV30(value);
        break;
      case 28:
        value = migrateV28ToV29(value);
        break;
      case 27:
        value = migrateV27ToV28(value);
        break;
      case 26:
        value = migrateV26ToV27(value);
        break;
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
