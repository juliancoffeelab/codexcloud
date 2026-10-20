import { BATTLE_MAPS } from "../game/maps";
import { dialoguePending } from "../game/campaign-progress";
import type { CampaignId, UnitType } from "../game/types";
import { writable, type Readable, type Writable } from "svelte/store";
import { tick } from "svelte";
import { TYPES, TERRAIN, type Tile, type Unit } from "../game/catalog";
import { createMoveUndo } from "../game/undo";
import { loadBattle } from "../save/load";
import { saveBattle } from "../save/write";
import { LiveGame } from "../game/live-game";
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

const initialMessage = "Select a unit. Capture villages. Defeat the red commander.";
const delay = (ms: number) => new Promise<void>((resolve) => setTimeout(resolve, ms));
const enemyThinkingDelay = 350;
const enemyActionDelay = 230;

export class BattleController {
  #game: LiveGame;
  private pendingEnemyReplay: boolean;
  private initialized = false;
  private soundRequestVersion = 0;
  private readonly history = createMoveUndo();
  private sessionVersion = 0;
  private disposed = false;
  private previousFeedbackState: ReturnType<LiveGame["snapshot"]> | null = null;
  private state: SessionState;
  private store: Writable<BattleView>;
  private gameView: GameView;
  readonly view: Readable<BattleView>;
  readonly commands: BattleCommands;

  constructor(
    private readonly pause: (ms: number) => Promise<void> = delay,
    private readonly nativeFactory: GameEngineFactory,
  ) {
    const restored = loadBattle(nativeFactory);
    const game = restored.game;
    this.#game = game;
    this.pendingEnemyReplay = restored.pendingEnemy;
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
    this.previousFeedbackState = game.snapshot();
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

  private requireSession(): void {
    if (this.disposed) {
      throw new Error("This battle session has closed.");
    }
  }

  private active() {
    return (
      this.#game.turn === "player" && !this.#game.winner && !dialoguePending(this.#game.campaign)
    );
  }

  private requirePlayerActor(unitId: string): void {
    this.requireSession();
    if (!this.active()) {
      throw new Error("Wait for your turn.");
    }
    if (this.#game.unit(unitId)?.side !== "player") {
      throw new Error("This unit cannot act now.");
    }
  }

  private currentView(): BattleView {
    return Object.freeze({
      ...this.state,
      game: this.gameView,
      combat: this.#game.combat,
      artwork: currentArtwork(),
      canUndo: !dialoguePending(this.#game.campaign) && this.history.canUndo(this.#game),
      recruitment: recruitmentContext(this.gameView, this.state.selected, this.state.inspectedTile),
    });
  }

  snapshot() {
    return this.#game.snapshot();
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

  private commitAction() {
    if (this.#game.turn === "player") {
      saveBattle(this.#game);
    }
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
    playInteraction({
      unitType: this.#game.at(tileId)?.type,
      terrain: this.#game.tile(tileId)?.terrain,
    });
  }

  private playBattleFeedback() {
    const current = this.#game.snapshot();
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
      playInteraction({
        unitType: changes.movedUnit.type,
        terrain: this.#game.tile(changes.movedUnit.tile)?.terrain,
      });
    } else if (changes.capturedTiles.length > 0) {
      playInteraction({ terrain: "village" });
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
      unit?.side === "player" &&
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
        occupant?.side === "enemy" &&
        unit?.side === "player" &&
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
    this.publish({ gameChanged: false });
    this.playSelectionFeedback(tile.id);
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
      const commander = this.#game.units.find(
        (unit) => unit.side === "player" && TYPES[unit.type].leader,
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
    saveBattle(this.#game, true);
    this.#game.endTurn();
    this.publish();
    return this.runEnemyTurn(turnSessionVersion);
  }

  /** Resolve the enemy turn before publishing its persistent checkpoint. */
  private async runEnemyTurn(turnSessionVersion: number) {
    if (!this.#game.winner) this.message("The enemy is planning its moves…");
    await this.pause(enemyThinkingDelay);
    if (turnSessionVersion !== this.sessionVersion) {
      return;
    }
    while (this.#game.turn === "enemy" && !this.#game.winner) {
      this.#game.enemyStep();
      this.publish();
      if (this.#game.turn !== "enemy" || this.#game.winner) break;
      await this.pause(enemyActionDelay);
      if (turnSessionVersion !== this.sessionVersion) return;
    }
    if (!this.#game.winner)
      this.message("Your turn. Capture villages, recruit reinforcements, or attack.");
    saveBattle(this.#game);
    return this.snapshot();
  }

  reset(mapId = "mirror") {
    this.requireSession();
    // Validate before changing the current battle, its undo history, or its turn.
    if (!BATTLE_MAPS.some((map) => map.id === mapId))
      throw new Error("Unknown battlefield. Choose a skirmish or start a campaign.");
    return this.replaceGame(LiveGame.create(this.nativeFactory, { kind: "scenario", map: mapId }));
  }

  startCampaign(id: CampaignId) {
    this.requireSession();
    return this.replaceGame(LiveGame.create(this.nativeFactory, { kind: "campaign", id }));
  }

  nextMission(recallIds: string[]) {
    this.requireSession();
    return this.replaceGame(this.#game.nextCampaignMission(recallIds));
  }

  retryMission() {
    this.requireSession();
    return this.replaceGame(this.#game.retryCampaignMission());
  }

  advanceDialogue(skip = false) {
    this.requireSession();
    if (!dialoguePending(this.#game.campaign)) return;
    this.#game.advanceDialogue(skip);
    this.commitDialogue();
  }

  previousDialogue() {
    this.requireSession();
    if (!dialoguePending(this.#game.campaign) || this.#game.campaign?.dialogueIndex === 0) return;
    this.#game.previousDialogue();
    this.commitDialogue();
  }

  private commitDialogue() {
    // An unfinished enemy turn must retain its pre-turn recovery checkpoint.
    if (this.#game.turn === "player" || this.#game.winner) saveBattle(this.#game);
    this.publish({ renderBattle: false, gameChanged: true });
  }

  private replaceGame(replacement: LiveGame) {
    cancelSound();
    this.sessionVersion++;
    this.history.clear();
    if (this.#game !== replacement) this.#game.dispose();
    this.#game = replacement;
    this.pendingEnemyReplay = false;
    this.previousFeedbackState = null;
    this.state.selected = null;
    this.state.target = null;
    this.state.inspectedTile = null;
    this.state.focusedTile =
      this.#game.units.find((unit) => unit.side === "player")?.tile ?? this.#game.tiles[0]!.id;
    this.state.status = replacement.campaign
      ? "Follow the mission objectives. Keep your leader safe."
      : initialMessage;
    this.commitAction();
    return this.snapshot();
  }

  undoMove() {
    this.requireSession();
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

  initialize() {
    this.requireSession();
    if (this.initialized) {
      return;
    }
    this.initialized = true;
    if (this.#game.turn === "player") {
      saveBattle(this.#game);
    }
    this.publish({ renderBattle: false });
    if (this.pendingEnemyReplay) {
      this.pendingEnemyReplay = false;
      void this.endTurn().catch((error) => this.error(error));
    } else if (this.#game.turn === "enemy" && !this.#game.winner) {
      void this.runEnemyTurn(this.sessionVersion).catch((error) => this.error(error));
    }
  }
}
