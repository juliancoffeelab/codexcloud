import { LiveGame, nativeValue } from "../src/game/live-game";
import { TYPES } from "../src/game/catalog";
import type { GameState, Unit, Tile, CampaignId, UnitType, Side, MapId } from "../src/game/types";
import type { SavedGame } from "../rust/pkg/hex_command";
import { createTestFactory } from "./native-fixture";
export { TYPES, TERRAIN, RACES, DAMAGE_TYPES, axial, distance } from "../src/game/catalog";

/** Keep the real showcase geometry but bound actors for ordinary behavior checks. */
export function developerFixture(types: readonly UnitType[] = []): Game {
  const engine = nativeValue(
    createTestFactory().create({ kind: "scenario", map: "developer" }, 17),
  );
  let checkpoint: SavedGame;
  try {
    checkpoint = structuredClone(nativeValue(engine.save())) as SavedGame;
  } finally {
    engine.dispose();
  }
  const leaders = new Set(
    (["player", "enemy"] as const).map(
      (side) =>
        checkpoint.state.units.find(
          (unit) => unit.side === side && TYPES[unit.type as UnitType].family === "commander",
        )!.id,
    ),
  );
  checkpoint.state.units = checkpoint.state.units.filter(
    (unit) => leaders.has(unit.id) || types.includes(unit.type as UnitType),
  );
  return Game.fromCheckpoint(checkpoint);
}

