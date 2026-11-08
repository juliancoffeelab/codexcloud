(* The frozen v18 -> v29 conversions, corresponding to upstream's individual
   version-to-version modules. JSON is read only at the migration boundary. *)
module T = Historical_types
module C = Historical_codec
module D = Migration_middle_data

let invalid message = invalid_arg message
let require condition message = if not condition then invalid message
let find predicate values = Array.find_opt predicate values
let lookup key values = List.assoc_opt key (Array.to_list values)
let member value values = Array.exists (( = ) value) values
let filter predicate values = Array.of_list (List.filter predicate (Array.to_list values))
let filter_map f values = Array.of_list (List.filter_map f (Array.to_list values))

let unique values =
  Array.fold_left
    (fun result value -> if member value result then result else Array.append result [| value |])
    [||] values

let starts prefix value =
  String.length value >= String.length prefix && String.sub value 0 (String.length prefix) = prefix

let suffix prefix value =
  String.sub value (String.length prefix) (String.length value - String.length prefix)

external marker_replace :
  string ->
  Js.Re.t ->
  ((string -> string -> string -> int -> string -> string)[@mel.uncurry]) ->
  string = "replace"
[@@mel.send]

let marker tile key =
  if starts "sealed:" key then "sealed:" ^ tile (suffix "sealed:" key)
  else if starts "charge:" key then
    let parts = String.split_on_char ':' key in
    let last = List.hd (List.rev parts) in
    if Js.Re.test ~str:(Migration_utf8.to_js last) (Js.Re.fromString "^\\d+,\\d+$") then
      String.concat ":" (List.rev (tile last :: List.tl (List.rev parts)))
    else key
  else if starts "ritual:" key || starts "spawn:ritual:" key then
    Migration_utf8.of_js
      (marker_replace (Migration_utf8.to_js key)
         (Js.Re.fromStringWithFlags ":(mark-|death-u[1-9]\\d*-)(\\d+,\\d+)(?=:[^:]+$|$)" ~flags:"g")
         (fun _ source coordinate _ _ ->
           Migration_utf8.to_js
             (":" ^ Migration_utf8.of_js source ^ tile (Migration_utf8.of_js coordinate))))
  else key

let memory tile key =
  match List.find_opt (fun prefix -> starts prefix key) [ "fog:"; "light:" ] with
  | None -> key
  | Some prefix -> prefix ^ tile (suffix prefix key)

open T
module Map = T.String_map

let choose replacement original = Option.value replacement ~default:original

let optional_merge replacement original =
  match replacement with None -> original | Some _ -> replacement

