open Browser_storage
module Map = Host_collections.Map
module Set = Host_collections.Set

type clip = { file : string; impact_ms : float [@mel.as "impactMs"] }

type palette_unit = {
  selection : string Js.Nullable.t;
  death : string Js.Nullable.t;
  attacks : string Js.Nullable.t Js.Dict.t;
}

type palette = {
  clips : clip Js.Dict.t;
  units : palette_unit Js.Dict.t;
  terrain : string Js.Nullable.t Js.Dict.t;
  fallback_attacks : string Js.Dict.t; [@mel.as "fallbackAttacks"]
  movement : string;
}

external palette : palette = "default" [@@mel.module "@presentation/audio-palette.json"]
external has_own : 'a Js.Dict.t -> string -> bool = "hasOwn" [@@mel.scope "Object"]

let lookup values key = if has_own values key then Js.Dict.get values key else None

let palette_clip family =
  if String.starts_with ~prefix:"palette:" family then
    lookup palette.clips (String.sub family 8 (String.length family - 8))
  else None

let action_families = [| "select"; "recruit"; "turn"; "victory"; "defeat" |]

let families =
  Array.append
    [|
      "attack-bow";
      "attack-melee";
      "unit-soldier";
      "unit-archer";
      "unit-scout";
      "unit-commander";
      "terrain-village";
      "terrain-water";
      "terrain-forest";
      "terrain-hill";
      "terrain-plains";
      "terrain-keep";
    |]
    (Array.append action_families [| "bell" |])

let family_set =
  let set = Set.create () in
  Array.iter
    (fun family ->
      let _ = Set.add set family in
      ())
    families;
  set

let legacy_impact =
  Js.Dict.fromArray
    [|
      ("attack-bow", 121.632653);
      ("attack-melee", 29.024943);
      ("bell", 61.814059);
      ("defeat", 258.61678);
      ("recruit", 88.798186);
      ("select", 8.344671);
      ("terrain-forest", 28.049887);
      ("terrain-hill", 53.446712);
      ("terrain-keep", 24.126984);
      ("terrain-plains", 52.60771);
      ("terrain-village", 10.385488);
      ("terrain-water", 11.972789);
      ("turn", 221.746032);
      ("unit-archer", 31.655329);
      ("unit-commander", 40.521542);
      ("unit-scout", 19.84127);
      ("unit-soldier", 9.727891);
      ("victory", 221.85941);
    |]

type sample = { family : string; key : string; url : string }

let base_url : string = [%mel.raw "import.meta.env?.BASE_URL ?? '/' "]
let revision = "reviewed-ranges-5"

let samples =
  Array.append
    (Array.map
       (fun family ->
         {
           family;
           key = family ^ "-1";
           url = base_url ^ "assets/audio/" ^ family ^ "-1.wav?v=" ^ revision;
         })
       families)
    (Array.map
       (fun (id, clip) ->
         {
           family = "palette:" ^ id;
           key = "palette:" ^ id;
           url = base_url ^ "assets/audio/" ^ clip.file ^ "?v=" ^ revision;
         })
       (Js.Dict.entries palette.clips))

let battle_sounds unit_types terrain_types =
  let cues = Set.create () in
  let _ = Set.add cues palette.movement in
  let add cue =
    Option.iter
      (fun cue ->
        let _ = Set.add cues cue in
        ())
      (Js.Nullable.toOption cue)
  in
  Array.iter
    (fun kind ->
      Option.iter
        (fun unit ->
          add unit.selection;
          add unit.death;
          Array.iter (fun (_, cue) -> add cue) (Js.Dict.entries unit.attacks))
        (lookup palette.units kind))
    unit_types;
  Array.iter (fun terrain -> Option.iter add (lookup palette.terrain terrain)) terrain_types;
  Set.values_array cues

type owner

external owner : unit -> owner = "Object" [@@mel.new]
external named_owner : string -> owner = "%identity"
external game_owner : Native.view -> owner = "%identity"
external combat_owner : Domain.combat_animation -> owner = "%identity"

