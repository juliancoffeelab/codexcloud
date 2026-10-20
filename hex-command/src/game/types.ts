import type { UnitRole } from "../../rust/pkg/hex_command";

export type Side = "player" | "enemy";
export type LeaderType =
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
  | "merfolk-tide-sovereign";
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
  | LeaderType
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
export type RecruitType = Exclude<UnitType, LeaderType>;
export type Race = "human" | "skeletons" | "beast" | "merfolk" | "spirit";
export type TerrainType =
  | "plains"
  | "road"
  | "forest"
  | "hill"
  | "water"
  | "bridge"
  | "ford"
  | "village"
  | "keep"
  | "marsh"
  | "ruins"
  | "fort"
  | "grove";
export type DamageType = "blade" | "pierce" | "impact";
export type AttackCategory = "melee" | "ranged";
export type CampaignId = "lantern-road" | "green-accord" | "ashen-oath" | "borrowed-tide";
export type CampaignMapId =
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
export type MapId =
  "mirror" | "river" | "developer" | "river-valley" | "two-routes" | CampaignMapId | CoastalMapId;
export interface Coordinates {
  col: number;
  row: number;
}
export type HexDirection = "e" | "se" | "sw" | "w" | "nw" | "ne";
export interface Tile extends Coordinates {
  id: string;
  terrain: TerrainType;
  owner: Side | null;
  /** Artwork only: road/village exits or opposite bridge/ford edges. */
  connections?: HexDirection[];
  deadEnds?: HexDirection[];
  /** Artwork only: reusable floor palette, independent of terrain rules. */
  floor?: string;
}
export interface Unit {
  /** Missing on pre-progression saves, which load as zero. */
  experience?: number;
  id: string;
  type: UnitType;
  side: Side;
  tile: string;
  hp: number;
  moved: boolean;
  attacked: boolean;
  name?: string;
  role?: "leader" | "courier";
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
  leader?: boolean;
}
export interface TerrainModifier {
  cost?: number;
  passable?: boolean;
  damageModifiers?: Partial<Record<DamageType | AttackCategory, number>>;
}
export interface RaceDefinition {
  name: string;
  terrainModifiers: Partial<Record<TerrainType, TerrainModifier>>;
  /** Signed damage received, inherited by each unit specification. */
  damageModifiers: Record<DamageType, number>;
}
export interface TerrainDefinition {
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
  floor?: string;
}
export interface StartingUnit extends Coordinates {
  type: UnitType;
  side: Side;
  name?: string;
  role?: "leader" | "courier";
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
}
export interface DamageBreakdown {
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
export interface VillageState {
  id: string;
  owner: Side | null;
}
export interface GameSnapshot {
  mapId: MapId;
  mapRevision: number;
  round: number;
  turn: Side;
  gold: Record<Side, number>;
  winner: Side | null;
  units: Unit[];
  villages: VillageState[];
  campaign?: CampaignState;
  sites?: VillageState[];
}
/** Full session state, including mutable terrain, for exact move undo. */
export interface GameState {
  combatRng?: number;
  mapId: MapId;
  mapRevision: number;
  cols: number;
  rows: number;
  round: number;
  turn: Side;
  gold: Record<Side, number>;
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
  speaker: string;
  portrait: DialoguePortrait;
  text: string;
}
export type MissionGoal =
  | { kind: "escort"; label: string; tile: string }
  | { kind: "escape"; label: string; tile: string }
  | { kind: "control"; label: string; tiles: string[] }
  | {
      kind: "hold";
      label: string;
      tile: string;
      turns: number;
      earliestRound: number;
      startEvent?: string;
    }
  | { kind: "vigil"; label: string; tile: string; turns: number; companion: string }
  | { kind: "refuge"; label: string; tile: string; turns: number }
  | { kind: "seal"; label: string; tiles: string[]; unitType?: UnitType }
  | { kind: "event"; label: string; tile: string; event: string }
  | { kind: "defeat"; label: string };
export interface MissionEvent {
  id: string;
  trigger:
    | { kind: "visit"; tile: string }
    | { kind: "round"; round: number }
    | { kind: "control"; tile: string; round?: number };
  requiresEvent?: string;
  unlessEvent?: string;
  beforeRound?: number;
  label?: string;
  dialogue: DialogueLine[];
  gold?: number;
  warDialogue?: DialogueLine[];
}
export interface ReinforcementWave {
  /** Stable within its mission; saved deployment must never depend on array order. */
  id: string;
  round: number;
  label: string;
  units: StartingUnit[];
  requiresUnsealed?: string;
}
export interface MissionDefinition {
  map: MapDefinition;
  subtitle: string;
  briefing: string;
  debrief?: string;
  requiredCompanions?: string[];
  hints: string[];
  goals: MissionGoal[];
  recruits: RecruitType[];
  enemyRecruits: RecruitType[];
  gold: number;
  enemyGold: number;
  /** Occupying guards patrol their control objectives instead of raiding the starting camp. */
  defendObjectives?: boolean;
  /** An enemy leader guards this post and its immediate approaches; retreat remains free. */
  enemyLeaderPost?: string;
  intro: DialogueLine[];
  victory: DialogueLine[];
  events: MissionEvent[];
  waves: ReinforcementWave[];
  /** Authored story changes when a named ally has fallen, without resurrecting them. */
  companionLosses?: Record<
    string,
    Partial<Pick<MissionDefinition, "briefing" | "debrief" | "hints" | "intro" | "victory">>
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
    briefing: string;
    hints: string[];
    intro: DialogueLine[];
    recruits: RecruitType[];
    reinforcements: StartingUnit[];
    victory?: DialogueLine[];
    debrief?: string;
  };
}
export interface CampaignDefinition {
  playerDiplomacy: ("pragmatic" | "restrained")[];
  id: CampaignId;
  name: string;
  tagline: string;
  description: string;
  race: Race;
  leader: LeaderType;
  missions: MissionDefinition[];
  ending: DialogueLine[];
}
export interface CampaignState {
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
  choices?: ("refuge-peace" | "refuge-war" | "green-dain-dead")[];
}
export interface CampaignVeteran {
  type: UnitType;
  experience: number;
  name?: string;
  role?: Unit["role"];
}
