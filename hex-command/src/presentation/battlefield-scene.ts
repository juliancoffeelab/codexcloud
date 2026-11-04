import type { BattlefieldTileView } from "../ui/battlefield-view";
import type { GameView } from "../game/view";
import { artFrame } from "./art";
import type { Artwork } from "./art";
import { pixelSprites } from "./pixel-art";
import { TYPES } from "../game/catalog";
import type { UnitType } from "../game/types";

export interface BattlefieldUnit {
  id: string;
  tile: string;
  x: number;
  y: number;
  url?: string;
  size: number;
  color: string;
  health: number;
  hp: number;
  spent: boolean;
  retreating: boolean;
}
export interface BattlefieldScene {
  key: string;
  reducedMotion: boolean;
  tiles: { id: string; x: number; y: number; classes: string; owner?: string }[];
  units: BattlefieldUnit[];
  items: { x: number; y: number; url?: string }[];
  fog: readonly string[];
  prefetch?: string[];
}

/** Artwork preparation only: this graph does not decide recruitment or promotion legality. */
function spritePrefetch(game: GameView, artwork: Artwork): string[] {
  const queue: { type: string; palette: string }[] = [];
  const seen = new Set<string>();
  const enqueue = (type: string, palette: string) => {
    const key = JSON.stringify([type, palette]);
    if (seen.has(key)) return;
    seen.add(key);
    queue.push({ type, palette });
  };
  // A visible archetype can appear later in another faction's palette (including
  // starting troops outside its recruit pool). This reads no concealed actors.
  for (const faction of game.factions) {
    for (const type of [
      ...(faction.recruits ?? []).map((recruit) => recruit.unit),
      ...game.recruitmentOptions(faction.id),
      ...game.units.map((unit) => unit.type),
    ])
      enqueue(type, faction.palette);
  }
  for (const unit of game.units)
    for (const type of game.promotionOptions(unit.id)) enqueue(type, game.faction(unit.id).palette);
  const urls = new Set<string>();
  // Breadth-first order prepares base artwork before deeper advancement tiers.
  // Bound the speculative list; required images always bypass this budget.
  for (let index = 0; index < queue.length && urls.size < 128; index++) {
    const { type, palette } = queue[index]!;
    const frame = artFrame(artwork, "units", type, palette);
    if (frame) urls.add(frame.url);
    for (const next of TYPES[type as UnitType]?.advancesTo ?? []) enqueue(next, palette);
  }
  return [...urls];
}

/** Presentation-only snapshot; gameplay and authored data still come from Rust. */
export function battlefieldScene(
  game: GameView,
  tiles: readonly BattlefieldTileView[],
  reducedMotion: boolean,
  key: string,
  artwork: Artwork,
): BattlefieldScene {
  const fog = new Set(game.presentation.fog);
  const centers = new Map(tiles.map((tile) => [tile.id, tile]));
  const piles = new Map<string, typeof game.groundItems>();
  for (const item of game.groundItems)
    piles.set(item.tile, [...(piles.get(item.tile) ?? []), item]);
  return {
    key,
    reducedMotion,
    prefetch: spritePrefetch(game, artwork),
    tiles: tiles.map((tile) => ({
      id: tile.id,
      x: tile.x,
      y: tile.y,
      classes: tile.classes,
      owner: tile.owner && !fog.has(tile.id) ? tile.ownerColor : undefined,
    })),
    units: tiles.flatMap((tile) => {
      const occupant = tile.occupant;
      if (!occupant || (fog.has(tile.id) && occupant.faction.relation === "hostile")) return [];
      return [
        {
          id: occupant.unit.id,
          tile: tile.id,
          x: tile.x,
          y: tile.y,
          url: occupant.frame?.url,
          size: occupant.spriteSize,
          color: occupant.teamColor,
          health: occupant.healthWidth,
          hp: occupant.unit.hp,
          spent: occupant.unit.attacked,
          retreating: !!occupant.unit.retreating,
        },
      ];
    }),
    items: [...piles].flatMap(([id, items]) => {
      const tile = centers.get(id);
      if (!tile || fog.has(id)) return [];
      return items.map((ground, index) => ({
        x: tile.x + (index - (items.length - 1) / 2) * 12,
        y: tile.y,
        url: pixelSprites[`terrain_prop_items_${ground.item.id}`]
          ? `${import.meta.env.BASE_URL}${pixelSprites[`terrain_prop_items_${ground.item.id}`]}`
          : undefined,
      }));
    }),
    fog: game.presentation.fog,
  };
}
