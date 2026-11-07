import type { BattleSave } from "./formats/envelope-v39";
import type { RepairRecord } from "./formats/repair-v2";
type SourceRecord = RepairRecord & { save?: BattleSave };
import { migrateSave } from "./index";
import { migrateHistoryIndex } from "./history";
import { currentFlashSave, currentTurn, isRecord } from "./formats/envelope-v39";

/** Storage v1: localStorage's envelope and index. Storage v2: transactional IDB records. */
export const SAVE_KEY = "hex-command-battle";
export const HISTORY_KEY = "hex-command-progress-index";
export const NEW_GAME_KEY = "hex-command-choose-new-game";
export const historyPayloadKey = (id: string) => `hex-command-progress:${id}`;
export const sourceKey = (key: string) =>
  [SAVE_KEY, HISTORY_KEY, NEW_GAME_KEY, `${SAVE_KEY}-unrestored`].includes(key) ||
  key.startsWith("hex-command-progress:");
export type LocalSource = Record<string, string | null>;

/** Version 37 was deployed in localStorage with the former current-save label. */
function importedValue(input: unknown) {
  let value = input;
  if (
    isRecord(value) &&
    value.version === 37 &&
    isRecord(value.progress) &&
    isRecord(value.progress.entry) &&
    value.progress.entry.kind === "recovery"
  ) {
    value = {
      ...value,
      progress: { ...value.progress, entry: { ...value.progress.entry, kind: "flash" } },
    };
  }
  return migrateSave(value).save;
}

function importedSave(raw: string) {
  return importedValue(JSON.parse(raw));
}

export function migrateStorageV1ToV2(source: LocalSource) {
  const raw: string | null = source[SAVE_KEY] ?? null;
  const current = raw === null ? undefined : currentFlashSave(importedSave(raw));
  const indexText: string | null = source[HISTORY_KEY] ?? null;
  const history = indexText === null ? undefined : migrateHistoryIndex(JSON.parse(indexText));
  const entries = history?.index.entries ?? [];
  const indexed = new Set(entries.map((entry) => historyPayloadKey(entry.id)));
  const retired = new Set(history?.removedIds.map(historyPayloadKey) ?? []);
  for (const key of Object.keys(source)) {
    if (
      (key.startsWith("hex-command-progress:") && !indexed.has(key) && !retired.has(key)) ||
      key === `${SAVE_KEY}-unrestored`
    )
      throw new Error("Unlisted saved data needs repair before importing.");
  }
  const turns = entries.map((entry) => {
    const payload = source[historyPayloadKey(entry.id)];
    if (payload === null || payload === undefined)
      throw new Error(`Missing saved turn ${entry.id}.`);
    const save = currentTurn(importedSave(payload));
    if (save.progress.entry.id !== entry.id || save.progress.entry.runId !== entry.runId)
      throw new Error(`Saved turn metadata does not match ${entry.id}.`);
    return save;
  });
  return { current, turns, chooseNewGame: source[NEW_GAME_KEY] === "true" };
}

