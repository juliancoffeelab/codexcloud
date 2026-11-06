import { DEFAULT_SETUP, opponentSummary, type SessionSetup } from "../game/setup";
import { BATTLE_MAPS } from "../game/maps";
import { dialoguePending } from "../game/campaign-progress";
import type { CampaignId, UnitType } from "../game/types";
import { writable, type Readable, type Writable } from "svelte/store";
import { tick } from "svelte";
import { TYPES, TERRAIN, type Tile, type Unit } from "../game/catalog";
import { createMoveUndo } from "../game/undo";
import { loadBattle, restoreBattleText, type LoadedBattle } from "../save/load";
import { serializeBattle, TurnCheckpoint } from "../save/write";
import { storeSave } from "../save/storage";
import { inspectImport, repairImport } from "../save/repair";
import type { ImportFailure, RepairTarget } from "../save/repair-types";
import { summarizeSave } from "../save/summary";
import { saveFailure, type SaveFailure } from "../save/failure";
import { captureBrowserSaves } from "../save/debug";
import {
  addTurn,
  currentProgress,
  listProgress,
  progressText,
  updateProgress,
  deleteProgress,
  deleteRun,
  pruneProgress,
} from "../save/progress";
import {
  describeFlashSave,
  newRun,
  progressId,
  progressIdentity,
  type ProgressIdentity,
  type TurnEntry,
} from "../save/metadata";
import { currentTurn } from "../save/schema";
import { LiveGame, nativeValue } from "../game/live-game";
import type { GameEngineFactory } from "../game/native-engine";
import { currentArtwork } from "../presentation/art";
import {
  soundEnabled,
  setSoundEnabled,
  unlockSound,
  playSound,
  playInteraction,
  cancelSound,
} from "../presentation/sound";
import { animateTile, animateTurn, type MotionKind } from "../presentation/motion";
import { describeChanges } from "./feedback";
import type { GameView } from "../game/view";
import {
  createBattleCommands,
  recruitmentContext,
  type BattleView,
  type BattleCommands,
  type RecruitType,
} from "./view";

export type { BattleView, RecruitType } from "./view";

interface PublishOptions {
  renderBattle?: boolean;
  gameChanged?: boolean;
  showWinnerMessage?: boolean;
}

interface SessionState {
  selected: string | null;
  target: string | null;
  focusedTile: string;
  inspectedTile: string | null;
  renderRevision: number;
  status: string;
  sound: boolean;
}

interface EnemyTurnRecording {
  checkpoint: string;
  setup: SessionSetup;
  commands: ReturnType<LiveGame["enemyStep"]>[];
  completed: boolean;
}

const initialMessage = "Select a unit. Capture villages. Defeat the red commander.";
const delay = (ms: number) => new Promise<void>((resolve) => setTimeout(resolve, ms));
const enemyThinkingDelay = 350;
const enemyActionDelay = 230;

export class BattleController {
  #game: LiveGame;
  private pendingEnemyReplay: boolean;
  private progress: ProgressIdentity;
  private persistence = "Not saved yet";
  private saveError: string | null = null;
  private saveFailure: SaveFailure | null = null;
  private historyError: string | null = null;
  private historyMaintenanceError: string | null = null;
  private pendingCheckpoint?: TurnCheckpoint;
  private pendingExplicit?: TurnCheckpoint;
  private awaitingNewGame = false;
  private importFailure: ImportFailure | null = null;
  private historyStart: boolean;
  private enemyRecordings: EnemyTurnRecording[] = [];
  private initialized = false;
  private storageQueue: Promise<unknown> = Promise.resolve();
  private initialization?: Promise<void>;
  private settlingSession?: number;
  private soundRequestVersion = 0;
  private readonly history = createMoveUndo();
  private sessionVersion = 0;
  private disposed = false;
  private previousFeedbackState: ReturnType<LiveGame["publishedSnapshot"]> | null = null;
  private state: SessionState;
  private store: Writable<BattleView>;
  private gameView: GameView;
  readonly view: Readable<BattleView>;
  readonly commands: BattleCommands;

  static async create(
    pause: ((ms: number) => Promise<void>) | undefined,
    nativeFactory: GameEngineFactory,
  ) {
    const restored = await loadBattle(nativeFactory);
    return new BattleController(pause, nativeFactory, restored);
  }

  private constructor(
    private readonly pause: (ms: number) => Promise<void> = delay,
    private readonly nativeFactory: GameEngineFactory,
    restored: LoadedBattle,
  ) {
    const game = restored.game;
    this.#game = game;
    this.awaitingNewGame = !restored.save && !!restored.chooseNewGame;
    this.importFailure = restored.importFailure ?? null;
    this.pendingEnemyReplay = restored.pendingEnemy;
    const identity = restored.save?.progress.entry;
    this.progress = identity ? progressIdentity(identity) : newRun();
    this.historyStart = !restored.save;
    this.history.restore(game, restored.undo ?? []);
    if (this.importFailure) this.persistence = "Saved data needs repair";
    if (restored.storageFailure) {
      this.failedSave(restored.storageFailure);
    } else if (restored.save) this.persistence = "Saved locally";
    this.state = {
      selected: null,
      target: null,
      focusedTile: game.units.find((u) => u.side === "player")?.tile ?? game.tiles[0]!.id,
      inspectedTile: null,
      renderRevision: 0,
      status:
        ("notice" in restored && restored.notice) ||
        (game.campaign ? "Follow the mission objectives. Keep your leader safe." : initialMessage),
      sound: soundEnabled(),
    };
    this.gameView = game.publishedView();
    this.store = writable(this.currentView());
    this.view = { subscribe: this.store.subscribe };
    this.previousFeedbackState = game.publishedSnapshot();
    this.commands = createBattleCommands(this);
  }

