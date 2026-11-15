module P = Phaser_pointer

type view = { left : float; top : float; zoom : float; canvas_left : float; canvas_top : float }

type command =
  | Pan of float * float
  | Zoom of float * float * float
  | Tap of P.scene_position
  | Coast of float * float

type phase = Idle | Pressed | Panning | Pinching | Suppressed

type release_reason =
  | Activate
  | No_press
  | Wrong_contact
  | Cancelled
  | Dragged
  | Pan_only
  | Too_far
  | Contact_suppressed

type track = {
  press : P.press;
  view : view;
  mutable origin : P.client_position;
  mutable last : P.sample;
  mutable vx : float;
  mutable vy : float;
}

type pinch = { a : int; b : int; distance : float; zoom : float; x : float; y : float }

type state =
  | Rest
  | Pending of track
  | Moving of track
  | Scaling of pinch
  | Awaiting_pinch of int list
  | Blocked of int list

type t = { mutable state : state }

type tracking = {
  pointer_id : int;
  left : float;
  top : float;
  x : float;
  y : float;
  vx : float;
  vy : float;
  time : float;
}

let create () = { state = Rest }

let phase t =
  match t.state with
  | Rest -> Idle
  | Pending _ -> Pressed
  | Moving _ -> Panning
  | Scaling _ | Awaiting_pinch _ -> Pinching
  | Blocked _ -> Suppressed

let distance (a : P.client_position) (b : P.client_position) =
  hypot (a.client_x -. b.client_x) (a.client_y -. b.client_y)

let ids active = Array.to_list (Array.map (fun (s : P.sample) -> s.id) active)
let blocked active = match ids active with [] -> Rest | ids -> Blocked ids
let find active id = Array.find_opt (fun (s : P.sample) -> s.id = id) active

let tracking t =
  match t.state with
  | Pending d | Moving d ->
      Some
        {
          pointer_id = d.press.sample.id;
          left = d.view.left -. (d.last.client.client_x -. d.origin.client_x);
          top = d.view.top -. (d.last.client.client_y -. d.origin.client_y);
          x = d.last.client.client_x;
          y = d.last.client.client_y;
          vx = d.vx;
          vy = d.vy;
          time = d.last.time;
        }
  | _ -> None

let press t (press : P.press) view ~active =
  let start () =
    t.state <-
      Pending { press; view; origin = press.sample.client; last = press.sample; vx = 0.; vy = 0. };
    true
  in
  let scale a b =
    t.state <-
      Scaling
        {
          a = a.P.id;
          b = b.P.id;
          distance = distance a.client b.client;
          zoom = view.zoom;
          x = ((a.client.client_x +. b.client.client_x) /. 2.) -. view.canvas_left;
          y = ((a.client.client_y +. b.client.client_y) /. 2.) -. view.canvas_top;
        };
    true
  in
  match t.state with
  | Rest -> start ()
  | Blocked waiting ->
      if List.exists (fun id -> Option.is_some (find active id)) waiting then false else start ()
  | Pending d | Moving d -> (
      if press.sample.id = d.press.sample.id then
        if
          press.sample.source = d.press.sample.source && press.sample.button = d.press.sample.button
        then start ()
        else false
      else
        match find active d.press.sample.id with
        | Some first -> scale first press.sample
        | None -> start ())
  | Scaling p -> (
      match (find active p.a, find active p.b) with Some a, Some b -> scale a b | _ -> false)
  | Awaiting_pinch waiting -> (
      if List.mem press.sample.id waiting then false
      else
        match List.find_map (find active) waiting with
        | Some first -> scale first press.sample
        | None -> start ())

let move t (sample : P.sample) ~active =
  match t.state with
  | Scaling p -> (
      match (find active p.a, find active p.b) with
      | Some a, Some b ->
          [ Zoom (p.zoom *. distance a.client b.client /. max 1. p.distance, p.x, p.y) ]
      | _ -> [])
  | (Pending d | Moving d)
    when sample.id = d.press.sample.id && Option.is_some (find active sample.id) ->
      let moving = match t.state with Moving _ -> true | _ -> false in
      if (not moving) && distance sample.client d.origin < 6. then []
      else (
        (if (not moving) && sample.source = P.Touch then
           let length = distance sample.client d.origin in
           d.origin <-
             {
               client_x =
                 d.origin.client_x +. ((sample.client.client_x -. d.origin.client_x) /. length *. 5.);
               client_y =
                 d.origin.client_y +. ((sample.client.client_y -. d.origin.client_y) /. length *. 5.);
             });
        if sample.client <> d.last.client then (
          let dt = max 8. (sample.time -. d.last.time) in
          d.vx <- (sample.client.client_x -. d.last.client.client_x) /. dt;
          d.vy <- (sample.client.client_y -. d.last.client.client_y) /. dt;
          d.last <- sample);
        t.state <- Moving d;
        [
          Pan
            ( d.view.left -. (sample.client.client_x -. d.origin.client_x),
              d.view.top -. (sample.client.client_y -. d.origin.client_y) );
        ])
  | _ -> []

let matches (press : P.press) (sample : P.sample) =
  press.sample.id = sample.id
  && press.sample.source = sample.source
  && (sample.source = P.Touch || press.sample.button = sample.button)

let release t (sample : P.sample) ~active =
  match t.state with
  | Rest -> (No_press, [])
  | Blocked _ ->
      t.state <- blocked active;
      (Contact_suppressed, [])
  | Scaling p ->
      if sample.id = p.a || sample.id = p.b then
        t.state <- (match ids active with [] -> Rest | remaining -> Awaiting_pinch remaining);
      (Dragged, [])
  | Awaiting_pinch _ ->
      t.state <- (match ids active with [] -> Rest | remaining -> Awaiting_pinch remaining);
      (Dragged, [])
  | Pending d | Moving d ->
      if not (matches d.press sample) then (
        if Option.is_none (find active d.press.sample.id) then t.state <- blocked active;
        (Wrong_contact, []))
      else
        let moving = match t.state with Moving _ -> true | _ -> false in
        t.state <- blocked active;
        if sample.cancelled then (Cancelled, [])
        else if moving then
          let effects =
            if
              Array.length active = 0
              && sample.source = P.Touch
              && sample.time -. d.last.time < 160.
              && hypot d.vx d.vy > 0.1
            then [ Coast (-.d.vx, -.d.vy) ]
            else []
          in
          (Dragged, effects)
        else if d.press.intent = P.Pan_only then (Pan_only, [])
        else if distance sample.client d.press.sample.client >= 6. then (Too_far, [])
        else (Activate, [ Tap sample.scene ])

let cancel t ~active = match t.state with Rest -> () | _ -> t.state <- blocked active
