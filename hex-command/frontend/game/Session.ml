module Event_action = Domain.Event_action
module Speech = Domain.Speech

type error = Native of Native.error | Rejected of string | Foreign of Foreign_error.cause

exception Failed of error

let error_message = function
  | Native error -> Native.message error
  | Rejected message -> message
  | Foreign cause -> Foreign_error.cause cause

let native = function Ok value -> value | Error error -> raise (Failed (Native error))

module Engine = struct
  let create factory start seed = native (Native.create factory start seed)
  let restore factory saved = native (Native.restore factory saved)
  let dispatch battle command = native (Native.dispatch battle command)
  let prepare_enemy_step battle options = native (Native.prepare_enemy_step battle options)
  let enemy_step_recorded battle options = native (Native.enemy_step_recorded battle options)
  let save battle = native (Native.save battle)
  let save_json battle = native (Native.save_json battle)
  let replace battle saved = native (Native.replace battle saved)
  let validate_undo factory request = native (Native.validate_undo factory request)
end

type model = {
  game : Native.view;
  interaction_game : Native.view;
  session_version : int;
  action : Action_presentation.token option;
  combat_playback : Combat_playback.frame option;
  settling : bool;
  catalog : Catalog.t;
  artwork : Art.t;
  details : Battle_details.t;
  map_focus : Battle_details.focus;
  hud : Hud_details.t;
  treasury : string;
  enemy_combat_details : bool;
  focused : string;
  revision : int;
  status : string;
  sound : bool;
  player_ai : Setup.policy option;
  follow_ai : bool;
  ai_focus : string array;
  can_undo : bool;
  failure : string option;
  setup : Setup.t;
  completed_turns : float;
  persistence : string;
  save_failure : Save_error.t option;
  load_problem : Js.Json.t option;
  awaiting_new_game : bool;
  active_run : string option;
}

type actions = {
  choose : string -> unit;
  focus : string -> unit;
  clear : bool -> unit;
  drop : string -> string -> unit;
  attack : string -> unit;
  recruit : string -> string -> unit;
  advance : string -> unit;
  end_turn : unit -> (unit, error) result Js.Promise.t;
  undo : unit -> unit;
  reset : ?setup:Setup.t -> ?armies:string * string -> string -> unit;
  start_campaign : ?setup:Setup.t -> string -> unit;
  next_mission : string list -> unit;
  retry : unit -> unit;
  dialogue : bool -> unit;
  previous_dialogue : unit -> unit;
  set_sound : bool -> unit Js.Promise.t;
  download_debug_report : unit -> unit;
}

type clock = float -> unit Js.Promise.t
type listener = { mutable alive : bool; receive : model -> unit }
type undo = Save_undo.entry
type phase_origin = Opening | Restored | Player_checkpoint of string

type enemy_task = {
  completion : (unit, error) result Js.Promise.t;
  complete : unit -> unit;
  fail : error -> unit;
}

type t = {
  factory : Native.factory;
  catalog : Catalog.t;
  audio : Phaser_sound.t;
  mutable saves : Save.t;
  clock : clock;
  mutable battle : Native.battle;
  mutable latest : model;
  mutable actions : actions option;
  mutable listeners : listener list;
  publications : model Queue.t;
  mutable notifying : bool;
  mutable disposed : bool;
  mutable initialized : bool;
  mutable battlefield_stopped : bool;
  mutable epoch : int;
  mutable pending_enemy : bool;
  mutable enemy : enemy_task option;
  mutable action_task : Action_presentation.t option;
  mutable playback : Combat_playback.t option;
  mutable presented_combat : Domain.combat_animation option;
  mutable suppress_playback_publish : bool;
  mutable history : undo list;
  mutable effects : (int * int * (unit -> unit)) list;
  mutable last_narration : string option;
  recordings : Debug_report.t;
}

external console_error : exn -> unit = "error" [@@mel.scope "console"]
external reject : 'a -> 'b Js.Promise.t = "reject" [@@mel.scope "Promise"]

let pause milliseconds =
  Js.Promise.make (fun ~resolve ~reject:_ ->
      let _ =
        Browser_time.timeout_float
          (fun () ->
            let completed = () in
            (resolve completed [@u]))
          milliseconds
      in
      ())

let preload catalog map = Terrain_preload.map catalog map
let initial_status = {js|Select a unit. Capture villages. Defeat the red commander.|js}
let state t = Native.state t.latest.interaction_game
let adopt t game = t.latest <- { t.latest with interaction_game = game }

let ensure_open t =
  if t.disposed then raise (Failed (Rejected {js|This battle session has closed.|js}))

let alive t =
  ensure_open t;
  if t.latest.failure <> None then
    raise (Failed (Rejected {js|The battle could not continue. Reload the game.|js}))

let require_session t =
  alive t;
  if Save.awaiting t.saves then raise (Failed (Rejected {js|Choose a new game to continue.|js}))

let active t =
  (not (Save.awaiting t.saves))
  && t.latest.player_ai = None && (not t.battlefield_stopped) && Option.is_none t.enemy
  &&
  let s = state t in
  s.turn = Domain.Turn_phase.Player && Option.is_none s.winner
  && not (Story.dialogue_pending s.campaign)

let require_actor t id =
  alive t;
  if not (active t) then raise (Failed (Rejected {js|Wait for your turn.|js}));
  match Native.unit t.latest.interaction_game id with
  | Some unit when Native.controllable t.latest.interaction_game unit.id -> ()
  | _ -> raise (Failed (Rejected {js|This unit cannot act now.|js}))

let can_undo t =
  match t.history with
  | previous :: _ when Save_undo.round previous = (state t).round && active t -> true
  | _ -> false

let view t = t.latest

let emit t model =
  Queue.push model t.publications;
  if not t.notifying then (
    t.notifying <- true;
    while (not (Queue.is_empty t.publications)) && not t.disposed do
      let next = Queue.pop t.publications in
      let listeners = t.listeners in
      List.iter
        (fun listener ->
          if listener.alive && not t.disposed then
            try listener.receive next with exception_ -> console_error exception_)
        listeners
    done;
    t.notifying <- false;
    if t.disposed then Queue.clear t.publications)

let subscribe t receive =
  ensure_open t;
  let listener = { alive = true; receive } in
  t.listeners <- t.listeners @ [ listener ];
  let cleanup () =
    listener.alive <- false;
    t.listeners <- List.filter (fun other -> other != listener) t.listeners
  in
  (try receive t.latest
   with exception_ ->
     cleanup ();
     raise exception_);
  cleanup

let cancel_enemy ?(reason = Action_presentation.Session_replaced) t =
  Option.iter (fun action -> Action_presentation.cancel action reason) t.action_task;
  t.action_task <- None;
  Option.iter Combat_playback.cancel t.playback;
  t.playback <- None;
  t.presented_combat <- None;
  let previous = t.enemy in
  t.enemy <- None;
  Option.iter (fun task -> task.complete ()) previous

let terminal_failure t failure =
  let reason = Native.message failure in
  if (not t.disposed) && t.latest.failure = None then (
    Option.iter (fun task -> task.fail (Native failure)) t.enemy;
    Option.iter
      (fun action ->
        Action_presentation.fail action (Foreign_error.Exception (Native.Rejected failure)))
      t.action_task;
    cancel_enemy t;
    t.epoch <- t.epoch + 1;
    t.effects <- [];
    t.history <- [];
    t.pending_enemy <- false;
    Save.close t.saves;
    Phaser_sound.close t.audio;
    let model =
      {
        t.latest with
        failure = Some reason;
        status = reason;
        details = Battle_details.None_;
        map_focus = Battle_details.empty_focus;
        hud = Hud_details.closed;
        can_undo = false;
        revision = t.latest.revision + 1;
      }
    in
    t.latest <- model;
    emit t model)