  /** Stop outstanding turn and preference work when this play session unmounts. */
  dispose(): void {
    this.disposed = true;
    this.sessionVersion++;
    cancelSound();
    this.soundRequestVersion++;
    this.#game.dispose();
  }

  private requireController(): void {
    if (this.disposed) {
      throw new Error("This battle session has closed.");
    }
  }

  private requireSession(): void {
    this.requireController();
    if (this.awaitingNewGame) throw new Error("Choose a new game to continue.");
  }

  private settling() {
    return this.settlingSession === this.sessionVersion;
  }

  private storageUpdate<T>(update: () => T | Promise<T>): Promise<T> {
    const result = this.storageQueue.then(() => {
      this.requireController();
      return update();
    });
    this.storageQueue = result.catch(() => {});
    return result;
  }

  /** Wait for captured action writes before inspecting durable state. */
  async saved(): Promise<void> {
    await this.storageQueue;
  }

  private requireSettled() {
    if (this.settling()) throw new Error("Wait for the turn to finish saving.");
  }

  private active() {
    return (
      !this.awaitingNewGame &&
      !this.settling() &&
      this.#game.turn === "player" &&
      !this.#game.winner &&
      !dialoguePending(this.#game.campaign)
    );
  }

  private requirePlayerActor(unitId: string): void {
    this.requireSession();
    if (!this.active()) {
      throw new Error("Wait for your turn.");
    }
    if (!this.#game.unit(unitId) || !this.#game.faction(unitId).controllable) {
      throw new Error("This unit cannot act now.");
    }
  }

  private currentView(): BattleView {
    return Object.freeze({
      ...this.state,
      game: this.gameView,
      needsNewGame: this.awaitingNewGame,
      canSave:
        !this.awaitingNewGame &&
        !this.importFailure &&
        !this.settling() &&
        (this.#game.turn === "player" || !!this.#game.winner),
      activeRunId: this.awaitingNewGame ? null : this.progress.runId,
      persistence: this.persistence,
      saveError: this.saveError,
      saveFailure: this.saveFailure,
      importFailure: this.importFailure,
      sessionVersion: this.sessionVersion,
      opponent: opponentSummary(this.#game.setup.enemy),
      combat: this.#game.combat,
      artwork: currentArtwork(),
      completedTurns: Math.max(
        0,
        (this.#game.round - 1) * 2 +
          (this.#game.turn === "enemy" ? 1 : 0) -
          (this.#game.setup.first === "enemy" ? 1 : 0),
      ),
      canUndo:
        !this.awaitingNewGame &&
        !this.settling() &&
        !dialoguePending(this.#game.campaign) &&
        this.history.canUndo(this.#game),
      recruitment: recruitmentContext(this.gameView, this.state.selected, this.state.inspectedTile),
    });
  }

  snapshot() {
    return this.#game.snapshot();
  }

  async downloadDebugReport(): Promise<void> {
    try {
      this.requireSession();
      const report = {
        format: "hex-command-debug",
        // Include native checkpoints and exact browser-save bytes for diagnosis.
        version: 1,
        createdAt: new Date().toISOString(),
        setup: this.#game.setup,
        checkpoint: nativeValue(this.#game.checkpoint()),
        enemyTurns: this.enemyRecordings.map(({ checkpoint, ...turn }) => ({
          ...turn,
          checkpoint: JSON.parse(checkpoint),
        })),
        browserSaves: await captureBrowserSaves(),
        controller: {
          progress: this.progress,
          persistence: this.persistence,
          saveError: this.saveError,
          saveFailure: this.saveFailure,
          historyError: this.historyError,
          historyMaintenanceError: this.historyMaintenanceError,
          pendingEnemy:
            this.pendingEnemyReplay || (this.#game.turn === "enemy" && !this.#game.winner),
          pendingEnemyReplay: this.pendingEnemyReplay,
          pendingCheckpoint: this.pendingCheckpoint ?? null,
          importFailure: this.importFailure,
          initialCheckpoint: this.historyStart,
          initialized: this.initialized,
          sessionVersion: this.sessionVersion,
        },
      };
      const url = URL.createObjectURL(
        new Blob([JSON.stringify(report)], { type: "application/json" }),
      );
      const link = document.createElement("a");
      link.href = url;
      link.download = `hex-command-debug-${this.#game.mapId}-turn-${this.#game.round}.json`;
      document.body.append(link);
      link.click();
      link.remove();
      setTimeout(() => URL.revokeObjectURL(url), 60_000);
      this.message("Debug report downloaded. You can attach it to the chat.");
    } catch (error) {
      this.error(error);
    }
  }

  message(status: string) {
    this.state.status = status;
    this.publish({ renderBattle: false, showWinnerMessage: false });
  }

  error(error: unknown) {
    this.message(error instanceof Error ? error.message : String(error));
  }

  private publish({
    renderBattle = true,
    gameChanged = renderBattle,
    showWinnerMessage = true,
  }: PublishOptions = {}) {
    if (this.disposed) {
      return;
    }
    if (renderBattle) {
      this.state.renderRevision++;
    }
    if (showWinnerMessage && this.#game.winner) {
      this.state.status =
        this.#game.campaign?.outcome ??
        (this.#game.winner === "player"
          ? "Victory! The enemy commander has fallen. The field is yours."
          : "Your commander has fallen. Start a new battle and try a different approach.");
    }
    // Selection and preferences reuse the detached game until an engine action.
    if (gameChanged) {
      this.gameView = this.#game.publishedView();
    }
    // Subscribers see a complete view immediately; Svelte batches its DOM work.
    this.store.set(this.currentView());
    if (renderBattle) {
      this.playBattleFeedback();
    }
  }

  private failedSave(error: unknown): void {
    this.persistence = "Local save failed";
    const failure = saveFailure(error);
    this.saveError = failure.detail;
    // A dismissed failure stays quiet while the same condition persists.
    if (!this.saveFailure || this.saveFailure.kind !== failure.kind) this.saveFailure = failure;
  }

  /** Capture synchronously so queued writes never serialize a later game or undo stack. */
  private flashSaveWrite(pendingEnemy = false): () => Promise<void> {
    if (this.awaitingNewGame || this.importFailure) return async () => {};
    const session = this.sessionVersion;
    let text: string | undefined;
    try {
      text = serializeBattle(this.#game, pendingEnemy, {
        undo: this.history.saved(this.#game),
        progress: { entry: describeFlashSave(this.#game, this.progress) },
      });
    } catch (error) {
      this.failedSave(error);
      return async () => {};
    }
    if (text) this.persistence = "Saving…";
    return async () => {
      if (!text || this.disposed || session !== this.sessionVersion) return;
      try {
        await storeSave(text);
        this.persistence = this.historyError ? "History not saved" : "Saved locally";
        this.saveError = this.historyError;
        if (!this.historyError) this.saveFailure = null;
      } catch (error) {
        this.failedSave(error);
      }
      if (session === this.sessionVersion) this.publish({ renderBattle: false });
    };
  }
  private queueFlashSave(pendingEnemy = false): Promise<void> {
    const write = this.flashSaveWrite(pendingEnemy);
    const result = this.storageQueue.then(write);
    this.storageQueue = result.catch((error) => {
      if (!this.disposed) this.error(error);
    });
    return this.storageQueue.then(() => {});
  }

  private persist(pendingEnemy = false): Promise<void> {
    return this.flashSaveWrite(pendingEnemy)();
  }

  private async turnCheckpoint(missionStart = false): Promise<void> {
    if (this.awaitingNewGame || this.importFailure) return;
    const previous = this.progress;
    const next = {
      ...progressIdentity(previous),
      checkpointId: progressId(),
      ordinal: previous.ordinal + 1,
    };
    // Make eligible age-based room before a new full payload needs quota.
    try {
      await pruneProgress();
    } catch (error) {
      this.historyMaintenanceError = String(error);
    }
    try {
      const checkpoint = TurnCheckpoint.capture(
        this.#game,
        next,
        missionStart,
        previous.checkpointId,
      );
      if (!checkpoint) return;
      this.pendingCheckpoint = checkpoint;
      await addTurn(checkpoint);
      this.pendingCheckpoint = undefined;
      this.historyError = null;
      this.progress = next;
      this.historyStart = false;
      // The flash save is independent of the historical point; pruning follows a complete turn write.
      await this.persist();
    } catch (error) {
      this.historyError = `Turn history could not be saved: ${String(error)}. Free storage in Saved progress.`;
      this.failedSave(error);
      return;
    }
    // Optional thinning cannot turn a committed checkpoint into a failed save.
    if (!this.saveError) {
      try {
        await pruneProgress();
        this.historyMaintenanceError = null;
      } catch (error) {
        this.historyMaintenanceError = String(error);
      }
    }
  }

  private requireManagedSaves(): void {
    if (this.importFailure)
      throw new Error("Use Repair saved data before opening normal save management.");
  }
  async savedProgress(): Promise<TurnEntry[]> {
    await this.saved();
    this.requireManagedSaves();
    return listProgress();
  }
  savedSummary(id: string) {
    return this.storageUpdate(async () => {
      this.requireManagedSaves();
      const restored = restoreBattleText(this.nativeFactory, await progressText(id), "turn");
      try {
        if (restored.migrated) await updateProgress(currentTurn(restored.save));
        return summarizeSave(restored.game);
      } finally {
        restored.game.dispose();
      }
    });
  }
  async savedFlashSave() {
    await this.saved();
    this.requireManagedSaves();
    return currentProgress();
  }
  private async finishPendingHistory(): Promise<void> {
    // A fresh in-memory game may run while retained source waits for repair.
    if (this.importFailure) return;
    try {
      if (this.pendingExplicit) {
        await addTurn(this.pendingExplicit);
        this.pendingExplicit = undefined;
      }
      if (this.pendingCheckpoint) {
        const pending = this.pendingCheckpoint;
        await addTurn(pending);
        this.progress = progressIdentity(pending.entry);
        this.pendingCheckpoint = undefined;
        this.historyStart = false;
        this.historyError = null;
      }
    } catch (error) {
      this.failedSave(error);
    }
  }

  repairInventory() {
    return this.storageUpdate(() => {
      if (!this.importFailure) throw new Error("There is no blocked import to repair.");
      return inspectImport(this.importFailure, (save) => {
        const restored = restoreBattleText(
          this.nativeFactory,
          JSON.stringify(save),
          save.progress.entry.kind === "turn" ? "turn" : "flash",
        );
        restored.game.dispose();
      });
    });
  }
  repairSavedData(target: RepairTarget) {
    return this.storageUpdate(async () => {
      if (!this.importFailure) throw new Error("There is no blocked import to repair.");
      await repairImport(this.importFailure, target);
    });
  }
  retryImport(): Promise<boolean> {
    return this.storageUpdate(async () => {
      if (!this.importFailure) return true;
      const restored = await loadBattle(this.nativeFactory);
      if (restored.storageFailure && !restored.importFailure) {
        restored.game.dispose();
        this.importFailure = null;
        this.failedSave(restored.storageFailure);
        this.publish({ renderBattle: false });
        return true;
      }
      if (restored.importFailure) {
        restored.game.dispose();
        this.importFailure = restored.importFailure;
        this.publish({ renderBattle: false });
        return false;
      }
      this.importFailure = null;
      this.pendingExplicit = this.pendingCheckpoint = undefined;
      this.historyError = null;
      if (restored.save) {
        this.progress = progressIdentity(restored.save.progress.entry);
        this.historyStart = false;
        this.installRestored(restored.game, restored.undo ?? [], restored.pendingEnemy);
      } else {
        restored.game.dispose();
        this.sessionVersion++;
        this.awaitingNewGame = true;
        this.history.clear();
        this.persistence = "No saved game";
        this.saveError = null;
        this.saveFailure = null;
        this.publish({ renderBattle: false });
      }
      return true;
    });
  }

  restoreProgress(id: string): Promise<void> {
    return this.storageUpdate(() => this.restoreCheckpoint(id));
  }

  private async restoreCheckpoint(id: string): Promise<void> {
    this.requireSession();
    this.requireManagedSaves();
    const restored = restoreBattleText(this.nativeFactory, await progressText(id), "turn");
    try {
      const turn = currentTurn(restored.save);
      if (restored.migrated) await updateProgress(turn);
      await this.finishPendingHistory();
      const original = turn.progress.entry;
      const identity = {
        runId: original.runId,
        branchId: progressId(),
        checkpointId: original.checkpointId,
        ordinal: original.ordinal,
      };
      const entry = describeFlashSave(restored.game, identity);
      await storeSave(
        JSON.stringify({
          ...restored.save,
          pendingEnemy: false,
          undo: [],
          progress: { entry },
        }),
      );
      this.progress = identity;
      this.pendingCheckpoint = undefined;
      this.historyError = null;
      this.importFailure = null;
    } catch (error) {
      restored.game.dispose();
      this.failedSave(error);
      this.publish({ renderBattle: false });
      throw error;
    }
    this.installRestored(restored.game, [], restored.pendingEnemy);
  }

  private installRestored(
    game: LiveGame,
    undo: import("../game/undo").SavedUndoEntry[],
    pendingEnemy: boolean,
  ): void {
    cancelSound();
    this.sessionVersion++;
    this.#game.dispose();
    this.#game = game;
    this.awaitingNewGame = false;
    this.history.restore(game, undo);
    this.pendingEnemyReplay = pendingEnemy;
    this.enemyRecordings = [];
    this.previousFeedbackState = null;
    this.state.selected = this.state.target = this.state.inspectedTile = null;
    this.state.focusedTile =
      game.units.find((unit) => unit.side === "player")?.tile ?? game.tiles[0]!.id;
    this.state.status = "Saved progress restored. Recorded turn starts remain in Saved games.";
    this.persistence = "Saved locally";
    this.saveError = null;
    this.saveFailure = null;
    this.publish();
    if (pendingEnemy) {
      this.pendingEnemyReplay = false;
      void this.endTurn().catch((error) => this.error(error));
    } else if (game.turn === "enemy" && !game.winner)
      void this.runEnemyTurn(this.sessionVersion).catch((error) => this.error(error));
  }

  deleteSavedProgress(id: string): Promise<void> {
    return this.storageUpdate(() => {
      this.requireManagedSaves();
      return this.cleanupSaves(() => deleteProgress(id));
    });
  }
  deleteSavedRun(id: string): Promise<void> {
    return this.storageUpdate(async () => {
      this.requireManagedSaves();
      if (id !== this.progress.runId || this.awaitingNewGame) {
        await this.cleanupSaves(() => deleteRun(id));
        return;
      }
      await deleteRun(id);
      // Invalidate in-flight enemy work before it can publish another checkpoint.
      this.sessionVersion++;
      this.awaitingNewGame = true;
      this.pendingCheckpoint = undefined;
      this.pendingExplicit = undefined;
      this.pendingEnemyReplay = false;
      this.importFailure = null;
      this.history.clear();
      this.saveFailure = null;
      this.saveError = this.historyError = null;
      this.state.selected = this.state.target = this.state.inspectedTile = null;
      cancelSound();
      this.publish({ renderBattle: false });
    });
  }
  saveGame(): Promise<void> {
    return this.storageUpdate(async () => {
      this.requireSession();
      this.requireSettled();
      if (this.importFailure)
        throw new Error("Finish updating saved progress before making a manual save.");
      try {
        const point = TurnCheckpoint.capture(
          this.#game,
          { ...this.progress, checkpointId: progressId() },
          false,
          this.progress.checkpointId,
          true,
        );
        if (!point) throw new Error("Wait for the enemy turn to finish before saving.");
        this.pendingExplicit = point;
        await addTurn(point);
        this.pendingExplicit = undefined;
        await this.persist();
        if (this.saveError) throw new Error(this.saveError);
      } catch (error) {
        this.failedSave(error);
        throw error;
      } finally {
        this.publish({ renderBattle: false });
      }
    });
  }
  pruneSavedRun(id: string): Promise<void> {
    return this.storageUpdate(() => {
      this.requireManagedSaves();
      return this.cleanupSaves(() => pruneProgress(id));
    });
  }
  private async cleanupSaves(cleanup: () => void | Promise<void>): Promise<void> {
    try {
      await cleanup();
      await this.saveAfterCleanup();
    } catch (error) {
      this.failedSave(error);
      this.publish({ renderBattle: false });
      throw error;
    }
  }

  private async saveAfterCleanup(): Promise<void> {
    if (this.awaitingNewGame) return;
    // Never persist an enemy phase in flight.
    if (this.#game.turn === "enemy" && !this.#game.winner) return;
    if (
      this.pendingCheckpoint &&
      this.pendingCheckpoint.entry.branchId === this.progress.branchId
    ) {
      try {
        await addTurn(this.pendingCheckpoint);
        this.progress = progressIdentity(this.pendingCheckpoint.entry);
        this.historyStart = false;
        this.pendingCheckpoint = undefined;
        this.historyError = null;
      } catch (error) {
        this.failedSave(error);
        this.publish({ renderBattle: false });
        return;
      }
    }
    if (this.pendingExplicit) {
      try {
        await addTurn(this.pendingExplicit);
        this.pendingExplicit = undefined;
      } catch (error) {
        this.failedSave(error);
        this.publish({ renderBattle: false });
        return;
      }
    }
    this.historyError = null;
    await this.persist();
    this.publish({ renderBattle: false });
  }

  private commitAction() {
    // The save writer owns turn-boundary rules, including enemy-first openings.
    void this.queueFlashSave();
    this.publish();
  }

  private afterRender(effect: () => void) {
    const sessionVersion = this.sessionVersion;
    const renderRevision = this.state.renderRevision;
    void tick().then(() => {
      if (
        this.disposed ||
        this.sessionVersion !== sessionVersion ||
        this.state.renderRevision !== renderRevision
      ) {
        return;
      }
      effect();
    });
  }

  private animateTileAfterRender(tile: string, kind: MotionKind) {
    this.afterRender(() => animateTile(tile, kind));
  }

  private playSelectionFeedback(tileId: string) {
    this.animateTileAfterRender(tileId, "select");
    const tile = this.#game.tile(tileId);
    playInteraction({
      unitType: this.#game.at(tileId)?.type,
      terrain: tile && TERRAIN[tile.terrain].role,
    });
  }

  private playBattleFeedback() {
    const current = this.#game.publishedSnapshot();
    const previous = this.previousFeedbackState;
    this.previousFeedbackState = current;
    if (!previous) {
      return;
    }

    const changes = describeChanges(previous, current);
    for (const [tile, kind] of changes.tileEffects) {
      this.animateTileAfterRender(tile, kind);
    }
    for (const tile of changes.capturedTiles) {
      this.animateTileAfterRender(tile, "capture");
    }
    if (changes.turnChanged) {
      this.afterRender(animateTurn);
    }

    // One cue per action, with combat taking precedence over movement/capture.
    if (changes.attacked) {
      const defaultSound = changes.attacker?.type === "archer" ? "attack-bow" : "attack-melee";
      playSound(this.#game.lastCombat?.sound || defaultSound);
    } else if (changes.newWinner) {
      playSound(changes.newWinner === "player" ? "victory" : "defeat");
    } else if (changes.recruited || changes.advanced) {
      playSound("recruit");
    } else if (changes.movedUnit) {
      const tile = this.#game.tile(changes.movedUnit.tile);
      playInteraction({
        unitType: changes.movedUnit.type,
        terrain: tile && TERRAIN[tile.terrain].role,
      });
    } else if (changes.capturedTiles.length > 0) {
      const tile = this.#game.tile(changes.capturedTiles[0]!);
      playInteraction({ terrain: tile && TERRAIN[tile.terrain].role });
    }
  }

  focusTile(tileId: string) {
    this.requireSession();
    if (this.state.focusedTile === tileId || !this.#game.tile(tileId)) return;
    this.state.focusedTile = tileId;
    this.publish({ renderBattle: false });
  }

  clearSelection(clearInspection = false) {
    this.requireSession();
    if (
      !this.state.selected &&
      !this.state.target &&
      !(clearInspection && this.state.inspectedTile)
    ) {
      return;
    }
    this.state.selected = null;
    this.state.target = null;
    if (clearInspection) {
      this.state.inspectedTile = null;
    }
    this.publish({ gameChanged: false });
    this.message("Selection cleared.");
  }

  choose(tileId: string) {
    this.requireSession();
    const tile = this.#game.tile(tileId);
    if (!tile) {
      return;
    }
    this.state.focusedTile = tileId;
    const occupant = this.#game.at(tileId);
    const unit = this.#game.unit(this.state.selected);
    const canMove =
      this.active() &&
      !occupant &&
      unit &&
      this.#game.faction(unit.id).controllable &&
      this.#game.reachable(unit.id).has(tileId);
    if (
      this.state.inspectedTile &&
      (tileId === this.state.inspectedTile ||
        occupant?.id === this.state.selected ||
        occupant?.id === this.state.target)
    ) {
      this.clearSelection(true);
      return;
    }
    this.state.inspectedTile = tileId;
    if (!this.active()) {
      this.publish({ renderBattle: false });
      this.playSelectionFeedback(tileId);
      return;
    }
    try {
      if (
        occupant &&
        unit &&
        this.#game.faction(unit.id).controllable &&
        this.#game.targets(unit.id).includes(occupant)
      ) {
        this.selectTarget(occupant);
        return;
      }
      if (occupant) {
        this.selectUnit(occupant);
        return;
      }
      if (unit && canMove) {
        this.move(unit.id, tileId);
        this.message("Moved. You can attack an enemy in range or select another unit.");
        return;
      }
      this.inspectTerrain(tile);
    } catch (error) {
      this.error(error);
    }
  }

  private selectTarget(target: Unit) {
    this.state.target = target.id;
    this.publish({ gameChanged: false });
    this.animateTileAfterRender(target.tile, "select");
    playSound("select");
    this.message("Tap a weapon to attack. Use the magnifying glass for damage details.");
  }

  private selectUnit(unit: Unit) {
    this.state.selected = unit.id;
    this.state.target = null;
    this.publish({ gameChanged: false });
    this.playSelectionFeedback(unit.tile);

    if (!this.#game.faction(unit.id).controllable) {
      this.message(
        this.#game.faction(unit.id).relation === "neutral"
          ? "Neutral unit selected."
          : "Enemy unit selected. Select one of your units to act.",
      );
    } else if (unit.attacked) {
      this.message("This unit has finished its turn.");
    } else {
      this.message(
        `Selected ${TYPES[unit.type].name.toLowerCase()}. Choose a blue hex to move or a red enemy to attack.`,
      );
    }
  }

  private inspectTerrain(tile: Tile) {
    this.state.selected = null;
    this.state.target = null;
    this.publish({ gameChanged: false });
    this.playSelectionFeedback(tile.id);
    if (this.#game.presentation.fog.includes(tile.id)) {
      this.message("Unexplored terrain.");
      return;
    }
    const terrain = TERRAIN[tile.terrain];
    this.message(`${terrain.name}: ${terrain.description}`);
  }

  move(unitId: string, tileId: string) {
    this.requirePlayerActor(unitId);
    this.history.move(this.#game, unitId, tileId);
    this.state.selected = unitId;
    this.state.inspectedTile = this.#game.unit(unitId)?.tile ?? this.state.inspectedTile;
    this.state.target = null;
    this.commitAction();
    return this.snapshot();
  }

  attack(unitId: string, targetId: string, attackId?: string) {
    return this.resolveAttack(unitId, targetId, attackId, { inspectActor: true });
  }

  private resolveAttack(
    unitId: string,
    targetId: string,
    attackId: string | undefined,
    { inspectActor }: { inspectActor: boolean },
  ) {
    this.requirePlayerActor(unitId);
    this.#game.attack(unitId, targetId, attackId);
    this.history.clear();
    this.state.selected = unitId;
    if (inspectActor) {
      this.state.inspectedTile = this.#game.unit(unitId)?.tile ?? this.state.inspectedTile;
    }
    this.state.target = null;
    this.commitAction();
    return this.snapshot();
  }

  attackSelected(attackId: string) {
    try {
      if (!this.state.selected || !this.state.target) {
        return;
      }
      this.resolveAttack(this.state.selected, this.state.target, attackId, { inspectActor: false });
      if (!this.#game.winner) {
        this.message("Attack complete. Select another unit or end your turn.");
      }
    } catch (error) {
      this.error(error);
    }
  }

  recruit(commanderId: string, type: RecruitType, tileId: string) {
    this.requirePlayerActor(commanderId);
    this.#game.recruit(commanderId, type, tileId);
    this.history.clear();
    this.state.inspectedTile = tileId;
    this.commitAction();
    return this.snapshot();
  }

  recruitHere(type: RecruitType, tileId: string) {
    try {
      const commander = recruitmentContext(this.gameView, null, tileId)?.commander;
      if (!commander) {
        throw new Error("This unit cannot act now.");
      }
      this.recruit(commander.id, type, tileId);
      this.message("Recruited. Your new unit will act next turn.");
    } catch (error) {
      this.error(error);
    }
  }

  advanceUnit(unitId: string, type: UnitType): void {
    try {
      this.advance(unitId, type);
      this.message(`Advanced to ${TYPES[type].name}. Ready next turn.`);
    } catch (error) {
      this.error(error);
    }
  }

  advance(unitId: string, type: UnitType) {
    this.requirePlayerActor(unitId);
    this.#game.advanceUnit(unitId, type);
    this.history.clear();
    this.commitAction();
    return this.snapshot();
  }

  dropItem(unitId: string, itemId: string): void {
    try {
      this.requirePlayerActor(unitId);
      this.#game.dropItem(unitId, itemId);
      this.history.clear();
      this.commitAction();
      this.message("Item dropped. Move away so another unit can collect it.");
    } catch (error) {
      this.error(error);
    }
  }

  async endTurn() {
    this.requireSession();
    if (!this.active()) {
      throw new Error("Wait for your turn.");
    }
    // Resetting or closing the session cancels this turn after its next pause.
    const turnSessionVersion = this.sessionVersion;
    this.history.clear();
    this.state.selected = null;
    this.state.target = null;
    // Interrupted enemy turns replay from this player-state checkpoint.
    const boundary = this.queueFlashSave(true);
    this.#game.endTurn();
    this.publish();
    return this.runEnemyTurn(turnSessionVersion, boundary);
  }

  /** Use the pacing window for the next read-only decision; actions keep their deadlines. */
  private async prepareDuringPause(ms: number, turnSessionVersion: number, publish = false) {
    const waiting = this.pause(ms);
    try {
      if (publish) this.publish();
      await tick();
      if (
        !this.disposed &&
        turnSessionVersion === this.sessionVersion &&
        this.#game.turn === "enemy" &&
        !this.#game.winner
      )
        this.#game.prepareEnemyStep();
    } finally {
      await waiting;
    }
  }

  /** Resolve the enemy turn before publishing its persistent checkpoint. */
  private async runEnemyTurn(
    turnSessionVersion: number,
    boundary: Promise<void> = Promise.resolve(),
  ) {
    this.settlingSession = turnSessionVersion;
    try {
      await boundary;
      if (this.disposed || turnSessionVersion !== this.sessionVersion) return;
      if (!this.#game.winner) this.message("The enemy is planning its moves…");
      await this.prepareDuringPause(enemyThinkingDelay, turnSessionVersion);
      if (turnSessionVersion !== this.sessionVersion) {
        return;
      }
      const recording: EnemyTurnRecording = {
        checkpoint: nativeValue(this.#game.serializedCheckpoint()),
        setup: structuredClone(this.#game.setup),
        commands: [],
        completed: false,
      };
      this.enemyRecordings.push(recording);
      this.enemyRecordings = this.enemyRecordings.slice(-5);
      while (this.#game.turn === "enemy" && !this.#game.winner) {
        recording.commands.push(this.#game.enemyStep());
        if (this.#game.turn !== "enemy" || this.#game.winner) break;
        await this.prepareDuringPause(enemyActionDelay, turnSessionVersion, true);
        if (turnSessionVersion !== this.sessionVersion) return;
      }
      recording.completed = true;
      await this.storageUpdate(async () => {
        if (turnSessionVersion !== this.sessionVersion) return;
        if (!this.#game.winner) await this.turnCheckpoint();
        await this.persist();
      });
      if (turnSessionVersion !== this.sessionVersion) return;
      if (!this.#game.winner)
        this.message("Your turn. Capture villages, recruit reinforcements, or attack.");
      return this.snapshot();
    } finally {
      if (this.settlingSession === turnSessionVersion) {
        this.settlingSession = undefined;
        if (!this.disposed && turnSessionVersion === this.sessionVersion) this.publish();
      }
    }
  }

  reset(mapId = "mirror", options?: { armies?: [string, string]; setup: SessionSetup }) {
    this.requireController();
    // Validate before changing the current battle, its undo history, or its turn.
    if (!BATTLE_MAPS.some((map) => map.id === mapId))
      throw new Error("Unknown battlefield. Choose a skirmish or start a campaign.");
    return this.storageUpdate(() =>
      this.replaceGame(
        LiveGame.create(
          this.nativeFactory,
          { kind: "scenario", map: mapId, armies: options?.armies, first: options?.setup.first },
          options?.setup,
        ),
        true,
      ),
    );
  }

  startCampaign(id: CampaignId, setup: SessionSetup = DEFAULT_SETUP) {
    this.requireController();
    return this.storageUpdate(() =>
      this.replaceGame(LiveGame.create(this.nativeFactory, { kind: "campaign", id }, setup), true),
    );
  }

  nextMission(recallIds: string[]) {
    this.requireSession();
    return this.storageUpdate(() => this.replaceGame(this.#game.nextCampaignMission(recallIds)));
  }

  retryMission() {
    this.requireSession();
    return this.storageUpdate(() => this.replaceGame(this.#game.retryCampaignMission()));
  }

  advanceDialogue(skip = false) {
    this.requireSession();
    if (!dialoguePending(this.#game.campaign)) return;
    this.requireSettled();
    this.#game.advanceDialogue(skip);
    this.commitDialogue();
  }

  previousDialogue() {
    this.requireSession();
    if (!dialoguePending(this.#game.campaign) || this.#game.campaign?.dialogueIndex === 0) return;
    this.requireSettled();
    this.#game.previousDialogue();
    this.commitDialogue();
  }

  private commitDialogue() {
    // An unfinished enemy turn must retain its pre-turn flash save checkpoint.
    if (this.#game.turn === "player" || this.#game.winner) {
      void this.queueFlashSave();
    }
    this.publish({ renderBattle: false, gameChanged: true });
  }

  private async replaceGame(replacement: LiveGame, newRunRequested = false) {
    if (newRunRequested) {
      try {
        await this.finishPendingHistory();
      } catch (error) {
        replacement.dispose();
        throw error;
      }
      this.progress = newRun();
    }
    cancelSound();
    this.sessionVersion++;
    this.history.clear();
    if (this.#game !== replacement) this.#game.dispose();
    this.#game = replacement;
    this.awaitingNewGame = false;
    this.pendingExplicit = undefined;
    this.pendingCheckpoint = undefined;
    this.historyError = null;
    this.historyStart = false;
    this.pendingEnemyReplay = false;
    this.enemyRecordings = [];
    this.previousFeedbackState = null;
    this.state.selected = null;
    this.state.target = null;
    this.state.inspectedTile = null;
    this.state.focusedTile =
      this.#game.units.find((unit) => unit.side === "player")?.tile ?? this.#game.tiles[0]!.id;
    this.state.status = replacement.campaign
      ? "Follow the mission objectives. Keep your leader safe."
      : initialMessage;
    await this.turnCheckpoint(true);
    await this.persist();
    this.publish();
    if (this.#game.turn === "enemy" && !this.#game.winner)
      void this.runEnemyTurn(this.sessionVersion).catch((error) => this.error(error));
    return this.snapshot();
  }

  undoMove() {
    this.requireSession();
    this.requireSettled();
    if (dialoguePending(this.#game.campaign))
      throw new Error("Finish the conversation before undoing a move.");
    const restored = this.history.undo(this.#game);
    if (!restored) {
      return this.snapshot();
    }
    this.state.selected = restored.unitId;
    this.state.target = null;
    this.state.focusedTile = restored.tileId;
    this.state.inspectedTile = restored.tileId;
    this.previousFeedbackState = null;
    this.commitAction();
    this.animateTileAfterRender(restored.tileId, "move");
    this.message("Move undone. Choose a new destination.");
    return this.snapshot();
  }

  actions(unitId: string) {
    const unit = this.#game.unit(unitId);
    if (!unit) {
      throw new Error("Unknown unit.");
    }
    return structuredClone({
      promotions: this.#game.promotionOptions(unitId),
      moves: [...this.#game.reachable(unitId)].map(([tile, cost]) => ({
        tile,
        cost,
      })),
      attacks: this.#game.targets(unitId).map((target) => {
        const { damage, retaliation } = this.#game.previewDetails(unit, target);
        return {
          id: target.id,
          damage,
          retaliation,
          recommendedAttackId: this.#game.bestAttack(unit, target)!.id,
          weapons: this.#game.attackOptions(unit, target).map((attack) => ({
            ...attack,
            ...this.#game.previewDetails(unit, target, attack.id),
          })),
        };
      }),
      recruitment: Object.fromEntries(
        this.#game
          .recruitmentOptions("player")
          .map((type) => [type, this.#game.recruitTiles(unitId, type).map((tile) => tile.id)]),
      ),
    });
  }

  async setSound(enabled: boolean) {
    this.requireSession();
    const requestVersion = ++this.soundRequestVersion;
    const sessionVersion = this.sessionVersion;
    const renderRevision = this.state.renderRevision;
    setSoundEnabled(enabled);
    this.state.sound = enabled;
    this.publish({ renderBattle: false });
    if (enabled) {
      await unlockSound();
      if (
        !this.disposed &&
        requestVersion === this.soundRequestVersion &&
        sessionVersion === this.sessionVersion &&
        renderRevision === this.state.renderRevision
      ) {
        playSound("select");
      }
    }
  }

  initialize(): Promise<void> {
    this.requireController();
    if (this.initialization) return this.initialization;
    const session = this.sessionVersion;
    this.settlingSession = session;
    this.initialization = this.storageUpdate(async () => {
      if (!this.awaitingNewGame && !this.importFailure) {
        if (this.historyStart) await this.turnCheckpoint(!this.pendingEnemyReplay);
        await this.persist(this.pendingEnemyReplay);
      }
    })
      .catch((error) => {
        if (!this.disposed) this.failedSave(error);
      })
      .then(() => {
        if (this.disposed || session !== this.sessionVersion) return;
        this.settlingSession = undefined;
        this.initialized = true;
        this.publish({ renderBattle: false });
        if (this.awaitingNewGame) return;
        if (this.pendingEnemyReplay) {
          this.pendingEnemyReplay = false;
          void this.endTurn().catch((error) => this.error(error));
        } else if (this.#game.turn === "enemy" && !this.#game.winner) {
          void this.runEnemyTurn(this.sessionVersion).catch((error) => this.error(error));
        }
      });
    return this.initialization;
  }
}
