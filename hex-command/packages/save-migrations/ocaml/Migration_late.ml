(* Explicit, frozen version-to-version conversions. JSON is confined to the
   unknown-input codecs, frozen recipe readers, and opaque undo patch values. *)
open Historical_types
module Map = Historical_types.String_map
module Data = Migration_late_data

let fail message = invalid_arg message
let required message = function Some value -> value | None -> fail message
let list value = Array.to_list value
let optional_array value = Option.value ~default:[||] value
let wire_order wire keys = { wire with order = Data.unique (wire.order @ keys) }
let new_wire keys = { empty_wire with order = keys }
let latest values = if Array.length values = 0 then None else Some values.(Array.length values - 1)

let flag_state flags key =
  match Option.bind (Map.find_opt key flags) latest with
  | None -> false
  | Some (change : fragments_native_state38_campaign_member1_flags_value_item) -> change.state

let v29 (value : V29.t) : V30.t =
  let setup : V30.setup option =
    Option.map
      (fun (setup : V29.setup) ->
        let enemy = setup.enemy in
        let diplomacy =
          match enemy.diplomacy with
          | `Restrained -> `Diplomatic
          | `Pragmatic -> `Unrestricted
          | `Diplomatic -> `Diplomatic
          | `Unrestricted -> `Unrestricted
        in
        ({
           wire = setup.wire;
           first = setup.first;
           enemy =
             {
               wire = enemy.wire;
               policy = enemy.policy;
               blocks = enemy.blocks;
               recruitment_limit = enemy.recruitment_limit;
               diplomacy;
             };
         }
          : V30.setup))
      value.setup
  in
  let state = value.checkpoint.state in
  let pending_enemy = ref value.pending_enemy in
  let state : V30.state =
    match state.campaign with
    | None -> state
    | Some old ->
        let recipe =
          Migration_json.field (Lazy.force Frozen_data.history_events_v29) state.map_id
          |> Data.recipe Historical_codec.read_fragments_factions7_item_recruits_member1_item
        in
        let stage = old.mission and turn = state.round in
        let marks = list old.marks and flags = ref old.flags in
        let add ?(at_stage = stage) ?(at_turn = turn) id =
          if not (flag_state !flags id) then
            let change : fragments_native_state38_campaign_member1_flags_value_item =
              {
                wire = new_wire [ "state"; "stage"; "turn" ];
                state = true;
                stage = at_stage;
                turn = at_turn;
              }
            in
            flags :=
              Map.add id
                (Array.append (Option.value ~default:[||] (Map.find_opt id !flags)) [| change |])
                !flags
        in
        List.iter
          (fun (from_, to_) ->
            match Map.find_opt from_ !flags with
            | None -> ()
            | Some incoming ->
                let merged =
                  Array.append (Option.value ~default:[||] (Map.find_opt to_ !flags)) incoming
                in
                Array.stable_sort
                  (fun (a : fragments_native_state38_campaign_member1_flags_value_item) b ->
                    let stage = compare a.stage b.stage in
                    if stage <> 0 then stage else compare a.turn b.turn)
                  merged;
                flags := Map.remove from_ (Map.add to_ merged !flags))
          [
            ("green-dain-dead", "dain-dead");
            ("refuge-war", "peace-lost");
            ("refuge-peace", "peace-kept");
          ];
        let existing = old.scripts in
        let active =
          ref
            (Data.unique
               (match existing with
               | None -> list recipe.active
               | Some scripts ->
                   List.filter (fun id -> Array.mem id recipe.active) (list scripts.active)
                   @ List.filter
                       (fun id -> not (Array.mem id recipe.previous_event_ids))
                       (list recipe.active)))
        in
        let failed = ref [] in
        let retire id = active := List.filter (( <> ) id) !active in
        let add_active id = if not (List.mem id !active) then active := !active @ [ id ] in
        let fail_objective id =
          match Array.find_opt (fun (_, objective) -> objective = id) recipe.objectives with
          | Some (owner, _) when List.mem owner !active ->
              retire owner;
              if not (List.mem id !failed) then failed := !failed @ [ id ]
          | _ -> ()
        in
        Array.iter
          (fun id ->
            if List.mem ("event:" ^ id) marks then (
              retire id;
              add id))
          recipe.old_events;
        Array.iter (fun id -> if List.mem ("wave:" ^ id) marks then retire id) recipe.waves;
        Array.iter
          (fun (tile, id) ->
            if List.mem ("sealed:" ^ tile) marks then (
              retire id;
              add id))
          recipe.seals;
        let incoming = list (optional_array old.fallen_companions) in
        let veterans = optional_array old.story_veterans in
        let alive =
          Array.to_list state.units |> List.filter_map (fun (unit_ : V29.unit_) -> unit_.character)
        in
        let departed who =
          flag_state !flags (Option.value ~default:"" (Data.get recipe.departed_when_flag who))
        in
        let died_now = ref [] in
        Data.unique
          (list recipe.named @ incoming
          @ List.filter_map
              (fun marker ->
                if Data.starts "fallen:" marker then Some (Data.suffix "fallen:" marker) else None)
              marks)
        |> List.iter (fun who ->
            if not (List.mem who alive || departed who) then (
              let present_at_entry =
                Array.exists
                  (fun (veteran : save26_checkpoint_state_campaign_member1_veterans_member1_item) ->
                    veteran.character = Some who)
                  veterans
              in
              let prior =
                List.mem who incoming && (not present_at_entry)
                && Option.value ~default:stage (Data.get recipe.first_seen who) < stage
              in
              if not prior then died_now := who :: !died_now;
              add
                ~at_stage:(if prior then max 0. (stage -. 1.) else stage)
                ~at_turn:(if prior then 0. else turn)
                (who ^ "-dead");
              Array.iter fail_objective
                (Option.value ~default:[||] (Data.get recipe.failure_units who))));
        Option.iter
          (fun (refuge : fragments_native_state38_campaign_member1_refuge_member1) ->
            if refuge.escalated then (
              add "peace-lost";
              fail_objective "refuge-peace";
              retire "peace-lost"))
          old.refuge;
        if flag_state !flags "ferryman-dead" then fail_objective "ford-shout";
        if flag_state !flags "medicine" then add "medicine-explained";
        if state.winner = `Player then (
          if Array.mem "refuge-peace" recipe.required && flag_state !flags "peace-lost" then (
            retire "refuge-war";
            fail_objective "refuge-peace")
          else Array.iter retire recipe.required;
          retire "chapter-victory");
        Array.iter
          (fun id ->
            if
              Data.starts "goal-" id
              && (not (List.mem id !active))
              && not (List.mem (Option.value ~default:id (Data.get recipe.objectives id)) !failed)
            then add (state.map_id ^ "-" ^ id))
          recipe.active;
        let holds =
          match existing with
          | Some scripts -> scripts.holds
          | None ->
              Array.to_list recipe.timers
              |> List.filter_map (fun (timer : Data.timer) ->
                  let source =
                    match timer.source with
                    | "holds" ->
                        Option.bind old.holds (Map.find_opt timer.tile)
                        |> Option.map
                             (fun
                               (counter : fragments_native_state38_campaign_member1_vigil_member1)
                             -> (counter.held_turns, counter.last_counted_round))
                    | "vigil" ->
                        Option.map
                          (fun (counter : fragments_native_state38_campaign_member1_vigil_member1)
                             -> (counter.held_turns, counter.last_counted_round))
                          old.vigil
                    | _ ->
                        Option.map
                          (fun (counter : fragments_native_state38_campaign_member1_refuge_member1)
                             -> (counter.held_turns, counter.last_counted_round))
                          old.refuge
                  in
                  Option.map
                    (fun (held_turns, last_counted_round) ->
                      ({
                         wire = new_wire [ "event"; "predicate"; "turns"; "lastRound" ];
                         event = timer.event;
                         predicate = 0.;
                         turns = min held_turns timer.turns;
                         last_round = last_counted_round;
                       }
                        : fragments_native_state38_campaign_member1_scripts_member1_holds_item))
                    source)
              |> Array.of_list
        in
        let recruitment =
          ref (match existing with None -> Map.empty | Some scripts -> scripts.recruitment)
        in
        Map.iter
          (fun faction extra ->
            let pool =
              ref (list (Option.value ~default:[||] (Data.get recipe.base_recruitment faction)))
            in
            Array.iter
              (fun (entry : V29.recruits) ->
                if not (List.exists (fun (old : V29.recruits) -> old.unit_ = entry.unit_) !pool)
                then pool := !pool @ [ entry ])
              extra;
            recruitment := Map.add faction (Array.of_list !pool) !recruitment)
          !recruitment;
        Option.iter
          (fun (branch : V29.recruits Data.branch) ->
            if flag_state !flags branch.flag then
              Array.iter
                (fun (faction, pool) -> recruitment := Map.add faction pool !recruitment)
                branch.recruitment)
          recipe.branch;
        Array.iter add_active recipe.repeat;
        let missing =
          Array.find_opt
            (fun (entry : Data.missing_character) ->
              not (List.mem entry.character alive || departed entry.character))
            recipe.defeat_on_missing
        in
        let completed, outcome, winner =
          match missing with
          | None -> (old.completed, old.outcome, state.winner)
          | Some entry ->
              pending_enemy := false;
              ( false,
                (match (state.winner, old.outcome) with
                | `Enemy, `String text when text <> "" -> old.outcome
                | _ -> `String entry.message),
                `Enemy )
        in
        let scripts : V29.scripts =
          {
            wire =
              wire_order
                (match existing with
                | None -> new_wire [ "marks"; "pending"; "nextMark" ]
                | Some scripts -> scripts.wire)
                [ "active"; "failed"; "holds"; "recruitment" ];
            marks = (match existing with None -> [||] | Some scripts -> scripts.marks);
            pending = (match existing with None -> [||] | Some scripts -> scripts.pending);
            next_mark = (match existing with None -> 1. | Some scripts -> scripts.next_mark);
            active = Array.of_list !active;
            failed = Some (Array.of_list !failed);
            holds;
            recruitment = !recruitment;
            completed = Option.bind existing (fun scripts -> scripts.completed);
            revealed = Option.bind existing (fun scripts -> scripts.revealed);
          }
        in
        let campaign : V29.campaign =
          {
            old with
            wire =
              wire_order
                (wire_remove (wire_remove (wire_remove old.wire "refuge") "vigil") "holds")
                [ "flags"; "marks"; "fallenCompanions"; "scripts" ];
            refuge = None;
            vigil = None;
            holds = None;
            completed;
            outcome;
            flags = !flags;
            marks = [||];
            fallen_companions =
              Some (Array.of_list (List.filter (fun id -> not (List.mem id !died_now)) incoming));
            scripts = Some scripts;
          }
        in
        { state with winner; campaign = Some campaign }
  in
  {
    wire = wire_order value.wire [ "setup" ];
    version = `V30;
    checkpoint = { value.checkpoint with state };
    setup;
    pending_enemy = !pending_enemy;
    progress = value.progress;
    undo = value.undo;
  }

