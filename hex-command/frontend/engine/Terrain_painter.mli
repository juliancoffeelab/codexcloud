type tile = {
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
(** Complete frame paint policy shared by offline assets and the browser worker. This module does
    not load artwork catalogs or resolve terrain geometry. *)

type dimensions = { width : float; height : float; display_width : float; display_height : float }
type context = Browser_canvas.context
type image = Browser_canvas.image

val dimensions : width:float -> height:float -> pixel_scale:float -> dimensions
val configure : context -> float -> unit

val paint : context -> tile array -> image Js.Dict.t -> unit
(** Draw caller-owned tiles in stable authored depth order without vector clipping. Every referenced
    frame must have a decoded image. *)
