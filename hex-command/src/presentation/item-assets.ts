/** Presentation assets are keyed by gameplay item ID; gameplay definitions come from Rust. */
import { AUTHORED_ITEMS } from "../game/authored-content";
import { prepareSounds } from "./sound";

export interface ItemAssets {
  images?: readonly string[];
  sounds?: readonly string[];
}
export const ITEM_ASSETS: Readonly<Record<string, ItemAssets>> = {
  "quarry-hammer": { sounds: ["attack-melee"] },
  "church-mail": {},
  "healing-bottle": {},
};
const images = new Map<string, HTMLImageElement>();
/** Called for the current map, including potential event rewards and carried equipment. */
export function preloadItemAssets(ids: readonly string[]): void {
  const sounds = new Set<string>();
  for (const id of new Set(ids)) {
    for (const attack of AUTHORED_ITEMS[id]?.attacks ?? []) sounds.add(attack.sound);
    for (const sound of ITEM_ASSETS[id]?.sounds ?? []) sounds.add(sound);
    for (const url of ITEM_ASSETS[id]?.images ?? []) {
      if (images.has(url)) continue;
      const image = new Image();
      images.set(url, image);
      image.onerror = () => images.delete(url);
      image.src = url;
    }
  }
  prepareSounds([...sounds]);
}