type interaction = {
  owner : owner option;
  attack_type : string option;
  attack_id : string option;
  attack_sound : string option;
  attack_category : string option;
  attack_damage_type : string option;
  unit_type : string option;
  terrain : string option;
}

let interaction =
  {
    owner = None;
    attack_type = None;
    attack_id = None;
    attack_sound = None;
    attack_category = None;
    attack_damage_type = None;
    unit_type = None;
    terrain = None;
  }

let sound_for_interaction details =
  let fallback () =
    match details.attack_sound with
    | Some family when Set.has family_set family -> family
    | _ -> if details.attack_type = Some "archer" then "attack-bow" else "attack-melee"
  in
  match (details.attack_type, details.attack_id) with
  | Some kind, Some id ->
      let cue =
        Option.bind (lookup palette.units kind) (fun unit ->
            Option.bind (lookup unit.attacks id) Js.Nullable.toOption)
      in
      let cue =
        match cue with
        | Some cue -> Some cue
        | None ->
            Option.bind details.attack_category (fun category ->
                Option.bind details.attack_damage_type (fun damage ->
                    lookup palette.fallback_attacks (category ^ ":" ^ damage)))
      in
      Option.value ~default:(fallback ()) cue
  | _ -> (
      let selected =
        Option.bind details.unit_type (fun kind ->
            Option.bind (lookup palette.units kind) (fun unit ->
                Js.Nullable.toOption unit.selection))
      in
      match selected with
      | Some cue -> cue
      | None ->
          Option.value ~default:"select"
            (Option.bind details.terrain (fun terrain ->
                 Option.bind (lookup palette.terrain terrain) Js.Nullable.toOption)))

type channel = Feedback | Combat
type voice = { sound : Phaser.sound; channel : channel; owner : owner; key : string option }
type request = { owner : owner; mutable cancelled : bool }
type readiness = Audio_state.readiness

let readiness_owner = Observable.create { Audio_state.ready = false; error = None }
let readiness = Observable.reader readiness_owner
let hud_readiness = Hud_audio.readiness

type t = {
  mutable enabled : bool;
  mutable closed : bool;
  mutable game : Phaser.game option;
  mutable hud : Hud_audio.t option;
  mutable ready : bool;
  mutable request_permission : (unit -> bool Js.Promise.t) option;
  voices : voice Set.t;
  requests : request Set.t;
  watchers : (owner, (unit -> unit) Set.t) Map.t;
  mutable voice_request : int;
  mutable unlocking : bool Js.Promise.t option;
  bags : (string, float array) Map.t;
  last_variation : (string, float) Map.t;
  mutable detach : (unit -> unit) option;
}

let report_readiness t ready error =
  if not t.closed then Observable.set readiness_owner { Audio_state.ready; error }

open Phaser_audio

external pop : float array -> float Js.Undefined.t = "pop" [@@mel.send]

let ( let* ) promise next = Js.Promise.then_ next promise

let create () =
  {
    enabled =
      (try Js.Nullable.toOption (storage_get "hex-command-sound-enabled") <> Some "false"
       with _ -> true);
    closed = false;
    game = None;
    hud = None;
    ready = false;
    request_permission = None;
    voices = Set.create ();
    requests = Set.create ();
    watchers = Map.create ();
    voice_request = 0;
    unlocking = None;
    bags = Map.create ();
    last_variation = Map.create ();
    detach = None;
  }

let notify t owner =
  Option.iter
    (fun watchers -> Set.each watchers (fun[@u] watch -> watch ()))
    (Map.get t.watchers owner)

let watch t owner watcher =
  let set = match Map.get t.watchers owner with Some set -> set | None -> Set.create () in
  let _ = Set.add set watcher in
  let _ = Map.set t.watchers owner set in
  fun () ->
    let _ = Set.delete set watcher in
    if Set.size set = 0 then
      let _ = Map.delete t.watchers owner in
      ()

