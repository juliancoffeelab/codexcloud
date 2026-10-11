(** Hex-grid coordinates, independent of the rendering layer. *)
type point = { x : float; y : float }
type dimensions = { width : float; height : float }
val center : col:int -> row:int -> point
val hex_points : float -> float -> string
val dimensions : cols:int -> rows:int -> dimensions
val color : string -> string
val preview_color : string -> string