let bind_failure t battle =
  Native.on_failure battle (fun reason -> if battle == t.battle then terminal_failure t reason)

let rec publish ?(render = true) ?(winner = true) ?(effects = fun () -> ()) ?(next = fun () -> ()) t
    =
  if (not t.disposed) && t.latest.failure = None then (
    let current = t.latest.interaction_game in
    let s = Native.state current in
    let status =
      if winner then
        match s.winner with
        | None -> t.latest.status
        | Some side ->
            Option.value
              ~default:
                (if side = Domain.Battle_result.Player_victory then
                   {js|Victory! The enemy commander has fallen. The field is yours.|js}
                 else
                   {js|Your commander has fallen. Start a new battle and try a different approach.|js})
              (Option.bind s.campaign (fun campaign -> campaign.outcome))
      else t.latest.status
    in
    let combat = Native.combat current in
    let same_combat =
      match (combat, t.presented_combat) with
      | None, None -> true
      | Some a, Some b -> a == b
      | _ -> false
    in
    if not same_combat then (
      t.presented_combat <- combat;
      Option.iter Combat_playback.cancel t.playback;
      t.playback <- None;
      Option.iter
        (fun combat ->
          if t.latest.session_version = t.epoch && (Native.state t.latest.game).map_id = s.map_id
          then (
            let before_game = t.latest.game in
            let playback =
              Combat_playback.create ~before_game combat current (fun frame ->
                  if
                    (not t.disposed) && (not t.suppress_playback_publish)
                    && Option.fold ~none:false
                         ~some:(fun playback ->
                           (Combat_playback.frame playback).combat == frame.Combat_playback.combat)
                         t.playback
                  then (
                    publish ~render:false t;
                    if
                      frame.complete
                      && (Native.state frame.game).winner <> None
                      && (Native.state frame.before).winner = None
                    then
                      let _ =
                        Phaser_sound.play
                          ~owner:
                            (match t.action_task with
                            | Some action ->
                                Action_presentation.owner (Action_presentation.token action)
                            | None -> Phaser_sound.combat_owner frame.combat)
                          t.audio
                          (if
                             (Native.state frame.game).winner
                             = Some Domain.Battle_result.Player_victory
                           then {js|victory|js}
                           else {js|defeat|js})
                      in
                      ()))
            in
            t.playback <- Some playback;
            if Option.is_none t.enemy || t.latest.enemy_combat_details then
              t.latest <-
                {
                  t.latest with
                  details =
                    Battle_details.Combat_report
                      {
                        report = Battle_details.report (Combat_playback.frame playback);
                        selected = t.latest.map_focus.selected;
                        tile = t.latest.map_focus.inspected;
                      };
                }))
        combat);
    let combat_playback = Option.map Combat_playback.frame t.playback in
    let playback_pending =
      Option.fold ~none:false
        ~some:(fun frame -> not frame.Combat_playback.complete)
        combat_playback
    in
    let details =
      match (t.latest.details, combat_playback) with
      | Battle_details.Combat_report old, Some frame when old.report.combat == frame.combat ->
          Battle_details.Combat_report { old with report = Battle_details.report frame }
      | details, _ -> details
    in
    let map_focus = Battle_details.focus details current in
    let revision = t.latest.revision + if render then 1 else 0 in
    let game = if playback_pending then (Option.get combat_playback).game else current in
    let hud = Hud_details.resolve t.catalog game details (float_of_int revision) in
    let treasury =
      Presentation.treasury t.catalog game ~selected:map_focus.selected
        ~inspected:map_focus.inspected
    in
    let persistence, save_failure, load_problem, awaiting_new_game, _ = Save.status t.saves in
    let model =
      {
        t.latest with
        details;
        map_focus;
        hud;
        treasury;
        persistence;
        save_failure;
        load_problem;
        awaiting_new_game;
        active_run = Save.active_run t.saves;
        game;
        combat_playback;
        action = Option.map Action_presentation.token t.action_task;
        interaction_game = current;
        session_version = t.epoch;
        settling = Option.is_some t.enemy;
        status;
        sound = Phaser_sound.enabled t.audio;
        player_ai = Setup.player_ai t.latest.setup;
        can_undo = t.latest.failure = None && can_undo t;
        completed_turns = Setup.completed_turns t.latest.setup s;
        revision;
      }
    in
    t.latest <- model;
    if render then t.effects <- [];
    effects ();
    emit t model;
    if (not t.disposed) && t.latest == model then next ())

let message t status =
  if (not t.disposed) && t.latest.failure = None then (
    t.latest <- { t.latest with status };
    publish ~render:false ~winner:false t)

let error t exception_ =
  let reason =
    match Js.Exn.asJsExn exception_ with
    | Some error -> Option.value ~default:{js|Unexpected session failure.|js} (Js.Exn.message error)
    | None -> (
        match exception_ with
        | Failed failure -> error_message failure
        | Native.Rejected failure -> Native.message failure
        | Failure reason | Invalid_argument reason -> reason
        | _ -> {js|Unexpected session failure.|js})
  in
  message t reason

let safely t action = try action () with exception_ -> error t exception_

let promise action =
  try action ()
  with exception_ -> (
    match Js.Exn.asJsExn exception_ with Some error -> reject error | None -> reject exception_)

let deferred t action = t.effects <- (t.epoch, t.latest.revision, action) :: t.effects

let rendered t revision =
  if (not t.disposed) && revision = t.latest.revision then (
    let effects = List.rev t.effects in
    t.effects <- [];
    List.iter
      (fun (epoch, requested, action) ->
        if (not t.disposed) && epoch = t.epoch && requested = t.latest.revision then action ())
      effects)

let persist t pending_enemy =
  Save.set_undo t.saves t.history;
  Save.write_json ~setup:t.latest.setup ~source:(state t) t.saves (Engine.save_json t.battle)
    ~pending_enemy

let save t pending_enemy =
  let current = state t in
  if current.turn = Domain.Turn_phase.Player || current.winner <> None then persist t pending_enemy

let sound_owner t =
  match t.action_task with
  | Some action -> Action_presentation.owner (Action_presentation.token action)
  | None -> Phaser_sound.game_owner t.latest.interaction_game

let feedback t before =
  let changes = Presentation.changes before (state t) in
  List.iter
    (fun (tile, motion) ->
      if motion <> Presentation.Attack || Option.is_none (Native.combat t.latest.interaction_game)
      then
        deferred t (fun () ->
            Motion.animate_tile tile
              (match motion with
              | Presentation.Move -> {js|move|js}
              | Attack -> {js|attack|js}
              | Recruit -> {js|recruit|js})))
    changes.tile_effects;
  List.iter
    (fun tile -> deferred t (fun () -> Motion.animate_tile tile {js|capture|js}))
    changes.captured_tiles;
  if changes.turn_changed then deferred t Motion.animate_turn;
  let combat_pending =
    Option.fold ~none:false
      ~some:(fun playback -> not (Combat_playback.frame playback).complete)
      t.playback
  in
  if (not combat_pending) && (not changes.attacked) && Option.is_none changes.moved_unit then
    match changes.new_winner with
    | Some side ->
        let _ =
          Phaser_sound.play ~owner:(sound_owner t) t.audio
            (if side = Domain.Battle_result.Player_victory then {js|victory|js} else {js|defeat|js})
        in
        ()
    | None when changes.recruited || changes.advanced ->
        let _ = Phaser_sound.play ~owner:(sound_owner t) t.audio {js|recruit|js} in
        ()
    | None -> (
        match changes.captured_tiles with
        | tile :: _ ->
            let terrain =
              Option.map
                (fun (tile : Domain.tile) -> tile.terrain)
                (Native.tile t.latest.interaction_game tile)
            in
            deferred t (fun () ->
                Motion.animate_tile
                  ~sound:{ Phaser_sound.interaction with terrain }
                  tile {js|capture|js})
        | [] -> ())

