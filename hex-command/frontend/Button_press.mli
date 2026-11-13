val pointer_down : (unit -> unit) -> React.Event.Pointer.t -> unit
(** Attach both handlers only to fixed buttons that explicitly act on press. Native onClick remains
    the default everywhere else. Mark these buttons with the [press-control] class so HUD audio
    follows their press activation and skips the subsequent pointer click. *)

val click : (unit -> unit) -> React.Event.Mouse.t -> unit
