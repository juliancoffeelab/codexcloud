type model = {
  game : Native.view; catalog : Catalog.t; artwork : Art.t;
  selected : string option; target : string option; inspected : string option;
  focused : string; revision : int; status : string; sound : bool;
  can_undo : bool;
}
type actions = {
  choose : string -> unit; focus : string -> unit; clear : bool -> unit;
  attack : string -> unit; recruit : string -> string -> unit;
  advance : string -> unit; end_turn : unit -> unit Js.Promise.t;
  undo : unit -> unit; reset : string -> unit; start_campaign : string -> unit;
  next_mission : string array -> unit; retry : unit -> unit;
  dialogue : bool -> unit; previous_dialogue : unit -> unit;
  set_sound : bool -> unit Js.Promise.t;
}
type clock = float -> unit Js.Promise.t
type listener = { mutable alive : bool; receive : model -> unit }
type undo = { checkpoint : Domain.saved_game; unit : string; round : float }
type t = {
  factory : Native.factory; catalog : Catalog.t; audio : Audio.t; saves : Save.t;
  clock : clock; mutable battle : Native.battle; mutable latest : model;
  mutable actions : actions option; mutable listeners : listener list;
  publications : model Queue.t; mutable notifying : bool;
  mutable disposed : bool; mutable initialized : bool; mutable epoch : int;
  mutable pending_enemy : bool; mutable history : undo list;
  mutable effects : (int * int * (unit -> unit)) list;
}
external timeout : (unit -> unit) -> float -> unit = "setTimeout"
external console_error : exn -> unit = "error" [@@mel.scope "console"]
external freeze : 'a -> 'a = "freeze" [@@mel.scope "Object"]
external reject : 'a -> 'b Js.Promise.t = "reject" [@@mel.scope "Promise"]
let pause milliseconds = Js.Promise.make (fun ~resolve ~reject:_ -> timeout (fun () -> let completed = () in (resolve completed [@u])) milliseconds)
external prebuilt : Terrain.sprite Js.Dict.t = "default" [@@mel.module "virtual:terrain-prebuilt"]
let preload map = Terrain.preload_manifest prebuilt map
let initial_status = "Select a unit. Capture villages. Defeat the red commander."
let state t = Native.state (Native.view t.battle)
let alive t = if t.disposed then Js.Exn.raiseError "This battle session has closed."
let active t = let s = state t in s.turn = Domain.side_player && Js.Nullable.isNullable s.winner && not (Story.dialogue_pending s.campaign)
let require_actor t id =
  alive t;
  if not (active t) then Js.Exn.raiseError "Wait for your turn.";
  match Native.unit (Native.view t.battle) id with
  | Some unit when unit.side = Domain.side_player -> ()
  | _ -> Js.Exn.raiseError "This unit cannot act now."
let can_undo t =
  match t.history with
  | previous :: _ when previous.round = (state t).round && active t -> true
  | _ -> false
let view t = t.latest
let emit t model =
  Queue.push model t.publications;
  if not t.notifying then (
    t.notifying <- true;
    while not (Queue.is_empty t.publications) && not t.disposed do
      let next = Queue.pop t.publications in
      let listeners = t.listeners in
      List.iter (fun listener -> if listener.alive && not t.disposed then
                    try listener.receive next with exception_ -> console_error exception_) listeners
    done;
    t.notifying <- false;
    if t.disposed then Queue.clear t.publications)
let subscribe t receive =
  alive t;
  let listener = {alive = true; receive} in
  t.listeners <- t.listeners @ [listener];
  let cleanup () = listener.alive <- false; t.listeners <- List.filter (fun other -> other != listener) t.listeners in
  (try receive t.latest with exception_ -> cleanup (); raise exception_);
  cleanup
let publish ?(render=true) ?(winner=true) ?(effects=(fun () -> ())) ?(next=(fun () -> ())) t =
  if not t.disposed then (
    let current = Native.view t.battle in
    let s = Native.state current in
    let status = if winner then match Js.Nullable.toOption s.winner with
        | None -> t.latest.status
        | Some side -> Option.value ~default:(if side = Domain.side_player then
                                                "Victory! The enemy commander has fallen. The field is yours."
                                              else "Your commander has fallen. Start a new battle and try a different approach.")
                         (Option.bind s.campaign (fun campaign -> Js.Nullable.toOption campaign.outcome))
      else t.latest.status in
    let model = freeze {t.latest with game = current; status; sound = Audio.enabled t.audio;
                                      can_undo = can_undo t; revision = t.latest.revision + (if render then 1 else 0)} in
    t.latest <- model;
    if render then t.effects <- [];
    effects ();
    emit t model;
    if not t.disposed && t.latest == model then next ())