let supersede_presentation t =
  if Option.is_none t.enemy then
    Option.iter
      (fun playback ->
        t.suppress_playback_publish <- true;
        Fun.protect
          ~finally:(fun () -> t.suppress_playback_publish <- false)
          (fun () -> (Combat_playback.frame playback).skip ());
        let frame = Combat_playback.frame playback in
        match t.latest.details with
        | Battle_details.Combat_report previous when previous.report.combat == frame.combat ->
            t.latest <-
              {
                t.latest with
                details =
                  Battle_details.Combat_report
                    { previous with report = Battle_details.report frame };
              }
        | _ -> ())
      t.playback

let commit ?(feedback_enabled = true) ?(next = fun () -> ()) t before =
  supersede_presentation t;
  save t false;
  publish ~effects:(fun () -> if feedback_enabled then feedback t before) ~next t

let dispatch ?(accepted = fun () -> ()) ?(next = fun () -> ()) t command =
  let before = state t in
  adopt t (Engine.dispatch t.battle command);
  accepted ();
  commit ~next t before

let selection_feedback t tile =
  let unit = Native.at t.latest.interaction_game tile in
  let sound =
    {
      Phaser_sound.interaction with
      owner =
        Some
          (Phaser_sound.named_owner
             (let js_text_part_1 =
                Option.fold ~none:tile ~some:(fun (unit : Domain.public_unit) -> unit.id) unit
              in
              {j|inspection:$(js_text_part_1)|j}));
      unit_type = Option.map (fun (unit : Domain.public_unit) -> unit.type_) unit;
      terrain =
        Option.map
          (fun (tile : Domain.tile) -> tile.terrain)
          (Native.tile t.latest.interaction_game tile);
    }
  in
  deferred t (fun () -> Motion.animate_tile ~sound tile {js|select|js})

let focus t tile =
  alive t;
  if tile <> t.latest.focused && Native.tile t.latest.interaction_game tile <> None then (
    t.latest <- { t.latest with focused = tile };
    publish ~render:false t)

let clear t inspection =
  alive t;
  if
    t.latest.map_focus.selected <> None
    || t.latest.map_focus.target <> None
    || (inspection && t.latest.details <> Battle_details.None_)
  then (
    t.latest <-
      {
        t.latest with
        details =
          (if inspection then Battle_details.None_
           else
             Option.fold ~none:Battle_details.None_
               ~some:(fun tile -> Battle_details.Terrain tile)
               t.latest.map_focus.inspected);
      };
    publish ~next:(fun () -> message t {js|Selection cleared.|js}) t)

let move ?(next = fun () -> ()) t unit tile =
  require_actor t unit;
  let checkpoint = Engine.save t.battle and before = state t in
  let persisted = Save.snapshot ~source:before t.saves (Engine.save_json t.battle) in
  adopt t
    (Engine.dispatch t.battle
       (Domain.Command.Move
          { unit = Native.unit_id unit; tile = Native.hex t.latest.interaction_game tile }));
  if active t then
    t.history <-
      Save_undo.create checkpoint (Save_json.field persisted {js|state|js}) unit before.round
      :: t.history;
  t.latest <- { t.latest with details = Battle_details.Unit unit };
  commit ~next t before

let choose t tile_id =
  safely t (fun () ->
      alive t;
      match Native.tile t.latest.interaction_game tile_id with
      | None -> ()
      | Some tile -> (
          let occupant =
            Option.bind (Native.at t.latest.interaction_game tile_id) (fun unit ->
                if Native.visible t.latest.interaction_game unit then Some unit else None)
          in
          let repeated =
            t.latest.map_focus.inspected <> None
            && (t.latest.map_focus.inspected = Some tile_id
               || Option.fold ~none:false
                    ~some:(fun (unit : Domain.public_unit) ->
                      Some unit.id = t.latest.map_focus.selected
                      || Some unit.id = t.latest.map_focus.target)
                    occupant)
          in
          t.latest <- { t.latest with focused = tile_id };
          if repeated then clear t true
          else if not (active t) then (
            t.latest <-
              {
                t.latest with
                details =
                  (match occupant with
                  | Some unit -> Battle_details.Unit unit.id
                  | None -> Battle_details.Terrain tile_id);
              };
            publish ~render:false ~effects:(fun () -> selection_feedback t tile_id) t)
          else
            match occupant with
            | Some occupant ->
                let is_target =
                  match t.latest.map_focus.selected with
                  | Some id ->
                      Option.fold ~none:false
                        ~some:(fun (unit : Domain.public_unit) ->
                          Native.controllable t.latest.interaction_game unit.id)
                        (Native.unit t.latest.interaction_game id)
                      && Array.exists
                           (fun (unit : Domain.public_unit) -> unit.id = occupant.id)
                           (Native.targets t.latest.interaction_game id)
                  | None -> false
                in
                if is_target then (
                  t.latest <-
                    {
                      t.latest with
                      details =
                        Battle_details.Attack
                          {
                            attacker = Option.get t.latest.map_focus.selected;
                            target = occupant.id;
                          };
                    };
                  publish
                    ~effects:(fun () ->
                      deferred t (fun () ->
                          let _ = Phaser_sound.play t.audio {js|select|js} in
                          ()))
                    ~next:(fun () ->
                      message t
                        {js|Tap a weapon to attack. Use the magnifying glass for damage details.|js})
                    t)
                else (
                  t.latest <- { t.latest with details = Battle_details.Unit occupant.id };
                  publish
                    ~effects:(fun () -> selection_feedback t tile_id)
                    ~next:(fun () ->
                      message t
                        (if not (Native.controllable t.latest.interaction_game occupant.id) then
                           match Native.faction t.latest.interaction_game occupant.id with
                           | Some faction when faction.relation = Domain.Relation.Neutral ->
                               {js|Neutral unit selected.|js}
                           | _ ->
                               {js|AI-controlled unit selected. Select one of your units to act.|js}
                         else if occupant.attack_spent then {js|This unit has finished its turn.|js}
                         else
                           let js_text_part_1 =
                             Js.String.toLowerCase (Catalog.unit t.catalog occupant.type_).name
                           in
                           {j|Selected $(js_text_part_1). Choose a blue hex to move or a red enemy to attack.|j}))
                    t)
            | None -> (
                match t.latest.map_focus.selected with
                | Some id
                  when Option.fold ~none:false
                         ~some:(fun (unit : Domain.public_unit) ->
                           Native.controllable t.latest.interaction_game unit.id)
                         (Native.unit t.latest.interaction_game id)
                       && Domain.Text_map.find_opt tile_id
                            (Native.reachable t.latest.interaction_game id)
                          <> None ->
                    move
                      ~next:(fun () ->
                        message t
                          {js|Moved. You can attack an enemy in range or select another unit.|js})
                      t id tile_id
                | _ ->
                    t.latest <- { t.latest with details = Battle_details.Terrain tile_id };
                    publish
                      ~effects:(fun () -> selection_feedback t tile_id)
                      ~next:(fun () ->
                        let terrain = Catalog.terrain_type t.catalog tile.terrain in
                        message t
                          (if Native.fogged t.latest.interaction_game tile.id then
                             {js|Unexplored terrain.|js}
                           else
                             let js_text_part_0 = terrain.name in
                             let js_text_part_2 = terrain.description in
                             {j|$(js_text_part_0): $(js_text_part_2)|j}))
                      t)))

