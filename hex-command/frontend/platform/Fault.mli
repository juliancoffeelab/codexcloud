val raise_original : exn -> 'a

(** Preserve a foreign exception's original identity when releasing resources; OCaml exceptions
    retain their usual representation. *)

val describe : exn -> string

(** Describe an exception without generic value formatting. JavaScript messages and common OCaml
    payloads/source locations are retained. The original JavaScript error or OCaml exception is
    logged for its stack and other fields; other OCaml exceptions are described by their supported
    exception-slot name. *)