let v30 (value : V30.t) : V31.t =
  {
    wire = value.wire;
    version = `V31;
    checkpoint = value.checkpoint;
    pending_enemy = value.pending_enemy;
    setup = value.setup;
    progress = value.progress;
    undo = value.undo;
  }

let v31 (value : V31.t) : V32.t =
  {
    wire = wire_remove value.wire "undo";
    version = `V32;
    checkpoint = value.checkpoint;
    pending_enemy = value.pending_enemy;
    setup = value.setup;
    progress = value.progress;
    undo = None;
  }

let v32 (value : V32.t) : V33.t =
  {
    wire = value.wire;
    version = `V33;
    checkpoint = value.checkpoint;
    pending_enemy = value.pending_enemy;
    setup = value.setup;
    progress = value.progress;
    undo = value.undo;
  }

let v33 (value : V33.t) : V34.t =
  let state = value.checkpoint.state in
  let campaign : V34.campaign option =
    Option.map
      (fun (campaign : V33.campaign) ->
        let scripts = required "Invalid campaign objective state" campaign.scripts in
        let active, completed, revealed =
          if wire_has scripts.wire "completed" || wire_has scripts.wire "revealed" then
            ( scripts.active,
              required "Incomplete saved objective history" scripts.completed,
              required "Incomplete saved objective history" scripts.revealed )
          else
            let objectives =
              Migration_json.field (Lazy.force Frozen_data.history_objectives_v30) state.map_id
              |> Migration_json.array |> Array.map Data.objective
            in
            let failed = required "Invalid historical objective state" scripts.failed in
            let rec matches = function
              | Data.Flag id -> flag_state campaign.flags id
              | Data.All conditions -> Array.for_all matches conditions
              | Data.Any conditions -> Array.exists matches conditions
              | Data.Not condition -> not (matches condition)
            in
            let recipe =
              Option.map Data.additions
                (Migration_json.get
                   (Lazy.force Frozen_data.history_objective_events_v31)
                   state.map_id)
            in
            let active =
              Data.unique
                (list scripts.active
                @ match recipe with None -> [] | Some recipe -> list recipe.added)
            in
            let next_active =
              match recipe with
              | None -> scripts.active
              | Some recipe ->
                  Array.of_list
                    (List.filter (fun id -> List.mem id active) (list recipe.order)
                    @ List.filter (fun id -> not (Array.mem id recipe.order)) (list scripts.active)
                    )
            in
            ( next_active,
              Array.of_list
                (Array.to_list objectives
                |> List.filter_map (fun (objective : Data.objective) ->
                    if
                      (not (List.mem objective.event active)) && not (Array.mem objective.id failed)
                    then Some objective.id
                    else None)),
              Array.of_list
                (Array.to_list objectives
                |> List.filter_map (fun (objective : Data.objective) ->
                    if Array.mem objective.id failed || matches objective.visible_when then
                      Some objective.id
                    else None)) )
        in
        let scripts : V34.scripts =
          {
            wire = wire_order scripts.wire [ "completed"; "revealed" ];
            recruitment = scripts.recruitment;
            failed = scripts.failed;
            active;
            holds = scripts.holds;
            next_mark = scripts.next_mark;
            pending = scripts.pending;
            marks = scripts.marks;
            completed;
            revealed;
          }
        in
        ({
           wire = campaign.wire;
           mission = campaign.mission;
           id = campaign.id;
           vigil = campaign.vigil;
           refuge = campaign.refuge;
           completed = campaign.completed;
           marks = campaign.marks;
           holds = campaign.holds;
           recall = campaign.recall;
           fallen_companions = campaign.fallen_companions;
           carry_gold = campaign.carry_gold;
           dialogue = campaign.dialogue;
           dialogue_index = campaign.dialogue_index;
           outcome = campaign.outcome;
           veterans = campaign.veterans;
           story_veterans = campaign.story_veterans;
           evacuated = campaign.evacuated;
           scripts = Some scripts;
           flags = campaign.flags;
         }
          : V34.campaign))
      state.campaign
  in
  let state : V34.state =
    {
      wire = state.wire;
      round = state.round;
      turn = state.turn;
      winner = state.winner;
      armies = state.armies;
      condition_memory = state.condition_memory;
      map_id = state.map_id;
      map_revision = state.map_revision;
      cols = state.cols;
      rows = state.rows;
      next_id = state.next_id;
      log = state.log;
      last_combat = state.last_combat;
      units = state.units;
      tiles = state.tiles;
      campaign;
      gold = state.gold;
      factions = state.factions;
      ground_items = state.ground_items;
      combat_rng = state.combat_rng;
    }
  in
  {
    wire = wire_remove value.wire "undo";
    version = `V34;
    checkpoint = { wire = value.checkpoint.wire; seed = value.checkpoint.seed; state };
    pending_enemy = value.pending_enemy;
    setup = value.setup;
    progress = value.progress;
    undo = None;
  }

