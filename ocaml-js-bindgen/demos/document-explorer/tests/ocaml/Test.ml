module B = Bindings

let ok = function Ok x -> x | Error _ -> failwith "unexpected document error"

let () =
  let d = ok (B.Document.create "{\"items\":[1,2]}") in
  let entries = B.Document.nodes d in
  assert (entries.(0).kind = B.Node_kind.Object);
  let id = entries.(0).id in
  let other = ok (B.Document.create "[]") in
  assert (B.Document.inspect other id = Error B.Document_error.Stale_node);
  let view = ok (B.NodeView.create d id) in
  let child = ok (B.NodeView.child view "items") in
  assert (B.Document.replace d "{}" = Error B.Document_error.Views_present);
  B.Document.dispose d;
  assert (B.NodeView.text child = "[1,2]");
  B.NodeView.dispose view;
  assert (B.NodeView.text child = "[1,2]");
  B.NodeView.dispose child;
  let old = (B.Document.nodes other).(0).id in
  let _ = ok (B.Document.replace other "[0,1,2,3,4,5,6,7,8,9]") in
  assert (B.Document.inspect other old = Error B.Document_error.Stale_node);
  B.Document.dispose other

module P = Js.Promise

let bind promise continuation = P.then_ continuation promise
let return = P.resolve

external done_ : unit -> unit = "done" [@@mel.scope "OcamljsTest"]

let run () =
  let blob = ok (B.Blob.of_text "{\"title\":\"OCaml λ🦀\"}") in
  bind (B.Document.of_blob blob) (fun result ->
      let document = ok result in
      (match B.Document.source document with
      | Some source -> assert (source == blob)
      | None -> assert false);
      let root = (B.Document.nodes document).(0).id in
      assert (B.Document.inspect document root = Ok "{\"title\":\"OCaml λ🦀\"}");
      let _ = ok (B.Document.replace document "{}") in
      assert (B.Document.source document = None);
      B.Document.dispose document;
      let invalid = ok (B.Blob.of_text "not JSON") in
      bind (B.Document.of_blob invalid) (fun result ->
          assert (result = Error B.Document_error.Invalid_json);
          let large = ok (B.Blob.of_text (String.make 65537 'x')) in
          bind (B.Document.of_blob large) (fun result ->
              assert (result = Error B.Document_error.Too_large);
              done_ ();
              return ())))

let () =
  let _ = run () in
  ()
