module C = Channel

let start () =
  let message text = Ui.text (Ui.by_id "message") text in
  let busy = ref false
  and closed = ref false
  and ready = ref false
  and checking = ref false
  and querying = ref false in
  let send command rollback =
    try C.send command
    with _ ->
      rollback ();
      message "Could not send search command"
  in
  message "Worker is loading";
  C.start (function
    | C.Ready ->
        ready := true;
        message "Worker ready"
    | C.Progress count ->
        Ui.text (Ui.by_id "progress") ("Indexed " ^ string_of_int count)
    | C.Indexed ->
        busy := false;
        message "Index ready"
    | C.Cancelled ->
        busy := false;
        message "Cancelled; future dropped and corpus reusable"
    | C.Search_error value ->
        busy := false;
        message value
    | C.Query_error value ->
        querying := false;
        message value
    | C.Worker_failed value ->
        busy := false;
        closed := true;
        message value
    | C.Clear -> Ui.clear (Ui.by_id "matches")
    | C.Match value ->
        let row = Ui.create "li" in
        Ui.text row value;
        Ui.append (Ui.by_id "matches") row
    | C.Queried -> querying := false
    | C.Validation passed ->
        checking := false;
        Ui.text (Ui.by_id "validation") (if passed then "PASS" else "FAILED")
    | C.Closed ->
        busy := false;
        closed := true;
        message "Worker closed gracefully");
  Ui.on_click (Ui.by_id "start") (fun () ->
      if !ready && (not !busy) && (not !checking) && not !closed then (
        busy := true;
        message "Indexing in worker…";
        send (C.Build (Ui.value (Ui.by_id "url"))) (fun () -> busy := false)));
  Ui.on_click (Ui.by_id "cancel") (fun () ->
      if !busy && not !closed then send C.Cancel (fun () -> ()));
  Ui.on_click (Ui.by_id "query") (fun () ->
      if !ready && (not !querying) && not !closed then (
        querying := true;
        send (C.Query (Ui.value (Ui.by_id "needle"))) (fun () -> querying := false)));
  Ui.on_click (Ui.by_id "validate") (fun () ->
      if !ready && (not !busy) && (not !checking) && not !closed then (
        checking := true;
        send C.Validate (fun () -> checking := false)));
  Ui.on_click (Ui.by_id "shutdown") (fun () ->
      if !ready && not !closed then send C.Shutdown (fun () -> ()))

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
