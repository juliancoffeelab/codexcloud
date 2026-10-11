/** Static geometry, frame selection, clipping and cache keys implemented in OCaml. */
import type { Tile } from '../rust/pkg/hex_command';
import type { Immutable } from './Native';
import type { Artwork, Frame } from './Art';
export interface TerrainTile { readonly id:string; readonly terrain:string; readonly x:number; readonly y:number; readonly points:string; readonly clip_id:string; readonly sprite_size:number; readonly fill:string; readonly name:string; readonly floor:string|undefined; readonly frame:Frame|undefined }
export interface Dimensions { readonly width:number; readonly height:number; readonly display_width:number; readonly display_height:number }
export interface Plan { readonly key:string; readonly map_id:string; readonly cols:number; readonly rows:number; readonly width:number; readonly height:number; readonly pixel_scale:number; readonly tiles:readonly TerrainTile[] }
export function key(mapId:string,cols:number,rows:number,tiles:Immutable<Tile[]>):string;
export function shape(tile:Immutable<Tile>):string;
export function same_tile(a:Immutable<Tile>,b:Immutable<Tile>):boolean;
export function canonical_name(terrain:string):string;
export function layout_name(tile:Immutable<Tile>,mapId:string):string;
export function prepare(mapId:string,cols:number,rows:number,tiles:Immutable<Tile[]>,artwork:Artwork):Plan;
export function dimensions(plan:Plan):Dimensions;
export function configure(context:Pick<CanvasRenderingContext2D,'imageSmoothingEnabled'|'scale'>,scale:number):void;
export function paint(context:Pick<CanvasRenderingContext2D,'save'|'restore'|'beginPath'|'moveTo'|'lineTo'|'closePath'|'fill'|'clip'|'fillStyle'|'filter'> & {drawImage(image:unknown,x:number,y:number,width:number,height:number):void},tiles:readonly TerrainTile[],images:Readonly<Record<string,unknown>>):void;
