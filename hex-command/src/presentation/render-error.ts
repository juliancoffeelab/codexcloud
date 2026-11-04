/** Preserve worker exceptions as text; Error objects lose useful fields in messages. */
export function renderErrorReport(label: string, error: unknown, context?: unknown): string {
  const seen = new Set<unknown>();
  const describe = (value: unknown, depth = 0): string => {
    if (seen.has(value)) return "[circular cause]";
    if (depth > 8) return "[cause depth limit]";
    if (value instanceof Error) {
      seen.add(value);
      return (
        (value.stack || `${value.name}: ${value.message}`) +
        (value.cause === undefined ? "" : `\nCaused by: ${describe(value.cause, depth + 1)}`)
      );
    }
    return String(value);
  };
  let details = "";
  try {
    details =
      context === undefined ? "" : `\n\nRenderer context:\n${JSON.stringify(context, null, 2)}`;
  } catch {
    details = "\nRenderer context unavailable";
  }
  return `${label}: ${error instanceof Error ? error.message : String(error)}\n\n${describe(error)}${details}`;
}
