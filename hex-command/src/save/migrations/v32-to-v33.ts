/** Version 33 distinguishes an adopted saved position from a historical turn start. */
export function migrateV32ToV33(value: Record<string, unknown>): Record<string, unknown> {
  return { ...value, version: 33 };
}
