import { activeGoalTiles, campaignMission, goalProgress } from "../game/campaign-progress";
import type { GameView, UnitView, TileView } from "../game/view";
import type { Artwork } from "../presentation/art";
import { TERRAIN, TYPES } from "../game/catalog";
import { artSprite } from "../presentation/art";
import {
  pixelDefaultFloors,
  pixelFloorColors,
  pixelTerrainLayouts,
} from "../presentation/pixel-art";
import { battlefieldSpriteSize, hexPoints, tileCenter } from "./geometry";

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

/** Connections select native frames; they do not change movement or protection. */
export function terrainArtNames(
  tile: Pick<TileView, "terrain" | "connections" | "deadEnds">,
): string[] {
  const directions = ["e", "se", "sw", "w", "nw", "ne"];
  const mask = (tile.connections ?? []).reduce(
    (mask, direction) => mask | (1 << directions.indexOf(direction)),
    0,
  );
  if (tile.terrain === "road" && tile.deadEnds?.length) {
    const capped = tile.deadEnds.reduce(
      (mask, direction) => mask | (1 << directions.indexOf(direction)),
      0,
    );
    return [`road_${mask}-end-${capped}`];
  }
  if (["road", "village"].includes(tile.terrain)) {
    if (tile.terrain === "village" && !mask) return ["village_0"];
    if (mask && mask !== 9) return [`${tile.terrain}_${mask}`];
  }
  if (tile.terrain === "bridge" || tile.terrain === "ford") {
    if (mask === 18) return [`${tile.terrain}_nw-se`];
    if (mask === 36) return [`${tile.terrain}_ne-sw`];
  }
  return [tile.terrain];
}

/** Stable per-map arrangements, independent of redraws, terrain rules and floor colors. */
export function terrainProps(
  tile: Pick<TileView, "terrain" | "id" | "connections" | "deadEnds">,
  mapId: string,
) {
  const layouts = pixelTerrainLayouts[terrainArtNames(tile)[0]!];
  if (!layouts?.length) return [];
  let hash = 2166136261;
  for (const character of `${mapId}:${tile.id}`) {
    hash = Math.imul(hash ^ character.charCodeAt(0), 16777619);
  }
  hash = Math.imul(hash ^ (hash >>> 16), 0x85ebca6b);
  hash ^= hash >>> 13;
  return layouts[(hash >>> 0) % layouts.length]!;
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
      const spriteSize = battlefieldSpriteSize;
      const spriteClasses = `unit-art ${unit.side === "enemy" ? "enemy-unit" : ""} ${unit.attacked ? "spent-unit" : ""}`;
      occupant = {
        unit,
        teamColor: unit.side === "player" ? "#6fdec0" : "#ff9590",
        artwork: artSprite(
          artwork,
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
      fill:
        (tile.floor && pixelFloorColors[tile.floor]) ||
        pixelFloorColors[pixelDefaultFloors[tile.terrain]]!,
      classes,
      label: tileLabel(tile, unit, reachable, target, objectiveTiles.get(tile.id) ?? []),
      terrainArtwork: [
        ...terrainArtNames(tile).map((name) => ({ name, x: 0, y: 0 })),
        ...terrainProps(tile, game.mapId),
      ]
        .map((piece) =>
          artSprite(
            artwork,
            "terrain",
            piece.name,
            x - battlefieldSpriteSize / 2 + (piece.x * battlefieldSpriteSize) / 64,
            y - battlefieldSpriteSize / 2 + (piece.y * battlefieldSpriteSize) / 64,
            battlefieldSpriteSize,
            battlefieldSpriteSize,
            "terrain-art",
            "player",
            tile.floor,
          ),
        )
        .join(""),
      occupant,
      ownerColor: tile.owner === "player" ? "#6fdec0" : "#ff9590",
    };
  });
}
