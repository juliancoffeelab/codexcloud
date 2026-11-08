module B = Bindings
module L = Lwt

let ( let* ) = L.bind

let done_ () : unit =
  let module J = Js_of_ocaml.Js in
  J.Unsafe.meth_call (J.Unsafe.get J.Unsafe.global "OcamljsTest") "done" [||]

let () =
  L.async (fun () ->
      let* result = B.fetch_cat "https://fixture/cat" in
      (match result with
      | Ok (cat : B.cat) -> assert (cat.id = "λ🦀")
      | Error _ -> assert false);
      let* result = B.fetch_cat "https://fixture/bad" in
      assert (result = Error B.Cat_error.Invalid_response);
      let* result = B.fetch_cat "https://fixture/failure" in
      assert (result = Error B.Cat_error.Network);
      let* () =
        L.catch
          (fun () ->
            let* _ = B.fetch_cat_throw "https://fixture/bad" in
            assert false)
          (function
            | B.Fetch_cat_throw_error B.Cat_error.Invalid_response -> L.return_unit
            | error -> L.fail error)
      in
      done_ ();
      L.return_unit)