/** Repair keeps every ownership claim; row order cannot establish truth. */
export function inspectStorageV1ToV2(source: LocalSource): SourceRecord[] {
  const records: SourceRecord[] = [];
  const metadata = new Map<string, import("./formats/metadata-v39").TurnEntry[]>();
  const retired = new Set<string>();
  const indexText: string | null = source[HISTORY_KEY] ?? null;
  let indexRecord: SourceRecord | undefined;
  if (indexText !== null) {
    indexRecord = { key: HISTORY_KEY, kind: "index", label: "Save list", ownership: "unknown" };
    try {
      const value: unknown = JSON.parse(indexText);
      if (!isRecord(value) || !Array.isArray(value.entries))
        throw new Error("Save list is unreadable.");
      for (const row of value.entries) {
        try {
          const result = migrateHistoryIndex({ version: value.version, entries: [row] });
          for (const id of result.removedIds) retired.add(id);
          for (const entry of result.index.entries) {
            const claims = metadata.get(entry.id) ?? [];
            claims.push(entry);
            metadata.set(entry.id, claims);
          }
        } catch (error) {
          indexRecord.issue = "index_invalid";
          indexRecord.error = String(error);
        }
      }
      const full = migrateHistoryIndex(value);
      for (const id of full.removedIds) retired.add(id);
    } catch (error) {
      indexRecord.issue = "index_invalid";
      indexRecord.error = String(error);
    }
    records.push(indexRecord);
  } else if (Object.keys(source).some((key) => key.startsWith("hex-command-progress:"))) {
    indexRecord = {
      key: HISTORY_KEY,
      kind: "index",
      label: "Save list",
      ownership: "unknown",
      issue: "index_missing",
      error: "No history index is stored.",
    };
    records.push(indexRecord);
  }
  const keys = new Set(
    Object.keys(source).filter(
      (key) =>
        (key === SAVE_KEY && source[key] !== null) || key.startsWith("hex-command-progress:"),
    ),
  );
  for (const id of metadata.keys()) keys.add(historyPayloadKey(id));
  let unlisted = 0;
  for (const key of keys) {
    const isFlash = key === SAVE_KEY;
    const id = key.slice("hex-command-progress:".length);
    if (!isFlash && retired.has(id)) continue;
    const claims = isFlash ? [] : (metadata.get(id) ?? []);
    const conflict = new Set(claims.map((entry) => entry.runId)).size > 1;
    const hint = conflict ? undefined : claims[0];
    const base = {
      key,
      kind: isFlash ? ("flash" as const) : ("turn" as const),
      label: isFlash ? "Flash save" : hint ? `Turn ${hint.round}` : "Unidentified turn save",
    };
    let record: SourceRecord = conflict
      ? {
          ...base,
          ownership: "conflicting",
          claims,
          issue: "ownership_conflict",
          error: "Duplicate index entries assign this snapshot to different campaigns.",
        }
      : hint
        ? { ...base, ownership: "known", runId: hint.runId, runName: hint.run, entry: hint }
        : { ...base, ownership: "unknown" };
    let stage: "parse" | "convert" | "identity" = "parse";
    try {
      const raw = source[key];
      if (raw === null || raw === undefined) {
        if (!conflict) record.issue = "missing_snapshot";
        throw new Error("Snapshot missing; only its list entry remains.");
      }
      const value: unknown = JSON.parse(raw);
      stage = "convert";
      const save = importedValue(value);
      stage = "identity";
      const entry = save.progress.entry;
      // Ownership evidence survives a wrong storage key or snapshot kind.
      // Rejecting identity first would leave the index's owner falsely trusted.
      const ownershipClaims = claims.some(
        (claim) => claim.runId === entry.runId && claim.id === entry.id,
      )
        ? claims
        : [...claims, entry];
      if (new Set(ownershipClaims.map((claim) => claim.runId)).size > 1) {
        record = {
          ...base,
          ownership: "conflicting",
          claims: ownershipClaims,
          issue: "ownership_conflict",
          error: "The snapshot and save list disagree about campaign ownership.",
        };
      }
      if (isFlash ? entry.kind !== "flash" : entry.kind !== "turn" || entry.id !== id)
        throw new Error("Snapshot identity does not match its storage key.");
      if (record.ownership === "conflicting") {
        record.save = save;
      } else {
        record = {
          ...base,
          label: isFlash ? "Flash save" : `Turn ${entry.round}`,
          ownership: "known",
          entry,
          runId: entry.runId,
          runName: entry.run,
          save,
        };
        if (!isFlash && !metadata.has(id)) unlisted++;
      }
    } catch (error) {
      if (!record.issue)
        record.issue =
          stage === "parse" ? "unreadable" : stage === "convert" ? "format" : "identity_conflict";
      record.error = [record.error, String(error)]
        .filter((value) => value !== undefined)
        .join("\n");
    }
    records.push(record);
  }
  if (indexRecord && unlisted > 0) {
    indexRecord.unlistedCount = unlisted;
    if (!indexRecord.issue) {
      indexRecord.issue = "unlisted_snapshots";
      indexRecord.error = "Identifiable snapshots are absent from the save list.";
    }
  }
  for (const key of Object.keys(source))
    if (key === `${SAVE_KEY}-unrestored`)
      records.push({
        key,
        kind: "other",
        label: "Unidentified retained save",
        ownership: "unknown",
        issue: "format",
        error: "This extra record is not part of the current import.",
      });
  return records;
}

/** Repair edits only explicit source records and their references; payload validity is irrelevant. */
export function repairStorageV1ToV2(
  source: LocalSource,
  target: import("./formats/repair-v2").RepairTarget,
) {
  const records = inspectStorageV1ToV2(source);
  const selected =
    target.kind === "all"
      ? Object.keys(source)
      : target.kind === "record"
        ? [target.key]
        : target.kind === "campaign"
          ? records
              .filter((r) => r.ownership === "known" && r.runId === target.runId)
              .map((r) => r.key)
          : [];
  if (selected.some((key) => !sourceKey(key)))
    throw new Error("This is not a save storage record.");
  const deleted = new Set(selected);
  const writes: LocalSource = {};
  const indexText: string | null = source[HISTORY_KEY] ?? null;
  if (target.kind === "all" || deleted.has(SAVE_KEY)) writes[NEW_GAME_KEY] = "true";
  if (target.kind === "index") {
    const entries = records
      .filter(
        (r) =>
          r.kind === "turn" && r.ownership === "known" && r.save?.progress.entry.kind === "turn",
      )
      .map((r) => r.entry!);
    // Preserve unresolved claims rather than laundering them into known ownership.
    for (const record of records)
      if (record.ownership === "conflicting")
        for (const claim of record.claims)
          if (claim.kind === "turn" && historyPayloadKey(claim.id) === record.key)
            entries.push(claim);
    writes[HISTORY_KEY] = JSON.stringify({ version: 4, entries });
  } else if (target.kind !== "all" && !deleted.has(HISTORY_KEY) && indexText !== null) {
    try {
      const index: unknown = JSON.parse(indexText);
      if (isRecord(index) && Array.isArray(index.entries)) {
        const entries = index.entries.filter(
          (entry) =>
            !isRecord(entry) ||
            typeof entry.id !== "string" ||
            !deleted.has(historyPayloadKey(entry.id)),
        );
        if (entries.length !== index.entries.length)
          writes[HISTORY_KEY] = JSON.stringify({ ...index, entries });
      }
    } catch {
      /* The broken index remains visible for explicit rebuilding or reset. */
    }
  }
  return { deleted: [...deleted], writes };
}
