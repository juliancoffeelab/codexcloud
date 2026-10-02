import { writable, type Readable } from "svelte/store";
import { tick, flushSync } from "svelte";
import { Game, TYPES, TERRAIN, type UnitType, type Tile, type Unit } from "../game/engine";
import { createMoveUndo } from "../game/undo";
import { loadBattle, saveBattle } from "../game/save";
import { savedArtStyle, saveArtStyle, loadArtStyle, type ArtStyle } from "../presentation/art";
import {
  soundEnabled,
  setSoundEnabled,
  unlockSound,
  playSound,
  playInteraction,
} from "../presentation/sound";
import { animateTile, animateTurn, type MotionKind } from "../presentation/motion";
import { describeChanges } from "./feedback";

interface PublishOptions {
  renderBattle?: boolean;
  showWinnerMessage?: boolean;
}

export type RecruitType = Exclude<UnitType, "commander">;
export const recruitTypes: RecruitType[] = ["soldier", "archer", "scout"];

export interface BattleView {
  game: Game;
  selected: string | null;
  target: string | null;
  focusedTile: string;
  inspectedTile: string | null;
  renderRevision: number;
  status: string;
  artStyle: ArtStyle;
  artLoading: boolean;
  sound: boolean;
}

const initialMessage = "Select a unit. Capture villages. Defeat the red commander.";
const delay = (ms: number) => new Promise<void>((resolve) => setTimeout(resolve, ms));
const enemyThinkingDelay = 350;
const enemyActionDelay = 230;
const enemyActionOrder: Record<UnitType, number> = {
  scout: 0,
  archer: 1,
  soldier: 2,
  commander: 3,
};

export class BattleController {
  private readonly restored = loadBattle();
  private readonly history = createMoveUndo();
  private sessionVersion = 0;
  private disposed = false;
  private previousFeedbackState: ReturnType<Game["snapshot"]> | null = null;
  private state: BattleView;
  private store;
  private gameView: Game;
  readonly view: Readable<BattleView>;

  constructor() {
    const game = this.restored.game;
    this.state = {
      game,
      selected: null,
      target: null,
      focusedTile: game.units.find((u) => u.side === "player")?.tile ?? game.tiles[0]!.id,
      inspectedTile: null,
      renderRevision: 0,
      status: initialMessage,
      artStyle: savedArtStyle(),
      artLoading: false,
      sound: soundEnabled(),
    };
    this.gameView = new Game(game.mapId, game.mapRevision);
    this.gameView.restoreState(game.captureState());
    this.store = writable({ ...this.state, game: this.gameView });
    this.view = { subscribe: this.store.subscribe };
    this.previousFeedbackState = game.snapshot();
  }

  /** Stop outstanding turn and preference work when this play session unmounts. */
  dispose(): void {
    this.disposed = true;
    this.sessionVersion++;
  }

  private requireSession(): void {
    if (this.disposed) {
      throw new Error("This battle session has closed.");
    }
  }

  get game() {
    return this.state.game;
  }

  active() {
    return this.game.turn === "player" && !this.game.winner;
  }

  canUndo() {
    return this.history.canUndo(this.game);
  }

  snapshot() {
    return this.game.snapshot();
  }

  message(status: string) {
    this.state.status = status;
    this.publish({ renderBattle: false, showWinnerMessage: false });
  }

  error(error: unknown) {
    this.message(error instanceof Error ? error.message : String(error));
  }

  private publish({ renderBattle = true, showWinnerMessage = true }: PublishOptions = {}) {
    if (this.disposed) {
      return;
    }
    if (renderBattle) {
      this.state.renderRevision++;
    }
    if (showWinnerMessage && this.game.winner) {
      this.state.status =
        this.game.winner === "player"
          ? "Victory! The enemy commander has fallen. The field is yours."
          : "Your commander has fallen. Start a new battle and try a different approach.";
    }
    const focusWasMap =
      renderBattle &&
      typeof document !== "undefined" &&
      document.activeElement?.closest("[data-tile]");
    // A fresh render snapshot keeps mutable engine state outside Svelte.
    // Status and preference updates reuse it until the next battle render.
    if (renderBattle) {
      this.gameView = new Game(this.game.mapId, this.game.mapRevision);
      this.gameView.restoreState(this.game.captureState());
    }
    flushSync(() => this.store.set({ ...this.state, game: this.gameView }));
    if (focusWasMap) {
      document
        .querySelector<SVGGElement>(`#map [data-tile="${this.state.focusedTile}"]`)
        ?.focus({ preventScroll: true });
    }
    if (renderBattle) {
      this.playBattleFeedback();
    }
  }

  private commitAction() {
    if (this.game.turn === "player") {
      saveBattle(this.game);
    }
    this.publish();
  }

  private animateTileAfterRender(tile: string, kind: MotionKind) {
    void tick().then(() => {
      if (!this.disposed) {
        animateTile(tile, kind);
      }
    });
  }

