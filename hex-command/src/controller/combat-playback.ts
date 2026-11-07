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
}

/** One presentation clock for every surface; the engine state is already committed. */
export class CombatPlayback {
  frame: CombatPlaybackFrame;
  readonly finished: Promise<void>;
  private release!: () => void;
  private cancelled = false;
  private timer?: ReturnType<typeof setTimeout>;

  constructor(
    before: BattleView,
    combat: NonNullable<BattleView["combat"]>,
    private readonly resolved: GameView,
    private readonly publish: (frame: CombatPlaybackFrame, strike: boolean) => void,
    reducedMotion: boolean,
  ) {
    this.finished = new Promise((resolve) => (this.release = resolve));
    this.frame = { before, combat, shown: 0, complete: false, game: before.game };
    if (reducedMotion || !combat.strikes.length) {
      this.frame = { ...this.frame, shown: combat.strikes.length, complete: true, game: resolved };
      this.release();
    } else this.timer = setTimeout(() => this.advance(), 200);
  }

  private advance() {
    if (this.cancelled) return;
    const { before, combat, shown } = this.frame;
    const hit = combat.strikes[shown];
    if (!hit) {
      this.frame = { ...this.frame, complete: true, game: this.resolved };
      this.release();
      this.publish(this.frame, false);
      return;
    }
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
    this.publish(this.frame, true);
    if (!this.cancelled) this.timer = setTimeout(() => this.advance(), 550);
  }

  cancel() {
    this.cancelled = true;
    clearTimeout(this.timer);
    this.release();
  }
}
