(** Browser operations used by this frontend. Host values stay in their native
    representation. *)

type element
type event
type native_string

external to_host : string -> native_string = "host_string"
[@@mel.scope "Ocamljs_life_demo"]
(** Convert an OCaml string to a host JavaScript string using the generated runtime. *)

external global : element = "globalThis"
external document_of : element -> element = "document" [@@mel.get]
external document_element : element -> element = "documentElement" [@@mel.get]
external body : element -> element = "body" [@@mel.get]

val find_id : element -> native_string -> element option
(** Find an element by id, returning None when absent. *)

external create_element : element -> native_string -> element = "createElement"
[@@mel.send]

external set_text : element -> native_string -> unit = "textContent" [@@mel.set]
external replace_children : element -> unit = "replaceChildren" [@@mel.send]
external append_child : element -> element -> unit = "appendChild" [@@mel.send]
external set_class : element -> native_string -> unit = "className" [@@mel.set]

external set_attribute : element -> native_string -> native_string -> unit
  = "setAttribute"
[@@mel.send]

external listen_raw : element -> native_string -> (event -> unit) -> unit
  = "addEventListener"
[@@mel.send]

val error_text : 'a -> string
(** Convert a host rejection value to display text. *)

val reload : unit -> unit
(** Reload the current page. *)

val exception_text : exn -> string
(** Render an OCaml or host JavaScript exception for display. *)
