(** Preserve a foreign exception's original identity when releasing resources;
    OCaml exceptions retain their usual representation. *)
val raise_original : exn -> 'a
