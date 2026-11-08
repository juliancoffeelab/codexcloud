type handle = Surface | North_west | North_east | South_west | South_east

let by_id = id =>
  switch Browser.find_id(Browser.document_of(Browser.global), id) {
  | Some(element) => element
  | None => failwith("Missing element " ++ id)
  }
let create = tag => Browser.create_element(Browser.document_of(Browser.global), tag)
let text = Browser.set_text
let append = Browser.append_child
let value = Browser.get_value
let set_value = Browser.set_value
let on_click = (element, callback) => Browser.listen_raw(element, "click", _ => callback())
let set_attribute = Browser.set_attribute
let button = (label, action) => {
  let element = create("button")
  text(element, label)
  on_click(element, action)
  element
}
let ready = () =>
  set_attribute(Browser.document_element(Browser.document_of(Browser.global)), "data-ready", "true")
let failed = reason => {
  let document = Browser.document_of(Browser.global)
  switch Browser.find_id(document, "loading") {
  | Some(loading) => Browser.set_hidden(loading, true)
  | None => ()
  }
  let error = create("pre")
  text(error, reason)
  append(Browser.body(document), error)
  append(Browser.body(document), button("Reload", Browser.reload))
}

let listen = (~passive=?, element, name, callback) =>
  switch passive {
  | None => Browser.listen_raw(element, name, callback)
  | Some(passive) =>
    Browser.listen_options(element, name, callback, Browser.listener_options(passive))
  }
let on_change = (element, callback) => listen(element, "input", _ => callback())
let on_resize = callback => listen(Browser.global, "resize", _ => callback())
let frame = callback => ignore(Browser.frame_raw(_ => callback()))
let disabled = Browser.set_disabled
let hidden = Browser.set_hidden
let pointer_x_value = ref(0.) and pointer_y_value = ref(0.)
let screen_x = ref(0.) and screen_y = ref(0.) and handle = ref(Surface)
let pinch_center_x = ref(0.) and pinch_center_y = ref(0.) and distance = ref(0.)
let wheel = ref(0.) and pressed_key = ref("")
let pointer_x = () => pointer_x_value.contents
let pointer_y = () => pointer_y_value.contents
let pointer_screen_x = () => screen_x.contents
let pointer_screen_y = () => screen_y.contents
let pointer_handle = () => handle.contents
let pinch_x = () => pinch_center_x.contents
let pinch_y = () => pinch_center_y.contents
let pinch_distance = () => distance.contents
let wheel_delta = () => wheel.contents
let key = () => pressed_key.contents
let closest = (element, selector) => Browser.closest(element, selector)
let number = value => {
  let result = string_of_float(value)
  if String.get(result, String.length(result) - 1) == '.' {
    result ++ "0"
  } else {
    result
  }
}