let pending t owner =
  Array.exists (fun (voice : voice) -> voice.owner == owner) (Set.values_array t.voices)
  || Array.exists
       (fun (request : request) -> request.owner == owner && not request.cancelled)
       (Set.values_array t.requests)

let remove_voice t (voice : voice) =
  let _ = Set.delete t.voices voice in
  voice.sound##destroy ();
  notify t voice.owner

let stop_owner ?key t owner =
  if Option.is_none key then
    Set.each t.requests (fun[@u] request ->
        if request.owner == owner then request.cancelled <- true);
  Set.each t.voices (fun[@u] voice ->
      if voice.owner == owner && (Option.is_none key || voice.key = key) then remove_voice t voice);
  notify t owner

let stop_native ?channel t =
  t.voice_request <- t.voice_request + 1;
  if Option.is_none channel then
    Set.each t.requests (fun[@u] request ->
        request.cancelled <- true;
        notify t request.owner);
  Set.each t.voices (fun[@u] voice ->
      if Option.is_none channel || channel = Some voice.channel then remove_voice t voice)

let enabled t = t.enabled

let attach t (game : Phaser.game) permission =
  stop_native t;
  t.game <- Some game;
  t.request_permission <- Some permission;
  t.ready <- false;
  Observable.set readiness_owner { ready = false; error = None };
  let current () = Option.fold ~none:false ~some:(fun current -> current == game) t.game in
  let activate event =
    if t.enabled && Browser_audio.trusted event then
      t.unlocking <-
        Some
          (Js.Promise.catch
             (fun cause ->
               if current () then
                 Observable.set readiness_owner
                   {
                     ready = t.ready;
                     error =
                       Some
                         ("Browser audio permission could not be resumed. Retry with another \
                           gesture.\n" ^ Foreign_error.describe cause);
                   };
               Js.Promise.resolve false)
             (permission ()))
  in
  Array.iter
    (fun name -> Browser_audio.listen name activate [%mel.obj { capture = true; passive = true }])
    Hud_audio.gestures;
  let detach () =
    Array.iter (fun name -> Browser_audio.unlisten name activate true) Hud_audio.gestures;
    if current () then (
      stop_native t;
      t.game <- None;
      t.request_permission <- None;
      t.unlocking <- None;
      t.voice_request <- t.voice_request + 1;
      t.ready <- false)
  in
  t.detach <- Some detach;
  detach

let sound_ready t game =
  if Option.fold ~none:false ~some:(fun current -> current == game) t.game then (
    t.ready <- true;
    Observable.set readiness_owner { ready = true; error = None })

let prepare _ requested =
  Array.iter
    (fun family ->
      if (not (Set.has family_set family)) && Option.is_none (palette_clip family) then
        failwith ("Unknown audio family: " ^ family))
    requested

let prepare_hud t =
  let hud =
    match t.hud with
    | Some hud -> hud
    | None ->
        let hud = Hud_audio.create (fun () -> t.enabled) in
        t.hud <- Some hud;
        hud
  in
  Hud_audio.prepare hud

let play_hud t =
  match t.hud with Some hud -> Hud_audio.play hud | None -> Js.Promise.resolve false

let dispose_hud t =
  Option.iter Hud_audio.dispose t.hud;
  t.hud <- None

let cancel t =
  Option.iter Hud_audio.stop t.hud;
  stop_native t

let set_enabled t value =
  t.enabled <- value;
  if not value then cancel t;
  try storage_set "hex-command-sound-enabled" (string_of_bool value) with _ -> ()

let unlock t ~trusted =
  if (not t.enabled) || not trusted then Js.Promise.resolve false
  else
    let allowed =
      match t.hud with Some hud -> Hud_audio.unlock hud | None -> Js.Promise.resolve false
    in
    let native =
      match t.request_permission with
      | Some permission -> permission ()
      | None -> Js.Promise.resolve false
    in
    let* values = Js.Promise.all [| allowed; native |] in
    Js.Promise.resolve (Array.exists Fun.id values)

