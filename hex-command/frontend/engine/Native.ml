external module_url : unit -> string = "default_source" [@@mel.module "@wasm/bindings.runtime.mjs"]

type error =
  | Engine of Domain.Engine_error.t
  | Initialization of Domain.Wasm.Load_error.t
  | Module_failed of Domain.Wasm.Failure.t
  | Disposed
  | Invalid_request of string
  | Publication of Foreign_error.cause

(* Synchronous unwinding preserves typed failures while releasing acquired
   owners. Fallible operations return Result; total owner queries use Rejected
   for lifecycle guards. *)
exception Rejected of error

let error_message = function
  | Domain.Engine_error.Invalid_request { message }
  | Domain.Engine_error.Invalid_save { message }
  | Domain.Engine_error.Content { message; _ } ->
      message
  | Domain.Engine_error.Unknown_map { map } -> {j|Unknown map: $(map)|j}
  | Domain.Engine_error.Unknown_campaign { id } -> {j|Unknown campaign: $(id)|j}
  | Domain.Engine_error.Unknown_battle -> {js|unknownBattle|js}
  | Domain.Engine_error.Disposed -> {js|disposed|js}
  | Domain.Engine_error.Stale_revision _ -> {js|staleRevision|js}
  | Domain.Engine_error.Command { error } -> (
      match error with
      | Domain.Command_error.Wrong_turn -> {js|WrongTurn|js}
      | Domain.Command_error.Battle_finished -> {js|BattleFinished|js}
      | Domain.Command_error.Unknown_unit id ->
          let js_text_part_1 = string_of_float id in
          {j|Unknown unit: $(js_text_part_1)|j}
      | Domain.Command_error.Unknown_tile id ->
          let js_text_part_1 = Hex.key id in
          {j|Unknown tile: $(js_text_part_1)|j}
      | Domain.Command_error.Unknown_type id -> {j|Unknown unit type: $(id)|j}
      | Domain.Command_error.Unknown_weapon id -> {j|Unknown weapon: $(id)|j}
      | Domain.Command_error.No_campaign -> {js|NoCampaign|js}
      | Domain.Command_error.No_next_mission -> {js|NoNextMission|js}
      | Domain.Command_error.Illegal_move -> {js|IllegalMove|js}
      | Domain.Command_error.Illegal_attack -> {js|IllegalAttack|js}
      | Domain.Command_error.Illegal_recruit -> {js|IllegalRecruit|js}
      | Domain.Command_error.Illegal_advance -> {js|IllegalAdvance|js}
      | Domain.Command_error.Illegal_item -> {js|IllegalItem|js}
      | Domain.Command_error.Illegal_intent -> {js|IllegalIntent|js}
      | Domain.Command_error.Battle_only_policy -> {js|BattleOnlyPolicy|js})

let message = function
  | Engine error -> error_message error
  | Initialization error -> Binding_text.to_js (Domain.Wasm.Load_error.message error)
  | Module_failed failure -> Binding_text.to_js failure.message
  | Disposed -> {js|disposed|js}
  | Invalid_request message -> message
  | Publication cause -> Foreign_error.cause cause

let value = function Ok value -> value | Error error -> raise (Rejected (Engine error))

let protect work =
  try Ok (work ()) with
  | Rejected error -> Error error
  | error -> Error (Publication (Foreign_error.Exception error))

external create_dictionary : unit Js.Nullable.t -> 'a Js.Dict.t = "create" [@@mel.scope "Object"]
external console_error : exn -> unit = "error" [@@mel.scope "console"]

let failure_reason = function
  | Rejected error -> error
  | error -> Publication (Foreign_error.Exception error)

let index key values =
  let result = create_dictionary Js.Nullable.null in
  Array.iter (fun entry -> Js.Dict.set result (key entry) entry) values;
  result

let tile_indices tiles =
  let result = create_dictionary Js.Nullable.null in
  Array.iteri (fun i (tile : Domain.tile) -> Js.Dict.set result tile.id i) tiles;
  result

