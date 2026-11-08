module B = Bindings

external done_ : unit -> unit = "done" [@@mel.scope "OcamljsTest"]

let () =
  let _ =
    B.fetch_cat "https://fixture/cat"
    |> Js.Promise.then_ (fun result ->
        (match result with
        | Ok (cat : B.cat) -> assert (cat.id = "λ🦀")
        | Error _ -> assert false);
        B.fetch_cat "https://fixture/bad")
    |> Js.Promise.then_ (fun result ->
        assert (result = Error B.Cat_error.Invalid_response);
        B.fetch_cat "https://fixture/failure")
    |> Js.Promise.then_ (fun result ->
        assert (result = Error B.Cat_error.Network);
        B.fetch_cat_throw "https://fixture/bad"
        |> Js.Promise.then_ (fun _ -> Js.Promise.resolve false)
        |> Js.Promise.catch (fun error ->
            match Js.Exn.anyToExnInternal error with
            | B.Fetch_cat_throw_error B.Cat_error.Invalid_response ->
                Js.Promise.resolve true
            | _ -> Js.Promise.resolve false)
        |> Js.Promise.then_ (fun thrown ->
            assert thrown;
            done_ ();
            Js.Promise.resolve ()))
  in
  ()
