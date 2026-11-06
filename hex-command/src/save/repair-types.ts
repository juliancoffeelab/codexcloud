import type { BattleSave } from "./schema";
import type { CurrentEntry, TurnEntry } from "./metadata";

export interface ImportFailure {
  source: "localStorage" | "indexedDB";
  detail: string;
}
export interface RepairRecord {
  key: string;
  kind: "flash" | "turn" | "index" | "other";
  label: string;
  runId?: string;
  runName?: string;
  entry?: CurrentEntry | TurnEntry;
  error?: string;
  /** Converted data is temporary repair input, never a gameplay repository. */
  save?: BattleSave;
}
export interface RepairInventory {
  records: RepairRecord[];
  errors: string[];
  authoritative: boolean;
}
export type RepairTarget =
  | { kind: "record"; key: string }
  | { kind: "campaign"; runId: string }
  | { kind: "all" }
  | { kind: "index" };
