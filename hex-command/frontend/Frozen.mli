(** Take ownership of detached, acyclic foreign data by freezing every object in
    place. Already shallow-frozen inputs still have their children frozen. No
    graph copies are made. Foreign arrays/dictionaries remain writable in OCaml's
    types; writes to owned publications are rejected at runtime. *)
val own : 'a -> 'a