let attack t weapon =
  safely t (fun () ->
      match (t.latest.map_focus.selected, t.latest.map_focus.target) with
      | Some unit, Some target ->
          require_actor t unit;
          dispatch
            ~accepted:(fun () ->
              t.history <- [];
              t.latest <-
                {
                  t.latest with
                  details =
                    Option.fold ~none:Battle_details.None_
                      ~some:(fun id -> Battle_details.Unit id)
                      t.latest.map_focus.selected;
                })
            ~next:(fun () ->
              if Option.is_none (state t).winner then
                message t {js|Attack complete. Select another unit or end your turn.|js})
            t
            (Domain.Command.Attack
               { unit = Native.unit_id unit; target = Native.unit_id target; weapon })
      | _ -> ())

let drop t unit item =
  safely t (fun () ->
      require_actor t unit;
      dispatch
        ~accepted:(fun () -> t.history <- [])
        ~next:(fun () -> message t {js|Item dropped. Move away so another unit can collect it.|js})
        t
        (Domain.Command.Drop_item { unit = Native.unit_id unit; item }))

let recruit t type_ tile =
  safely t (fun () ->
      match
        Option.bind (Native.tile t.latest.interaction_game tile) (fun tile ->
            Option.bind
              (Presentation.recruitment_context t.catalog t.latest.interaction_game None tile)
              (fun (commander, _) -> commander))
      with
      | None -> raise (Failed (Rejected {js|This unit cannot act now.|js}))
      | Some unit ->
          require_actor t unit.id;
          dispatch
            ~accepted:(fun () ->
              t.history <- [];
              t.latest <- { t.latest with details = Battle_details.Terrain tile })
            ~next:(fun () -> message t {js|Recruited. Your new unit will act next turn.|js})
            t
            (Domain.Command.Recruit
               {
                 unit = Native.unit_id unit.id;
                 kind = type_;
                 tile = Native.hex t.latest.interaction_game tile;
               }))

let advance t type_ =
  safely t (fun () ->
      match t.latest.map_focus.selected with
      | None -> ()
      | Some id ->
          require_actor t id;
          dispatch
            ~accepted:(fun () -> t.history <- [])
            ~next:(fun () ->
              message t
                (let js_text_part_1 = (Catalog.unit t.catalog type_).name in
                 {j|Advanced to $(js_text_part_1). Ready next turn.|j}))
            t
            (Domain.Command.Advance { unit = Native.unit_id id; kind = type_ }))

(* Tools address explicit identities and propagate errors to their caller. The
   UI wrappers above retain selection-based interaction and error presentation. *)
let move_unit t unit tile = move t unit tile

let attack_unit t unit target weapon =
  require_actor t unit;
  let weapon =
    match weapon with
    | Some weapon -> weapon
    | None -> (
        match Native.best_attack t.latest.interaction_game unit target with
        | Some attack -> attack.attack.id
        | None -> raise (Failed (Rejected {js|Illegal attack|js})))
  in
  dispatch
    ~accepted:(fun () ->
      t.history <- [];
      t.latest <-
        {
          t.latest with
          details =
            Option.fold ~none:Battle_details.None_
              ~some:(fun id -> Battle_details.Unit id)
              t.latest.map_focus.selected;
        })
    t
    (Domain.Command.Attack { unit = Native.unit_id unit; target = Native.unit_id target; weapon })

let recruit_unit t unit kind tile =
  require_actor t unit;
  dispatch
    ~accepted:(fun () ->
      t.history <- [];
      t.latest <- { t.latest with details = Battle_details.Terrain tile })
    t
    (Domain.Command.Recruit
       { unit = Native.unit_id unit; kind; tile = Native.hex t.latest.interaction_game tile })

let advance_unit t unit kind =
  require_actor t unit;
  dispatch
    ~accepted:(fun () -> t.history <- [])
    t
    (Domain.Command.Advance { unit = Native.unit_id unit; kind })

let rejection_error error =
  match Js.Exn.anyToExnInternal error with
  | Failed failure -> failure
  | Native.Rejected failure -> Native failure
  | _ -> Foreign (Foreign_error.Rejection error)

let result_promise work =
  let pending =
    try Js.Promise.then_ (fun value -> Js.Promise.resolve (Ok value)) (work ()) with
    | Failed failure -> Js.Promise.resolve (Error failure)
    | Native.Rejected failure -> Js.Promise.resolve (Error (Native failure))
    | error -> Js.Promise.resolve (Error (Foreign (Foreign_error.Exception error)))
  in
  Js.Promise.catch (fun error -> Js.Promise.resolve (Error (rejection_error error))) pending

(* A phase owns one completion and all pacing. Install its identity before any
   notification; cancellation settles callers and leaves stale clocks harmless. *)
let ai_phase t = (state t).turn = Domain.Turn_phase.Ai || t.latest.player_ai <> None

let ai_camera_focus before after command =
  let actors =
    match command with
    | Domain.Command.Move { unit; _ }
    | Domain.Command.Recruit { unit; _ }
    | Domain.Command.Advance { unit; _ } ->
        [ unit ]
    | Domain.Command.Attack { unit; target; _ } -> [ unit; target ]
    | _ -> []
  in
  actors
  |> List.concat_map (fun unit ->
      let id =
        let js_text_part_1 = string_of_int (int_of_float unit) in
        {j|u$(js_text_part_1)|j}
      in
      [ before; after ]
      |> List.filter_map (fun game ->
          Option.map (fun (u : Domain.public_unit) -> Hex.key u.tile) (Native.unit game id)))
  |> List.sort_uniq Stdlib.compare |> Array.of_list

