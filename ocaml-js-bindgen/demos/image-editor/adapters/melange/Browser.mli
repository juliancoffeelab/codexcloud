(** Browser operations used by this frontend. Host values stay in their native
    representation. *)

type element
type event
type rect
type style
type native_string
type events
type touches
type touch
type listener_options = { passive : bool }

external style : element -> style = "style" [@@mel.get]
external touches : event -> touches = "touches" [@@mel.get]

external to_host : string -> native_string = "host_string"
[@@mel.scope "Ocamljs_image_editor_demo"]
(** Convert an OCaml string to a host JavaScript string using the generated runtime. *)

external of_host : native_string -> string = "native_string"
[@@mel.scope "Ocamljs_image_editor_demo"]
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
external append_child : element -> element -> unit = "appendChild" [@@mel.send]
external get_value : element -> native_string = "value" [@@mel.get]
external set_value : element -> native_string -> unit = "value" [@@mel.set]

external set_attribute : element -> native_string -> native_string -> unit
  = "setAttribute"
[@@mel.send]

val get_attribute : element -> native_string -> native_string option
val closest : element -> native_string -> element option
external click : element -> unit = "click" [@@mel.send]

external listen_raw : element -> native_string -> (event -> unit) -> unit
  = "addEventListener"
[@@mel.send]

external listen_options :
  element -> native_string -> (event -> unit) -> listener_options -> unit
  = "addEventListener"
[@@mel.send]

external frame_raw : (float -> unit) -> int = "requestAnimationFrame"
external cancel_frame : int -> unit = "cancelAnimationFrame"
external set_disabled : element -> bool -> unit = "disabled" [@@mel.set]
external disabled : element -> bool = "disabled" [@@mel.get]
external set_hidden : element -> bool -> unit = "hidden" [@@mel.set]
external width : element -> float = "width" [@@mel.get]
external height : element -> float = "height" [@@mel.get]
external client_width : element -> float = "clientWidth" [@@mel.get]
external client_height : element -> float = "clientHeight" [@@mel.get]
external bounds : element -> rect = "getBoundingClientRect" [@@mel.send]
external left : rect -> float = "left" [@@mel.get]
external top : rect -> float = "top" [@@mel.get]
external rect_width : rect -> float = "width" [@@mel.get]
external rect_height : rect -> float = "height" [@@mel.get]

external set_style : style -> native_string -> native_string -> unit = "setProperty"
[@@mel.send]

external capture : element -> int -> unit = "setPointerCapture" [@@mel.send]
external target : event -> element = "target" [@@mel.get]
external client_x : event -> float = "clientX" [@@mel.get]
external client_y : event -> float = "clientY" [@@mel.get]
external pointer_id : event -> int = "pointerId" [@@mel.get]
external button : event -> int = "button" [@@mel.get]
external pointer_type_raw : event -> native_string = "pointerType" [@@mel.get]
external prevent_default : event -> unit = "preventDefault" [@@mel.send]
external events_length : events -> int = "length" [@@mel.get]
external event_at : events -> int -> event = "" [@@mel.get_index]
external touches_length : touches -> int = "length" [@@mel.get]
external touch_at : touches -> int -> touch = "item" [@@mel.send]
external touch_x : touch -> float = "clientX" [@@mel.get]
external touch_y : touch -> float = "clientY" [@@mel.get]
external delta_y : event -> float = "deltaY" [@@mel.get]
external delta_mode : event -> int = "deltaMode" [@@mel.get]
external key_raw : event -> native_string = "key" [@@mel.get]
external ctrl : event -> bool = "ctrlKey" [@@mel.get]
external meta : event -> bool = "metaKey" [@@mel.get]
external shift : event -> bool = "shiftKey" [@@mel.get]
val listener_options : bool -> listener_options
val coalesced : event -> events option

val error_text : 'a -> string
(** Convert a host rejection value to display text. *)

val reload : unit -> unit
(** Reload the current page. *)

val save_png : Bindings.Uint8ClampedArray.t -> unit

val exception_text : exn -> string
(** Render an OCaml or host JavaScript exception for display. *)
