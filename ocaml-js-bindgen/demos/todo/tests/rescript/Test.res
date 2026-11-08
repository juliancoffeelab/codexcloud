module B = Bindings

let () = {
  let t = B.Todo.create()
  assert(B.Todo.add(t, "") == Error(B.Task_error.Empty_title))
  let id = switch B.Todo.add(t, "Write OCaml UI") {
  | Ok(id) => id
  | Error(_) => assert(false)
  }

  assert(B.Todo.remaining(t) == 1.)
  assert(B.Todo.toggle(t, id) == Ok())
  let snapshot = B.Todo.tasks(t)
  assert(snapshot[0].done_)
  assert(B.Todo.remaining(t) == 0.)
  assert(B.Todo.remove(t, id) == Ok())
  assert(Array.length(B.Todo.tasks(t)) == 0)
  assert(snapshot[0].title == "Write OCaml UI")
  assert(B.Todo.toggle(t, id) == Error(B.Task_error.Unknown_task(id)))
  B.Todo.dispose(t)
  let consumed = B.Todo.create()
  try {
    ignore(B.Todo.checked_add(consumed, ""))
    assert(false)
  } catch {
  | B.Todo.Checked_add_error(B.Task_error.Empty_title) => ()
  }
  ignore(B.Todo.checked_add(consumed, "Consume this owner"))
  assert(B.Todo.into_count(consumed) == 1.)
  assert(try {
    ignore(B.Todo.remaining(consumed))
    false
  } catch {
  | _ => true
  })
  print_endline("todo native ABI passed")
}
