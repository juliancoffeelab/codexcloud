let start () =
  let module Run = struct
    module B = Bindings
    module C = Channel
    module V = Browser_checks

    let corpus = B.Corpus.create ()
    let cancellation = ref None
    let index = ref None
    let closing = ref false
    let checking = ref false

    let clear_index () =
      match !index with
      | None -> ()
      | Some value ->
          index := None;
          B.Index.dispose value

    let shutdown () =
      clear_index ();
      B.Corpus.dispose corpus;
      assert (B.live_entries () = 0.);
      C.emit C.Closed;
      C.close ()

    let cancel () =
      match !cancellation with None -> () | Some token -> B.SearchCancel.cancel token

    let build url =
      let token = B.SearchCancel.create () in
      cancellation := Some token;
      let pending =
        B.Index.build_url corpus url token (fun (value : B.progress) ->
            C.emit (C.Progress (int_of_float value.records)))
      in
      let finished () =
        cancellation := None;
        B.SearchCancel.dispose token
      in
      Async.observe pending
        (fun result ->
          finished ();
          (match result with
          | Error B.Index_error.Cancelled -> C.emit C.Cancelled
          | Error _ ->
              C.emit (C.Search_error "Indexing failed; previous corpus unchanged")
          | Ok value ->
              clear_index ();
              index := Some value;
              C.emit C.Indexed);
          if !closing then shutdown ())
        (fun () ->
          finished ();
          C.emit (C.Search_error "Indexing failed");
          if !closing then shutdown ())

    let () =
      C.listen (fun command ->
          if not !closing then
            try
              match command with
              | C.Build url when !cancellation = None && not !checking -> build url
              | C.Cancel -> cancel ()
              | C.Query needle ->
                  C.emit C.Clear;
                  (match !index with
                  | None -> ()
                  | Some value_ ->
                      Array.iter
                        (fun text -> C.emit (C.Match text))
                        (B.Index.query value_ needle));
                  C.emit C.Queried
              | C.Validate when !cancellation = None && not !checking ->
                  checking := true;
                  V.run (fun passed ->
                      checking := false;
                      C.emit (C.Validation passed);
                      if !closing then shutdown ())
              | C.Shutdown -> (
                  closing := true;
                  match !cancellation with
                  | None -> if not !checking then shutdown ()
                  | Some _ -> cancel ())
              | _ -> ()
            with _ -> (
              match command with
              | C.Query _ -> C.emit (C.Query_error "Search query failed")
              | C.Validate ->
                  checking := false;
                  C.emit (C.Validation false)
              | _ -> C.emit (C.Search_error "Search command failed")));
      C.emit C.Ready

    let initialized = ()
  end in
  Run.initialized

let done_ = start
let terminal = ref false

let failed reason =
  if not !terminal then (
    terminal := true;
    (try Channel.emit (Channel.Worker_failed reason) with _ -> ());
    Browser.close_worker ())

let () =
  Browser.listen_raw Browser.global (Browser.to_host "unhandledrejection") (fun event ->
      Browser.prevent_default event;
      failed (Browser.rejection_text event));
  try
    Async.observe_error
      (Bindings.Wasm.load "./backend.wasm")
      (function
        | Error error -> failed (Bindings.Wasm.Load_error.message error)
        | Ok () -> try done_ () with error -> failed (Browser.exception_text error))
      failed
  with error -> failed (Browser.exception_text error)
