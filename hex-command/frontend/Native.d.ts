/** Direct OCaml owner of raw WASM resources and immutable publications. */
import type {State,BattleInspection,Start,SavedGame,Command,Effects,AiOptions,Tile,PublicUnit,UnitActions,AttackDetails,Side} from "../rust/pkg/hex_command";
import type {t as Catalog} from "./Catalog";
/** Detached recursively immutable native JSON. */
export type Immutable<T> = T extends object ? {readonly [K in keyof T]:Immutable<T[K]>} : T;
declare const factoryBrand:unique symbol;
declare const battleBrand:unique symbol;
declare const viewBrand:unique symbol;
export interface factory {readonly [factoryBrand]:true}
export interface battle {readonly [battleBrand]:true}
export interface view {readonly [viewBrand]:true}
export function create_factory():Promise<factory>;
export function catalog(factory:factory):Catalog;
export function dispose_factory(factory:factory):void;
export function create(factory:factory,start:Start,seed:number):battle;
export function restore(factory:factory,saved:SavedGame|Immutable<SavedGame>):battle;
export function dispose(battle:battle):void;
/** Notify the owner after an accepted mutation fails to publish and releases its handle. */
export function on_failure(battle:battle,receive:(reason:string)=>void):void;
export function view(battle:battle):view;
export function state(view:view):Immutable<State>;
export function inspection(view:view):Immutable<BattleInspection>;
export function revision(view:view):number;
export function dispatch(battle:battle,command:Command|Immutable<Command>):Immutable<Effects>;
export function enemy_step(battle:battle,options:AiOptions):Immutable<Effects>;
export function save(battle:battle):Immutable<SavedGame>;
export function replace(battle:battle,saved:SavedGame|Immutable<SavedGame>):void;
export function tile(view:view,id:string):Immutable<Tile>|undefined;
export function unit(view:view,id:string):Immutable<PublicUnit>|undefined;
export function at(view:view,id:string):Immutable<PublicUnit>|undefined;
export function actions(view:view,id:string):Immutable<UnitActions>|undefined;
export function neighbors(view:view,id:string):readonly Immutable<Tile>[];
export function reachable(view:view,id:string):Readonly<Record<string,number>>;
export function targets(view:view,id:string):readonly Immutable<PublicUnit>[];
export function attack_options(view:view,id:string,target:string):readonly Immutable<AttackDetails>[];
export function best_attack(view:view,id:string,target:string):Immutable<AttackDetails>|undefined;
export function attack_details(view:view,id:string,target:string,weapon:string):Immutable<AttackDetails>|undefined;
export function promotion_options(view:view,id:string):readonly string[];
export function recruitment_options(view:view,side:Side):readonly string[];
export function recruit_tiles(view:view,id:string,type:string):readonly Immutable<Tile>[];
export function movement_cost(view:view,type:string,terrain:string):number;
export function income(view:view,side:Side):number;
export function unit_id(id:string):number;
