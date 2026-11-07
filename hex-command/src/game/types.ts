import type { Hex, TileRef } from "../../rust/pkg/hex_command";
import type { CombatStrike, HpForecast, UnitRole } from "../../rust/pkg/hex_command";

export type Side = "player" | "enemy";
export type AdvancedTroop =
  | "briarweaver"
  | "deatharcher"
  | "direwolf"
  | "greatboar"
  | "wraith"
  | "spectre"
  | "lancer"
  | "mage"
  | "marksman"
  | "merfolk-maceguard-l2"
  | "ranger"
  | "revenant"
  | "skeleton-impact-caster-l2"
  | "soldier"
  | "tidehunter"
  | "tidesage"
  | "tridentguard"
  | "warden"
  | "champion"
  | "hammerman"
  | "warhammer"
  | "sentinel"
  | "halberdier"
  | "halberd-master"
  | "shadowblade"
  | "slinger"
  | "fireslinger"
  | "sharpshooter"
  | "crossbowman"
  | "arbalester"
  | "archmage"
  | "healer"
  | "high-healer"
  | "knight"
  | "horse-bowman"
  | "horse-marksman"
  | "skeleton-dreadguard"
  | "skeleton-flanker-l2"
  | "skeleton-flanker-l3"
  | "skeleton-impact-caster-l3"
  | "skeleton-scythe-caster-l2"
  | "skeleton-scythe-caster-l3"
  | "skeleton-rider-l2"
  | "skeleton-rider-l3"
  | "skeleton-bow-master"
  | "skeleton-axeman-l2"
  | "skeleton-axeman-l3"
  | "woodland-alpha-wolf"
  | "woodland-ancient-boar"
  | "woodland-thorn-oracle"
  | "woodland-bramble-warden"
  | "woodland-thornbinder"
  | "fox-spearguard-l2"
  | "fox-spearguard-l3"
  | "treant-l2"
  | "treant-l3"
  | "merfolk-harpoon-master"
  | "merfolk-netter-l2"
  | "merfolk-netter-l3"
  | "merfolk-reef-champion"
  | "merfolk-trident-l2"
  | "merfolk-trident-l3"
  | "merfolk-tide-oracle"
  | "merfolk-tidecrusher-l2"
  | "merfolk-tidecrusher-l3"
  | "merfolk-maceguard-l3";
export type UnitType =
  | "dragon"
  | "skeletal-hound"
  | "captives"
  | "zombie-walker"
  | "zombie-decay-caster"
  | "bear"
  | "giant-bat"
  | "giant-spider"
  | "commander"
  | "grovewarden"
  | "lich"
  | "marshal"
  | "necromancer"
  | "tidekeeper"
  | "tideseer"
  | "wildkeeper"
  | "grand-marshal"
  | "lich-commander-l3"
  | "woodland-grove-oracle"
  | "merfolk-tide-sovereign"
  | AdvancedTroop
  | "adept"
  | "archer"
  | "boar"
  | "bonearcher"
  | "ghost"
  | "harpooner"
  | "merfolk-maceguard"
  | "reefguard"
  | "rider"
  | "scout"
  | "skeleton-impact-caster"
  | "skeleton"
  | "spearman"
  | "swordsman"
  | "thorncaller"
  | "wavemender"
  | "wolf"
  | "horse-archer"
  | "skeleton-rider"
  | "skeleton-axeman"
  | "fox-spearguard"
  | "treant";
export type RecruitType = UnitType;
export type Race = "human" | "skeletons" | "beast" | "merfolk" | "spirit" | "zombies";
export type TerrainType =
  | "monastery"
  | "church"
  | "graveyard"
  | "ritual-star"
  | "ritual-spent"
  | "castle-wall"
  | "castle-bell"
  | "plains"
  | "road"
  | "forest"
  | "hill"
  | "water"
  | "bridge"
  | "ford"
  | "landing"
  | "ferry"
  | "house"
  | "mountain"
  | "cave"
  | "signpost"
  | "burned-village"
  | "village"
  | "keep"
  | "marsh"
  | "ruins"
  | "fort"
  | "grove";
