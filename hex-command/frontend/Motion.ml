type element
type classes
type media
type timer
type record = { class_name : string; mutable timer : timer option }
type weak_map
external make_map : unit -> weak_map = "WeakMap" [@@mel.new]
external get : weak_map -> element -> record Js.Undefined.t = "get" [@@mel.send]
external set : weak_map -> element -> record -> unit = "set" [@@mel.send]
external remove : weak_map -> element -> unit = "delete" [@@mel.send]
external classes : element -> classes = "classList" [@@mel.get]
external add_class : classes -> string -> unit = "add" [@@mel.send]
external remove_class : classes -> string -> unit = "remove" [@@mel.send]
external bounds : element -> unit = "getBoundingClientRect" [@@mel.send]
external delay : (unit -> unit) -> int -> timer = "setTimeout"
external clear : timer -> unit = "clearTimeout"
external document : unit Js.Undefined.t = "document" [@@mel.scope "globalThis"]
external media_function : unit Js.Undefined.t = "matchMedia" [@@mel.scope "globalThis"]
external match_media : string -> media = "matchMedia"
external matches : media -> bool = "matches" [@@mel.get]
external query : string -> element Js.Nullable.t = "querySelector" [@@mel.scope "document"]
external query_all : string -> element array = "querySelectorAll" [@@mel.scope "document"]
external attribute : element -> string -> string Js.Nullable.t = "getAttribute" [@@mel.send]
external child : element -> string -> element Js.Nullable.t = "querySelector" [@@mel.send]
let active = make_map ()
let reduced_motion () =
  Js.Undefined.toOption media_function <> None && matches (match_media "(prefers-reduced-motion: reduce)")
let pulse element kind =
  if not (reduced_motion ()) then (
    Option.iter (fun previous ->
        Option.iter clear previous.timer;
        remove_class (classes element) previous.class_name)
      (Js.Undefined.toOption (get active element));
    let class_name = "motion-" ^ kind in
    remove_class (classes element) class_name;
    bounds element;
    add_class (classes element) class_name;
    let record = { class_name; timer = None } in
    record.timer <- Some (delay (fun () ->
        remove_class (classes element) class_name;
        match Js.Undefined.toOption (get active element) with
        | Some current when current == record -> remove active element
        | _ -> ()) 260);
    set active element record)
let animate_tile tile_id kind =
  if Js.Undefined.toOption document <> None &&
     List.mem kind ["select"; "move"; "attack"; "recruit"; "capture"] then
    let attribute_name = match kind with
      | "select" -> "data-tile" | "capture" -> "data-indicator-tile"
      | _ -> "data-unit-tile" in
    let nodes = query_all ("#map [" ^ attribute_name ^ "]") in
    let rec find index =
      if index < Array.length nodes then
        let node = nodes.(index) in
        if Js.Nullable.toOption (attribute node attribute_name) = Some tile_id then (
          if not (List.mem kind ["move"; "attack"; "recruit"]) ||
             Js.Nullable.toOption (child node ".unit-art") <> None then pulse node kind)
        else find (index + 1) in
    find 0
let animate_turn () =
  if Js.Undefined.toOption document <> None then
    Option.iter (fun element -> pulse element "turn")
      (Js.Nullable.toOption (query "#end-turn"))
