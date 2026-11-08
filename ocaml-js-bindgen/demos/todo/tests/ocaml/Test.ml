module B = Bindings

let () =
  let t = B.Todo.create () in
  assert (B.Todo.add t "" = Error B.Task_error.Empty_title);
  let id =
    match B.Todo.add t "Write OCaml UI" with
    | Ok id -> id
    | Error _ -> assert false
  in
  assert (B.Todo.remaining t = 1.);
  assert (B.Todo.toggle t id = Ok ());
  let snapshot = B.Todo.tasks t in
  assert snapshot.(0).done_;
  assert (B.Todo.remaining t = 0.);
  assert (B.Todo.remove t id = Ok ());
  assert (Array.length (B.Todo.tasks t) = 0);
  assert (snapshot.(0).title = "Write OCaml UI");
  assert (B.Todo.toggle t id = Error (B.Task_error.Unknown_task id));
  B.Todo.dispose t;
  let consumed = B.Todo.create () in
  (try
     let _ = B.Todo.checked_add consumed "" in
     assert false
   with B.Todo.Checked_add_error B.Task_error.Empty_title -> ());
  let _ = B.Todo.checked_add consumed "Consume this owner" in
  assert (B.Todo.into_count consumed = 1.);
  assert (
    try
      let _ = B.Todo.remaining consumed in
      false
    with _ -> true);
  print_endline "todo native ABI passed"
