type handle = Surface | North_west | North_east | South_west | South_east

let by_id id =
  match
    Browser.find_id (Browser.document_of Browser.global) (Browser.to_host id)
  with
  | Some element -> element
  | None -> failwith ("Missing element " ^ id)

let create tag =
  Browser.create_element
    (Browser.document_of Browser.global)
    (Browser.to_host tag)

let text element value = Browser.set_text element (Browser.to_host value)
let append = Browser.append_child
let value element = Browser.of_host (Browser.get_value element)
let set_value element value = Browser.set_value element (Browser.to_host value)

let on_click element callback =
  Browser.listen_raw element (Browser.to_host "click") (fun _ -> callback ())

let set_attribute element name value =
  Browser.set_attribute element (Browser.to_host name) (Browser.to_host value)

let button label action =
  let element = create "button" in
  text element label;
  on_click element action;
  element

let ready () =
  set_attribute
    (Browser.document_element (Browser.document_of Browser.global))
    "data-ready" "true"

let failed reason =
  let document = Browser.document_of Browser.global in
  (match Browser.find_id document (Browser.to_host "loading") with
  | Some loading -> Browser.set_hidden loading true
  | None -> ());
  let error = create "pre" in
  text error reason;
  append (Browser.body document) error;
  append (Browser.body document) (button "Reload" Browser.reload)

let listen ?passive element name callback =
  match passive with
  | None -> Browser.listen_raw element (Browser.to_host name) callback
  | Some passive ->
      Browser.listen_options element (Browser.to_host name) callback
        (Browser.listener_options passive)

let on_change element callback = listen element "input" (fun _ -> callback ())
let on_resize callback = listen Browser.global "resize" (fun _ -> callback ())

let frame callback =
  let _ = Browser.frame_raw (fun _ -> callback ()) in
  ()

let disabled = Browser.set_disabled
let hidden = Browser.set_hidden

let pointer_x_value = ref 0.
and pointer_y_value = ref 0.

let screen_x = ref 0.
and screen_y = ref 0.
and handle = ref Surface

let pinch_center_x = ref 0.
and pinch_center_y = ref 0.
and distance = ref 0.

let wheel = ref 0.
and pressed_key = ref ""

let pointer_x () = !pointer_x_value
let pointer_y () = !pointer_y_value
let pointer_screen_x () = !screen_x
let pointer_screen_y () = !screen_y
let pointer_handle () = !handle
let pinch_x () = !pinch_center_x
let pinch_y () = !pinch_center_y
let pinch_distance () = !distance
let wheel_delta () = !wheel
let key () = !pressed_key

let closest element selector =
  Browser.closest element (Browser.to_host selector)

let number value =
  let result = string_of_float value in
  if result.[String.length result - 1] = '.' then result ^ "0" else result

type contact = { id : int; mutable event : Browser.event }

