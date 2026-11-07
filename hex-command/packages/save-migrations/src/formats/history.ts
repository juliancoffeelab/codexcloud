/** Frozen historical wire contracts. Shared shapes refer only to other frozen types. */
import type * as Native from "./native-v38";
import type { BattleSave as Save39 } from "./envelope-v39";
import type { SessionSetup } from "./setup-v39";
import type { SaveInfo, TurnEntry } from "./metadata-v39";

export const SAVE_VERSIONS = [
  2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27,
  28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39,
] as const;
export type Version = (typeof SAVE_VERSIONS)[number];

type Before26 =
  | 2
  | 3
  | 4
  | 5
  | 6
  | 7
  | 8
  | 9
  | 10
  | 11
  | 12
  | 13
  | 14
  | 15
  | 16
  | 17
  | 18
  | 19
  | 20
  | 21
  | 22
  | 23
  | 24
  | 25;
type Before28 = Before26 | 26 | 27;
type Before29 = Before28 | 28;
type Before30 = Before29 | 29;
type Before34 = Before30 | 30 | 31 | 32 | 33;
type Before35 = Before34 | 34;
type Before37 = Before35 | 35 | 36;

export type Unit<V extends Version> = Omit<Native.PublicUnit, "character" | "experience"> &
  (V extends Before26 ? { name?: string; character?: string } : { character?: string }) & {
    experience?: number;
  };
export type Veteran<V extends Version> = Omit<Native.Veteran, "character"> &
  (V extends Before26 ? { name?: string; character?: string } : { character?: string });
export type Condition<V extends Version> =
  | Exclude<Native.Condition, { kind: "all" | "any" | "hasUnits" | "noUnits" | "proximity" }>
  | { kind: "all" | "any"; conditions: Condition<V>[] }
  | (Extract<Native.Condition, { kind: "hasUnits" | "noUnits" | "proximity" }> &
      (V extends Before26 ? { name?: string } : {}));
export interface Rule<V extends Version> {
  condition: Condition<V>;
  remember?: boolean;
}
export type Light<V extends Version> = Omit<Native.TileLight, "when"> & { when?: Rule<V> };
export type Tile<V extends Version> = Omit<Native.Tile, "fog" | "light"> & {
  fog?: Rule<V>;
  light?: Light<V>;
} & (V extends 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 | 16
    ? { wallCorners?: number[] }
    : {}) &
  (V extends 17 ? { gates?: string[] } : {});
export type Faction<V extends Version> = Omit<Native.Faction, "transfer"> & {
  transfer?: { to: string; when: Rule<V> };
};
export type LegacyMember = Native.SpawnMember;
export type Scripts<V extends Version> = Omit<
  Native.EventState,
  "completed" | "revealed" | "pending" | "marks"
> & {
  pending: { area: string[]; faction: string; aiGroup?: string | null; units: LegacyMember[] }[];
  marks: { id: number; type: string; tile: string; light?: Light<V>; lightRemembered?: boolean }[];
} & (V extends Before34
    ? { completed?: string[]; revealed?: string[] }
    : { completed: string[]; revealed: string[] });
export type Campaign<V extends Version> = Omit<
  Native.CampaignState,
  "scripts" | "evacuated" | "veterans" | "storyVeterans" | "flags"
> & {
  veterans?: Veteran<V>[];
  storyVeterans?: Veteran<V>[];
  evacuated?: { event: string; unit: Unit<V> }[];
} & (V extends Before28 ? {} : { scripts?: Scripts<V> }) &
  (V extends Before29 ? { choices?: string[] } : { flags: Record<string, Native.FlagChange[]> });
export type State<V extends Version> = V extends 39
  ? Save39["checkpoint"]["state"]
  : Omit<
      Native.State,
      "units" | "tiles" | "factions" | "groundItems" | "campaign" | "gold" | "combatRng"
    > & {
      units: Unit<V>[];
      tiles: Tile<V>[];
      campaign?: Campaign<V>;
      gold: V extends
        2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 | 16 | 17 | 18 | 19 | 20
        ? { player: number; enemy: number }
        : Record<string, number>;
    } & (V extends 2 | 3 | 4 | 5 | 6 ? { factions?: Faction<V>[] } : { factions: Faction<V>[] }) &
      (V extends 2 | 3 | 4 | 5 | 6 | 7
        ? { groundItems?: Native.GroundItem[] }
        : { groundItems: Native.GroundItem[] }) &
      (V extends Before35 ? { combatRng?: number } : { combatRng: number });
export type Setup<V extends Version> = Omit<SessionSetup, "enemy"> & {
  enemy: Omit<Native.AiOptions, "diplomacy"> & {
    diplomacy: V extends Before30
      ? Native.Diplomacy | "pragmatic" | "restrained"
      : Native.Diplomacy;
  };
};
export type Entry<V extends Version> = V extends Before35
  ? SaveInfo & { kind: "turn" | "recovery" | "resume"; parentId?: string }
  : V extends Before37
    ? SaveInfo & { kind: "turn" | "recovery" | "manual"; parentId?: string }
    : Save39["progress"]["entry"];
export type Save<V extends Version> = V extends 39
  ? Save39
  : {
      version: V;
      checkpoint: { seed: number; state: State<V> };
      pendingEnemy: boolean;
    } & (V extends 2 | 3 | 4 ? { planner?: unknown } : {}) &
      (V extends Before35
        ? { setup?: Setup<V>; progress?: { entry: Entry<V> } }
        : { setup: Setup<V>; progress: { entry: Entry<V> } }) &
      (V extends Before34 ? { undo?: unknown } : { undo?: Native.UndoStep[] });
export type History<V extends 1 | 2 | 3 | 4> = {
  version: V;
  entries: (V extends 4
    ? TurnEntry
    : V extends 1
      ? Entry<34>
      : V extends 2
        ? Entry<35> & { kind: "turn" }
        : Entry<36> & { kind: "turn" | "manual" })[];
};
export type AnySave = { [V in Version]: Save<V> }[Version];
/** Frozen JSON fixture entry points use these same readers rather than casts. */
export interface Fragments {
  nativeState38: Native.State;
  factions7: Faction<7>[];
  factions11: Faction<11>[];
  factions21: Faction<21>[];
  factions24: Faction<24>[];
  tiles11: Tile<11>[];
  rule11: Rule<11>;
  delta19: (Partial<Tile<19>> & { id: string })[];
  delta20: (Partial<Tile<20>> & { id: string })[];
  delta21: (Partial<Tile<21>> & { id: string })[];
  light23: Light<23>;
  light28: Light<28> | null;
}

/** Both deployed v1 envelopes; the engine tag selects the native format. */
export interface NativeSave1 {
  engine: "rust-wasm";
  version?: 1;
  pendingEnemy: boolean;
  saved: Save<2>["checkpoint"] & { version: 1 };
}
export interface JavascriptSave1 {
  version: 1;
  pendingEnemy: boolean;
  campaignDeploymentFormat?: "named-waves-v1";
  state: {
    mapId?: string;
    mapRevision?: number | null;
    round: number;
    turn: Native.Side;
    winner: Native.Side | null;
    gold: { player: number; enemy: number };
    nextId: number;
    log: string[];
    units: (Omit<import("../legacy/game-types").Unit, "experience"> & { experience?: number })[];
    villages: import("../legacy/game-types").VillageState[];
    sites?: import("../legacy/game-types").VillageState[];
    campaign?: import("../legacy/campaign-state").LegacyCampaignState;
    lastCombat?: import("../legacy/game-types").LastCombat | null;
  };
}
