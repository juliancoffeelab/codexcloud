type handle = Surface | North_west | North_east | South_west | South_east

val by_id : string -> Browser.element
(** Find a required element by its HTML id. Raises Failure if it is absent. *)

val text : Browser.element -> string -> unit
(** Replace the element text content. *)

val value : Browser.element -> string
(** Read the current form control value. *)

val set_value : Browser.element -> string -> unit
(** Set the form control value. *)

val on_click : Browser.element -> (unit -> unit) -> unit
(** Register a click callback. *)

val set_attribute : Browser.element -> string -> string -> unit
(** Set an HTML attribute: element, name, value. *)

val ready : unit -> unit
(** Mark the document ready for interaction. *)

val failed : string -> unit
(** Append the failure message and a Reload button to the page. *)

val on_change : Browser.element -> (unit -> unit) -> unit
(** Register a form change callback. *)

val on_resize : (unit -> unit) -> unit
(** Register a window resize callback. *)

val frame : (unit -> unit) -> unit
(** Schedule one callback for the next animation frame. *)

val disabled : Browser.element -> bool -> unit
(** Set whether a form control is disabled. *)

val hidden : Browser.element -> bool -> unit
(** Set whether an element is hidden. *)

val pointer_x : unit -> float
(** Read the last pointer position in canvas pixels, along the x axis. *)

val pointer_y : unit -> float
(** Read the last pointer position in canvas pixels, along the y axis. *)

val pointer_screen_x : unit -> float
(** Read the last pointer x coordinate in viewport CSS pixels. *)

val pointer_screen_y : unit -> float
(** Read the last pointer y coordinate in viewport CSS pixels. *)

val pointer_handle : unit -> handle
(** Read the canvas surface or selection corner associated with the current pointer. *)

val pinch_x : unit -> float
(** Read the pinch center x offset from the interaction element center, in CSS pixels.
*)

val pinch_y : unit -> float
(** Read the pinch center y offset from the interaction element center, in CSS pixels.
*)

val pinch_distance : unit -> float
(** Read the distance between the two pinch contacts, in CSS pixels. *)

val wheel_delta : unit -> float
(** Read the last wheel displacement, normalized to CSS pixels. *)

val key : unit -> string
(** Read the current normalized keyboard shortcut. *)

val on_pointer :
  Browser.element ->
  (unit -> unit) ->
  (unit -> unit) ->
  (unit -> unit) ->
  (unit -> unit) ->
  (unit -> unit) ->
  (unit -> unit) ->
  (unit -> unit) ->
  unit
(** Register callbacks in order: down, move, up, cancel, pinch start, pinch move, pinch
    end. Coordinate getters are updated before callbacks run. *)

val view : float -> float -> float -> unit
(** Set canvas translation x, y in CSS pixels and its zoom factor. *)

val fit_scale : unit -> float
(** Compute a zoom factor that fits the canvas into the workspace. *)

val selection : float -> float -> float -> float -> unit
(** Display the selection rectangle: x, y, width, height in canvas pixels. *)

val on_wheel : Browser.element -> (unit -> unit) -> unit
(** Register a wheel callback and suppress native scrolling over this element. *)

val on_key : (unit -> bool) -> unit
(** Register a keyboard callback outside editable controls. Return true to prevent the
    browser default. *)

val shortcut : Browser.element -> string -> unit
(** Associate a normalized keyboard shortcut with a button click. *)

val save_png : Bindings.Uint8ClampedArray.t -> unit
(** Download the supplied PNG bytes. *)
