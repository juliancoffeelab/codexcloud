(** Browser operations used by this frontend. Host values stay in their native
    representation. *)

module Js = Js_of_ocaml.Js
module Dom = Js_of_ocaml.Dom
module Html = Js_of_ocaml.Dom_html

type element = Html.element Js.t
type event = Html.event Js.t
type rect = Html.clientRect Js.t
type style = Html.cssStyleDeclaration Js.t
type native_string = Js.js_string Js.t
type events
type touches = Html.touchList Js.t
type touch = Html.touch Js.t
type listener_options = { passive : bool }

val to_host : string -> native_string
(** Convert an OCaml string to a host JavaScript string using the generated runtime. *)

val of_host : native_string -> string
(** Convert a host JavaScript string to an OCaml string. *)

val global : Html.window Js.t
val document_of : Html.window Js.t -> Html.document Js.t
val document_element : Html.document Js.t -> element
val body : Html.document Js.t -> Html.bodyElement Js.t

val find_id : Html.document Js.t -> native_string -> element option
(** Find an element by id, returning None when absent. *)

val create_element : Html.document Js.t -> native_string -> element
val set_text : element -> native_string -> unit
val append_child : #Dom.node Js.t -> #Dom.node Js.t -> unit
val get_value : element -> native_string
val set_value : element -> native_string -> unit
val set_attribute : element -> native_string -> native_string -> unit
val get_attribute : element -> native_string -> native_string option
val closest : element -> native_string -> element option
val click : element -> unit
val listen_raw : < .. > Js.t -> native_string -> (#Html.event Js.t -> 'a) -> unit

val listen_options :
  < .. > Js.t -> native_string -> (#Html.event Js.t -> 'a) -> listener_options -> unit

val frame_raw : (float -> unit) -> Html.animation_frame_request_id
val cancel_frame : Html.animation_frame_request_id -> unit
val set_disabled : element -> bool -> unit
val disabled : element -> bool
val set_hidden : element -> bool -> unit
val width : element -> float
val height : element -> float
val client_width : element -> float
val client_height : element -> float
val bounds : element -> rect
val left : rect -> float
val top : rect -> float
val rect_width : rect -> float
val rect_height : rect -> float
val style : element -> style
val set_style : style -> native_string -> native_string -> unit
val capture : element -> int -> unit
val target : event -> element
val client_x : event -> float
val client_y : event -> float
val pointer_id : event -> int
val button : event -> int
val pointer_type_raw : event -> native_string
val prevent_default : 'a #Dom.event Js.t -> unit
val events_length : events -> int
val event_at : events -> int -> event
val touches : event -> touches
val touches_length : touches -> int
val touch_at : touches -> int -> touch
val touch_x : touch -> float
val touch_y : touch -> float
val delta_y : event -> float
val delta_mode : event -> int
val key_raw : event -> native_string
val ctrl : event -> bool
val meta : event -> bool
val shift : event -> bool
val listener_options : bool -> listener_options
val coalesced : event -> events option

(** Convert a host rejection value to display text. *)

val reload : unit -> unit
(** Reload the current page. *)

val save_png : Bindings.Uint8ClampedArray.t -> unit

val exception_text : exn -> string
(** Render an OCaml or host JavaScript exception for display. *)
