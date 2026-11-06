import type { LoadProblem, RepairInventory, RepairRecord, RepairIssue } from "../save/repair-types";
import { runName, saveTime, turnName, saveOrder } from "./save-browser";

export function recordName(record: RepairRecord): string {
  if (record.kind === "flash")
    return record.ownership === "known"
      ? `Flash save · ${runName(record.entry)}`
      : "Unidentified flash save";
  if (record.kind === "index") return "Save list";
  if (record.entry?.kind === "turn") return turnName(record.entry);
  return record.kind === "turn" ? "Unidentified turn save" : "Unidentified save";
}
export function issueText(record: RepairRecord): string {
  const messages: Record<RepairIssue, string> = {
    unreadable: "Couldn’t read this save",
    format: "This save’s format isn’t recognized",
    missing_snapshot: "Snapshot missing",
    identity_conflict: "The snapshot doesn’t match its save entry",
    ownership_conflict: "Conflicting campaign information",
    invalid_battle: "The saved battle failed validation",
    index_missing: "The save list is missing",
    index_invalid: "The save list is damaged",
    unlisted_snapshots: "Turns are missing from the save list",
  };
  return record.issue ? messages[record.issue] : "";
}
export function indexMessage(record: RepairRecord): string {
  if (record.unlistedCount)
    return `${record.unlistedCount} identifiable ${record.unlistedCount === 1 ? "turn is" : "turns are"} missing from the list.`;
  return record.issue === "index_missing"
    ? "Rebuild it from identifiable snapshots."
    : "Rebuilding retains identifiable snapshots. Unresolved records stay available for cleanup.";
}
export function repairModel(inventory: RepairInventory, problem: LoadProblem | null | undefined) {
  const flash = inventory.records.find((record) => record.kind === "flash") ?? null;
  const attention = inventory.records
    .filter((record) => record.issue)
    .sort(
      (a, b) =>
        Number(b.kind === "flash") - Number(a.kind === "flash") ||
        Number(b.kind === "index") - Number(a.kind === "index"),
    );
  const groups = new Map<string, { id: string; name: string; records: RepairRecord[] }>();
  for (const record of inventory.records)
    if (record.ownership === "known" && record.kind !== "index") {
      const group = groups.get(record.runId) ?? {
        id: record.runId,
        name: runName(record.entry),
        records: [],
      };
      group.records.push(record);
      groups.set(record.runId, group);
    }
  const campaigns = [...groups.values()];
  for (const group of campaigns)
    group.records.sort((a, b) => (a.entry && b.entry ? saveOrder(a.entry, b.entry) : 0));
  const readable = inventory.records.filter((record) => record.kind !== "index" && !record.issue);
  const canContinue = inventory.errors.length === 0 && attention.length === 0;
  const ready = inventory.authoritative && canContinue;
  let summary = "Review the saved games below.";
  if (inventory.errors.length)
    summary = "Saved data couldn’t be checked completely. You can review what was readable below.";
  else if (ready)
    summary = inventory.completedElsewhere
      ? "Another tab finished updating your saves."
      : "Your saved games can now be read.";
  else if (problem?.kind === "full")
    summary =
      "There isn’t enough space to finish updating saved games. Delete saves you no longer need, then finish the update.";
  else if (flash && flash.ownership !== "known")
    summary = "We can’t identify the flash save’s campaign.";
  else if (attention.some((record) => record.kind === "index"))
    summary = "The save list needs attention.";
  else if (attention.length === 1 && attention[0]?.issue === "missing_snapshot")
    summary = "One turn is listed, but its snapshot is missing.";
  else if (attention.length === 1 && attention[0]?.issue === "ownership_conflict")
    summary = "One turn has conflicting campaign information.";
  else if (attention.length === 1 && attention[0]?.kind === "turn")
    summary = "One turn save can’t be restored.";
  else if (attention.length) summary = `${attention.length} saved records need attention.`;
  else if (!inventory.records.length) summary = "No retained saves remain.";
  else summary = "The remaining saved games are ready to update.";
  if (!ready && flash && !flash.issue && problem?.kind !== "full" && !inventory.errors.length)
    summary += " Your flash save is intact.";
  const continuation = flash
    ? ready
      ? "Continue from flash save"
      : problem?.kind === "full"
        ? "Finish update"
        : "Restore saved game"
    : "Finish and choose new game";
  const hint = canContinue
    ? ""
    : flash && flash.ownership !== "known"
      ? "Delete or resolve the unidentified flash save to continue."
      : "Resolve the items above to continue.";
  return { flash, attention, campaigns, readable, canContinue, ready, summary, continuation, hint };
}
export interface RepairConfirmation {
  title: string;
  subject?: string;
  copy: string;
  notice?: string;
  warning?: boolean;
  action: string;
  destructive: boolean;
}
export function campaignConfirmation(
  inventory: RepairInventory,
  runId: string,
): RepairConfirmation {
  const records = inventory.records.filter(
    (record) => record.ownership === "known" && record.runId === runId,
  );
  const entry = records.find((record) => record.entry)?.entry;
  const name = entry ? runName(entry) : "Selected campaign";
  const turns = records.filter((record) => record.kind === "turn").length;
  const flash = inventory.records.find((record) => record.kind === "flash");
  const includesFlash = flash?.ownership === "known" && flash.runId === runId;
  const count = turns
    ? `${turns} turn ${turns === 1 ? "save" : "saves"}${includesFlash ? " and its flash save" : ""} will be deleted.`
    : includesFlash
      ? "The flash save will be deleted."
      : "No identified saves remain for this campaign.";
  const unknown = flash && flash.ownership !== "known";
  const other = flash?.ownership === "known" && flash.runId !== runId;
  return {
    title: "Delete campaign saves?",
    subject: name,
    copy: count,
    notice: unknown
      ? "The unidentified flash save will be kept. We strongly recommend deleting it separately."
      : other
        ? `The flash save for ${runName(flash.entry)} will be kept.`
        : undefined,
    warning: !!unknown,
    action: "Delete campaign saves",
    destructive: true,
  };
}
export function recordConfirmation(record: RepairRecord): RepairConfirmation {
  const missing = record.issue === "missing_snapshot";
  const title = missing
    ? "Remove missing turn from list?"
    : record.kind === "flash"
      ? "Delete flash save?"
      : record.kind === "turn"
        ? "Delete turn save?"
        : "Delete retained save?";
  return {
    title,
    subject: record.entry
      ? `${runName(record.entry)} · ${recordName(record)} · ${saveTime(record.entry.savedAt)}`
      : recordName(record),
    copy: missing
      ? "Only its list entry remains. It will be removed."
      : "This save will be deleted. Other saves will remain.",
    action: missing
      ? "Remove entry"
      : record.kind === "flash"
        ? "Delete flash save"
        : record.kind === "turn"
          ? "Delete turn save"
          : "Delete retained save",
    destructive: true,
  };
}