(* Geometry and its frontend lookup representation have the same map lifetime. *)
type topology = { neighbors : string array Js.Dict.t }

type view = {
  revision : float;
  combat : Domain.combat_animation option;
  state : Domain.state;
  presentation : Domain.battlefield_presentation;
  fog_index : bool Js.Dict.t;
  visible_ground_items : Domain.ground_item array;
  visible_units : Domain.public_unit array;
  displayed_campaign : Domain.campaign_state option;
  inspection : Domain.battle_inspection;
  tiles : Domain.tile Js.Dict.t;
  tile_indices : int Js.Dict.t;
  units : Domain.public_unit Js.Dict.t;
  occupancy : Domain.public_unit Js.Dict.t;
  choices : Domain.unit_actions Js.Dict.t;
  neighbors : string array Js.Dict.t;
}

and factory = {
  handle : Domain.NativeEngineFactory.t;
  catalog : Catalog.t;
  mutable topologies : ((string * float) * topology) list;
  mutable disposed : bool;
  mutable battles : battle list;
}

and battle = {
  factory : factory;
  mutable id : float;
  mutable current : view;
  mutable released : bool;
  mutable failed : (error -> unit) option;
}

(* All factories use the generated module's terminal trap state. Mark the
   entire ownership graph before callbacks, which may synchronously dispose it. *)
type 'a weak

external weak : 'a -> 'a weak = "WeakRef" [@@mel.new]
external deref : 'a weak -> 'a Js.Undefined.t = "deref" [@@mel.send]

let factories : factory weak list ref = ref []

let registered () =
  List.filter_map (fun reference -> Js.Undefined.toOption (deref reference)) !factories

let notify receive reason =
  Option.iter (fun receive -> try receive reason with failure -> console_error failure) receive

let invalidate reason =
  let affected = registered () in
  factories := [];
  let receivers =
    List.concat_map
      (fun factory ->
        factory.disposed <- true;
        let battles = factory.battles in
        factory.battles <- [];
        factory.topologies <- [];
        List.map
          (fun battle ->
            battle.released <- true;
            let receive = battle.failed in
            battle.failed <- None;
            receive)
          battles)
      affected
  in
  List.iter (fun receive -> notify receive reason) receivers

let check_failure () =
  match Domain.Wasm.failure () with
  | None -> ()
  | Some reason ->
      invalidate (Module_failed reason);
      raise (Rejected (Module_failed reason))

let foreign action =
  try action ()
  with error -> (
    match Domain.Wasm.failure () with
    | None -> Fault.raise_original error
    | Some reason ->
        invalidate (Module_failed reason);
        raise (Rejected (Module_failed reason)))

let ensure_factory factory =
  if factory.disposed then raise (Rejected Disposed);
  check_failure ()

let ensure battle =
  ensure_factory battle.factory;
  if battle.released then raise (Rejected Disposed)

let catalog factory =
  ensure_factory factory;
  factory.catalog

let create_factory ?documents () =
  Js.Promise.then_
    (function
      | Error error -> Js.Promise.resolve (Error (Initialization error))
      | Ok () ->
          Js.Promise.resolve
            (protect (fun () ->
                 check_failure ();
                 let handle =
                   foreign (fun () ->
                       match documents with
                       | None -> value (Domain.NativeEngineFactory.create ())
                       | Some documents ->
                           value (Domain.NativeEngineFactory.with_documents documents))
                 in
                 try
                   let catalog =
                     Catalog.of_view (foreign (fun () -> Domain.NativeEngineFactory.catalog handle))
                   in
                   let factory =
                     { handle; catalog; topologies = []; disposed = false; battles = [] }
                   in
                   factories := weak factory :: List.map weak (registered ());
                   factory
                 with error ->
                   (if Domain.Wasm.failure () = None then
                      try Domain.NativeEngineFactory.dispose handle with _ -> ());
                   Fault.raise_original error)))
    (Domain.Wasm.load (Binding_text.of_js (module_url ())))

