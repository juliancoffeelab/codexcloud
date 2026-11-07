/** Frozen serialized version-39 contracts, including their complete type closure. */
/**
 * A complete owned battle snapshot suitable for persistence and transport.
 */
export interface State {
  factions: Faction[];
  groundItems: GroundItem[];
  /**
   * Selected skirmish factions; campaigns use their authored recruitment.
   */
  armies?: [string, string];
  /**
   * Durable facts about conditions that have been satisfied at least once.
   */
  conditionMemory?: string[];
  mapId: string;
  mapRevision: number;
  cols: number;
  rows: number;
  round: number;
  turn: Side;
  gold: Gold;
  winner: Side | null;
  nextId: number;
  /**
   * Complete persisted combat PRNG state.
   */
  combatRng: number;
  log: string[];
  tiles: Tile[];
  units: PublicUnit[];
  campaign?: CampaignState;
  lastCombat?: LastCombat | null;
}

/**
 * A data-only backward change; this never records a command or planner state.
 */
export type UndoChange =
  { op: "set"; path: string[]; value: unknown } | { op: "remove"; path: string[] };

/**
 * A geometric cell address. Never an authored location name.
 */
export interface Hex {
  col: number;
  row: number;
}

/**
 * A stable member identity prevents repeated or skipped spawns after restoration.
 */
export interface SpawnUnit {
  faction?: string;
  id: string;
  type: string;
  side: Side;
  character?: string;
  role?: Role;
}

/**
 * A surviving unit carried between campaign chapters with its earned experience.
 */
export interface Veteran {
  items?: Item[];
  type: string;
  experience: number;
  character?: string;
  role?: Role;
}

/**
 * A tile snapshot combining authored coordinates with current terrain and ownership.
 */
export interface Tile {
  /**
   * Coordinate address (`col,row`), used by commands and snapshots.
   */
  id: string;
  /**
   * Optional authored landmark identity; ordinary cells have none.
   */
  location?: string;
  col: number;
  row: number;
  terrain: string;
  /**
   * Owning faction identifier, independent of its turn phase.
   */
  owner: string | null;
  /**
   * Road/village exits or opposite crossing edges; artwork only.
   */
  connections?: HexDirection[];
  /**
   * Short road spurs capped inside this tile rather than meeting another tile.
   */
  deadEnds?: HexDirection[];
  /**
   * Explicit complete wall edges; never inferred from neighbors.
   */
  walls?: HexDirection[];
  /**
   * Reusable ground palette; independent of terrain and its gameplay rules.
   */
  floor?: string;
  /**
   * Immutable condition shared by snapshots; Arc keeps native catalogs thread-safe.
   */
  fog?: ConditionRule;
  /**
   * Terrain light anchored to this tile, using the same contextual rules as fog.
   */
  light?: TileLight;
}

/**
 * A unit safely removed from the board, retaining identity and belongings.
 */
export interface EvacuatedUnit {
  event: string;
  unit: PublicUnit;
}

/**
 * An authored attack option with independent uniform strike rolls.
 */
export interface Weapon {
  id: string;
  name: string;
  damage: number;
  /**
   * Uniform raw roll is damage ± spread, inclusive; omitted means fixed damage.
   */
  spread?: number;
  /**
   * Hits in the selected attack; old definitions retain one hit.
   */
  count?: number;
  damageType: DamageType;
  category: Category;
  range: number;
  sound: string;
  /**
   * Percentage of actual HP removed restored to a surviving wielder.
   */
  drain?: number;
}

/**
 * An explicit authored reference. Runtime positions never use this type.
 */
export type TileRef = { location: string } | { hex: Hex } | { binding: TileBinding };

/**
 * An item left on a battlefield tile. Ownership passes only when collected.
 */
export interface GroundItem {
  tile: Hex;
  item: Item;
}

/**
 * An owned unit snapshot preserving stable IDs and optional campaign metadata.
 */
