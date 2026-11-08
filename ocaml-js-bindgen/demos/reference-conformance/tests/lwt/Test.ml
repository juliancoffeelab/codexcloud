module B = Bindings
module P = Lwt

let bind = P.bind
let return = P.return

let done_ () : unit =
  Js_of_ocaml.Js.Unsafe.meth_call
    (Js_of_ocaml.Js.Unsafe.get Js_of_ocaml.Js.Unsafe.global "OcamljsTest")
    "done" [||]

let rejection_checks value =
  let native = Checks.get (B.rejected value) in
  bind
    (P.catch
       (fun () -> bind native (fun _ -> assert false))
       (function
         | B.Wasm.Promise_rejected error ->
             assert (Checks.get (B.error_matches error value));
             bind
               (B.await_object (P.fail (B.Wasm.Promise_rejected error)))
               (function
                 | Error same ->
                     assert (Checks.get (B.error_matches same value));
                     return ()
                 | Ok _ -> assert false)
         | _ -> assert false))
    (fun () ->
      let source, resolver = P.task () in
      let waiting = B.await_object source in
      P.cancel waiting;
      (match P.state waiting with P.Fail P.Canceled -> () | _ -> assert false);
      assert (P.is_sleeping source);
      P.wakeup_later resolver value;
      return ())

let carrier_checks () =
  let module J = Js_of_ocaml.Js in
  let value = Checks.get (B.parse_json "{}")
  and replacement = Checks.get (B.parse_json "{}") in
  let calls = ref 0 in
  J.Unsafe.set value "then"
    (J.wrap_callback (fun resolve _ ->
         incr calls;
         let _ = J.Unsafe.fun_call resolve [| J.Unsafe.inject replacement |] in
         ()));
  bind
    (B.await_object (P.return value))
    (fun received ->
      assert (Checks.get received == value && !calls = 0);
      bind (B.async_identity value) (fun received ->
          assert (received == value && !calls = 0);
          return ()))

let run () =
  let value = Checks.run () in
  let native = Checks.get (B.native_promise value) in
  bind (B.promise_identity native) (fun received ->
      assert (received == value);
      bind (rejection_checks value) (fun () ->
          bind (B.alias_promise native) (fun received ->
              assert (received == value);
              bind (B.await_object native) (fun result ->
                  assert (Checks.get result == value);
                  bind (B.await_borrowed native) (fun result ->
                      assert (Checks.get result == value);
                      let calls = ref 0 in
                      bind
                        (B.notify_after native (fun (event : B.host_event) ->
                             assert (event.value == value);
                             assert (event.label = "\239\187\191Host λ🦀");
                             assert (event.sequence = 2.);
                             incr calls))
                        (fun result ->
                          assert (!calls = 1);
                          assert (Checks.get result == value);
                          done_ ();
                          return ()))))))

let () = P.async (fun () -> bind (carrier_checks ()) run)
