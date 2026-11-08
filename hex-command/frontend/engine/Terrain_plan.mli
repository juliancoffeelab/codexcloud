type tile = Terrain_painter.tile = {
  col : float;
  row : float;
  id : string;
  terrain : string;
  x : float;
  y : float;
  points : string;
  sprite_size : float;
  sprite_x : float;
  sprite_y : float;
  sprite_width : float;
  sprite_height : float;
  role : Domain.Terrain_role.t;
  fill : string;
  name : string;
  floor : string option;
  frame : Art.frame option;
}
(** Portable static-terrain policy used by browser scenes and the Node asset builder. Geometry,
    frame selection, anchors and keys have one implementation. *)

type dimensions = Terrain_painter.dimensions = {
  width : float;
  height : float;
  display_width : float;
  display_height : float;
}

type t = private {
  key : string;
  map_id : string;
  cols : float;
  rows : float;
  width : float;
  height : float;
  pixel_scale : float;
  tiles : tile array;
}

val key : map_id:string -> cols:float -> rows:float -> Domain.tile array -> string
(** Stable row/column ordering; units, ownership and turn state do not affect keys. *)

val same_tile : Domain.tile -> Domain.tile -> bool

val layout_name : Domain.tile -> string -> string
(** Deterministic authored prop layout chosen from the map and cell identifiers. *)

val prepare :
  roles:Domain.Terrain_role.t Js.Dict.t ->
  map_id:string ->
  cols:float ->
  rows:float ->
  Domain.tile array ->
  Art.t ->
  t
(** Build a fresh caller-owned drawing plan. Its mutable tile array is independent of the input
    state and of retained Terrain scenes; tile/frame records are shared. Terrain retains its own
    plans behind its scene interface. *)

val dimensions : t -> dimensions
(** Round native bitmap bounds upward and return their exact logical display size. *)

type context = Terrain_painter.context
(** Native canvas contexts and image objects pass directly through this boundary. *)

type image = Terrain_painter.image

val configure : context -> float -> unit

val paint : context -> tile array -> image Js.Dict.t -> unit
(** Paint complete alpha frames in authored depth order. Every declared frame must have a decoded
    image; missing images raise so the owner can discard and retry an incomplete bake. *)

val asset_key : string -> string