export interface PublicUnit {
  faction?: string;
  items?: Item[];
  aiGroup?: string;
  id: string;
  type: string;
  side: Side;
  tile: Hex;
  hp: number;
  moved: boolean;
  attacked: boolean;
  experience?: number;
  character?: string;
  role?: Role;
  retreating?: boolean;
  rebuilding?: boolean;
}

/**
 * Authored battlefield role used for mixed composition and tactical placement.
 */
export type UnitRole = "warrior" | "flank" | "ranger" | "healer";

/**
 * Clockwise edge names for a pointy-top hex. These describe artwork only.
 */
export type HexDirection = "e" | "se" | "sw" | "w" | "nw" | "ne";

/**
 * Closed authored role vocabulary.
 */
export type Role = "leader" | "courier" | "captive";

/**
 * Consecutive defended turns and the last round counted for an objective.
 */
export interface Counter {
  heldTurns: number;
  lastCountedRound: number;
}

/**
 * Current checkpoint data. Hosts version their save envelope and migrate before restore.
 */
export interface SavedGame {
  /**
   * Seed used when creating a new planning workspace.
   */
  seed: number;
  state: State;
}

/**
 * Defended refuge turns and whether mediation has escalated into battle.
 */
export interface RefugeCounter {
  heldTurns: number;
  lastCountedRound: number;
  escalated: boolean;
}

/**
 * Equipment owned by an individual unit, including its granted attacks.
 */
export interface Item {
  armor?: ItemArmor;
  id: string;
  name: string;
  description: string;
  attacks: Weapon[];
  /**
   * A carried aura, applied at the start of the carrier's allied turn.
   */
  healing?: ItemHealing;
}

/**
 * Explicit player-facing destinations, independent of AI intent and interaction radius.
 */
export interface PlayerMarker {
  tiles?: TileRef[];
  units?: UnitMatch | undefined;
  visibleInFog?: boolean;
}

/**
 * Feedback identifying the latest attack until the next turn clears it.
 */
export interface LastCombat {
  attackerId: string;
  attackId: string;
  sound: string;
}

/**
 * Gameplay meaning of terrain, independent of its authored/artwork identifier.
 */
export type TerrainRole =
  | "bridge"
  | "burned-village"
  | "cave"
  | "church"
  | "ferry"
  | "ford"
  | "forest"
  | "fort"
  | "fortification"
  | "graveyard"
  | "grove"
  | "hill"
  | "house"
  | "keep"
  | "landing"
  | "marsh"
  | "monastery"
  | "mountain"
  | "plains"
  | "ritual-spent"
  | "ritual-star"
  | "road"
  | "ruins"
  | "signpost"
  | "village"
  | "water";

/**
 * Map-wide ambience; terrain emitters belong to their tiles.
 */
export interface MapLighting {
  time: TimeOfDay;
}

/**
 * One explicit flag change. Stage uses the campaign's zero-based mission index.
 */
export interface FlagChange {
  state: boolean;
  stage: number;
  turn: number;
}

/**
 * One scripted dialogue line with its speaker, portrait and display text.
 */
export interface Dialogue {
  sound?: string;
  speaker?: string;
  character?: string;
  portrait: string;
  text: string;
}

/**
 * Persistent chapter progress, recalled companions, story choices and dialogue.
 */
export interface CampaignState {
  scripts?: EventState;
  evacuated?: EvacuatedUnit[];
  id: string;
  mission: number;
  recall: string[];
  veterans?: Veteran[];
  storyVeterans?: Veteran[];
  fallenCompanions?: string[];
  carryGold: number;
  marks: string[];
  dialogue: Dialogue[];
  dialogueIndex: number;
  outcome: string | null;
  completed: boolean;
  refuge?: RefugeCounter;
  vigil?: Counter;
  holds?: Record<string, Counter>;
  flags: Record<string, FlagChange[]>;
}

/**
 * Physical damage category indexing resistance and terrain modifiers.
 */
