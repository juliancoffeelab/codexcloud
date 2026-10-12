type context
type buffer
type source
type gain
type parameter
type destination
type response
type array_buffer
type voice = { source : source; gain : gain }
type waiter = { finish : bool -> unit }
type t = {
  mutable closed : bool;
  mutable waiters : waiter list;
  mutable enabled : bool; mutable context : context option; mutable master : gain option;
  mutable unlocking : unit Js.Promise.t option; mutable primed : bool;
  mutable prime_source : source option;
  mutable request : int; mutable voices : voice list;
  mutable buffers : buffer Js.Promise.t Js.Dict.t;
  bags : int list Js.Dict.t; previous : int Js.Dict.t;
}
external storage_get : string -> string Js.Nullable.t = "getItem" [@@mel.scope "localStorage"]
external storage_set : string -> string -> unit = "setItem" [@@mel.scope "localStorage"]
external audio_available : unit Js.Undefined.t = "AudioContext" [@@mel.scope "globalThis"]
external webkit_available : unit Js.Undefined.t = "webkitAudioContext" [@@mel.scope "globalThis"]
external new_context : < latencyHint : string > Js.t -> context = "AudioContext" [@@mel.new]
external new_webkit_context : < latencyHint : string > Js.t -> context = "webkitAudioContext" [@@mel.new]
external state : context -> string = "state" [@@mel.get]
external now : context -> float = "currentTime" [@@mel.get]
external rate : context -> float = "sampleRate" [@@mel.get]
external destination : context -> destination = "destination" [@@mel.get]
external create_gain : context -> gain = "createGain" [@@mel.send]
external create_source : context -> source = "createBufferSource" [@@mel.send]
external create_buffer : context -> int -> int -> float -> buffer = "createBuffer" [@@mel.send]
external gain_parameter : gain -> parameter = "gain" [@@mel.get]
external parameter_value : parameter -> float = "value" [@@mel.get]
external set_value : parameter -> float -> unit = "value" [@@mel.set]
external cancel_values : parameter -> float -> unit = "cancelScheduledValues" [@@mel.send]
external value_at : parameter -> float -> float -> unit = "setValueAtTime" [@@mel.send]
external ramp : parameter -> float -> float -> unit = "linearRampToValueAtTime" [@@mel.send]
external connect_gain : gain -> 'a -> unit = "connect" [@@mel.send]
external connect_source : source -> gain -> unit = "connect" [@@mel.send]
external disconnect_source : source -> unit = "disconnect" [@@mel.send]
external disconnect_gain : gain -> unit = "disconnect" [@@mel.send]
external source_buffer : source -> buffer -> unit = "buffer" [@@mel.set]
external on_ended : source -> (unit -> unit) -> unit = "onended" [@@mel.set]
external start : source -> unit = "start" [@@mel.send]
external stop : source -> float -> unit = "stop" [@@mel.send]
external resume : context -> unit Js.Promise.t = "resume" [@@mel.send]
external close_context : context -> unit Js.Promise.t = "close" [@@mel.send]
external fetch : string -> response Js.Promise.t = "fetch"
external response_ok : response -> bool = "ok" [@@mel.get]
external array_buffer : response -> array_buffer Js.Promise.t = "arrayBuffer" [@@mel.send]
external decode : context -> array_buffer -> buffer Js.Promise.t = "decodeAudioData" [@@mel.send]
external delete : 'a Js.Dict.t -> string -> unit = "deleteProperty" [@@mel.scope "Reflect"]
external reject : 'a -> 'b Js.Promise.t = "reject" [@@mel.scope "Promise"]
let base_url : string = [%mel.raw "import.meta.env?.BASE_URL ?? '/' "]
let families = ["attack-bow"; "attack-melee"; "unit-soldier"; "unit-archer";
                "unit-scout"; "unit-commander"; "terrain-village"; "terrain-water";
                "terrain-forest"; "terrain-hill"; "terrain-plains"; "terrain-keep";
                "select"; "recruit"; "turn"; "victory"; "defeat"]
let create () =
  { closed = false; waiters = [];
    enabled = (try Js.Nullable.toOption (storage_get "hex-command-sound-enabled") <> Some "false" with _ -> true);
    context = None; master = None; unlocking = None; primed = false; prime_source = None; request = 0;
    voices = []; buffers = Js.Dict.empty (); bags = Js.Dict.empty (); previous = Js.Dict.empty () }
