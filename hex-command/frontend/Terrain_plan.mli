(** Portable static-terrain policy used by browser scenes and the Node asset
    builder. Geometry, frame selection, clipping and keys have one implementation. *)
type tile = { id : string; terrain : string; x : float; y : float;
              points : string; clip_id : string; sprite_size : float;
              fill : string; name : string; floor : string option;
              frame : Art.frame option }
type dimensions = { width : float; height : float; display_width : float;
                    display_height : float }
type t = private { key : string; map_id : string; cols : float; rows : float;
                   width : float; height : float; pixel_scale : float;
                   tiles : tile array }
(** Stable row/column ordering; units, ownership and turn state do not affect keys. *)
val key : map_id:string -> cols:float -> rows:float -> Domain.tile array -> string
val shape : Domain.tile -> string
val same_tile : Domain.tile -> Domain.tile -> bool
(** Deterministic authored prop layout chosen from the map and cell identifiers. *)
val layout_name : Domain.tile -> string -> string
val prepare : map_id:string -> cols:float -> rows:float -> Domain.tile array -> Art.t -> t
(** Round native bitmap bounds upward and return their exact logical display size. *)
val dimensions : t -> dimensions
(** Native canvas contexts and image objects pass directly through this boundary. *)
type context
type image
val configure : context -> float -> unit
(** Paint one clipped batch. Every declared frame must have a decoded image;
    missing images raise so the owner can discard and retry an incomplete bake. *)
val paint : context -> tile array -> image Js.Dict.t -> unit
