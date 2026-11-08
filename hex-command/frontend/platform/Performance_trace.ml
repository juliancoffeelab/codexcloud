(* Opt-in Chrome/Perfetto recording. No observer or frame loop exists while off. *)
module Set = Host_collections.Set

type event = {
  name : string;
  cat : string;
  ph : string;
  ts : float;
  dur : float option;
  pid : int;
  tid : int;
  s : string option;
  args : Js.Json.t option;
}

type config = { id : int; origin : float }
type status = Empty | Recording | Stopping | Ready
type state = { status : status; events : int; dropped : int; reason : string option }
type timer
type observer
type entry = { name : string; start_time : float; [@mel.as "startTime"] duration : float }
type entry_list

external now : unit -> float = "now" [@@mel.scope "performance"]
external origin : float = "timeOrigin" [@@mel.scope "performance"]
external timeout : (unit -> unit) -> int -> timer = "setTimeout"
external clear_timeout : timer -> unit = "clearTimeout"
external frame : (unit -> unit) -> int = "requestAnimationFrame"
external cancel_frame : int -> unit = "cancelAnimationFrame"
external entries : entry_list -> entry array = "getEntries" [@@mel.send]

external supported : string array Js.Undefined.t = "supportedEntryTypes"
[@@mel.scope "globalThis", "PerformanceObserver"]

external observer_available : unit Js.Undefined.t = "PerformanceObserver" [@@mel.scope "globalThis"]
external new_observer : ((entry_list -> unit)[@u]) -> observer = "PerformanceObserver" [@@mel.new]

type content_type = { kind : string [@mel.as "type"] }

external observe : observer -> content_type -> unit = "observe" [@@mel.send]
external disconnect : observer -> unit = "disconnect" [@@mel.send]
external resources : string -> entry array = "getEntriesByType" [@@mel.scope "performance"]
external visibility : string = "visibilityState" [@@mel.scope "document"]
external user_agent : string = "userAgent" [@@mel.scope "navigator"]
external inner_width : int = "innerWidth" [@@mel.scope "globalThis"]
external inner_height : int = "innerHeight" [@@mel.scope "globalThis"]
external pixel_ratio : float = "devicePixelRatio" [@@mel.scope "globalThis"]

external listen_document : string -> (unit -> unit) -> unit = "addEventListener"
[@@mel.scope "document"]

external unlisten_document : string -> (unit -> unit) -> unit = "removeEventListener"
[@@mel.scope "document"]

external listen : string -> (Dom.event -> unit) -> unit = "addEventListener" [@@mel.scope "window"]

external unlisten : string -> (Dom.event -> unit) -> unit = "removeEventListener"
[@@mel.scope "window"]

external message : Dom.event -> string = "message" [@@mel.get]
external reason : Dom.event -> Foreign_error.t = "reason" [@@mel.get]
external includes : string -> string -> bool = "includes" [@@mel.send]

type date

external date : unit -> date = "Date" [@@mel.new]
external iso : date -> string = "toISOString" [@@mel.send]

let timestamp () = iso (date ())
let clock () = origin +. now ()
let obj fields = Js.Json.object_ (Js.Dict.fromArray (Array.of_list fields))
let string = Js.Json.string
let number = Js.Json.number
let int value = number (float_of_int value)
let listeners : (state -> unit) Set.t = Set.create ()
let max_events = 250000
let max_duration = 600000
let status = ref Empty
let dropped = ref 0
let reason_ = ref None
let config = ref None
let serial = ref 0
let events : event array ref = ref [||]
let metadata : Js.Json.t Js.Dict.t ref = ref (Js.Dict.empty ())
let context : (unit -> Js.Json.t) option ref = ref None
let frame_id = ref 0
let timer = ref None
let notification = ref None
let observer = ref None
let stopping : unit Js.Promise.t option ref = ref None

let state () =
  { status = !status; events = Array.length !events; dropped = !dropped; reason = !reason_ }

let notify () =
  let state = state () in
  Set.each listeners (fun[@u] listener -> listener state)

let subscribe listener =
  let _ = Set.add listeners listener in
  listener (state ());
  fun () ->
    let _ = Set.delete listeners listener in
    ()

let configuration () = if !status = Recording then !config else None
let event ?args ?dur ?s ~cat ~ph ~ts name = { name; cat; ph; ts; dur; pid = 1; tid = 1; s; args }
let stop_current : (string -> unit Js.Promise.t) ref = ref (fun _ -> Js.Promise.resolve ())