export type DamageType = "blade" | "pierce" | "impact";

/**
 * Policy configuration retained for the current planning phase.
 */
export interface AiOptions {
  /**
   * Optional independent blocks; omitted callers use the policy preset.
   */
  blocks?: Blocks;
  policy: Policy;
  diplomacy: Diplomacy;
  /**
   * Explicit recruitment override; omitted calls use the policy's default.
   */
  recruitmentLimit?: number;
}

/**
 * Remaining health across all attack, healing and retaliation outcomes.
 */
export interface HpForecast {
  minimum: number;
  maximum: number;
  expected: number;
}

/**
 * Remembering satisfaction is independent of the predicate being evaluated.
 */
export interface ConditionRule {
  condition: Condition;
  remember?: boolean;
}

/**
 * Resolved current presentation; sent with views, never persisted as a cache.
 */
export interface BattlefieldPresentation {
  lighting: BattlefieldLighting;
  fog: Hex[];
}

/**
 * String shorthand sets true; the explicit form can also clear a flag.
 */
export type FlagUpdate = string | { id: string; state: boolean };

/**
 * Tactical and recruitment policy, preserving each policy's stable choices.
 */
export type FactionPolicy =
  "classic" | "rush" | "balanced" | "rotated" | "situational" | "predictor";

/**
 * Tactical and recruitment policy, preserving each policy's stable choices.
 */
export type Policy = "classic" | "rush" | "balanced" | "rotated" | "situational" | "predictor";

/**
 * The actual roll, modifiers and HP effects of one executed strike.
 */
export interface CombatStrike {
  side: Side;
  weapon: string;
  rollMin: number;
  rollMax: number;
  raw: number;
  modifier: number;
  terrain: number;
  damage: number;
  effective: number;
  drain: number;
  healed: number;
  thorns: number;
  reflected: number;
  health: [number, number];
}

/**
 * The human and automatic turn phases. Factions define diplomatic allegiance.
 */
export type Side = "player" | "enemy";

/**
 * Transfer the same unit when a shared authored condition is satisfied.
 * Contextual conditions use the unit's current tile.
 */
export interface FactionTransfer {
  to: string;
  when: ConditionRule;
}

/**
 * Treasury balances keyed by faction identifier.
 */
export type Gold = Record<string, number>;

/**
 * Treatment of retreating and mediated opponents during campaign battles.
 */
export type Diplomacy = "diplomatic" | "unrestricted";

/**
 * Unit allegiance selectors are distinct from the two turn/ownership phases.
 * `Any` tests existence without excluding passive or rescued actors.
 */
export type ConditionSide = "player" | "enemy" | "neutral" | "any";

/**
 * Weapon category used to select compatible retaliation attacks.
 */
export type Category = "melee" | "ranged";

export interface AllegianceChange {
  units: UnitMatch;
  faction: string;
}

export interface BattlefieldLight {
  color?: [number, number, number];
  fixture?: boolean;
  col: number;
  row: number;
  dx?: number;
  dy?: number;
  radius: number;
  power: number;
  when?: ConditionRule;
}

export interface BattlefieldLighting {
  time: TimeOfDay;
  lights?: BattlefieldLight[];
}

export interface Blocks {
  recruiter: RecruiterKind;
  campaign: CampaignerKind;
  planner: PlannerKind;
}

export interface DeathSelection extends UnitMatch {
  initiatedBy?: string | undefined;
  killedBy?: string | undefined;
  defending?: boolean | undefined;
}

export interface DialogueBranch {
  with: string;
  dialogue: Speech[];
  alternative?: Speech[];
}

export interface EventHold {
  event: string;
  predicate: number;
  turns: number;
  lastRound: number;
}

export interface EventObjective {
  id?: string | undefined;
  preserveLives?: UnitMatch | undefined;
  label: string;
  optional?: boolean;
  visibleWhen?: When | undefined;
  availableWhen?: When | undefined;
  /**
   * None is a permanent achievement; Some is a live requirement until victory.
   */
  satisfiedWhen?: When | undefined;
  intent: EventIntent;
  playerMarker?: PlayerMarker | undefined;
}

