type 'a described = { original : 'a; description : string }

(** Typed test support, excluded from application entry points. *)

val samples : unit -> string array
val generic : unit -> exn described
val foreign : unit -> Js.Exn.t described
