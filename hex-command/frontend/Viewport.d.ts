/** Native inspection visibility and callback lifecycle, implemented in OCaml. */
export interface Rect {readonly left:number;readonly right:number;readonly top:number;readonly bottom:number}
export interface Extent {readonly width:number;readonly height:number;readonly scroll_width:number;readonly scroll_height:number;readonly scroll_left:number;readonly scroll_top:number}
export interface Scroll {readonly left:number;readonly top:number}
export function position(area:Rect,viewport:Extent,bounds:readonly Rect[]):Scroll;
export function reveal_map(scroller:Element,board:Element,ids:readonly string[]):()=>void;
export function reveal_attack(panel:Element):()=>void;
