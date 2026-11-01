import { isRecord } from "../schema";

// Frozen correction for the published v12 church reward. Do not import current content.
const mail = {
  name: "Enchanted mail",
  description:
    "Silver wards are worked into the links. Incoming blade and piercing damage reduced by 1 per strike. Drops when its carrier dies.",
};
const oldDiscovery =
  "The vestry has been stripped of food, but two pieces of armor remain beneath a bench.";
const discovery =
  "A shirt of mail beneath the bench. No rust, and silver runes worked into the links.";

function items(value: unknown): unknown[] {
  if (!Array.isArray(value) || !value.every(isRecord))
    throw new Error("Invalid historical equipment");
  return value
    .filter((item) => item.id !== "church-helm")
    .map((item) => (item.id === "church-mail" ? { ...item, ...mail } : item));
}
function carrier(value: unknown): Record<string, unknown> {
  if (!isRecord(value)) throw new Error("Invalid historical item carrier");
  return value.items === undefined ? { ...value } : { ...value, items: items(value.items) };
}

/** Remove the unintended helmet everywhere it could have been carried or dropped. */
export function migrateV12ToV13(value: Record<string, unknown>): Record<string, unknown> {
  if (value.version !== 12 || !isRecord(value.checkpoint) || !isRecord(value.checkpoint.state))
    throw new Error("Invalid version-twelve save");
  const old = value.checkpoint.state;
  const campaign = old.campaign;
  if (old.mapId !== "lantern-bell" && (!isRecord(campaign) || campaign.id !== "lantern-road"))
    return { ...value, version: 13 };
  if (
    !Array.isArray(old.units) ||
    !Array.isArray(old.groundItems) ||
    !old.groundItems.every(isRecord)
  )
    throw new Error("Invalid historical church inventory");
  const groundItems = old.groundItems
    .filter((ground) => {
      if (!isRecord(ground.item)) throw new Error("Invalid historical ground item");
      return ground.item.id !== "church-helm";
    })
    .map((ground) => ({ ...ground, item: items([ground.item])[0] }));
  const progress = isRecord(campaign) ? { ...campaign } : campaign;
  if (isRecord(progress)) {
    for (const key of ["veterans", "storyVeterans"])
      if (progress[key] !== undefined) {
        if (!Array.isArray(progress[key])) throw new Error("Invalid historical veterans");
        progress[key] = progress[key].map(carrier);
      }
    if (progress.evacuated !== undefined) {
      if (!Array.isArray(progress.evacuated) || !progress.evacuated.every(isRecord))
        throw new Error("Invalid historical evacuees");
      progress.evacuated = progress.evacuated.map((entry) => ({
        ...entry,
        unit: carrier(entry.unit),
      }));
    }
    if (Array.isArray(progress.dialogue))
      progress.dialogue = progress.dialogue.map((line) =>
        isRecord(line) && line.text === oldDiscovery ? { ...line, text: discovery } : line,
      );
  }
  return {
    ...value,
    version: 13,
    checkpoint: {
      ...value.checkpoint,
      state: {
        ...old,
        units: old.units.map(carrier),
        groundItems,
        ...(progress !== undefined ? { campaign: progress } : {}),
      },
    },
  };
}
