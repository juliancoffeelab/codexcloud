type keyboard_event
type page_transition_event

external showModal : Dom.element -> unit = "showModal" [@@mel.send]
external closeModal : Dom.element -> unit = "close" [@@mel.send]
external isOpen : Dom.element -> bool = "open" [@@mel.get]
external focus : Dom.element -> unit = "focus" [@@mel.send]
external focusWith : Dom.element -> < preventScroll : bool > Js.t -> unit = "focus" [@@mel.send]
external byId : string -> Dom.element Js.Nullable.t = "getElementById" [@@mel.scope "document"]
external query : string -> Dom.element Js.Nullable.t = "querySelector" [@@mel.scope "document"]

external queryWithin : Dom.element -> string -> Dom.element Js.Nullable.t = "querySelector"
[@@mel.send]

external target : Dom.event -> Dom.element = "target" [@@mel.get]
external nodeType : Dom.element -> int = "nodeType" [@@mel.get]
external closest : Dom.element -> string -> Dom.element Js.Nullable.t = "closest" [@@mel.send]
external inputChecked : Dom.element -> bool = "checked" [@@mel.get]
external eventKey : keyboard_event -> string = "key" [@@mel.get]
external keyTarget : keyboard_event -> Dom.element = "target" [@@mel.get]

external on_keydown : Dom.document -> (_[@mel.as "keydown"]) -> (keyboard_event -> unit) -> unit
  = "addEventListener"
[@@mel.send]

external off_keydown : Dom.document -> (_[@mel.as "keydown"]) -> (keyboard_event -> unit) -> unit
  = "removeEventListener"
[@@mel.send]

external addListener : Dom.element -> string -> (Dom.event -> unit) -> unit = "addEventListener"
[@@mel.send]

external removeListener : Dom.element -> string -> (Dom.event -> unit) -> unit
  = "removeEventListener"
[@@mel.send]

external document : Dom.document = "document"
external scroll : Dom.element -> < block : string > Js.t -> unit = "scrollIntoView" [@@mel.send]

type bounds = { left : float; right : float; top : float; bottom : float }

external bounds : Dom.element -> bounds = "getBoundingClientRect" [@@mel.send]
external reload : unit -> unit = "reload" [@@mel.scope "window.location"]
external copy_text : string -> unit Js.Promise.t = "writeText" [@@mel.scope "navigator.clipboard"]

external listen_capture : Dom.element -> string -> (Dom.event -> unit) -> bool -> unit
  = "addEventListener"
[@@mel.send]

external remove_capture : Dom.element -> string -> (Dom.event -> unit) -> bool -> unit
  = "removeEventListener"
[@@mel.send]

external trusted : Dom.event -> bool = "isTrusted" [@@mel.get]
external event : string -> Dom.event = "Event" [@@mel.new]
external notify : Dom.event -> unit = "dispatchEvent" [@@mel.scope "window"]
external tag_name : Dom.element -> string = "tagName" [@@mel.get]
external tagName : Dom.element -> string Js.Undefined.t = "tagName" [@@mel.get]
external page_persisted : page_transition_event -> bool = "persisted" [@@mel.get]

external listen_pagehide : (_[@mel.as "pagehide"]) -> (page_transition_event -> unit) -> unit
  = "addEventListener"
[@@mel.scope "window"]

external listen_window : string -> (unit -> unit) -> unit = "addEventListener"
[@@mel.scope "window"]

external listen_error : string -> (Dom.event -> unit) -> unit = "addEventListener"
[@@mel.scope "window"]

external remove_error : string -> (Dom.event -> unit) -> unit = "removeEventListener"
[@@mel.scope "window"]

external event_error : Dom.event -> Foreign_error.t Js.Undefined.t = "error" [@@mel.get]
external event_reason : Dom.event -> Foreign_error.t = "reason" [@@mel.get]
external set_text : Dom.element -> string -> unit = "textContent" [@@mel.set]
external hidden : Dom.element -> bool -> unit = "hidden" [@@mel.set]
external open_ : Dom.element -> bool -> unit = "open" [@@mel.set]

module Element_motion = struct
  type timer = Browser_time.timer
  type media
  type classes

  external classes : Dom.element -> classes = "classList" [@@mel.get]
  external add_class : classes -> string -> unit = "add" [@@mel.send]
  external remove_class : classes -> string -> unit = "remove" [@@mel.send]
  external bounds : Dom.element -> unit = "getBoundingClientRect" [@@mel.send]
  external delay : (unit -> unit) -> int -> timer = "setTimeout"
  external clear : timer -> unit = "clearTimeout"
  external document : unit Js.Undefined.t = "document" [@@mel.scope "globalThis"]
  external media_function : unit Js.Undefined.t = "matchMedia" [@@mel.scope "globalThis"]
  external match_media : string -> media = "matchMedia"
  external matches : media -> bool = "matches" [@@mel.get]
  external query : string -> Dom.element Js.Nullable.t = "querySelector" [@@mel.scope "document"]
  external dispatch : Dom.element -> Dom.event -> bool = "dispatchEvent" [@@mel.send]
  external media_on : media -> string -> (unit -> unit) -> unit = "addEventListener" [@@mel.send]

  external media_off : media -> string -> (unit -> unit) -> unit = "removeEventListener"
  [@@mel.send]
end

module Event (Detail : sig
  type t
end) =
struct
  type event

  external create : string -> < detail : Detail.t > Js.t -> event = "CustomEvent" [@@mel.new]
  external detail : event -> Detail.t = "detail" [@@mel.get]
  external on : Dom.element -> string -> (event -> unit) -> unit = "addEventListener" [@@mel.send]

  external off : Dom.element -> string -> (event -> unit) -> unit = "removeEventListener"
  [@@mel.send]

  external dispatch : Dom.element -> event -> bool = "dispatchEvent" [@@mel.send]
end

external descendants : Dom.element -> string -> Dom.element array = "querySelectorAll" [@@mel.send]
external activeElement : Dom.element Js.Nullable.t = "activeElement" [@@mel.scope "document"]
external scrollTop : Dom.element -> float = "scrollTop" [@@mel.get]
external setScrollTop : Dom.element -> float -> unit = "scrollTop" [@@mel.set]

external addDocumentListener : Dom.document -> string -> (Dom.event -> unit) -> unit
  = "addEventListener"
[@@mel.send]

external removeDocumentListener : Dom.document -> string -> (Dom.event -> unit) -> unit
  = "removeEventListener"
[@@mel.send]

external scrollTo : Dom.element -> < top : float > Js.t -> unit = "scrollTo" [@@mel.send]
