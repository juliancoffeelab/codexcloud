export type Side = "player" | "enemy";
export type LeaderType =
  | "commander"
  | "wildkeeper"
  | "necromancer"
  | "tidekeeper"
  | "marshal"
  | "grovewarden"
  | "lich"
  | "tideseer";
export type UnitType =
  | LeaderType
  | AdvancedTroop
  | "swordsman"
  | "spearman"
  | "reefguard"
  | "harpooner"
  | "wavemender"
  | "soldier"
  | "archer"
  | "scout"
  | "warden"
  | "adept"
  | "skeleton"
  | "bonearcher"
  | "wolf"
  | "boar"
  | "thorncaller";
export type AdvancedTroop =
  | "marksman"
  | "ranger"
  | "mage"
  | "revenant"
  | "deatharcher"
  | "direwolf"
  | "greatboar"
  | "briarweaver"
  | "tridentguard"
  | "tidehunter"
  | "tidesage";
export type RecruitType = Exclude<UnitType, LeaderType>;
export const UNIT_TYPES = [
  "commander",
  "soldier",
  "archer",
  "scout",
  "warden",
  "adept",
  "skeleton",
  "bonearcher",
  "wolf",
  "boar",
  "wildkeeper",
  "necromancer",
  "thorncaller",
  "swordsman",
  "spearman",
  "tidekeeper",
  "reefguard",
  "harpooner",
  "wavemender",
  "marksman",
  "ranger",
  "mage",
  "revenant",
  "deatharcher",
  "direwolf",
  "greatboar",
  "briarweaver",
  "tridentguard",
  "tidehunter",
  "tidesage",
  "marshal",
  "grovewarden",
  "lich",
  "tideseer",
] as const satisfies readonly UnitType[];
export type Race = "human" | "undead" | "beast" | "merfolk";
export type TerrainType =
  | "plains"
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
export type MapId = "mirror" | "river" | "developer" | CampaignMapId | CoastalMapId;
export interface Coordinates {
  col: number;
  row: number;
}
export interface Tile extends Coordinates {
  id: string;
  terrain: TerrainType;
  owner: Side | null;
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
  id: string;
  name: string;
  damage: number;
  damageType: DamageType;
  category: AttackCategory;
  range: number;
  sound: string;
}
export interface UnitDefinition {
  level?: number;
  family?: UnitType;
  experienceNeeded?: number;
  advancesTo?: UnitType[];
  /** Adjacent allies healed at the start of this unit's side's turn. */
  healing?: number;
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
  /** Overrides applied after the race defaults. */
  terrainModifiers?: Partial<Record<TerrainType, TerrainModifier>>;
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
}
export interface StartingUnit extends Coordinates {
  type: UnitType;
  side: Side;
  name?: string;
  role?: "leader" | "courier";
}
export interface MapDefinition {
  id: MapId;
  revision?: number;
  name: string;
  description: string;
  cols: number;
  rows: number;
  terrain: TerrainOverride[];
  units: StartingUnit[];
}
export interface DamageBreakdown {
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
  | { kind: "hold"; label: string; tile: string; turns: number; earliestRound: number }
  | { kind: "vigil"; label: string; tile: string; turns: number; companion: string }
  | { kind: "refuge"; label: string; tile: string; turns: number }
  | { kind: "seal"; label: string; tiles: string[]; unitType?: UnitType }
  | { kind: "event"; label: string; tile: string; event: string }
  | { kind: "defeat"; label: string };
export interface MissionEvent {
  id: string;
  trigger: { kind: "visit"; tile: string } | { kind: "round"; round: number };
  label?: string;
  dialogue: DialogueLine[];
  gold?: number;
  warDialogue?: DialogueLine[];
}
export interface ReinforcementWave {
  /** Stable within its mission; saved deployment must never depend on array order. */
  id: string;
  /** Position used by the last published numeric-marker save format. */
  legacyIndex: number;
  round: number;
  label: string;
  units: StartingUnit[];
  /** This wave replaces troops already present in older saved battles. */
  previouslyDeployedBeforeRevision?: number;
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
  /** Previously published lines remain valid in ongoing saved conversations. */
  legacyDialogue?: DialogueLine[];
}
export interface CampaignDefinition {
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
export type Attack = Weapon;
export type DamageDetail = DamageBreakdown;
