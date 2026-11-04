import type { BattlefieldTileView } from "../ui/battlefield-view";
import type { GameView } from "../game/view";
import { pixelSprites } from "./pixel-art";

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
  const piles = new Map<string, typeof game.groundItems>();
  for (const item of game.groundItems)
    piles.set(item.tile, [...(piles.get(item.tile) ?? []), item]);
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
