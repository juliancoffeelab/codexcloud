import { cellKey } from "../game/hex";
import type { BattleView } from "./view";
import type { GameView } from "../game/view";
import { immutable } from "../game/immutable";

export interface CombatPlaybackFrame {
  readonly before: BattleView;
  readonly combat: NonNullable<BattleView["combat"]>;
  readonly shown: number;
  readonly complete: boolean;
  readonly game: GameView;
  readonly advance: (index: number) => boolean;
  readonly finish: () => void;
  readonly skip: () => void;
}

/** Published combat phases. Phaser advances them; Rust has already committed the result. */
export class CombatPlayback {
  frame: CombatPlaybackFrame;
  readonly finished: Promise<void>;
  private release!: () => void;
  private cancelled = false;

  constructor(
    before: BattleView,
    combat: NonNullable<BattleView["combat"]>,
    private readonly resolved: GameView,
    private readonly publish: (frame: CombatPlaybackFrame) => void,
    reducedMotion: boolean,
  ) {
    this.finished = new Promise((resolve) => (this.release = resolve));
    this.frame = {
      before,
      combat,
      shown: 0,
      complete: false,
      game: before.game,
      advance: (index) => this.advance(index),
      finish: () => this.complete(),
      skip: () => this.complete(true),
    };
    if (reducedMotion || !combat.strikes.length) {
      this.frame = { ...this.frame, shown: combat.strikes.length, complete: true, game: resolved };
      this.release();
    }
  }

  private advance(index: number): boolean {
    if (this.cancelled || this.frame.complete || index !== this.frame.shown) return false;
    const { before, combat, shown } = this.frame;
    const hit = combat.strikes[shown];
    if (!hit) return false;
    const units = before.game.units.flatMap((unit) => {
      const index = combat.units.findIndex((actor) => actor.id === unit.id);
      if (index < 0) return [unit];
      const hp = hit.health[index]!;
      return hp > 0 ? [{ ...unit, hp }] : [];
    });
    // Keep pre-combat fog, loot, objectives and victory hidden until completion.
    const game: GameView = immutable({
      ...before.game,
      units,
      unit: (id: string | null | undefined) => units.find((unit) => unit.id === id),
      at: (tile: Parameters<GameView["at"]>[0]) =>
        tile == null ? undefined : units.find((unit) => cellKey(unit.tile) === cellKey(tile)),
    });
    this.frame = { ...this.frame, shown: shown + 1, game };
    this.publish(this.frame);
    return true;
  }

  private complete(skip = false) {
    if (
      this.cancelled ||
      this.frame.complete ||
      (!skip && this.frame.shown !== this.frame.combat.strikes.length)
    )
      return;
    this.frame = {
      ...this.frame,
      shown: this.frame.combat.strikes.length,
      complete: true,
      game: this.resolved,
    };
    this.release();
    this.publish(this.frame);
  }

  cancel() {
    this.cancelled = true;
    this.release();
  }
}
