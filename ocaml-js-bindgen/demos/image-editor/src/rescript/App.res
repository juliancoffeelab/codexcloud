module B = Bindings

let get = x =>
  switch x {
  | Ok(value) => value
  | Error(_) => failwith("Image operation failed")
  }
let clamp = (low, high, value) => max(low, min(high, value))
let region = (x, y, width, height): B.region => {x, y, width, height}

type mode = Draw(B.Tool.t) | Hand | Select
type gesture =
  | Idle
  | Drawing
  | Pinching(float, float, float, float)
  | Panning(float, float, float, float, bool)
  | Selecting(float, float, option<B.region>)
  | Transforming(B.region, Ui.handle, float, float)

let start = load_file => {
  let document = ref(get(B.Paint.create(768., 512.)))
  let canvas = get(B.Canvas.by_id("pixels"))
  let queued = ref(false) and dirty = ref(false) and version = ref(0)
  let mode = ref(Draw(B.Tool.Brush)) and gesture = ref(Idle)
  let selected: ref<option<B.region>> = ref(None)
  let pending = ref(None)
  let pan_x = ref(0.) and pan_y = ref(0.) and zoom = ref(1.)
  let auto_fit = ref(true)
  let message = text => Ui.text(Ui.by_id("message"), text)
  let show_view = () => {
    Ui.view(pan_x.contents, pan_y.contents, zoom.contents)
    Ui.text(
      Ui.by_id("zoom-value"),
      string_of_int(int_of_float(floor(zoom.contents *. 100. +. 0.5))) ++ "%",
    )
  }

  let fit = () => {
    auto_fit := true
    pan_x := 0.
    pan_y := 0.
    zoom := clamp(0.1, 8., Ui.fit_scale())
    show_view()
  }

  let show_selection = () => {
    Ui.hidden(Ui.by_id("selection-settings"), selected.contents == None)
    switch selected.contents {
    | None => Ui.selection(0., 0., 0., 0.)
    | Some(r) =>
      Ui.selection(r.x, r.y, r.width, r.height)
      Ui.set_value(Ui.by_id("selection-width"), string_of_int(int_of_float(r.width)))
      Ui.set_value(Ui.by_id("selection-height"), string_of_int(int_of_float(r.height)))
    }
  }

  let render = () => {
    dirty := false
    ignore(get(B.Paint.draw(document.contents, canvas)))
    let state = B.Paint.status(document.contents)
    Ui.text(
      Ui.by_id("dimensions"),
      string_of_int(int_of_float(state.width)) ++
      (" x " ++
      string_of_int(int_of_float(state.height))),
    )
    Ui.disabled(Ui.by_id("undo"), !state.can_undo)
    Ui.disabled(Ui.by_id("redo"), !state.can_redo)
    show_selection()
  }

  let destination = (original: B.region, handle, sx, sy, x, y) => {
    let state = B.Paint.status(document.contents)
    let dx = floor(x -. sx +. 0.5) and dy = floor(y -. sy +. 0.5)
    if handle == Ui.Surface {
      region(
        clamp(0., state.width -. original.width, original.x +. dx),
        clamp(0., state.height -. original.height, original.y +. dy),
        original.width,
        original.height,
      )
    } else {
      let right = original.x +. original.width and bottom = original.y +. original.height
      let left = if handle == Ui.North_west || handle == Ui.South_west {
        clamp(0., right -. 1., original.x +. dx)
      } else {
        original.x
      }
      let top = if handle == Ui.North_west || handle == Ui.North_east {
        clamp(0., bottom -. 1., original.y +. dy)
      } else {
        original.y
      }
      let right = if handle == Ui.North_east || handle == Ui.South_east {
        clamp(left +. 1., state.width, right +. dx)
      } else {
        right
      }
      let bottom = if handle == Ui.South_west || handle == Ui.South_east {
        clamp(top +. 1., state.height, bottom +. dy)
      } else {
        bottom
      }
      region(left, top, right -. left, bottom -. top)
    }
  }

  let apply_pending = () =>
    switch pending.contents {
    | None => ()
    | Some(x, y) =>
      pending := None
      switch gesture.contents {
      | Drawing => B.Paint.move_to(document.contents, x, y)
      | Selecting(sx, sy, _) =>
        let state = B.Paint.status(document.contents)
        let x = clamp(0., state.width, floor(x)) and y = clamp(0., state.height, floor(y))
        selected := Some(region(min(sx, x), min(sy, y), abs_float(x -. sx), abs_float(y -. sy)))
      | Transforming(original, handle, sx, sy) =>
        let next = destination(original, handle, sx, sy, x, y)
        ignore(get(B.Paint.transform_to(document.contents, next)))
        selected := Some(next)
      | _ => ()
      }
    }

  let refresh = () => {
    dirty := true
    if !queued.contents {
      queued := true
      Ui.frame(() => {
        queued := false
        apply_pending()
        if dirty.contents {
          switch gesture.contents {
          | Selecting(_) =>
            dirty := false
            show_selection()
          | _ => render()
          }
        }
      })
    }
  }

  let stop = () => {
    apply_pending()
    switch gesture.contents {
    | Drawing | Transforming(_) => B.Paint.finish(document.contents)
    | Selecting(_) =>
      switch selected.contents {
      | Some(r) if r.width == 0. || r.height == 0. => selected := None
      | _ => ()
      }
    | _ => ()
    }
    gesture := Idle
    pending := None
  }

  let cancel = () => {
    pending := None
    switch gesture.contents {
    | Drawing => B.Paint.cancel(document.contents)
    | Transforming(original, _, _, _) =>
      B.Paint.cancel(document.contents)
      selected := Some(original)
    | Selecting(_, _, previous) => selected := previous
    | Panning(_, _, x, y, fitted) =>
      pan_x := x
      pan_y := y
      auto_fit := fitted
      show_view()
    | Pinching(_) | Idle => ()
    }
    gesture := Idle
    render()
  }

  let edit = action => {
    incr(version)
    stop()
    selected := None
    action(document.contents)
    render()
  }

  let select = (name, value) => {
    stop()
    mode := value
    switch value {
    | Draw(_) => selected := None
    | _ => ()
    }
    Ui.set_attribute(Ui.by_id("workspace"), "data-tool", name)
    List.iter(id =>
      Ui.set_attribute(
        Ui.by_id(id),
        "aria-pressed",
        if id == name {
          "true"
        } else {
          "false"
        },
      )
    , list{"hand", "select", "brush", "eraser", "fill", "line", "rectangle", "ellipse"})
    show_selection()
    message(
      switch value {
      | Hand => "Drag to pan. Pinch with two fingers or use + and - to zoom."
      | Select => "Drag a selection. Move it or drag a corner to resize."
      | _ => String.capitalize_ascii(name) ++ " selected"
      },
    )
  }

  List.iter(((name, value, key)) => {
    let button = Ui.by_id(name)
    Ui.on_click(button, () => select(name, value))
    Ui.shortcut(button, key)
  }, list{
    ("hand", Hand, "h"),
    ("select", Select, "v"),
    ("brush", Draw(B.Tool.Brush), "b"),
    ("eraser", Draw(B.Tool.Eraser), "e"),
    ("fill", Draw(B.Tool.Fill), "f"),
    ("line", Draw(B.Tool.Line), "l"),
    ("rectangle", Draw(B.Tool.Rectangle), "r"),
    ("ellipse", Draw(B.Tool.Ellipse), "o"),
  })
  List.iter(color =>
    Ui.on_click(Ui.by_id("color-" ++ color), () => {
      Ui.set_value(Ui.by_id("color"), "#" ++ color)
      message("Color #" ++ color)
    })
  , list{
    "20252b",
    "ffffff",
    "e75b52",
    "f2a541",
    "f4d35e",
    "78b887",
    "398c86",
    "5a98d0",
    "5865a8",
    "9276b5",
    "df94b5",
    "a4775b",
  })
  Ui.on_change(Ui.by_id("size"), () =>
    Ui.text(Ui.by_id("size-value"), Ui.value(Ui.by_id("size")) ++ " px")
  )
  Ui.on_pointer(
    Ui.by_id("workspace"),
    () => {
      stop()
      let x = Ui.pointer_x() and y = Ui.pointer_y()
      let state = B.Paint.status(document.contents)
      switch mode.contents {
      | Hand =>
        gesture :=
          Panning(
            Ui.pointer_screen_x(),
            Ui.pointer_screen_y(),
            pan_x.contents,
            pan_y.contents,
            auto_fit.contents,
          )
      | _
        if (x < 0. || (y < 0. || (x >= state.width || y >= state.height))) &&
          Ui.pointer_handle() == Ui.Surface => ()
      | Draw(tool) =>
        incr(version)
        gesture := Drawing
        let color = Ui.value(Ui.by_id("color"))
        let rgb = int_of_string("0x" ++ String.sub(color, 1, 6))
        B.Paint.begin_(
          document.contents,
          x,
          y,
          tool,
          float_of_int(rgb),
          float_of_string(Ui.value(Ui.by_id("size"))),
        )
        refresh()
      | Select =>
        let handle = Ui.pointer_handle()
        switch selected.contents {
        | Some(r)
          if handle != Ui.Surface ||
            (x >= r.x && (y >= r.y && (x <= r.x +. r.width && y <= r.y +. r.height))) =>
          incr(version)
          ignore(get(B.Paint.transform_start(document.contents, r)))
          gesture := Transforming(r, handle, x, y)
        | previous =>
          gesture := Selecting(floor(x), floor(y), previous)
          selected := None
          show_selection()
        }
      }
    },
    () =>
      switch gesture.contents {
      | Panning(sx, sy, px, py, _) =>
        auto_fit := false
        pan_x := px +. Ui.pointer_screen_x() -. sx
        pan_y := py +. Ui.pointer_screen_y() -. sy
        show_view()
      | Drawing =>
        switch mode.contents {
        | Draw(B.Tool.Brush | B.Tool.Eraser) =>
          B.Paint.move_to(document.contents, Ui.pointer_x(), Ui.pointer_y())
          refresh()
        | Draw(B.Tool.Fill) => ()
        | _ =>
          pending := Some(Ui.pointer_x(), Ui.pointer_y())
          refresh()
        }
      | Selecting(_) | Transforming(_) =>
        pending := Some(Ui.pointer_x(), Ui.pointer_y())
        refresh()
      | Pinching(_) | Idle => ()
      },
    () => {
      switch gesture.contents {
      | Drawing | Selecting(_) | Transforming(_) => pending := Some(Ui.pointer_x(), Ui.pointer_y())
      | _ => ()
      }
      stop()
      render()
    },
    () => {
      cancel()
      message("Gesture cancelled")
    },
    () => {
      switch gesture.contents {
      | Panning(_) => gesture := Idle
      | _ => cancel()
      }
      gesture :=
        Pinching(
          max(1., Ui.pinch_distance()),
          zoom.contents,
          (Ui.pinch_x() -. pan_x.contents) /. zoom.contents,
          (Ui.pinch_y() -. pan_y.contents) /. zoom.contents,
        )
    },
    () =>
      switch gesture.contents {
      | Pinching(distance, initial_zoom, anchor_x, anchor_y) =>
        auto_fit := false
        zoom := clamp(0.1, 8., initial_zoom *. Ui.pinch_distance() /. distance)
        pan_x := Ui.pinch_x() -. anchor_x *. zoom.contents
        pan_y := Ui.pinch_y() -. anchor_y *. zoom.contents
        show_view()
      | _ => ()
      },
    () => gesture := Idle,
  )
  let set_zoom = value => {
    stop()
    auto_fit := false
    zoom := clamp(0.1, 8., value)
    show_view()
  }
  Ui.on_click(Ui.by_id("zoom-in"), () => set_zoom(zoom.contents *. 1.25))
  Ui.on_click(Ui.by_id("zoom-out"), () => set_zoom(zoom.contents /. 1.25))
  Ui.on_click(Ui.by_id("fit"), () => {
    stop()
    fit()
  })
  Ui.on_resize(() =>
    if auto_fit.contents {
      stop()
      fit()
    }
  )
  Ui.on_wheel(Ui.by_id("workspace"), () =>
    set_zoom(zoom.contents *. exp(-.Ui.wheel_delta() /. 400.))
  )
  Ui.on_click(Ui.by_id("deselect"), () => {
    stop()
    selected := None
    show_selection()
  })
  let delete_selection = () => {
    stop()
    switch selected.contents {
    | None => ()
    | Some(r) =>
      incr(version)
      ignore(get(B.Paint.delete_region(document.contents, r)))
      selected := None
      render()
      message("Selection deleted - undo to restore")
    }
  }

  Ui.on_click(Ui.by_id("delete-selection"), delete_selection)
  Ui.on_click(Ui.by_id("resize-selection"), () => {
    stop()
    switch selected.contents {
    | None => ()
    | Some(r) =>
      try {
        let state = B.Paint.status(document.contents)
        let width = float_of_string(Ui.value(Ui.by_id("selection-width")))
        and height = float_of_string(Ui.value(Ui.by_id("selection-height")))
        if classify_float(width) == FP_nan || classify_float(height) == FP_nan {
          failwith("Invalid size")
        }
        let next = region(
          r.x,
          r.y,
          clamp(1., state.width -. r.x, floor(width)),
          clamp(1., state.height -. r.y, floor(height)),
        )
        incr(version)
        ignore(get(B.Paint.transform_start(document.contents, r)))
        ignore(get(B.Paint.transform_to(document.contents, next)))
        B.Paint.finish(document.contents)
        selected := Some(next)
        render()
        message("Selection resized")
      } catch {
      | Failure(_) => message("Enter a width and height in pixels")
      }
    }
  })
  Ui.on_key(() => {
    let key = Ui.key()
    if key == "escape" {
      cancel()
      selected := None
      show_selection()
      true
    } else if key == "delete" || key == "backspace" {
      switch selected.contents {
      | None => false
      | Some(_) =>
        delete_selection()
        true
      }
    } else if key == "mod+a" {
      select("select", Select)
      let state = B.Paint.status(document.contents)
      selected := Some(region(0., 0., state.width, state.height))
      show_selection()
      true
    } else {
      let shift = String.length(key) > 6 && String.sub(key, 0, 6) == "shift+"
      let plain = if shift {
        String.sub(key, 6, String.length(key) - 6)
      } else {
        key
      }
      let distance = if shift {
        10.
      } else {
        1.
      }
      let (dx, dy) = switch plain {
      | "arrowleft" => (-.distance, 0.)
      | "arrowright" => (distance, 0.)
      | "arrowup" => (0., -.distance)
      | "arrowdown" => (0., distance)
      | _ => (0., 0.)
      }
      if dx == 0. && dy == 0. {
        false
      } else {
        stop()
        switch selected.contents {
        | Some(r) =>
          incr(version)
          let next = destination(r, Ui.Surface, 0., 0., dx, dy)
          ignore(get(B.Paint.transform_start(document.contents, r)))
          ignore(get(B.Paint.transform_to(document.contents, next)))
          B.Paint.finish(document.contents)
          selected := Some(next)
          render()
          true
        | _ => false
        }
      }
    }
  })
  Ui.on_click(Ui.by_id("undo"), () => {
    edit(B.Paint.undo)
    message("Undone")
  })
  Ui.on_click(Ui.by_id("redo"), () => {
    edit(B.Paint.redo)
    message("Redone")
  })
  Ui.shortcut(Ui.by_id("undo"), "mod+z")
  Ui.shortcut(Ui.by_id("redo"), "mod+shift+z")
  Ui.shortcut(Ui.by_id("redo"), "mod+y")
  Ui.on_click(Ui.by_id("clear"), () => {
    edit(B.Paint.clear)
    message("Canvas cleared - undo to restore")
  })
  Ui.on_click(Ui.by_id("new"), () => {
    incr(version)
    pending := None
    gesture := Idle
    selected := None
    let next = get(B.Paint.create(768., 512.))
    B.Paint.dispose(document.contents)
    document := next
    Ui.text(Ui.by_id("filename"), "Untitled")
    render()
    fit()
    message("Fresh canvas, fresh ideas")
  })
  Ui.on_change(Ui.by_id("file"), () => {
    incr(version)
    let request = version.contents
    message("Opening PNG...")
    load_file(
      result =>
        switch result {
        | Ok(next) =>
          if request != version.contents {
            B.Paint.dispose(next)
          } else {
            pending := None
            gesture := Idle
            selected := None
            B.Paint.dispose(document.contents)
            document := next
            Ui.text(Ui.by_id("filename"), "Imported image")
            render()
            fit()
            message("PNG opened")
          }
        | Error(B.Image_error.Too_large) =>
          if request == version.contents {
            message("Choose a PNG up to 2048 x 2048 and 8 MB")
          }
        | Error(_) =>
          if request == version.contents {
            message("Could not open that PNG. Your drawing is safe.")
          }
        },
      () =>
        if request == version.contents {
          message("Could not read that file. Your drawing is safe.")
        },
    )
  })
  Ui.on_click(Ui.by_id("save"), () => {
    stop()
    switch B.Paint.png(document.contents) {
    | Ok(bytes) => {
        Ui.save_png(bytes)
        message("PNG saved - keep creating")
      }
    | Error(_) => message("Could not save the image. Please try again.")
    }
  })
  Ui.shortcut(Ui.by_id("save"), "mod+s")
  render()
  fit()
  message("Pick a color and make your mark")
}
