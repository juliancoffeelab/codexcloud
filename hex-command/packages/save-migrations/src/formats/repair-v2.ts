/** Frozen storage-v2 repair data. */
import type { CurrentEntry, TurnEntry } from "./metadata-v39";

export interface LoadProblem {
  source: "localStorage" | "indexedDB";
  kind: "data" | "full" | "blocked" | "read" | "ready";
  detail: string;
}
export type RepairIssue =
  | "unreadable"
  | "format"
  | "missing_snapshot"
  | "identity_conflict"
  | "ownership_conflict"
  | "invalid_battle"
  | "index_missing"
  | "index_invalid"
  | "unlisted_snapshots";
interface RecordInfo {
  key: string;
  kind: "flash" | "turn" | "index" | "other";
  label: string;
  issue?: RepairIssue;
  error?: string;
  unlistedCount?: number;
}
/** Conflicting claims cannot simultaneously be presented as known ownership. */
export type RepairRecord = RecordInfo &
  (
    | {
        ownership: "known";
        runId: string;
        runName: string;
        entry: CurrentEntry | TurnEntry;
        claims?: never;
      }
    | { ownership: "unknown"; runId?: never; runName?: never; entry?: never; claims?: never }
    | {
        ownership: "conflicting";
        claims: (CurrentEntry | TurnEntry)[];
        runId?: never;
        runName?: never;
        entry?: never;
      }
  );
export interface RepairInventory {
  records: RepairRecord[];
  errors: string[];
  authoritative: boolean;
  completedElsewhere: boolean;
}
export type RepairTarget =
  | { kind: "record"; key: string }
  | { kind: "campaign"; runId: string }
  | { kind: "all" }
  | { kind: "index" };