let ensure_enemy t origin =
  if
    t.disposed || t.latest.failure <> None
    || (not (ai_phase t))
    || Save.awaiting t.saves || t.battlefield_stopped
    || Story.dialogue_pending (state t).campaign
    || (state t).winner <> None
  then Js.Promise.resolve (Ok ())
  else
    match t.enemy with
    | Some task -> task.completion
    | None ->
        let complete = ref (fun () -> ()) and fail = ref (fun (_ : error) -> ()) in
        let completion =
          Js.Promise.make (fun ~resolve ~reject:_ ->
              (complete :=
                 fun () ->
                   let completed = Ok () in
                   (resolve completed [@u]));
              fail := fun error -> (resolve (Error error) [@u]))
        in
        let task = { completion; complete = !complete; fail = !fail } in
        t.enemy <- Some task;
        let phase = (state t).turn in
        let player_control = t.latest.player_ai in
        let planning_setup =
          if phase = Domain.Turn_phase.Player then Setup.campaign (Option.get t.latest.player_ai)
          else t.latest.setup
        in
        let options = Setup.ai_options planning_setup in
        let phase_finished () =
          (state t).turn <> phase || (state t).winner <> None
          || (phase = Domain.Turn_phase.Player && t.latest.player_ai <> player_control)
          || Story.dialogue_pending (state t).campaign
        in
        let current () =
          (not t.disposed) && t.latest.failure = None
          && match t.enemy with Some owner -> owner == task | None -> false
        in
        let recording = ref None in
        let finish before =
          Option.iter Debug_report.complete !recording;
          if not (current ()) then Js.Promise.resolve ()
          else (
            save t false;
            Js.Promise.then_
              (fun () ->
                if current () then (
                  t.enemy <- None;
                  t.latest <- { t.latest with ai_focus = [||] };
                  if (state t).winner = None then
                    t.latest <-
                      {
                        t.latest with
                        status =
                          {js|Your turn. Capture villages, recruit reinforcements, or attack.|js};
                      };
                  publish ~effects:(fun () -> feedback t before) t;
                  task.complete ());
                Js.Promise.resolve ())
              (Save.saved t.saves))
        in
        let prepare_during_pause ?(publish_current = fun () -> ()) milliseconds =
          let waiting = t.clock milliseconds in
          (* Start the deadline before publication and preparation. Both complete
           in the existing pacing window; the action still waits for its clock. *)
          let preparation =
            promise (fun () ->
                publish_current ();
                Js.Promise.then_
                  (fun () ->
                    if current () && not (phase_finished ()) then
                      Engine.prepare_enemy_step t.battle options;
                    Js.Promise.resolve ())
                  (Js.Promise.resolve ()))
          in
          Js.Promise.then_
            (fun () -> waiting)
            (Js.Promise.catch
               (fun error -> Js.Promise.then_ (fun () -> reject error) waiting)
               preparation)
        in
        let rec step () =
          if not (current ()) then Js.Promise.resolve ()
          else if phase_finished () then finish (state t)
          else
            let before = state t and observed_before = t.latest.interaction_game in
            let game, command = Engine.enemy_step_recorded t.battle options in
            Option.iter (fun turn -> Debug_report.record turn command) !recording;
            adopt t game;
            t.history <- [];
            if
              phase = Domain.Turn_phase.Player && command = Domain.Command.End_turn
              && (state t).turn = Domain.Turn_phase.Ai
              && (state t).winner = None
            then (
              (* pendingEnemy records a deferred EndTurn from the completed player
                 position, exactly as a human EndTurn does. Never attach that
                 intent to an already-advanced AI-phase checkpoint. *)
              let saved = Engine.save t.battle in
              Save.set_undo t.saves [];
              Save.write ~setup:t.latest.setup t.saves { saved with state = before }
                ~pending_enemy:true);
            t.latest <- { t.latest with ai_focus = ai_camera_focus observed_before game command };
            let action = Action_presentation.create (Native.combat game) in
            t.action_task <- Some action;
            publish ~effects:(fun () -> feedback t before) t;
            Js.Promise.then_
              (fun outcome ->
                (match t.action_task with
                | Some current when current == action -> t.action_task <- None
                | _ -> ());
                match outcome with
                | Action_presentation.Cancelled _ -> Js.Promise.resolve ()
                | Action_presentation.Failed cause -> raise (Failed (Foreign cause))
                | Action_presentation.Completed ->
                    if not (current ()) then Js.Promise.resolve ()
                    else if phase_finished () then finish (state t)
                    else Js.Promise.then_ step (prepare_during_pause 80.))
              (Action_presentation.finished action)
        in
        (* Checkpoint intent belongs to the transition creating this phase, never
         to setup metadata or a round counter. A restored phase keeps its bytes. *)
        let work =
          promise (fun () ->
              (match origin with
              | Opening -> persist t false
              | Restored -> ()
              | Player_checkpoint checkpoint ->
                  Save.write_json ~setup:t.latest.setup t.saves checkpoint ~pending_enemy:true);
              t.latest <-
                {
                  t.latest with
                  status =
                    (if phase = Domain.Turn_phase.Player then {js|AI is playing your turn…|js}
                     else {js|The enemy is planning its moves…|js});
                };
              Js.Promise.then_
                (fun () ->
                  if current () then
                    Js.Promise.then_
                      (fun () ->
                        if current () then (
                          recording :=
                            Some
                              (Debug_report.begin_turn t.recordings (Engine.save_json t.battle)
                                 planning_setup);
                          step ())
                        else Js.Promise.resolve ())
                      (prepare_during_pause 0.)
                  else Js.Promise.resolve ())
                (Save.saved t.saves))
        in
        let _ =
          Js.Promise.catch
            (fun error ->
              if current () then (
                (* A failed phase retains its settled completion until replacement
               or terminal cancellation; initialize cannot restart partial AI. *)
                let failure = rejection_error error in
                task.fail failure;
                message t (error_message failure));
              Js.Promise.resolve ())
            work
        in
        (* Background phases and explicit end_turn callers share this promise. *) completion

let continue_ai t =
  if t.initialized && t.latest.failure = None && not t.disposed then
    ignore (ensure_enemy t Restored)

let set_player_ai t policy =
  alive t;
  if (state t).turn = Domain.Turn_phase.Player then (
    t.latest <-
      { t.latest with player_ai = policy; setup = Setup.with_player_ai t.latest.setup policy };
    if t.initialized && not (Save.awaiting t.saves) then persist t false;
    publish t)

let set_follow_ai t enabled =
  alive t;
  t.latest <- { t.latest with follow_ai = enabled };
  publish t

let end_turn ?(restored = false) t () =
  let pending =
    result_promise (fun () ->
        alive t;
        if
          (not (active t))
          && not
               (restored
               && (state t).turn = Domain.Turn_phase.Player
               && t.enemy = None && (state t).winner = None
               && not (Story.dialogue_pending (state t).campaign))
        then raise (Failed (Rejected {js|Wait for your turn.|js}));
        supersede_presentation t;
        let before = state t and checkpoint = Engine.save_json t.battle in
        adopt t (Engine.dispatch t.battle Domain.Command.End_turn);
        t.pending_enemy <- false;
        t.history <- [];
        t.latest <-
          {
            t.latest with
            details =
              Option.fold ~none:Battle_details.None_
                ~some:(fun tile -> Battle_details.Terrain tile)
                t.latest.map_focus.inspected;
          };
        let after = state t in
        let completion =
          if after.turn = Domain.Turn_phase.Ai && after.winner = None then
            ensure_enemy t (Player_checkpoint checkpoint)
          else (
            save t false;
            Js.Promise.resolve (Ok ()))
        in
        publish ~effects:(fun () -> feedback t before) t;
        completion)
  in
  Js.Promise.then_
    (function
      | Error error -> Js.Promise.resolve (Error error) | Ok outcome -> Js.Promise.resolve outcome)
    pending

let initial_focus game =
  let s = Native.state game in
  match
    Array.find_opt (fun (unit : Domain.public_unit) -> unit.faction = {js|player|js}) s.units
  with
  | Some unit -> Hex.key unit.tile
  | None -> if Array.length s.tiles = 0 then {js||js} else s.tiles.(0).id

let replace ?setup ?(new_run = true) ?(persist = true) t replacement game =
  cancel_enemy t;
  if new_run then Save.new_run t.saves else Save.invalidate t.saves;
  if persist then Save.chapter_start t.saves;
  Phaser_sound.cancel t.audio;
  t.epoch <- t.epoch + 1;
  t.history <- [];
  t.effects <- [];
  let previous = t.battle in
  t.battle <- replacement;
  adopt t game;
  bind_failure t replacement;
  (if previous != replacement then try Native.dispose previous with error -> console_error error);
  alive t;
  t.pending_enemy <- false;
  Debug_report.clear t.recordings;
  Save.resume t.saves;
  t.latest <-
    {
      t.latest with
      setup = Option.value ~default:t.latest.setup setup;
      player_ai = Setup.player_ai (Option.value ~default:t.latest.setup setup);
      ai_focus = [||];
      details = Battle_details.None_;
      focused = initial_focus game;
      status =
        (if (state t).campaign = None then initial_status
         else {js|Follow the mission objectives. Keep your leader safe.|js});
    };
  if persist then save t false;
  (if persist then
     let _ = ensure_enemy t Opening in
     ());
  if persist then publish t