let patch_tile (tile : V18.tile) (patch : Fragments.delta19_item) : V18.tile =
  {
    wire = merge_wire (remove_fields tile.wire [ "connections" ]) patch.wire;
    owner =
      (match patch.owner with
      | None -> tile.owner
      | Some `Null -> `Null
      | Some (`String value) -> `String value
      | Some `Undefined -> invalid "Undefined frozen tile owner");
    id = patch.id;
    location = optional_merge patch.location tile.location;
    col = choose patch.col tile.col;
    row = choose patch.row tile.row;
    terrain = choose patch.terrain tile.terrain;
    connections = patch.connections;
    dead_ends = optional_merge patch.dead_ends tile.dead_ends;
    walls = optional_merge patch.walls tile.walls;
    floor = optional_merge patch.floor tile.floor;
    fog = optional_merge patch.fog tile.fog;
    light = optional_merge patch.light tile.light;
  }

let state18 (state : V18.state) : V19.state =
  if state.map_id <> "lantern-villages" || state.map_revision <> 4. then state
  else
    let old_keep = find (fun (tile : V18.tile) -> tile.id = "16,4") state.tiles in
    let occupant = find (fun (unit_ : V18.unit_) -> unit_.tile = "15,7") state.units in
    let owner =
      match occupant with
      | Some unit_ -> `String (match unit_.side with `Player -> "player" | `Enemy -> "enemy")
      | None ->
          Option.fold ~none:(`String "enemy") ~some:(fun (tile : V18.tile) -> tile.owner) old_keep
    in
    let changes = C.Fragment.delta19 (Lazy.force Frozen_data.history_lantern_villages_v18) in
    let tiles =
      Array.map
        (fun (tile : V18.tile) ->
          match find (fun (change : Fragments.delta19_item) -> change.id = tile.id) changes with
          | None -> tile
          | Some change ->
              let patched = patch_tile tile change in
              {
                patched with
                owner =
                  (if change.id = "15,7" then owner
                   else if change.id = "16,4" then `Null
                   else tile.owner);
              })
        state.tiles
    in
    { state with map_revision = 5.; tiles }

let state19 (state : V19.state) : V20.state =
  if state.map_id <> "lantern-villages" || state.map_revision <> 5. then state
  else
    let changes = C.Fragment.delta20 (Lazy.force Frozen_data.history_lantern_villages_v19) in
    let tiles =
      Array.map
        (fun (tile : V19.tile) ->
          match find (fun (change : Fragments.delta19_item) -> change.id = tile.id) changes with
          | None -> tile
          | Some change -> patch_tile tile change)
        state.tiles
    in
    { state with map_revision = 6.; tiles }

let state20 (old : V20.state) : V21.state =
  let frozen = D.economy C.Fragment.delta21 C.Fragment.factions21 in
  let old_gold =
    List.fold_left
      (fun gold key ->
        match key with
        | "player" -> Map.add key old.gold.player gold
        | "enemy" -> Map.add key old.gold.enemy gold
        | _ -> (
            match List.assoc_opt key old.gold.wire.extra with
            | None -> gold
            | Some value -> Map.add key (C.read_json_number value) gold))
      Map.empty
      (old.gold.wire.order @ [ "player"; "enemy" ])
  in
  let gold =
    Array.fold_left
      (fun gold (faction : V20.faction) ->
        if Map.mem faction.id gold then gold else Map.add faction.id 0. gold)
      old_gold old.factions
  in
  let factions = old.factions in
  let condition_memory =
    if old.map_id = frozen.map_id then
      Option.map (filter (( <> ) "group:swamp-dead")) old.condition_memory
    else old.condition_memory
  in
  let units, tiles, factions, map_revision =
    if old.map_id <> frozen.map_id || old.map_revision <> frozen.from_revision then
      (old.units, old.tiles, factions, old.map_revision)
    else
      let units =
        Array.map
          (fun (unit_ : V20.unit_) ->
            if unit_.name = Some frozen.leader.name && unit_.faction = Some frozen.leader.faction
            then { unit_ with role = Some `Leader; ai_group = Some frozen.leader.ai_group }
            else unit_)
          old.units
      in
      let tiles =
        Array.map
          (fun (tile : V20.tile) ->
            match
              find (fun (patch : Fragments.delta19_item) -> patch.id = tile.id) frozen.tiles
            with
            | None -> tile
            | Some change ->
                let occupant = find (fun (unit_ : V20.unit_) -> unit_.tile = tile.id) units in
                let fallback =
                  match occupant with
                  | Some unit_ ->
                      `String
                        (Option.value unit_.faction
                           ~default:
                             (match unit_.side with `Player -> "player" | `Enemy -> "enemy"))
                  | None -> (
                      match change.owner with Some (`String owner) -> `String owner | _ -> `Null)
                in
                let owner =
                  match tile.owner with `Null -> fallback | `String _ as owner -> owner
                in
                { (patch_tile tile change) with owner })
          old.tiles
      in
      let factions =
        Array.map
          (fun (faction : V20.faction) ->
            if faction.id <> frozen.faction.id then faction
            else
              {
                wire = merge_wire faction.wire frozen.faction.wire;
                id = frozen.faction.id;
                name = optional_merge frozen.faction.name faction.name;
                phase = frozen.faction.phase;
                controller = frozen.faction.controller;
                palette = frozen.faction.palette;
                recruits = optional_merge frozen.faction.recruits faction.recruits;
                relations = optional_merge frozen.faction.relations faction.relations;
                transfer = optional_merge frozen.faction.transfer faction.transfer;
              })
          factions
      in
      (units, tiles, factions, frozen.to_revision)
  in
  {
    wire = old.wire;
    round = old.round;
    turn = old.turn;
    winner = old.winner;
    armies = old.armies;
    condition_memory;
    map_id = old.map_id;
    map_revision;
    cols = old.cols;
    rows = old.rows;
    next_id = old.next_id;
    log = old.log;
    last_combat = old.last_combat;
    units;
    tiles;
    campaign = old.campaign;
    gold;
    factions;
    ground_items = old.ground_items;
    combat_rng = old.combat_rng;
  }

let state21 (old : V21.state) : V22.state =
  if old.map_id <> "lantern-villages" || old.map_revision <> 7. then old
  else
    let tiles =
      Array.map
        (fun (tile : V21.tile) ->
          if tile.id <> "13,7" && tile.id <> "16,10" then tile
          else
            {
              tile with
              wire = remove_fields tile.wire [ "connections" ];
              connections = None;
              terrain = (if tile.id = "13,7" then "forest" else "marsh");
              floor = Some (if tile.id = "13,7" then "green-forest" else "marsh");
            })
        old.tiles
    in
    { old with tiles; map_revision = 8. }

let state22 (old : V22.state) : V23.state =
  let maps = D.lighting C.Fragment.light23 in
  let lights =
    match lookup old.map_id maps with
    | Some map when map.revision = old.map_revision -> map.lights
    | _ -> [||]
  in
  let tiles =
    Array.map
      (fun (tile : V22.tile) ->
        match find (fun (entry : V23.light D.light_entry) -> entry.tile = tile.id) lights with
        | None -> tile
        | Some entry -> { tile with light = Some entry.light })
      old.tiles
  in
  let condition_memory =
    Option.map
      (Array.map (fun key ->
           match find (fun (entry : V23.light D.light_entry) -> entry.legacy_key = key) lights with
           | None -> key
           | Some entry -> "light:" ^ entry.tile))
      old.condition_memory
  in
  { old with tiles; condition_memory }

let state23 (old : V23.state) : V24.state =
  let defaults = C.Fragment.factions24 (Lazy.force Frozen_data.history_base_factions_v23) in
  let factions =
    Array.fold_left
      (fun existing (faction : V24.faction) ->
        if Array.exists (fun (entry : V23.faction) -> entry.id = faction.id) existing then existing
        else Array.append existing [| faction |])
      old.factions defaults
  in
  { old with factions }

let state24 (old : V24.state) : V25.state =
  let player = find (fun (faction : V24.faction) -> faction.id = "player") old.factions in
  let enemy = find (fun (faction : V24.faction) -> faction.id = "enemy") old.factions in
  let explicit (faction : V24.faction) other =
    Option.fold ~none:false ~some:(Map.mem other) faction.relations
  in
  let factions =
    match (player, enemy) with
    | Some player, Some enemy when not (explicit player "enemy" || explicit enemy "player") ->
        Array.map
          (fun (faction : V24.faction) ->
            if faction.id <> "player" && faction.id <> "enemy" then faction
            else
              {
                faction with
                relations =
                  Some
                    (Map.add
                       (if faction.id = "player" then "enemy" else "player")
                       `Hostile
                       (Option.value faction.relations ~default:Map.empty));
              })
          old.factions
    | _ -> old.factions
  in
  { old with factions }

let state25 (old : V25.state) : V26.state =
  let frozen = Lazy.force D.characters in
  let campaign_id =
    match old.campaign with
    | Some campaign -> Some campaign.id
    | None -> lookup old.map_id frozen.maps
  in
  let registry = Option.bind campaign_id (fun id -> lookup id frozen.campaigns) in
  let id name =
    match registry with
    | None -> invalid "Unknown historical character"
    | Some registry -> (
        match lookup name registry.aliases with
        | Some character when character <> "" -> character
        | _ ->
            if
              Array.exists
                (fun (character : D.character) -> character.id = name)
                registry.characters
            then name
            else invalid ("Unknown historical character " ^ name))
  in
  let named name character =
    Option.map id (match character with Some _ -> character | None -> name)
  in
  let rec condition (value : V25.condition) : V26.condition =
    match value with
    | `Owner value -> `Owner value
    | `Visible_units value -> `Visible_units value
    | `Choice value -> `Choice value
    | `Flag value -> `Flag value
    | `Event value -> `Event value
    | `Round value -> `Round value
    | `Object10 value ->
        `Object10
          {
            wire = value.wire;
            kind = value.kind;
            conditions = Array.map condition value.conditions;
          }
    | `Proximity value ->
        `Proximity
          {
            wire = remove_fields value.wire [ "name" ];
            kind = value.kind;
            unit_types = value.unit_types;
            tile = value.tile;
            radius = value.radius;
            side = value.side;
            unit_type = value.unit_type;
            character = named value.name value.character;
          }
    | `Has_units value ->
        `Has_units
          {
            wire = remove_fields value.wire [ "name" ];
            kind = value.kind;
            ai_group = value.ai_group;
            side = value.side;
            unit_type = value.unit_type;
            character = named value.name value.character;
          }
    | `No_units value ->
        `No_units
          {
            wire = remove_fields value.wire [ "name" ];
            kind = value.kind;
            side = value.side;
            unit_type = value.unit_type;
            character = named value.name value.character;
          }
  in
  let rule (value : V25.rule) : V26.rule =
    { wire = value.wire; condition = condition value.condition; remember = value.remember }
  in
  let light (value : V25.light) : V26.light =
    {
      wire = value.wire;
      radius = value.radius;
      color = value.color;
      fixture = value.fixture;
      dx = value.dx;
      dy = value.dy;
      power = value.power;
      when_ = Option.map rule value.when_;
    }
  in
  let unit_ (value : V25.unit_) : V26.unit_ =
    {
      wire = remove_fields value.wire [ "name" ];
      tile = value.tile;
      id = value.id;
      side = value.side;
      role = value.role;
      faction = value.faction;
      items = value.items;
      ai_group = value.ai_group;
      type_ = value.type_;
      hp = value.hp;
      moved = value.moved;
      attacked = value.attacked;
      retreating = value.retreating;
      rebuilding = value.rebuilding;
      character = named value.name value.character;
      experience = value.experience;
    }
  in
  let veteran (value : V25.veteran) : V26.veteran =
    {
      wire = remove_fields value.wire [ "name" ];
      role = value.role;
      experience = value.experience;
      items = value.items;
      type_ = value.type_;
      character = named value.name value.character;
    }
  in
  let tiles =
    Array.map
      (fun (value : V25.tile) : V26.tile ->
        {
          wire = value.wire;
          owner = value.owner;
          id = value.id;
          location = value.location;
          col = value.col;
          row = value.row;
          terrain = value.terrain;
          connections = value.connections;
          dead_ends = value.dead_ends;
          walls = value.walls;
          floor = value.floor;
          fog = Option.map rule value.fog;
          light = Option.map light value.light;
        })
      old.tiles
  in
  let factions =
    Array.map
      (fun (value : V25.faction) : V26.faction ->
        {
          wire = value.wire;
          id = value.id;
          name = value.name;
          phase = value.phase;
          controller = value.controller;
          palette = value.palette;
          recruits = value.recruits;
          relations = value.relations;
          transfer =
            Option.map
              (fun (value : V25.transfer) : V26.transfer ->
                { wire = value.wire; to_ = value.to_; when_ = rule value.when_ })
              value.transfer;
        })
      old.factions
  in
  let campaign =
    Option.map
      (fun (value : V25.campaign) : V26.campaign ->
        let dialogue =
          Array.map
            (fun (line : V25.dialogue) ->
              let character =
                match line.character with
                | Some character when character <> "" -> Some (id character)
                | _ ->
                    Option.bind registry (fun registry ->
                        Option.bind line.speaker (fun speaker ->
                            if speaker = "" then None else lookup speaker registry.aliases))
              in
              match character with
              | Some character when character <> "" ->
                  { line with character = Some character; speaker = Some "" }
              | _ -> line)
            value.dialogue
        in
        {
          wire = value.wire;
          mission = value.mission;
          id = value.id;
          vigil = value.vigil;
          refuge = value.refuge;
          completed = value.completed;
          marks =
            Array.map
              (fun mark ->
                if starts "fallen:" mark then "fallen:" ^ id (suffix "fallen:" mark) else mark)
              value.marks;
          holds = value.holds;
          recall = value.recall;
          fallen_companions = Option.map (Array.map id) value.fallen_companions;
          carry_gold = value.carry_gold;
          dialogue;
          dialogue_index = value.dialogue_index;
          outcome = value.outcome;
          veterans = Option.map (Array.map veteran) value.veterans;
          story_veterans = Option.map (Array.map veteran) value.story_veterans;
          evacuated =
            Option.map
              (Array.map (fun (value : V25.evacuated) : V26.evacuated ->
                   { wire = value.wire; event = value.event; unit_ = unit_ value.unit_ }))
              value.evacuated;
          choices = value.choices;
        })
      old.campaign
  in
  {
    wire = old.wire;
    round = old.round;
    turn = old.turn;
    winner = old.winner;
    armies = old.armies;
    condition_memory = old.condition_memory;
    map_id = old.map_id;
    map_revision = old.map_revision;
    cols = old.cols;
    rows = old.rows;
    next_id = old.next_id;
    log = old.log;
    last_combat = old.last_combat;
    units = Array.map unit_ old.units;
    tiles;
    campaign;
    gold = old.gold;
    factions;
    ground_items = old.ground_items;
    combat_rng = old.combat_rng;
  }

let state26 (old : V26.state) : V27.state =
  let ids =
    match lookup old.map_id (Lazy.force D.boards) with
    | None -> invalid "Unknown historical board"
    | Some ids -> ids
  in
  let tile input =
    match lookup input ids with
    | Some id when id <> "" -> id
    | _ -> invalid "Unknown historical tile"
  in
  let rec condition (value : V26.condition) : V27.condition =
    match value with
    | `Owner value -> `Owner { value with tile = Option.map tile value.tile }
    | `Proximity value -> `Proximity { value with tile = Option.map tile value.tile }
    | `Object10 value -> `Object10 { value with conditions = Array.map condition value.conditions }
    | `Visible_units _ | `Has_units _ | `No_units _ | `Choice _ | `Flag _ | `Event _ | `Round _ ->
        value
  in
  let rule (value : V26.rule) : V27.rule = { value with condition = condition value.condition } in
  let light (value : V26.light) : V27.light = { value with when_ = Option.map rule value.when_ } in
  let unit_ (value : V26.unit_) : V27.unit_ = { value with tile = tile value.tile } in
  let tiles =
    Array.map
      (fun (value : V26.tile) ->
        {
          value with
          id = tile value.id;
          fog = Option.map rule value.fog;
          light = Option.map light value.light;
        })
      old.tiles
  in
  let factions =
    Array.map
      (fun (value : V26.faction) ->
        {
          value with
          transfer =
            Option.map
              (fun (value : V26.transfer) -> { value with when_ = rule value.when_ })
              value.transfer;
        })
      old.factions
  in
  let campaign =
    Option.map
      (fun (value : V26.campaign) ->
        {
          value with
          marks = Array.map (marker tile) value.marks;
          holds =
            Option.map
              (fun values ->
                Map.of_list (List.map (fun (key, value) -> (tile key, value)) (Map.bindings values)))
              value.holds;
          evacuated =
            Option.map
              (Array.map (fun (value : V26.evacuated) -> { value with unit_ = unit_ value.unit_ }))
              value.evacuated;
        })
      old.campaign
  in
  {
    old with
    tiles;
    units = Array.map unit_ old.units;
    ground_items =
      Array.map
        (fun (value : V26.ground_items) -> { value with tile = tile value.tile })
        old.ground_items;
    factions;
    condition_memory = Option.map (Array.map (memory tile)) old.condition_memory;
    campaign;
  }

let campaign28 (old : V27.campaign) scripts : V28.campaign =
  let scripts =
    match scripts with
    | Some _ -> scripts
    | None -> (
        match List.assoc_opt "scripts" old.wire.extra with
        | None -> None
        | Some value when Migration_json.is_undefined value -> None
        | Some value -> Some (C.read_save28_checkpoint_state_campaign_member1_scripts_member1 value)
        )
  in
  {
    wire = old.wire;
    mission = old.mission;
    id = old.id;
    vigil = old.vigil;
    refuge = old.refuge;
    completed = old.completed;
    marks = old.marks;
    holds = old.holds;
    recall = old.recall;
    fallen_companions = old.fallen_companions;
    carry_gold = old.carry_gold;
    dialogue = old.dialogue;
    dialogue_index = old.dialogue_index;
    outcome = old.outcome;
    veterans = old.veterans;
    story_veterans = old.story_veterans;
    evacuated = old.evacuated;
    scripts;
    choices = old.choices;
  }

let state27 (old : V27.state) : V28.state =
  let recipe = lookup old.map_id (Lazy.force D.events) in
  let units, tiles, factions, campaign =
    match (old.campaign, recipe) with
    | Some campaign, Some recipe ->
        let marks = unique campaign.marks in
        let flags = ref (unique (Option.value campaign.choices ~default:[||])) in
        let active = ref (unique recipe.event_ids) in
        let add target id =
          if not (member id !target) then target := Array.append !target [| id |]
        in
        let retire id = active := filter (( <> ) id) !active in
        Array.iter
          (fun id ->
            if member ("event:" ^ id) marks then (
              retire (Option.value (lookup id recipe.aliases) ~default:id);
              add flags id;
              Option.iter (add flags) (lookup id recipe.choice_flags)))
          recipe.old_event_ids;
        Array.iter (fun id -> if member ("wave:" ^ id) marks then retire id) recipe.waves;
        let recruitment = ref Map.empty in
        Array.iter
          (fun (event, pools) ->
            if member ("event:" ^ event) marks then
              Array.iter
                (fun (faction, recruits) ->
                  let pool = ref (Option.value (Map.find_opt faction !recruitment) ~default:[||]) in
                  Array.iter
                    (fun (recruit : D.recruit) ->
                      if
                        not
                          (Array.exists
                             (fun (other : V28.recruits) -> other.unit_ = recruit.unit)
                             !pool)
                      then
                        pool :=
                          Array.append !pool
                            [| { wire = empty_wire; unit_ = recruit.unit; cost = recruit.cost } |])
                    recruits;
                  recruitment := Map.add faction !pool !recruitment)
                pools)
          recipe.recruitment_grants;
        let member_ (source : D.historical_member) :
            fragments_native_state38_campaign_member1_scripts_member1_pending_item_units_item =
          let role =
            Option.map
              (function
                | "leader" -> `Leader
                | "courier" -> `Courier
                | "captive" -> `Captive
                | _ -> invalid "Invalid frozen role")
              source.role
          in
          {
            wire =
              merge_wire
                (remove_fields source.wire [ "oldId"; "unitType"; "role" ])
                { empty_wire with order = [ "type"; "role" ] };
            type_ = source.unit_type;
            character = Option.map (fun character -> `String character) source.character;
            role;
            items = Some source.items;
          }
        in
        let pending = ref [||] in
        let ai_group = function
          | D.Missing -> Some `Undefined
          | D.Null -> Some `Null
          | D.Value value -> Some (`String value)
        in
        let add_pending area faction ai_group units =
          if Array.length units > 0 then
            pending :=
              Array.append !pending
                [|
                  ({
                     wire = { empty_wire with order = [ "area"; "faction"; "aiGroup"; "units" ] };
                     area;
                     faction;
                     ai_group;
                     units;
                   }
                    : V28.pending);
                |]
        in
        Array.iter
          (fun (group : D.spawn) ->
            if member ("event:" ^ group.event) marks then
              let units =
                filter_map
                  (fun (unit_ : D.historical_member) ->
                    if member ("spawn:" ^ group.event ^ ":" ^ unit_.old_id) marks then None
                    else Some (member_ unit_))
                  group.units
              in
              add_pending group.area group.faction (ai_group group.ai_group) units)
          recipe.spawns;
        let units =
          Array.map
            (fun (unit_ : V27.unit_) ->
              if unit_.type_ = "captives" then { unit_ with role = Some `Captive } else unit_)
            old.units
        in
        let units =
          if old.map_id <> "lantern-ferry" then units
          else (
            if member "event:ford-shout" marks then (
              add flags "ferryman-saved";
              add flags "ferryman-alive")
            else if old.round >= 16. then (
              add flags "ferryman-dead";
              retire "ferryman-death");
            let bears =
              filter
                (fun (unit_ : V27.unit_) -> unit_.type_ = "bear" && unit_.faction = Some "wildlife")
                units
            in
            if member "spawn:quarry-bear:member-1" marks && Array.length bears = 1 then
              Array.map
                (fun (unit_ : V27.unit_) ->
                  if unit_ == bears.(0) then { unit_ with character = Some "quarry-bear" }
                  else unit_)
                units
            else units)
        in
        Array.iter
          (fun (unit_ : V27.unit_) ->
            if unit_.side = `Player then
              Option.iter (fun character -> retire ("rescue-" ^ character)) unit_.character)
          units;
        let holds =
          filter_map
            (fun (hold : D.hold) ->
              Option.bind campaign.holds (fun counters ->
                  Option.map
                    (fun (counter : fragments_native_state38_campaign_member1_vigil_member1) :
                         fragments_native_state38_campaign_member1_scripts_member1_holds_item ->
                      {
                        wire = empty_wire;
                        event = hold.event;
                        predicate = hold.predicate;
                        turns = counter.held_turns;
                        last_round = counter.last_counted_round;
                      })
                    (Map.find_opt hold.tile counters)))
            recipe.holds
        in
        let map_marks = ref [||] in
        Option.iter
          (fun (ritual : D.ritual) ->
            let source marker =
              let body =
                String.concat ":" (List.tl (List.tl (List.tl (String.split_on_char ':' marker))))
              in
              if starts "mark-" body then suffix "mark-" body
              else
                match
                  Js.Re.exec ~str:(Migration_utf8.to_js body)
                    (Js.Re.fromString "^death-u[1-9]\\d*-(.+)$")
                with
                | None -> invalid "Invalid historical ritual group"
                | Some result ->
                    Migration_utf8.of_js
                      (Option.get (Js.Nullable.toOption (Js.Re.captures result).(1)))
            in
            let area id =
              let origin =
                match find (fun (tile : V27.tile) -> tile.id = id) old.tiles with
                | None -> invalid "Unknown historical ritual location"
                | Some origin -> origin
              in
              old.tiles
              |> Array.mapi (fun index (tile : V27.tile) ->
                  let r = tile.row -. origin.row in
                  let q =
                    tile.col -. floor (tile.row /. 2.) -. origin.col +. floor (origin.row /. 2.)
                  in
                  (tile, index, (abs_float q +. abs_float r +. abs_float (q +. r)) /. 2.))
              |> filter (fun (_, _, d) -> d <= ritual.radius)
              |> Array.to_list
              |> List.sort (fun (_, i, d) (_, j, e) ->
                  let c = Float.compare d e in
                  if c = 0 then Int.compare i j else c)
              |> List.map (fun ((tile : V27.tile), _, _) -> tile.id)
              |> Array.of_list
            in
            Array.iter
              (fun marker ->
                (if starts ("charge:" ^ ritual.id ^ ":") marker then
                   let light =
                     if
                       ritual.light = Js.Json.null
                       || ritual.light = Js.Json.boolean false
                       || ritual.light = Js.Json.number 0.
                       || ritual.light = Js.Json.string ""
                     then None
                     else
                       match C.Fragment.light28 ritual.light with
                       | `Null -> None
                       | `Object2 light -> Some light
                   in
                   let mark : save28_checkpoint_state_campaign_member1_scripts_member1_marks_item =
                     {
                       wire = empty_wire;
                       id = float_of_int (Array.length !map_marks + 1);
                       type_ = "captive-death";
                       tile =
                         String.concat ":"
                           (List.tl (List.tl (List.tl (String.split_on_char ':' marker))));
                       light;
                       light_remembered = None;
                     }
                   in
                   map_marks := Array.append !map_marks [| mark |]);
                if starts ("ritual:" ^ ritual.id ^ ":") marker then
                  let units =
                    filter_map
                      (fun (unit_ : D.historical_member) ->
                        if member ("spawn:" ^ marker ^ ":" ^ unit_.old_id) marks then None
                        else Some (member_ unit_))
                      ritual.units
                  in
                  if Array.length units > 0 then
                    add_pending
                      (area (source marker))
                      "enemy"
                      (Some (`String ritual.ai_group))
                      units)
              marks;
            Array.iteri
              (fun index id ->
                if index < Array.length ritual.sources then
                  match
                    find (fun (tile : V27.tile) -> tile.id = ritual.sources.(index)) old.tiles
                  with
                  | Some tile when tile.terrain = "ritual-spent" ->
                      retire id;
                      add flags id
                  | _ -> ())
              [| "first-toll"; "second-toll"; "third-toll" |])
          recipe.ritual;
        let rec condition (value : V27.condition) : V28.condition =
          match value with
          | `Event value -> `Flag { wire = empty_wire; kind = `Flag; flag = value.event }
          | `Object10 value ->
              `Object10 { value with conditions = Array.map condition value.conditions }
          | `Owner _ | `Proximity _ | `Visible_units _ | `Has_units _ | `No_units _ | `Choice _
          | `Flag _ | `Round _ ->
              value
        in
        let rule (value : V27.rule) : V28.rule =
          { value with condition = condition value.condition }
        in
        let light (value : V27.light) : V28.light =
          { value with when_ = Option.map rule value.when_ }
        in
        let tiles =
          Array.map
            (fun (tile : V27.tile) ->
              { tile with fog = Option.map rule tile.fog; light = Option.map light tile.light })
            old.tiles
        in
        let factions =
          Array.map
            (fun (faction : V27.faction) ->
              {
                faction with
                transfer =
                  (if member faction.id recipe.removed_transfers then None
                   else
                     Option.map
                       (fun (transfer : V27.transfer) ->
                         { transfer with when_ = rule transfer.when_ })
                       faction.transfer);
              })
            old.factions
        in
        let scripts : V28.scripts =
          {
            wire =
              {
                empty_wire with
                order = [ "active"; "holds"; "marks"; "pending"; "nextMark"; "recruitment" ];
              };
            active = !active;
            holds;
            marks = !map_marks;
            pending = !pending;
            next_mark = float_of_int (Array.length !map_marks + 1);
            recruitment = !recruitment;
            failed = None;
            completed = None;
            revealed = None;
          }
        in
        let campaign =
          {
            (campaign28 campaign (Some scripts)) with
            wire =
              merge_wire campaign.wire
                {
                  empty_wire with
                  order =
                    [ "fallenCompanions"; "marks"; "holds"; "evacuated"; "choices"; "scripts" ];
                };
            fallen_companions =
              Some
                (unique
                   (Array.append
                      (Option.value campaign.fallen_companions ~default:[||])
                      (filter_map
                         (fun mark ->
                           if starts "fallen:" mark then Some (suffix "fallen:" mark) else None)
                         marks)));
            marks = [||];
            holds = Some Map.empty;
            evacuated = Some [||];
            choices = Some !flags;
          }
        in
        (units, tiles, factions, Some campaign)
    | campaign, _ ->
        ( old.units,
          old.tiles,
          old.factions,
          Option.map (fun campaign -> campaign28 campaign None) campaign )
  in
  {
    wire = old.wire;
    round = old.round;
    turn = old.turn;
    winner = old.winner;
    armies = old.armies;
    condition_memory = old.condition_memory;
    map_id = old.map_id;
    map_revision = old.map_revision;
    cols = old.cols;
    rows = old.rows;
    next_id = old.next_id;
    log = old.log;
    last_combat = old.last_combat;
    units;
    tiles;
    campaign;
    gold = old.gold;
    factions;
    ground_items = old.ground_items;
    combat_rng = old.combat_rng;
  }

let state28 (old : V28.state) : V29.state =
  let campaign =
    Option.map
      (fun (campaign : V28.campaign) : V29.campaign ->
        require
          (Migration_json.safe_integer campaign.mission
          && campaign.mission >= 0.
          && Migration_json.safe_integer old.round
          && old.round >= 1.)
          "Invalid historical flags";
        let origins = Option.value (lookup campaign.id (Lazy.force D.origins)) ~default:[||] in
        let flags =
          Array.fold_left
            (fun flags id ->
              let source = lookup id origins in
              let produced =
                match source with
                | Some source when source <= campaign.mission -> source
                | _ -> campaign.mission
              in
              let change : fragments_native_state38_campaign_member1_flags_value_item =
                {
                  wire = empty_wire;
                  state = true;
                  stage = produced;
                  turn = (if source = Some campaign.mission then old.round else 0.);
                }
              in
              Map.add id [| change |] flags)
            Map.empty
            (Option.value campaign.choices ~default:[||])
        in
        {
          wire = remove_fields campaign.wire [ "choices" ];
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
          scripts = campaign.scripts;
          flags;
        })
      old.campaign
  in
  {
    wire = old.wire;
    round = old.round;
    turn = old.turn;
    winner = old.winner;
    armies = old.armies;
    condition_memory = old.condition_memory;
    map_id = old.map_id;
    map_revision = old.map_revision;
    cols = old.cols;
    rows = old.rows;
    next_id = old.next_id;
    log = old.log;
    last_combat = old.last_combat;
    units = old.units;
    tiles = old.tiles;
    campaign;
    gold = old.gold;
    factions = old.factions;
    ground_items = old.ground_items;
    combat_rng = old.combat_rng;
  }

let v18 (value : V18.t) : V19.t =
  {
    wire = value.wire;
    version = `V19;
    checkpoint =
      {
        wire = value.checkpoint.wire;
        seed = value.checkpoint.seed;
        state = state18 value.checkpoint.state;
      };
    pending_enemy = value.pending_enemy;
    setup = value.setup;
    progress = value.progress;
    undo = value.undo;
  }

let v19 (value : V19.t) : V20.t =
  {
    wire = value.wire;
    version = `V20;
    checkpoint =
      {
        wire = value.checkpoint.wire;
        seed = value.checkpoint.seed;
        state = state19 value.checkpoint.state;
      };
    pending_enemy = value.pending_enemy;
    setup = value.setup;
    progress = value.progress;
    undo = value.undo;
  }

let v20 (value : V20.t) : V21.t =
  {
    wire = value.wire;
    version = `V21;
    checkpoint =
      {
        wire = value.checkpoint.wire;
        seed = value.checkpoint.seed;
        state = state20 value.checkpoint.state;
      };
    pending_enemy = value.pending_enemy;
    setup = value.setup;
    progress = value.progress;
    undo = value.undo;
  }

let v21 (value : V21.t) : V22.t =
  {
    wire = value.wire;
    version = `V22;
    checkpoint =
      {
        wire = value.checkpoint.wire;
        seed = value.checkpoint.seed;
        state = state21 value.checkpoint.state;
      };
    pending_enemy = value.pending_enemy;
    setup = value.setup;
    progress = value.progress;
    undo = value.undo;
  }

let v22 (value : V22.t) : V23.t =
  {
    wire = value.wire;
    version = `V23;
    checkpoint =
      {
        wire = value.checkpoint.wire;
        seed = value.checkpoint.seed;
        state = state22 value.checkpoint.state;
      };
    pending_enemy = value.pending_enemy;
    setup = value.setup;
    progress = value.progress;
    undo = value.undo;
  }

let v23 (value : V23.t) : V24.t =
  {
    wire = value.wire;
    version = `V24;
    checkpoint =
      {
        wire = value.checkpoint.wire;
        seed = value.checkpoint.seed;
        state = state23 value.checkpoint.state;
      };
    pending_enemy = value.pending_enemy;
    setup = value.setup;
    progress = value.progress;
    undo = value.undo;
  }

let v24 (value : V24.t) : V25.t =
  {
    wire = value.wire;
    version = `V25;
    checkpoint =
      {
        wire = value.checkpoint.wire;
        seed = value.checkpoint.seed;
        state = state24 value.checkpoint.state;
      };
    pending_enemy = value.pending_enemy;
    setup = value.setup;
    progress = value.progress;
    undo = value.undo;
  }

let v25 (value : V25.t) : V26.t =
  {
    wire = value.wire;
    version = `V26;
    checkpoint =
      {
        wire = value.checkpoint.wire;
        seed = value.checkpoint.seed;
        state = state25 value.checkpoint.state;
      };
    pending_enemy = value.pending_enemy;
    setup = value.setup;
    progress = value.progress;
    undo = value.undo;
  }

let v26 (value : V26.t) : V27.t =
  {
    wire = value.wire;
    version = `V27;
    checkpoint =
      {
        wire = value.checkpoint.wire;
        seed = value.checkpoint.seed;
        state = state26 value.checkpoint.state;
      };
    pending_enemy = value.pending_enemy;
    setup = value.setup;
    progress = value.progress;
    undo = value.undo;
  }

let v27 (value : V27.t) : V28.t =
  {
    wire = value.wire;
    version = `V28;
    checkpoint =
      {
        wire = value.checkpoint.wire;
        seed = value.checkpoint.seed;
        state = state27 value.checkpoint.state;
      };
    pending_enemy = value.pending_enemy;
    setup = value.setup;
    progress = value.progress;
    undo = value.undo;
  }

let v28 (value : V28.t) : V29.t =
  {
    wire = value.wire;
    version = `V29;
    checkpoint =
      {
        wire = value.checkpoint.wire;
        seed = value.checkpoint.seed;
        state = state28 value.checkpoint.state;
      };
    pending_enemy = value.pending_enemy;
    setup = value.setup;
    progress = value.progress;
    undo = value.undo;
  }
