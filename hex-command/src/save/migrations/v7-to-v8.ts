import { isRecord } from "../schema";

/** Version seven had inventory, but no items on the battlefield. */
export function migrateV7ToV8(value: Record<string, unknown>): Record<string, unknown> {
  if (value.version !== 7 || !isRecord(value.checkpoint) || !isRecord(value.checkpoint.state))
    throw new Error("Invalid version-seven save");
  return {
    ...value,
    version: 8,
    checkpoint: { ...value.checkpoint, state: { ...value.checkpoint.state, groundItems: [] } },
  };
}
