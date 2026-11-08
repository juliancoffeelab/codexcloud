type rect = { left : float; right : float; top : float; bottom : float }

type extent = {
  width : float;
  height : float;
  scroll_width : float;
  scroll_height : float;
  scroll_left : float;
  scroll_top : float;
}

type scroll = { left : float; top : float }

external finite : float -> bool = "isFinite" [@@mel.scope "Number"]

let valid (rect : rect) =
  finite rect.left && finite rect.right && finite rect.top && finite rect.bottom
  && rect.right >= rect.left && rect.bottom >= rect.top

let position ~(area : rect) ~(viewport : extent) bounds =
  let unchanged = { left = viewport.scroll_left; top = viewport.scroll_top } in
  let bounds = Array.to_list bounds |> List.filter valid |> Array.of_list in
  if
    Array.length bounds = 0
    || (not (valid area))
    || viewport.width <= 0. || viewport.height <= 0.
    || not
         (List.for_all finite
            [
              viewport.width;
              viewport.height;
              viewport.scroll_width;
              viewport.scroll_height;
              viewport.scroll_left;
              viewport.scroll_top;
            ])
  then unchanged
  else
    let last = bounds.(Array.length bounds - 1) in
    let union =
      Array.fold_left
        (fun (union : rect) (rect : rect) ->
          {
            left = min union.left rect.left;
            right = max union.right rect.right;
            top = min union.top rect.top;
            bottom = max union.bottom rect.bottom;
          })
        last bounds
    in
    let x1, x2 =
      if union.right -. union.left +. 24. <= viewport.width then (union.left, union.right)
      else (last.left, last.right)
    and y1, y2 =
      if union.bottom -. union.top +. 24. <= viewport.height then (union.top, union.bottom)
      else (last.top, last.bottom)
    in
    let delta start finish origin size =
      let before = min 0. (start -. origin -. 12.) in
      if before <> 0. then before else max 0. (finish -. origin -. size +. 12.)
    in
    let clamp value limit = max 0. (min value (max 0. limit)) in
    {
      left =
        clamp
          (viewport.scroll_left +. delta x1 x2 area.left viewport.width)
          (viewport.scroll_width -. viewport.width);
      top =
        clamp
          (viewport.scroll_top +. delta y1 y2 area.top viewport.height)
          (viewport.scroll_height -. viewport.height);
    }

external bounds : Dom.element -> rect = "getBoundingClientRect" [@@mel.send]
external nodes : Dom.element -> string -> Dom.element array = "querySelectorAll" [@@mel.send]
external attribute : Dom.element -> string -> string Js.Nullable.t = "getAttribute" [@@mel.send]
external child : Dom.element -> string -> Dom.element Js.Nullable.t = "querySelector" [@@mel.send]
external width : Dom.element -> float = "clientWidth" [@@mel.get]
external height : Dom.element -> float = "clientHeight" [@@mel.get]
external scroll_width : Dom.element -> float = "scrollWidth" [@@mel.get]
external scroll_height : Dom.element -> float = "scrollHeight" [@@mel.get]
external scroll_left : Dom.element -> float = "scrollLeft" [@@mel.get]
external scroll_top : Dom.element -> float = "scrollTop" [@@mel.get]
external set_left : Dom.element -> float -> unit = "scrollLeft" [@@mel.set]
external set_top : Dom.element -> float -> unit = "scrollTop" [@@mel.set]

type observer

external observer_function : unit Js.Undefined.t = "ResizeObserver" [@@mel.scope "globalThis"]
external make_observer : (unit -> unit) -> observer = "ResizeObserver" [@@mel.new]
external observe : observer -> Dom.element -> unit = "observe" [@@mel.send]
external disconnect : observer -> unit = "disconnect" [@@mel.send]

let settled task =
  let _ =
    Js.Promise.then_
      (fun () ->
        task ();
        Js.Promise.resolve ())
      (Js.Promise.resolve ())
  in
  ()

let reveal_map ~scroller ~board ~ids =
  let current = ref true and seen = Js.Dict.empty () in
  let ids =
    Array.to_list ids |> List.rev
    |> List.filter (fun id ->
        if id = "" || Js.Dict.get seen id <> None then false
        else (
          Js.Dict.set seen id true;
          true))
    |> List.rev
  in
  let reveal () =
    if !current then (
      let measurements = Js.Dict.empty () in
      nodes board "[data-tile], [data-unit-tile]"
      |> Array.iter (fun node ->
          let tile = Js.Nullable.toOption (attribute node "data-tile")
          and unit_tile = Js.Nullable.toOption (attribute node "data-unit-tile") in
          let add = function
            | Some id when Js.Dict.get seen id <> None ->
                let existing = Option.value (Js.Dict.get measurements id) ~default:[] in
                Js.Dict.set measurements id (bounds node :: existing)
            | _ -> ()
          in
          add tile;
          if tile <> unit_tile then add unit_tile);
      let rectangles =
        ids
        |> List.concat_map (fun id ->
            Option.value (Js.Dict.get measurements id) ~default:[] |> List.rev)
        |> Array.of_list
      in
      let viewport =
        {
          width = width scroller;
          height = height scroller;
          scroll_width = scroll_width scroller;
          scroll_height = scroll_height scroller;
          scroll_left = scroll_left scroller;
          scroll_top = scroll_top scroller;
        }
      in
      let target = position ~area:(bounds scroller) ~viewport rectangles in
      if target.left <> viewport.scroll_left then set_left scroller target.left;
      if target.top <> viewport.scroll_top then set_top scroller target.top)
  in
  if ids = [] then fun () -> current := false
  else (
    settled reveal;
    let observer =
      if Js.Undefined.toOption observer_function = None then None
      else
        try
          let observer = make_observer reveal in
          observe observer scroller;
          Some observer
        with _ -> None
    in
    fun () ->
      if !current then (
        current := false;
        Option.iter disconnect observer))

type media
type scroll_options = { block : string }

external media_function : unit Js.Undefined.t = "matchMedia" [@@mel.scope "globalThis"]
external match_media : string -> media = "matchMedia"
external matches : media -> bool = "matches" [@@mel.get]
external scroll_function : Dom.element -> unit Js.Undefined.t = "scrollIntoView" [@@mel.get]
external scroll_into_view : Dom.element -> scroll_options -> unit = "scrollIntoView" [@@mel.send]

let reveal_attack ~panel =
  let current = ref true in
  settled (fun () ->
      if
        !current
        && Js.Undefined.toOption media_function <> None
        && matches (match_media "(max-width:1000px)")
      then
        match Js.Nullable.toOption (child panel ".attack-actions") with
        | Some node when Js.Undefined.toOption (scroll_function node) <> None ->
            scroll_into_view node { block = "nearest" }
        | _ -> ());
  fun () -> current := false
