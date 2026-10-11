/** Bounded scene ownership and cancellable native bitmap generation in OCaml. */
import type { State } from '../rust/pkg/hex_command';
import type { Immutable } from './Native';
import type { Artwork } from './Art';
import type { TerrainTile,Dimensions } from './Terrain_plan';
export interface Sprite {readonly url:string;readonly width:number;readonly height:number;readonly displayWidth:number;readonly displayHeight:number}
export interface Owner {readonly _owner:unique symbol}
export interface Scene {readonly _scene:unique symbol}
export interface Preview {readonly _preview:unique symbol}
export interface Bitmap {readonly _bitmap:unique symbol}
export function create(manifest:Readonly<Record<string,Sprite>>):Owner;
export function prepare(owner:Owner,state:Immutable<State>,artwork:Artwork):Scene;
export function activate(owner:Owner,scene:Scene):void;
export function key(scene:Scene):string;
export function tiles(scene:Scene):readonly TerrainTile[];
export function inspection_urls(scene:Scene,artwork:Artwork):readonly string[];
export function get_tile(scene:Scene,id:string):TerrainTile|undefined;
export function size(scene:Scene):{readonly width:number;readonly height:number};
export function preview(owner:Owner,scene:Scene):Preview|undefined;
export function sprite(preview:Preview):Sprite;
export function changed(preview:Preview):readonly string[];
export function changed_contains(preview:Preview,id:string):boolean;
export function bake(owner:Owner,scene:Scene):Promise<Bitmap|undefined>;
export function valid_bitmap(scene:Scene,bitmap:Bitmap):boolean;
export function bitmap_size(bitmap:Bitmap):Dimensions;
export function close(owner:Owner):void;
export function mount(element:Element,bitmap:Bitmap):void;
export function clear(element:Element):void;
export function preload(owner:Owner,mapId:string):void;
export function preload_manifest(manifest:Readonly<Record<string,Sprite>>,mapId:string):void;
