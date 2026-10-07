export const SAVE_KEY = "hex-command-battle";
export const REJECTED_SAVE_KEY = `${SAVE_KEY}-unrestored`;

/** Browser storage is optional; failure must leave the current battle playable. */
export function readSave(): string | null {
  try {
    return localStorage.getItem(SAVE_KEY);
  } catch {
    return null;
  }
}
export function storeSave(text: string): void {
  try {
    localStorage.setItem(SAVE_KEY, text);
  } catch {
    /* Full or disabled storage. */
  }
}
/** Preserve the original bytes, including malformed JSON, before allowing a fresh save. */
export function preserveRejectedSave(text: string): boolean {
  try {
    localStorage.setItem(REJECTED_SAVE_KEY, text);
    return true;
  } catch {
    /* Full or disabled storage. */
    return false;
  }
}
