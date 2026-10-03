export type Side = "player" | "enemy";
export type LeaderType = "commander" | "wildkeeper" | "necromancer";
export type UnitType =
  | LeaderType
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
export type RecruitType = Exclude<UnitType, LeaderType>;
export type Race = "human" | "undead" | "beast";
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
export type CampaignId = "lantern-road" | "green-accord" | "ashen-oath";
export type CampaignMapId =
  | "lantern-ferry"
  | "lantern-villages"
  | "lantern-bell"
  | "green-border"
  | "green-vigil"
  | "green-pact"
  | "ashen-quarry"
  | "ashen-refuge"
  | "ashen-dawn";
export type MapId = "mirror" | "river" | CampaignMapId;
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
  race?: Race;
  leader?: boolean;
  woodland?: boolean;
}
export interface TerrainDefinition {
  name: string;
  cost: number;
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

export interface DialogueLine {
  speaker: string;
  portrait: UnitType;
  text: string;
}
export type MissionGoal =
  | { kind: "escort"; label: string; tile: string }
  | { kind: "escape"; label: string; tile: string }
  | { kind: "control"; label: string; tiles: string[] }
  | { kind: "hold"; label: string; tile: string; round: number }
  | { kind: "refuge"; label: string; tile: string; turns: number }
  | { kind: "seal"; label: string; tiles: string[] }
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
  round: number;
  label: string;
  units: StartingUnit[];
  requiresUnsealed?: string;
}
export interface MissionDefinition {
  map: MapDefinition;
  subtitle: string;
  briefing: string;
  hints: string[];
  goals: MissionGoal[];
  recruits: RecruitType[];
  enemyRecruits: RecruitType[];
  gold: number;
  enemyGold: number;
  intro: DialogueLine[];
  victory: DialogueLine[];
  events: MissionEvent[];
  waves: ReinforcementWave[];
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
  carryGold: number;
  marks: string[];
  dialogue: DialogueLine[];
  dialogueIndex: number;
  outcome: string | null;
  completed: boolean;
  refuge?: { heldTurns: number; lastCountedRound: number; escalated: boolean };
  choices?: ("refuge-peace" | "refuge-war")[];
}
export type Attack = Weapon;
export type DamageDetail = DamageBreakdown;
