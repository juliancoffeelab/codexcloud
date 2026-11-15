type root

(** Browser embedding owns the React root. [current] prevents a cancelled startup from constructing
    a session or rendering after WASM initialization. Returned cleanup removes gesture handlers,
    unregisters optional browser tools and closes session/factory exactly once. [failed] reports
    descendant render and effect failures to that owner; it must schedule cleanup outside the
    render. The boundary retains a factory, creating the App element inside its callback rather than
    retaining a rendered descendant element in its props. *)

val mount :
  root ->
  Dom.element ->
  (unit -> bool) ->
  (Foreign_error.t -> unit) ->
  (unit -> unit, string) result Js.Promise.t
