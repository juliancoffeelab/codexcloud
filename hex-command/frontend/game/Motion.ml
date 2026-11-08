open Browser_dom.Element_motion

type record = { class_name : string; mutable timer : timer option }

module Weak_map = Host_collections.Weak_map

let active : (Dom.element, record) Weak_map.t = Weak_map.create ()

let reduced_motion () =
  Js.Undefined.toOption media_function <> None
  && matches (match_media "(prefers-reduced-motion: reduce)")

let pulse element kind =
  if not (reduced_motion ()) then (
    let class_name = "motion-" ^ kind in
    let previous = Weak_map.get active element in
    let restarting =
      Option.fold ~none:false ~some:(fun record -> record.class_name = class_name) previous
    in
    Option.iter
      (fun previous ->
        Option.iter clear previous.timer;
        remove_class (classes element) previous.class_name)
      previous;
    if restarting then bounds element;
    add_class (classes element) class_name;
    let record = { class_name; timer = None } in
    record.timer <-
      Some
        (delay
           (fun () ->
             remove_class (classes element) class_name;
             match Weak_map.get active element with
             | Some current when current == record ->
                 let _ = Weak_map.delete active element in
                 ()
             | _ -> ())
           260);
    let _ = Weak_map.set active element record in
    ())

type tile_event = { tile : string; kind : string; sound : Phaser_sound.interaction option }

module Event = Browser_dom.Event (struct
  type t = tile_event
end)

type event = Event.event

let detail = Event.detail
let listen node receive = Event.on node "battlefieldmotion" receive
let unlisten node receive = Event.off node "battlefieldmotion" receive

let animate_tile ?sound tile kind =
  if
    Js.Undefined.toOption document <> None
    && List.mem kind [ "select"; "move"; "attack"; "recruit"; "capture" ]
  then
    Option.iter
      (fun node ->
        let _ =
          Event.dispatch node
            (Event.create "battlefieldmotion" [%mel.obj { detail = { tile; kind; sound } }])
        in
        ())
      (Js.Nullable.toOption (query "#map[data-phaser-battlefield]"))

let animate_turn () =
  if Js.Undefined.toOption document <> None then
    Option.iter (fun element -> pulse element "turn") (Js.Nullable.toOption (query "#end-turn"))

let subscribe_reduced receive =
  let preference = match_media "(prefers-reduced-motion: reduce)" in
  let update () = receive (matches preference) in
  update ();
  media_on preference "change" update;
  fun () -> media_off preference "change" update
