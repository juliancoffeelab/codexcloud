/** Scene-specific background warming, implemented in OCaml. */
import type {Scene} from './Terrain';
import type {Artwork} from './Art';
export interface Owner {readonly _prefetch:unique symbol}
export type Cancel=()=>void;
export type Defer=(task:()=>void)=>Cancel;
export type Load=(url:string)=>Promise<void>;
export function create(defer?:Defer,load?:Load,unit?:undefined):Owner;
export function start(owner:Owner,scene:Scene,artwork:Artwork):Cancel;
export function close(owner:Owner):void;