let validate_undo factory request =
  ensure_factory factory;
  value
    (foreign (fun () ->
         Domain.NativeEngineFactory.validate_undo factory.handle (Js.Json.stringify request)))

let publication ?previous ?inspection ?tiles ?indices ?(combat = None) factory id
    (source : Domain.engine_view) =
  Performance_trace.measure {js|native.publication|js} (fun () ->
      let state = source.Domain.state in
      let inspection =
        match inspection with
        | Some inspection -> inspection
        | None ->
            foreign (fun () ->
                value (Domain.NativeEngineFactory.inspect factory.handle id source.Domain.revision))
      in
      let inspection =
        match previous with
        | None -> inspection
        | Some previous ->
            let units =
              Array.map
                (fun (actions : Domain.unit_actions) ->
                  let faction =
                    match Js.Dict.get previous.choices actions.unit with
                    | Some old when Snapshot_equal.unit_faction old.faction actions.faction ->
                        old.faction
                    | _ -> actions.faction
                  in
                  (* Complete construction makes any new native action field a compiler
             error here. Discarded equal faction graphs are never traversed. *)
                  {
                    Domain.faction;
                    details = actions.details;
                    promotion_details = actions.promotion_details;
                    name = actions.name;
                    unit = actions.unit;
                    reachable = actions.reachable;
                    attacks = actions.attacks;
                    promotions = actions.promotions;
                    recruitment = actions.recruitment;
                  })
                inspection.units
            in
            { inspection with units }
      in
      if source.Domain.revision <> inspection.Domain.revision then
        failwith {js|Incoherent native publication|js};
      let key = (state.map_id, state.map_revision) in
      let topology =
        match List.assoc_opt key factory.topologies with
        | Some topology -> topology
        | None ->
            let topology =
              foreign (fun () ->
                  value (Domain.NativeEngineFactory.geometry factory.handle id source.revision))
            in
            let topology : topology =
              {
                neighbors =
                  Js.Dict.map ~f:(fun[@u] values -> Array.map Hex.key values) topology.neighbors;
              }
            in
            factory.topologies <- (key, topology) :: factory.topologies;
            topology
      in
      let incoming = source.presentation in
      let presentation =
        match previous with
        | None -> incoming
        | Some previous ->
            let fog =
              if Array_ext.equal Hex.equal previous.presentation.fog incoming.fog then
                previous.presentation.fog
              else incoming.fog
            in
            let lighting =
              if
                Snapshot_equal.battlefield_lighting previous.presentation.lighting incoming.lighting
              then previous.presentation.lighting
              else incoming.lighting
            in
            { incoming with Domain.fog; lighting }
      in
      let fog_index =
        match previous with
        | Some previous when previous.presentation.fog == presentation.fog -> previous.fog_index
        | _ ->
            let index = create_dictionary Js.Nullable.null in
            Array.iter (fun id -> Js.Dict.set index (Hex.key id) true) presentation.fog;
            index
      in
      let visible_ground_items =
        Array_ext.filter
          (fun (ground : Domain.ground_item) ->
            Js.Dict.get fog_index (Hex.key ground.tile) <> Some true)
          state.ground_items
      in
      let displayed_campaign =
        Option.map
          (fun (campaign : Domain.campaign_state) ->
            {
              campaign with
              dialogue =
                Array.mapi
                  (fun index (line : Domain.dialogue) ->
                    if index < Array.length inspection.speakers then
                      { line with speaker = inspection.speakers.(index) }
                    else line)
                  campaign.dialogue;
            })
          state.campaign
      in
      let choices = index (fun (actions : Domain.unit_actions) -> actions.unit) inspection.units in
      (* Identity reuse belongs to the visible display projection, not authoritative
     state. Concealed records remain in state without an equality traversal. *)
      let visible_units =
        Array_ext.filter_map
          (fun (unit : Domain.public_unit) ->
            let visible = Array.exists (( = ) unit.id) presentation.visible_units in
            if not visible then None
            else
              Some
                (match
                   Option.bind previous (fun previous -> Js.Dict.get previous.units unit.id)
                 with
                | Some old when Snapshot_equal.public_unit old unit -> old
                | _ -> unit))
          state.units
      in
      {
        displayed_campaign;
        visible_ground_items;
        visible_units;
        fog_index;
        revision = source.revision;
        combat;
        state;
        presentation;
        inspection;
        tiles =
          (match tiles with
          | Some tiles -> tiles
          | None -> index (fun (tile : Domain.tile) -> tile.id) state.tiles);
        tile_indices =
          (match indices with Some indices -> indices | None -> tile_indices state.tiles);
        units = index (fun (unit : Domain.public_unit) -> unit.id) visible_units;
        occupancy = index (fun (unit : Domain.public_unit) -> Hex.key unit.tile) visible_units;
        choices;
        neighbors = topology.neighbors;
      })