external host_length : string -> int = "length" [@@mel.get]
external code_unit : string -> int -> int = "charCodeAt" [@@mel.send]

let entry_info_order =
  [
    "id";
    "runId";
    "branchId";
    "checkpointId";
    "ordinal";
    "savedAt";
    "run";
    "mission";
    "round";
    "side";
    "missionStart";
  ]

let v34 (value : V34.t) : V35.t =
  (* This is deliberately encoded before the new default facts are added: the
     deployed converter hashes the exact original checkpoint property order. *)
  let original_text =
    Js.Json.stringify (Historical_codec.encode_save34_checkpoint value.checkpoint)
  in
  let old_state = value.checkpoint.state in
  let units =
    Array.map
      (fun (unit_ : V34.unit_) ->
        match unit_.experience with
        | Some _ -> unit_
        | None -> { unit_ with wire = wire_order unit_.wire [ "experience" ]; experience = Some 0. })
      old_state.units
  in
  let combat_rng = Option.value ~default:1831565813. old_state.combat_rng in
  let state : V35.state =
    {
      wire = wire_order old_state.wire [ "combatRng" ];
      round = old_state.round;
      turn = old_state.turn;
      winner = old_state.winner;
      armies = old_state.armies;
      condition_memory = old_state.condition_memory;
      map_id = old_state.map_id;
      map_revision = old_state.map_revision;
      cols = old_state.cols;
      rows = old_state.rows;
      next_id = old_state.next_id;
      log = old_state.log;
      last_combat = old_state.last_combat;
      units;
      tiles = old_state.tiles;
      campaign = old_state.campaign;
      gold = old_state.gold;
      factions = old_state.factions;
      ground_items = old_state.ground_items;
      combat_rng;
    }
  in
  let setup : V35.setup =
    match value.setup with
    | Some setup -> setup
    | None ->
        {
          wire = new_wire [ "first"; "enemy" ];
          first = `Player;
          enemy =
            {
              wire = new_wire [ "policy"; "diplomacy" ];
              policy = `Classic;
              diplomacy = `Diplomatic;
              blocks = None;
              recruitment_limit = None;
            };
        }
  in
  let entry : V35.entry =
    match value.progress with
    | None ->
        let hashes = [| -2128831035l; -2048144777l; -1028477379l; 668265263l |] in
        for i = 0 to host_length original_text - 1 do
          Array.iteri
            (fun j hash ->
              hashes.(j) <-
                Int32.mul (Int32.logxor hash (Int32.of_int (code_unit original_text i))) 16777619l)
            hashes
        done;
        let id =
          "migrated-" ^ String.concat "" (Array.to_list (Array.map (Printf.sprintf "%08lx") hashes))
        in
        let run, mission =
          match state.campaign with
          | None -> (state.map_id, state.map_id)
          | Some campaign ->
              ( campaign.id,
                "Mission "
                ^ Migration_utf8.of_js (Js.Float.toString (campaign.mission +. 1.))
                ^ " · " ^ state.map_id )
        in
        {
          wire =
            new_wire
              [
                "id";
                "runId";
                "branchId";
                "checkpointId";
                "ordinal";
                "savedAt";
                "run";
                "mission";
                "round";
                "side";
                "kind";
                "missionStart";
              ];
          id = id ^ "-branch";
          run_id = id;
          branch_id = id ^ "-branch";
          checkpoint_id = id ^ "-point";
          ordinal = 0.;
          saved_at = 0.;
          run;
          mission;
          round = state.round;
          side = (match state.turn with `Player -> "player" | `Enemy -> "enemy");
          kind = `Recovery;
          mission_start = false;
          parent_id = None;
        }
    | Some progress ->
        let old = progress.entry in
        let kind = match old.kind with `Turn -> `Turn | `Recovery | `Resume -> `Recovery in
        let turn = kind = `Turn in
        {
          wire =
            new_wire
              (entry_info_order @ [ "kind" ]
              @ if turn && Option.is_some old.parent_id then [ "parentId" ] else []);
          id = (if turn then old.id else old.branch_id);
          saved_at = old.saved_at;
          run = old.run;
          mission = old.mission;
          round = old.round;
          side = old.side;
          mission_start = (if turn then old.mission_start else false);
          run_id = old.run_id;
          branch_id = old.branch_id;
          checkpoint_id = old.checkpoint_id;
          ordinal = old.ordinal;
          kind;
          parent_id = (if turn then old.parent_id else None);
        }
  in
  {
    wire = wire_order (wire_remove value.wire "undo") [ "setup"; "checkpoint"; "progress"; "undo" ];
    version = `V35;
    checkpoint = { wire = value.checkpoint.wire; seed = value.checkpoint.seed; state };
    pending_enemy = value.pending_enemy;
    setup;
    progress = { wire = new_wire [ "entry" ]; entry };
    undo = (if entry.kind = `Recovery then Some [||] else None);
  }

