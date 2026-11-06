import type { BattleSave } from "../schema";
import type { RepairRecord } from "../repair-types";
type SourceRecord = RepairRecord & { save?: BattleSave };
import { migrateSave } from "./index";
import { migrateHistoryIndex } from "./history";
import { currentFlashSave, currentTurn, isRecord } from "../schema";

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
function importedSave(raw: string) {
  let value: unknown = JSON.parse(raw);
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

export function migrateStorageV1ToV2(source: LocalSource) {
  const raw = source[SAVE_KEY];
  const current =
    raw !== undefined && raw !== null ? currentFlashSave(importedSave(raw)) : undefined;
  const text = source[HISTORY_KEY];
  const history =
    text !== undefined && text !== null ? migrateHistoryIndex(JSON.parse(text)) : undefined;
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

/** Repair reads each record independently; no complete conversion is required for deletion. */
export function inspectStorageV1ToV2(source: LocalSource): SourceRecord[] {
  const records: SourceRecord[] = [];
  const metadata = new Map<string, import("../metadata").TurnEntry>();
  const index = source[HISTORY_KEY];
  if (index !== undefined && index !== null) {
    const record: SourceRecord = {
      key: HISTORY_KEY,
      kind: "index",
      label: "History list",
    };
    try {
      const value: unknown = JSON.parse(index);
      if (!isRecord(value) || !Array.isArray(value.entries))
        throw new Error("History list is unreadable.");
      // A broken row must not hide other readable rows.
      for (const entry of value.entries) {
        try {
          const converted = migrateHistoryIndex({ version: value.version, entries: [entry] }).index
            .entries;
          for (const item of converted) metadata.set(item.id, item);
        } catch {
          record.error =
            "Some history labels are unreadable. Rebuild the history list after removing damaged records.";
        }
      }
      if (!record.error) migrateHistoryIndex(value);
    } catch (error) {
      record.error = String(error);
    }
    records.push(record);
  }
  if (!index && Object.keys(source).some((key) => key.startsWith("hex-command-progress:")))
    records.push({
      key: HISTORY_KEY,
      kind: "index",
      label: "Missing history list",
      error: "Rebuild the history list to retain identifiable snapshots.",
    });
  const keys = new Set(
    Object.keys(source).filter(
      (key) =>
        (key === SAVE_KEY && source[key] !== null) || key.startsWith("hex-command-progress:"),
    ),
  );
  for (const id of metadata.keys()) keys.add(historyPayloadKey(id));
  for (const key of keys) {
    const isFlash = key === SAVE_KEY;
    const id = key.slice("hex-command-progress:".length);
    const hint = isFlash ? undefined : metadata.get(id);
    const record: SourceRecord = {
      key,
      kind: isFlash ? "flash" : "turn",
      label: isFlash ? "Flash save" : hint ? `Turn ${hint.round}` : `Unidentified save · ${id}`,
      entry: hint,
      runId: hint?.runId,
      runName: hint?.run,
    };
    const raw = source[key];
    try {
      if (raw === undefined || raw === null)
        throw new Error("The snapshot is missing; only its history label remains.");
      const save = importedSave(raw);
      if (
        isFlash
          ? save.progress.entry.kind !== "flash"
          : save.progress.entry.kind !== "turn" || save.progress.entry.id !== id
      )
        throw new Error("Save identity does not match its storage record.");
      const entry = save.progress.entry;
      if (hint && hint.runId !== entry.runId) {
        record.runId = undefined;
        record.runName = undefined;
        record.entry = undefined;
        throw new Error("The history list and snapshot disagree about campaign ownership.");
      }
      record.save = save;
      record.entry = entry;
      record.runId = entry.runId;
      record.runName = entry.run;
      if (!isFlash) record.label = `Turn ${entry.round}`;
    } catch (error) {
      record.error = String(error);
    }
    records.push(record);
  }
  for (const key of Object.keys(source))
    if (key === `${SAVE_KEY}-unrestored`)
      records.push({
        key,
        kind: "other",
        label: "Unidentified retained save",
        error: "This extra record is not part of the current import.",
      });
  return records;
}

/** Repair edits only explicit source records and their references; payload validity is irrelevant. */
export function repairStorageV1ToV2(
  source: LocalSource,
  target: import("../repair-types").RepairTarget,
) {
  const records = inspectStorageV1ToV2(source);
  const selected =
    target.kind === "all"
      ? Object.keys(source)
      : target.kind === "record"
        ? [target.key]
        : target.kind === "campaign"
          ? records.filter((r) => r.runId === target.runId).map((r) => r.key)
          : [];
  if (selected.some((key) => !sourceKey(key)))
    throw new Error("This is not a save storage record.");
  const deleted = new Set(selected);
  const writes: LocalSource = {};
  if (target.kind === "all" || deleted.has(SAVE_KEY)) writes[NEW_GAME_KEY] = "true";
  if (target.kind === "index") {
    const entries = records
      .filter((r) => r.kind === "turn" && r.save?.progress.entry.kind === "turn")
      .map((r) => r.entry!);
    writes[HISTORY_KEY] = JSON.stringify({ version: 4, entries });
  } else if (target.kind !== "all" && !deleted.has(HISTORY_KEY) && source[HISTORY_KEY]) {
    try {
      const index: unknown = JSON.parse(source[HISTORY_KEY]!);
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
