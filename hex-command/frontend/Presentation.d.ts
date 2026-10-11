/** Pure display projections over detached OCaml publications; Rust owns legality. */
import type { PublicUnit, UnitRule, Side, Weapon, DamageType, DamageDetail, Tile, State } from "../rust/pkg/hex_command";
import type { t as Catalog } from "./Catalog";
import type { view as View, Immutable } from "./Native";
export type { Immutable } from "./Native";
export interface ability { readonly name: string; readonly description: string; readonly facts: readonly string[] }
export interface fact { readonly label: string; readonly value: number }
export interface terrain_effect { readonly terrain: string; readonly name: string; readonly label: string; readonly facts: readonly fact[]; readonly class_name: string }
export interface unit_details { readonly unit: Readonly<PublicUnit>; readonly definition: Immutable<UnitRule>; readonly terrain_name: string; readonly action_status: string; readonly promotions: readonly Immutable<UnitRule>[]; readonly ability: ability; readonly terrain_effects: readonly terrain_effect[] }
/** Melange representation: Hidden is 0; payload-bearing variants use numeric TAG. */
export type panel = 0 | { readonly TAG: 0; readonly _0: string; readonly _1: string } | { readonly TAG: 1; readonly _0: unit_details } | { readonly TAG: 2; readonly _0: Readonly<PublicUnit>; readonly _1: Readonly<PublicUnit>; readonly _2: string };
export function unit_panel(catalog: Catalog, view: View, selected: string | undefined, target: string | undefined, revision: number): panel;
export interface forecast { readonly side: Side; readonly remaining_hp: number; readonly current_hp: number; readonly maximum_hp: number; readonly health_label: string; readonly current_percent: number; readonly remaining_percent: number; readonly impact_label: string }
export interface calculation { readonly description: string; readonly equation: string }
export interface combat_result { readonly player: forecast; readonly enemy: forecast; readonly attack: calculation; readonly retaliation: calculation }
export interface attack { readonly weapon: Immutable<Weapon>; readonly button_label: string; readonly result: combat_result | undefined }
export function attacks(catalog: Catalog, view: View, attacker: Immutable<PublicUnit>, defender: Immutable<PublicUnit>): readonly attack[];
export function damage_equation(detail: Immutable<DamageDetail>): string;
export function signed_value(value: number): string;
export function terrain_damage_facts(rows: number[][]): readonly fact[];
export function ability(catalog: Catalog, unit: Immutable<PublicUnit>): ability;
export interface modifier { readonly name: string; readonly label: string; readonly class_name: string }
export function armor(catalog: Catalog, unit: Immutable<PublicUnit>): readonly modifier[];
export function weapon_range(weapon: Immutable<Weapon>): string;
export function damage_name(type: DamageType): string;
export interface recruit { readonly definition: Immutable<UnitRule>; readonly enabled: boolean }
export interface recruitment { readonly options: readonly recruit[]; readonly hint: string }
export interface terrain_panel { readonly tile: Readonly<Tile>; readonly name: string; readonly description: string; readonly ownership: readonly [string, string] | undefined; readonly facts: readonly string[]; readonly recruitment: recruitment | undefined }
export function terrain_panel(catalog: Catalog, view: View, selected: string | undefined, inspected: string | undefined): terrain_panel | undefined;
export interface tile_display { readonly tile: Readonly<Tile>; readonly occupant: Readonly<PublicUnit> | undefined; readonly classes: string; readonly label: string; readonly owner_color: string; readonly health_width: number }
export function battlefield(catalog: Catalog, view: View, selected: string | undefined, inspected: string | undefined): readonly tile_display[];
/** Nullary OCaml variants: Move=0, Attack=1, Recruit=2. */
export type motion = 0 | 1 | 2;
export interface changes { readonly tile_effects: readonly (readonly [string, motion])[]; readonly captured_tiles: readonly string[]; readonly turn_changed: boolean; readonly new_winner: Side | undefined; readonly attacked: boolean; readonly recruited: boolean; readonly advanced: boolean; readonly moved_unit: Readonly<PublicUnit> | undefined; readonly attacker: Readonly<PublicUnit> | undefined }
export function changes(before: Immutable<State>, after: Immutable<State>): changes;
