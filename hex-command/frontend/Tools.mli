val register : Session.t -> unit -> unit

(** Optional browser model-context tools. Inputs and results use ordinary browser JSON; all
    mutations use the same Session commands as the interface. Returns idempotent cleanup that
    disables retained callbacks. Operations for each tool name are serialized across mounts; the
    latest registration owns the registry's tool names. A browser operation's returned promise must
    settle after its registry side effect completes. Registration failures never block play, and
    cleanup cannot unregister a newer owner. *)
