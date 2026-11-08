let start () =
  App.start (fun success failure ->
      try Async.observe (Bindings.Paint.open_ "file") success failure
      with _ -> failure ())

let () =
  Async.observe_error
    (Bindings.Wasm.load "./backend.wasm")
    (function
      | Error error -> Ui.failed (Bindings.Wasm.Load_error.message error)
      | Ok () ->
      try
        start ();
        Ui.ready ()
      with error -> Ui.failed (Browser.exception_text error))
    Ui.failed
