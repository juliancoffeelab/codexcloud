module B = Bindings

let start () =
  let todo = B.Todo.create () in
  let list = Ui.by_id "tasks" and message = Ui.by_id "message" in
  let rec render () =
    Ui.clear list;
    Array.iter
      (fun (task : B.task) ->
        let row = Ui.create "li" and title = Ui.create "span" in
        Ui.text title task.title;
        Ui.class_name title (if task.done_ then "done" else "");
        Ui.append row title;
        Ui.append row
          (Ui.button
             (if task.done_ then "Reopen" else "Done")
             (fun () ->
               let _ = B.Todo.toggle todo task.id in
               render ()));
        Ui.append row
          (Ui.button "Delete" (fun () ->
               let _ = B.Todo.remove todo task.id in
               render ()));
        Ui.append list row)
      (B.Todo.tasks todo);
    Ui.text (Ui.by_id "remaining")
      (string_of_int (int_of_float (B.Todo.remaining todo)) ^ " remaining")
  in
  Ui.on_click (Ui.by_id "add") (fun () ->
      let input = Ui.by_id "title" in
      match B.Todo.add todo (Ui.value input) with
      | Error B.Task_error.Empty_title -> Ui.text message "Enter a task title"
      | Error _ -> Ui.text message "Could not add task"
      | Ok _ ->
          Ui.set_value input "";
          Ui.text message "";
          render ());
  render ()

let () =
  Async.observe_error
    (Bindings.Wasm.load "./backend.wasm")
    (function
      | Error error -> Ui.failed (Bindings.Wasm.Load_error.message error)
      | Ok () -> (
          try
            start ();
            Ui.ready ()
          with error -> Ui.failed (Browser.exception_text error)))
    Ui.failed