let message t status = if not t.disposed then (t.latest <- {t.latest with status}; publish ~render:false ~winner:false t)
let error t exception_ =
  let reason = match Js.Exn.asJsExn exception_ with
    | Some error -> Option.value ~default:"Unexpected session failure." (Js.Exn.message error)
    | None -> match exception_ with Failure reason | Invalid_argument reason -> reason | _ -> "Unexpected session failure." in
  message t reason
let safely t action = try action () with exception_ -> error t exception_
let promise action = try action () with exception_ ->
match Js.Exn.asJsExn exception_ with Some error -> reject error | None -> reject exception_
let deferred t action = t.effects <- (t.epoch, t.latest.revision, action) :: t.effects
let rendered t revision =
  if not t.disposed && revision = t.latest.revision then (
    let effects = List.rev t.effects in
    t.effects <- [];
    List.iter (fun (epoch, requested, action) ->
        if not t.disposed && epoch = t.epoch && requested = t.latest.revision then action ()) effects)
let save t pending_enemy = Save.write t.saves (Native.save t.battle) ~pending_enemy
let feedback t before =
  let after = state t in
  let changes = Presentation.changes before after in
  Array.iter (fun (tile, motion) -> deferred t (fun () -> Motion.animate_tile tile
                                                   (match motion with Presentation.Move -> "move" | Attack -> "attack" | Recruit -> "recruit"))) changes.tile_effects;
  Array.iter (fun tile -> deferred t (fun () -> Motion.animate_tile tile "capture")) changes.captured_tiles;
  if changes.turn_changed then deferred t Motion.animate_turn;
  let cue () =
    if changes.attacked then (
      let sound = Option.bind after.lastCombat Js.Nullable.toOption |> Option.map (fun (combat : Domain.last_combat) -> combat.sound) in
      ignore (Audio.play t.audio (Option.value ~default:"attack-melee" sound)))
    else match changes.new_winner with
      | Some side -> ignore (Audio.play t.audio (if side = Domain.side_player then "victory" else "defeat"))
      | None when changes.recruited || changes.advanced -> ignore (Audio.play t.audio "recruit")
      | None -> match changes.moved_unit with
        | Some unit -> ignore (Audio.play_interaction t.audio ~unit_type:unit.type_
                                 ?terrain:(Option.map (fun (tile : Domain.tile) -> tile.terrain) (Native.tile (Native.view t.battle) unit.tile)) ())
        | None -> if Array.length changes.captured_tiles > 0 then ignore (Audio.play_interaction t.audio ~terrain:"village" ()) in
  deferred t cue
let commit ?(feedback_enabled=true) ?(next=(fun () -> ())) t before =
  if (state t).turn = Domain.side_player then save t false;
  publish ~effects:(fun () -> if feedback_enabled then feedback t before) ~next t
let dispatch ?(accepted=(fun () -> ())) ?(next=(fun () -> ())) t command =
  let before = state t in
  ignore (Native.dispatch t.battle command);
  accepted ();
  commit ~next t before
let selection_feedback t tile =
  deferred t (fun () ->
      Motion.animate_tile tile "select";
      ignore (Audio.play_interaction t.audio
                ?unit_type:(Option.map (fun (unit : Domain.public_unit) -> unit.type_) (Native.at (Native.view t.battle) tile))
                ?terrain:(Option.map (fun (tile : Domain.tile) -> tile.terrain) (Native.tile (Native.view t.battle) tile)) ()))
let focus t tile =
  alive t;
  if tile <> t.latest.focused && Native.tile t.latest.game tile <> None then (
    t.latest <- {t.latest with focused = tile}; publish ~render:false t)
