/** Browser operations used by this frontend. Host values stay in their native representation. */
type element

type event

type message = {kind: string, value: string}

external global: element = "globalThis"

@get external document_of: element => element = "document"

@get external document_element: element => element = "documentElement"

@get external body: element => element = "body"

/** Find an element by id, returning None when absent. */
let find_id: (element, string) => option<element>

@send external create_element: (element, string) => element = "createElement"

@set external set_text: (element, string) => unit = "textContent"

@send external replace_children: element => unit = "replaceChildren"

@send external append_child: (element, element) => unit = "appendChild"

@send external set_attribute: (element, string, string) => unit = "setAttribute"

@send external listen_raw: (element, string, event => unit) => unit = "addEventListener"

external post_global: message => unit = "postMessage"

external close_worker: unit => unit = "close"

/** Convert a host rejection value to display text. */
external error_text: 'a => string = "String"

/** Reload the current page. */
let reload: unit => unit

/** Render an OCaml or host JavaScript exception for display. */
let exception_text: exn => string

let rejection_text: event => string

@send external prevent_default: event => unit = "preventDefault"