let v35 (value : V35.t) : V36.t =
  {
    wire = value.wire;
    version = `V36;
    checkpoint = value.checkpoint;
    pending_enemy = value.pending_enemy;
    setup = value.setup;
    progress = value.progress;
    undo = value.undo;
  }

let v36 (value : V36.t) : V37.t =
  let old = value.progress.entry in
  let entry : V37.entry =
    match old.kind with
    | `Recovery ->
        (* Flash identity has no historical parent link. Retire it from both
           declared fields and preserved wire metadata. *)
        let wire = wire_remove old.wire "parentId" in
        `Flash
          {
            wire;
            kind = `Flash;
            id = old.id;
            saved_at = old.saved_at;
            run = old.run;
            mission = old.mission;
            round = old.round;
            side = old.side;
            mission_start = old.mission_start;
            run_id = old.run_id;
            branch_id = old.branch_id;
            checkpoint_id = old.checkpoint_id;
            ordinal = old.ordinal;
          }
    | `Manual | `Turn ->
        `Turn
          {
            wire = wire_order old.wire [ "manual" ];
            kind = `Turn;
            manual = old.kind = `Manual;
            parent_id = old.parent_id;
            id = old.id;
            saved_at = old.saved_at;
            run = old.run;
            mission = old.mission;
            round = old.round;
            side = old.side;
            mission_start = old.mission_start;
            run_id = old.run_id;
            branch_id = old.branch_id;
            checkpoint_id = old.checkpoint_id;
            ordinal = old.ordinal;
          }
  in
  let flash = old.kind = `Recovery in
  {
    wire = (if flash then value.wire else wire_remove value.wire "undo");
    version = `V37;
    checkpoint = value.checkpoint;
    pending_enemy = value.pending_enemy;
    setup = value.setup;
    progress = { wire = (if flash then value.progress.wire else new_wire [ "entry" ]); entry };
    undo = (if flash then value.undo else None);
  }

(* Undo patches are deliberately opaque data in the wire contract. Each patched
   full state is decoded as its historical version before the typed conversion. *)
let convert_state_and_undo_with_codecs ~encode_change ~decode_change ~encode_old ~decode_old
    ~encode_new convert current undo =
  let previous = ref (encode_old current) in
  let converted = ref (convert current) in
  let state = !converted in
  let undo =
    Option.map
      (fun steps ->
        Array.to_list steps |> List.rev
        |> List.map (fun (step : save34_undo_member1_item) ->
            Array.iter
              (fun change -> previous := Migration_undo.apply !previous (encode_change change) 0)
              step.changes;
            let next = convert (decode_old !previous) in
            let changes =
              Migration_undo.changes (encode_new !converted) (encode_new next) []
              |> List.map decode_change |> Array.of_list
            in
            converted := next;
            { step with changes })
        |> List.rev |> Array.of_list)
      undo
  in
  (state, undo)

let convert_state_and_undo ~encode_old ~decode_old ~encode_new convert current undo =
  convert_state_and_undo_with_codecs
    ~encode_change:Historical_codec.encode_save34_undo_member1_item_changes_item
    ~decode_change:Historical_codec.read_save34_undo_member1_item_changes_item ~encode_old
    ~decode_old ~encode_new convert current undo

let state37 (old : V37.state) : V38.state =
  let names =
    match Migration_json.get (Lazy.force Frozen_data.history_locations_v37) old.map_id with
    | None -> [||]
    | Some value ->
        Migration_json.entries value
        |> Array.map (fun (key, name) -> (key, Migration_json.string name))
  in
  let addresses = ref Map.empty and occupied = ref [] in
  Array.iter
    (fun (tile : V37.tile) ->
      if not (Migration_json.safe_integer tile.col && Migration_json.safe_integer tile.row) then
        fail "Invalid historical cell";
      let address =
        Migration_utf8.of_js (Js.Float.toString tile.col)
        ^ ","
        ^ Migration_utf8.of_js (Js.Float.toString tile.row)
      in
      if Map.mem tile.id !addresses || List.mem address !occupied then
        fail "Duplicate historical cell";
      addresses := Map.add tile.id address !addresses;
      occupied := address :: !occupied)
    old.tiles;
  let tile id = required "Unknown historical cell reference" (Map.find_opt id !addresses) in
  let event id =
    if
      Option.fold ~none:false
        ~some:(fun (campaign : V37.campaign) -> campaign.id = "ashen-oath")
        old.campaign
    then
      match id with
      | "sealed-hex-0941-278b487380" -> "northern-seal-closed"
      | "sealed-hex-0957-c94afbbfa1" -> "southern-seal-closed"
      | _ -> id
    else id
  in
  let rec condition : V37.condition -> V38.condition = function
    | `Object10 c -> `Object10 { c with conditions = Array.map condition c.conditions }
    | `Owner c -> `Owner { c with tile = Option.map tile c.tile }
    | `Proximity c -> `Proximity { c with tile = Option.map tile c.tile }
    | `Event c -> `Event { c with event = event c.event }
    | `Flag c -> `Flag { c with flag = event c.flag }
    | `Choice c -> `Choice { c with choice = event c.choice }
    | (`Visible_units _ | `Has_units _ | `No_units _ | `Round _) as c -> c
  in
  let rule (r : V37.rule) : V38.rule = { r with condition = condition r.condition } in
  let light (l : V37.light) : V38.light = { l with when_ = Option.map rule l.when_ } in
  let unit_ (u : V37.unit_) : V38.unit_ = { u with tile = tile u.tile } in
  let memory key =
    match List.find_opt (fun prefix -> Data.starts prefix key) [ "fog:"; "light:" ] with
    | None -> key
    | Some prefix -> prefix ^ tile (Data.suffix prefix key)
  in
  let marker key =
    if Data.starts "event:" key then "event:" ^ event (Data.suffix "event:" key)
    else if Data.starts "sealed:" key then "sealed:" ^ tile (Data.suffix "sealed:" key)
    else key
  in
  let keys values rename =
    Map.fold
      (fun id value result ->
        let id = rename id in
        if Map.mem id result then fail "Ambiguous historical reference rename";
        Map.add id value result)
      values Map.empty
  in
  {
    old with
    tiles =
      Array.map
        (fun (t : V37.tile) ->
          let location = Data.get names t.id in
          {
            t with
            wire =
              (match location with None -> t.wire | Some _ -> wire_order t.wire [ "location" ]);
            id = tile t.id;
            location =
              (match location with Some value when value <> "" -> Some value | _ -> t.location);
            fog = Option.map rule t.fog;
            light = Option.map light t.light;
          })
        old.tiles;
    units = Array.map unit_ old.units;
    ground_items =
      Array.map
        (fun (item : V37.ground_items) -> { item with tile = tile item.tile })
        old.ground_items;
    factions =
      Array.map
        (fun (faction : V37.faction) ->
          {
            faction with
            transfer =
              Option.map
                (fun (transfer : V37.transfer) -> { transfer with when_ = rule transfer.when_ })
                faction.transfer;
          })
        old.factions;
    condition_memory = Option.map (Array.map memory) old.condition_memory;
    campaign =
      Option.map
        (fun (campaign : V37.campaign) ->
          {
            campaign with
            flags = keys campaign.flags event;
            marks = Array.map marker campaign.marks;
            holds =
              Option.map
                (fun values ->
                  Map.fold
                    (fun id counter result -> Map.add (tile id) counter result)
                    values Map.empty)
                campaign.holds;
            evacuated =
              Option.map
                (Array.map
                   (fun (entry : save26_checkpoint_state_campaign_member1_evacuated_member1_item) ->
                     { entry with event = event entry.event; unit_ = unit_ entry.unit_ }))
                campaign.evacuated;
            scripts =
              Option.map
                (fun (scripts : V37.scripts) ->
                  {
                    scripts with
                    active = Array.map event scripts.active;
                    completed = Array.map event scripts.completed;
                    revealed = Array.map event scripts.revealed;
                    failed = Option.map (Array.map event) scripts.failed;
                    holds =
                      Array.map
                        (fun (hold :
                               fragments_native_state38_campaign_member1_scripts_member1_holds_item)
                           -> { hold with event = event hold.event })
                        scripts.holds;
                    marks =
                      Array.map
                        (fun (mark :
                               save28_checkpoint_state_campaign_member1_scripts_member1_marks_item)
                           ->
                          { mark with tile = tile mark.tile; light = Option.map light mark.light })
                        scripts.marks;
                    pending =
                      Array.map
                        (fun (pending :
                               fragments_native_state38_campaign_member1_scripts_member1_pending_item)
                           -> { pending with area = Array.map tile pending.area })
                        scripts.pending;
                  })
                campaign.scripts;
          })
        old.campaign;
  }

let v37 (value : V37.t) : V38.t =
  let state, undo =
    convert_state_and_undo ~encode_old:Historical_codec.encode_save35_checkpoint_state
      ~decode_old:Historical_codec.read_save35_checkpoint_state
      ~encode_new:Historical_codec.encode_save35_checkpoint_state state37 value.checkpoint.state
      value.undo
  in
  {
    wire = value.wire;
    version = `V38;
    checkpoint = { value.checkpoint with state };
    pending_enemy = value.pending_enemy;
    setup = value.setup;
    progress = value.progress;
    undo;
  }

