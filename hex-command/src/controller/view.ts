import type { GameView, UnitView } from "../game/view";
import type {
  GameSnapshot,
  CampaignId,
  UnitType,
  RecruitType as GameRecruitType,
} from "../game/types";
import { TYPES } from "../game/catalog";
import type { Artwork } from "../presentation/art";

export type RecruitType = GameRecruitType;

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
  readonly artwork: Artwork;
  readonly sound: boolean;
  readonly canUndo: boolean;
  readonly recruitment: RecruitmentContext | null;
}

/** Components can issue commands, but cannot read hidden controller state. */
export interface BattleCommands {
  startCampaign(id: CampaignId): GameSnapshot;
  nextMission(recallIds: string[]): GameSnapshot;
  retryMission(): GameSnapshot;
  advanceDialogue(skip?: boolean): void;
  previousDialogue(): void;
  choose(tileId: string): void;
  focusTile(tileId: string): void;
  clearSelection(clearInspection?: boolean): void;
  attackSelected(attackId: string): void;
  recruitHere(type: RecruitType, tileId: string): void;
  advanceUnit(unitId: string, type: UnitType): void;
  endTurn(): Promise<GameSnapshot | undefined>;
  undoMove(): GameSnapshot;
  reset(mapId?: string): GameSnapshot;
  setSound(enabled: boolean): Promise<void>;
  error(error: unknown): void;
}

export function createBattleCommands(controller: BattleCommands): BattleCommands {
  return Object.freeze({
    startCampaign: controller.startCampaign.bind(controller),
    nextMission: controller.nextMission.bind(controller),
    retryMission: controller.retryMission.bind(controller),
    advanceDialogue: controller.advanceDialogue.bind(controller),
    previousDialogue: controller.previousDialogue.bind(controller),
    choose: controller.choose.bind(controller),
    focusTile: controller.focusTile.bind(controller),
    clearSelection: controller.clearSelection.bind(controller),
    attackSelected: controller.attackSelected.bind(controller),
    recruitHere: controller.recruitHere.bind(controller),
    advanceUnit: controller.advanceUnit.bind(controller),
    endTurn: controller.endTurn.bind(controller),
    undoMove: controller.undoMove.bind(controller),
    reset: controller.reset.bind(controller),
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
  if (!tile || game.at(tile.id) || game.winner || game.unit(selected)) {
    return null;
  }
  const keep = game
    .neighbors(tile.id)
    .find((neighbor) => neighbor.terrain === "keep" && neighbor.owner === "player");
  if (!keep) return null;
  const commander = game.units.find((unit) => unit.side === "player" && TYPES[unit.type].leader);
  return Object.freeze({ commander, stationed: commander?.tile === keep.id });
}
