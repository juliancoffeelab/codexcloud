import { migrateSave as convert } from "@hex-command/save-migrations";
import { currentSave, type BattleSave } from "../schema";
export interface MigrationResult {
  save: BattleSave;
  migrated: boolean;
}
export function migrateSave(input: unknown): MigrationResult {
  const result = convert(input);
  return { ...result, save: currentSave(result.save) };
}

// Rust's generated declarations use Map for JSON dictionaries. Compare the wire
// shapes here; historical definitions remain independent of today's application.
// Optional Rust Option fields may serialize as null rather than be omitted.
type Wire<T> =
  T extends Map<infer K, infer V>
    ? Record<Extract<K, string>, Wire<V>>
    : T extends object
      ? { [K in keyof T]: Wire<undefined extends T[K] ? Exclude<T[K], null> : T[K]> }
      : T;
type Equal<A, B> =
  (<T>() => T extends A ? 1 : 2) extends <T>() => T extends B ? 1 : 2 ? true : false;
type Assert<T extends true> = T;
type Output = ReturnType<typeof convert>["save"];
export type MigrationContractMatches = Assert<Equal<Wire<Output>, Wire<BattleSave>>>;