export type DamageType = "blade" | "pierce" | "impact";
export type AttackCategory = "melee" | "ranged";
export type CampaignId =
  "lantern-road" | "green-accord" | "ashen-oath" | "borrowed-tide" | "mirror-night" | "bell-night";
export type CampaignMapId =
  | "bell-night"
  | "mirror-night"
  | CoastalMapId
  | "lantern-ferry"
  | "lantern-villages"
  | "lantern-bell"
  | "green-border"
  | "green-vigil"
  | "green-pact"
  | "ashen-quarry"
  | "ashen-refuge"
  | "ashen-dawn";
export type CoastalMapId = "tide-nets" | "tide-wreck" | "tide-sluice";
export type MapId = "mirror" | "developer" | "river-valley" | CampaignMapId | CoastalMapId;
export interface Coordinates {
  col: number;
  row: number;
}
export type HexDirection = "e" | "se" | "sw" | "w" | "nw" | "ne";
export interface Tile extends Coordinates {
  location?: string;
  id: string;
  terrain: TerrainType;
  owner: string | null;
  /** Artwork only: road/village exits or opposite bridge/ford edges. */
  connections?: HexDirection[];
  deadEnds?: HexDirection[];
  /** Explicit wall edges; selects a complete native tile. */
  walls?: HexDirection[];
  /** Artwork only: reusable floor palette, independent of terrain rules. */
  floor?: string;
  fog?: import("../../rust/pkg/hex_command").ConditionRule;
  light?: import("../../rust/pkg/hex_command").TileLight;
}
export interface Unit {
  faction?: string;
  items?: import("./immutable").Immutable<
    (Omit<import("../../rust/pkg/hex_command").Item, "attacks"> & { attacks: Weapon[] })[]
  >;
  aiGroup?: string;
  experience?: number;
  id: string;
  type: UnitType;
  side: Side;
  tile: Hex;
  hp: number;
  moved: boolean;
  attacked: boolean;
  character?: string;
  /** Resolved by Rust inspection for display; absent from native checkpoints. */
  name?: string;
  role?: "leader" | "courier" | "captive";
  /** Set by the enemy AI when withdrawing; visible before the player chooses to pursue. */
  retreating?: boolean;
  /** AI reinforcement hysteresis; saved so reloading does not release a rebuilding leader. */
  rebuilding?: boolean;
}
export interface Weapon {
  spread?: number;
  drain?: number;
  count: number;
  id: string;
  name: string;
  damage: number;
  damageType: DamageType;
  category: AttackCategory;
  range: number;
  sound: string;
}
export interface UnitDefinition {
  /** Rust-resolved values in canonical terrain order; null means impassable. */
  costs: (number | null)[];
  /** Incoming terrain modifiers indexed by terrain, melee/ranged, Blade/Pierce/Impact. */
  environment: number[][][];
  role: UnitRole;
  level?: number;
  family?: UnitType;
  experienceNeeded?: number;
  advancesTo?: UnitType[];
  /** Adjacent allies healed at the start of this unit's side's turn. */
  healing?: number;
  thorns?: number;
  thornAura?: number;
  name: string;
  hp: number;
  move: number;
  damage: number;
  range: number;
  cost: number;
  attacks: [Weapon, ...Weapon[]];
  /** Signed damage received: negative is armor, positive is weakness. */
  damageModifiers: Record<DamageType, number>;
  ability: string;
  description: string;
  race: Race;
}
export interface TerrainModifier {
  cost?: number;
  passable?: boolean;
  damageModifiers?: Partial<Record<DamageType | AttackCategory, number>>;
}
export interface RaceDefinition {
  name: string;
  terrainModifiers: Partial<
    Record<import("../../rust/pkg/hex_command").TerrainRole, TerrainModifier>
  >;
  /** Signed damage received, inherited by each unit specification. */
  damageModifiers: Record<DamageType, number>;
}
export interface TerrainDefinition {
  role: import("../../rust/pkg/hex_command").TerrainRole;
  name: string;
  cost: number;
  passable: boolean;
  damageModifiers?: TerrainModifier["damageModifiers"];
  description: string;
}
export interface TerrainOverride extends Coordinates {
  terrain: TerrainType;
  owner?: Side | null;
  connections?: HexDirection[];
  deadEnds?: HexDirection[];
  /** Explicit wall edges; selects a complete native tile. */
  walls?: HexDirection[];
  floor?: string;
  fog?: import("../../rust/pkg/hex_command").ConditionRule;
  light?: import("../../rust/pkg/hex_command").TileLight;
}
export interface StartingUnit {
  tile: Hex;
  aiGroup?: string;
  type: UnitType;
  side: Side;
  character?: string;
  role?: "leader" | "courier" | "captive";
}
export interface MapDefinition {
  id: MapId;
  hidden?: boolean;
  revision?: number;
  name: string;
  description: string;
  cols: number;
  rows: number;
  terrain: TerrainOverride[];
  units: StartingUnit[];
  lighting?: import("../../rust/pkg/hex_command").MapLighting;
  presentation: import("../../rust/pkg/hex_command").BattlefieldPresentation;
}
export interface DamageBreakdown {
  probabilities?: readonly number[];
  rollMin?: number;
  rollMax?: number;
  minimum?: number;
  maximum?: number;
  expected?: number;
  count?: number;
  total?: number;
  base: number;
  damageType: DamageType;
  modifier: number;
  modified: number;
  protection: number;
  exposure: number;
  terrain?: TerrainType;
  damage: number;
}
export interface CombatPreview {
  damage: number;
  retaliation: number;
}
export interface CombatPreviewDetails extends CombatPreview {
  attackerHealth: HpForecast;
  defenderHealth: HpForecast;
  killChance: number;
  deathChance: number;
  retaliationMin: number;
  retaliationMax: number;
  thornDamage: number;
  healing?: number;
  retaliationHealing?: number;
  recommended: boolean;
  attack: Weapon;
  response: Weapon | null;
  damageDetail: DamageBreakdown;
  retaliationDetail: DamageBreakdown | null;
}
export interface LastCombat {
  attackerId: string;
  attackId: string;
  sound: string;
}
export interface CombatAnimation {
  units: [Unit, Unit];
  strikes: CombatStrike[];
}
export interface VillageState {
  id: string;
  owner: string | null;
}
export interface GameSnapshot {
  mapId: MapId;
  mapRevision: number;
  round: number;
  turn: Side;
  gold: Record<string, number> & Record<Side, number>;
  winner: Side | null;
  units: Unit[];
  villages: VillageState[];
  campaign?: CampaignState;
  sites?: VillageState[];
}
/** Full session state, including mutable terrain, for exact move undo. */
export interface GameState {
  groundItems: import("../../rust/pkg/hex_command").GroundItem[];
  factions?: import("../../rust/pkg/hex_command").Faction[];
  conditionMemory?: string[];
  armies?: [string, string];
  combatRng?: number;
  mapId: MapId;
  mapRevision: number;
  cols: number;
  rows: number;
  round: number;
  turn: Side;
  gold: Record<string, number> & Record<Side, number>;
  winner: Side | null;
  nextId: number;
  log: string[];
  tiles: Tile[];
  units: Unit[];
  lastCombat?: LastCombat | null;
  campaign?: CampaignState;
}

