type t = Js.Promise.error
(** Arbitrary JavaScript rejections are opaque, including non-Error values. *)

type cause = Rejection of t | Exception of exn

val describe : t -> string
(** Guarded diagnostic text, including available stacks and nested causes. Cyclic causes terminate
    safely. *)

val message : t -> string
val exception_ : exn -> string
val cause : cause -> string

val cause_detail : cause -> string
(** Full diagnostic details for either an OCaml exception or a foreign rejection. *)