let frame_state (frame : Domain.state_frame) tiles : Domain.state =
  let {
    Domain.lighting;
    map_modifiers;
    vision_groups;
    factions;
    ground_items;
    armies;
    condition_memory;
    map_id;
    map_revision;
    cols;
    rows;
    round;
    turn;
    gold;
    winner;
    next_id;
    combat_rng;
    log;
    units;
    campaign;
    last_combat;
  } =
    frame
  in
  {
    Domain.lighting;
    map_modifiers;
    vision_groups;
    factions;
    ground_items;
    armies;
    condition_memory;
    map_id;
    map_revision;
    cols;
    rows;
    round;
    turn;
    gold;
    winner;
    next_id;
    combat_rng;
    log;
    units;
    campaign;
    last_combat;
    tiles;
  }

let update_publication factory id previous (update : Domain.dispatch_update) =
  if update.base_revision <> previous.revision || update.revision <> update.base_revision +. 1. then
    failwith {js|Incoherent native update revision|js};
  let frame = update.state in
  let same =
    (frame.map_id, frame.map_revision, frame.cols, frame.rows)
    = (previous.state.map_id, previous.state.map_revision, previous.state.cols, previous.state.rows)
  in
  let tiles, table, indices =
    Performance_trace.measure {js|native.terrain_update|js} (fun () ->
        match update.terrain with
        | Domain.Terrain_update.Keep ->
            if not same then failwith {js|Incoherent native terrain keep|js};
            (previous.state.tiles, previous.tiles, previous.tile_indices)
        | Domain.Terrain_update.Replace { tiles } ->
            (tiles, index (fun (tile : Domain.tile) -> tile.id) tiles, tile_indices tiles)
        | Domain.Terrain_update.Patch { tiles } ->
            if not same then failwith {js|Incoherent native terrain patch|js};
            let changed = Array.copy previous.state.tiles in
            Array.iter
              (fun (tile : Domain.tile) ->
                match Js.Dict.get previous.tile_indices tile.id with
                | None -> failwith {js|Unknown native patch tile|js}
                | Some position -> changed.(position) <- tile)
              tiles;
            (changed, index (fun (tile : Domain.tile) -> tile.id) changed, previous.tile_indices))
  in
  let state = frame_state frame tiles in
  publication ~previous ?inspection:update.inspection ~tiles:table ~indices ~combat:update.combat
    factory id
    { Domain.revision = update.revision; state; presentation = update.presentation }

let accept factory (handle : Domain.battle_handle) =
  let id = handle.Domain.id in
  try
    let current = publication factory id handle.view in
    let battle = { factory; id; current; released = false; failed = None } in
    factory.battles <- battle :: factory.battles;
    battle
  with error ->
    (if not factory.disposed then
       try
         let _ = foreign (fun () -> Domain.NativeEngineFactory.dispose_battle factory.handle id) in
         ()
       with _ -> ());
    Fault.raise_original error