let receive id batch =
  if
    Option.fold ~none:false ~some:(fun config -> config.id = id) !config
    && (!status = Recording || !status = Stopping)
  then (
    let room = max_events - Array.length !events in
    for i = 0 to min room (Array.length batch) - 1 do
      let _ = Array_ext.append_value !events batch.(i) in
      ()
    done;
    dropped := !dropped + max 0 (Array.length batch - room);
    if Option.is_none !notification then
      notification :=
        Some
          (timeout
             (fun () ->
               notification := None;
               notify ())
             500);
    if Array.length !events >= max_events && !status = Recording then
      let _ = !stop_current "Event limit reached" in
      ())

let instant ?args name =
  match !config with
  | Some current when !status = Recording || !status = Stopping ->
      receive current.id
        [|
          event ?args ~s:"t" ~cat:"browser" ~ph:"i" ~ts:((clock () -. current.origin) *. 1000.) name;
        |]
  | _ -> ()

let command_begin current name args started =
  let args = Option.map (fun input -> obj [ ("input", input) ]) args in
  receive current.id
    [|
      event ?args ~s:"t" ~cat:"action" ~ph:"i"
        ~ts:((started -. current.origin) *. 1000.)
        (name ^ " begin");
    |]

let command_end current name args started error =
  let args =
    match (args, error) with
    | None, None -> None
    | _ ->
        let fields = match args with None -> [] | Some args -> [ ("input", args) ] in
        Some
          (obj (fields @ match error with None -> [] | Some error -> [ ("error", string error) ]))
  in
  receive current.id
    [|
      event ?args
        ~dur:((clock () -. started) *. 1000.)
        ~cat:"action" ~ph:"X"
        ~ts:((started -. current.origin) *. 1000.)
        name;
    |]

let measure ?args name operation =
  match configuration () with
  | None -> operation ()
  | Some current -> (
      let started = clock () in
      command_begin current name args started;
      match operation () with
      | result ->
          command_end current name args started None;
          result
      | exception error ->
          command_end current name args started (Some (Foreign_error.exception_ error));
          Fault.raise_original error)

