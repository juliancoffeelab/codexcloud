open Browser_dom.Element_motion

type record = { class_name : string; mutable timer : timer option }

module Weak_map = Host_collections.Weak_map

let active : (Dom.element, record) Weak_map.t = Weak_map.create ()

let reduced_motion () =
  Js.Undefined.toOption media_function <> None
  && matches (match_media {js|(prefers-reduced-motion: reduce)|js})

let pulse element kind =
  if not (reduced_motion ()) then (
    let class_name = {j|motion-$(kind)|j} in
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
let listen node receive = Event.on node {js|battlefieldmotion|js} receive
let unlisten node receive = Event.off node {js|battlefieldmotion|js} receive

let animate_tile ?sound tile kind =
  if
    Js.Undefined.toOption document <> None
    && List.mem kind
         [ {js|select|js}; {js|move|js}; {js|attack|js}; {js|recruit|js}; {js|capture|js} ]
  then
    Option.iter
      (fun node ->
        let _ =
          Event.dispatch node
            (Event.create {js|battlefieldmotion|js} [%mel.obj { detail = { tile; kind; sound } }])
        in
        ())
      (Js.Nullable.toOption (query {js|#map[data-phaser-battlefield]|js}))

let animate_turn () =
  if Js.Undefined.toOption document <> None then
    Option.iter
      (fun element -> pulse element {js|turn|js})
      (Js.Nullable.toOption (query {js|#end-turn|js}))

let subscribe_reduced receive =
  let preference = match_media {js|(prefers-reduced-motion: reduce)|js} in
  let update () = receive (matches preference) in
  update ();
  media_on preference {js|change|js} update;
  fun () -> media_off preference {js|change|js} update
