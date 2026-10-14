import type {
  BattleInspection,
  Command,
  EngineResult,
  SavedGame,
  Start,
} from "../../rust/pkg/hex_command";
import type { GameEngine, GameEngineFactory } from "./native-engine";
import type { GameView } from "./view";
import type {
  CombatPreviewDetails,
  GameSnapshot,
  GameState,
  RecruitType,
  Side,
  Tile,
  Unit,
  UnitType,
  Weapon,
} from "./types";
import { immutable } from "./immutable";

/** Turn native rule errors into the controller's existing error/status path. */
export function nativeValue<T>(result: EngineResult<T>): T {
  if (result.ok) return result.value;
  const error = result.error;
  throw new Error(
    "message" in error
      ? error.message
      : error.code === "command"
        ? String(error.error)
        : error.code,
  );
}

/** Browser orchestration only. State mutations and rule queries are owned by Rust. */
export class LiveGame {
  private state!: GameState;
  private inspection!: BattleInspection;
  constructor(
    private readonly factory: GameEngineFactory,
    private engine: GameEngine,
  ) {
    try {
      this.refresh();
    } catch (error) {
      engine.dispose();
      throw error;
    }
  }
  static create(factory: GameEngineFactory, start: Start): LiveGame {
    return new LiveGame(factory, nativeValue(factory.create(start, 17)));
  }
  static restore(factory: GameEngineFactory, saved: SavedGame): LiveGame {
    return new LiveGame(factory, nativeValue(factory.restore(saved)));
  }
  private refresh(): void {
    const view = nativeValue(this.engine.view());
    // Authored string IDs cross into the existing presentation vocabulary here.
    this.state = view.state as unknown as GameState;
    this.inspection = nativeValue(this.engine.inspect()) as BattleInspection;
    if (view.revision !== this.inspection.revision)
      throw new Error("Incoherent native publication");
  }
  private dispatch(command: Command): void {
    nativeValue(this.engine.dispatch(command));
    this.refresh();
  }
  private id(id: string): number {
    if (!/^u[1-9]\d*$/.test(id)) throw new Error("Unknown unit.");
    return Number(id.slice(1));
  }
  get mapId() {
    return this.state.mapId;
  }
  get mapRevision() {
    return this.state.mapRevision;
  }
  get cols() {
    return this.state.cols;
  }
  get rows() {
    return this.state.rows;
  }
  get round() {
    return this.state.round;
  }
  get turn() {
    return this.state.turn;
  }
  get gold() {
    return this.state.gold;
  }
  get winner() {
    return this.state.winner;
  }
  get log() {
    return this.state.log;
  }
  get tiles() {
    return this.state.tiles;
  }
  get units() {
    return this.state.units;
  }
  get campaign() {
    return this.state.campaign;
  }
  get lastCombat() {
    return this.state.lastCombat;
  }
  tile(id: string | null | undefined): Tile | undefined {
    return this.tiles.find((tile) => tile.id === id);
  }
  unit(id: string | null | undefined): Unit | undefined {
    return this.units.find((unit) => unit.id === id);
  }
  at(id: string | null | undefined): Unit | undefined {
    return this.units.find((unit) => unit.tile === id);
  }
  neighbors(id: string): Tile[] {
    return (this.inspection.neighbors[id] ?? []).map((tile) => this.tile(tile)!);
  }
  cost(unit: Pick<Unit, "type">, tile: Pick<Tile, "terrain">): number {
    return this.inspection.costs[unit.type]?.[tile.terrain] ?? Infinity;
  }
  private choices(id: string | null | undefined) {
    return this.inspection.units.find((unit) => unit.unit === id);
  }
  reachable(id: string | null | undefined): Map<string, number> {
    return new Map(this.choices(id)?.reachable.map(({ tile, cost }) => [tile, cost]) ?? []);
  }
  targets(id: string | null | undefined): Unit[] {
    if (!this.unit(id) || this.unit(id)?.attacked || this.winner) return [];
    const targets = new Set(this.choices(id)?.attacks.map((attack) => attack.target));
    return this.units.filter((unit) => targets.has(unit.id));
  }
  canAttack(unit: Unit, target: Unit, attackId: string): boolean {
    return !!this.choices(unit.id)?.attacks.some(
      (attack) => attack.target === target.id && attack.attack.id === attackId,
    );
  }
  attackOptions(unit: Unit, target: Unit): Weapon[] {
    return (this.choices(unit.id)
      ?.attacks.filter((attack) => attack.target === target.id)
      .map((attack) => attack.attack) ?? []) as Weapon[];
  }
  bestAttack(unit: Unit, target: Unit): Weapon | null {
    return (this.choices(unit.id)?.attacks.find(
      (attack) => attack.target === target.id && attack.recommended,
    )?.attack ?? null) as Weapon | null;
  }
  previewDetails(unit: Unit, target: Unit, attackId?: string | null): CombatPreviewDetails {
    const weapon = attackId ?? this.bestAttack(unit, target)?.id;
    const details = this.choices(unit.id)?.attacks.find(
      (attack) => attack.target === target.id && attack.attack.id === weapon,
    );
    if (!details) throw new Error("That attack cannot reach this enemy.");
    return details as CombatPreviewDetails;
  }
  promotionOptions(id: string): UnitType[] {
    return (this.choices(id)?.promotions ?? []) as UnitType[];
  }
  recruitmentOptions(side: Side): RecruitType[] {
    return this.inspection.recruitment[side] as RecruitType[];
  }
  recruitTiles(id: string, type: UnitType): Tile[] {
    return (this.choices(id)?.recruitment[type] ?? []).map((tile) => this.tile(tile)!);
  }
  income(side: Side): number {
    return this.inspection.income[side];
  }
  move(id: string, tile: string): Unit {
    this.dispatch({ kind: "move", unit: this.id(id), tile });
    return this.unit(id)!;
  }
  attack(id: string, target: string, weapon?: string) {
    const selected = weapon ?? this.bestAttack(this.unit(id)!, this.unit(target)!)?.id;
    if (!selected) throw new Error("That attack cannot reach this enemy.");
    this.dispatch({ kind: "attack", unit: this.id(id), target: this.id(target), weapon: selected });
  }
  recruit(id: string, type: UnitType, tile: string): void {
    this.dispatch({ kind: "recruit", unit: this.id(id), type, tile });
  }
  advanceUnit(id: string, type: UnitType): void {
    this.dispatch({ kind: "advance", unit: this.id(id), type });
  }
  endTurn(): void {
    this.dispatch({ kind: "endTurn" });
  }
  /** One staged enemy action; the controller owns pacing and cancellation. */
  enemyStep(): void {
    const decision = nativeValue(
      this.engine.chooseAiCommand({
        policy: "classic",
        diplomacy: "restrained",
        recruitmentLimit: 2,
      }),
    );
    nativeValue(this.engine.dispatch(decision.command, decision.revision));
    this.refresh();
  }
  advanceDialogue(skip = false): void {
    this.dispatch({ kind: skip ? "skipDialogue" : "dismissDialogue" });
  }
  previousDialogue(): void {
    this.dispatch({ kind: "previousDialogue" });
  }
  nextCampaignMission(ids: string[]): LiveGame {
    this.dispatch({ kind: "recallMission", units: ids.map((id) => this.id(id)) });
    return this;
  }
  retryCampaignMission(): LiveGame {
    this.dispatch({ kind: "retryMission" });
    return this;
  }
  captureState(): GameState {
    return structuredClone(this.state);
  }
  restoreState(state: GameState): void {
    const checkpoint = nativeValue(this.engine.save());
    const replacement = nativeValue(
      this.factory.restore({
        ...checkpoint,
        state: state as unknown as SavedGame["state"],
      }),
    );
    nativeValue(this.engine.dispose());
    this.engine = replacement;
    this.refresh();
  }
  snapshot(): GameSnapshot {
    const { mapId, mapRevision, round, turn, gold, winner, units, campaign, tiles } = this.state;
    const sites = tiles
      .filter((tile) => ["grove", "fort", "ruins"].includes(tile.terrain))
      .map(({ id, owner }) => ({ id, owner }));
    return structuredClone({
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
    });
  }
  /** Current native checkpoint of resolved game state. */
  checkpoint(): ReturnType<GameEngine["save"]> {
    return this.engine.save();
  }
  dispose(): void {
    nativeValue(this.engine.dispose());
  }
  /** Capture only owned data in callbacks: old UI revisions never query the engine. */
  publishedView(): GameView {
    const state = this.state;
    const inspection = this.inspection;
    const detached = Object.create(LiveGame.prototype) as LiveGame;
    detached.state = state;
    detached.inspection = inspection;
    return immutable({
      campaign: state.campaign,
      mapId: state.mapId,
      mapRevision: state.mapRevision,
      cols: state.cols,
      rows: state.rows,
      round: state.round,
      turn: state.turn,
      gold: state.gold,
      winner: state.winner,
      log: state.log,
      tiles: state.tiles,
      units: state.units,
      tile: detached.tile.bind(detached),
      unit: detached.unit.bind(detached),
      at: detached.at.bind(detached),
      neighbors: (id: string) => immutable(detached.neighbors(id)),
      cost: detached.cost.bind(detached),
      reachable: detached.reachable.bind(detached),
      targets: (id: string | null | undefined) => immutable(detached.targets(id)),
      recruitTiles: (id: string, type: UnitType) => immutable(detached.recruitTiles(id, type)),
      canAttack: detached.canAttack.bind(detached),
      previewDetails: detached.previewDetails.bind(detached),
      recruitmentOptions: detached.recruitmentOptions.bind(detached),
      promotionOptions: (id: string) => immutable(detached.promotionOptions(id)),
      income: detached.income.bind(detached),
      snapshot: () => immutable(detached.snapshot()),
    }) as GameView;
  }
}