let clear t inspection =
  alive t;
  if t.latest.selected <> None || t.latest.target <> None || (inspection && t.latest.inspected <> None) then (
    t.latest <- {t.latest with selected=None; target=None; inspected=(if inspection then None else t.latest.inspected)};
    publish ~next:(fun () -> message t "Selection cleared.") t)
let move ?(next=(fun () -> ())) t unit tile =
  require_actor t unit;
  let checkpoint = Native.save t.battle and before = state t in
  ignore (Native.dispatch t.battle (Domain.Command_move {unit=Native.unit_id unit; tile}));
  if active t then t.history <- {checkpoint; unit; round=before.round} :: t.history;
  t.latest <- {t.latest with selected=Some unit; target=None; inspected=Some tile};
  commit ~next t before
let choose t tile_id = safely t (fun () ->
    alive t;
    match Native.tile t.latest.game tile_id with
    | None -> ()
    | Some tile ->
      t.latest <- {t.latest with inspected=Some tile_id; focused=tile_id};
      if not (active t) then publish ~render:false ~effects:(fun () -> selection_feedback t tile_id) t
      else match Native.at t.latest.game tile_id with
        | Some unit when Some unit.id = t.latest.selected || Some unit.id = t.latest.target ->
          clear t false; selection_feedback t tile_id
        | Some occupant ->
          let is_target = match t.latest.selected with
            | Some id -> Array.exists (fun (unit : Domain.public_unit) -> unit.id = occupant.id) (Native.targets t.latest.game id)
            | None -> false in
          if is_target then (
            t.latest <- {t.latest with target=Some occupant.id};
            publish ~effects:(fun () -> deferred t (fun () -> ignore (Audio.play t.audio "select")))
              ~next:(fun () -> message t "Tap a weapon to attack. Use the magnifying glass for damage details.") t)
          else (
            t.latest <- {t.latest with selected=Some occupant.id; target=None};
            publish ~effects:(fun () -> selection_feedback t tile_id)
              ~next:(fun () -> message t (if occupant.side = Domain.side_enemy then "Enemy unit selected. Select one of your units to act."
                                          else if occupant.attacked then "This unit has finished its turn."
                                          else "Choose a blue hex to move or a red enemy to attack.")) t)
        | None ->
          match t.latest.selected with
          | Some id when Js.Dict.get (Native.reachable t.latest.game id) tile_id <> None ->
            move ~next:(fun () -> message t "Moved. You can attack an enemy in range or select another unit.") t id tile_id
          | _ ->
            t.latest <- {t.latest with selected=None;target=None};
            publish ~effects:(fun () -> selection_feedback t tile_id)
              ~next:(fun () -> let terrain = Catalog.terrain_type t.catalog tile.terrain in
                      message t (terrain.name ^ ": " ^ terrain.description)) t)
let attack t weapon = safely t (fun () -> match t.latest.selected,t.latest.target with
    | Some unit,Some target ->
      require_actor t unit;
      dispatch ~accepted:(fun () -> t.history <- []; t.latest <- {t.latest with target=None})
        ~next:(fun () -> if Js.Nullable.isNullable (state t).winner then message t "Attack complete. Select another unit or end your turn.")
        t (Domain.Command_attack {unit=Native.unit_id unit;target=Native.unit_id target;weapon})
    | _ -> ())
let leader t = Array.find_opt (fun (unit : Domain.public_unit) -> unit.side=Domain.side_player && (Catalog.unit t.catalog unit.type_).leader = Some true) (state t).units
let recruit t type_ tile = safely t (fun () -> match leader t with
    | None -> Js.Exn.raiseError "This unit cannot act now."
    | Some unit -> require_actor t unit.id;
      dispatch ~accepted:(fun () -> t.history <- []; t.latest <- {t.latest with inspected=Some tile})
        ~next:(fun () -> message t "Recruited. Your new unit will act next turn.")
        t (Domain.Command_recruit {unit=Native.unit_id unit.id;type_;tile}))
let advance t type_ = safely t (fun () -> match t.latest.selected with
    | None -> ()
    | Some id -> require_actor t id;
      dispatch ~accepted:(fun () -> t.history <- [])
        ~next:(fun () -> message t ("Advanced to " ^ (Catalog.unit t.catalog type_).name ^ ". Ready next turn."))
        t (Domain.Command_advance {unit=Native.unit_id id;type_}))
