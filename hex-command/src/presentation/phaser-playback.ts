import { TYPES, unitWeapons } from "../game/catalog";
import type { CombatPlaybackFrame } from "../controller/combat-playback";
import { cancelCombatSound, scheduleCue, scheduleDeath, soundForInteraction } from "./sound";

type Combat = CombatPlaybackFrame["combat"];
type Event = { at: number; run: () => void };
type Pending = {
  source: CombatPlaybackFrame;
  visible: ReadonlySet<string>;
  reduced: boolean;
};
type Playing = Pending & {
  kind: "playing";
  health: readonly [number, number];
  events: Event[];
  next: number;
  completeAt: number;
  deathAt: Map<string, number>;
};
type PlaybackState =
  | { kind: "idle" }
  | ({ kind: "waiting" } & Pending)
  | Playing
  | { kind: "complete"; combat: Combat };

/** Phaser owns the schedule; the controller retains the acknowledged strike
 * cursor across renderer failures. Waiting never acknowledges or skips a phase. */
export class BattlefieldPlayback {
  private state: PlaybackState = { kind: "idle" };
  constructor(
    private windup: (unit: string, at: number) => void,
    private impact: (combat: Combat, index: number, at: number) => void,
    private invalidate: () => void,
    private fail: (error: unknown) => void,
  ) {}

  present(
    source: CombatPlaybackFrame | null | undefined,
    visible: ReadonlySet<string>,
    reduced: boolean,
  ) {
    if (!source) {
      this.stop();
      return;
    }
    const state = this.state;
    if (
      (state.kind === "playing" && state.source.combat === source.combat) ||
      (state.kind === "complete" && state.combat === source.combat)
    )
      return;
    if (state.kind !== "waiting" || state.source.combat !== source.combat) this.stop();
    this.state = { kind: "waiting", source, visible, reduced };
    this.invalidate();
  }

  /** Called only after the scene has rendered a complete, ready battlefield. */
  ready() {
    const pending = this.state;
    if (pending.kind !== "waiting") return;
    const { source, visible, reduced } = pending;
    const combat = source.combat;
    try {
      if (!combat.units.some((unit) => visible.has(unit.id))) {
        this.state = { kind: "complete", combat };
        source.skip(); // Explicitly omit an exchange invisible to this player.
        return;
      }
      if (source.complete) {
        this.state = { kind: "complete", combat };
        // Reduced motion already published the resolved world. Preserve its cue
        // without replaying each intentionally omitted visual phase.
        if (combat.strikes[0]) {
          scheduleCue(this.family(combat, 0), performance.now(), "combat", false);
          const health = combat.strikes.at(-1)!.health;
          for (let i = 0; i < combat.units.length; i++)
            if (health[i]! <= 0 && visible.has(combat.units[i]!.id))
              scheduleDeath(combat.units[i]!.type, performance.now());
        }
        return;
      }
      const health = source.shown
        ? combat.strikes[source.shown - 1]!.health
        : ([combat.units[0].hp, combat.units[1].hp] as const);
      const playback: Playing = {
        kind: "playing",
        source,
        visible,
        reduced,
        health,
        events: [],
        next: 0,
        completeAt: Infinity,
        deathAt: new Map(),
      };
      for (let i = 0; i < combat.units.length; i++)
        if (health[i]! <= 0) playback.deathAt.set(combat.units[i]!.id, -Infinity);
      this.state = playback;
      this.scheduleStrike(playback, source.shown, performance.now() + (reduced ? 0 : 200));
    } catch (error) {
      this.stop();
      this.fail(error);
    }
  }
  /** Schedule just the next strike. Native voices stay bounded even for long
   * exchanges; each later phase is anchored when the preceding impact occurs. */
  private scheduleStrike(playback: Playing, index: number, at: number) {
    const combat = playback.source.combat;
    const strike = combat.strikes[index];
    if (!strike) {
      playback.completeAt = performance.now();
      return;
    }
    const actor = combat.units.find((unit) => unit.id === strike.unitId)!;
    const strikeAt = scheduleCue(
      this.family(combat, index),
      at,
      "combat",
      !playback.reduced,
    ).impactAt;
    playback.events = [];
    playback.next = 0;
    if (!playback.reduced)
      playback.events.push({
        at: strikeAt - 120,
        run: () => this.windup(actor.id, strikeAt - 120),
      });
    playback.events.push({
      at: strikeAt,
      run: () => {
        if (!playback.source.advance(index)) {
          this.stop();
          return;
        }
        playback.health = strike.health;
        this.impact(combat, index, strikeAt);
        if (index + 1 < combat.strikes.length)
          this.scheduleStrike(playback, index + 1, strikeAt + (playback.reduced ? 0 : 550));
        else playback.completeAt = strikeAt + (playback.reduced ? 0 : 300);
      },
    });
    for (let i = 0; i < combat.units.length; i++) {
      const unit = combat.units[i]!;
      if (strike.health[i]! > 0 || playback.deathAt.has(unit.id)) continue;
      playback.deathAt.set(unit.id, strikeAt);
      if (playback.visible.has(unit.id)) scheduleDeath(unit.type, strikeAt);
    }
    this.invalidate();
  }

  healthWidth(id: string): number | undefined {
    const state = this.state;
    if (state.kind !== "playing") return;
    const units = state.source.combat.units;
    const index = units.findIndex((unit) => unit.id === id);
    if (index >= 0) return (38 * state.health[index]!) / TYPES[units[index]!.type].hp;
  }
  death(id: string): number | undefined {
    return this.state.kind === "playing" ? this.state.deathAt.get(id) : undefined;
  }
  holds(id: string) {
    return this.state.kind === "playing" && this.state.source.combat.units.some((u) => u.id === id);
  }
  update(now: number) {
    const playback = this.state;
    if (playback.kind !== "playing") return;
    while (
      this.state === playback &&
      playback.next < playback.events.length &&
      playback.events[playback.next]!.at <= now
    )
      playback.events[playback.next++]!.run();
    if (this.state !== playback) return;
    if (now >= playback.completeAt) {
      this.state = { kind: "complete", combat: playback.source.combat };
      playback.source.finish();
    }
    this.invalidate();
  }
  private family(combat: Combat, index: number) {
    const strike = combat.strikes[index]!;
    const actor = combat.units.find((unit) => unit.id === strike.unitId);
    if (!actor) throw new Error(`Combat strike has no actor: ${strike.unitId}`);
    const weapon = unitWeapons(actor).find((weapon) => weapon.id === strike.weaponId);
    if (!weapon) throw new Error(`Combat strike has no weapon: ${strike.weaponId}`);
    return soundForInteraction({
      attackType: actor.type,
      attackId: weapon.id,
      attackSound: weapon.sound,
      attackCategory: weapon.category,
      attackDamageType: weapon.damageType,
    });
  }

  /** Release this renderer's resources without changing controller progress. */
  stop() {
    if (this.state.kind === "playing") cancelCombatSound();
    this.state = { kind: "idle" };
  }
  /** An explicit request to omit presentation, e.g. leaving the foreground. */
  skip() {
    const state = this.state;
    this.stop();
    if (state.kind === "waiting" || state.kind === "playing") {
      this.state = { kind: "complete", combat: state.source.combat };
      state.source.skip();
    }
  }
}
