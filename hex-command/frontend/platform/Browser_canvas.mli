type context
type image

external save : context -> unit = "save" [@@mel.send]
external restore : context -> unit = "restore" [@@mel.send]
external set_filter : context -> string -> unit = "filter" [@@mel.set]
external set_smoothing : context -> bool -> unit = "imageSmoothingEnabled" [@@mel.set]
external scale : context -> float -> float -> unit = "scale" [@@mel.send]

external draw : context -> image -> float -> float -> float -> float -> unit = "drawImage"
[@@mel.send]

type gradient

external fill_rect : context -> float -> float -> float -> float -> unit = "fillRect" [@@mel.send]

external gradient : context -> float -> float -> float -> float -> float -> float -> gradient
  = "createRadialGradient"
[@@mel.send]

external stop : gradient -> float -> string -> unit = "addColorStop" [@@mel.send]
external fill : context -> string -> unit = "fillStyle" [@@mel.set]
external fill_gradient : context -> gradient -> unit = "fillStyle" [@@mel.set]
external composite : context -> string -> unit = "globalCompositeOperation" [@@mel.set]
external begin_path : context -> unit = "beginPath" [@@mel.send]
external close_path : context -> unit = "closePath" [@@mel.send]
external move_to : context -> float -> float -> unit = "moveTo" [@@mel.send]
external line_to : context -> float -> float -> unit = "lineTo" [@@mel.send]
external fill_path : context -> unit = "fill" [@@mel.send]
external filter : context -> string -> unit = "filter" [@@mel.set]