export type DialoguePortrait = UnitType | "worker";
export interface DialogueLine {
  sound?: string;
  speaker: string;
  character?: string;
  portrait: DialoguePortrait;
  text: string;
}
/** Catalog goals use the generated Rust contract, including every authored target. */
export type MissionGoal = import("../../rust/pkg/hex_command").Goal;

export interface MissionEvent {
  id: string;
  recruitment?: Record<string, import("../../rust/pkg/hex_command").FactionRecruit[]>;
  spawn?: import("../../rust/pkg/hex_command").SpawnGroup;
  trigger:
    | { kind: "condition"; condition: import("../../rust/pkg/hex_command").Condition }
    | { kind: "visit"; tile: TileRef }
    | { kind: "round"; round: number }
    | { kind: "held"; tile: TileRef; turns: number }
    | { kind: "control"; tile: TileRef; round?: number };
  requiresEvent?: string;
  unlessEvent?: string;
  beforeRound?: number;
  label?: string;
  dialogue: DialogueLine[];
  gold?: number;
  warDialogue?: DialogueLine[];
}
export interface ReinforcementWave {
  dialogue?: DialogueLine[];
  /** Stable within its mission; saved deployment must never depend on array order. */
  id: string;
  round: number;
  label: string;
  units: StartingUnit[];
  requiresUnsealed?: string;
}
export interface MissionDefinition {
  itemIds?: string[];
  scriptedEvents?: import("../../rust/pkg/hex_command").ScriptEvent[];
  map: MapDefinition;
  subtitle: string;
  briefing: import("../../rust/pkg/hex_command").ConditionalText;
  debrief?: import("../../rust/pkg/hex_command").ConditionalText;
  requiredCompanions?: string[];
  arrivals?: { character: string; tile: Hex }[];
  hints: import("../../rust/pkg/hex_command").ConditionalHint[];
  goals: MissionGoal[];
  optionalGoals?: MissionGoal[];
  recruitment: Record<string, import("../../rust/pkg/hex_command").FactionRecruit[]>;
  gold: import("../../rust/pkg/hex_command").Gold;
  /** Occupying guards patrol their control objectives instead of raiding the starting camp. */
  defendObjectives?: boolean;
  /** An enemy leader guards this post and its immediate approaches; retreat remains free. */
  enemyLeaderPost?: TileRef;
  intro: import("../../rust/pkg/hex_command").Speech[];
  victory: import("../../rust/pkg/hex_command").Speech[];
  events: MissionEvent[];
  waves: ReinforcementWave[];
  /** Authored story changes when a named ally has fallen, without resurrecting them. */
  companionLosses?: Record<
    string,
    Partial<
      Pick<
        MissionDefinition,
        "briefing" | "debrief" | "hints" | "intro" | "victory" | "recruitment"
      >
    >
  >;
  refuge?: {
    medicineEvent: string;
    medicineTurns: number;
    escalation: DialogueLine[];
    warVictory: DialogueLine[];
    warBriefing: string;
    warHints: string[];
  };
  withoutDain?: {
    briefing: import("../../rust/pkg/hex_command").ConditionalText;
    hints: import("../../rust/pkg/hex_command").ConditionalHint[];
    intro: import("../../rust/pkg/hex_command").Speech[];
    recruitment: Record<string, import("../../rust/pkg/hex_command").FactionRecruit[]>;
    reinforcements: StartingUnit[];
    victory?: DialogueLine[];
    debrief?: import("../../rust/pkg/hex_command").ConditionalText;
  };
}
export interface CampaignDefinition {
  characters: { id: string; name: string }[];
  playerDiplomacy: ("unrestricted" | "diplomatic")[];
  id: CampaignId;
  name: string;
  tagline: string;
  description: string;
  shortDescription: string;
  race: Race;
  leader: UnitType;
  missions: MissionDefinition[];
  ending: import("../../rust/pkg/hex_command").Speech[];
}
export interface CampaignState {
  scripts?: import("../../rust/pkg/hex_command").EventState;
  evacuated?: { event: string; unit: Unit }[];
  id: CampaignId;
  mission: number;
  recall: RecruitType[];
  /** Incoming veterans are a checkpoint: retry never keeps failed-attempt XP. */
  veterans?: CampaignVeteran[];
  storyVeterans?: CampaignVeteran[];
  /** Deaths before this chapter; retry restores this incoming checkpoint. */
  fallenCompanions?: string[];
  carryGold: number;
  marks: string[];
  dialogue: DialogueLine[];
  dialogueIndex: number;
  outcome: string | null;
  completed: boolean;
  refuge?: { heldTurns: number; lastCountedRound: number; escalated: boolean };
  vigil?: { heldTurns: number; lastCountedRound: number };
  holds?: Record<string, { heldTurns: number; lastCountedRound: number }>;
  flags: Record<string, import("../../rust/pkg/hex_command").FlagChange[]>;
}
export interface CampaignVeteran {
  type: UnitType;
  experience: number;
  name?: string;
  role?: Unit["role"];
}
