export const SAVE_KEY = "hex-command-battle";
export const HISTORY_KEY = "hex-command-progress-index";
export const historyPayloadKey = (id: string): string => `hex-command-progress:${id}`;

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
/** Failed validation discards only the current save. */
export function wipeSave(): void {
  try {
    localStorage.removeItem(SAVE_KEY);
  } catch {
    /* Disabled browser storage must not stop a fresh game. */
  }
}
