/** Version 32 replaces repeated full-world undo snapshots with compact backward changes. */
export function migrateV31ToV32(value: Record<string, unknown>): Record<string, unknown> {
  const { undo: _discardedUndo, ...save } = value;
  return { ...save, version: 32 };
}