let enabled t = t.enabled
let stop_voices t =
  let voices = t.voices in
  t.voices <- [];
  match t.context with
  | None -> ()
  | Some context -> List.iter (fun voice -> try
                                  let time = now context in
                                  let parameter = gain_parameter voice.gain in
                                  cancel_values parameter time;
                                  value_at parameter (parameter_value parameter) time;
                                  ramp parameter 0. (time +. 0.008);
                                  stop voice.source (time +. 0.009)
                                with _ -> ()) voices
let cancel t = t.request <- t.request + 1; stop_voices t
let close t =
  if not t.closed then (
    t.closed <- true;
    let voices = t.voices and context = t.context and master = t.master and prime_source = t.prime_source in
    cancel t;
    t.context <- None;
    t.master <- None;
    t.unlocking <- None;
    t.prime_source <- None;
    Array.iter (delete t.buffers) (Js.Dict.keys t.buffers);
    t.buffers <- Js.Dict.empty ();
    Array.iter (delete t.bags) (Js.Dict.keys t.bags);
    Array.iter (delete t.previous) (Js.Dict.keys t.previous);
    let waiters = t.waiters in
    t.waiters <- [];
    List.iter (fun waiter -> waiter.finish false) waiters;
    List.iter (fun voice ->
        (try disconnect_source voice.source with _ -> ());
        (try disconnect_gain voice.gain with _ -> ())) voices;
    Option.iter (fun source -> try disconnect_source source with _ -> ()) prime_source;
    Option.iter (fun master -> try disconnect_gain master with _ -> ()) master;
    Option.iter (fun context -> try
                    ignore (Js.Promise.catch (fun _ -> Js.Promise.resolve ()) (close_context context))
                  with _ -> ()) context)
let set_enabled t value =
  if not t.closed then (
    t.enabled <- value;
    t.request <- t.request + 1;
    (try storage_set "hex-command-sound-enabled" (string_of_bool value) with _ -> ());
    if not value then stop_voices t)
let interaction_family ?attack ?unit_type ?terrain () =
  match attack, unit_type with
  | Some "archer", _ -> "attack-bow"
  | Some _, _ -> "attack-melee"
  | None, Some ("scout" | "archer" | "wolf" | "thorncaller" | "adept") -> "terrain-plains"
  | None, Some ("soldier" | "warden" | "boar" | "merfolk-maceguard" | "merfolk-maceguard-l2") -> "unit-archer"
  | None, Some ("wildkeeper" | "necromancer" | "skeleton" | "bonearcher" | "skeleton-impact-caster" | "skeleton-impact-caster-l2") -> "unit-commander"
  | None, Some kind when List.mem ("unit-" ^ kind) families -> "unit-" ^ kind
  | _ ->
    let terrain = match terrain with
      | Some ("ford" | "marsh") -> "water" | Some "grove" -> "forest"
      | Some ("ruins" | "fort") -> "keep" | Some value -> value | None -> "" in
    let family = "terrain-" ^ terrain in
    if List.mem family families then family else "select"
let variation t family =
  let bag = match Js.Dict.get t.bags family with
    | Some (_ :: _ as bag) -> bag
    | _ ->
      let values = [|1; 2; 3|] in
      for i = 2 downto 1 do
        let j = int_of_float (Js.Math.random () *. float_of_int (i + 1)) in
        let previous = values.(i) in values.(i) <- values.(j); values.(j) <- previous
      done;
      if Js.Dict.get t.previous family = Some values.(2) then (
        let previous = values.(0) in values.(0) <- values.(2); values.(2) <- previous);
      [values.(2); values.(1); values.(0)] in
  match bag with
  | chosen :: remaining -> Js.Dict.set t.bags family remaining; Js.Dict.set t.previous family chosen; chosen
  | [] -> assert false
let sample t context family variant =
  let name = family ^ "-" ^ string_of_int variant in
  match Js.Dict.get t.buffers name with
  | Some loading -> loading
  | None ->
    let cache = t.buffers in
    let loading = Js.Promise.then_ (fun response ->
        if t.closed || not (response_ok response) then reject "Sound unavailable"
        else Js.Promise.then_ (fun bytes ->
            if t.closed then reject "Sound unavailable" else decode context bytes)
            (array_buffer response))
        (fetch (base_url ^ "assets/audio/" ^ name ^ ".wav?v=6ad634f16189")) in
    let guarded = Js.Promise.catch (fun error -> delete cache name; reject error) loading in
    Js.Dict.set cache name guarded;
    guarded
