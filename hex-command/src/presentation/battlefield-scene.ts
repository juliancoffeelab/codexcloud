import type { BattlefieldTileView } from "../ui/battlefield-view";
import type { GameView } from "../game/view";
import { currentArtwork, itemFrame } from "./art";

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
  items: { key: string; x: number; y: number; url: string }[];
  fog: readonly string[];
}

/** Presentation-only snapshot; gameplay and authored data still come from Rust. */
export function battlefieldScene(
  game: GameView,
  tiles: readonly BattlefieldTileView[],
  reducedMotion: boolean,
  key: string,
): BattlefieldScene {
  const fog = new Set(game.presentation.fog);
  const centers = new Map(tiles.map((tile) => [tile.id, tile]));
  const piles = new Map<string, Array<(typeof game.groundItems)[number]>>();
  for (const item of game.groundItems) {
    let pile = piles.get(item.tile);
    if (!pile) {
      pile = [];
      piles.set(item.tile, pile);
    }
    pile.push(item);
  }
  return {
    key,
    reducedMotion,
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
      const occurrences = new Map<string, number>();
      return items.map((ground, index) => {
        const identity = JSON.stringify(ground.item),
          occurrence = occurrences.get(identity) ?? 0;
        occurrences.set(identity, occurrence + 1);
        return {
          key: `${id}:${identity}:${occurrence}`,
          x: tile.x + (index - (items.length - 1) / 2) * 12,
          y: tile.y,
          url: itemFrame(currentArtwork(), ground.item.id).url,
        };
      });
    }),
    fog: game.presentation.fog,
  };
}
