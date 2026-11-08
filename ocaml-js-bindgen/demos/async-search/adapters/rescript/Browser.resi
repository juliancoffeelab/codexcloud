/** Browser operations used by this frontend. Host values stay in their native representation. */
type element

type event

type worker

type worker_options = {@as("type") kind: string}

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

@get external get_value: element => string = "value"

@send external set_attribute: (element, string, string) => unit = "setAttribute"

@send external listen_raw: (element, string, event => unit) => unit = "addEventListener"

@new external new_worker: (string, worker_options) => worker = "Worker"

@send external terminate: worker => unit = "terminate"

@send external post: (worker, message) => unit = "postMessage"

@set external worker_message: (worker, event => unit) => unit = "onmessage"

@set external worker_error: (worker, event => unit) => unit = "onerror"

@set external worker_message_error: (worker, event => unit) => unit = "onmessageerror"

external post_global: message => unit = "postMessage"

external close_worker: unit => unit = "close"

/** Create options for an ES module worker. */
let worker_options: unit => worker_options

/** Check a message against the supplied maximum payload length. */
let message_valid: (message, int) => bool

/** Read an event payload, returning None if absent. Use message_valid before accessing its fields. */
let data: event => option<message>

/** Convert a host rejection value to display text. */
external error_text: 'a => string = "String"

/** Reload the current page. */
let reload: unit => unit

/** Render an OCaml or host JavaScript exception for display. */
let exception_text: exn => string

let rejection_text: event => string

@send external prevent_default: event => unit = "preventDefault"
