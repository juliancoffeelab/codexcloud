/** Version 36 adds explicitly requested manual checkpoints; existing facts are unchanged. */
export function migrateV35ToV36(value: Record<string, unknown>): Record<string, unknown> {
  if (value.version !== 35) throw new Error("Invalid version-thirty-five save");
  return { ...value, version: 36 };
}