let on_pointer element down move up cancel pinch_start pinch_move pinch_end =
  let canvas = by_id "pixels" in
  let active = ref None and contacts = ref [] in
  let multitouch = ref false and pinching = ref false in
  let pinch_frame = ref None and samples = ref None in
  let measure ax ay bx by =
    let box = Browser.bounds element in
    pinch_center_x :=
      ((ax +. bx) /. 2.) -. Browser.left box -. (Browser.rect_width box /. 2.);
    pinch_center_y :=
      ((ay +. by) /. 2.) -. Browser.top box -. (Browser.rect_height box /. 2.);
    let dx = ax -. bx and dy = ay -. by in
    distance := sqrt ((dx *. dx) +. (dy *. dy))
  in
  let flush () =
    (match !pinch_frame with Some id -> Browser.cancel_frame id | None -> ());
    pinch_frame := None;
    match !samples with
    | Some (ax, ay, bx, by) when !pinching ->
        samples := None;
        measure ax ay bx by;
        pinch_move ()
    | _ -> ()
  in
  listen ~passive:true element "touchmove" (fun event ->
      let touches = Browser.touches event in
      if !pinching && Browser.touches_length touches = 2 then (
        let a = Browser.touch_at touches 0 and b = Browser.touch_at touches 1 in
        samples :=
          Some
            ( Browser.touch_x a,
              Browser.touch_y a,
              Browser.touch_x b,
              Browser.touch_y b );
        if !pinch_frame = None then
          pinch_frame := Some (Browser.frame_raw (fun _ -> flush ()))));
  let locate event =
    let box = Browser.bounds canvas in
    screen_x := Browser.client_x event;
    screen_y := Browser.client_y event;
    pointer_x_value :=
      (!screen_x -. Browser.left box)
      *. Browser.width canvas /. Browser.rect_width box;
    pointer_y_value :=
      (!screen_y -. Browser.top box)
      *. Browser.height canvas /. Browser.rect_height box
  in
  listen element "pointerdown" (fun event ->
      let target = Browser.target event in
      if
        Browser.button event = 0
        && (closest target "button, input" = None
           || closest target "[data-handle]" <> None)
      then
        let id = Browser.pointer_id event in
        let touch =
          Browser.of_host (Browser.pointer_type_raw event) = "touch"
        in
        if (not touch) || List.length !contacts < 2 then (
          if touch then (
            contacts := !contacts @ [ { id; event } ];
            Browser.capture element id);
          match !contacts with
          | [ a; b ] when touch ->
              active := None;
              multitouch := true;
              pinching := true;
              measure (Browser.client_x a.event) (Browser.client_y a.event)
                (Browser.client_x b.event) (Browser.client_y b.event);
              pinch_start ();
              Browser.prevent_default event
          | _ when !active = None && not !multitouch ->
              active := Some id;
              (handle :=
                 match closest target "[data-handle]" with
                 | None -> Surface
                 | Some node -> (
                     match
                       Browser.get_attribute node
                         (Browser.to_host "data-handle")
                     with
                     | None -> Surface
                     | Some value -> (
                         match Browser.of_host value with
                         | "nw" -> North_west
                         | "ne" -> North_east
                         | "sw" -> South_west
                         | "se" -> South_east
                         | _ -> Surface)));
              Browser.capture element id;
              locate event;
              down ();
              Browser.prevent_default event
          | _ -> ()));
  listen element "pointermove" (fun event ->
      let id = Browser.pointer_id event in
      List.iter
        (fun contact -> if contact.id = id then contact.event <- event)
        !contacts;
      if (not !multitouch) && !active = Some id then
        match Browser.coalesced event with
        | Some events when Browser.events_length events > 0 ->
            for i = 0 to Browser.events_length events - 1 do
              locate (Browser.event_at events i);
              move ()
            done
        | _ ->
            locate event;
            move ());
  let end_touch event cancelled =
    let id = Browser.pointer_id event in
    if not (List.exists (fun contact -> contact.id = id) !contacts) then false
    else (
      if !multitouch && !pinching then (
        if not cancelled then flush ();
        (match !pinch_frame with
        | Some id -> Browser.cancel_frame id
        | None -> ());
        pinch_frame := None;
        samples := None;
        pinching := false;
        pinch_end ());
      contacts := List.filter (fun contact -> contact.id <> id) !contacts;
      if not !multitouch then false
      else (
        if !contacts = [] then multitouch := false;
        true))
  in
  listen element "pointerup" (fun event ->
      if
        (not (end_touch event false))
        && !active = Some (Browser.pointer_id event)
      then (
        active := None;
        locate event;
        up ()));
  let cancelled event =
    if (not (end_touch event true)) && !active = Some (Browser.pointer_id event)
    then (
      active := None;
      cancel ())
  in
  listen element "pointercancel" cancelled;
  listen element "lostpointercapture" cancelled

let set_style element key value =
  Browser.set_style (Browser.style element) (Browser.to_host key)
    (Browser.to_host value)

let view x y zoom =
  let canvas = by_id "canvas-wrap" in
  set_style canvas "--view-zoom" (number zoom);
  set_style canvas "transform"
    ("translate(-50%, -50%) translate(" ^ number x ^ "px, " ^ number y
   ^ "px) scale(" ^ number zoom ^ ")")

let fit_scale () =
  let workspace = by_id "workspace" and canvas = by_id "pixels" in
  min 1.
    (min
       ((Browser.client_width workspace -. 40.) /. Browser.width canvas)
       ((Browser.client_height workspace -. 70.) /. Browser.height canvas))

let selection x y width height =
  let canvas = by_id "pixels" and box = by_id "selection" in
  hidden box (width <= 0. || height <= 0.);
  List.iter
    (fun (name, value, total) ->
      set_style box name (number (100. *. value /. total) ^ "%"))
    [
      ("left", x, Browser.width canvas);
      ("top", y, Browser.height canvas);
      ("width", width, Browser.width canvas);
      ("height", height, Browser.height canvas);
    ]

let on_wheel element callback =
  listen ~passive:false element "wheel" (fun event ->
      (wheel :=
         Browser.delta_y event
         *.
         match Browser.delta_mode event with
         | 1 -> 16.
         | 2 -> Browser.client_height element
         | _ -> 1.);
      Browser.prevent_default event;
      callback ())

let editing event =
  closest (Browser.target event)
    "input, textarea, select, [contenteditable=true]"
  <> None

let combination event =
  (if Browser.ctrl event || Browser.meta event then "mod+" else "")
  ^ (if Browser.shift event then "shift+" else "")
  ^ String.lowercase_ascii (Browser.of_host (Browser.key_raw event))

let on_key callback =
  listen (Browser.document_of Browser.global) "keydown" (fun event ->
      if not (editing event) then (
        pressed_key := combination event;
        if callback () then Browser.prevent_default event))

let shortcut button key =
  listen (Browser.document_of Browser.global) "keydown" (fun event ->
      if (not (editing event)) && combination event = key then (
        Browser.prevent_default event;
        if not (Browser.disabled button) then Browser.click button))

let save_png bytes = Browser.save_png bytes
