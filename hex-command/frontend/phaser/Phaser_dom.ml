type rect = { left : float; top : float; width : float; height : float }

type touch = {
  identifier : int;
  client_x : float; [@mel.as "clientX"]
  client_y : float; [@mel.as "clientY"]
}

type touch_list
type computed_style

external parent : Dom.element -> Dom.element Js.Nullable.t = "parentElement" [@@mel.get]
external closest : Dom.element -> string -> Dom.element Js.Nullable.t = "closest" [@@mel.send]
external width : Dom.element -> int = "clientWidth" [@@mel.get]
external height : Dom.element -> int = "clientHeight" [@@mel.get]
external client_left : Dom.element -> int = "clientLeft" [@@mel.get]
external client_top : Dom.element -> int = "clientTop" [@@mel.get]
external bounds : Dom.element -> rect = "getBoundingClientRect" [@@mel.send]
external canvas_bounds : Phaser.canvas -> rect = "getBoundingClientRect" [@@mel.send]
external computed_style : Dom.element -> computed_style = "getComputedStyle"
external overflow_x : computed_style -> string = "overflowX" [@@mel.get]
external overflow_y : computed_style -> string = "overflowY" [@@mel.get]
external device_ratio : float = "devicePixelRatio" [@@mel.scope "globalThis"]
external inner_width : int = "innerWidth" [@@mel.scope "window"]
external query : string -> Dom.element Js.Nullable.t = "querySelector" [@@mel.scope "document"]
external active_element : Dom.element Js.Nullable.t = "activeElement" [@@mel.scope "document"]
external focus : Phaser.canvas -> < preventScroll : bool > Js.t -> unit = "focus" [@@mel.send]
external tab_index : Phaser.canvas -> int -> unit = "tabIndex" [@@mel.set]
external cursor : Phaser.style -> string -> unit = "cursor" [@@mel.set]
external css_width : Phaser.style -> string -> unit = "width" [@@mel.set]
external css_height : Phaser.style -> string -> unit = "height" [@@mel.set]
external client_x : Dom.event -> float = "clientX" [@@mel.get]
external client_y : Dom.event -> float = "clientY" [@@mel.get]
external changed_touches : Dom.event -> touch_list Js.Undefined.t = "changedTouches" [@@mel.get]
external touch_count : touch_list -> int = "length" [@@mel.get]
external touch_at : touch_list -> int -> touch = "" [@@mel.get_index]
external event_time : Dom.event -> float = "timeStamp" [@@mel.get]
external delta_mode : Dom.event -> int = "deltaMode" [@@mel.get]
external ctrl : Dom.event -> bool = "ctrlKey" [@@mel.get]
external meta : Dom.event -> bool = "metaKey" [@@mel.get]
external shift : Dom.event -> bool = "shiftKey" [@@mel.get]
external alt : Dom.event -> bool = "altKey" [@@mel.get]
external key : Dom.event -> string = "key" [@@mel.get]
external pointer_type : Dom.event -> string = "pointerType" [@@mel.get]
external pointer_id : Dom.event -> int = "pointerId" [@@mel.get]
external button : Dom.event -> int = "button" [@@mel.get]
external buttons : Dom.event -> int = "buttons" [@@mel.get]
external default_prevented : Dom.event -> bool = "defaultPrevented" [@@mel.get]

external on_canvas_capture :
  Phaser.canvas -> string -> (Dom.event -> unit) -> < capture : bool ; passive : bool > Js.t -> unit
  = "addEventListener"
[@@mel.send]

external off_canvas_capture : Phaser.canvas -> string -> (Dom.event -> unit) -> bool -> unit
  = "removeEventListener"
[@@mel.send]

external event_type : Dom.event -> string = "type" [@@mel.get]
external prevent : Dom.event -> unit = "preventDefault" [@@mel.send]
external capture : Phaser.canvas -> int -> unit = "setPointerCapture" [@@mel.send]
external has_capture : Phaser.canvas -> int -> bool = "hasPointerCapture" [@@mel.send]
external release : Phaser.canvas -> int -> unit = "releasePointerCapture" [@@mel.send]

external on_canvas : Phaser.canvas -> string -> (Dom.event -> unit) -> unit = "addEventListener"
[@@mel.send]

external off_canvas : Phaser.canvas -> string -> (Dom.event -> unit) -> unit = "removeEventListener"
[@@mel.send]

external on_bound :
  Phaser.canvas -> string -> (unit -> unit) -> < capture : bool ; passive : bool > Js.t -> unit
  = "addEventListener"
[@@mel.send]

external off_bound : Phaser.canvas -> string -> (unit -> unit) -> bool -> unit
  = "removeEventListener"
