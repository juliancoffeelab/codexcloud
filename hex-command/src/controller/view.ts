import { cellKey } from "../game/hex";
import { traceCommand } from "../presentation/performance-trace";
import type { SessionSetup, OpponentSummary } from "../game/setup";
import type { GameView, UnitView, Immutable } from "../game/view";
import type {
  GameSnapshot,
  CombatAnimation,
  CampaignId,
  UnitType,
  RecruitType as GameRecruitType,
} from "../game/types";
import { TYPES, TERRAIN, isLeader } from "../game/catalog";
import type { Artwork } from "../presentation/art";

export type RecruitType = GameRecruitType;

export interface RecruitmentContext {
  readonly commander: UnitView | undefined;
  readonly stationed: boolean;
}

/** Every display decision belongs to one published revision. */
export interface BattleView {
  readonly game: GameView;
  readonly needsNewGame: boolean;
  readonly canSave: boolean;
  readonly activeRunId: string | null;
  readonly selected: string | null;
  readonly target: string | null;
  readonly inspectedTile: string | null;
  readonly renderRevision: number;
  readonly sessionVersion: number;
  readonly status: string;
  readonly persistence: string;
  readonly saveError: string | null;
  readonly loadProblem?: import("../save/repair-types").LoadProblem | null;
  readonly saveFailure?: import("../save/failure").SaveFailure | null;
  readonly artwork: Artwork;
  readonly sound: boolean;
  readonly opponent: OpponentSummary;
  readonly canUndo: boolean;
  readonly completedTurns: number;
  readonly recruitment: RecruitmentContext | null;
  readonly combatPlayback?: import("./combat-playback").CombatPlaybackFrame | null;
  readonly combat?: Immutable<CombatAnimation> | null;
}

/** Components can issue commands, but cannot read hidden controller state. */
export interface BattleCommands {
  startCampaign(id: CampaignId, setup?: SessionSetup): Promise<GameSnapshot>;
  nextMission(recallIds: string[]): Promise<GameSnapshot>;
  retryMission(): Promise<GameSnapshot>;
  advanceDialogue(skip?: boolean): void;
  previousDialogue(): void;
  choose(tileId: string): void;
  clearSelection(clearInspection?: boolean): void;
  attackSelected(attackId: string): void;
  recruitHere(type: RecruitType, tileId: string): void;
  advanceUnit(unitId: string, type: UnitType): void;
  dropItem(unitId: string, itemId: string): void;
  endTurn(): Promise<GameSnapshot | undefined>;
  undoMove(): GameSnapshot;
  reset(
    mapId?: string,
    options?: { armies?: [string, string]; setup: SessionSetup },
  ): Promise<GameSnapshot>;
  setSound(enabled: boolean): Promise<void>;
  downloadDebugReport(): Promise<void>;
  repairInventory(): Promise<import("../save/repair-types").RepairInventory>;
  repairSavedData(target: import("../save/repair-types").RepairTarget): Promise<void>;
  retrySavedProgress(expected?: import("../save/metadata").CurrentEntry | null): Promise<boolean>;
  savedProgress(): Promise<import("../save/metadata").TurnEntry[]>;
  saveGame(): Promise<void>;
  savedSummary(id: string): Promise<import("../save/summary").SaveSummary>;
  savedFlashSave(): Promise<import("../save/metadata").CurrentEntry | undefined>;
  restoreProgress(id: string): Promise<void>;
  deleteSavedProgress(id: string): Promise<void>;
  deleteSavedRun(id: string): Promise<void>;
  pruneSavedRun(id: string): Promise<void>;
  error(error: unknown): void;
}

export function createBattleCommands(controller: BattleCommands): BattleCommands {
  return Object.freeze({
    repairInventory: controller.repairInventory.bind(controller),
    repairSavedData: controller.repairSavedData.bind(controller),
    retrySavedProgress: controller.retrySavedProgress.bind(controller),
    savedProgress: controller.savedProgress.bind(controller),
    saveGame: controller.saveGame.bind(controller),
    savedSummary: controller.savedSummary.bind(controller),
    savedFlashSave: controller.savedFlashSave.bind(controller),
    restoreProgress: controller.restoreProgress.bind(controller),
    deleteSavedProgress: controller.deleteSavedProgress.bind(controller),
    deleteSavedRun: controller.deleteSavedRun.bind(controller),
    pruneSavedRun: controller.pruneSavedRun.bind(controller),
    startCampaign: traceCommand("action.startCampaign", controller.startCampaign.bind(controller)),
    nextMission: traceCommand("action.nextMission", controller.nextMission.bind(controller)),
    retryMission: traceCommand("action.retryMission", controller.retryMission.bind(controller)),
    advanceDialogue: traceCommand(
      "action.advanceDialogue",
      controller.advanceDialogue.bind(controller),
    ),
    previousDialogue: traceCommand(
      "action.previousDialogue",
      controller.previousDialogue.bind(controller),
    ),
    choose: traceCommand("action.choose", controller.choose.bind(controller)),
    clearSelection: traceCommand(
      "action.clearSelection",
      controller.clearSelection.bind(controller),
    ),
    attackSelected: traceCommand(
      "action.attackSelected",
      controller.attackSelected.bind(controller),
    ),
    recruitHere: traceCommand("action.recruitHere", controller.recruitHere.bind(controller)),
    advanceUnit: traceCommand("action.advanceUnit", controller.advanceUnit.bind(controller)),
    dropItem: traceCommand("action.dropItem", controller.dropItem.bind(controller)),
    endTurn: traceCommand("action.endTurn", controller.endTurn.bind(controller)),
    undoMove: traceCommand("action.undoMove", controller.undoMove.bind(controller)),
    reset: traceCommand("action.reset", controller.reset.bind(controller)),
    setSound: traceCommand("action.setSound", controller.setSound.bind(controller)),
    downloadDebugReport: traceCommand(
      "action.downloadDebugReport",
      controller.downloadDebugReport.bind(controller),
    ),
    error: traceCommand("action.error", controller.error.bind(controller)),
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
  const keeps = game
    .neighbors(tile.id)
    .filter(
      (neighbor) =>
        TERRAIN[neighbor.terrain].role === "keep" &&
        (neighbor.owner === "player" ||
          game.factions.some(
            (faction) => faction.id === neighbor.owner && faction.controller.kind === "human",
          )),
    );
  if (!keeps.length) return null;
  const commanders = game.units.filter(
    (unit) => game.faction(unit.id).controllable && isLeader(unit),
  );
  const commander =
    commanders.find((unit) =>
      keeps.some(
        (keep) => keep.id === cellKey(unit.tile) && keep.owner === game.faction(unit.id).id,
      ),
    ) ?? commanders.find((unit) => keeps.some((keep) => keep.owner === game.faction(unit.id).id));
  return Object.freeze({
    commander,
    stationed:
      !!commander &&
      keeps.some(
        (keep) =>
          keep.id === cellKey(commander.tile) && keep.owner === game.faction(commander.id).id,
      ),
  });
}
