val migrate_typed : Js.Json.t -> Current_save.t * bool
(** Read either deployed version-one format or any frozen numbered format, then compose typed
    version-to-version conversions, then narrow the current output to valid browser preferences. No
    intermediate conversion uses an untyped save model. *)

val migrate : Js.Json.t -> Js.Json.t * bool
(** Storage adapter for [migrate_typed]. The source is never mutated; native restoration and
    persistence remain application responsibilities. *)
