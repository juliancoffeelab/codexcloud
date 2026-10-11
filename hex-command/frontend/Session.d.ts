/** Stable embedding contract for the OCaml battle owner. */
import type {factory,view as NativeView} from "./Native";
import type {t as Catalog} from "./Catalog";
import type {Artwork} from "./Art";
declare const sessionBrand:unique symbol;
export interface t {readonly [sessionBrand]:true}
export interface model {
  readonly game:NativeView;
  readonly catalog:Catalog;
  readonly artwork:Artwork;
  readonly selected:string|undefined;
  readonly target:string|undefined;
  readonly inspected:string|undefined;
  readonly focused:string;
  readonly revision:number;
  readonly status:string;
  readonly sound:boolean;
  readonly can_undo:boolean;
}
export interface actions {
  readonly choose:(tile:string)=>void;
  readonly focus:(tile:string)=>void;
  readonly clear:(inspection:boolean)=>void;
  readonly attack:(weapon:string)=>void;
  readonly recruit:(type:string,tile:string)=>void;
  readonly advance:(type:string)=>void;
  readonly end_turn:()=>Promise<void>;
  readonly undo:()=>void;
  readonly reset:(map:string)=>void;
  readonly start_campaign:(id:string)=>void;
  readonly next_mission:(ids:string[])=>void;
  readonly retry:()=>void;
  readonly dialogue:(skip:boolean)=>void;
  readonly previous_dialogue:()=>void;
  readonly set_sound:(enabled:boolean)=>Promise<void>;
}
export type clock=(milliseconds:number)=>Promise<void>;
/** The embedding retains factory ownership; close releases this session only. */
export function create(clock:clock|undefined,factory:factory):t;
export function view(session:t):model;
export function commands(session:t):actions;
export function subscribe(session:t,receive:(view:model)=>void):()=>void;
export function rendered(session:t,revision:number):void;
export function initialize(session:t):void;
export function close(session:t):void;
export function message(session:t,status:string):void;
export function error(session:t,error:unknown):void;
export function gesture(session:t,trusted:boolean):void;
export function pause(milliseconds:number):Promise<void>;
