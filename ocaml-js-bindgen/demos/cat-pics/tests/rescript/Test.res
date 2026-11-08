module B = Bindings
@val @scope("OcamljsTest") external done_: unit => unit = "done"
let run = async () => {
  switch await B.fetch_cat("https://fixture/cat") {
  | Ok(cat) => assert(cat.id == "λ🦀")
  | Error(_) => assert(false)
  }
  assert((await B.fetch_cat("https://fixture/bad")) == Error(B.Cat_error.Invalid_response))
  assert((await B.fetch_cat("https://fixture/failure")) == Error(B.Cat_error.Network))
  let thrown = try {
    let _ = await B.fetch_cat_throw("https://fixture/bad")
    false
  } catch {
  | B.Fetch_cat_throw_error(B.Cat_error.Invalid_response) => true
  | _ => false
  }
  assert(thrown)
  done_()
}
let () = run()->ignore
