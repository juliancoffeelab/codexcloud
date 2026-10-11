/** OCaml persistence owner. Rejected bytes stay protected until an explicit new battle. */
import type {CatalogView,SavedGame} from "../rust/pkg/hex_command";
import type {Immutable} from "./Native";
declare const saveBrand:unique symbol;
export interface t {readonly [saveBrand]:true}
export interface envelope {readonly checkpoint:Immutable<SavedGame>;readonly pending_enemy:boolean}
export interface loaded<Game> {readonly game:Game;readonly pending_enemy:boolean;readonly notice:string|undefined;readonly saves:t}
export const version:number;
export const key:string;
export const rejected_key:string;
export function create():t;
export function resume(owner:t):void;
/** Pure historical conversion; callers must restore natively before committing. */
export function migrate(catalog:Immutable<CatalogView>,input:unknown):[envelope,boolean];
export function load<Game>(catalog:Immutable<CatalogView>,restore:(saved:Immutable<SavedGame>)=>Game,fresh:()=>Game,dispose:(game:Game)=>void,checkpoint:(game:Game)=>Immutable<SavedGame>):loaded<Game>;
export function write(owner:t,checkpoint:Immutable<SavedGame>,pending_enemy:boolean):void;