let ai_options : Domain.ai_options = {policy=Domain.policy_classic;diplomacy=Domain.diplomacy_restrained;recruitmentLimit=2.}
external rejection_json : Js.Promise.error -> Js.Json.t = "%identity"
let rejection_reason error =
  let value = rejection_json error in
  let text = Option.bind (Js.Json.decodeObject value) (fun object_ -> Option.bind (Js.Dict.get object_ "message") Js.Json.decodeString) in
  Option.value ~default:(Option.value ~default:"Unexpected session failure." (Js.Json.stringifyAny value)) text
let run_enemy t epoch =
  let current () = not t.disposed && epoch=t.epoch in
  let rec step () =
    if not (current ()) then Js.Promise.resolve ()
    else if (state t).turn=Domain.side_enemy && Js.Nullable.isNullable (state t).winner then (
      let before = state t in ignore (Native.enemy_step t.battle ai_options); publish ~effects:(fun () -> feedback t before) t;
      if current () && (state t).turn=Domain.side_enemy && Js.Nullable.isNullable (state t).winner
      then Js.Promise.then_ step (t.clock 230.) else finish ())
    else finish ()
  and finish () =
    if current () then (
      if Js.Nullable.isNullable (state t).winner then message t "Your turn. Capture villages, recruit reinforcements, or attack.";
      if current () then save t false);
    Js.Promise.resolve () in
  if current () then message t {js|The enemy is planning its moves…|js};
  promise (fun () -> Js.Promise.then_ step (t.clock 350.))
  |> Js.Promise.catch (fun error -> if current () then (message t (rejection_reason error); reject error) else Js.Promise.resolve ())
let end_turn t () = promise (fun () ->
    alive t;
    if not (active t) then Js.Exn.raiseError "Wait for your turn.";
    let epoch=t.epoch in
    let before = state t and checkpoint = Native.save t.battle in
    ignore (Native.dispatch t.battle (Domain.Command_endTurn {ignored=None}));
    t.history <- []; t.latest <- {t.latest with selected=None;target=None};
    Save.write t.saves checkpoint ~pending_enemy:true;
    publish ~effects:(fun () -> feedback t before) t;
    run_enemy t epoch)
let initial_focus game =
  let s = Native.state game in
  match Array.find_opt (fun (unit : Domain.public_unit) -> unit.side=Domain.side_player) s.units with
  | Some unit -> unit.tile | None -> if Array.length s.tiles=0 then "" else s.tiles.(0).id
let replace t replacement =
  Audio.cancel t.audio; t.epoch <- t.epoch+1; t.history <- []; t.effects <- [];
  if replacement != t.battle then Native.dispose t.battle;
  t.battle <- replacement; t.pending_enemy <- false; Save.resume t.saves;
  t.latest <- {t.latest with selected=None;target=None;inspected=None;focused=initial_focus (Native.view replacement);
                             status=(if (state t).campaign=None then initial_status else "Follow the mission objectives. Keep your leader safe.")};
  save t false; publish t
let reset t map =
  alive t;
  if not (Array.exists (fun (candidate : Domain.map_definition) -> candidate.id=map) (Catalog.scenario_maps t.catalog)) then
    Js.Exn.raiseError "Unknown battlefield. Choose a skirmish or start a campaign.";
  preload map;
  replace t (Native.create t.factory (Domain.Start_scenario {map;revision=None}) 17.)
let start_campaign t id =
  alive t; let campaign = Catalog.campaign t.catalog id in
  if Array.length campaign.missions > 0 then preload campaign.missions.(0).map.id;
  replace t (Native.create t.factory (Domain.Start_campaign {id}) 17.)
let next_mission t ids =
  alive t;
  ignore (Native.dispatch t.battle (Domain.Command_recallMission {units=Array.map Native.unit_id ids}));
  replace t t.battle