export interface EventState {
  failed?: string[];
  completed: string[];
  revealed: string[];
  recruitment: Record<string, FactionRecruit[]>;
  active: string[];
  holds: EventHold[];
  marks: MapMark[];
  pending: PendingEventSpawn[];
  nextMark: number;
}

export interface Faction {
  id: string;
  /**
   * Phase in which this controller acts; this is not its diplomatic relation.
   */
  phase: Side;
  controller: Controller;
  palette: string;
  name?: string;
  /**
   * Named factions author their own pool. Prices are recruitment economics,
   * not modifications to a unit's combat profile.
   */
  recruits?: FactionRecruit[];
  relations?: Record<string, Relation>;
  transfer?: FactionTransfer;
}

export interface FactionRecruit {
  unit: string;
  cost?: number;
}

export interface GiveItem extends UnitMatch {
  item: string;
  tile?: TileRef | undefined;
}

export interface GoldGrant {
  faction: string;
  amount: number;
}

export interface HoldSelection extends UnitMatch {
  tile: TileRef;
  turns: number;
  when?: When | undefined;
}

export interface ItemArmor {
  blade: number;
  pierce: number;
  impact: number;
}

export interface ItemHealing {
  amount: number;
  radius: number;
}

export interface LoopSelection extends UnitMatch {
  tile?: TileRef | undefined;
}

export interface MapMark {
  lightRemembered?: boolean;
  id: number;
  type: string;
  tile: Hex;
  light?: TileLight;
}

export interface MarkLoop {
  select: MarkSelection;
  actions: EventAction[];
}

export interface MarkSelection {
  type: string;
}

export interface MarkSpec {
  type: string;
  tile: TileRef;
  light?: TileLight | undefined;
}

export interface OwnSelection {
  tile: TileRef;
  faction: string;
  turns?: number | undefined;
  when?: When | undefined;
}

export interface OwnerSelection {
  tile: TileRef;
  faction: string;
}

export interface PendingEventSpawn {
  area: Hex[];
  faction: string;
  aiGroup?: string | null | undefined;
  units: SpawnMember[];
}

export interface PlaceItem {
  item: string;
  tile: TileRef;
}

export interface ProximitySelection extends UnitMatch {
  tile?: TileRef | undefined;
  target?: UnitMatch | undefined;
  radius: number;
  through?: boolean;
}

export interface RecruitmentUnlock {
  faction: string;
  units: FactionRecruit[];
}

export interface ScriptEvent {
  id: string;
  when: When;
  remembers?: boolean;
  priority?: number;
  objective?: EventObjective | undefined;
  actions: EventAction[];
}

export interface ScriptLine {
  character?: string | undefined;
  speaker?: string | undefined;
  portrait?: string | undefined;
  text: string;
  sound?: string | undefined;
}

export interface ScriptSpawn {
  faction: string;
  tile?: TileRef | undefined;
  area?: TileRef[];
  radius?: number;
  aiGroup?: string | undefined;
  units: SpawnEntry[];
}

export interface SiteSelection extends UnitMatch {
  tile: TileRef;
  through?: boolean;
}

export interface SpawnGroup {
  faction?: string;
  aiGroup?: string;
  area: TileRef[];
  units: SpawnUnit[];
}

export interface SpawnMember {
  type: string;
  character?: string | null | undefined;
  role?: Role | null | undefined;
  items?: string[];
}

export interface TerrainChange {
  tile: TileRef;
  terrain: string;
}

export interface TextVariant {
  when?: When | undefined;
  text: string;
}

export interface TileLight {
  color?: [number, number, number];
  fixture?: boolean;
  dx?: number;
  dy?: number;
  radius: number;
  power: number;
  when?: ConditionRule;
}

