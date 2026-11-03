import { TERRAIN } from "./catalog";
import type {
  BattleInspection,
  BoardInspection,
  BattlefieldPresentation,
  Command,
  EngineResult,
  EngineView,
  SavedGame,
  Start,
} from "../../rust/pkg/hex_command";
import { DEFAULT_SETUP, type SessionSetup } from "./setup";
import type { GameEngine, GameEngineFactory } from "./native-engine";
import type { GameView } from "./view";
import type {
  CombatPreviewDetails,
  CombatAnimation,
  GameSnapshot,
  GameState,
  RecruitType,
  Side,
  Tile,
  Unit,
  UnitType,
  Weapon,
} from "./types";
import { immutable, type Immutable } from "./immutable";

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

/** Compare immutable Rust records before preserving display identities across revisions. */
function sameData(a: unknown, b: unknown): boolean {
  if (a === b) return true;
  if (!a || !b || typeof a !== "object" || typeof b !== "object") return false;
  if (Array.isArray(a) !== Array.isArray(b)) return false;
  const left = a as Record<string, unknown>,
    right = b as Record<string, unknown>;
  const keys = Object.keys(left);
  return (
    keys.length === Object.keys(right).length &&
    keys.every((key) => Object.hasOwn(right, key) && sameData(left[key], right[key]))
  );
}

