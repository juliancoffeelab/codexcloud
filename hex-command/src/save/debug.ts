import { SAVE_KEY } from "./storage";

interface StorageDiagnosticError {
  operation: "access" | "length" | "key" | "read";
  key?: string;
  index?: number;
  detail: string;
}
export interface BrowserSaveDiagnostics {
  /** Exact stored strings; null means a successful read found no value. Failed reads are omitted. */
  values: Record<string, string | null>;
  errors: StorageDiagnosticError[];
}

/** Read only game-owned persistence, independently of parsing, migration, or index validity. */
export function captureBrowserSaves(): BrowserSaveDiagnostics {
  const report: BrowserSaveDiagnostics = { values: {}, errors: [] };
  const error = (
    operation: StorageDiagnosticError["operation"],
    reason: unknown,
    context: { key?: string; index?: number } = {},
  ) =>
    report.errors.push({
      operation,
      ...context,
      detail: reason instanceof Error ? (reason.stack ?? reason.message) : String(reason),
    });
  let storage: Storage;
  try {
    storage = localStorage;
  } catch (reason) {
    error("access", reason);
    return report;
  }

  const keys = new Set([SAVE_KEY, "hex-command-progress-index", `${SAVE_KEY}-unrestored`]);
  // Enumerate storage itself rather than the progress index, retaining orphan payloads.
  try {
    const length = storage.length;
    for (let index = 0; index < length; index++) {
      try {
        const key = storage.key(index);
        if (key?.startsWith("hex-command-progress:")) keys.add(key);
      } catch (reason) {
        error("key", reason, { index });
      }
    }
  } catch (reason) {
    error("length", reason);
  }
  for (const key of keys) {
    try {
      report.values[key] = storage.getItem(key);
    } catch (reason) {
      error("read", reason, { key });
    }
  }
  return report;
}
