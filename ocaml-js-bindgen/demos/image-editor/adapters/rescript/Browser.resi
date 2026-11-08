/** Browser operations used by this frontend. Host values stay in their native representation. */
type element

type event

type rect

type style

type events

type touches

type touch

type listener_options = {passive: bool}

@get external style: element => style = "style"

@get external touches: event => touches = "touches"

external global: element = "globalThis"

@get external document_of: element => element = "document"

@get external document_element: element => element = "documentElement"

@get external body: element => element = "body"

/** Find an element by id, returning None when absent. */
let find_id: (element, string) => option<element>

@send external create_element: (element, string) => element = "createElement"

@set external set_text: (element, string) => unit = "textContent"

@send external append_child: (element, element) => unit = "appendChild"

@get external get_value: element => string = "value"

@set external set_value: (element, string) => unit = "value"

@send external set_attribute: (element, string, string) => unit = "setAttribute"
@send
let get_attribute: (element, string) => option<string>

let closest: (element, string) => option<element>

@send external click: element => unit = "click"

@send external listen_raw: (element, string, event => unit) => unit = "addEventListener"
@send
external listen_options: (element, string, event => unit, listener_options) => unit =
  "addEventListener"

external frame_raw: (float => unit) => int = "requestAnimationFrame"

external cancel_frame: int => unit = "cancelAnimationFrame"

@set external set_disabled: (element, bool) => unit = "disabled"

@get external disabled: element => bool = "disabled"

@set external set_hidden: (element, bool) => unit = "hidden"

@get external width: element => float = "width"

@get external height: element => float = "height"

@get external client_width: element => float = "clientWidth"

@get external client_height: element => float = "clientHeight"

@send external bounds: element => rect = "getBoundingClientRect"

@get external left: rect => float = "left"

@get external top: rect => float = "top"

@get external rect_width: rect => float = "width"

@get external rect_height: rect => float = "height"

@send external set_style: (style, string, string) => unit = "setProperty"

@send external capture: (element, int) => unit = "setPointerCapture"

@get external target: event => element = "target"

@get external client_x: event => float = "clientX"

@get external client_y: event => float = "clientY"

@get external pointer_id: event => int = "pointerId"

@get external button: event => int = "button"

@get external pointer_type_raw: event => string = "pointerType"

@send external prevent_default: event => unit = "preventDefault"

@get external events_length: events => int = "length"

@get_index external event_at: (events, int) => event = ""

@get external touches_length: touches => int = "length"

@send external touch_at: (touches, int) => touch = "item"

@get external touch_x: touch => float = "clientX"

@get external touch_y: touch => float = "clientY"

@get external delta_y: event => float = "deltaY"

@get external delta_mode: event => int = "deltaMode"

@get external key_raw: event => string = "key"

@get external ctrl: event => bool = "ctrlKey"

@get external meta: event => bool = "metaKey"

@get external shift: event => bool = "shiftKey"

let listener_options: bool => listener_options

let coalesced: event => option<events>

/** Convert a host rejection value to display text. */
external error_text: 'a => string = "String"

/** Reload the current page. */
let reload: unit => unit

let save_png: Bindings.Uint8ClampedArray.t => unit

/** Render an OCaml or host JavaScript exception for display. */
let exception_text: exn => string