/** Browser orchestration only. State mutations and rule queries are owned by Rust. */
export class LiveGame {
  private state!: GameState;
  private snapshotView?: Immutable<GameSnapshot>;
  private resolvedPresentation!: Immutable<BattlefieldPresentation>;
  get presentation() {
    return this.resolvedPresentation;
  }
  private inspection!: BattleInspection;
  private displayedCampaign?: Immutable<NonNullable<GameState["campaign"]>>;
  private geometry!: Immutable<BoardInspection>;
  private tilesById!: Map<string, Tile>;
  private visibleUnits!: Unit[];
  private visibleGroundItems!: GameState["groundItems"];
  private unitsById!: Map<string, Unit>;
  private unitsByTile!: Map<string, Unit>;
  private inspectedUnits!: Map<string, BattleInspection["units"][number]>;
  private combatEvent: Immutable<CombatAnimation> | null = null;
  get combat() {
    return this.combatEvent;
  }
  constructor(
    private readonly factory: GameEngineFactory,
    private engine: GameEngine,
    readonly setup: SessionSetup = DEFAULT_SETUP,
  ) {
    try {
      this.refresh(nativeValue(this.engine.view()));
    } catch (error) {
      engine.dispose();
      throw error;
    }
  }
  static create(
    factory: GameEngineFactory,
    start: Start,
    setup: SessionSetup = DEFAULT_SETUP,
  ): LiveGame {
    return new LiveGame(factory, nativeValue(factory.create(start, 17)), structuredClone(setup));
  }
  static restore(
    factory: GameEngineFactory,
    saved: SavedGame,
    setup: SessionSetup = DEFAULT_SETUP,
  ): LiveGame {
    return new LiveGame(factory, nativeValue(factory.restore(saved)), structuredClone(setup));
  }
  private refresh(view: Immutable<EngineView>): void {
    this.snapshotView = undefined;
    if (
      !this.state ||
      this.state.mapId !== view.state.mapId ||
      this.state.mapRevision !== view.state.mapRevision ||
      this.state.cols !== view.state.cols ||
      this.state.rows !== view.state.rows
    )
      this.geometry = nativeValue(this.engine.geometry());
    // Keep indexes with this immutable revision; published views retain these maps.
    if (this.state?.tiles !== view.state.tiles)
      this.tilesById = new Map(
        (view.state.tiles as unknown as Tile[]).map((tile) => [tile.id, tile]),
      );
    // Authored string IDs cross into the existing presentation vocabulary here.
    this.state = view.state as unknown as GameState;
    const previousPresentation = this.resolvedPresentation;
    this.resolvedPresentation = immutable({
      ...view.presentation,
      fog:
        previousPresentation && sameData(previousPresentation.fog, view.presentation.fog)
          ? previousPresentation.fog
          : view.presentation.fog,
      lighting:
        previousPresentation && sameData(previousPresentation.lighting, view.presentation.lighting)
          ? previousPresentation.lighting
          : view.presentation.lighting,
    });
    this.inspection = nativeValue(this.engine.inspect()) as BattleInspection;
    if (view.revision !== this.inspection.revision)
      throw new Error("Incoherent native publication");
    this.displayedCampaign = this.state.campaign
      ? immutable({
          ...this.state.campaign,
          dialogue: this.state.campaign.dialogue.map((line, index) => ({
            ...line,
            speaker: this.inspection.speakers[index] ?? line.speaker,
          })),
        })
      : undefined;
    const previousInspections = this.inspectedUnits;
    this.inspectedUnits = new Map(
      this.inspection.units.map((unit) => {
        const previous = previousInspections?.get(unit.unit);
        return [
          unit.unit,
          previous && sameData(previous.faction, unit.faction)
            ? { ...unit, faction: previous.faction }
            : unit,
        ];
      }),
    );
    const fog = new Set(this.presentation.fog);
    const previousUnits = this.unitsById;
    this.visibleUnits = this.state.units
      .filter((unit) => this.choices(unit.id)?.faction.controllable || !fog.has(unit.tile))
      .map((source) => {
        const name = this.choices(source.id)?.name;
        const unit = name ? { ...source, name } : source;
        const previous = previousUnits?.get(unit.id);
        return previous && sameData(previous, unit) ? previous : unit;
      });
    this.visibleGroundItems = this.state.groundItems.filter((item) => !fog.has(item.tile));
    this.unitsById = new Map(this.visibleUnits.map((unit) => [unit.id, unit]));
    this.unitsByTile = new Map();
    for (const unit of this.visibleUnits)
      if (!this.unitsByTile.has(unit.tile)) this.unitsByTile.set(unit.tile, unit);
  }
  private dispatch(command: Immutable<Command>, expectedRevision?: number): void {
    const result = nativeValue(this.engine.dispatch(command, expectedRevision));
    this.combatEvent = immutable(result.combat as unknown as CombatAnimation | null);
    // Dispatch already owns the matching view. Reading it again resolves fog,
    // serializes and clones the complete battlefield a second time per action.
    this.refresh(result.view);
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
  get objectives() {
    return this.inspection.objectives;
  }
  get round() {
    return this.state.round;
  }
  get turn() {
    return this.state.turn;
  }
  get factions() {
    if (!this.state.factions) throw new Error("Native faction metadata is missing.");
    return this.state.factions;
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
  get groundItems() {
    return [...this.visibleGroundItems];
  }
  get units() {
    return [...this.visibleUnits];
  }
  get campaign() {
    return this.displayedCampaign;
  }
  get lastCombat() {
    return this.state.lastCombat;
  }
  tile(id: string | null | undefined): Tile | undefined {
    return id == null ? undefined : this.tilesById.get(id);
  }
  unit(id: string | null | undefined): Unit | undefined {
    return id == null ? undefined : this.unitsById.get(id);
  }
  at(id: string | null | undefined): Unit | undefined {
    return id == null ? undefined : this.unitsByTile.get(id);
  }
  neighbors(id: string): Tile[] {
    return (this.geometry.neighbors[id] ?? []).map((tile) => this.tile(tile)!);
  }
  cost(unit: Pick<Unit, "type">, tile: Pick<Tile, "terrain">): number {
    return this.geometry.costs[unit.type]?.[tile.terrain] ?? Infinity;
  }
  private choices(id: string | null | undefined) {
    return id == null ? undefined : this.inspectedUnits.get(id);
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
  faction(id: string) {
    const profile = this.choices(id)?.faction;
    if (!profile) throw new Error("Unknown unit.");
    return profile;
  }
  promotionOptions(id: string): UnitType[] {
    return (this.choices(id)?.promotions ?? []) as UnitType[];
  }
  recruitmentOptions(faction: string): RecruitType[] {
    return (this.inspection.recruitment[faction] ?? []) as RecruitType[];
  }
  recruitTiles(id: string, type: UnitType): Tile[] {
    return (this.choices(id)?.recruitment[type] ?? []).map((tile) => this.tile(tile)!);
  }
  recruitPrice(faction: string, type: UnitType): number {
    const price = this.inspection.recruitment_prices[faction]?.[type];
    if (price === undefined) throw new Error("Unknown faction recruit.");
    return price;
  }
  income(faction: string): number {
    return this.inspection.income[faction] ?? 0;
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
  dropItem(id: string, item: string): void {
    this.dispatch({ kind: "dropItem", unit: this.id(id), item });
  }
  endTurn(): void {
    this.dispatch({ kind: "endTurn" });
  }
  /** Plan against the current revision without advancing any durable game state. */
  prepareEnemyStep(): void {
    nativeValue(this.engine.chooseAiCommand(this.setup.enemy));
  }
  /** One staged enemy action; the controller owns pacing and cancellation. */
  enemyStep(): Immutable<Command> {
    const decision = nativeValue(this.engine.chooseAiCommand(this.setup.enemy));
    this.dispatch(decision.command, decision.revision);
    return decision.command;
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
  /** Undo retains an already-owned immutable revision, not another world clone. */
  captureUndoState(): Immutable<GameState> {
    return this.state;
  }
  restoreState(state: Immutable<GameState>): void {
    const checkpoint = nativeValue(this.engine.save());
    const replacement = nativeValue(
      this.factory.restore({
        ...checkpoint,
        state: state as unknown as SavedGame["state"],
      }),
    );
    nativeValue(this.engine.dispose());
    this.engine = replacement;
    this.refresh(nativeValue(this.engine.view()));
  }
  snapshot(): GameSnapshot {
    return structuredClone(this.publishedSnapshot()) as GameSnapshot;
  }
  /** A detached read-only revision for presentation and feedback. */
  publishedSnapshot(): Immutable<GameSnapshot> {
    if (this.snapshotView) return this.snapshotView;
    const { mapId, mapRevision, round, turn, gold, winner, campaign, tiles } = this.state;
    const sites = tiles
      .filter((tile) => ["grove", "fort", "ruins"].includes(TERRAIN[tile.terrain].role))
      .map(({ id, owner }) => ({ id, owner }));
    return (this.snapshotView = immutable({
      mapId,
      mapRevision,
      round,
      turn,
      gold,
      winner,
      units: this.visibleUnits,
      ...(campaign ? { campaign } : {}),
      villages: tiles
        .filter((tile) => TERRAIN[tile.terrain].role === "village")
        .map(({ id, owner }) => ({ id, owner })),
      ...(campaign || sites.length ? { sites } : {}),
    }));
  }
  /** Current native checkpoint of resolved game state. */
  checkpoint(): ReturnType<GameEngine["save"]> {
    return this.engine.save();
  }
  serializedCheckpoint(): ReturnType<GameEngine["saveJson"]> {
    return this.engine.saveJson();
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
    detached.displayedCampaign = this.displayedCampaign;
    detached.inspection = inspection;
    detached.snapshotView = this.snapshotView;
    detached.geometry = this.geometry;
    detached.resolvedPresentation = this.resolvedPresentation;
    detached.tilesById = this.tilesById;
    detached.visibleUnits = this.visibleUnits;
    detached.visibleGroundItems = this.visibleGroundItems;
    detached.unitsById = this.unitsById;
    detached.unitsByTile = this.unitsByTile;
    detached.inspectedUnits = this.inspectedUnits;
    return immutable({
      presentation: this.resolvedPresentation,
      objectives: this.objectives,
      campaign: this.displayedCampaign,
      mapId: state.mapId,
      mapRevision: state.mapRevision,
      cols: state.cols,
      rows: state.rows,
      round: state.round,
      turn: state.turn,
      gold: state.gold,
      factions: detached.factions,
      winner: state.winner,
      log: state.log,
      tiles: state.tiles,
      units: detached.units,
      groundItems: detached.groundItems,
      tile: detached.tile.bind(detached),
      unit: detached.unit.bind(detached),
      faction: detached.faction.bind(detached),
      at: detached.at.bind(detached),
      neighbors: (id: string) => immutable(detached.neighbors(id)),
      cost: detached.cost.bind(detached),
      reachable: detached.reachable.bind(detached),
      targets: (id: string | null | undefined) => immutable(detached.targets(id)),
      recruitTiles: (id: string, type: UnitType) => immutable(detached.recruitTiles(id, type)),
      canAttack: detached.canAttack.bind(detached),
      previewDetails: detached.previewDetails.bind(detached),
      recruitmentOptions: detached.recruitmentOptions.bind(detached),
      recruitPrice: detached.recruitPrice.bind(detached),
      promotionOptions: (id: string) => immutable(detached.promotionOptions(id)),
      income: detached.income.bind(detached),
      snapshot: () => detached.publishedSnapshot(),
    }) as GameView;
  }
}
