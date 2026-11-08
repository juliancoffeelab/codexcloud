val observe_error : 'a Js_of_ocaml.Promise.t -> ('a -> unit) -> (string -> unit) -> unit
(** Observe fulfillment or rejection. The rejection callback receives the host error
    converted to text. This does not cancel the operation. *)
