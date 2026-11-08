val pointer_down : (unit -> unit) -> React.Event.Pointer.t -> unit
(** Attach both handlers only to fixed buttons that explicitly act on press. Native onClick remains
    the default everywhere else. *)

val click : (unit -> unit) -> React.Event.Mouse.t -> unit