external then_result :
  'a Js.Promise.t ->
  (('a -> 'b Js.Promise.t)[@u]) ->
  ((Foreign_error.t -> 'b Js.Promise.t)[@u]) ->
  'b Js.Promise.t = "then"
[@@mel.send]

external reject : Foreign_error.t -> 'a Js.Promise.t = "reject" [@@mel.scope "Promise"]

let on_error event = instant "Browser error" ~args:(obj [ ("message", string (message event)) ])

let on_rejection event =
  instant "Unhandled rejection"
    ~args:(obj [ ("message", string (Foreign_error.describe (reason event))) ])

let on_visibility () =
  instant "Visibility changed" ~args:(obj [ ("visibility", string visibility) ])

let supported_entries () =
  if Option.is_none (Js.Undefined.toOption observer_available) then [||]
  else Option.value ~default:[||] (Js.Undefined.toOption supported)

let start get_context =
  if !status <> Recording && !status <> Stopping then (
    events := [||];
    context := Some get_context;
    stopping := None;
    incr serial;
    let current = { id = !serial; origin = clock () } in
    config := Some current;
    status := Recording;
    dropped := 0;
    reason_ := None;
    let assets =
      resources "resource"
      |> Array_ext.filter (fun entry ->
          let path = List.hd (String.split_on_char '?' entry.name) in
          includes entry.name "/assets/"
          && (String.ends_with ~suffix:".js" path || String.ends_with ~suffix:".css" path))
    in
    metadata :=
      Js.Dict.fromArray
        [|
          ("format", string "hex-command-performance");
          ("version", int 1);
          ("startedAt", string (timestamp ()));
          ("recordingOrigin", number current.origin);
          ("performanceTimeOrigin", number origin);
          ("browser", string user_agent);
          ("supportedPerformanceEntries", Js.Json.array (Array.map string (supported_entries ())));
          ("assets", Js.Json.array (Array.map (fun entry -> string entry.name) assets));
          ( "viewport",
            obj
              [
                ("width", int inner_width);
                ("height", int inner_height);
                ("pixelRatio", number pixel_ratio);
              ] );
          ("start", get_context ());
        |];
    receive current.id
      [|
        event
          ~args:(obj [ ("name", string "Browser / actions") ])
          ~cat:"metadata" ~ph:"M" ~ts:0. "thread_name";
      |];
    instant "Recording started";
    let previous = ref (now ()) in
    let rec tick () =
      if !status = Recording then (
        let at = now () in
        receive current.id
          [|
            event
              ~args:(obj [ ("visible", string visibility) ])
              ~dur:((at -. !previous) *. 1000.)
              ~cat:"frame" ~ph:"X"
              ~ts:((origin +. !previous -. current.origin) *. 1000.)
              "Browser frame interval";
          |];
        previous := at;
        frame_id := frame tick)
    in
    frame_id := frame tick;
    if Array.exists (( = ) "longtask") (supported_entries ()) then (
      let active =
        new_observer (fun[@u] list ->
            match !config with
            | Some _ when !status = Recording ->
                let batch =
                  Array_ext.filter_map
                    (fun entry ->
                      let started = origin +. entry.start_time in
                      let ended = started +. entry.duration in
                      let clipped = max started current.origin in
                      if ended < clipped then None
                      else
                        Some
                          (event
                             ~dur:((ended -. clipped) *. 1000.)
                             ~cat:"longtask" ~ph:"X"
                             ~ts:((clipped -. current.origin) *. 1000.)
                             "Browser long task"))
                    (entries list)
                in
                receive current.id batch
            | _ -> ())
      in
      observe active { kind = "longtask" };
      observer := Some active);
    listen_document "visibilitychange" on_visibility;
    listen "error" on_error;
    listen "unhandledrejection" on_rejection;
    timer :=
      Some
        (timeout
           (fun () ->
             let _ = !stop_current "Ten-minute limit reached" in
             ())
           max_duration);
    notify ())

type settlement = < status : string > Js.t

external all_settled : unit Js.Promise.t array -> settlement array Js.Promise.t = "allSettled"
[@@mel.scope "Promise"]

let stop ?(reason = "Stopped by user") () =
  match !stopping with
  | Some stopping -> stopping
  | None -> (
      match !config with
      | Some _ when !status = Recording ->
          status := Stopping;
          reason_ := Some reason;
          instant "Recording stopped" ~args:(obj [ ("reason", string reason) ]);
          Option.iter clear_timeout !timer;
          cancel_frame !frame_id;
          Option.iter disconnect !observer;
          observer := None;
          unlisten_document "visibilitychange" on_visibility;
          unlisten "error" on_error;
          unlisten "unhandledrejection" on_rejection;
          notify ();
          let flushing = [||] in
          let finished =
            Js.Promise.then_
              (fun results ->
                Js.Dict.set !metadata "stoppedAt" (string (timestamp ()));
                Option.iter (fun context -> Js.Dict.set !metadata "end" (context ())) !context;
                Js.Dict.set !metadata "workerFlushFailures"
                  (int
                     (Array.fold_left
                        (fun count result ->
                          if result##status = "rejected" then count + 1 else count)
                        0 results));
                status := Ready;
                Option.iter clear_timeout !notification;
                notification := None;
                notify ();
                Js.Promise.resolve ())
              (all_settled flushing)
          in
          stopping := Some finished;
          finished
      | _ -> Js.Promise.resolve ())

let () = stop_current := fun reason -> stop ~reason ()

type file = {
  trace_events : event array; [@mel.as "traceEvents"]
  display_time_unit : string; [@mel.as "displayTimeUnit"]
  metadata : Js.Json.t;
}

external stringify : file -> string = "stringify" [@@mel.scope "JSON"]
external replace_all : string -> string -> string -> string = "replaceAll" [@@mel.send]

let download () =
  Js.Promise.then_
    (fun () ->
      if !status = Ready then (
        let metadata = Js.Dict.fromArray (Js.Dict.entries !metadata) in
        Js.Dict.set metadata "events" (int (Array.length !events));
        Js.Dict.set metadata "dropped" (int !dropped);
        Option.iter (fun reason -> Js.Dict.set metadata "stopReason" (string reason)) !reason_;
        Js.Dict.set metadata "gpuTiming"
          (string
             "CPU render-submission durations do not measure GPU completion. Browser frame \
              intervals are scheduling intervals, not physical display FPS.");
        let file =
          { trace_events = !events; display_time_unit = "ms"; metadata = Js.Json.object_ metadata }
        in
        Browser_download.json ~revoke_after:1000
          ("hex-performance-" ^ replace_all (timestamp ()) ":" "-" ^ ".json")
          (stringify file));
      Js.Promise.resolve ())
    (stop ())
