import type { Save } from "./formats/history";
import { readSave } from "./formats/read";
import { isRecord } from "./formats/envelope-v39";

// Published v11 valley starters, not recalled veterans. Keep any that have acted,
// earned experience, taken damage or received equipment in an ongoing battle.
const extras = [
  { id: "u3", type: "crossbowman", tile: "1,15", hp: 24 },
  { id: "u4", type: "spearman", tile: "3,15", hp: 25 },
];
const discardedOpening = new Set([
  "The tracks divide here. Some went north; the rest kept along the low road.",
  "We follow the living. Keep a watch on the high ground.",
  "Three groups, taken to the old burial grounds. We heard the bell before the screaming stopped. The north path climbs to the monastery; the low road leads toward the keep.",
  "Keep your lamps burning. We’ll find them.",
]);
const lead = {
  speaker: "Oren",
  portrait: "adept",
  text: "The monastery is on the hill to the north. Get me there, and I can use its wards to clear this fog.",
};

/** Correct the published valley opening while preserving recalled and active troops. */
export function migrateV11ToV12(input: unknown): Save<12> {
  const value = readSave(input, 11);
  if (value.version !== 11 || !isRecord(value.checkpoint) || !isRecord(value.checkpoint.state))
    throw new Error("Invalid version-eleven save");
  const old = value.checkpoint.state;
  if (old.mapId !== "lantern-bell" || old.mapRevision !== 3) return { ...value, version: 12 };
  if (
    !Array.isArray(old.units) ||
    !old.units.every(isRecord) ||
    !isRecord(old.campaign) ||
    !Array.isArray(old.campaign.dialogue) ||
    !old.campaign.dialogue.every(isRecord) ||
    !Array.isArray(old.campaign.marks) ||
    !Number.isInteger(old.campaign.dialogueIndex) ||
    Number(old.campaign.dialogueIndex) < 0 ||
    Number(old.campaign.dialogueIndex) > old.campaign.dialogue.length
  )
    throw new Error("Invalid historical valley opening");
  const units = old.units.filter(
    (unit) =>
      !(
        old.round === 1 &&
        old.winner === null &&
        unit.side === "player" &&
        !unit.name &&
        !unit.role &&
        (unit.experience === undefined || unit.experience === 0) &&
        !unit.moved &&
        !unit.attacked &&
        (unit.items === undefined || (Array.isArray(unit.items) && unit.items.length === 0)) &&
        extras.some(
          (extra) =>
            unit.id === extra.id &&
            unit.type === extra.type &&
            unit.tile === extra.tile &&
            unit.hp === extra.hp,
        )
      ),
  );
  const dialogue = old.campaign.dialogue.filter((line) => !discardedOpening.has(String(line.text)));
  const dialogueIndex = old.campaign.dialogue
    .slice(0, Number(old.campaign.dialogueIndex))
    .filter((line) => !discardedOpening.has(String(line.text))).length;
  const marks = [...old.campaign.marks];
  if (
    old.winner === null &&
    old.turn === "player" &&
    !value.pendingEnemy &&
    units.some((unit) => unit.side === "player" && unit.name === "Oren") &&
    !marks.includes("event:monastery-start") &&
    !marks.includes("event:monastery-lead")
  ) {
    dialogue.push({ ...lead });
    marks.push("event:monastery-lead");
  }
  return {
    ...value,
    version: 12,
    checkpoint: {
      ...value.checkpoint,
      state: { ...old, units, campaign: { ...old.campaign, dialogue, dialogueIndex, marks } },
    },
  };
}
