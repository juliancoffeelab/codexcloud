type command = Build(string) | Cancel | Query(string) | Validate | Shutdown

type event =
  | Ready
  | Progress(int)
  | Indexed
  | Cancelled
  | Search_error(string)
  | Query_error(string)
  | Worker_failed(string)
  | Clear
  | Match(string)
  | Queried
  | Validation(bool)
  | Closed

let command_to_wire = x =>
  switch x {
  | Build(url) => ("build", url)
  | Cancel => ("cancel", "")
  | Query(needle) => ("query", needle)
  | Validate => ("validate", "")
  | Shutdown => ("shutdown", "")
  }

let command_of_wire = x =>
  switch x {
  | ("build", url) => Some(Build(url))
  | ("cancel", "") => Some(Cancel)
  | ("query", needle) => Some(Query(needle))
  | ("validate", "") => Some(Validate)
  | ("shutdown", "") => Some(Shutdown)
  | _ => None
  }

let event_to_wire = x =>
  switch x {
  | Ready => ("ready", "")
  | Progress(count) => ("progress", string_of_int(count))
  | Indexed => ("index", "")
  | Cancelled => ("cancelled", "")
  | Search_error(reason) => ("error", reason)
  | Query_error(reason) => ("query_error", reason)
  | Worker_failed(reason) => ("failed", reason)
  | Clear => ("clear", "")
  | Match(text) => ("match", text)
  | Queried => ("queried", "")
  | Validation(passed) => (
      "validation",
      if passed {
        "PASS"
      } else {
        "FAILED"
      },
    )
  | Closed => ("closed", "")
  }

let event_of_wire = x =>
  switch x {
  | ("ready", "") => Some(Ready)
  | ("progress", count) =>
    switch int_of_string_opt(count) {
    | Some(count) if count >= 0 && count <= 4096 => Some(Progress(count))
    | _ => None
    }
  | ("index", "") => Some(Indexed)
  | ("cancelled", "") => Some(Cancelled)
  | ("error", reason) => Some(Search_error(reason))
  | ("query_error", reason) => Some(Query_error(reason))
  | ("failed", reason) => Some(Worker_failed(reason))
  | ("clear", "") => Some(Clear)
  | ("match", text) => Some(Match(text))
  | ("queried", "") => Some(Queried)
  | ("validation", "PASS") => Some(Validation(true))
  | ("validation", "FAILED") => Some(Validation(false))
  | ("closed", "") => Some(Closed)
  | _ => None
  }

let worker: ref<option<Browser.worker>> = ref(None)

let message = ((kind, value)): Browser.message => {kind, value}

let payload = (data: Browser.message) => (data.kind, data.value)

let start = receive => {
  if worker.contents != None {
    failwith("Search worker already started")
  }
  let session = Browser.new_worker("./worker.mjs", Browser.worker_options())
  worker := Some(session)
  let current = () =>
    switch worker.contents {
    | Some(current) => current === session
    | None => false
    }
  Browser.worker_message(session, event =>
    switch Browser.data(event) {
    | Some(data) if current() && Browser.message_valid(data, 1024 * 1024) =>
      switch event_of_wire(payload(data)) {
      | Some(event) =>
        switch event {
        | Closed | Worker_failed(_) =>
          worker := None
          Browser.terminate(session)
        | _ => ()
        }
        receive(event)
      | None => ()
      }
    | _ => ()
    }
  )
  let failed = _ =>
    if current() {
      worker := None
      Browser.terminate(session)
      receive(Worker_failed("Search worker failed; reload to restart"))
    }

  Browser.worker_error(session, failed)
  Browser.worker_message_error(session, failed)
}

let send = command =>
  switch worker.contents {
  | None => failwith("Search worker is closed")
  | Some(worker) =>
    let data = message(command_to_wire(command))
    if !Browser.message_valid(data, 32768) {
      failwith("Invalid search command")
    }
    Browser.post(worker, data)
  }

let listen = receive =>
  Browser.listen_raw(Browser.global, "message", event =>
    switch Browser.data(event) {
    | Some(data) if Browser.message_valid(data, 32768) =>
      switch command_of_wire(payload(data)) {
      | Some(command) => receive(command)
      | None => ()
      }
    | _ => ()
    }
  )

let emit = event => Browser.post_global(message(event_to_wire(event)))
let close = () => Browser.close_worker()
