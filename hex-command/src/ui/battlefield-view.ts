import { activeGoalTiles, campaignMission, goalProgress } from "../game/campaign-progress";
import type { GameView, UnitView, TileView } from "../game/view";
import type { Artwork } from "../presentation/art";
import type { FrameImage } from "../presentation/sprite-images";
import { TERRAIN, TYPES } from "../game/catalog";
import { artFrame } from "../presentation/art";
import { pixelDefaultFloors, pixelFloorColors } from "../presentation/pixel-art";
import { battlefieldSpriteSize, hexPoints, tileCenter } from "./geometry";
import { terrainLayout } from "../presentation/terrain-layout";
export { terrainArtNames } from "../presentation/terrain-connections";

export interface TileUnitView {
  unit: UnitView;
  teamColor: string;
  frame: Readonly<FrameImage> | undefined;
  spriteSize: number;
  healthWidth: number;
}

export interface BattlefieldTileView extends TileView {
  x: number;
  y: number;
  points: string;
  clipId: string;
  fill: string;
  classes: string;
  label: string;
  terrainFrame: Readonly<FrameImage> | undefined;
  spriteSize: number;
  occupant: TileUnitView | null;
  ownerColor: string;
}

export function terrainProps(
  tile: Pick<TileView, "terrain" | "id" | "connections" | "deadEnds">,
  mapId: string,
) {
  return terrainLayout(tile, mapId).props;
}

function tileLabel(
  tile: TileView,
  occupant: UnitView | undefined,
  reachable: boolean,
  target: boolean,
  objectives: readonly string[],
): string {
  const parts = [TERRAIN[tile.terrain].name, `column ${tile.col + 1}`, `row ${tile.row + 1}`];
  if (tile.owner) parts.push(tile.owner === "player" ? "your territory" : "enemy territory");
  if (occupant) {
    const team = occupant.side === "player" ? "your" : "enemy";
    const typeName = TYPES[occupant.type].name;
    const name = occupant.name ? `${occupant.name} (${typeName})` : typeName;
    parts.push(`${team} ${name}`, `${occupant.hp} health`);
    if (occupant.attacked) parts.push("turn finished");
    if (occupant.retreating) parts.push("retreating");
  }
  for (const objective of objectives) parts.push(`Objective: ${objective}`);
  if (reachable) parts.push("move here");
  if (target) parts.push("attack available");
  return parts.join(", ");
}

/** Prepare geometry and display state once so SVG markup only describes its layers. */
export function battlefieldTiles(
  game: GameView,
  selected: string | null,
  artwork: Artwork,
  recruitTileId: string | null,
): BattlefieldTileView[] {
  const selectedUnit = game.unit(selected);
  const canAct = game.turn === "player" && !game.winner && selectedUnit?.side === "player";
  const reachableTiles = canAct ? game.reachable(selected) : new Map<string, number>();
  const targetTiles = new Set(canAct ? game.targets(selected).map((unit) => unit.tile) : []);

  const objectiveTiles = new Map<string, string[]>();
  if (game.campaign) {
    for (const goal of campaignMission(game.campaign).goals) {
      const progress = goalProgress(game, goal);
      for (const tile of activeGoalTiles(game, goal)) {
        const labels = objectiveTiles.get(tile) ?? [];
        labels.push(progress.label);
        objectiveTiles.set(tile, labels);
      }
    }
  }
  return game.tiles.map((tile) => {
    const { x, y } = tileCenter(tile);
    const unit = game.at(tile.id);
    const reachable = reachableTiles.has(tile.id);
    const target = targetTiles.has(tile.id);
    const classes = [
      "hex",
      objectiveTiles.has(tile.id) ? "objective" : "",
      `terrain-${tile.terrain}`,
      selectedUnit?.tile === tile.id ? "selected" : "",
      reachable ? "reachable" : "",
      target ? "target" : "",
      tile.id === recruitTileId ? "recruitable" : "",
    ]
      .filter(Boolean)
      .join(" ");

    let occupant: TileUnitView | null = null;
    if (unit) {
      occupant = {
        unit,
        teamColor: unit.side === "player" ? "#6fdec0" : "#ff9590",
        frame: artFrame(artwork, "units", unit.type, unit.side),
        spriteSize: battlefieldSpriteSize,
        healthWidth: (38 * unit.hp) / TYPES[unit.type].hp,
      };
    }

    return {
      ...tile,
      x,
      y,
      points: hexPoints(x, y),
      clipId: `hex-clip-${tile.col}-${tile.row}`,
      fill:
        (tile.floor && pixelFloorColors[tile.floor]) ||
        pixelFloorColors[pixelDefaultFloors[tile.terrain]]!,
      classes,
      label: tileLabel(tile, unit, reachable, target, objectiveTiles.get(tile.id) ?? []),
      spriteSize: battlefieldSpriteSize,
      terrainFrame: artFrame(
        artwork,
        "terrain",
        terrainLayout(tile, game.mapId).name,
        "player",
        tile.floor,
      ),
      occupant,
      ownerColor: tile.owner === "player" ? "#6fdec0" : "#ff9590",
    };
  });
}
