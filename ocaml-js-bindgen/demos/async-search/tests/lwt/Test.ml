let observe promise success failure =
  Lwt.on_any promise success (fun _ -> failure ())

module B = Bindings

let rejects f =
  try
    f ();
    false
  with _ -> true

let () =
  let notified = ref 0. in
  assert (
    B.numeric_notify [| 1.; 2. |]
      (fun (progress : B.progress) -> notified := progress.records)
      [| 3.; 4. |]
    = 10.);
  assert (!notified = 2.);
  observe
    (B.numeric_sum_after_yield [| 1.; 2. |] [| 3.; 4. |])
    (fun value -> assert (value = 10.))
    (fun () -> assert false);
  observe (B.Arithmetic.twice 21.)
    (fun value -> assert (value = 42.))
    (fun () -> assert false);
  let corpus = B.Corpus.create ()
  and other = B.Corpus.create ()
  and input = B.Input.create () in
  let pending = B.Index.create corpus input (fun _ -> ()) in
  assert (
    rejects (fun () ->
        let _ = B.Corpus.revision corpus in
        ()));
  assert (rejects (fun () -> B.Corpus.dispose corpus));
  assert (B.Corpus.revision other = 0.);
  B.Input.finish input;
  observe pending
    (function
      | Ok index ->
          B.Index.dispose index;
          B.Input.dispose input;
          B.Corpus.dispose corpus;
          B.Corpus.dispose other
      | Error _ -> assert false)
    (fun () -> assert false);
  observe (B.digest "λ🦀")
    (fun result -> assert (result = Ok 2.))
    (fun () -> assert false);
  observe
    (B.borrowed_text "λ" (Bytes.of_string "AB"))
    (fun count -> assert (count = 132.))
    (fun () -> assert false);
  observe (B.reject ()) (fun _ -> assert false) (fun () -> ())

module L = Lwt

let ( let* ) = L.bind

module J = Js_of_ocaml.Js

let host = J.Unsafe.get J.Unsafe.global "OcamljsNative"

let before_delivery callback : unit =
  J.Unsafe.meth_call host "before_delivery"
    [| J.Unsafe.inject (J.wrap_callback callback) |]

let next_turn () =
  let promise, wake = L.task () in
  let () : unit =
    J.Unsafe.meth_call host "later"
      [| J.Unsafe.inject (J.wrap_callback (fun () -> L.wakeup wake ())) |]
  in
  promise

let () =
  L.async (fun () ->
      let* count = B.digest "λ🦀" in
      assert (count = Ok 2.);
      let* count = B.borrowed_text "λ" (Bytes.of_string "AB") in
      assert (count = 132.);
      let c = B.Corpus.create () and input = B.Input.create () in
      let progress _ = () in
      let pending = B.Index.create c input progress in
      let* () = next_turn () in
      assert (B.Input.waiting input);
      L.cancel pending;
      assert (L.state pending = L.Fail L.Canceled);
      assert (B.Corpus.revision c = 0.);
      B.Input.dispose input;
      B.Corpus.dispose c;
      let stale = B.Corpus.create () in
      B.Corpus.dispose stale;
      let failed = B.Corpus.shared stale in
      assert (match L.state failed with L.Fail _ -> true | _ -> false);
      let* () =
        L.catch
          (fun () ->
            let* _ = failed in
            L.fail (Failure "accepted stale owner"))
          (fun _ -> L.return_unit)
      in
      let* () =
        L.catch
          (fun () ->
            let* _ = B.reject () in
            L.fail (Failure "missing typed exception"))
          (function
            | B.Reject_error B.Index_error.Invalid_record -> L.return_unit
            | error -> L.fail error)
      in
      let* () =
        L.catch
          (fun () ->
            let* _ = B.Index.throw_owned () in
            L.fail (Failure "missing owned exception"))
          (function
            | B.Index.Throw_owned_error owner ->
                B.Index.dispose owner;
                L.return_unit
            | error -> L.fail error)
      in
      let cancelled_after_ready pending =
        before_delivery (fun () -> L.cancel pending);
        let* () =
          L.catch
            (fun () ->
              let* _ = pending in
              L.fail (Failure "race cancellation lost"))
            (function L.Canceled -> L.return_unit | error -> L.fail error)
        in
        let* () = next_turn () in
        assert (B.live_entries () = 0.);
        L.return_unit
      in
      let* () = cancelled_after_ready (B.Index.own_index ()) in
      let* () = cancelled_after_ready (B.Index.throw_owned ()) in
      let module J = Js_of_ocaml.Js in
      let () : unit =
        J.Unsafe.meth_call
          (J.Unsafe.get J.Unsafe.global "OcamljsNative")
          "done" [||]
      in
      L.return_unit)
