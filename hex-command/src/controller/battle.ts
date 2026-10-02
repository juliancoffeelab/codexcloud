import { writable, type Readable } from "svelte/store";
import { tick, flushSync } from "svelte";
import { Game, TYPES, TERRAIN, type UnitType, type Tile } from "../game/engine";
import { createMoveUndo } from "../game/undo";
import { loadBattle, saveBattle } from "../game/save";
import {
  savedArtStyle,
  saveArtStyle,
  loadArtStyle,
  type ArtStyle,
} from "../presentation/art";
import {
  soundEnabled,
  setSoundEnabled,
  unlockSound,
  playSound,
  playInteraction,
} from "../presentation/sound";
import {
  animateTile,
  animateTurn,
  type MotionKind,
} from "../presentation/motion";
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
const initialMessage =
  "Select a unit. Capture villages. Defeat the red commander.";
const delay = (ms: number) =>
  new Promise<void>((resolve) => setTimeout(resolve, ms));
export class BattleController {
  private restored = loadBattle();
  private history = createMoveUndo();
  private epoch = 0;
  private disposed = false;
  private previous: ReturnType<Game["snapshot"]> | null = null;
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
      focusedTile:
        game.units.find((u) => u.side === "player")?.tile ?? game.tiles[0]!.id,
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
    this.previous = game.snapshot();
  }
  /** Stop outstanding turn and preference work when this play session unmounts. */
  dispose(): void {
    this.disposed = true;
    this.epoch++;
  }
  private requireSession(): void {
    if (this.disposed) throw new Error("This battle session has closed.");
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
    this.publish(false, false);
  }
  error(error: unknown) {
    this.message(error instanceof Error ? error.message : String(error));
  }
  private publish(feedback = true, winnerMessage = true) {
    if (this.disposed) return;
    if (feedback) this.state.renderRevision++;
    if (winnerMessage && this.game.winner)
      this.state.status =
        this.game.winner === "player"
          ? "Victory! The enemy commander has fallen. The field is yours."
          : "Your commander has fallen. Start a new battle and try a different approach.";
    const focusWasMap =
      feedback &&
      typeof document !== "undefined" &&
      document.activeElement?.closest("[data-tile]");
    // A fresh render snapshot keeps mutable engine state outside Svelte.
    // Status and preference updates reuse it until the next battle render.
    if (feedback) {
      this.gameView = new Game(this.game.mapId, this.game.mapRevision);
      this.gameView.restoreState(this.game.captureState());
    }
    flushSync(() => this.store.set({ ...this.state, game: this.gameView }));
    if (focusWasMap)
      document
        .querySelector<SVGGElement>(
          `#map [data-tile="${this.state.focusedTile}"]`,
        )
        ?.focus({ preventScroll: true });
    if (feedback) this.feedback();
  }
  private committed() {
    if (this.game.turn === "player") saveBattle(this.game);
    this.publish();
  }
  private pulse(tile: string, kind: MotionKind) {
    void tick().then(() => {
      if (!this.disposed) animateTile(tile, kind);
    });
  }
  private selectionFeedback(tileId: string) {
    this.pulse(tileId, "select");
    playInteraction({
      unitType: this.game.at(tileId)?.type,
      terrain: this.game.tile(tileId)?.terrain,
    });
  }
  private feedback() {
    const now = this.game.snapshot(),
      before = this.previous;
    this.previous = now;
    if (!before) return;
    const oldUnits = new Map(before.units.map((unit) => [unit.id, unit]));
    const effects = new Map<string, MotionKind>();
    let moved = false,
      recruited = false,
      attacked = false,
      captured = false;
    let movingUnit: (typeof now.units)[number] | null = null;
    for (const unit of now.units) {
      const old = oldUnits.get(unit.id);
      if (!old) {
        effects.set(unit.tile, "recruit");
        recruited = true;
      } else {
        if (old.tile !== unit.tile) {
          effects.set(unit.tile, "move");
          moved = true;
          movingUnit = unit;
        }
        if (old.hp > unit.hp) {
          effects.set(unit.tile, "attack");
          attacked = true;
        }
      }
      oldUnits.delete(unit.id);
    }
    for (const fallen of oldUnits.values()) {
      effects.set(fallen.tile, "attack");
      attacked = true;
    }
    for (const [tile, kind] of effects) this.pulse(tile, kind);
    for (const village of now.villages)
      if (
        village.owner &&
        village.owner !==
          before.villages.find((tile) => tile.id === village.id)?.owner
      ) {
        this.pulse(village.id, "capture");
        captured = true;
      }
    if (before.turn !== now.turn || before.round !== now.round)
      void tick().then(() => { if (!this.disposed) animateTurn(); });
    if (attacked) {
      const attacker =
        now.units.find(
          (unit) =>
            unit.side === before.turn &&
            unit.attacked &&
            before.units.some((old) => old.id === unit.id && !old.attacked),
        ) ||
        before.units.find(
          (unit) =>
            unit.side === before.turn &&
            !now.units.some((alive) => alive.id === unit.id),
        );
      playSound(
        this.game.lastCombat?.sound ||
          (attacker?.type === "archer" ? "attack-bow" : "attack-melee"),
      );
    } else if (now.winner && now.winner !== before.winner)
      playSound(now.winner === "player" ? "victory" : "defeat");
    else if (recruited) playSound("recruit");
    else if (moved && movingUnit)
      playInteraction({
        unitType: movingUnit.type,
        terrain: this.game.tile(movingUnit.tile)?.terrain,
      });
    else if (captured) playInteraction({ terrain: "village" });
  }
  recruitmentContext(tile: Tile | undefined) {
    if (
      !tile ||
      tile.terrain === "water" ||
      this.game.at(tile.id) ||
      this.game.winner ||
      this.game.unit(this.state.selected)
    )
      return null;
    const keep = this.game
      .neighbors(tile.id)
      .find(
        (neighbor) =>
          neighbor.terrain === "keep" && neighbor.owner === "player",
      );
    if (!keep) return null;
    const commander = this.game.units.find(
      (unit) => unit.side === "player" && unit.type === "commander",
    );
    return { commander, stationed: commander?.tile === keep.id };
  }
  focusTile(tileId: string) {
    this.state.focusedTile = tileId;
    this.publish(false);
  }
  clearSelection(clearInspection = false) {
    if (
      !this.state.selected &&
      !this.state.target &&
      !(clearInspection && this.state.inspectedTile)
    )
      return;
    this.state.selected = null;
    this.state.target = null;
    if (clearInspection) this.state.inspectedTile = null;
    this.publish();
    this.message("Selection cleared.");
  }
  choose(tileId: string) {
    const tile = this.game.tile(tileId);
    if (!tile) return;
    this.state.inspectedTile = tileId;
    this.state.focusedTile = tileId;
    if (!this.active()) {
      this.publish(false);
      this.selectionFeedback(tileId);
      return;
    }
    try {
      const occupant = this.game.at(tileId),
        unit = this.game.unit(this.state.selected);
      if (
        occupant &&
        (occupant.id === this.state.selected ||
          occupant.id === this.state.target)
      ) {
        this.clearSelection();
        this.selectionFeedback(tileId);
        return;
      }
      if (
        occupant?.side === "enemy" &&
        unit?.side === "player" &&
        this.game.targets(unit.id).includes(occupant)
      ) {
        this.state.target = occupant.id;
        this.publish();
        this.pulse(tileId, "select");
        playSound("select");
        this.message(
          "Tap a weapon to attack. Use the magnifying glass for damage details.",
        );
        if (matchMedia("(max-width:900px)").matches)
          void tick().then(() =>
            document
              .getElementById("unit-panel")
              ?.scrollIntoView({ block: "nearest" }),
          );
        return;
      }
      if (occupant) {
        this.state.selected = occupant.id;
        this.state.target = null;
        this.publish();
        this.selectionFeedback(tileId);
        this.message(
          occupant.side === "player"
            ? occupant.attacked
              ? "This unit has finished its turn."
              : `Selected ${TYPES[occupant.type].name.toLowerCase()}. Choose a blue hex to move or a red enemy to attack.`
            : "Enemy unit selected. Select one of your units to act.",
        );
        return;
      }
      if (unit?.side === "player" && this.game.reachable(unit.id).has(tileId)) {
        this.move(unit.id, tileId);
        this.message(
          "Moved. You can attack an enemy in range or select another unit.",
        );
        return;
      }
      this.state.selected = null;
      this.state.target = null;
      this.publish();
      this.selectionFeedback(tileId);
      this.message(
        `${TERRAIN[tile.terrain].name}: ${TERRAIN[tile.terrain].description}`,
      );
    } catch (error) {
      this.error(error);
    }
  }
  move(unitId: string, tileId: string) {
    this.requireSession();
    this.history.move(this.game, unitId, tileId);
    this.state.selected = unitId;
    this.state.inspectedTile =
      this.game.unit(unitId)?.tile ?? this.state.inspectedTile;
    this.state.target = null;
    this.committed();
    return this.snapshot();
  }
  attack(unitId: string, targetId: string, attackId?: string) {
    return this.resolveAttack(unitId, targetId, attackId, true);
  }
  private resolveAttack(
    unitId: string,
    targetId: string,
    attackId: string | undefined,
    inspectActor: boolean,
  ) {
    this.requireSession();
    this.game.attack(unitId, targetId, attackId);
    this.history.clear();
    this.state.selected = unitId;
    if (inspectActor)
      this.state.inspectedTile =
        this.game.unit(unitId)?.tile ?? this.state.inspectedTile;
    this.state.target = null;
    this.committed();
    return this.snapshot();
  }
  attackSelected(attackId: string) {
    try {
      if (!this.state.selected || !this.state.target) return;
      this.resolveAttack(
        this.state.selected,
        this.state.target,
        attackId,
        false,
      );
      if (!this.game.winner)
        this.message("Attack complete. Select another unit or end your turn.");
    } catch (error) {
      this.error(error);
    }
  }
  recruit(commanderId: string, type: RecruitType, tileId: string) {
    this.requireSession();
    this.game.recruit(commanderId, type, tileId);
    this.history.clear();
    this.state.inspectedTile = tileId;
    this.committed();
    return this.snapshot();
  }
  recruitHere(type: RecruitType, tileId: string) {
    try {
      const commander = this.game.units.find(
        (unit) => unit.side === "player" && unit.type === "commander",
      );
      if (!commander) throw new Error("This unit cannot act now.");
      this.recruit(commander.id, type, tileId);
      this.message("Recruited. Your new unit will act next turn.");
    } catch (error) {
      this.error(error);
    }
  }
  async endTurn() {
    this.requireSession();
    if (!this.active()) throw new Error("Wait for your turn.");
    const currentEpoch = this.epoch;
    this.history.clear();
    this.state.selected = null;
    this.state.target = null;
    saveBattle(this.game, true);
    this.game.endTurn();
    this.publish();
    this.message("The enemy is planning its moves…");
    await delay(350);
    if (currentEpoch !== this.epoch) return;
    this.game.enemyRecruit();
    this.publish();
    const order: Record<UnitType, number> = {
      scout: 0,
      archer: 1,
      soldier: 2,
      commander: 3,
    };
    const queue = this.game.units
      .filter((u) => u.side === "enemy")
      .sort((a, b) => order[a.type] - order[b.type])
      .map((u) => u.id);
    for (const id of queue) {
      await delay(230);
      if (currentEpoch !== this.epoch) return;
      this.game.enemyAct(id);
      this.publish();
      if (this.game.winner) break;
    }
    if (!this.game.winner) {
      this.game.endTurn();
      this.publish();
      this.message(
        "Your turn. Capture villages, recruit reinforcements, or attack.",
      );
    }
    saveBattle(this.game);
    return this.snapshot();
  }
  reset(mapId = "mirror") {
    this.requireSession();
    const replacement = new Game(mapId);
    this.epoch++;
    this.history.clear();
    this.state.game = replacement;
    this.previous = null;
    this.state.selected = null;
    this.state.target = null;
    this.state.inspectedTile = null;
    this.state.focusedTile =
      this.game.units.find((unit) => unit.side === "player")?.tile ??
      this.game.tiles[0]!.id;
    this.state.status = initialMessage;
    this.committed();
    return this.snapshot();
  }
  undoMove() {
    this.requireSession();
    const restored = this.history.undo(this.game);
    if (!restored) return this.snapshot();
    this.state.selected = restored.unitId;
    this.state.target = null;
    this.state.focusedTile = restored.tileId;
    this.state.inspectedTile = restored.tileId;
    this.previous = null;
    this.committed();
    this.pulse(restored.tileId, "move");
    this.message("Move undone. Choose a new destination.");
    return this.snapshot();
  }
  actions(unitId: string) {
    const unit = this.game.unit(unitId);
    if (!unit) throw new Error("Unknown unit.");
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
    this.publish(false);
    if (enabled) {
      await unlockSound();
      if (!this.disposed) playSound("select");
    }
  }
  async setArtStyle(next: ArtStyle) {
    this.requireSession();
    this.state.artLoading = true;
    this.publish(false);
    const ready = await loadArtStyle(next);
    if (this.disposed) return;
    if (ready) {
      this.state.artStyle = next;
      saveArtStyle(next);
      document.documentElement.dataset.artStyle = next;
      this.publish();
    } else this.message("Could not load that artwork. Please try again.");
    this.state.artLoading = false;
    this.publish(false);
  }
  initialize() {
    this.requireSession();
    document.documentElement.dataset.artStyle = this.state.artStyle;
    if (this.game.turn === "player") saveBattle(this.game);
    this.publish(false);
    if (this.restored.pendingEnemy)
      void this.endTurn().catch((error) => this.error(error));
    const initial = this.state.artStyle;
    void loadArtStyle(initial).then(async (ready) => {
      if (
        this.disposed ||
        ready ||
        this.state.artStyle !== initial ||
        this.state.artLoading
      )
        return;
      const fallbacks: ArtStyle[] = ["normal", "low-poly", "pixel"];
      for (const fallback of fallbacks.filter((style) => style !== initial))
        if (await loadArtStyle(fallback)) {
          if (this.disposed) return;
          if (this.state.artStyle === initial && !this.state.artLoading) {
            this.state.artStyle = fallback;
            document.documentElement.dataset.artStyle = fallback;
            this.publish();
            this.message(
              "Using alternate artwork. Retry your preferred style in Settings.",
            );
          }
          return;
        }
      if (!this.disposed)
        this.message("Artwork could not load. Please reload to retry.");
    });
  }
}
