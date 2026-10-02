import type { GameView, UnitView } from "../game/view";
import type { GameSnapshot, UnitType } from "../game/types";
import type { ArtStyle } from "../presentation/art";

export type RecruitType = Exclude<UnitType, "commander">;
export const recruitTypes = ["soldier", "archer", "scout"] as const;

export interface RecruitmentContext {
  readonly commander: UnitView | undefined;
  readonly stationed: boolean;
}

/** Every display decision belongs to one published revision. */
export interface BattleView {
  readonly game: GameView;
  readonly selected: string | null;
  readonly target: string | null;
  readonly focusedTile: string;
  readonly inspectedTile: string | null;
  readonly renderRevision: number;
  readonly status: string;
  readonly artStyle: ArtStyle;
  readonly artLoading: boolean;
  readonly sound: boolean;
  readonly canUndo: boolean;
  readonly recruitment: RecruitmentContext | null;
}

/** Components can issue commands, but cannot read hidden controller state. */
export interface BattleCommands {
  choose(tileId: string): void;
  focusTile(tileId: string): void;
  clearSelection(clearInspection?: boolean): void;
  attackSelected(attackId: string): void;
  recruitHere(type: RecruitType, tileId: string): void;
  endTurn(): Promise<GameSnapshot | undefined>;
  undoMove(): GameSnapshot;
  reset(mapId?: string): GameSnapshot;
  setArtStyle(style: ArtStyle): Promise<void>;
  setSound(enabled: boolean): Promise<void>;
  error(error: unknown): void;
}

export function createBattleCommands(controller: BattleCommands): BattleCommands {
  return Object.freeze({
    choose: controller.choose.bind(controller),
    focusTile: controller.focusTile.bind(controller),
    clearSelection: controller.clearSelection.bind(controller),
    attackSelected: controller.attackSelected.bind(controller),
    recruitHere: controller.recruitHere.bind(controller),
    endTurn: controller.endTurn.bind(controller),
    undoMove: controller.undoMove.bind(controller),
    reset: controller.reset.bind(controller),
    setArtStyle: controller.setArtStyle.bind(controller),
    setSound: controller.setSound.bind(controller),
    error: controller.error.bind(controller),
  });
}

/** Recruitment uses the same immutable game and selection as the rest of the UI. */
export function recruitmentContext(
  game: GameView,
  selected: string | null,
  inspectedTile: string | null,
): RecruitmentContext | null {
  const tile = game.tile(inspectedTile);
  if (!tile || tile.terrain === "water" || game.at(tile.id) || game.winner || game.unit(selected)) {
    return null;
  }
  const keep = game
    .neighbors(tile.id)
    .find((neighbor) => neighbor.terrain === "keep" && neighbor.owner === "player");
  if (!keep) return null;
  const commander = game.units.find((unit) => unit.side === "player" && unit.type === "commander");
  return Object.freeze({ commander, stationed: commander?.tile === keep.id });
}
