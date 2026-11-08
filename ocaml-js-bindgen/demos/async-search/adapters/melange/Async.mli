val observe_error : 'a Js.Promise.t -> ('a -> unit) -> (string -> unit) -> unit
(** Observe fulfillment or rejection. The rejection callback receives the host error
    converted to text. This does not cancel the operation. *)

val observe : 'a Js.Promise.t -> ('a -> unit) -> (unit -> unit) -> unit
(** Observe fulfillment or rejection, discarding the rejection reason. This does not
    cancel the operation. *)
