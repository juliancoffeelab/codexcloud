import { STATE, ENTRIES, TURNS, transaction, request } from "./storage";
import { captureImportSource } from "./import";

export interface BrowserSaveDiagnostics {
  indexedDB?: Record<string, Record<string, unknown>>;
  localStorage?: Record<string, string | null>;
  errors: { operation: string; detail: string }[];
}
/** Raw repository records, including orphans, without parsing or migration. */
export async function captureBrowserSaves(): Promise<BrowserSaveDiagnostics> {
  const report: BrowserSaveDiagnostics = { errors: [] };
  try {
    report.indexedDB = await transaction([STATE, ENTRIES, TURNS], "readonly", async (tx) => {
      const stores: Record<string, Record<string, unknown>> = {};
      for (const name of [STATE, ENTRIES, TURNS]) {
        const store = tx.objectStore(name);
        const [keys, values] = await Promise.all([
          request(store.getAllKeys()),
          request(store.getAll()),
        ]);
        stores[name] = Object.fromEntries(keys.map((key, i) => [String(key), values[i]]));
      }
      return stores;
    });
  } catch (error) {
    report.errors.push({ operation: "indexedDB", detail: String(error) });
  }
  try {
    report.localStorage = captureImportSource();
  } catch (error) {
    report.errors.push({ operation: "localStorage migration source", detail: String(error) });
  }
  return report;
}
