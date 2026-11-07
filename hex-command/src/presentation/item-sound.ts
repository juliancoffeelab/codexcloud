/** Presentation assets are keyed by gameplay item ID; gameplay definitions come from Rust. */
import { AUTHORED_ITEMS } from "../game/authored-content";

/** Called for the current map, including potential event rewards and carried equipment. */
export function itemSounds(ids: readonly string[]): string[] {
  const sounds = new Set<string>();
  for (const id of new Set(ids)) {
    const item = AUTHORED_ITEMS[id];
    if (!item) throw new Error(`Unknown item: ${id}`);
    for (const attack of item.attacks ?? []) sounds.add(attack.sound);
  }
  return [...sounds];
}
