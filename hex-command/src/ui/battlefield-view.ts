import { activeGoalTiles, campaignMission, goalProgress } from "../game/campaign-progress";
import type { GameView, UnitView, TileView } from "../game/view";
import type { ArtStyle } from "../presentation/art";
import { TERRAIN, TYPES } from "../game/engine";
import { artSprite } from "../presentation/art";
import { pixelTerrainColors } from "../presentation/pixel-art";
import { colors, hexPoints, tileCenter } from "./geometry";

export interface TileUnitView {
  unit: UnitView;
  teamColor: string;
  artwork: string;
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
  terrainArtwork: string;
  occupant: TileUnitView | null;
  ownerColor: string;
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
  artStyle: ArtStyle,
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
      const spriteSize = TYPES[unit.type].leader ? 62 : 56;
      const spriteClasses = `unit-art ${unit.side === "enemy" ? "enemy-unit" : ""} ${unit.attacked ? "spent-unit" : ""}`;
      occupant = {
        unit,
        teamColor: unit.side === "player" ? "#6fdec0" : "#ff9590",
        artwork: artSprite(
          artStyle,
          "units",
          unit.type,
          x - spriteSize / 2,
          y + 20 - spriteSize,
          spriteSize,
          spriteSize,
          spriteClasses,
          unit.side,
        ),
        healthWidth: (38 * unit.hp) / TYPES[unit.type].hp,
      };
    }

    return {
      ...tile,
      x,
      y,
      points: hexPoints(x, y),
      clipId: `hex-clip-${tile.col}-${tile.row}`,
      fill: artStyle === "pixel" ? pixelTerrainColors[tile.terrain] : colors[tile.terrain],
      classes,
      label: tileLabel(tile, unit, reachable, target, objectiveTiles.get(tile.id) ?? []),
      terrainArtwork: artSprite(
        artStyle,
        "terrain",
        tile.terrain,
        x - 35,
        y - 35,
        70,
        70,
        "terrain-art",
      ),
      occupant,
      ownerColor: tile.owner === "player" ? "#6fdec0" : "#ff9590",
    };
  });
}
