import type { Save } from "./formats/history";
import { readSave } from "./formats/read";
import { isRecord } from "./formats/envelope-v39";
import frozen from "./legacy/data/flags-v28.json";

const origins = frozen as Record<string, Record<string, number>>;
/** Flat flags did not record write times. Preserve values and known producing stages. */
export function migrateV28ToV29(input: unknown): Save<29> {
  const value = readSave(input, 28);
  if (value.version !== 28 || !isRecord(value.checkpoint) || !isRecord(value.checkpoint.state))
    throw new Error("Invalid version-twenty-eight save");
  const state = value.checkpoint.state;
  if (state.campaign === undefined || state.campaign === null)
    return {
      ...value,
      version: 29,
      checkpoint: { ...value.checkpoint, state: { ...state, campaign: undefined } },
    };
  if (!isRecord(state.campaign)) throw new Error("Invalid historical campaign");
  const { choices, ...campaign } = state.campaign;
  const stage = campaign.mission;
  if (
    !Number.isSafeInteger(stage) ||
    Number(stage) < 0 ||
    !Number.isSafeInteger(state.round) ||
    Number(state.round) < 1 ||
    (choices !== undefined &&
      choices !== null &&
      (!Array.isArray(choices) || choices.some((v) => typeof v !== "string")))
  )
    throw new Error("Invalid historical flags");
  const flags: Record<string, { state: boolean; stage: number; turn: number }[]> = {};
  for (const id of (choices as string[]) ?? []) {
    const source = origins[String(campaign.id)]?.[id];
    // Unknown flags retain their value as incoming facts. The old save has no provenance.
    const produced = source !== undefined && source <= Number(stage) ? source : Number(stage);
    Object.defineProperty(flags, id, {
      value: [
        { state: true, stage: produced, turn: source === Number(stage) ? Number(state.round) : 0 },
      ],
      enumerable: true,
      writable: true,
      configurable: true,
    });
  }
  return {
    ...value,
    version: 29,
    checkpoint: { ...value.checkpoint, state: { ...state, campaign: { ...campaign, flags } } },
  };
}