let variation t family =
  let bag =
    match Map.get t.bags family with
    | Some bag when Array.length bag > 0 -> bag
    | _ ->
        let bag = [| -1.; -2. /. 3.; -1. /. 3.; 0.; 1. /. 3.; 2. /. 3.; 1. |] in
        for i = Array.length bag - 1 downto 1 do
          let j = int_of_float (floor (Js.Math.random () *. float_of_int (i + 1))) in
          let previous = bag.(i) in
          bag.(i) <- bag.(j);
          bag.(j) <- previous
        done;
        if Map.get t.last_variation family = Some bag.(Array.length bag - 1) then (
          let last = Array.length bag - 1 in
          let previous = bag.(0) in
          bag.(0) <- bag.(last);
          bag.(last) <- previous);
        let _ = Map.set t.bags family bag in
        bag
  in
  let chosen = Option.get (Js.Undefined.toOption (pop bag)) in
  let _ = Map.set t.last_variation family chosen in
  chosen

type timing = { impact_at : float; end_at : float; played : bool }

let schedule_cue ?(use_impact = true) ?(pitch_spread = 0.8) ?owner:owned ?key ?(replace = false) t
    family impact_at channel =
  let omitted () = { impact_at; end_at = impact_at; played = false } in
  match t.game with
  | Some game when t.enabled && t.ready -> (
      let manager = game##sound in
      match Js.Nullable.toOption manager##context with
      | Some context when context_state context = "running" ->
          let clip = palette_clip family in
          if Option.is_none clip && not (Set.has family_set family) then
            failwith ("Unknown audio family: " ^ family);
          let sample_key = if Option.is_some clip then family else family ^ "-1" in
          if not (game##cache##audio##exists sample_key) then (
            Observable.set readiness_owner
              { ready = false; error = Some ("Prepared audio missing: " ^ sample_key) };
            omitted ())
          else
            let owner = match owned with Some owner -> owner | None -> owner () in
            if replace || Option.is_some key then stop_owner ?key t owner;
            let rate = 2. ** (variation t family *. pitch_spread /. 12.) in
            let anchor =
              if use_impact then
                Option.fold
                  ~none:(Option.value ~default:nan (lookup legacy_impact family))
                  ~some:(fun clip -> clip.impact_ms)
                  clip
                /. rate
              else 0.
            in
            let at = now () in
            let stamp =
              if Option.is_some (Js.Undefined.toOption (has_output context)) then
                Some (output context)
              else None
            in
            let mapped =
              Option.fold ~none:false
                ~some:(fun stamp ->
                  Option.value ~default:0. stamp.context_time > 0.
                  && Option.value ~default:0. stamp.performance_time > 0.)
                stamp
            in
            let clock_at =
              if mapped then Option.get (Option.get stamp).context_time else context_time context
            and performance_at =
              if mapped then Option.get (Option.get stamp).performance_time else at
            in
            let requested_start = clock_at +. ((impact_at -. performance_at -. anchor) /. 1000.) in
            let start = max (context_time context) requested_start in
            let volume =
              if Option.is_some clip then 1. else 0.29 *. if family = "select" then 0.25 else 1.
            in
            let sound = add_sound manager sample_key [%mel.obj { volume; rate }] in
            let voice = { sound; channel; owner; key } in
            let _ = Set.add t.voices voice in
            notify t owner;
            let _ = sound##once "complete" (fun[@u] () -> remove_voice t voice) in
            if not (play_delay sound [%mel.obj { delay = max 0. (start -. context_time context) }])
            then (
              remove_voice t voice;
              omitted ())
            else
              let duration = sound##duration /. rate in
              let started = start_time sound in
              Option.iter
                (fun node ->
                  let gain = gain node in
                  value_at gain 0. started;
                  ramp gain volume (started +. min 0.005 (duration /. 4.));
                  value_at gain volume (started +. duration -. min 0.008 (duration /. 4.));
                  ramp gain 0. (started +. duration))
                (Js.Undefined.toOption (volume_node sound));
              let audible_start = performance_at +. ((started -. clock_at) *. 1000.) in
              let actual_impact = audible_start +. anchor in
              Performance_trace.instant "Audio cue scheduled"
                ~args:
                  (Performance_trace.obj
                     [
                       ("family", Js.Json.string family);
                       ("requestedImpact", Js.Json.number impact_at);
                       ("impactAt", Js.Json.number actual_impact);
                       ("sourceStart", Js.Json.number started);
                       ("rate", Js.Json.number rate);
                       ("anchor", Js.Json.number anchor);
                       ("outputTimestamp", Js.Json.boolean mapped);
                     ]);
              {
                impact_at = actual_impact;
                end_at = audible_start +. (duration *. 1000.);
                played = true;
              }
      | _ -> omitted ())
  | _ -> omitted ()