export interface UndoStep {
  round: number;
  unitId: string;
  changes: UndoChange[];
}

export interface UnitLoop {
  select: LoopSelection;
  actions: EventAction[];
}

export interface UnitMatch {
  character?: string | undefined;
  role?: Role | undefined;
  side?: ConditionSide | undefined;
  faction?: string | undefined;
  unitType?: string | undefined;
  unitTypes?: string[];
  aiGroup?: string | undefined;
  retreating?: boolean | undefined;
}

export type CampaignerKind = "arena" | "mission";

export type Condition =
  | { kind: "owner"; tile?: TileRef; side: string }
  | {
      kind: "proximity";
      unitTypes?: string[];
      tile?: TileRef;
      radius: number;
      side: ConditionSide;
      unitType?: string;
      character?: string;
    }
  | { kind: "visibleUnits"; side: ConditionSide; unitTypes?: string[] }
  | {
      kind: "hasUnits";
      aiGroup?: string;
      side: ConditionSide;
      unitType?: string;
      character?: string;
    }
  | { kind: "noUnits"; side: ConditionSide; unitType?: string; character?: string }
  | { kind: "choice"; choice: string }
  | { kind: "flag"; flag: string }
  | { kind: "event"; event: string }
  | { kind: "round"; round: number }
  | { kind: "all"; conditions: Condition[] }
  | { kind: "any"; conditions: Condition[] };

export type ConditionalHint = string | TextVariant;

export type ConditionalText = string | TextVariant[];

export type Controller =
  { kind: "human" } | { kind: "passive" } | { kind: "ai"; policy?: FactionPolicy };

export type EventAction =
  | { spawn: ScriptSpawn }
  | { give: GiveItem }
  | { placeItem: PlaceItem }
  | { changeAllegiance: AllegianceChange }
  | { deleteUnit: UnitReference }
  | { setFlag: FlagUpdate }
  | { say: Speech[] }
  | { sound: string }
  | { createMark: MarkSpec }
  | { removeMark: string }
  | { forEachUnit: UnitLoop }
  | { forEachMark: MarkLoop }
  | { setTerrain: TerrainChange }
  | { gold: GoldGrant }
  | { unlockRecruits: RecruitmentUnlock }
  | { victory: string }
  | { defeat: string }
  | { log: string }
  | { failObjective: string }
  | { completeObjective: string }
  | { recruits: RecruitmentUnlock };

export type EventIntent =
  | { hold: HoldSelection }
  | { own: OwnSelection }
  | { escort: SiteSelection }
  | { goTo: SiteSelection }
  | { escape: SiteSelection }
  | { approach: ProximitySelection }
  | { capture: TileRef[] }
  | { defeat: UnitMatch };

export type PlannerKind = "classic" | "full" | "queued";

export type RecruiterKind =
  | "classic"
  | "rotation"
  | "frontline"
  | "balanced"
  | "situational"
  | "predictor"
  | "aggressive-predictor";

export type Relation = "allied" | "neutral" | "hostile";

export type SpawnEntry = string | SpawnMember;

export type Speech = DialogueBranch | ScriptLine;

export type TileBinding = "unit" | "mark" | "death";

export type TimeOfDay = "day" | "night";

export type TurnSelection = number | { number: number; faction: string };

export type UnitReference = string | UnitMatch;

export type When =
  | { always: boolean }
  | { all: When[] }
  | { any: When[] }
  | { not: When }
  | { stand: SiteSelection }
  | { proximity: ProximitySelection }
  | { killed: DeathSelection }
  | { left: SiteSelection }
  | { turn: TurnSelection }
  | { turnAtLeast: number }
  | { chapterStarted: boolean }
  | { flag: string }
  | { objectiveSatisfied: string }
  | { hasUnits: UnitMatch }
  | { noUnits: UnitMatch }
  | { visibleUnits: UnitMatch }
  | { owner: OwnerSelection }
  | { own: OwnSelection }
  | { held: HoldSelection };
