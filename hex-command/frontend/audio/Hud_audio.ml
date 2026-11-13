(* HUD cues are prepared independently of the battlefield renderer. *)
type cue = Action | Choice | Disclosure | Human | Woodland | Skeletons | Merfolk

let clips =
  [|
    (Action, "select-1.wav?v=3a6cff557cea", 0.29 *. 0.25);
    (Choice, "palette/village-contact.mp3", 1.);
    (Disclosure, "palette/light-cloth-rustle.mp3", 0.39);
    (Human, "palette/chainmail-rattle.mp3", 0.39);
    (Woodland, "palette/enchanted-forest-rustle.mp3", 0.39);
    (Skeletons, "palette/bone-crunch.mp3", 0.39);
    (Merfolk, "palette/underwater-belt-rustle.mp3", 0.39);
  |]

type readiness = Audio_state.readiness = { ready : bool; error : string option }

let readiness_owner = Observable.create { ready = false; error = None }
let readiness = Observable.reader readiness_owner

open Browser_audio

let base_url : string = [%mel.raw "import.meta.env?.BASE_URL ?? '/' "]
let gestures = [| "pointerdown"; "pointerup"; "touchend"; "keydown"; "click" |]
let ( let* ) promise next = Js.Promise.then_ next promise

type t = {
  enabled : unit -> bool;
  mutable context : context option;
  mutable buffers : (cue * buffer) list;
  mutable source : source option;
  mutable loading : unit Js.Promise.t option;
  mutable disposed : bool;
  mutable request : int;
  mutable abort : controller option;
  mutable suspended : bool;
  mutable detach : unit -> unit;
}

external backgrounded : Dom.document -> bool = "hidden" [@@mel.get]

let prepared t = List.length t.buffers = Array.length clips
let current_request t request = (not t.disposed) && (not t.suspended) && t.request = request

let owns_preparation t controller =
  (not t.disposed) && (not t.suspended)
  && Option.fold ~none:false ~some:(fun current -> current == controller) t.abort

let cancel_preparation t =
  let active = t.abort in
  t.abort <- None;
  t.loading <- None;
  Option.iter abort active

let stop t =
  t.request <- t.request + 1;
  Option.iter
    (fun source ->
      (try stop_source source with _ -> ());
      disconnect_source source)
    t.source;
  t.source <- None

let unlock t =
  match t.context with
  | None -> Js.Promise.resolve false
  | Some _ when (not (t.enabled ())) || t.disposed || t.suspended -> Js.Promise.resolve false
  | Some owner ->
      let owner =
        if state owner = "closed" then (
          let owner = context [%mel.obj { latencyHint = "interactive" }] in
          t.context <- Some owner;
          owner)
        else owner
      in
      let* () = if state owner <> "running" then resume owner else Js.Promise.resolve () in
      Js.Promise.resolve ((not t.disposed) && state owner = "running")

let prepare t =
  if t.disposed || t.suspended then Js.Promise.resolve ()
  else if prepared t then (
    Observable.set readiness_owner { ready = true; error = None };
    Js.Promise.resolve ())
  else
    match t.loading with
    | Some loading -> loading
    | None ->
        Observable.set readiness_owner { ready = false; error = None };
        let controller = new_controller () in
        t.abort <- Some controller;
        let attempt =
          try
            let owner =
              match t.context with
              | Some context -> context
              | None ->
                  let owner = context [%mel.obj { latencyHint = "interactive" }] in
                  t.context <- Some owner;
                  owner
            in
            let load (cue, file, _) =
              let url = base_url ^ "assets/audio/" ^ file in
              let* response =
                fetch url [%mel.obj { signal = any_signal [| signal controller; deadline 30000 |] }]
              in
              Boot_process.response ~url ~status:(status response) ~source:"hud-fetch"
                ~get_header:(fun name -> header (headers response) name);
              if not (ok response) then
                failwith
                  ("HTTP "
                  ^ string_of_int (status response)
                  ^ " " ^ status_text response ^ " " ^ url);
              let* bytes = array_buffer response in
              if owns_preparation t controller then (
                let* buffer = decode owner bytes in
                if owns_preparation t controller then
                  t.buffers <- (cue, buffer) :: List.remove_assoc cue t.buffers;
                Js.Promise.resolve ())
              else Js.Promise.resolve ()
            in
            let missing =
              Array_ext.filter (fun (cue, _, _) -> not (List.mem_assoc cue t.buffers)) clips
            in
            let* _ = Js.Promise.all (Array.map load missing) in
            if owns_preparation t controller then (
              t.abort <- None;
              Observable.set readiness_owner { ready = prepared t; error = None });
            Js.Promise.resolve ()
          with cause -> Js.Promise.reject cause
        in
        let attempt =
          Js.Promise.catch
            (fun cause ->
              if owns_preparation t controller then (
                t.abort <- None;
                abort controller;
                Observable.set readiness_owner
                  {
                    ready = prepared t;
                    error = Some ("Cannot prepare HUD cues\n" ^ Foreign_error.describe cause);
                  });
              Js.Promise.resolve ())
            attempt
        in
        t.loading <- Some attempt;
        let _ =
          Js.Promise.then_
            (fun () ->
              (match t.loading with
              | Some current when current == attempt -> t.loading <- None
              | _ -> ());
              Js.Promise.resolve ())
            attempt
        in
        attempt

