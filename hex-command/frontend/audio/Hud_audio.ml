(* The HUD's one short menu cue is the explicitly separate pre-renderer owner. *)
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
  mutable buffer : buffer option;
  mutable source : source option;
  mutable loading : unit Js.Promise.t option;
  mutable disposed : bool;
  mutable request : int;
  abort : controller;
  mutable detach : unit -> unit;
}

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
  | Some _ when (not (t.enabled ())) || t.disposed -> Js.Promise.resolve false
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

let create enabled =
  let t =
    {
      enabled;
      context = None;
      buffer = None;
      source = None;
      loading = None;
      disposed = false;
      request = 0;
      abort = new_controller ();
      detach = (fun () -> ());
    }
  in
  let activate event =
    if trusted event then
      let _ =
        Js.Promise.catch
          (fun cause ->
            if not t.disposed then
              Observable.set readiness_owner
                {
                  ready = Option.is_some t.buffer;
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
  t.detach <- (fun () -> Array.iter (fun name -> unlisten name activate true) gestures);
  t

let prepare t =
  match t.loading with
  | Some loading -> loading
  | None ->
      Observable.set readiness_owner { ready = false; error = None };
      let url = base_url ^ "assets/audio/select-1.wav?v=3a6cff557cea" in
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
          let* response =
            fetch url [%mel.obj { signal = any_signal [| signal t.abort; deadline 30000 |] }]
          in
          Boot_process.response ~url ~status:(status response) ~source:"hud-fetch"
            ~get_header:(fun name -> header (headers response) name);
          if not (ok response) then
            failwith ("HTTP " ^ string_of_int (status response) ^ " " ^ status_text response);
          let* bytes = array_buffer response in
          let* prepared = decode owner bytes in
          if not t.disposed then (
            t.buffer <- Some prepared;
            Observable.set readiness_owner { ready = true; error = None });
          Js.Promise.resolve ()
        with cause -> Js.Promise.reject cause
      in
      let attempt =
        Js.Promise.catch
          (fun cause ->
            if not t.disposed then
              Observable.set readiness_owner
                {
                  ready = false;
                  error =
                    Some ("Cannot prepare HUD cue " ^ url ^ "\n" ^ Foreign_error.describe cause);
                };
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

let play t =
  match (t.context, t.buffer) with
  | Some _, Some _ when t.enabled () && not t.disposed -> (
      stop t;
      let current = t.request in
      let allowed =
        Js.Promise.catch
          (fun cause ->
            if not t.disposed then
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
      match (allowed, t.context, t.buffer) with
      | true, Some context, Some buffer when (not t.disposed) && current = t.request && t.enabled ()
        ->
          let voice = create_source context and gain = create_gain context in
          gain_value (gain_parameter gain) (0.29 *. 0.25);
          source_buffer voice buffer;
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
    abort t.abort;
    stop t;
    t.buffer <- None;
    t.detach ();
    Option.iter
      (fun context ->
        let _ = Js.Promise.catch (fun _ -> Js.Promise.resolve ()) (close_context context) in
        ())
      t.context;
    t.context <- None)