let ignore_failure promise = Js.Promise.catch (fun _ -> Js.Promise.resolve ()) promise
let pending t promise =
  if t.closed then Js.Promise.resolve false else
    Js.Promise.make (fun ~resolve ~reject:_ ->
        let waiter = { finish = (fun result -> (resolve result [@u])) } in
        t.waiters <- waiter :: t.waiters;
        ignore (Js.Promise.then_ (fun result ->
            t.waiters <- List.filter (fun current -> current != waiter) t.waiters;
            waiter.finish result;
            Js.Promise.resolve ()) promise))
let unlock t ~trusted =
  if t.closed || not t.enabled || not trusted then Js.Promise.resolve false else
    try
      let context = match t.context with
        | Some context when state context <> "closed" -> Some context
        | _ ->
          let options = [%mel.obj { latencyHint = "interactive" }] in
          if Js.Undefined.toOption audio_available <> None then Some (new_context options)
          else if Js.Undefined.toOption webkit_available <> None then Some (new_webkit_context options)
          else None in
      match context with
      | None -> Js.Promise.resolve false
      | Some context ->
        let existing = match t.context with Some existing -> existing == context | None -> false in
        if not existing then (
          Option.iter (fun master -> try disconnect_gain master with _ -> ()) t.master;
          Option.iter (fun source -> try disconnect_source source with _ -> ()) t.prime_source;
          t.request <- t.request + 1;
          t.context <- Some context;
          t.primed <- false;
          t.prime_source <- None;
          t.buffers <- Js.Dict.empty ();
          let master = create_gain context in
          set_value (gain_parameter master) 0.29;
          connect_gain master (destination context);
          (* Load only the selected cue. Warming every variation here would
             launch 51 fetch/decode jobs ahead of the user's first interaction. *)
          t.master <- Some master);
        let attempt = if state context = "running" then Js.Promise.resolve () else ignore_failure (resume context) in
        (* Resume is attempted in every gesture; an unresolved touch-start request
           must never prevent the completed tap from unlocking audio. *)
        t.unlocking <- Some attempt;
        ignore (Js.Promise.then_ (fun () ->
            (match t.unlocking with Some current when current == attempt -> t.unlocking <- None | _ -> ());
            Js.Promise.resolve ()) attempt);
        if not t.primed then (match t.master with
            | None -> ()
            | Some master ->
              let source = create_source context in
              source_buffer source (create_buffer context 1 1 (rate context));
              connect_source source master;
              t.prime_source <- Some source;
              on_ended source (fun () ->
                  (match t.prime_source with Some current when current == source -> t.prime_source <- None | _ -> ());
                  try disconnect_source source with _ -> ());
              start source;
              t.primed <- true);
        pending t (Js.Promise.catch (fun _ -> Js.Promise.resolve false)
                     (Js.Promise.then_ (fun () ->
                          let current = match t.context with Some current -> current == context | None -> false in
                          Js.Promise.resolve (not t.closed && t.enabled && current && state context = "running")) attempt))
    with _ -> Js.Promise.resolve false
let play t family =
  match t.context, t.master with
  | Some context, Some master when not t.closed && t.enabled && List.mem family families ->
    t.request <- t.request + 1;
    let request = t.request in
    stop_voices t;
    let ready = match t.unlocking with Some pending -> pending | None -> Js.Promise.resolve () in
    pending t (Js.Promise.catch (fun _ -> Js.Promise.resolve false)
                 (Js.Promise.then_ (fun () ->
                      if t.closed || state context <> "running" then Js.Promise.resolve false else
                        Js.Promise.then_ (fun buffer ->
                            if t.closed || not t.enabled || request <> t.request || state context <> "running" then Js.Promise.resolve false else (
                              let source = create_source context and gain = create_gain context in
                              source_buffer source buffer;
                              set_value (gain_parameter gain) (if family = "select" then 0.25 else 1.);
                              connect_source source gain;
                              connect_gain gain master;
                              let voice = { source; gain } in
                              t.voices <- voice :: t.voices;
                              on_ended source (fun () ->
                                  t.voices <- List.filter (fun current -> current != voice) t.voices;
                                  (try disconnect_source source with _ -> ());
                                  (try disconnect_gain gain with _ -> ()));
                              start source;
                              Js.Promise.resolve true)) (sample t context family (variation t family))) ready))
  | _ -> Js.Promise.resolve false
let play_interaction t ?attack ?unit_type ?terrain () =
  play t (interaction_family ?attack ?unit_type ?terrain ())
