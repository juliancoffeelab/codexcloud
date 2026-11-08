type t
(** Direct WebGL values used by the Phaser pipelines. *)

type floats

external parameter : t -> int -> int = "getParameter" [@@mel.send]
external floats : int -> floats = "Float32Array" [@@mel.new]
external set : floats -> int -> float -> unit = "" [@@mel.set_index]

type viewport_values

external viewport_array : viewport_values -> int array = "from" [@@mel.scope "Array"]
external viewport_parameter : t -> int -> viewport_values = "getParameter" [@@mel.send]
