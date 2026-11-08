module B = Bindings

let node_kind_label = kind =>
  switch kind {
  | B.Node_kind.Object => "object"
  | B.Node_kind.Array => "array"
  | B.Node_kind.String => "string"
  | B.Node_kind.Number => "number"
  | B.Node_kind.Bool => "bool"
  | B.Node_kind.Null => "null"
  }

let start = load_blob => {
  let current = ref(None) and selected = ref(None)
  let version = ref(0)
  let message = text => Ui.text(Ui.by_id("message"), text)
  let clear_view = () => {
    Belt.Option.forEach(selected.contents, B.NodeView.dispose)
    selected := None
    Ui.text(Ui.by_id("selection"), "No live view")
  }

  let render = () => {
    let list = Ui.by_id("nodes")
    Ui.clear(list)
    Belt.Option.forEach(current.contents, document => Array.iter((node: B.summary) =>
        Ui.append(
          list,
          Ui.button(
            node.label ++ (" (" ++ (node_kind_label(node.kind) ++ ")")),
            () => {
              clear_view()
              switch B.NodeView.create(document, node.id) {
              | Error(_) => message("Stale node")
              | Ok(view) =>
                selected := Some(view)
                Ui.text(Ui.by_id("selection"), B.NodeView.text(view))
              }
            },
          ),
        )
      , B.Document.nodes(document)))
  }

  let create = () => {
    incr(version)
    clear_view()
    Belt.Option.forEach(current.contents, B.Document.dispose)
    current := None
    switch B.Document.create(Ui.value(Ui.by_id("source"))) {
    | Error(_) =>
      render()
      message("Invalid JSON")
    | Ok(document) =>
      current := Some(document)
      render()
      message("Document parsed")
    }
  }

  Ui.on_click(Ui.by_id("parse"), create)
  Ui.on_click(Ui.by_id("replace"), () =>
    Belt.Option.forEach(current.contents, document =>
      switch B.Document.replace(document, Ui.value(Ui.by_id("source"))) {
      | Error(B.Document_error.Views_present) => message("Release live views before mutation")
      | Error(_) => message("Invalid JSON")
      | Ok() =>
        incr(version)
        render()
        message("Document replaced; old node IDs stale")
      }
    )
  )
  Ui.on_click(Ui.by_id("release-view"), clear_view)
  Ui.on_click(Ui.by_id("release-parent"), () =>
    Belt.Option.forEach(current.contents, document => {
      incr(version)
      B.Document.dispose(document)
      current := None
      render()
      switch selected.contents {
      | None => message("Document released")
      | Some(view) =>
        Ui.text(Ui.by_id("selection"), B.NodeView.text(view))
        message("Parent owner released; selected view retains storage")
      }
    })
  )
  Ui.on_click(Ui.by_id("load-file"), () =>
    switch B.Blob.from_input("file") {
    | Error(_) => message("Select a JSON file")
    | Ok(source) =>
      incr(version)
      let request = version.contents
      message("Reading file…")
      load_blob(
        source,
        result =>
          switch result {
          | Ok(document) =>
            if request != version.contents {
              B.Document.dispose(document)
            } else {
              clear_view()
              Belt.Option.forEach(current.contents, B.Document.dispose)
              current := Some(document)
              render()
              message("JSON file loaded")
            }
          | Error(B.Document_error.Too_large) =>
            if request == version.contents {
              message("File is too large")
            }
          | Error(_) =>
            if request == version.contents {
              message("File could not be loaded")
            }
          },
        () =>
          if request == version.contents {
            message("Binding operation failed")
          },
      )
    }
  )
  create()
}