type contact = {id: int, mutable event: Browser.event}
let on_pointer = (element, down, move, up, cancel, pinch_start, pinch_move, pinch_end) => {
  let canvas = by_id("pixels")
  let active = ref(None) and contacts = ref(list{})
  let multitouch = ref(false) and pinching = ref(false)
  let pinch_frame = ref(None) and samples = ref(None)
  let measure = (ax, ay, bx, by) => {
    let box = Browser.bounds(element)
    pinch_center_x := (ax +. bx) /. 2. -. Browser.left(box) -. Browser.rect_width(box) /. 2.
    pinch_center_y := (ay +. by) /. 2. -. Browser.top(box) -. Browser.rect_height(box) /. 2.
    let dx = ax -. bx and dy = ay -. by
    distance := sqrt(dx *. dx +. dy *. dy)
  }

  let flush = () => {
    switch pinch_frame.contents {
    | Some(id) => Browser.cancel_frame(id)
    | None => ()
    }
    pinch_frame := None
    switch samples.contents {
    | Some(ax, ay, bx, by) if pinching.contents =>
      samples := None
      measure(ax, ay, bx, by)
      pinch_move()
    | _ => ()
    }
  }

  listen(~passive=true, element, "touchmove", event => {
    let touches = Browser.touches(event)
    if pinching.contents && Browser.touches_length(touches) == 2 {
      let a = Browser.touch_at(touches, 0) and b = Browser.touch_at(touches, 1)
      samples :=
        Some(Browser.touch_x(a), Browser.touch_y(a), Browser.touch_x(b), Browser.touch_y(b))
      if pinch_frame.contents == None {
        pinch_frame := Some(Browser.frame_raw(_ => flush()))
      }
    }
  })
  let locate = event => {
    let box = Browser.bounds(canvas)
    screen_x := Browser.client_x(event)
    screen_y := Browser.client_y(event)
    pointer_x_value :=
      (screen_x.contents -. Browser.left(box)) *. Browser.width(canvas) /. Browser.rect_width(box)
    pointer_y_value :=
      (screen_y.contents -. Browser.top(box)) *. Browser.height(canvas) /. Browser.rect_height(box)
  }

  listen(element, "pointerdown", event => {
    let target = Browser.target(event)
    if (
      Browser.button(event) == 0 &&
        (closest(target, "button, input") == None || closest(target, "[data-handle]") != None)
    ) {
      let id = Browser.pointer_id(event)
      let touch = Browser.pointer_type_raw(event) == "touch"
      if !touch || List.length(contacts.contents) < 2 {
        if touch {
          contacts := \"@"(contacts.contents, list{{id, event}})
          Browser.capture(element, id)
        }
        switch contacts.contents {
        | list{a, b} if touch =>
          active := None
          multitouch := true
          pinching := true
          measure(
            Browser.client_x(a.event),
            Browser.client_y(a.event),
            Browser.client_x(b.event),
            Browser.client_y(b.event),
          )
          pinch_start()
          Browser.prevent_default(event)
        | _ if active.contents == None && !multitouch.contents =>
          active := Some(id)
          handle :=
            switch closest(target, "[data-handle]") {
            | None => Surface
            | Some(node) =>
              switch Browser.get_attribute(node, "data-handle") {
              | None => Surface
              | Some(value) =>
                switch value {
                | "nw" => North_west
                | "ne" => North_east
                | "sw" => South_west
                | "se" => South_east
                | _ => Surface
                }
              }
            }
          Browser.capture(element, id)
          locate(event)
          down()
          Browser.prevent_default(event)
        | _ => ()
        }
      }
    }
  })
  listen(element, "pointermove", event => {
    let id = Browser.pointer_id(event)
    List.iter(contact =>
      if contact.id == id {
        contact.event = event
      }
    , contacts.contents)
    if !multitouch.contents && active.contents == Some(id) {
      switch Browser.coalesced(event) {
      | Some(events) if Browser.events_length(events) > 0 =>
        for i in 0 to Browser.events_length(events) - 1 {
          locate(Browser.event_at(events, i))
          move()
        }
      | _ =>
        locate(event)
        move()
      }
    }
  })
  let end_touch = (event, cancelled) => {
    let id = Browser.pointer_id(event)
    if !List.exists(contact => contact.id == id, contacts.contents) {
      false
    } else {
      if multitouch.contents && pinching.contents {
        if !cancelled {
          flush()
        }
        switch pinch_frame.contents {
        | Some(id) => Browser.cancel_frame(id)
        | None => ()
        }
        pinch_frame := None
        samples := None
        pinching := false
        pinch_end()
      }
      contacts := List.filter(contact => contact.id != id, contacts.contents)
      if !multitouch.contents {
        false
      } else {
        if contacts.contents == list{} {
          multitouch := false
        }
        true
      }
    }
  }

  listen(element, "pointerup", event =>
    if !end_touch(event, false) && active.contents == Some(Browser.pointer_id(event)) {
      active := None
      locate(event)
      up()
    }
  )
  let cancelled = event =>
    if !end_touch(event, true) && active.contents == Some(Browser.pointer_id(event)) {
      active := None
      cancel()
    }

  listen(element, "pointercancel", cancelled)
  listen(element, "lostpointercapture", cancelled)
}
let set_style = (element, key, value) => Browser.set_style(Browser.style(element), key, value)
let view = (x, y, zoom) => {
  let canvas = by_id("canvas-wrap")
  set_style(canvas, "--view-zoom", number(zoom))
  set_style(
    canvas,
    "transform",
    "translate(-50%, -50%) translate(" ++
    (number(x) ++
    ("px, " ++ (number(y) ++ ("px) scale(" ++ (number(zoom) ++ ")"))))),
  )
}
let fit_scale = () => {
  let workspace = by_id("workspace") and canvas = by_id("pixels")
  min(
    1.,
    min(
      (Browser.client_width(workspace) -. 40.) /. Browser.width(canvas),
      (Browser.client_height(workspace) -. 70.) /. Browser.height(canvas),
    ),
  )
}
let selection = (x, y, width, height) => {
  let canvas = by_id("pixels") and box = by_id("selection")
  hidden(box, width <= 0. || height <= 0.)
  List.iter(
    ((name, value, total)) => set_style(box, name, number(100. *. value /. total) ++ "%"),
    list{
      ("left", x, Browser.width(canvas)),
      ("top", y, Browser.height(canvas)),
      ("width", width, Browser.width(canvas)),
      ("height", height, Browser.height(canvas)),
    },
  )
}
let on_wheel = (element, callback) =>
  listen(~passive=false, element, "wheel", event => {
    wheel :=
      Browser.delta_y(event) *.
      switch Browser.delta_mode(event) {
      | 1 => 16.
      | 2 => Browser.client_height(element)
      | _ => 1.
      }
    Browser.prevent_default(event)
    callback()
  })
let editing = event =>
  closest(Browser.target(event), "input, textarea, select, [contenteditable=true]") != None
let combination = event =>
  if Browser.ctrl(event) || Browser.meta(event) {
    "mod+"
  } else {
    ""
  } ++
  (if Browser.shift(event) {
    "shift+"
  } else {
    ""
  } ++
  String.lowercase_ascii(Browser.key_raw(event)))
let on_key = callback =>
  listen(Browser.document_of(Browser.global), "keydown", event =>
    if !editing(event) {
      pressed_key := combination(event)
      if callback() {
        Browser.prevent_default(event)
      }
    }
  )
let shortcut = (button, key) =>
  listen(Browser.document_of(Browser.global), "keydown", event =>
    if !editing(event) && combination(event) == key {
      Browser.prevent_default(event)
      if !Browser.disabled(button) {
        Browser.click(button)
      }
    }
  )
let save_png = bytes => Browser.save_png(bytes)
