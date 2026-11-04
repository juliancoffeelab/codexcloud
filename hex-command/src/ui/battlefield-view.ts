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
interface CachedTile {
  source: TileView;
  objectives: readonly string[] | undefined;
  view: BattlefieldTileView;
}

export class BattlefieldViewCache {
  private artwork?: Artwork;
  private terrain?: ReadonlyMap<string, TerrainTileView>;
  private mapId?: string;
  private cells?: CachedTile[];

  prepare(
    game: GameView,
    selected: string | null,
    artwork: Artwork,
    recruitTileId: string | null,
    terrain?: ReadonlyMap<string, TerrainTileView>,
  ) {
    const next: CachedTile[] = [];
    const previous =
      this.artwork === artwork && this.terrain === terrain && this.mapId === game.mapId
        ? this.cells
        : undefined;
    const tiles = battlefieldTiles(game, selected, artwork, recruitTileId, terrain, {
      previous,
      next,
    });
    this.artwork = artwork;
    this.terrain = terrain;
    this.mapId = game.mapId;
    this.cells = next;
    return tiles;
  }
}

export function terrainProps(
  tile: Pick<TileView, "terrain" | "id" | "connections" | "deadEnds" | "walls">,
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
  if (tile.owner)
    parts.push(
      tile.owner === "player" ? "your territory" : `${tile.owner.replaceAll("-", " ")} territory`,
    );
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
  reuse?: { previous?: readonly CachedTile[]; next: CachedTile[] },
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
  for (const status of game.objectives?.goals ?? []) {
    if (!status.visible) continue;
    for (const tile of status.activeTiles) {
      const labels = objectiveTiles.get(tile) ?? [];
      labels.push(status.progress.label);
      objectiveTiles.set(tile, labels);
    }
  }
  const occupants = new Map(game.units.map((unit) => [unit.tile, unit]));
  const terrainViews =
    terrain ??
    new Map(game.tiles.map((tile) => [tile.id, terrainTileView(tile, game.mapId, artwork)]));
  return game.tiles.map((tile, index) => {
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

    const faction = unit ? game.faction(unit.id) : undefined;
    const objectives = objectiveTiles.get(tile.id);
    const old = reuse?.previous?.[index];
    if (
      old?.source === tile &&
      old.view.occupant?.unit === unit &&
      old.view.occupant?.faction === faction &&
      old.view.classes === classes &&
      (old.objectives === objectives ||
        (old.objectives?.length === objectives?.length &&
          old.objectives?.every((label, i) => label === objectives?.[i])))
    ) {
      reuse!.next.push(old);
      return old.view;
    }
    let occupant: TileUnitView | null = null;
    if (unit && faction) {
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

    const view = {
      ...tile,
      ...terrainViews.get(tile.id)!,
      classes,
      label: tileLabel(tile, unit, reachable, target, objectives ?? [], occupant?.faction),
      occupant,
      ownerColor:
        tile.owner === "player"
          ? "#6fdec0"
          : (({ green: "#83b765", passive: "#efecdc", enemy: "#ff9590" } as Record<string, string>)[
              game.factions.find((f) => f.id === tile.owner)?.palette ?? "enemy"
            ] ?? "#ff9590"),
    };
    reuse?.next.push({ source: tile, objectives, view });
    return view;
  });
}