let retry t () = alive t; ignore (Native.dispatch t.battle (Domain.Command_retryMission {ignored=None})); replace t t.battle
let undo t () =
  alive t;
  if Story.dialogue_pending (state t).campaign then Js.Exn.raiseError "Finish the conversation before undoing a move.";
  if can_undo t then match t.history with
    | [] -> ()
    | entry::rest -> Native.replace t.battle entry.checkpoint; t.history <- rest;
      let tile = Option.map (fun (unit : Domain.public_unit) -> unit.tile) (Native.unit (Native.view t.battle) entry.unit) in
      t.latest <- {t.latest with selected=Some entry.unit;target=None;focused=Option.value ~default:t.latest.focused tile;inspected=tile};
      save t false; publish ~effects:(fun () ->
          Option.iter (fun tile -> deferred t (fun () -> Motion.animate_tile tile "move")) tile)
        ~next:(fun () -> message t "Move undone. Choose a new destination.") t
let dialogue t skip =
  alive t;
  if Story.dialogue_pending (state t).campaign then (
    ignore (Native.dispatch t.battle (if skip then Domain.Command_skipDialogue {ignored=None} else Domain.Command_dismissDialogue {ignored=None}));
    if (state t).turn=Domain.side_player || not (Js.Nullable.isNullable (state t).winner) then save t false;
    publish ~render:false t)
let previous_dialogue t () = alive t; match (state t).campaign with
  | Some campaign when Story.dialogue_pending (Some campaign) && campaign.dialogueIndex>0. ->
    ignore (Native.dispatch t.battle (Domain.Command_previousDialogue {ignored=None}));
    if (state t).turn=Domain.side_player || not (Js.Nullable.isNullable (state t).winner) then save t false;
    publish ~render:false t
  | _ -> ()
let set_sound t enabled =
  alive t;
  Audio.set_enabled t.audio enabled; publish ~render:false t;
  Js.Promise.resolve ()
let gesture t trusted = if not t.disposed then ignore (Audio.unlock t.audio ~trusted)
let commands t = Option.get t.actions
let create ?(clock=pause) factory =
  let catalog = Native.catalog factory in
  let loaded = Save.load ~catalog:(Catalog.raw catalog)
      ~restore:(fun checkpoint -> preload checkpoint.Domain.state.mapId; Native.restore factory checkpoint)
      ~fresh:(fun () -> preload "mirror"; Native.create factory (Domain.Start_scenario {map="mirror";revision=None}) 17.)
      ~dispose:Native.dispose ~checkpoint:Native.save in
  try
    let audio = Audio.create () and game=Native.view loaded.game in
    let model = freeze {game;catalog;artwork=Art.current ();selected=None;target=None;inspected=None;
                        focused=initial_focus game;revision=0;status=Option.value ~default:initial_status loaded.notice;
                        sound=Audio.enabled audio;can_undo=false} in
    let t={factory;catalog;audio;saves=loaded.saves;clock;battle=loaded.game;latest=model;
           actions=None;listeners=[];publications=Queue.create ();notifying=false;disposed=false;initialized=false;epoch=0;
           pending_enemy=loaded.pending_enemy;history=[];effects=[]} in
    t.actions <- Some (freeze {choose=choose t;focus=focus t;clear=clear t;attack=attack t;recruit=recruit t;advance=advance t;
                               end_turn=end_turn t;undo=undo t;reset=reset t;start_campaign=start_campaign t;next_mission=next_mission t;
                               retry=retry t;dialogue=dialogue t;previous_dialogue=previous_dialogue t;set_sound=set_sound t});
    t
  with exception_ -> Native.dispose loaded.game; raise exception_
let initialize t =
  alive t;
  if not t.initialized then (
    t.initialized <- true;
    if (state t).turn=Domain.side_player then save t false;
    publish ~render:false t;
    let epoch=t.epoch in
    let observe pending = ignore (Js.Promise.catch (fun error ->
        if not t.disposed && epoch=t.epoch then
          message t (rejection_reason error);
        Js.Promise.resolve ()) pending) in
    if t.pending_enemy then (t.pending_enemy <- false; observe (end_turn t ()))
    else if (state t).turn=Domain.side_enemy && Js.Nullable.isNullable (state t).winner then observe (run_enemy t epoch))
let close t =
  if not t.disposed then (
    t.disposed <- true; t.epoch <- t.epoch+1; t.effects <- []; t.history <- [];
    List.iter (fun listener -> listener.alive <- false) t.listeners; t.listeners <- [];
    Queue.clear t.publications; Audio.cancel t.audio; Native.dispose t.battle)
