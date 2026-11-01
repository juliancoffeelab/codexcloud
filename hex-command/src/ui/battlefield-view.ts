import { activeGoalTiles, campaignMission, goalProgress } from "../game/campaign-progress";
import type { GameView, UnitView, TileView } from "../game/view";
import type { Artwork } from "../presentation/art";
import type { FrameImage } from "../presentation/art";
import { TERRAIN, TYPES } from "../game/catalog";
import { artFrame } from "../presentation/art";
import { terrainTileView, type TerrainTileView } from "../presentation/terrain-map";
import { battlefieldSpriteSize } from "./geometry";
import { terrainLayout } from "../presentation/terrain-layout";
export { terrainArtNames } from "../presentation/terrain-connections";

export interface TileUnitView {
  unit: UnitView;
  faction: import("../../rust/pkg/hex_command").UnitFaction;
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
  spriteX: number;
  spriteY: number;
  spriteWidth: number;
  spriteHeight: number;
  occupant: TileUnitView | null;
  ownerColor: string;
}

/** Preserve cells whose immutable terrain, occupant and display attributes are
 * unchanged. Incremental engine publications retain the underlying tile values,
 * so moving a unit does not invalidate every empty cell's SVG subtree.
 */
export class BattlefieldViewCache {
  private scene?: {
    game: GameView;
    artwork: Artwork;
    terrain: ReadonlyMap<string, TerrainTileView> | undefined;
    tiles: BattlefieldTileView[];
  };

  prepare(
    game: GameView,
    selected: string | null,
    artwork: Artwork,
    recruitTileId: string | null,
    terrain?: ReadonlyMap<string, TerrainTileView>,
  ) {
    const previous = this.scene;
    let tiles = battlefieldTiles(game, selected, artwork, recruitTileId, terrain);
    if (
      previous &&
      previous.artwork === artwork &&
      previous.terrain === terrain &&
      previous.game.tiles.length === game.tiles.length
    ) {
      tiles = tiles.map((tile, index) => {
        const old = previous.tiles[index]!;
        return previous.game.tiles[index] === game.tiles[index] &&
          old.occupant?.unit === tile.occupant?.unit &&
          old.occupant?.faction === tile.occupant?.faction &&
          old.classes === tile.classes &&
          old.label === tile.label
          ? old
          : tile;
      });
    }
    this.scene = { game, artwork, terrain, tiles };
    return tiles;
  }
}

export function terrainProps(
  tile: Pick<TileView, "terrain" | "id" | "connections" | "deadEnds" | "wallCorners">,
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
  faction?: import("../../rust/pkg/hex_command").UnitFaction,
): string {
  const parts = [TERRAIN[tile.terrain].name, `column ${tile.col + 1}`, `row ${tile.row + 1}`];
  if (tile.owner) parts.push(tile.owner === "player" ? "your territory" : "enemy territory");
  if (occupant) {
    const team =
      faction?.relation === "neutral"
        ? "neutral"
        : faction?.controllable
          ? "your"
          : faction?.relation === "allied"
            ? "allied"
            : "enemy";
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
  terrain?: ReadonlyMap<string, TerrainTileView>,
): BattlefieldTileView[] {
  const selectedUnit = game.unit(selected);
  const canAct =
    game.turn === "player" &&
    !game.winner &&
    selectedUnit &&
    game.faction(selectedUnit.id).controllable;
  const reachableTiles = canAct ? game.reachable(selected) : new Map<string, number>();
  const targetTiles = new Set(canAct ? game.targets(selected).map((unit) => unit.tile) : []);

  const objectiveTiles = new Map<string, string[]>();
  if (game.campaign) {
    for (const [index, goal] of campaignMission(game.campaign).goals.entries()) {
      if (game.objectives && !game.objectives.goals[index]?.visible) continue;
      const progress = goalProgress(game, goal);
      for (const tile of activeGoalTiles(game, goal)) {
        const labels = objectiveTiles.get(tile) ?? [];
        labels.push(progress.label);
        objectiveTiles.set(tile, labels);
      }
    }
  }
  const occupants = new Map(game.units.map((unit) => [unit.tile, unit]));
  const terrainViews =
    terrain ??
    new Map(game.tiles.map((tile) => [tile.id, terrainTileView(tile, game.mapId, artwork)]));
  return game.tiles.map((tile) => {
    const unit = occupants.get(tile.id);
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
      const faction = game.faction(unit.id);
      occupant = {
        unit,
        faction,
        teamColor:
          (
            { player: "#6fdec0", enemy: "#ff9590", passive: "#efecdc", green: "#83b765" } as Record<
              string,
              string
            >
          )[faction.palette] ?? "#efecdc",
        frame: artFrame(artwork, "units", unit.type, faction.palette),
        spriteSize: battlefieldSpriteSize,
        healthWidth: (38 * unit.hp) / TYPES[unit.type].hp,
      };
    }

    return {
      ...tile,
      ...terrainViews.get(tile.id)!,
      classes,
      label: tileLabel(
        tile,
        unit,
        reachable,
        target,
        objectiveTiles.get(tile.id) ?? [],
        occupant?.faction,
      ),
      occupant,
      ownerColor: tile.owner === "player" ? "#6fdec0" : "#ff9590",
    };
  });
}