let play_sequence ?(channel = Feedback) ?pitch_spread ?owner:owned ?(replace = false) t cues =
  match t.game with
  | Some game when t.enabled && t.ready ->
      let owner = match owned with Some owner -> owner | None -> owner () in
      if replace then stop_owner t owner;
      let pending = { owner; cancelled = false } in
      let _ = Set.add t.requests pending in
      notify t owner;
      let current_request = t.voice_request in
      let allowed =
        match Js.Nullable.toOption game##sound##context with
        | Some context when context_state context <> "running" -> (
            match t.unlocking with Some unlocking -> unlocking | None -> Js.Promise.resolve false)
        | _ -> Js.Promise.resolve true
      in
      let cleanup () =
        let _ = Set.delete t.requests pending in
        notify t owner
      in
      let operation =
        let* allowed = allowed in
        let current = Option.fold ~none:false ~some:(fun current -> current == game) t.game in
        if
          (not allowed) || (not t.enabled) || (not current)
          || current_request <> t.voice_request
          || pending.cancelled
        then Js.Promise.resolve false
        else
          let at = ref (now ()) and started = ref false in
          Array.iteri
            (fun i family ->
              let spread =
                match pitch_spread with
                | Some spread -> spread
                | None ->
                    if family = "select" then 0.35
                    else if channel = Combat && i > 0 then 1.2
                    else 0.8
              in
              let timing =
                schedule_cue ~use_impact:false ~pitch_spread:spread ~owner t family !at channel
              in
              started := !started || timing.played;
              at := min timing.end_at (!at +. 180.))
            cues;
          Js.Promise.resolve !started
      in
      Performance_trace.then_result operation
        (fun[@u] result ->
          cleanup ();
          Js.Promise.resolve result)
        (fun[@u] error ->
          cleanup ();
          Performance_trace.reject error)
  | _ -> Js.Promise.resolve false

let play ?owner t family = play_sequence ?owner t [| family |]

let play_interaction ?owner t (details : interaction) =
  let owner = match owner with Some _ -> owner | None -> details.owner in
  let spread =
    if Option.is_some details.attack_type || Option.is_some details.unit_type then 0.8
    else if Option.is_some details.terrain then 0.6
    else 0.35
  in
  play_sequence ~pitch_spread:spread ?owner ~replace:true t [| sound_for_interaction details |]

let schedule_movement ?owner t landing_at =
  schedule_cue ~pitch_spread:0.35 ?owner ~key:"movement" t palette.movement landing_at Feedback

let schedule_death ?owner ?key t kind at =
  match Option.bind (lookup palette.units kind) (fun unit -> Js.Nullable.toOption unit.death) with
  | None -> { impact_at = at; end_at = at; played = false }
  | Some cue ->
      schedule_cue ~use_impact:false ~pitch_spread:1.2 ?owner
        ~key:("death:" ^ Option.value ~default:kind key)
        t cue at Combat

let close t =
  if not t.closed then (
    t.closed <- true;
    Option.iter (fun detach -> detach ()) t.detach;
    t.detach <- None;
    cancel t;
    dispose_hud t)
