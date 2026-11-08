module C = Channel

let start = () => {
  let message = text => Ui.text(Ui.by_id("message"), text)
  let busy = ref(false)
  and closed = ref(false)
  and ready = ref(false)
  and checking = ref(false)
  and querying = ref(false)
  let send = (command, rollback) =>
    try C.send(command) catch {
    | _ =>
      rollback()
      message("Could not send search command")
    }

  message("Worker is loading")
  C.start(x =>
    switch x {
    | C.Ready =>
      ready := true
      message("Worker ready")
    | C.Progress(count) => Ui.text(Ui.by_id("progress"), "Indexed " ++ string_of_int(count))
    | C.Indexed =>
      busy := false
      message("Index ready")
    | C.Cancelled =>
      busy := false
      message("Cancelled; future dropped and corpus reusable")
    | C.Search_error(value) =>
      busy := false
      message(value)
    | C.Query_error(value) =>
      querying := false
      message(value)
    | C.Worker_failed(value) =>
      busy := false
      closed := true
      message(value)
    | C.Clear => Ui.clear(Ui.by_id("matches"))
    | C.Match(value) =>
      let row = Ui.create("li")
      Ui.text(row, value)
      Ui.append(Ui.by_id("matches"), row)
    | C.Queried => querying := false
    | C.Validation(passed) =>
      checking := false
      Ui.text(
        Ui.by_id("validation"),
        if passed {
          "PASS"
        } else {
          "FAILED"
        },
      )
    | C.Closed =>
      busy := false
      closed := true
      message("Worker closed gracefully")
    }
  )
  Ui.on_click(Ui.by_id("start"), () =>
    if ready.contents && (!busy.contents && (!checking.contents && !closed.contents)) {
      busy := true
      message("Indexing in worker…")
      send(C.Build(Ui.value(Ui.by_id("url"))), () => busy := false)
    }
  )
  Ui.on_click(Ui.by_id("cancel"), () =>
    if busy.contents && !closed.contents {
      send(C.Cancel, () => ())
    }
  )
  Ui.on_click(Ui.by_id("query"), () =>
    if ready.contents && (!querying.contents && !closed.contents) {
      querying := true
      send(C.Query(Ui.value(Ui.by_id("needle"))), () => querying := false)
    }
  )
  Ui.on_click(Ui.by_id("validate"), () =>
    if ready.contents && (!busy.contents && (!checking.contents && !closed.contents)) {
      checking := true
      send(C.Validate, () => checking := false)
    }
  )
  Ui.on_click(Ui.by_id("shutdown"), () =>
    if ready.contents && !closed.contents {
      send(C.Shutdown, () => ())
    }
  )
}

let () = Async.observe_error(
  Bindings.Wasm.load("./backend.wasm"),
  loadResult =>
    switch loadResult {
    | Error(error) => Ui.failed(Bindings.Wasm.Load_error.message(error))
    | Ok() =>
      try {
        start()
        Ui.ready()
      } catch {
      | error => Ui.failed(Browser.exception_text(error))
      }
    },
  Ui.failed,
)
