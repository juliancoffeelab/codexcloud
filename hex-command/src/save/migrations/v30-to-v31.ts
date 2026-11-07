/** Version 31 adds optional local progress identity and move undo. Old saves need neither. */
export function migrateV30ToV31(value: Record<string, unknown>): Record<string, unknown> {
  return { ...value, version: 31 };
}
