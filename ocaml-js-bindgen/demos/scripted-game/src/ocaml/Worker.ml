module C = Checks

let done_ () =
  C.run ();
  Browser.post_global
    { Browser.kind = Browser.to_host "ready"; value = Browser.to_host "" };
  Browser.close_worker ()

let terminal = ref false

let failed reason =
  if not !terminal then (
    terminal := true;
    (try
       Browser.post_global
         { Browser.kind = Browser.to_host "failed"; value = Browser.to_host reason }
     with _ -> ());
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