let reset ?(setup = Setup.default) ?armies t map =
  alive t;
  if
    not
      (List.exists
         (fun (candidate : Domain.browser_map) -> candidate.id = map)
         (Catalog.scenario_maps t.catalog))
  then raise (Failed (Rejected {js|Unknown battlefield. Choose a skirmish or start a campaign.|js}));
  preload t.catalog map;
  let armies = Option.map (fun (player, enemy) -> [| player; enemy |]) armies in
  let replacement =
    Engine.create t.factory
      (Domain.Start.Scenario { map; revision = None; armies; first = Some (Setup.first setup) })
      17.
  in
  replace ~setup t replacement (Native.view replacement)

let start_campaign ?(setup = Setup.default) t id =
  alive t;
  let campaign = Catalog.campaign t.catalog id in
  if Array.length campaign.missions > 0 then preload t.catalog campaign.missions.(0).map.id;
  let replacement = Engine.create t.factory (Domain.Start.Campaign { id }) 17. in
  replace ~setup t replacement (Native.view replacement)

let next_mission t ids =
  require_session t;
  adopt t
    (Engine.dispatch t.battle
       (Domain.Command.Recall_mission { units = Array.of_list (List.map Native.unit_id ids) }));
  replace ~new_run:false t t.battle t.latest.interaction_game

let retry t () =
  require_session t;
  let current = state t in
  match current.campaign with
  | None ->
      let armies = Option.map (fun armies -> (armies.(0), armies.(1))) current.armies in
      reset ~setup:t.latest.setup ?armies t current.map_id
  | Some _ ->
      adopt t (Engine.dispatch t.battle Domain.Command.Retry_mission);
      replace ~new_run:false t t.battle t.latest.interaction_game

let undo t () =
  require_session t;
  if Story.dialogue_pending (state t).campaign then
    raise (Failed (Rejected {js|Finish the conversation before undoing a move.|js}));
  if can_undo t then
    match t.history with
    | [] -> ()
    | entry :: rest ->
        adopt t (Engine.replace t.battle (Save_undo.checkpoint entry));
        t.history <- rest;
        let tile =
          Option.map
            (fun (unit : Domain.public_unit) -> Hex.key unit.tile)
            (Native.unit t.latest.interaction_game (Save_undo.unit_id entry))
        in
        t.latest <-
          {
            t.latest with
            details = Battle_details.Unit (Save_undo.unit_id entry);
            focused = Option.value ~default:t.latest.focused tile;
          };
        save t false;
        publish
          ~effects:(fun () ->
            Option.iter
              (fun tile -> deferred t (fun () -> Motion.animate_tile tile {js|move|js}))
              tile)
          ~next:(fun () -> message t {js|Move undone. Choose a new destination.|js})
          t

let require_settled t =
  if Option.is_some t.enemy then
    raise (Failed (Rejected {js|Wait for the turn to finish saving.|js}))

let dialogue t skip =
  require_session t;
  if Story.dialogue_pending (state t).campaign then (
    require_settled t;
    adopt t
      (Engine.dispatch t.battle
         (if skip then Domain.Command.Skip_dialogue else Domain.Command.Dismiss_dialogue));
    if (state t).turn = Domain.Turn_phase.Player || not (Option.is_none (state t).winner) then
      save t false;
    publish ~render:false t)

let previous_dialogue t () =
  require_session t;
  match (state t).campaign with
  | Some campaign when Story.dialogue_pending (Some campaign) && campaign.dialogue_index > 0. ->
      require_settled t;
      adopt t (Engine.dispatch t.battle Domain.Command.Previous_dialogue);
      if (state t).turn = Domain.Turn_phase.Player || not (Option.is_none (state t).winner) then
        save t false;
      publish ~render:false t
  | _ -> ()

let set_sound t enabled =
  alive t;
  Phaser_sound.set_enabled t.audio enabled;
  publish ~render:false t;
  Js.Promise.resolve ()

let gesture t trusted =
  if not t.disposed then
    let _ = Phaser_sound.unlock t.audio ~trusted in
    ()

let commands t = Option.get t.actions
let prepare_audio t = Phaser_sound.prepare_hud t.audio
let menu_cue ?cue t = Phaser_sound.play_hud ?cue t.audio

let from_loaded ?(clock = pause) ~audio factory (loaded : Native.battle Save.loaded) =
  let catalog = Native.catalog factory in
  try
    let game = Native.view loaded.game in
    let model =
      {
        game;
        interaction_game = game;
        session_version = 0;
        action = None;
        combat_playback = None;
        settling = false;
        enemy_combat_details = false;
        treasury = {js|player|js};
        catalog;
        artwork = Art.current ();
        details = Battle_details.None_;
        map_focus = Battle_details.empty_focus;
        hud = Hud_details.closed;
        focused = initial_focus game;
        revision = 0;
        status =
          Option.value
            ~default:
              (if Option.is_some (Native.state game).campaign then
                 {js|Follow the mission objectives. Keep your leader safe.|js}
               else initial_status)
            loaded.notice;
        sound = Phaser_sound.enabled audio;
        player_ai = Setup.player_ai loaded.setup;
        follow_ai = true;
        ai_focus = [||];
        can_undo = false;
        failure = None;
        persistence = {js|Not saved yet|js};
        save_failure = None;
        load_problem = Save.problem loaded.saves;
        awaiting_new_game = Save.awaiting loaded.saves;
        active_run = Save.active_run loaded.saves;
        setup = loaded.setup;
        completed_turns = Setup.completed_turns loaded.setup (Native.state game);
      }
    in
    let t =
      {
        factory;
        catalog;
        audio;
        saves = loaded.saves;
        clock;
        battle = loaded.game;
        latest = model;
        actions = None;
        listeners = [];
        publications = Queue.create ();
        notifying = false;
        disposed = false;
        initialized = false;
        battlefield_stopped = false;
        epoch = 0;
        pending_enemy = loaded.pending_enemy;
        enemy = None;
        action_task = None;
        playback = None;
        presented_combat = None;
        suppress_playback_publish = false;
        history = loaded.undo;
        effects = [];
        last_narration = None;
        recordings = Debug_report.create ();
      }
    in
    t.actions <-
      Some
        {
          choose = choose t;
          focus = focus t;
          clear = clear t;
          drop = drop t;
          attack = attack t;
          recruit = recruit t;
          advance = advance t;
          end_turn = end_turn t;
          undo = undo t;
          reset = (fun ?setup ?armies map -> reset ?setup ?armies t map);
          start_campaign = (fun ?setup id -> start_campaign ?setup t id);
          next_mission = next_mission t;
          retry = retry t;
          dialogue = dialogue t;
          previous_dialogue = previous_dialogue t;
          set_sound = set_sound t;
          download_debug_report =
            (fun () ->
              safely t (fun () ->
                  alive t;
                  let checkpoint = Engine.save_json t.battle
                  and setup = t.latest.setup
                  and state = state t in
                  let _ =
                    Js.Promise.then_
                      (fun browser_saves ->
                        Debug_report.download ~browser_saves t.recordings checkpoint setup state;
                        message t {js|Debug report downloaded. You can attach it to the chat.|js};
                        Js.Promise.resolve ())
                      (Save.capture ())
                  in
                  ()));
        };
    bind_failure t loaded.game;
    Save.subscribe t.saves (fun () -> publish ~render:false t);
    publish ~render:false t;
    t
  with exception_ ->
    (try Native.dispose loaded.game with _ -> ());
    raise exception_