external is_integer : float -> bool = "isInteger" [@@mel.scope "Number"]

let valid_seed seed = is_integer seed && seed >= 0. && seed <= 4294967295.

let create factory start seed =
  ensure_factory factory;
  if not (valid_seed seed) then raise (Rejected (Invalid_request {js|seed must be a uint32|js}));
  accept factory
    (foreign (fun () -> value (Domain.NativeEngineFactory.new_battle factory.handle start seed)))

let restore factory (saved : Domain.saved_game) =
  ensure_factory factory;
  if not (valid_seed saved.Domain.seed) then
    raise (Rejected (Invalid_request {js|saved seed must be a uint32|js}));
  accept factory
    (foreign (fun () -> value (Domain.NativeEngineFactory.restore factory.handle saved)))

let dispose battle =
  if not battle.released then
    (* Detect a trap raised through a direct generated call before cleanup. *)
    match Domain.Wasm.failure () with
    | Some reason -> invalidate (Module_failed reason)
    | None ->
        battle.released <- true;
        battle.failed <- None;
        battle.factory.battles <-
          List.filter (fun current -> current != battle) battle.factory.battles;
        if not battle.factory.disposed then
          let _ =
            foreign (fun () ->
                value (Domain.NativeEngineFactory.dispose_battle battle.factory.handle battle.id))
          in
          ()

let dispose_factory factory =
  if not factory.disposed then (
    let error = ref None in
    List.iter
      (fun battle ->
        try dispose battle with failure -> if !error = None then error := Some failure)
      factory.battles;
    if not factory.disposed then (
      factory.disposed <- true;
      factory.topologies <- [];
      factories :=
        List.filter_map
          (fun reference ->
            match Js.Undefined.toOption (deref reference) with
            | Some current when current != factory -> Some reference
            | _ -> None)
          !factories;
      try foreign (fun () -> Domain.NativeEngineFactory.dispose factory.handle)
      with failure -> if !error = None then error := Some failure);
    match !error with None -> () | Some failure -> Fault.raise_original failure)

let view battle =
  ensure battle;
  battle.current

let on_failure battle receive =
  ensure battle;
  battle.failed <- Some receive

let state view = view.state
let presentation view = view.presentation
let inspection view = view.inspection
let combat view = view.combat
let revision view = view.revision
let state_json view = foreign (fun () -> Domain.state_to_json view.state)

let actions_json view id =
  Option.map
    (fun actions -> foreign (fun () -> Domain.unit_actions_to_json actions))
    (Js.Dict.get view.choices id)

let apply battle expected command =
  ensure battle;
  let result =
    Performance_trace.measure {js|engine.dispatch|js} (fun () ->
        foreign (fun () ->
            value
              (Domain.NativeEngineFactory.dispatch_update battle.factory.handle battle.id expected
                 command true)))
  in
  (* Once accepted, an inspection failure is terminal even without a WASM trap.
     A trap has already invalidated all owners in [foreign]. *)
  try
    let accepted =
      Performance_trace.measure {js|native.reconcile|js} (fun () ->
          update_publication battle.factory battle.id battle.current result)
    in
    battle.current <- accepted;
    accepted
  with error ->
    if not battle.released then (
      let receive = battle.failed in
      (try dispose battle with _ -> ());
      notify receive (failure_reason error));
    Fault.raise_original error

let dispatch battle command = apply battle battle.current.revision command

let choose_ai_command battle options =
  ensure battle;
  Performance_trace.measure {js|engine.chooseAiCommand|js} (fun () ->
      foreign (fun () ->
          value
            (Domain.NativeEngineFactory.choose_ai_command battle.factory.handle battle.id
               battle.current.revision options)))

let prepare_enemy_step battle options =
  let _ = choose_ai_command battle options in
  ()

let enemy_step_recorded battle options =
  let decision = choose_ai_command battle options in
  let view = apply battle decision.Domain.revision decision.Domain.command in
  (view, decision.Domain.command)

