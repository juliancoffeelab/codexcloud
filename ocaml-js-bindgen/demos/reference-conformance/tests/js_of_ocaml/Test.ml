module B = Bindings
module P = Js_of_ocaml.Promise

let bind pending fn = P.bind fn pending
let return = P.resolve

(* Test-only host-value branding for the Rust identity checks below. It performs
   no validation; the tests must compare the exact original rejection value. *)
external host_error : Js_of_ocaml.Js.Unsafe.any -> B.HostError.t = "%identity"

let done_ () : unit =
  Js_of_ocaml.Js.Unsafe.meth_call
    (Js_of_ocaml.Js.Unsafe.get Js_of_ocaml.Js.Unsafe.global "OcamljsTest")
    "done" [||]

let rejection_checks value =
  let native = Checks.get (B.rejected value) in
  P.catch
    (fun error ->
      assert (
        Checks.get (B.error_matches (host_error (P.error_to_any error)) value));
      bind
        (B.await_object (P.reject error))
        (function
          | Error same ->
              assert (Checks.get (B.error_matches same value));
              return ()
          | Ok _ -> assert false))
    (bind native (fun _ -> assert false))

(* Test-only: inspect exceptions raised by the compiled throws/callback fixtures.
   Matching the expected constructor and payload is the assertion; unmatched
   rejections are propagated. Never use this bridge for general JS errors. *)
external exception_of_any : Js_of_ocaml.Js.Unsafe.any -> exn = "%identity"

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
  let pending = P.resolve value in
  let same = B.promise_identity pending in
  assert (same == pending);
  bind same (fun received ->
      assert (received == value);
      bind (B.await_object pending) (fun received ->
          assert (Checks.get received == value);
          bind (B.async_identity value) (fun received ->
              assert (received == value && !calls = 0);
              let bad = Checks.get (B.parse_json "{}") in
              let _ =
                J.Unsafe.fun_call
                  (J.Unsafe.get
                     (J.Unsafe.get J.Unsafe.global "Object")
                     "defineProperty")
                  [|
                    J.Unsafe.inject bad;
                    J.Unsafe.inject (J.string "then");
                    J.Unsafe.inject
                      (J.Unsafe.obj
                         [|
                           ( "get",
                             J.Unsafe.inject
                               (J.wrap_callback (fun () ->
                                    failwith "throwing then getter")) );
                         |]);
                  |]
              in
              P.catch
                (fun error ->
                  match exception_of_any (P.error_to_any error) with
                  | Failure reason when reason = "throwing then getter" ->
                      P.resolve ()
                  | _ -> P.reject error)
                (bind (B.async_identity bad) (fun _ -> assert false)))))

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

let () =
  let _ = bind (carrier_checks ()) run in
  ()