let initialize t =
  alive t;
  if not t.initialized then (
    t.initialized <- true;
    if t.pending_enemy then (
      t.pending_enemy <- false;
      let _ = end_turn ~restored:true t () in
      ())
    else (
      save t false;
      let _ = ensure_enemy t Restored in
      publish ~render:false t))

let close t =
  if not t.disposed then (
    cancel_enemy ~reason:Action_presentation.Disposed t;
    t.disposed <- true;
    t.epoch <- t.epoch + 1;
    t.effects <- [];
    t.history <- [];
    List.iter (fun listener -> listener.alive <- false) t.listeners;
    t.listeners <- [];
    Queue.clear t.publications;
    Save.close t.saves;
    Phaser_sound.close t.audio;
    Native.dispose t.battle)

let scene_audio t =
  let open Domain in
  let state = state t in
  let rec speech nodes =
    Array.to_list nodes
    |> List.concat_map (function
      | Speech.Line line -> Option.to_list line.sound
      | Speech.Branch branch -> speech branch.dialogue @ speech branch.alternative)
  in
  let rec actions values =
    Array.to_list values
    |> List.concat_map (function
      | Event_action.Sound sound -> [ sound ]
      | Event_action.Say nodes -> speech nodes
      | Event_action.For_each_unit loop -> actions loop.actions
      | Event_action.For_each_mark loop -> actions loop.actions
      | _ -> [])
  in
  let narrative =
    match state.campaign with
    | None -> []
    | Some progress ->
        let campaign = Catalog.campaign t.catalog progress.id in
        let mission = campaign.missions.(int_of_float progress.mission) in
        speech mission.intro @ speech mission.victory
        @ (if int_of_float progress.mission = Array.length campaign.missions - 1 then
             speech campaign.ending
           else [])
        @ (Array.to_list mission.scripted_events
          |> List.concat_map (fun (event : script_event) -> actions event.actions))
  in
  let items = Array.to_list (Catalog.raw t.catalog).items in
  let item_sounds =
    List.concat_map
      (fun (item : wiki_item) ->
        (if item.id = {js|quarry-hammer|js} then [ {js|attack-melee|js} ] else [])
        @ (Array.to_list item.own_attacks |> List.map (fun (attack : wiki_attack) -> attack.sound)))
      items
  in
  Phaser_sound.prepare t.audio (Array.of_list (narrative @ item_sounds))

let narrate t view =
  let open Domain in
  let state = Native.state view in
  Option.iter
    (fun (progress : campaign_state) ->
      let index = int_of_float progress.dialogue_index in
      if index < Array.length progress.dialogue then
        Option.iter
          (fun sound ->
            let key =
              let js_text_part_0 = string_of_int t.epoch in
              let js_text_part_2 = progress.id in
              let js_text_part_4 = string_of_int index in
              let js_text_part_6 = string_of_float progress.mission in
              {j|$(js_text_part_0):$(js_text_part_2):$(js_text_part_4):$(js_text_part_6):$(sound)|j}
            in
            if t.last_narration <> Some key then (
              t.last_narration <- Some key;
              let _ = Phaser_sound.play t.audio sound in
              ()))
          progress.dialogue.(index).sound)
    state.campaign

let load factory =
  Save.load ~restore:(Engine.restore factory) ~validate:(Engine.validate_undo factory)
    ~fresh:(fun () ->
      Engine.create factory
        (Domain.Start.Scenario
           { map = {js|mirror|js}; revision = None; armies = None; first = None })
        17.)
    ~dispose:Native.dispose

let create_with_audio ?clock audio factory =
  Js.Promise.catch
    (fun error ->
      Phaser_sound.close audio;
      reject error)
    (Js.Promise.then_
       (fun loaded -> Js.Promise.resolve (from_loaded ?clock ~audio factory loaded))
       (load factory))

let create_async ?clock factory = create_with_audio ?clock (Phaser_sound.create ()) factory
let saved t = Save.saved t.saves

let saved_progress t =
  alive t;
  Save.list t.saves

let saved_flash t =
  alive t;
  Save.flash t.saves

let restore_text t raw kind =
  Save.restore_text ~restore:(Engine.restore t.factory) ~validate:(Engine.validate_undo t.factory)
    raw kind

let save_game t =
  promise (fun () ->
      alive t;
      if
        Save.awaiting t.saves || Option.is_some t.enemy
        || ((state t).turn = Domain.Turn_phase.Ai && (state t).winner = None)
      then Save_error.raise_internal {js|Wait for the enemy turn to finish before saving.|js};
      let checkpoint = Engine.save_json t.battle in
      let work = Save.manual t.saves t.latest.setup checkpoint in
      Js.Promise.catch
        (fun error -> reject error)
        (Js.Promise.then_
           (fun () ->
             persist t false;
             Js.Promise.then_
               (fun () ->
                 let _, failure, _, _, _ = Save.status t.saves in
                 match failure with
                 | None -> Js.Promise.resolve ()
                 | Some failure -> Save_error.reject failure)
               (Save.saved t.saves))
           work))

let summarize t game =
  let open Save_json in
  let view = Native.view game in
  let state = Native.state view in
  let armies =
    {js|player|js}
    :: (Array.to_list state.factions
       |> List.filter_map (fun (f : Domain.faction) ->
           if f.controller = Domain.Controller.Human then Some f.id else None))
    |> List.sort_uniq Stdlib.compare
  in
  let levels = Hashtbl.create 4 in
  List.iter (fun level -> Hashtbl.replace levels level 0) [ 1; 2; 3 ];
  Array.iter
    (fun (unit : Domain.public_unit) ->
      if Native.controllable view unit.id then
        let level = int_of_float (Catalog.unit t.catalog unit.type_).level in
        Hashtbl.replace levels level (1 + Option.value ~default:0 (Hashtbl.find_opt levels level)))
    state.units;
  let villages =
    Array.to_list state.tiles
    |> List.filter (fun (tile : Domain.tile) ->
        Catalog.terrain_role t.catalog tile.terrain = Domain.Terrain_role.Village)
  in
  let goals =
    match (Native.inspection view).objectives with
    | None -> None
    | Some objectives ->
        Some
          (Array.append objectives.goals objectives.optional
          |> Array.to_list
          |> List.filter (fun (o : Domain.objective_status) -> o.visible))
  in
  obj
    [
      ( {js|gold|js},
        num (List.fold_left (fun total id -> total +. Native.gold state.gold id) 0. armies) );
      ( {js|villages|js},
        obj
          [
            ( {js|owned|js},
              num
                (float_of_int
                   (List.length
                      (List.filter
                         (fun (tile : Domain.tile) ->
                           Option.fold ~none:false ~some:(fun id -> List.mem id armies) tile.owner)
                         villages))) );
            ({js|total|js}, num (float_of_int (List.length villages)));
          ] );
      ( {js|objectives|js},
        match goals with
        | None -> Js.Json.null
        | Some goals ->
            obj
              [
                ( {js|completed|js},
                  num
                    (float_of_int
                       (List.length
                          (List.filter
                             (fun (o : Domain.objective_status) -> o.progress.done_)
                             goals))) );
                ({js|total|js}, num (float_of_int (List.length goals)));
              ] );
      ( {js|units|js},
        arr
          (Hashtbl.fold (fun level count result -> (level, count) :: result) levels []
          |> List.sort compare
          |> List.map (fun (level, count) ->
              obj
                [
                  ({js|level|js}, num (float_of_int level));
                  ({js|count|js}, num (float_of_int count));
                ])) );
    ]

