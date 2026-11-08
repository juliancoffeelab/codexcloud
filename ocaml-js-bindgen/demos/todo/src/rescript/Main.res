module B = Bindings

let start = () => {
  let todo = B.Todo.create()
  let list = Ui.by_id("tasks") and message = Ui.by_id("message")
  let rec render = () => {
    Ui.clear(list)
    Array.iter((task: B.task) => {
      let row = Ui.create("li") and title = Ui.create("span")
      Ui.text(title, task.title)
      Ui.class_name(
        title,
        if task.done_ {
          "done"
        } else {
          ""
        },
      )
      Ui.append(row, title)
      Ui.append(
        row,
        Ui.button(
          if task.done_ {
            "Reopen"
          } else {
            "Done"
          },
          () => {
            ignore(B.Todo.toggle(todo, task.id))
            render()
          },
        ),
      )
      Ui.append(
        row,
        Ui.button("Delete", () => {
          ignore(B.Todo.remove(todo, task.id))
          render()
        }),
      )
      Ui.append(list, row)
    }, B.Todo.tasks(todo))
    Ui.text(
      Ui.by_id("remaining"),
      string_of_int(int_of_float(B.Todo.remaining(todo))) ++ " remaining",
    )
  }

  Ui.on_click(Ui.by_id("add"), () => {
    let input = Ui.by_id("title")
    switch B.Todo.add(todo, Ui.value(input)) {
    | Error(B.Task_error.Empty_title) => Ui.text(message, "Enter a task title")
    | Error(_) => Ui.text(message, "Could not add task")
    | Ok(_) =>
      Ui.set_value(input, "")
      Ui.text(message, "")
      render()
    }
  })
  render()
}

let () = Async.observe_error(
  Bindings.Wasm.load("./backend.wasm"),
  loadResult =>
    switch loadResult {
    | Error(error) => Ui.failed(Bindings.Wasm.Load_error.message(error))
    | Ok() =>
      try {
        start()
        Ui.ready()
      } catch {
      | error => Ui.failed(Browser.exception_text(error))
      }
    },
  Ui.failed,
)
