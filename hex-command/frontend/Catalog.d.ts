/** Read-only indexed metadata from the Rust-validated catalog. */
import type {CatalogView,UnitRule,TerrainDefinition,RaceDefinition,MapDefinition,Campaign,Mission} from "../rust/pkg/hex_command";
import type {Immutable} from "./Native";
declare const catalogBrand:unique symbol;
export interface t {readonly [catalogBrand]:true}
export function of_view(view:CatalogView|Immutable<CatalogView>):t;
export function raw(catalog:t):Immutable<CatalogView>;
export function units(catalog:t):readonly Immutable<UnitRule>[];
export function terrain(catalog:t):readonly Immutable<TerrainDefinition>[];
export function races(catalog:t):readonly Immutable<RaceDefinition>[];
export function maps(catalog:t):readonly Immutable<MapDefinition>[];
export function campaigns(catalog:t):readonly Immutable<Campaign>[];
export function unit(catalog:t,id:string):Immutable<UnitRule>;
export function terrain_type(catalog:t,id:string):Immutable<TerrainDefinition>;
export function race(catalog:t,id:string):Immutable<RaceDefinition>;
export function map(catalog:t,id:string):Immutable<MapDefinition>;
export function campaign(catalog:t,id:string):Immutable<Campaign>;
/** Visible skirmish choices; hidden layouts and campaign maps are excluded. */
export function battle_maps(catalog:t):readonly Immutable<MapDefinition>[];
/** Command-addressable scenarios, including hidden developer layouts. */
export function scenario_maps(catalog:t):readonly Immutable<MapDefinition>[];
export function mission_map(catalog:t,mission:Mission|Immutable<Mission>):Immutable<MapDefinition>;