let enemy_step battle options = fst (enemy_step_recorded battle options)

let save battle =
  ensure battle;
  foreign (fun () ->
      value
        (Domain.NativeEngineFactory.save battle.factory.handle battle.id battle.current.revision))

let save_json battle =
  ensure battle;
  foreign (fun () ->
      value
        (Domain.NativeEngineFactory.save_json battle.factory.handle battle.id
           battle.current.revision))

let replace battle saved =
  ensure battle;
  let replacement = restore battle.factory saved in
  let previous = battle.id in
  battle.id <- replacement.id;
  battle.current <- replacement.current;
  replacement.released <- true;
  battle.factory.battles <-
    List.filter (fun current -> current != replacement) battle.factory.battles;
  let _ =
    foreign (fun () ->
        value (Domain.NativeEngineFactory.dispose_battle battle.factory.handle previous))
  in
  battle.current

let faction view id =
  Option.map (fun (actions : Domain.unit_actions) -> actions.faction) (Js.Dict.get view.choices id)

let controllable view id =
  Option.fold ~none:false
    ~some:(fun (faction : Domain.unit_faction) -> faction.controllable)
    (faction view id)

let fogged view id = Js.Dict.get view.fog_index id = Some true

let visible view (unit : Domain.public_unit) =
  Array.exists (( = ) unit.id) view.presentation.visible_units

let units view = view.visible_units
let ground_items view = view.visible_ground_items
let tile view id = Js.Dict.get view.tiles id

let hex view id =
  match tile view id with
  | Some tile -> Hex.of_tile tile
  | None -> raise (Rejected (Invalid_request {j|Unknown tile: $(id)|j}))

let unit view id =
  Option.bind (Js.Dict.get view.units id) (fun unit ->
      if visible view unit then Some unit else None)

(* Display-only combat projection. It never mutates the engine publication or
   reveals resolved fog, loot, objectives or victory before presentation ends. *)
let project_units view units =
  let visible_units = Array_ext.filter (visible view) units in
  {
    view with
    state = { view.state with units };
    visible_units;
    units = index (fun (unit : Domain.public_unit) -> unit.id) visible_units;
    occupancy = index (fun (unit : Domain.public_unit) -> Hex.key unit.tile) visible_units;
  }

let at view id =
  Option.bind (Js.Dict.get view.occupancy id) (fun unit ->
      if visible view unit then Some unit else None)

let actions view id = Js.Dict.get view.choices id
let tiles_for view ids = Array_ext.filter_map (tile view) ids
let neighbors view id = tiles_for view (Option.value ~default:[||] (Js.Dict.get view.neighbors id))

let reachable view id =
  match actions view id with
  | None -> Domain.Text_map.empty
  | Some choices ->
      Array.fold_left
        (fun result (entry : Domain.reachable_tile) ->
          Domain.Text_map.add (Hex.key entry.tile) (float_of_int entry.cost) result)
        Domain.Text_map.empty choices.reachable

let attack_options view id target =
  match actions view id with
  | None -> [||]
  | Some choices ->
      Array_ext.filter
        (fun (attack : Domain.attack_details) -> attack.target = target)
        choices.attacks

let targets view id =
  match unit view id with
  | None -> [||]
  | Some actor when actor.Domain.attack_spent || view.state.Domain.winner <> None -> [||]
  | Some _ ->
      let ids = create_dictionary Js.Nullable.null in
      (match actions view id with
      | None -> ()
      | Some choices ->
          Array.iter
            (fun (attack : Domain.attack_details) -> Js.Dict.set ids attack.Domain.target true)
            choices.attacks);
      Array_ext.filter
        (fun (candidate : Domain.public_unit) -> Js.Dict.get ids candidate.id <> None)
        (Array_ext.filter (visible view) view.state.units)

let best_attack view id target =
  Array.find_opt
    (fun (attack : Domain.attack_details) -> attack.recommended)
    (attack_options view id target)

