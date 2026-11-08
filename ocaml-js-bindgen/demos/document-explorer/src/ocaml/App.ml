module B = Bindings

let node_kind_label = function
  | B.Node_kind.Object -> "object"
  | B.Node_kind.Array -> "array"
  | B.Node_kind.String -> "string"
  | B.Node_kind.Number -> "number"
  | B.Node_kind.Bool -> "bool"
  | B.Node_kind.Null -> "null"

let start load_blob =
  let current = ref None and selected = ref None in
  let version = ref 0 in
  let message text = Ui.text (Ui.by_id "message") text in
  let clear_view () =
    Option.iter B.NodeView.dispose !selected;
    selected := None;
    Ui.text (Ui.by_id "selection") "No live view"
  in
  let render () =
    let list = Ui.by_id "nodes" in
    Ui.clear list;
    Option.iter
      (fun document ->
        Array.iter
          (fun (node : B.summary) ->
            Ui.append list
              (Ui.button
                 (node.label ^ " (" ^ node_kind_label node.kind ^ ")")
                 (fun () ->
                   clear_view ();
                   match B.NodeView.create document node.id with
                   | Error _ -> message "Stale node"
                   | Ok view ->
                       selected := Some view;
                       Ui.text (Ui.by_id "selection") (B.NodeView.text view))))
          (B.Document.nodes document))
      !current
  in
  let create () =
    incr version;
    clear_view ();
    Option.iter B.Document.dispose !current;
    current := None;
    match B.Document.create (Ui.value (Ui.by_id "source")) with
    | Error _ ->
        render ();
        message "Invalid JSON"
    | Ok document ->
        current := Some document;
        render ();
        message "Document parsed"
  in
  Ui.on_click (Ui.by_id "parse") create;
  Ui.on_click (Ui.by_id "replace") (fun () ->
      Option.iter
        (fun document ->
          match B.Document.replace document (Ui.value (Ui.by_id "source")) with
          | Error B.Document_error.Views_present ->
              message "Release live views before mutation"
          | Error _ -> message "Invalid JSON"
          | Ok () ->
              incr version;
              render ();
              message "Document replaced; old node IDs stale")
        !current);
  Ui.on_click (Ui.by_id "release-view") clear_view;
  Ui.on_click (Ui.by_id "release-parent") (fun () ->
      Option.iter
        (fun document ->
          incr version;
          B.Document.dispose document;
          current := None;
          render ();
          match !selected with
          | None -> message "Document released"
          | Some view ->
              Ui.text (Ui.by_id "selection") (B.NodeView.text view);
              message "Parent owner released; selected view retains storage")
        !current);
  Ui.on_click (Ui.by_id "load-file") (fun () ->
      match B.Blob.from_input "file" with
      | Error _ -> message "Select a JSON file"
      | Ok source ->
          incr version;
          let request = !version in
          message "Reading file…";
          load_blob source
            (fun result ->
              match result with
              | Ok document ->
                  if request <> !version then B.Document.dispose document
                  else (
                    clear_view ();
                    Option.iter B.Document.dispose !current;
                    current := Some document;
                    render ();
                    message "JSON file loaded")
              | Error B.Document_error.Too_large ->
                  if request = !version then message "File is too large"
              | Error _ -> if request = !version then message "File could not be loaded")
            (fun () -> if request = !version then message "Binding operation failed"));
  create ()