/** Editable test data backed by the actual Rust engine for every query and command. */
export class Game implements GameState {
  declare mapId: MapId;
  declare mapRevision: number;
  declare cols: number;
  declare rows: number;
  declare round: number;
  declare turn: Side;
  declare gold: GameState["gold"];
  declare winner: GameState["winner"];
  declare nextId: number;
  declare log: string[];
  declare tiles: Tile[];
  declare units: Unit[];
  declare campaign?: GameState["campaign"];
  declare lastCombat?: GameState["lastCombat"];
  #native: LiveGame;
  #signature = "";
  #cost!: LiveGame["cost"];
  constructor(map: string | SavedGame = "mirror", revision?: number) {
    this.#native =
      typeof map === "string"
        ? LiveGame.create(createTestFactory(), {
            kind: "scenario",
            map,
            ...(revision === undefined ? {} : { revision }),
          })
        : LiveGame.restore(createTestFactory(), map);
    this.#cost = this.#native.publishedView().cost;
    this.restoreData(this.#native.captureState());
  }
  static startCampaign(id: CampaignId, mission = 0): Game {
    const fixture = new Game();
    fixture.#native.dispose();
    fixture.#native = LiveGame.create(createTestFactory(), { kind: "campaign", id });
    fixture.restoreData(fixture.#native.captureState());
    for (let chapter = 0; chapter < mission; chapter++) {
      fixture.gold.player = 0;
      fixture.winner = "player";
      fixture.nextCampaignMission([]);
    }
    return fixture;
  }
  /** Fixture editing changes only serialized state; synchronization runs native validation. */
  private native(): LiveGame {
    const state = this.captureState(),
      signature = JSON.stringify(state);
    if (signature !== this.#signature) {
      this.#native.restoreState(state);
      this.#signature = signature;
    }
    return this.#native;
  }
  private restoreData(state: GameState, retainUnits = false): void {
    if (retainUnits && state.mapId === this.mapId)
      state.units = state.units.map((unit) => {
        const previous = this.unit(unit.id);
        if (!previous) return unit;
        for (const key of Object.keys(previous)) Reflect.deleteProperty(previous, key);
        return Object.assign(previous, unit);
      });
    delete this.campaign;
    delete this.lastCombat;
    Object.assign(this, state);
    this.#signature = JSON.stringify(state);
  }
  private apply(action: (native: LiveGame) => void): void {
    const native = this.native();
    action(native);
    this.restoreData(native.captureState(), true);
  }
  captureState(): GameState {
    return structuredClone({ ...this });
  }
  restoreState(state: GameState): void {
    this.#native.restoreState(state);
    this.restoreData(structuredClone(state));
  }
  tile(id: string | null | undefined) {
    return this.tiles.find((tile) => tile.id === id);
  }
  unit(id: string | null | undefined) {
    return this.units.find((unit) => unit.id === id);
  }
  at(id: string | null | undefined) {
    return this.units.find((unit) => unit.tile === id);
  }
  addUnit(type: UnitType, side: Side, tile: string): Unit {
    const unit = {
      id: `u${this.nextId++}`,
      type,
      side,
      tile,
      hp: TYPES[type].hp,
      experience: 0,
      moved: false,
      attacked: false,
      remainingAttacks: Object.fromEntries(
        TYPES[type].attacks.map((attack) => [attack.id, attack.count]),
      ),
    };
    this.units.push(unit);
    return unit;
  }
  record(message: string): void {
    this.log = [message, ...this.log].slice(0, 8);
  }
  publishedView() {
    return this.native().publishedView();
  }
  snapshot() {
    return this.native().snapshot();
  }
  neighbors(id: string): Tile[] {
    return this.native()
      .neighbors(id)
      .map((tile) => this.tile(tile.id)!);
  }
  cost(unit: Pick<Unit, "type">, tile: Pick<Tile, "terrain">) {
    return this.#cost(unit, tile);
  }
  reachable(id: string | null | undefined) {
    return this.native().reachable(id);
  }
  targets(id: string | null | undefined) {
    return this.native()
      .targets(id)
      .map((unit) => this.unit(unit.id)!);
  }
  recruitTiles(id: string, type: UnitType) {
    return this.native()
      .recruitTiles(id, type)
      .map((tile) => this.tile(tile.id)!);
  }
  recruitmentOptions(side: Side) {
    return this.native().recruitmentOptions(side);
  }
  promotionOptions(id: string) {
    return this.native().promotionOptions(id);
  }
  previewDetails(...args: Parameters<LiveGame["previewDetails"]>) {
    return this.native().previewDetails(...args);
  }
  preview(...args: Parameters<LiveGame["preview"]>) {
    return this.native().preview(...args);
  }
  attackOptions(...args: Parameters<LiveGame["attackOptions"]>) {
    return this.native().attackOptions(...args);
  }
  bestAttack(...args: Parameters<LiveGame["bestAttack"]>) {
    return this.native().bestAttack(...args);
  }
  canAttackFrom(...args: Parameters<LiveGame["canAttackFrom"]>) {
    return this.native().canAttackFrom(...args);
  }
  income(side: Side) {
    return this.native().income(side);
  }
  move(id: string, tile: string) {
    this.apply((native) => native.move(id, tile));
    return this.unit(id)!;
  }
  attack(...args: Parameters<LiveGame["attack"]>) {
    this.apply((native) => native.attack(...args));
  }
  recruit(...args: Parameters<LiveGame["recruit"]>) {
    const id = `u${this.nextId}`;
    this.apply((native) => native.recruit(...args));
    return this.unit(id)!;
  }
  advanceUnit(...args: Parameters<LiveGame["advanceUnit"]>) {
    this.apply((native) => native.advanceUnit(...args));
  }
  endTurn() {
    this.apply((native) => native.endTurn());
  }
  advanceDialogue(skip = false) {
    this.apply((native) => native.advanceDialogue(skip));
  }
  previousDialogue() {
    this.apply((native) => native.previousDialogue());
  }
  nextCampaignMission(ids: string[]) {
    this.apply((native) => native.nextCampaignMission(ids));
    return this;
  }
  retryCampaignMission() {
    this.apply((native) => native.retryCampaignMission());
    return this;
  }
  /** Explicit fixture restoration still uses the real native acceptance path. */
  static fromCheckpoint(checkpoint: SavedGame): Game {
    return new Game(checkpoint);
  }
}
