module P = Js_of_ocaml.Promise

let observe promise success failure =
  let _ =
    P.then_
      ~on_error:(fun _ ->
        failure ();
        P.resolve ())
      (fun value ->
        success value;
        P.resolve ())
      promise
  in
  ()

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

let ( let* ) pending fn = P.bind fn pending

(* Test-only bridge: these calls reject with exceptions raised by compiled OCaml.
   The constructor/payload matches below are assertions; other rejections fail.
   This is not a converter for arbitrary JavaScript rejection values. *)
external error_exn : P.error -> exn = "%identity"

let () =
  let _ =
    let* count = B.digest "λ🦀" in
    assert (count = Ok 2.);
    let corpus = B.Corpus.create () and input = B.Input.create () in
    let pending = B.Index.create corpus input (fun _ -> ()) in
    assert (rejects (fun () -> B.Corpus.dispose corpus));
    B.Input.finish input;
    let* result = pending in
    (match result with
    | Ok index -> B.Index.dispose index
    | Error _ -> assert false);
    B.Input.dispose input;
    B.Corpus.dispose corpus;
    let stale = B.Corpus.create () in
    B.Corpus.dispose stale;
    let* rejected =
      P.then_
        ~on_error:(fun _ -> P.resolve true)
        (fun _ -> P.resolve false)
        (B.Corpus.shared stale)
    in
    assert rejected;
    let* () =
      P.catch
        (fun error ->
          match error_exn error with
          | B.Reject_error B.Index_error.Invalid_record -> P.resolve ()
          | _ -> P.reject error)
        (P.bind (fun _ -> assert false) (B.reject ()))
    in
    let* owner = B.Index.own_index () in
    B.Index.dispose owner;
    let* () =
      P.catch
        (fun error ->
          match error_exn error with
          | B.Index.Throw_owned_error owner ->
              B.Index.dispose owner;
              P.resolve ()
          | _ -> P.reject error)
        (P.bind (fun _ -> assert false) (B.Index.throw_owned ()))
    in
    let corpus = B.Corpus.create () and token = B.SearchCancel.create () in
    B.SearchCancel.cancel token;
    let* result =
      B.Index.build_url corpus "https://unused.invalid" token (fun _ -> ())
    in
    assert (result = Error B.Index_error.Cancelled);
    B.SearchCancel.dispose token;
    B.Corpus.dispose corpus;
    assert (B.live_entries () = 0.);
    let module J = Js_of_ocaml.Js in
    let () : unit =
      J.Unsafe.meth_call
        (J.Unsafe.get J.Unsafe.global "OcamljsNative")
        "done" [||]
    in
    P.resolve ()
  in
  ()
