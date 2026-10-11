(** Browser embedding owns the React root. [current] prevents a cancelled startup
    from constructing a session or rendering after WASM initialization. Returned
    cleanup removes gesture handlers and closes session/factory exactly once. *)
type root
val mount : root -> Dom.element -> (unit -> bool) -> (unit -> unit) Js.Promise.t