let v38 (value : V38.t) : V39.t =
  let convert state =
    Historical_codec.encode_save35_checkpoint_state state
    |> Historical_codec.Fragments.native_state38_of_json |> Migration_coordinates.state
  in
  let state, undo =
    convert_state_and_undo ~encode_old:Historical_codec.encode_save35_checkpoint_state
      ~decode_old:Historical_codec.read_save35_checkpoint_state
      ~encode_new:Historical_codec.encode_save39_member0_checkpoint_state convert
      value.checkpoint.state value.undo
  in
  let checkpoint : V39.current_checkpoint =
    { wire = value.checkpoint.wire; seed = value.checkpoint.seed; state }
  in
  let setup : V39.current_setup =
    {
      wire = value.setup.wire;
      first = value.setup.first;
      enemy =
        {
          wire = value.setup.enemy.wire;
          blocks = value.setup.enemy.blocks;
          policy = value.setup.enemy.policy;
          diplomacy = value.setup.enemy.diplomacy;
          recruitment_limit = value.setup.enemy.recruitment_limit;
        };
    }
  in
  let result : V39.t =
    match value.progress.entry with
    | `Flash entry ->
        let undo = required "Invalid current save envelope" undo in
        `Current
          {
            wire = value.wire;
            pending_enemy = value.pending_enemy;
            undo;
            progress = { wire = value.progress.wire; entry };
            version = `V39;
            checkpoint;
            setup;
          }
    | `Turn entry ->
        if value.pending_enemy || Option.is_some undo then fail "Invalid current save envelope";
        `Turn
          {
            wire = value.wire;
            pending_enemy = `False;
            undo = None;
            progress = { wire = value.progress.wire; entry };
            version = `V39;
            checkpoint;
            setup;
          }
  in
  let _ = Historical_codec.V39.of_json (Historical_codec.V39.to_json result) in
  result
