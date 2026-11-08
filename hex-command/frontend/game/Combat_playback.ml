type frame = {
  before : Native.view;
  combat : Domain.combat_animation;
  shown : int;
  complete : bool;
  game : Native.view;
  advance : int -> bool;
  finish : unit -> unit;
  skip : unit -> unit;
}

type t = {
  mutable frame : frame;
  mutable cancelled : bool;
  before_game : Native.view;
  resolved : Native.view;
  publish : frame -> unit;
}

let advance t index =
  let frame = t.frame in
  if
    t.cancelled || frame.complete || index <> frame.shown
    || frame.shown >= Array.length frame.combat.strikes
  then false
  else
    let hit = frame.combat.strikes.(frame.shown) in
    let units =
      Array_ext.filter_map
        (fun (unit : Domain.public_unit) ->
          let index = ref (-1) in
          Array.iteri
            (fun i (actor : Domain.public_unit) -> if actor.id = unit.id then index := i)
            frame.combat.units;
          if !index < 0 then Some unit
          else
            let hp = hit.health.(!index) in
            if hp > 0l then Some { unit with hp } else None)
        (Native.state t.before_game).units
    in
    let game = Native.project_units t.before_game units in
    t.frame <- { frame with shown = frame.shown + 1; game };
    t.publish t.frame;
    true

let complete t skip =
  let frame = t.frame in
  if
    (not t.cancelled) && (not frame.complete)
    && (skip || frame.shown = Array.length frame.combat.strikes)
  then (
    t.frame <-
      { frame with shown = Array.length frame.combat.strikes; complete = true; game = t.resolved };
    t.publish t.frame)

let create ~before_game combat resolved publish =
  let reference = ref None in
  let current () =
    match !reference with
    | Some current -> current
    | None -> failwith "Combat playback not initialized"
  in
  let frame =
    {
      before = before_game;
      combat;
      shown = 0;
      complete = false;
      game = before_game;
      advance = (fun index -> advance (current ()) index);
      finish = (fun () -> complete (current ()) false);
      skip = (fun () -> complete (current ()) true);
    }
  in
  let frame =
    if Array.length combat.strikes = 0 then
      { frame with shown = Array.length combat.strikes; complete = true; game = resolved }
    else frame
  in
  let t = { frame; cancelled = false; before_game; resolved; publish } in
  reference := Some t;
  t

let frame t = t.frame
let cancel t = t.cancelled <- true
