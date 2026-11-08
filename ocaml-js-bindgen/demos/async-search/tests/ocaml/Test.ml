let observe promise success failure =
  let settled =
    promise
    |> Js.Promise.then_ (fun value -> Js.Promise.resolve (Ok value))
    |> Js.Promise.catch (fun _ -> Js.Promise.resolve (Error ()))
  in
  let _ =
    settled
    |> Js.Promise.then_ (fun result ->
        (match result with Ok value -> success value | Error () -> failure ());
        Js.Promise.resolve ())
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