let attack_details view id target weapon =
  match actions view id with
  | None -> None
  | Some choices ->
      Array.find_opt
        (fun (attack : Domain.attack_details) ->
          attack.target = target && attack.attack.id = weapon)
        choices.attacks

let promotion_options view id =
  match actions view id with None -> [||] | Some choices -> choices.Domain.promotions

let recruitment_options view faction =
  Option.value ~default:[||] (Js.Dict.get view.inspection.recruitment faction)

let recruit_tiles view id kind =
  match actions view id with
  | None -> [||]
  | Some choices ->
      tiles_for view
        (Array.map Hex.key (Option.value ~default:[||] (Js.Dict.get choices.recruitment kind)))

let unit_details view id =
  match actions view id with
  | Some choices -> choices.Domain.details
  | None -> invalid_arg {j|No positioned unit details for $(id)|j}

let promotion_details view id =
  match actions view id with None -> [||] | Some choices -> choices.Domain.promotion_details

let recruit_details view faction kind =
  match
    Option.bind (Domain.Text_map.find_opt faction view.inspection.recruit_details) (fun profiles ->
        Array.find_opt (fun (profile : Domain.unit_details) -> profile.id = kind) profiles)
  with
  | Some profile -> profile
  | None -> invalid_arg {j|No resolved recruit details for $(faction):$(kind)|j}

let gold (value : Domain.gold) faction =
  Option.fold ~none:0. ~some:Int32.to_float (Domain.Text_map.find_opt faction value)

let income view faction = gold view.inspection.income faction

let unit_name view id =
  Option.bind (actions view id) (fun (choices : Domain.unit_actions) -> choices.name)

let recruit_price view faction kind =
  match
    Option.bind (Js.Dict.get view.inspection.recruitment_prices faction) (fun prices ->
        Js.Dict.get prices kind)
  with
  | Some price -> price
  | None -> invalid_arg {js|Unknown faction recruit.|js}

let unit_id id =
  let length = Js.String.length id in
  let unknown () = raise (Rejected (Invalid_request {js|Unknown unit.|js})) in
  if
    length < 2
    || Js.String.get id 0 <> {js|u|js}
    || Js.String.get id 1 < {js|1|js}
    || Js.String.get id 1 > {js|9|js}
  then unknown ();
  for index = 2 to length - 1 do
    if Js.String.get id index < {js|0|js} || Js.String.get id index > {js|9|js} then unknown ()
  done;
  let number = float_of_string (Js.String.slice ~start:1 ~end_:(1 + (length - 1)) id) in
  if number > 4294967295. then unknown ();
  number

let campaign view = view.displayed_campaign
let validate_undo factory request = protect (fun () -> validate_undo factory request)
let create factory start seed = protect (fun () -> create factory start seed)
let restore factory saved = protect (fun () -> restore factory saved)
let dispatch battle command = protect (fun () -> dispatch battle command)
let choose_ai_command battle options = protect (fun () -> choose_ai_command battle options)
let prepare_enemy_step battle options = protect (fun () -> prepare_enemy_step battle options)
let enemy_step_recorded battle options = protect (fun () -> enemy_step_recorded battle options)
let enemy_step battle options = protect (fun () -> enemy_step battle options)
let save battle = protect (fun () -> save battle)
let save_json battle = protect (fun () -> save_json battle)
let replace battle saved = protect (fun () -> replace battle saved)

let preview_unit battle kind faction tile =
  protect (fun () ->
      if battle.released then raise (Rejected Disposed);
      value
        (Domain.NativeEngineFactory.preview_unit battle.factory.handle battle.id
           battle.current.revision kind faction tile))

let tile_details view id =
  match
    Array.find_opt
      (fun (tile : Domain.tile_details) -> Hex.key tile.tile = id)
      view.inspection.tiles
  with
  | Some details -> details
  | None -> invalid_arg {j|No resolved terrain details for $(id)|j}
