import type { TileView } from "../game/view";
import { pixelTerrainLayouts } from "./pixel-art";
import { terrainArtNames } from "./terrain-connections";

/** Stable per-map arrangements, independent of redraws, rules and floor colors. */
export function terrainLayout(
  tile: Pick<TileView, "terrain" | "id" | "connections" | "deadEnds" | "walls">,
  mapId: string,
) {
  const name = terrainArtNames(tile)[0]!;
  const layouts = pixelTerrainLayouts[name];
  if (!layouts?.length) return { name, props: [] };
  let hash = 2166136261;
  for (const character of `${mapId}:${tile.id}`) {
    hash = Math.imul(hash ^ character.charCodeAt(0), 16777619);
  }
  hash = Math.imul(hash ^ (hash >>> 16), 0x85ebca6b);
  hash ^= hash >>> 13;
  const index = (hash >>> 0) % layouts.length;
  return { name: `${name}_layout_${index}`, props: layouts[index]! };
}
