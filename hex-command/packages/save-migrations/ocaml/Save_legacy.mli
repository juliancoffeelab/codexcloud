val import : Js.Json.t -> Js.Json.t
(** Decode the frozen browser-v1 wire contract, perform its typed pure conversion, and encode the
    resulting version-two native state for import tooling. *)

val wave_marker : string -> Js.Json.t array -> string
(** Tooling adapter for the frozen positional deployment-marker conversion. *)