let create enabled =
  let t =
    {
      enabled;
      context = None;
      buffers = [];
      source = None;
      loading = None;
      disposed = false;
      request = 0;
      abort = None;
      suspended = backgrounded Browser_dom.document;
      detach = (fun () -> ());
    }
  in
  let activate event =
    if trusted event then
      let current = t.request in
      let _ =
        Js.Promise.catch
          (fun cause ->
            if current_request t current then
              Observable.set readiness_owner
                {
                  ready = prepared t;
                  error = Some ("Cannot unlock HUD audio\n" ^ Foreign_error.describe cause);
                };
            Js.Promise.resolve false)
          (unlock t)
      in
      ()
  in
  Array.iter
    (fun name -> listen name activate [%mel.obj { capture = true; passive = true }])
    gestures;
  let suspend _ =
    if not t.disposed then (
      t.suspended <- true;
      cancel_preparation t;
      stop t)
  in
  let resume _ =
    if (not t.disposed) && not (backgrounded Browser_dom.document) then (
      t.suspended <- false;
      let _ = prepare t in
      ())
  in
  let visibility event =
    if backgrounded Browser_dom.document then suspend event else resume event
  in
  let window_events = [ ("pagehide", suspend); ("pageshow", resume); ("online", resume) ] in
  Browser_dom.addDocumentListener Browser_dom.document "visibilitychange" visibility;
  List.iter (fun (event, handler) -> Browser_dom.listen_error event handler) window_events;
  t.detach <-
    (fun () ->
      Array.iter (fun name -> unlisten name activate true) gestures;
      Browser_dom.removeDocumentListener Browser_dom.document "visibilitychange" visibility;
      List.iter (fun (event, handler) -> Browser_dom.remove_error event handler) window_events);
  t

let play ?(cue = Action) t =
  match (t.context, List.assoc_opt cue t.buffers) with
  | Some _, Some _ when t.enabled () && (not t.disposed) && not t.suspended -> (
      stop t;
      let current = t.request in
      let allowed =
        Js.Promise.catch
          (fun cause ->
            if current_request t current then
              Observable.set readiness_owner
                {
                  ready = true;
                  error =
                    Some ("Cannot unlock HUD audio for playback\n" ^ Foreign_error.describe cause);
                };
            Js.Promise.resolve false)
          (unlock t)
      in
      let* allowed = allowed in
      match (allowed, t.context, List.assoc_opt cue t.buffers) with
      | true, Some context, Some buffer when current_request t current && t.enabled () ->
          let voice = create_source context and gain = create_gain context in
          let _, _, volume = Option.get (Array.find_opt (fun (kind, _, _) -> kind = cue) clips) in
          gain_value (gain_parameter gain) volume;
          source_buffer voice buffer;
          (* Match the selection cue's subtle +/-0.35 semitone variation. *)
          gain_value (detune_parameter voice) ((Js.Math.random () -. 0.5) *. 70.);
          connect_source voice gain;
          connect_gain gain (destination context);
          ended voice (fun[@u] () ->
              disconnect_source voice;
              disconnect_gain gain;
              if Option.fold ~none:false ~some:(fun current -> current == voice) t.source then
                t.source <- None);
          t.source <- Some voice;
          start_source voice;
          Js.Promise.resolve true
      | _ -> Js.Promise.resolve false)
  | _ -> Js.Promise.resolve false

let dispose t =
  if not t.disposed then (
    t.disposed <- true;
    cancel_preparation t;
    stop t;
    t.buffers <- [];
    t.detach ();
    Option.iter
      (fun context ->
        let _ = Js.Promise.catch (fun _ -> Js.Promise.resolve ()) (close_context context) in
        ())
      t.context;
    t.context <- None)
