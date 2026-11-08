module B = Bindings

let get = function
  | Ok value -> value
  | Error _ -> failwith "Image operation failed"

let clamp low high value = max low (min high value)
let region x y width height : B.region = { x; y; width; height }

type mode = Draw of B.Tool.t | Hand | Select

type gesture =
  | Idle
  | Drawing
  | Pinching of float * float * float * float
  | Panning of float * float * float * float * bool
  | Selecting of float * float * B.region option
  | Transforming of B.region * Ui.handle * float * float

let start load_file =
  let document = ref (get (B.Paint.create 768. 512.)) in
  let canvas = get (B.Canvas.by_id "pixels") in
  let queued = ref false and dirty = ref false and version = ref 0 in
  let mode = ref (Draw B.Tool.Brush) and gesture = ref Idle in
  let selected : B.region option ref = ref None in
  let pending = ref None in
  let pan_x = ref 0. and pan_y = ref 0. and zoom = ref 1. in
  let auto_fit = ref true in
  let message text = Ui.text (Ui.by_id "message") text in
  let show_view () =
    Ui.view !pan_x !pan_y !zoom;
    Ui.text (Ui.by_id "zoom-value")
      (string_of_int (int_of_float (floor ((!zoom *. 100.) +. 0.5))) ^ "%")
  in
  let fit () =
    auto_fit := true;
    pan_x := 0.;
    pan_y := 0.;
    zoom := clamp 0.1 8. (Ui.fit_scale ());
    show_view ()
  in
  let show_selection () =
    Ui.hidden (Ui.by_id "selection-settings") (!selected = None);
    match !selected with
    | None -> Ui.selection 0. 0. 0. 0.
    | Some r ->
        Ui.selection r.x r.y r.width r.height;
        Ui.set_value
          (Ui.by_id "selection-width")
          (string_of_int (int_of_float r.width));
        Ui.set_value
          (Ui.by_id "selection-height")
          (string_of_int (int_of_float r.height))
  in
  let render () =
    dirty := false;
    let _ = get (B.Paint.draw !document canvas) in
    let state = B.Paint.status !document in
    Ui.text (Ui.by_id "dimensions")
      (string_of_int (int_of_float state.width)
      ^ " x "
      ^ string_of_int (int_of_float state.height));
    Ui.disabled (Ui.by_id "undo") (not state.can_undo);
    Ui.disabled (Ui.by_id "redo") (not state.can_redo);
    show_selection ()
  in
  let destination (original : B.region) handle sx sy x y =
    let state = B.Paint.status !document in
    let dx = floor (x -. sx +. 0.5) and dy = floor (y -. sy +. 0.5) in
    if handle = Ui.Surface then
      region
        (clamp 0. (state.width -. original.width) (original.x +. dx))
        (clamp 0. (state.height -. original.height) (original.y +. dy))
        original.width original.height
    else
      let right = original.x +. original.width
      and bottom = original.y +. original.height in
      let left =
        if handle = Ui.North_west || handle = Ui.South_west then
          clamp 0. (right -. 1.) (original.x +. dx)
        else original.x
      in
      let top =
        if handle = Ui.North_west || handle = Ui.North_east then
          clamp 0. (bottom -. 1.) (original.y +. dy)
        else original.y
      in
      let right =
        if handle = Ui.North_east || handle = Ui.South_east then
          clamp (left +. 1.) state.width (right +. dx)
        else right
      in
      let bottom =
        if handle = Ui.South_west || handle = Ui.South_east then
          clamp (top +. 1.) state.height (bottom +. dy)
        else bottom
      in
      region left top (right -. left) (bottom -. top)
  in
  let apply_pending () =
    match !pending with
    | None -> ()
    | Some (x, y) -> (
        pending := None;
        match !gesture with
        | Drawing -> B.Paint.move_to !document x y
        | Selecting (sx, sy, _) ->
            let state = B.Paint.status !document in
            let x = clamp 0. state.width (floor x)
            and y = clamp 0. state.height (floor y) in
            selected :=
              Some
                (region (min sx x) (min sy y)
                   (abs_float (x -. sx))
                   (abs_float (y -. sy)))
        | Transforming (original, handle, sx, sy) ->
            let next = destination original handle sx sy x y in
            let _ = get (B.Paint.transform_to !document next) in
            selected := Some next
        | _ -> ())
  in
  let refresh () =
    dirty := true;
    if not !queued then (
      queued := true;
      Ui.frame (fun () ->
          queued := false;
          apply_pending ();
          if !dirty then
            match !gesture with
            | Selecting _ ->
                dirty := false;
                show_selection ()
            | _ -> render ()))
  in
  let stop () =
    apply_pending ();
    (match !gesture with
    | Drawing | Transforming _ -> B.Paint.finish !document
    | Selecting _ -> (
        match !selected with
        | Some r when r.width = 0. || r.height = 0. -> selected := None
        | _ -> ())
    | _ -> ());
    gesture := Idle;
    pending := None
  in
  let cancel () =
    pending := None;
    (match !gesture with
    | Drawing -> B.Paint.cancel !document
    | Transforming (original, _, _, _) ->
        B.Paint.cancel !document;
        selected := Some original
    | Selecting (_, _, previous) -> selected := previous
    | Panning (_, _, x, y, fitted) ->
        pan_x := x;
        pan_y := y;
        auto_fit := fitted;
        show_view ()
    | Pinching _ | Idle -> ());
    gesture := Idle;
    render ()
  in
  let edit action =
    incr version;
    stop ();
    selected := None;
    action !document;
    render ()
  in
  let select name value =
    stop ();
    mode := value;
    (match value with Draw _ -> selected := None | _ -> ());
    Ui.set_attribute (Ui.by_id "workspace") "data-tool" name;
    List.iter
      (fun id ->
        Ui.set_attribute (Ui.by_id id) "aria-pressed"
          (if id = name then "true" else "false"))
      [
        "hand";
        "select";
        "brush";
        "eraser";
        "fill";
        "line";
        "rectangle";
        "ellipse";
      ];
    show_selection ();
    message
      (match value with
      | Hand -> "Drag to pan. Pinch with two fingers or use + and - to zoom."
      | Select -> "Drag a selection. Move it or drag a corner to resize."
      | _ -> String.capitalize_ascii name ^ " selected")
  in
  List.iter
    (fun (name, value, key) ->
      let button = Ui.by_id name in
      Ui.on_click button (fun () -> select name value);
      Ui.shortcut button key)
    [
      ("hand", Hand, "h");
      ("select", Select, "v");
      ("brush", Draw B.Tool.Brush, "b");
      ("eraser", Draw B.Tool.Eraser, "e");
      ("fill", Draw B.Tool.Fill, "f");
      ("line", Draw B.Tool.Line, "l");
      ("rectangle", Draw B.Tool.Rectangle, "r");
      ("ellipse", Draw B.Tool.Ellipse, "o");
    ];
  List.iter
    (fun color ->
      Ui.on_click
        (Ui.by_id ("color-" ^ color))
        (fun () ->
          Ui.set_value (Ui.by_id "color") ("#" ^ color);
          message ("Color #" ^ color)))
    [
      "20252b";
      "ffffff";
      "e75b52";
      "f2a541";
      "f4d35e";
      "78b887";
      "398c86";
      "5a98d0";
      "5865a8";
      "9276b5";
      "df94b5";
      "a4775b";
    ];
  Ui.on_change (Ui.by_id "size") (fun () ->
      Ui.text (Ui.by_id "size-value") (Ui.value (Ui.by_id "size") ^ " px"));
  Ui.on_pointer (Ui.by_id "workspace")
    (fun () ->
      stop ();
      let x = Ui.pointer_x () and y = Ui.pointer_y () in
      let state = B.Paint.status !document in
      match !mode with
      | Hand ->
          gesture :=
            Panning
              ( Ui.pointer_screen_x (),
                Ui.pointer_screen_y (),
                !pan_x,
                !pan_y,
                !auto_fit )
      | _
        when (x < 0. || y < 0. || x >= state.width || y >= state.height)
             && Ui.pointer_handle () = Ui.Surface ->
          ()
      | Draw tool ->
          incr version;
          gesture := Drawing;
          let color = Ui.value (Ui.by_id "color") in
          let rgb = int_of_string ("0x" ^ String.sub color 1 6) in
          B.Paint.begin_ !document x y tool (float_of_int rgb)
            (float_of_string (Ui.value (Ui.by_id "size")));
          refresh ()
      | Select -> (
          let handle = Ui.pointer_handle () in
          match !selected with
          | Some r
            when handle <> Ui.Surface
                 || x >= r.x && y >= r.y
                    && x <= r.x +. r.width
                    && y <= r.y +. r.height ->
              incr version;
              let _ = get (B.Paint.transform_start !document r) in
              gesture := Transforming (r, handle, x, y)
          | previous ->
              gesture := Selecting (floor x, floor y, previous);
              selected := None;
              show_selection ()))
    (fun () ->
      match !gesture with
      | Panning (sx, sy, px, py, _) ->
          auto_fit := false;
          pan_x := px +. Ui.pointer_screen_x () -. sx;
          pan_y := py +. Ui.pointer_screen_y () -. sy;
          show_view ()
      | Drawing -> (
          match !mode with
          | Draw (B.Tool.Brush | B.Tool.Eraser) ->
              B.Paint.move_to !document (Ui.pointer_x ()) (Ui.pointer_y ());
              refresh ()
          | Draw B.Tool.Fill -> ()
          | _ ->
              pending := Some (Ui.pointer_x (), Ui.pointer_y ());
              refresh ())
      | Selecting _ | Transforming _ ->
          pending := Some (Ui.pointer_x (), Ui.pointer_y ());
          refresh ()
      | Pinching _ | Idle -> ())
    (fun () ->
      (match !gesture with
      | Drawing | Selecting _ | Transforming _ ->
          pending := Some (Ui.pointer_x (), Ui.pointer_y ())
      | _ -> ());
      stop ();
      render ())
    (fun () ->
      cancel ();
      message "Gesture cancelled")
    (fun () ->
      (match !gesture with Panning _ -> gesture := Idle | _ -> cancel ());
      gesture :=
        Pinching
          ( max 1. (Ui.pinch_distance ()),
            !zoom,
            (Ui.pinch_x () -. !pan_x) /. !zoom,
            (Ui.pinch_y () -. !pan_y) /. !zoom ))
    (fun () ->
      match !gesture with
      | Pinching (distance, initial_zoom, anchor_x, anchor_y) ->
          auto_fit := false;
          zoom := clamp 0.1 8. (initial_zoom *. Ui.pinch_distance () /. distance);
          pan_x := Ui.pinch_x () -. (anchor_x *. !zoom);
          pan_y := Ui.pinch_y () -. (anchor_y *. !zoom);
          show_view ()
      | _ -> ())
    (fun () -> gesture := Idle);
  let set_zoom value =
    stop ();
    auto_fit := false;
    zoom := clamp 0.1 8. value;
    show_view ()
  in
  Ui.on_click (Ui.by_id "zoom-in") (fun () -> set_zoom (!zoom *. 1.25));
  Ui.on_click (Ui.by_id "zoom-out") (fun () -> set_zoom (!zoom /. 1.25));
  Ui.on_click (Ui.by_id "fit") (fun () ->
      stop ();
      fit ());
  Ui.on_resize (fun () ->
      if !auto_fit then (
        stop ();
        fit ()));
  Ui.on_wheel (Ui.by_id "workspace") (fun () ->
      set_zoom (!zoom *. exp (-.Ui.wheel_delta () /. 400.)));
  Ui.on_click (Ui.by_id "deselect") (fun () ->
      stop ();
      selected := None;
      show_selection ());
  let delete_selection () =
    stop ();
    match !selected with
    | None -> ()
    | Some r ->
        incr version;
        let _ = get (B.Paint.delete_region !document r) in
        selected := None;
        render ();
        message "Selection deleted - undo to restore"
  in
  Ui.on_click (Ui.by_id "delete-selection") delete_selection;
  Ui.on_click (Ui.by_id "resize-selection") (fun () ->
      stop ();
      match !selected with
      | None -> ()
      | Some r -> (
          try
            let state = B.Paint.status !document in
            let width = float_of_string (Ui.value (Ui.by_id "selection-width"))
            and height =
              float_of_string (Ui.value (Ui.by_id "selection-height"))
            in
            if classify_float width = FP_nan || classify_float height = FP_nan
            then failwith "Invalid size";
            let next =
              region r.x r.y
                (clamp 1. (state.width -. r.x) (floor width))
                (clamp 1. (state.height -. r.y) (floor height))
            in
            incr version;
            let _ = get (B.Paint.transform_start !document r) in
            let _ = get (B.Paint.transform_to !document next) in
            B.Paint.finish !document;
            selected := Some next;
            render ();
            message "Selection resized"
          with Failure _ -> message "Enter a width and height in pixels"));
  Ui.on_key (fun () ->
      let key = Ui.key () in
      if key = "escape" then (
        cancel ();
        selected := None;
        show_selection ();
        true)
      else if key = "delete" || key = "backspace" then (
        match !selected with
        | None -> false
        | Some _ ->
            delete_selection ();
            true)
      else if key = "mod+a" then (
        select "select" Select;
        let state = B.Paint.status !document in
        selected := Some (region 0. 0. state.width state.height);
        show_selection ();
        true)
      else
        let shift = String.length key > 6 && String.sub key 0 6 = "shift+" in
        let plain =
          if shift then String.sub key 6 (String.length key - 6) else key
        in
        let distance = if shift then 10. else 1. in
        let dx, dy =
          match plain with
          | "arrowleft" -> (-.distance, 0.)
          | "arrowright" -> (distance, 0.)
          | "arrowup" -> (0., -.distance)
          | "arrowdown" -> (0., distance)
          | _ -> (0., 0.)
        in
        if dx = 0. && dy = 0. then false
        else (
          stop ();
          match !selected with
          | Some r ->
              incr version;
              let next = destination r Ui.Surface 0. 0. dx dy in
              let _ = get (B.Paint.transform_start !document r) in
              let _ = get (B.Paint.transform_to !document next) in
              B.Paint.finish !document;
              selected := Some next;
              render ();
              true
          | _ -> false));
  Ui.on_click (Ui.by_id "undo") (fun () ->
      edit B.Paint.undo;
      message "Undone");
  Ui.on_click (Ui.by_id "redo") (fun () ->
      edit B.Paint.redo;
      message "Redone");
  Ui.shortcut (Ui.by_id "undo") "mod+z";
  Ui.shortcut (Ui.by_id "redo") "mod+shift+z";
  Ui.shortcut (Ui.by_id "redo") "mod+y";
  Ui.on_click (Ui.by_id "clear") (fun () ->
      edit B.Paint.clear;
      message "Canvas cleared - undo to restore");
  Ui.on_click (Ui.by_id "new") (fun () ->
      incr version;
      pending := None;
      gesture := Idle;
      selected := None;
      let next = get (B.Paint.create 768. 512.) in
      B.Paint.dispose !document;
      document := next;
      Ui.text (Ui.by_id "filename") "Untitled";
      render ();
      fit ();
      message "Fresh canvas, fresh ideas");
  Ui.on_change (Ui.by_id "file") (fun () ->
      incr version;
      let request = !version in
      message "Opening PNG...";
      load_file
        (fun result ->
          match result with
          | Ok next ->
              if request <> !version then B.Paint.dispose next
              else (
                pending := None;
                gesture := Idle;
                selected := None;
                B.Paint.dispose !document;
                document := next;
                Ui.text (Ui.by_id "filename") "Imported image";
                render ();
                fit ();
                message "PNG opened")
          | Error B.Image_error.Too_large ->
              if request = !version then
                message "Choose a PNG up to 2048 x 2048 and 8 MB"
          | Error _ ->
              if request = !version then
                message "Could not open that PNG. Your drawing is safe.")
        (fun () ->
          if request = !version then
            message "Could not read that file. Your drawing is safe."));
  Ui.on_click (Ui.by_id "save") (fun () ->
      stop ();
      match B.Paint.png !document with
      | Ok bytes ->
          Ui.save_png bytes;
          message "PNG saved - keep creating"
      | Error _ -> message "Could not save the image. Please try again.");
  Ui.shortcut (Ui.by_id "save") "mod+s";
  render ();
  fit ();
  message "Pick a color and make your mark"
