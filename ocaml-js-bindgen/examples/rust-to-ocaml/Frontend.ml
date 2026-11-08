module B = Bindings
module G = B.Geometry
module T = B.Token
module Ts = B.Tokens
module J = Js_of_ocaml.Js
module P = Js_of_ocaml.Promise

let document = J.Unsafe.get J.Unsafe.global "document"

let element id =
  J.Unsafe.meth_call document "getElementById"
    [| J.Unsafe.inject (J.string id) |]

let text id value = J.Unsafe.set (element id) "textContent" (J.string value)
let value id = J.to_string (J.Unsafe.get (element id) "value")

let on_click id action =
  J.Unsafe.set (element id) "onclick"
    (J.wrap_callback (fun _ ->
         action ();
         J._false))

let error_text error =
  J.to_string
    (J.Unsafe.fun_call
       (J.Unsafe.get J.Unsafe.global "String")
       [| P.error_to_any error |])

let read_token () =
  match Ts.parse (value "token-input") with
  | Error message -> text "token-result" message
  | Ok token ->
      Fun.protect
        ~finally:(fun () -> T.dispose token)
        (fun () ->
          text "token-result"
            (Printf.sprintf "Token value: %.0f" (T.value token)))

let fetch_cat () =
  text "cat-status" "Loading…";
  let _ =
    P.then_
      ~on_error:(fun error ->
        text "cat-status" (error_text error);
        P.resolve ())
      (fun result ->
        (match result with
        | Ok (cat : B.cat) ->
            J.Unsafe.set (element "cat") "src" (J.string cat.url);
            text "cat-status" ("Cat " ^ cat.id)
        | Error B.Cat_error.Network -> text "cat-status" "Network error"
        | Error B.Cat_error.Too_large -> text "cat-status" "Response too large"
        | Error B.Cat_error.Invalid_response ->
            text "cat-status" "Invalid cat response");
        P.resolve ())
      (B.fetch_cat (value "cat-api"))
  in
  ()

let start () =
  let a : G.point = { x = 0.; y = 0. } and b : G.point = { x = 3.; y = 4. } in
  assert (G.distance a b = 5.);
  text "distance" (Printf.sprintf "Distance: %.0f" (G.distance a b));
  read_token ();
  on_click "read-token" read_token;
  on_click "fetch-cat" fetch_cat;
  let _ =
    J.Unsafe.meth_call
      (J.Unsafe.get document "documentElement")
      "setAttribute"
      [|
        J.Unsafe.inject (J.string "data-ready");
        J.Unsafe.inject (J.string "true");
      |]
  in
  ()

let () =
  let _ =
    P.catch
      (fun error ->
        text "distance" (error_text error);
        P.resolve ())
      (P.map
         (function
           | Ok () -> start ()
           | Error error -> failwith (B.Wasm.Load_error.message error))
         (B.Wasm.load "./backend.wasm"))
  in
  ()
