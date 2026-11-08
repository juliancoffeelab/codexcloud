module B = Bindings

let ok = x =>
  switch x {
  | Ok(x) => x
  | Error(_) => failwith("unexpected document error")
  }

let () = {
  let d = ok(B.Document.create("{\"items\":[1,2]}"))
  let entries = B.Document.nodes(d)
  assert(entries[0].kind == B.Node_kind.Object)
  let id = entries[0].id
  let other = ok(B.Document.create("[]"))
  assert(B.Document.inspect(other, id) == Error(B.Document_error.Stale_node))
  let view = ok(B.NodeView.create(d, id))
  let child = ok(B.NodeView.child(view, "items"))
  assert(B.Document.replace(d, "{}") == Error(B.Document_error.Views_present))
  B.Document.dispose(d)
  assert(B.NodeView.text(child) == "[1,2]")
  B.NodeView.dispose(view)
  assert(B.NodeView.text(child) == "[1,2]")
  B.NodeView.dispose(child)
  let old = B.Document.nodes(other)[0].id
  ignore(ok(B.Document.replace(other, "[0,1,2,3,4,5,6,7,8,9]")))
  assert(B.Document.inspect(other, old) == Error(B.Document_error.Stale_node))
  B.Document.dispose(other)
}

module P = Js.Promise
let bind = (promise, continuation) => P.then_(continuation, promise)
let return = P.resolve
@scope("OcamljsTest") external done_: unit => unit = "done"
let run = () => {
  let blob = ok(B.Blob.of_text("{\"title\":\"OCaml λ🦀\"}"))
  bind(B.Document.of_blob(blob), result => {
    let document = ok(result)
    switch B.Document.source(document) {
    | Some(source) => assert(source === blob)
    | None => assert(false)
    }
    let root = B.Document.nodes(document)[0].id
    assert(B.Document.inspect(document, root) == Ok("{\"title\":\"OCaml λ🦀\"}"))
    ignore(ok(B.Document.replace(document, "{}")))
    assert(B.Document.source(document) == None)
    B.Document.dispose(document)
    let invalid = ok(B.Blob.of_text("not JSON"))
    bind(B.Document.of_blob(invalid), result => {
      assert(result == Error(B.Document_error.Invalid_json))
      let large = ok(B.Blob.of_text(String.make(65537, 'x')))
      bind(
        B.Document.of_blob(large),
        result => {
          assert(result == Error(B.Document_error.Too_large))
          done_()
          return()
        },
      )
    })
  })
}
let () = ignore(run())
