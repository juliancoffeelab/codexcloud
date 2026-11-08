module C = Checks

let done_ = () => {
  C.run()
  Browser.post_global({kind: "ready", value: ""})
  Browser.close_worker()
}
let terminal = ref(false)
let failed = reason =>
  if !terminal.contents {
    terminal := true
    try {
      Browser.post_global({kind: "failed", value: reason})
    } catch {
    | _ => ()
    }
    Browser.close_worker()
  }
let () = {
  Browser.listen_raw(Browser.global, "unhandledrejection", event => {
    Browser.prevent_default(event)
    failed(Browser.rejection_text(event))
  })
  try {
    Async.observe_error(
      Bindings.Wasm.load("./backend.wasm"),
      loadResult =>
        switch loadResult {
        | Error(error) => failed(Bindings.Wasm.Load_error.message(error))
        | Ok() =>
          try {done_()} catch {
          | error => failed(Browser.exception_text(error))
          }
        },
      failed,
    )
  } catch {
  | error => failed(Browser.exception_text(error))
  }
}
