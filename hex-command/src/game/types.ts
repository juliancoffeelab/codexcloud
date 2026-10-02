export type Side = "player" | "enemy";
export type UnitType = "commander" | "soldier" | "archer" | "scout";
export type TerrainType =
  "plains" | "forest" | "hill" | "water" | "bridge" | "ford" | "village" | "keep";
export type DamageType = "blade" | "pierce" | "impact";
export type AttackCategory = "melee" | "ranged";
export type MapId = "mirror" | "river";
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
}
export type Attack = Weapon;
export type DamageDetail = DamageBreakdown;
