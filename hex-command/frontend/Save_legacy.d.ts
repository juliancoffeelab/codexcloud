/** Private historical snapshot compatibility, implemented entirely in OCaml. */
import type {CatalogView,State} from "../rust/pkg/hex_command";
import type {Immutable} from "./Native";
export function $$import(catalog:Immutable<CatalogView>,snapshot:unknown):Immutable<State>;
export function wave_marker(mark:string,waves:readonly {id:string;legacyIndex?:number}[]):string;
