(** Browser operations used by this frontend. Host values stay in their native
    representation. *)

type element
type event
type native_string
type worker
type worker_options
type message = { kind : native_string; value : native_string }

external to_host : string -> native_string = "host_string"
[@@mel.scope "Ocamljs_async_search_demo"]
(** Convert an OCaml string to a host JavaScript string using the generated runtime. *)

external of_host : native_string -> string = "native_string"
[@@mel.scope "Ocamljs_async_search_demo"]
(** Convert a host JavaScript string to an OCaml string. *)

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
external get_value : element -> native_string = "value" [@@mel.get]

external set_attribute : element -> native_string -> native_string -> unit
  = "setAttribute"
[@@mel.send]

external listen_raw : element -> native_string -> (event -> unit) -> unit
  = "addEventListener"
[@@mel.send]

external new_worker : native_string -> worker_options -> worker = "Worker" [@@mel.new]
external terminate : worker -> unit = "terminate" [@@mel.send]
external post : worker -> message -> unit = "postMessage" [@@mel.send]
external worker_message : worker -> (event -> unit) -> unit = "onmessage" [@@mel.set]
external worker_error : worker -> (event -> unit) -> unit = "onerror" [@@mel.set]

external worker_message_error : worker -> (event -> unit) -> unit = "onmessageerror"
[@@mel.set]

external post_global : message -> unit = "postMessage"
external close_worker : unit -> unit = "close"

val worker_options : unit -> worker_options
(** Create options for an ES module worker. *)

val message_valid : message -> int -> bool
(** Check a message against the supplied maximum payload length. *)

val data : event -> message option
(** Read an event payload, returning None if absent. Use message_valid before accessing
    its fields. *)

val error_text : 'a -> string
(** Convert a host rejection value to display text. *)

val reload : unit -> unit
(** Reload the current page. *)

val exception_text : exn -> string
(** Render an OCaml or host JavaScript exception for display. *)

val rejection_text : event -> string
external prevent_default : event -> unit = "preventDefault" [@@mel.send]