let saved_summary t id =
  Save.inspect_progress t.saves
    ~current:(fun () -> ensure_open t)
    ~restore:(Engine.restore t.factory) ~validate:(Engine.validate_undo t.factory)
    ~dispose:Native.dispose ~inspect:(summarize t) id

let install_restored t game setup undo pending =
  replace ~setup ~new_run:false ~persist:false t game (Native.view game);
  Save.position t.saves (state t).map_id (state t).round;
  t.history <- undo;
  t.latest <-
    {
      t.latest with
      status = {js|Saved progress restored. Recorded turn starts remain in Saved games.|js};
    };
  publish t;
  if pending then
    let _ = end_turn ~restored:true t () in
    ()
  else
    let _ = ensure_enemy t Restored in
    ()

let restore_checkpoint ?(reload = false) t id =
  let epoch = t.epoch in
  let current () =
    ensure_open t;
    if epoch <> t.epoch then raise (Failed (Rejected {js|This play session has changed.|js}))
  in
  Save.restore_progress t.saves ~current ~restore:(Engine.restore t.factory)
    ~validate:(Engine.validate_undo t.factory) ~dispose:Native.dispose
    ~install:(fun game setup ->
      if reload then Native.dispose game else install_restored t game setup [] false)
    id

let restore_progress t id = restore_checkpoint t id

let save_after_cleanup t =
  if Save.awaiting t.saves || ((state t).turn = Domain.Turn_phase.Ai && (state t).winner = None)
  then Js.Promise.resolve ()
  else
    Js.Promise.then_
      (function
        | Error failure ->
            Save.failed t.saves (Save_error.exception_ failure);
            Js.Promise.resolve ()
        | Ok () ->
            persist t false;
            Save.saved t.saves)
      (Save.retry_pending Save.After_cleanup t.saves)

let delete_saved_progress t id =
  Js.Promise.then_ (fun () -> save_after_cleanup t) (Save.delete_progress t.saves id)

let delete_saved_run t id =
  let active = Save.active_run t.saves = Some id in
  Js.Promise.then_
    (fun () ->
      if active then (
        cancel_enemy t;
        t.epoch <- t.epoch + 1;
        t.history <- [];
        t.effects <- [];
        t.pending_enemy <- false;
        Phaser_sound.cancel t.audio;
        Save.mark_idle t.saves;
        t.latest <- { t.latest with details = Battle_details.None_ };
        publish ~render:false t;
        Js.Promise.resolve ())
      else save_after_cleanup t)
    (Save.delete_run t.saves id)

let repair_inventory t =
  Save.inspect t.saves (fun save ->
      let kind =
        Save_json.text
          (Save_json.field (Save_json.field save {js|progress|js}) {js|entry|js})
          {js|kind|js}
      in
      let game, _, _, _ = restore_text t (Js.Json.stringify save) kind in
      Native.dispose game)

let repair_saved_data t target = Save.repair t.saves target

let retry_saved_progress t expected =
  Save.queue t.saves (fun () ->
      ensure_open t;
      let open Save_json in
      let* loaded = load t.factory in
      (try ensure_open t
       with error ->
         Native.dispose loaded.game;
         raise error);
      if Save.problem loaded.saves <> None then (
        Native.dispose loaded.game;
        Save.close t.saves;
        t.saves <- loaded.saves;
        Save.subscribe t.saves (fun () -> publish ~render:false t);
        publish ~render:false t;
        return false)
      else
        let actual = loaded.entry in
        let normalized v =
          match v with
          | None -> {js|null|js}
          | Some v ->
              Js.Json.stringify
                (obj
                   (Array.to_list (entries v) |> List.sort (fun (a, _) (b, _) -> Stdlib.compare a b)))
        in
        if
          Option.fold ~none:false
            ~some:(fun expected -> normalized actual <> normalized expected)
            expected
        then (
          Native.dispose loaded.game;
          Save_error.reject_internal
            {js|Saved progress changed in another tab. Review the latest details before continuing.|js})
        else (
          Save.close t.saves;
          t.saves <- loaded.saves;
          Save.subscribe t.saves (fun () -> publish ~render:false t);
          if actual = None then (
            Native.dispose loaded.game;
            cancel_enemy t;
            t.epoch <- t.epoch + 1;
            Save.mark_idle t.saves;
            publish ~render:false t)
          else install_restored t loaded.game loaded.setup loaded.undo loaded.pending_enemy;
          return true))

let renderer_failed t message =
  if (not t.disposed) && not t.battlefield_stopped then (
    t.battlefield_stopped <- true;
    let cause = Foreign_error.Exception (Failure message) in
    Option.iter (fun action -> Action_presentation.fail action cause) t.action_task;
    Option.iter (fun task -> task.fail (Foreign cause)) t.enemy;
    t.enemy <- None;
    Option.iter Combat_playback.cancel t.playback;
    Phaser_sound.cancel t.audio;
    t.effects <- [];
    publish ~render:false t)

let prepare_reload t source =
  promise (fun () ->
      ensure_open t;
      Js.Promise.then_
        (function
          | Error message -> raise (Failed (Rejected message))
          | Ok None -> Js.Promise.resolve ()
          | Ok (Some id) -> restore_checkpoint ~reload:true t id)
        (Save.reload_target t.saves source))

let trace_context t =
  let open Save_json in
  let state = state t and presentation = Native.presentation t.latest.game in
  (* save_json already returns host JSON text; decoding it as native bytes
     corrupts Unicode checkpoints. *)
  let serialized = Engine.save_json t.battle |> Js.Json.parseExn in
  let hex (hex : Domain.tile_id) =
    obj [ ({js|col|js}, num (Int32.to_float hex.col)); ({js|row|js}, num (Int32.to_float hex.row)) ]
  in
  let light (light : Domain.battlefield_light) =
    obj
      ([
         ({js|col|js}, num (Int32.to_float light.col));
         ({js|row|js}, num (Int32.to_float light.row));
         ({js|radius|js}, num light.radius);
         ({js|power|js}, num light.power);
       ]
      @ Option.fold ~none:[] ~some:(fun value -> [ ({js|dx|js}, num value) ]) light.dx
      @ Option.fold ~none:[] ~some:(fun value -> [ ({js|dy|js}, num value) ]) light.dy
      @ Option.fold ~none:[] ~some:(fun value -> [ ({js|fixture|js}, bool value) ]) light.fixture
      @ Option.fold ~none:[]
          ~some:(fun value ->
            [
              ( {js|color|js},
                Js.Json.array (Array.map (fun value -> num (float_of_int value)) value) );
            ])
          light.color)
  in
  obj
    [
      ({js|mapId|js}, str state.map_id);
      ({js|cols|js}, num (Int32.to_float state.cols));
      ({js|rows|js}, num (Int32.to_float state.rows));
      ({js|round|js}, num state.round);
      ({js|turn|js}, str (Enum.string_of_phase state.turn));
      ({js|snapshot|js}, field serialized {js|state|js});
      ( {js|lighting|js},
        obj
          [
            ({js|unlitTiles|js}, Js.Json.array (Array.map hex presentation.lighting.unlit_tiles));
            ({js|lights|js}, Js.Json.array (Array.map light presentation.lighting.lights));
          ] );
      ({js|fog|js}, Js.Json.array (Array.map hex presentation.fog));
    ]

let set_enemy_combat_details t enabled =
  if (not t.disposed) && t.latest.enemy_combat_details <> enabled then (
    t.latest <- { t.latest with enemy_combat_details = enabled };
    publish ~render:false t)
