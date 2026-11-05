export const SAVE_KEY = "hex-command-battle";

/** Browser storage is optional; failure must leave the current battle playable. */
export function readSave(): string | null {
  try {
    return localStorage.getItem(SAVE_KEY);
  } catch {
    return null;
  }
}
export function storeSave(text: string): void {
  localStorage.setItem(SAVE_KEY, text);
}
/** Failed migration or validation discards the save; historical backups are obsolete. */
export function wipeSave(): void {
  try {
    localStorage.removeItem(SAVE_KEY);
    localStorage.removeItem(`${SAVE_KEY}-unrestored`);
  } catch {
    /* Disabled browser storage must not stop a fresh game. */
  }
}