  private playSelectionFeedback(tileId: string) {
    this.animateTileAfterRender(tileId, "select");
    playInteraction({
      unitType: this.game.at(tileId)?.type,
      terrain: this.game.tile(tileId)?.terrain,
    });
  }

  private playBattleFeedback() {
    const current = this.game.snapshot();
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
      void tick().then(() => {
        if (!this.disposed) {
          animateTurn();
        }
      });
    }

    // One cue per action, with combat taking precedence over movement/capture.
    if (changes.attacked) {
      const defaultSound = changes.attacker?.type === "archer" ? "attack-bow" : "attack-melee";
      playSound(this.game.lastCombat?.sound || defaultSound);
    } else if (changes.newWinner) {
      playSound(changes.newWinner === "player" ? "victory" : "defeat");
    } else if (changes.recruited) {
      playSound("recruit");
    } else if (changes.movedUnit) {
      playInteraction({
        unitType: changes.movedUnit.type,
        terrain: this.game.tile(changes.movedUnit.tile)?.terrain,
      });
    } else if (changes.capturedTiles.length > 0) {
      playInteraction({ terrain: "village" });
    }
  }

  recruitmentContext(tile: Tile | undefined) {
    if (
      !tile ||
      tile.terrain === "water" ||
      this.game.at(tile.id) ||
      this.game.winner ||
      this.game.unit(this.state.selected)
    ) {
      return null;
    }
    const keep = this.game
      .neighbors(tile.id)
      .find((neighbor) => neighbor.terrain === "keep" && neighbor.owner === "player");
    if (!keep) {
      return null;
    }
    const commander = this.game.units.find(
      (unit) => unit.side === "player" && unit.type === "commander",
    );
    return { commander, stationed: commander?.tile === keep.id };
  }

  focusTile(tileId: string) {
    this.state.focusedTile = tileId;
    this.publish({ renderBattle: false });
  }

  clearSelection(clearInspection = false) {
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
    this.publish();
    this.message("Selection cleared.");
  }

  choose(tileId: string) {
    const tile = this.game.tile(tileId);
    if (!tile) {
      return;
    }
    this.state.inspectedTile = tileId;
    this.state.focusedTile = tileId;
    if (!this.active()) {
      this.publish({ renderBattle: false });
      this.playSelectionFeedback(tileId);
      return;
    }
    try {
      const occupant = this.game.at(tileId);
      const unit = this.game.unit(this.state.selected);
      if (occupant && (occupant.id === this.state.selected || occupant.id === this.state.target)) {
        this.clearSelection();
        this.playSelectionFeedback(tileId);
        return;
      }
      if (
        occupant?.side === "enemy" &&
        unit?.side === "player" &&
        this.game.targets(unit.id).includes(occupant)
      ) {
        this.selectTarget(occupant);
        return;
      }
      if (occupant) {
        this.selectUnit(occupant);
        return;
      }
      if (unit?.side === "player" && this.game.reachable(unit.id).has(tileId)) {
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
    this.publish();
    this.animateTileAfterRender(target.tile, "select");
    playSound("select");
    this.message("Tap a weapon to attack. Use the magnifying glass for damage details.");
    if (matchMedia("(max-width:900px)").matches) {
      void tick().then(() => {
        document.getElementById("unit-panel")?.scrollIntoView({ block: "nearest" });
      });
    }
  }

  private selectUnit(unit: Unit) {
    this.state.selected = unit.id;
    this.state.target = null;
    this.publish();
    this.playSelectionFeedback(unit.tile);

    if (unit.side === "enemy") {
      this.message("Enemy unit selected. Select one of your units to act.");
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
    this.publish();
    this.playSelectionFeedback(tile.id);
    const terrain = TERRAIN[tile.terrain];
    this.message(`${terrain.name}: ${terrain.description}`);
  }

  move(unitId: string, tileId: string) {
    this.requireSession();
    this.history.move(this.game, unitId, tileId);
    this.state.selected = unitId;
    this.state.inspectedTile = this.game.unit(unitId)?.tile ?? this.state.inspectedTile;
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
    this.requireSession();
    this.game.attack(unitId, targetId, attackId);
    this.history.clear();
    this.state.selected = unitId;
    if (inspectActor) {
      this.state.inspectedTile = this.game.unit(unitId)?.tile ?? this.state.inspectedTile;
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
      if (!this.game.winner) {
        this.message("Attack complete. Select another unit or end your turn.");
      }
    } catch (error) {
      this.error(error);
    }
  }

  recruit(commanderId: string, type: RecruitType, tileId: string) {
    this.requireSession();
    this.game.recruit(commanderId, type, tileId);
    this.history.clear();
    this.state.inspectedTile = tileId;
    this.commitAction();
    return this.snapshot();
  }

  recruitHere(type: RecruitType, tileId: string) {
    try {
      const commander = this.game.units.find(
        (unit) => unit.side === "player" && unit.type === "commander",
      );
      if (!commander) {
        throw new Error("This unit cannot act now.");
      }
      this.recruit(commander.id, type, tileId);
      this.message("Recruited. Your new unit will act next turn.");
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
    saveBattle(this.game, true);
    this.game.endTurn();
    this.publish();
    this.message("The enemy is planning its moves…");
    await delay(enemyThinkingDelay);
    if (turnSessionVersion !== this.sessionVersion) {
      return;
    }
    this.game.enemyRecruit();
    this.publish();
    const queue = this.game.units
      .filter((u) => u.side === "enemy")
      .sort((a, b) => enemyActionOrder[a.type] - enemyActionOrder[b.type])
      .map((u) => u.id);
    for (const id of queue) {
      await delay(enemyActionDelay);
      if (turnSessionVersion !== this.sessionVersion) {
        return;
      }
      this.game.enemyAct(id);
      this.publish();
      if (this.game.winner) {
        break;
      }
    }
    if (!this.game.winner) {
      this.game.endTurn();
      this.publish();
      this.message("Your turn. Capture villages, recruit reinforcements, or attack.");
    }
    saveBattle(this.game);
    return this.snapshot();
  }

  reset(mapId = "mirror") {
    this.requireSession();
    // Validate before changing the current battle, its undo history, or its turn.
    const replacement = new Game(mapId);
    this.sessionVersion++;
    this.history.clear();
    this.state.game = replacement;
    this.previousFeedbackState = null;
    this.state.selected = null;
    this.state.target = null;
    this.state.inspectedTile = null;
    this.state.focusedTile =
      this.game.units.find((unit) => unit.side === "player")?.tile ?? this.game.tiles[0]!.id;
    this.state.status = initialMessage;
    this.commitAction();
    return this.snapshot();
  }

  undoMove() {
    this.requireSession();
    const restored = this.history.undo(this.game);
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
    const unit = this.game.unit(unitId);
    if (!unit) {
      throw new Error("Unknown unit.");
    }
    return {
      moves: [...this.game.reachable(unitId)].map(([tile, cost]) => ({
        tile,
        cost,
      })),
      attacks: this.game.targets(unitId).map((target) => ({
        id: target.id,
        ...this.game.preview(unit, target),
        recommendedAttackId: this.game.bestAttack(unit, target)!.id,
        weapons: this.game.attackOptions(unit, target).map((attack) => ({
          ...attack,
          ...this.game.previewDetails(unit, target, attack.id),
        })),
      })),
      recruitment: Object.fromEntries(
        recruitTypes.map((type) => [
          type,
          this.game.recruitTiles(unitId, type).map((tile) => tile.id),
        ]),
      ),
    };
  }

  async setSound(enabled: boolean) {
    this.requireSession();
    setSoundEnabled(enabled);
    this.state.sound = enabled;
    this.publish({ renderBattle: false });
    if (enabled) {
      await unlockSound();
      if (!this.disposed) {
        playSound("select");
      }
    }
  }

  async setArtStyle(next: ArtStyle) {
    this.requireSession();
    this.state.artLoading = true;
    this.publish({ renderBattle: false });
    const ready = await loadArtStyle(next);
    if (this.disposed) {
      return;
    }
    if (ready) {
      this.state.artStyle = next;
      saveArtStyle(next);
      document.documentElement.dataset.artStyle = next;
      this.publish();
    } else {
      this.message("Could not load that artwork. Please try again.");
    }
    this.state.artLoading = false;
    this.publish({ renderBattle: false });
  }

  initialize() {
    this.requireSession();
    document.documentElement.dataset.artStyle = this.state.artStyle;
    if (this.game.turn === "player") {
      saveBattle(this.game);
    }
    this.publish({ renderBattle: false });
    if (this.restored.pendingEnemy) {
      void this.endTurn().catch((error) => this.error(error));
    }
    void this.loadInitialArtwork(this.state.artStyle);
  }

  private async loadInitialArtwork(initialStyle: ArtStyle) {
    const ready = await loadArtStyle(initialStyle);
    if (this.disposed || ready || this.state.artStyle !== initialStyle || this.state.artLoading) {
      return;
    }

    const fallbacks: ArtStyle[] = ["normal", "low-poly", "pixel"];
    for (const fallback of fallbacks.filter((style) => style !== initialStyle)) {
      if (await loadArtStyle(fallback)) {
        if (this.disposed) {
          return;
        }
        if (this.state.artStyle === initialStyle && !this.state.artLoading) {
          this.state.artStyle = fallback;
          document.documentElement.dataset.artStyle = fallback;
          this.publish();
          this.message("Using alternate artwork. Retry your preferred style in Settings.");
        }
        return;
      }
    }
    if (!this.disposed) {
      this.message("Artwork could not load. Please reload to retry.");
    }
  }
}
