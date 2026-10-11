(** Hex-grid coordinates, independent of the rendering layer. *)
type point = { x : float; y : float }
type dimensions = { width : float; height : float }
val sprite_size : float
val native_pixel_scale : float
val column_spacing : float
val row_spacing : float
val center : col:int -> row:int -> point
val hex_points : float -> float -> string
val dimensions : cols:int -> rows:int -> dimensions
val preview_dimensions : cols:int -> rows:int -> dimensions
val preview_color : string -> string

