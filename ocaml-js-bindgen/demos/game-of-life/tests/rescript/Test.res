module B = Bindings

let () = {
  assert(B.Universe.create(1., 1.) == Error(B.Universe.Invalid_dimensions))
  let u = switch B.Universe.create(5., 5.) {
  | Ok(u) => u
  | Error(_) => assert(false)
  }
  List.iter(x => assert(B.Universe.toggle(u, x, 2.) == Ok()), list{1., 2., 3.})
  let before = B.Universe.snapshot(u)
  assert(B.Universe.step(u) == 1.)
  assert(B.Universe.alive(u, 2., 1.) &&
  (B.Universe.alive(u, 2., 2.) &&
  B.Universe.alive(u, 2., 3.)))
  assert(!B.Universe.alive(u, 1., 2.))
  assert(before.cells[11])
  assert(B.Universe.toggle(u, 9., 3.) == Error(B.Universe.Outside_grid({x: 9., y: 3.})))
  B.Universe.dispose(u)
  print_endline("life native ABI passed")
}