[@@mel.send]

external on_window : string -> (unit -> unit) -> unit = "addEventListener" [@@mel.scope "window"]

external off_window : string -> (unit -> unit) -> unit = "removeEventListener"
[@@mel.scope "window"]

external on_window_event : string -> (Dom.event -> unit) -> unit = "addEventListener"
[@@mel.scope "window"]

external off_window_event : string -> (Dom.event -> unit) -> unit = "removeEventListener"
[@@mel.scope "window"]

external on_scroll : string -> (unit -> unit) -> < capture : bool ; passive : bool > Js.t -> unit
  = "addEventListener"
[@@mel.scope "document"]

external off_scroll : string -> (unit -> unit) -> bool -> unit = "removeEventListener"
[@@mel.scope "document"]

external on_pointer : Phaser.scene_input -> string -> ((Phaser.pointer -> unit)[@u]) -> unit = "on"
[@@mel.send]

external off_pointer : Phaser.scene_input -> string -> ((Phaser.pointer -> unit)[@u]) -> unit
  = "off"
[@@mel.send]

type game_objects

external on_wheel :
  Phaser.scene_input ->
  string ->
  ((Phaser.pointer -> game_objects -> float -> float -> float -> unit)[@u]) ->
  unit = "on"
[@@mel.send]

external off_wheel :
  Phaser.scene_input ->
  string ->
  ((Phaser.pointer -> game_objects -> float -> float -> float -> unit)[@u]) ->
  unit = "off"
[@@mel.send]

external on_input : Phaser.scene_input -> string -> ((unit -> unit)[@u]) -> unit = "on" [@@mel.send]

external off_input : Phaser.scene_input -> string -> ((unit -> unit)[@u]) -> unit = "off"
[@@mel.send]

external add_pointer : Phaser.scene_input -> int -> unit = "addPointer" [@@mel.send]

type keyboard

external keyboard : Phaser.scene_input -> keyboard Js.Nullable.t = "keyboard" [@@mel.get]
external on_key : keyboard -> string -> ((Dom.event -> unit)[@u]) -> unit = "on" [@@mel.send]
external off_key : keyboard -> string -> ((Dom.event -> unit)[@u]) -> unit = "off" [@@mel.send]

external on_step : Phaser.events -> string -> ((float -> float -> unit)[@u]) -> unit = "on"
[@@mel.send]

external off_step : Phaser.events -> string -> ((float -> float -> unit)[@u]) -> unit = "off"
[@@mel.send]

type resize_observer
type mutation_observer

external resize_observer : ((unit -> unit)[@u]) -> resize_observer = "ResizeObserver" [@@mel.new]
external observe_resize : resize_observer -> Dom.element -> unit = "observe" [@@mel.send]
external disconnect_resize : resize_observer -> unit = "disconnect" [@@mel.send]

external mutation_observer : ((unit -> unit)[@u]) -> mutation_observer = "MutationObserver"
[@@mel.new]

external observe_dialogs :
  mutation_observer ->
  Dom.element ->
  < attributes : bool ; attributeFilter : string array ; childList : bool ; subtree : bool > Js.t ->
  unit = "observe"
[@@mel.send]

external disconnect_mutation : mutation_observer -> unit = "disconnect" [@@mel.send]
external body : Dom.element = "body" [@@mel.scope "document"]
external dispatch : Dom.element -> Dom.event -> unit = "dispatchEvent" [@@mel.send]

type tween

external stop_tween : tween -> unit = "stop" [@@mel.send]
external tween_value : tween -> float Js.Nullable.t = "getValue" [@@mel.send]

type counter_config = {
  from : float;
  target : float; [@mel.as "to"]
  duration : float;
  ease : string;
  on_update : (tween -> unit[@u]); [@mel.as "onUpdate"]
  on_complete : (unit -> unit[@u]); [@mel.as "onComplete"]
}

external counter : Phaser.tweens -> counter_config -> tween = "addCounter" [@@mel.send]

module Tween_target (Target : sig
  type t
end) =
struct
  external kill : Phaser.tweens -> Target.t -> unit = "killTweensOf" [@@mel.send]
end

external style_visibility : Phaser.style -> string = "visibility" [@@mel.get]
external set_visibility : Phaser.style -> string -> unit = "visibility" [@@mel.set]
external image_width : Phaser.image_element -> int = "width" [@@mel.get]
external image_height : Phaser.image_element -> int = "height" [@@mel.get]
external natural_width : Phaser.image_element -> int = "naturalWidth" [@@mel.get]
external natural_height : Phaser.image_element -> int = "naturalHeight" [@@mel.get]
