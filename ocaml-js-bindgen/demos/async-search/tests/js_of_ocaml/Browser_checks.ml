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

let run done_ =
  let before = B.live_entries () in
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
  assert (B.Input.push input "{\"text\":\"OCaml proof λ\"}\n" = Ok ());
  B.Input.finish input;
  observe pending
    (function
      | Ok index ->
          assert (B.Index.query index "proof" = [| "OCaml proof λ" |]);
          B.Index.dispose index;
          B.Input.dispose input;
          B.Corpus.dispose corpus;
          B.Corpus.dispose other;
          assert (B.live_entries () = before);
          let native_corpus = B.Corpus.create () in
          let source =
            match B.Blob.of_text "{\"text\":\"Blob proof λ🦀\"}\n" with
            | Ok value -> value
            | Error _ -> assert false
          in
          let pending = B.Index.of_blob native_corpus source (fun _ -> ()) in
          assert (rejects (fun () -> B.Corpus.dispose native_corpus));
          observe pending
            (function
              | Ok native_index ->
                  assert (
                    B.Index.query native_index "proof" = [| "Blob proof λ🦀" |]);
                  B.Index.dispose native_index;
                  B.Corpus.dispose native_corpus;
                  assert (B.live_entries () = before);
                  done_ true
              | _ -> done_ false)
            (fun () -> done_ false)
      | _ -> done_ false)
    (fun () -> done_ false)
