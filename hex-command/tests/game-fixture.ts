import type {
  State,
  PublicUnit,
  Tile,
  SavedGame,
  Weapon,
  Side,
  AttackDetails,
  Command,
} from "../rust/pkg/hex_command";
import { Native, Catalog, catalog, createBattle, restoreBattle } from "./native-fixture";
export { catalog } from "./native-fixture";
export const TYPES = Object.fromEntries(Catalog.units(catalog).map((rule) => [rule.id, rule]));
export const TERRAIN = Object.fromEntries(Catalog.terrain(catalog).map((rule) => [rule.id, rule]));
export const RACES = Object.fromEntries(Catalog.races(catalog).map((rule) => [rule.id, rule]));
export const DAMAGE_TYPES = { blade: "Blade", pierce: "Pierce", impact: "Impact" };

/** Editable test DTOs. Every query and mutation goes through actual OCaml/Rust. */
export class Game implements State {
  declare mapId: string;
  declare mapRevision: number;
  declare cols: number;
  declare rows: number;
  declare round: number;
  declare turn: Side;
  declare gold: State["gold"];
  declare winner: State["winner"];
  declare nextId: number;
  declare log: string[];
  declare tiles: Tile[];
  declare units: PublicUnit[];
  declare campaign?: State["campaign"];
  declare lastCombat?: State["lastCombat"];
  #battle: Native.battle;
  #signature = "";
  constructor(map = "mirror", revision?: number) {
    this.#battle = createBattle({
      kind: "scenario",
      map,
      ...(revision === undefined ? {} : { revision }),
    });
    this.restoreData(structuredClone(Native.state(Native.view(this.#battle))) as State);
  }
  static startCampaign(id: string, mission = 0): Game {
    const fixture = new Game();
    Native.dispose(fixture.#battle);
    fixture.#battle = createBattle({ kind: "campaign", id });
    fixture.restoreData(structuredClone(Native.state(Native.view(fixture.#battle))) as State);
    for (let chapter = 0; chapter < mission; chapter++) {
      fixture.gold.player = 0;
      fixture.winner = "player";
      fixture.nextCampaignMission([]);
    }
    return fixture;
  }
  /** Revalidate fixture edits as a native checkpoint before publishing them. */
  private native() {
    const state = this.captureState(),
      signature = JSON.stringify(state);
    if (signature !== this.#signature) {
      const checkpoint = structuredClone(Native.save(this.#battle)) as SavedGame;
      Native.replace(this.#battle, { ...checkpoint, state, planner: undefined });
      this.#signature = signature;
    }
    return this.#battle;
  }
  private restoreData(state: State, retainUnits = false): void {
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
  private apply(command: Command): void {
    Native.dispatch(this.native(), command);
    this.restoreData(structuredClone(Native.state(Native.view(this.#battle))) as State, true);
  }
  captureState(): State {
    return structuredClone({ ...this });
  }
  restoreState(state: State): void {
    const checkpoint = structuredClone(Native.save(this.#battle)) as SavedGame;
    Native.replace(this.#battle, { ...checkpoint, state, planner: undefined });
    this.restoreData(structuredClone(state));
  }
  tile(id: string | null | undefined): Tile | undefined {
    return this.tiles.find((tile) => tile.id === id);
  }
  unit(id: string | null | undefined): PublicUnit | undefined {
    return this.units.find((unit) => unit.id === id);
  }
  at(id: string | null | undefined): PublicUnit | undefined {
    return this.units.find((unit) => unit.tile === id);
  }
  addUnit(type: string, side: Side, tile: string): PublicUnit {
    const unit = {
      id: `u${this.nextId++}`,
      type,
      side,
      tile,
      hp: Catalog.unit(catalog, type).hp,
      experience: 0,
      moved: false,
      attacked: false,
    };
    this.units.push(unit);
    return unit;
  }
  record(message: string): void {
    this.log = [message, ...this.log].slice(0, 8);
  }
  publishedView() {
    return Native.view(this.native());
  }
  nativeBattle() {
    return this.native();
  }
  checkpoint(): Native.Immutable<SavedGame> {
    return Native.save(this.native());
  }
  snapshot() {
    const { mapId, mapRevision, round, turn, gold, winner, units, campaign, tiles } =
      this.captureState();
    const sites = tiles
      .filter((tile) => ["grove", "fort", "ruins"].includes(tile.terrain))
      .map(({ id, owner }) => ({ id, owner }));
    return {
      mapId,
      mapRevision,
      round,
      turn,
      gold,
      winner,
      units,
      ...(campaign ? { campaign } : {}),
      villages: tiles
        .filter((tile) => tile.terrain === "village")
        .map(({ id, owner }) => ({ id, owner })),
      ...(campaign || sites.length ? { sites } : {}),
    };
  }
  neighbors(id: string): Tile[] {
    return Native.neighbors(this.publishedView(), id).map((tile: Tile) => this.tile(tile.id)!);
  }
  cost(unit: Pick<PublicUnit, "type">, tile: Pick<Tile, "terrain">): number {
    return Native.movement_cost(Native.view(this.#battle), unit.type, tile.terrain);
  }
  reachable(id: string | null | undefined): Map<string, number> {
    return new Map(
      Object.entries(Native.reachable(this.publishedView(), id ?? "")) as [string, number][],
    );
  }
  targets(id: string | null | undefined): PublicUnit[] {
    return Native.targets(this.publishedView(), id ?? "").map((unit: PublicUnit) =>
      this.unit(unit.id)!,
    );
  }
  recruitTiles(id: string, type: string): Tile[] {
    return Native.recruit_tiles(this.publishedView(), id, type).map((tile: Tile) =>
      this.tile(tile.id)!,
    );
  }
  recruitmentOptions(side: Side): readonly string[] {
    return Native.recruitment_options(this.publishedView(), side);
  }
  promotionOptions(id: string): readonly string[] {
    return Native.promotion_options(this.publishedView(), id);
  }
  attackOptions(unit: PublicUnit, target: PublicUnit, from = unit.tile): Weapon[] {
    if (from !== unit.tile) return [];
    return Native.attack_options(this.publishedView(), unit.id, target.id).map(
      (attack: AttackDetails) => attack.attack,
    );
  }
  bestAttack(unit: PublicUnit, target: PublicUnit, from = unit.tile): Weapon | null {
    if (from !== unit.tile) return null;
    return Native.best_attack(this.publishedView(), unit.id, target.id)?.attack ?? null;
  }
  previewDetails(
    unit: PublicUnit,
    target: PublicUnit,
    attackId?: string | null,
    from = unit.tile,
  ): AttackDetails {
    const weapon = attackId ?? this.bestAttack(unit, target, from)?.id;
    const details =
      from === unit.tile &&
      Native.attack_details(this.publishedView(), unit.id, target.id, weapon ?? "");
    if (!details) throw new Error("That attack cannot reach this enemy.");
    return details;
  }
  preview(unit: PublicUnit, target: PublicUnit, attackId?: string | null, from = unit.tile) {
    const { damage, retaliation } = this.previewDetails(unit, target, attackId, from);
    return { damage, retaliation };
  }
  canAttackFrom(
    unit: PublicUnit | undefined,
    from: string,
    target: PublicUnit | undefined,
    attackId: string | null = null,
  ): boolean {
    return (
      !!unit &&
      !!target &&
      from === unit.tile &&
      this.attackOptions(unit, target, from).some(
        (attack) => attackId === null || attack.id === attackId,
      )
    );
  }
  income(side: Side): number {
    return Native.income(this.publishedView(), side);
  }
  move(id: string, tile: string): PublicUnit {
    this.apply({ kind: "move", unit: Native.unit_id(id), tile });
    return this.unit(id)!;
  }
  attack(id: string, target: string, weapon?: string): void {
    const selected = weapon ?? this.bestAttack(this.unit(id)!, this.unit(target)!)?.id;
    if (!selected) throw new Error("That attack cannot reach this enemy.");
    this.apply({
      kind: "attack",
      unit: Native.unit_id(id),
      target: Native.unit_id(target),
      weapon: selected,
    });
  }
  recruit(id: string, type: string, tile: string): PublicUnit {
    const unit = `u${this.nextId}`;
    this.apply({ kind: "recruit", unit: Native.unit_id(id), type, tile });
    return this.unit(unit)!;
  }
  advanceUnit(id: string, type: string): void {
    this.apply({ kind: "advance", unit: Native.unit_id(id), type });
  }
  endTurn(): void {
    this.apply({ kind: "endTurn" });
  }
  enemyStep(): void {
    Native.enemy_step(this.native(), {
      policy: "classic",
      diplomacy: "restrained",
      recruitmentLimit: 2,
    });
    this.restoreData(structuredClone(Native.state(Native.view(this.#battle))) as State, true);
  }
  advanceDialogue(skip = false): void {
    this.apply({ kind: skip ? "skipDialogue" : "dismissDialogue" });
  }
  previousDialogue(): void {
    this.apply({ kind: "previousDialogue" });
  }
  nextCampaignMission(ids: string[]): this {
    this.apply({ kind: "recallMission", units: ids.map(Native.unit_id) });
    return this;
  }
  retryCampaignMission(): this {
    this.apply({ kind: "retryMission" });
    return this;
  }
  dispose(): void {
    Native.dispose(this.#battle);
  }
  static fromCheckpoint(checkpoint: SavedGame): Game {
    const fixture = new Game();
    Native.dispose(fixture.#battle);
    fixture.#battle = restoreBattle(checkpoint);
    fixture.restoreData(structuredClone(Native.state(Native.view(fixture.#battle))) as State);
    return fixture;
  }
}
