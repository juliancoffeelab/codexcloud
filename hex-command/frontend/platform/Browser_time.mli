type timer

external now : unit -> float = "now" [@@mel.scope "performance"]
external origin : float = "timeOrigin" [@@mel.scope "performance"]
external microtask : (unit -> unit) -> unit = "queueMicrotask"
external timeout : (unit -> unit) -> int -> timer = "setTimeout"
external timeout_float : (unit -> unit) -> float -> timer = "setTimeout"
external clear_timeout : timer -> unit = "clearTimeout"
external frame : (unit -> unit) -> int = "requestAnimationFrame"
external cancel_frame : int -> unit = "cancelAnimationFrame"
