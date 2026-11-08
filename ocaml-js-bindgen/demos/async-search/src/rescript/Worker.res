module Run = () => {
  module B = Bindings
  module C = Channel
  module V = Browser_checks
  let corpus = B.Corpus.create()
  let cancellation = ref(None)
  let index = ref(None)
  let closing = ref(false)
  let checking = ref(false)

  let clear_index = () =>
    switch index.contents {
    | None => ()
    | Some(value) =>
      index := None
      B.Index.dispose(value)
    }

  let shutdown = () => {
    clear_index()
    B.Corpus.dispose(corpus)
    assert(B.live_entries() == 0.)
    C.emit(C.Closed)
    C.close()
  }

  let cancel = () =>
    switch cancellation.contents {
    | None => ()
    | Some(token) => B.SearchCancel.cancel(token)
    }

  let build = url => {
    let token = B.SearchCancel.create()
    cancellation := Some(token)
    let pending = B.Index.build_url(corpus, url, token, (value: B.progress) =>
      C.emit(C.Progress(int_of_float(value.records)))
    )

    let finished = () => {
      cancellation := None
      B.SearchCancel.dispose(token)
    }

    Async.observe(
      pending,
      result => {
        finished()
        switch result {
        | Error(B.Index_error.Cancelled) => C.emit(C.Cancelled)
        | Error(_) => C.emit(C.Search_error("Indexing failed; previous corpus unchanged"))
        | Ok(value) =>
          clear_index()
          index := Some(value)
          C.emit(C.Indexed)
        }
        if closing.contents {
          shutdown()
        }
      },
      () => {
        finished()
        C.emit(C.Search_error("Indexing failed"))
        if closing.contents {
          shutdown()
        }
      },
    )
  }

  let () = {
    C.listen(command =>
      if !closing.contents {
        try switch command {
        | C.Build(url) if cancellation.contents == None && !checking.contents => build(url)
        | C.Cancel => cancel()
        | C.Query(needle) =>
          C.emit(C.Clear)
          switch index.contents {
          | None => ()
          | Some(value_) => Array.iter(text => C.emit(C.Match(text)), B.Index.query(value_, needle))
          }
          C.emit(C.Queried)
        | C.Validate if cancellation.contents == None && !checking.contents =>
          checking := true
          V.run(passed => {
            checking := false
            C.emit(C.Validation(passed))
            if closing.contents {
              shutdown()
            }
          })
        | C.Shutdown =>
          closing := true
          switch cancellation.contents {
          | None =>
            if !checking.contents {
              shutdown()
            }
          | Some(_) => cancel()
          }
        | _ => ()
        } catch {
        | _ =>
          switch command {
          | C.Query(_) => C.emit(C.Query_error("Search query failed"))
          | C.Validate =>
            checking := false
            C.emit(C.Validation(false))
          | _ => C.emit(C.Search_error("Search command failed"))
          }
        }
      }
    )
    C.emit(C.Ready)
  }
}
let start = () => {
  module Running = Run()
  ()
}
let done_ = start
let terminal = ref(false)
let failed = reason =>
  if !terminal.contents {
    terminal := true
    try {
      Channel.emit(Channel.Worker_failed(reason))
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
