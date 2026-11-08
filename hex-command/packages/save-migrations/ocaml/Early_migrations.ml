(* Sequential ports of the frozen TypeScript v1-v17 conversions. *)
open Historical_types
module C = Historical_codec
module J = Migration_json

let require condition message = if not condition then invalid_arg message

let inserted wire keys =
  { wire with order = wire.order @ List.filter (fun key -> not (List.mem key wire.order)) keys }

let add_unique item values =
  if Array.mem item values then values else Array.append values [| item |]

let filter_map f values = Array.of_list (List.filter_map f (Array.to_list values))
let filter f values = Array.of_list (List.filter f (Array.to_list values))

let frozen_map id =
  List.find_opt (fun map -> J.text map "id" = id) (J.list Save_history.catalog "maps")

let map_revision map = Option.fold ~none:1. ~some:J.number (J.get map "revision")

let v1_native (value : Native1.t) : V2.t =
  {
    version = `V2;
    checkpoint = { seed = value.saved.seed; state = value.saved.state; wire = value.saved.wire };
    pending_enemy = value.pending_enemy;
    planner = None;
    setup = None;
    progress = None;
    undo = None;
    wire = empty_wire;
  }

(* These recursive types differ only in the frozen historical union ordering. *)
let rec condition2 (value : save2_checkpoint_state_tiles_item_fog_member1_condition) :
    fragments_rule11_condition =
  match value with
  | `Owner x -> `Owner x
  | `Visible_units x -> `Visible_units x
  | `Choice x -> `Choice x
  | `Flag x -> `Flag x
  | `Event x -> `Event x
  | `Round x -> `Round x
  | `Proximity x -> `Proximity x
  | `Has_units x -> `Has_units x
  | `No_units x -> `No_units x
  | `Object7 x ->
      `Object10 { kind = x.kind; conditions = Array.map condition2 x.conditions; wire = x.wire }

let rule2 (value : save2_checkpoint_state_tiles_item_fog_member1) : fragments_rule11 =
  { condition = condition2 value.condition; remember = value.remember; wire = value.wire }

let light2 (value : save2_checkpoint_state_tiles_item_light_member1) : fragments_light23 =
  {
    radius = value.radius;
    color = value.color;
    fixture = value.fixture;
    dx = value.dx;
    dy = value.dy;
    power = value.power;
    when_ = Option.map rule2 value.when_;
    wire = value.wire;
  }

let tile2 (value : V2.tile) : V3.tile =
  {
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
    fog = Option.map rule2 value.fog;
    light = Option.map light2 value.light;
    wall_corners = value.wall_corners;
    wire = value.wire;
  }

let faction2 (value : V2.faction) : V3.faction =
  {
    id = value.id;
    name = value.name;
    phase = value.phase;
    controller = value.controller;
    palette = value.palette;
    recruits = value.recruits;
    relations = value.relations;
    transfer =
      Option.map
        (fun (x : save2_checkpoint_state_factions_member1_item_transfer_member1) ->
          ({ to_ = x.to_; when_ = rule2 x.when_; wire = x.wire }
            : fragments_factions7_item_transfer_member1))
        value.transfer;
    wire = value.wire;
  }

let v2 (value : V2.t) : V3.t =
  let old = value.checkpoint.state in
  let revision =
    if old.map_revision = 1. && old.map_id = "ashen-refuge" then
      match frozen_map old.map_id with
      | Some map -> map_revision map
      | None -> invalid_arg "Unknown historical map"
    else old.map_revision
  in
  let state : V3.state =
    {
      round = old.round;
      turn = old.turn;
      winner = old.winner;
      armies = old.armies;
      condition_memory = old.condition_memory;
      map_id = old.map_id;
      map_revision = revision;
      cols = old.cols;
      rows = old.rows;
      next_id = old.next_id;
      log = old.log;
      last_combat = old.last_combat;
      units = old.units;
      tiles = Array.map tile2 old.tiles;
      campaign = old.campaign;
      gold = old.gold;
      factions = Option.map (Array.map faction2) old.factions;
      ground_items = old.ground_items;
      combat_rng = old.combat_rng;
      wire = old.wire;
    }
  in
  {
    version = `V3;
    checkpoint = { seed = value.checkpoint.seed; state; wire = value.checkpoint.wire };
    pending_enemy = value.pending_enemy;
    planner = value.planner;
    setup = value.setup;
    progress = value.progress;
    undo = value.undo;
    wire = value.wire;
  }

let finish3 (value : V3.t) (state : V4.state) : V4.t =
  {
    version = `V4;
    checkpoint = { seed = value.checkpoint.seed; state; wire = value.checkpoint.wire };
    pending_enemy = value.pending_enemy;
    planner = value.planner;
    setup = value.setup;
    progress = value.progress;
    undo = value.undo;
    wire = value.wire;
  }

let finish4 (value : V4.t) (state : V5.state) : V5.t =
  {
    version = `V5;
    checkpoint =
      {
        seed = value.checkpoint.seed;
        state;
        wire = { empty_wire with order = [ "seed"; "state" ] };
      };
    pending_enemy = value.pending_enemy;
    setup = None;
    progress = None;
    undo = None;
    wire = { empty_wire with order = [ "version"; "pendingEnemy"; "checkpoint" ] };
  }

let finish5 (value : V5.t) (state : V6.state) : V6.t =
  {
    version = `V6;
    checkpoint = { seed = value.checkpoint.seed; state; wire = value.checkpoint.wire };
    pending_enemy = value.pending_enemy;
    setup = value.setup;
    progress = value.progress;
    undo = value.undo;
    wire = value.wire;
  }

let finish6 (value : V6.t) (state : V7.state) : V7.t =
  {
    version = `V7;
    checkpoint = { seed = value.checkpoint.seed; state; wire = value.checkpoint.wire };
    pending_enemy = value.pending_enemy;
    setup = value.setup;
    progress = value.progress;
    undo = value.undo;
    wire = value.wire;
  }

let finish7 (value : V7.t) (state : V8.state) : V8.t =
  {
    version = `V8;
    checkpoint = { seed = value.checkpoint.seed; state; wire = value.checkpoint.wire };
    pending_enemy = value.pending_enemy;
    setup = value.setup;
    progress = value.progress;
    undo = value.undo;
    wire = value.wire;
  }

let finish8 (value : V8.t) (state : V9.state) : V9.t =
  {
    version = `V9;
    checkpoint = { seed = value.checkpoint.seed; state; wire = value.checkpoint.wire };
    pending_enemy = value.pending_enemy;
    setup = value.setup;
    progress = value.progress;
    undo = value.undo;
    wire = value.wire;
  }

let finish9 (value : V9.t) (state : V10.state) : V10.t =
  {
    version = `V10;
    checkpoint = { seed = value.checkpoint.seed; state; wire = value.checkpoint.wire };
    pending_enemy = value.pending_enemy;
    setup = value.setup;
    progress = value.progress;
    undo = value.undo;
    wire = value.wire;
  }

let finish10 (value : V10.t) (state : V11.state) : V11.t =
  {
    version = `V11;
    checkpoint = { seed = value.checkpoint.seed; state; wire = value.checkpoint.wire };
    pending_enemy = value.pending_enemy;
    setup = value.setup;
    progress = value.progress;
    undo = value.undo;
    wire = value.wire;
  }

let finish11 (value : V11.t) (state : V12.state) : V12.t =
  {
    version = `V12;
    checkpoint = { seed = value.checkpoint.seed; state; wire = value.checkpoint.wire };
    pending_enemy = value.pending_enemy;
    setup = value.setup;
    progress = value.progress;
    undo = value.undo;
    wire = value.wire;
  }

let finish12 (value : V12.t) (state : V13.state) : V13.t =
  {
    version = `V13;
    checkpoint = { seed = value.checkpoint.seed; state; wire = value.checkpoint.wire };
    pending_enemy = value.pending_enemy;
    setup = value.setup;
    progress = value.progress;
    undo = value.undo;
    wire = value.wire;
  }

let finish13 (value : V13.t) (state : V14.state) : V14.t =
  {
    version = `V14;
    checkpoint = { seed = value.checkpoint.seed; state; wire = value.checkpoint.wire };
    pending_enemy = value.pending_enemy;
    setup = value.setup;
    progress = value.progress;
    undo = value.undo;
    wire = value.wire;
  }

let finish14 (value : V14.t) (state : V15.state) : V15.t =
  {
    version = `V15;
    checkpoint = { seed = value.checkpoint.seed; state; wire = value.checkpoint.wire };
    pending_enemy = value.pending_enemy;
    setup = value.setup;
    progress = value.progress;
    undo = value.undo;
    wire = value.wire;
  }

let finish15 (value : V15.t) (state : V16.state) : V16.t =
  {
    version = `V16;
    checkpoint = { seed = value.checkpoint.seed; state; wire = value.checkpoint.wire };
    pending_enemy = value.pending_enemy;
    setup = value.setup;
    progress = value.progress;
    undo = value.undo;
    wire = value.wire;
  }

let finish16 (value : V16.t) (state : V17.state) : V17.t =
  {
    version = `V17;
    checkpoint = { seed = value.checkpoint.seed; state; wire = value.checkpoint.wire };
    pending_enemy = value.pending_enemy;
    setup = value.setup;
    progress = value.progress;
    undo = value.undo;
    wire = value.wire;
  }

let finish17 (value : V17.t) (state : V18.state) : V18.t =
  {
    version = `V18;
    checkpoint = { seed = value.checkpoint.seed; state; wire = value.checkpoint.wire };
    pending_enemy = value.pending_enemy;
    setup = value.setup;
    progress = value.progress;
    undo = value.undo;
    wire = value.wire;
  }

let recipe_tile ~id ~col ~row source : V3.tile =
  let base =
    J.obj
      [
        ("id", J.s id);
        ("col", J.n col);
        ("row", J.n row);
        ("terrain", J.s "plains");
        ("owner", Js.Json.null);
      ]
  in
  Option.iter (fun source -> Array.iter (fun (k, v) -> J.set base k v) (J.entries source)) source;
  (C.Fragment.tiles11 (J.a [ base ])).(0)

let v3 (value : V3.t) : V4.t =
  let old = value.checkpoint.state in
  let state =
    if old.map_id <> "developer" || old.map_revision <> 1. then old
    else (
      require
        ((old.cols = 6. && old.rows = 20.) || (old.cols = 13. && old.rows = 23.))
        "Unknown historical developer geometry";
      let old_tiles = Hashtbl.create (Array.length old.tiles) in
      Array.iter
        (fun (tile : V3.tile) ->
          require
            (floor tile.col = tile.col
            && floor tile.row = tile.row
            && tile.col >= 0. && tile.col < old.cols && tile.row >= 0. && tile.row < old.rows
            && tile.id = Printf.sprintf "%.0f,%.0f" tile.col tile.row
            && not (Hashtbl.mem old_tiles tile.id))
            "Invalid historical developer tile";
          Hashtbl.add old_tiles tile.id tile)
        old.tiles;
      require
        (float_of_int (Hashtbl.length old_tiles) = old.cols *. old.rows)
        "Incomplete historical developer map";
      let map = Option.get (frozen_map "developer") in
      let terrain = Hashtbl.create 32 in
      List.iter
        (fun tile ->
          Hashtbl.replace terrain
            (Printf.sprintf "%.0f,%.0f" (J.num tile "col") (J.num tile "row"))
            tile)
        (J.list map "terrain");
      let cols = J.num map "cols" and rows = J.num map "rows" in
      let width = int_of_float cols in
      let tiles =
        Array.init
          (width * int_of_float rows)
          (fun index ->
            let col = float_of_int (index mod width) and row = float_of_int (index / width) in
            let id = Printf.sprintf "%.0f,%.0f" col row in
            Option.value (Hashtbl.find_opt old_tiles id)
              ~default:(recipe_tile ~id ~col ~row (Hashtbl.find_opt terrain id)))
      in
      { old with map_revision = map_revision map; cols; rows; tiles })
  in
  finish3 value state

let v4 (value : V4.t) : V5.t = finish4 value value.checkpoint.state

let v5 (value : V5.t) : V6.t =
  let old = Lantern_migrations.ferry value.checkpoint.state in
  let leader_types =
    Array.map J.string (J.array (Lazy.force Frozen_data.history_type_leaders_v5))
  in
  let units =
    Array.map
      (fun (unit : V5.unit_) ->
        if unit.role = None && Array.mem unit.type_ leader_types then
          { unit with role = Some `Leader; wire = inserted unit.wire [ "role" ] }
        else unit)
      old.units
  in
  finish5 value { old with units }

let v6_with_preplaced_sella preplaced_sella (value : V6.t) : V7.t =
  let old = Lantern_migrations.villages ~preplaced_sella value.checkpoint.state in
  let conversion =
    List.find_opt
      (fun map -> J.text map "mapId" = old.map_id && J.num map "mapRevision" = old.map_revision)
      (J.list (Lazy.force Frozen_data.history_factions_v6) "maps")
  in
  let factions = ref (Option.value old.factions ~default:[||]) in
  Option.iter
    (fun data ->
      Array.iter
        (fun (faction : V7.faction) ->
          if not (Array.exists (fun (existing : V7.faction) -> existing.id = faction.id) !factions)
          then factions := Array.append !factions [| faction |])
        (C.Fragment.factions7 (J.field data "factions")))
    conversion;
  let units =
    Array.map
      (fun (unit : V6.unit_) ->
        match conversion with
        | Some data
          when unit.side = `Enemy && unit.faction = None
               && (Option.fold ~none:false
                     ~some:(fun group ->
                       List.mem group (List.map J.string (J.list data "wildlifeGroups")))
                     unit.ai_group
                  || List.mem unit.type_ (List.map J.string (J.list data "wildlifeTypes"))) ->
            { unit with faction = Some "wildlife"; wire = inserted unit.wire [ "faction" ] }
        | _ -> unit)
      old.units
  in
  let state : V7.state =
    {
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
      tiles = old.tiles;
      campaign = old.campaign;
      gold = old.gold;
      factions = !factions;
      ground_items = old.ground_items;
      combat_rng = old.combat_rng;
      wire = inserted old.wire [ "factions" ];
    }
  in
  finish6 value state

let v6 value = v6_with_preplaced_sella false value

let v7 (value : V7.t) : V8.t =
  let old = value.checkpoint.state in
  let state : V8.state =
    {
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
      campaign = old.campaign;
      gold = old.gold;
      factions = old.factions;
      ground_items = [||];
      combat_rng = old.combat_rng;
      wire = inserted old.wire [ "groundItems" ];
    }
  in
  finish7 value state

let v8 (value : V8.t) : V9.t =
  let old = value.checkpoint.state in
  let historical = Lazy.force Frozen_data.history_toll_road_v8 in
  let state =
    if
      old.map_id <> J.text historical "mapId" || old.map_revision <> J.num historical "fromRevision"
    then old
    else
      let campaign =
        match old.campaign with
        | Some c -> c
        | None -> invalid_arg "Invalid historical Toll Road checkpoint"
      in
      let marks = ref campaign.marks in
      let met = Array.mem "event:village-contact" !marks in
      let memory = Option.value old.condition_memory ~default:[||] in
      let add mark = marks := add_unique mark !marks in
      let sella = J.text historical "sellaName" in
      if
        met
        || Array.mem ("fallen:" ^ sella) !marks
        || Option.fold ~none:false ~some:(Array.mem sella) campaign.fallen_companions
      then add (J.text historical "sellaSpawn");
      if
        met
        && Array.exists
             (fun (tile : V8.tile) ->
               tile.id = J.text historical "camp" && tile.owner = `String "player")
             old.tiles
      then add "event:foothold-established";
      (if Array.mem (J.text historical "swampMemory") memory then add "event:swamp-sighting"
       else
         let removed = ref false in
         marks :=
           filter
             (fun mark ->
               if mark = "event:swamp-sighting" && not !removed then (
                 removed := true;
                 false)
               else true)
             !marks);
      let units =
        old.units
        |> filter (fun (unit : V8.unit_) -> met || unit.name <> Some sella)
        |> Array.map (fun (unit : V8.unit_) ->
            if unit.name = Some sella && unit.faction = Some (J.text historical "oldSellaFaction")
            then { unit with faction = Some "player" }
            else if
              unit.id = J.text historical "casterId"
              && unit.faction = Some (J.text historical "casterFaction")
            then
              {
                unit with
                name = Some (J.text historical "casterName");
                wire = inserted unit.wire [ "name" ];
              }
            else unit)
      in
      let factions =
        old.factions
        |> filter (fun (faction : V8.faction) -> faction.id <> J.text historical "oldSellaFaction")
        |> Array.map (fun (faction : V8.faction) ->
            {
              faction with
              relations =
                Option.map
                  (String_map.remove (J.text historical "oldSellaFaction"))
                  faction.relations;
            })
      in
      {
        old with
        map_revision = J.num historical "toRevision";
        units;
        factions;
        condition_memory = Some (filter (( <> ) (J.text historical "oldLightMemory")) memory);
        campaign = Some { campaign with marks = !marks };
        wire = inserted old.wire [ "conditionMemory" ];
      }
  in
  finish8 value state

let v9 (value : V9.t) : V10.t =
  let orientations =
    [
      ("signpost", [| `E; `W |]);
      ("road", [| `E; `W |]);
      ("bridge", [| `E; `W |]);
      ("ford", [| `E; `W |]);
      ("village", [||]);
      ("landing", [| `W |]);
      ("ferry", [| `W |]);
    ]
  in
  let old = value.checkpoint.state in
  let tiles =
    Array.map
      (fun (tile : V9.tile) ->
        match (tile.connections, List.assoc_opt tile.terrain orientations) with
        | None, Some connections ->
            {
              tile with
              connections = Some connections;
              wire = inserted tile.wire [ "connections" ];
            }
        | _ -> tile)
      old.tiles
  in
  finish9 value { old with tiles }

let v10 (value : V10.t) : V11.t =
  let old = value.checkpoint.state in
  let data = Lazy.force Frozen_data.history_bell_valley_v10 in
  let patches = J.optional_list (J.field data "patches") old.map_id in
  let tiles =
    Array.map
      (fun (tile : V10.tile) ->
        match
          List.find_opt
            (fun patch ->
              J.text (J.field patch "tile") "id" = tile.id
              && J.text patch "oldTerrain" = tile.terrain)
            patches
        with
        | None -> tile
        | Some patch ->
            let fresh = (C.Fragment.tiles11 (J.a [ J.field patch "tile" ])).(0) in
            if String.starts_with ~prefix:"castle-" fresh.terrain then
              { fresh with owner = tile.owner }
            else fresh)
      old.tiles
  in
  let state = { old with tiles } in
  let state =
    if
      old.map_id <> J.text data "mapId"
      || not (List.mem old.map_revision (List.map J.number (J.list data "fromRevisions")))
    then state
    else (
      require
        (old.cols = J.num data "fromCols"
        && old.rows = J.num data "fromRows"
        && float_of_int (Array.length old.tiles) = old.cols *. old.rows
        && List.length
             (List.sort_uniq String.compare
                (List.map (fun (t : V10.tile) -> t.id) (Array.to_list old.tiles)))
           = Array.length old.tiles)
        "Invalid historical bell battle";
      let campaign =
        match old.campaign with Some c -> c | None -> invalid_arg "Invalid historical bell battle"
      in
      let mapping = J.field data "mapping" in
      let destination id =
        match J.get mapping id with
        | None -> invalid_arg "Unknown historical bell tile"
        | Some x -> J.string x
      in
      Array.iter
        (fun (tile : V10.tile) ->
          require
            (tile.id = Printf.sprintf "%.0f,%.0f" tile.col tile.row)
            "Invalid historical bell coordinate";
          let _ = destination tile.id in
          ())
        old.tiles;
      let rename name = if name = Some "Meri" then Some "Oren" else name in
      let roster =
        Option.map (Array.map (fun (u : V10.veteran) -> { u with name = rename u.name }))
      in
      let campaign =
        {
          campaign with
          wire = inserted campaign.wire [ "holds" ];
          veterans = roster campaign.veterans;
          story_veterans = roster campaign.story_veterans;
          fallen_companions =
            Option.map
              (Array.map (fun n -> if n = "Meri" then "Oren" else n))
              campaign.fallen_companions;
          marks = Array.map (fun m -> if m = "fallen:Meri" then "fallen:Oren" else m) campaign.marks;
          holds =
            Some
              (String_map.singleton "5,5"
                 { held_turns = 0.; last_counted_round = old.round; wire = empty_wire });
        }
      in
      let ownership = Hashtbl.create (Array.length old.tiles) in
      Array.iter
        (fun (t : V10.tile) -> Hashtbl.replace ownership (destination t.id) t.owner)
        old.tiles;
      let spent = J.field data "spent" in
      let spent_tiles =
        filter_map
          (fun mark -> Option.map J.string (J.get spent mark))
          (Option.get old.campaign).marks
      in
      let fog = C.Fragment.rule11 (J.field data "fog") in
      let tiles =
        C.Fragment.tiles11 (J.field data "tiles")
        |> Array.map (fun (tile : V11.tile) ->
            {
              tile with
              fog = Some fog;
              wire = inserted tile.wire [ "fog" ];
              owner = Option.value (Hashtbl.find_opt ownership tile.id) ~default:tile.owner;
              terrain = (if Array.mem tile.id spent_tiles then "ritual-spent" else tile.terrain);
            })
      in
      {
        state with
        map_revision = J.num data "toRevision";
        cols = J.num data "cols";
        rows = J.num data "rows";
        factions = C.Fragment.factions11 (J.field data "factions");
        condition_memory = Some [||];
        wire = inserted state.wire [ "conditionMemory" ];
        campaign = Some campaign;
        tiles;
        units =
          Array.map
            (fun (u : V10.unit_) -> { u with name = rename u.name; tile = destination u.tile })
            old.units;
        ground_items =
          Array.map
            (fun (g : V10.ground_items) -> { g with tile = destination g.tile })
            old.ground_items;
      })
  in
  finish10 value state

let v11 (value : V11.t) : V12.t =
  let old = value.checkpoint.state in
  let state =
    if old.map_id <> "lantern-bell" || old.map_revision <> 3. then old
    else
      let campaign =
        match old.campaign with
        | Some c -> c
        | None -> invalid_arg "Invalid historical valley opening"
      in
      require
        (floor campaign.dialogue_index = campaign.dialogue_index
        && campaign.dialogue_index >= 0.
        && campaign.dialogue_index <= float_of_int (Array.length campaign.dialogue))
        "Invalid historical valley opening";
      let extras = [ ("u3", "crossbowman", "1,15", 24.); ("u4", "spearman", "3,15", 25.) ] in
      let units =
        filter
          (fun (u : V11.unit_) ->
            not
              (old.round = 1. && old.winner = `Null && u.side = `Player
              && (u.name = None || u.name = Some "")
              && (u.role = None || u.role = Some `Undefined)
              && (u.experience = None || u.experience = Some 0.)
              && (not u.moved) && (not u.attacked)
              && (u.items = None || u.items = Some [||])
              && List.mem (u.id, u.type_, u.tile, u.hp) extras))
          old.units
      in
      let discarded =
        [
          "The tracks divide here. Some went north; the rest kept along the low road.";
          "We follow the living. Keep a watch on the high ground.";
          "Three groups, taken to the old burial grounds. We heard the bell before the screaming \
           stopped. The north path climbs to the monastery; the low road leads toward the keep.";
          "Keep your lamps burning. We’ll find them.";
        ]
      in
      let keep (line : V11.dialogue) = not (List.mem line.text discarded) in
      let dialogue = ref (filter keep campaign.dialogue) in
      let dialogue_index =
        float_of_int
          (Array.length
             (filter keep (Array.sub campaign.dialogue 0 (int_of_float campaign.dialogue_index))))
      in
      let marks = ref campaign.marks in
      if
        old.winner = `Null && old.turn = `Player && (not value.pending_enemy)
        && Array.exists (fun (u : V11.unit_) -> u.side = `Player && u.name = Some "Oren") units
        && (not (Array.mem "event:monastery-start" !marks))
        && not (Array.mem "event:monastery-lead" !marks)
      then (
        dialogue :=
          Array.append !dialogue
            [|
              {
                speaker = Some "Oren";
                portrait = "adept";
                text =
                  "The monastery is on the hill to the north. Get me there, and I can use its \
                   wards to clear this fog.";
                character = None;
                sound = None;
                wire = empty_wire;
              };
            |];
        marks := Array.append !marks [| "event:monastery-lead" |]);
      {
        old with
        units;
        campaign = Some { campaign with dialogue = !dialogue; dialogue_index; marks = !marks };
      }
  in
  finish11 value state

let church_items (items : V12.item array) : V13.item array =
  items
  |> filter (fun (item : V12.item) -> item.id <> "church-helm")
  |> Array.map (fun (item : V12.item) ->
      if item.id = "church-mail" then
        {
          item with
          name = "Enchanted mail";
          description =
            "Silver wards are worked into the links. Incoming blade and piercing damage reduced by \
             1 per strike. Drops when its carrier dies.";
        }
      else item)

let church_unit (u : V12.unit_) = { u with items = Option.map church_items u.items }
let church_veteran (u : V12.veteran) = { u with items = Option.map church_items u.items }

let v12 (value : V12.t) : V13.t =
  let old = value.checkpoint.state in
  let state =
    if
      old.map_id <> "lantern-bell"
      && not
           (Option.fold ~none:false
              ~some:(fun (c : V12.campaign) -> c.id = "lantern-road")
              old.campaign)
    then old
    else
      let ground_items =
        old.ground_items
        |> filter (fun (g : V12.ground_items) -> g.item.id <> "church-helm")
        |> Array.map (fun (g : V12.ground_items) ->
            { g with item = (church_items [| g.item |]).(0) })
      in
      let campaign =
        Option.map
          (fun (c : V12.campaign) ->
            {
              c with
              veterans = Option.map (Array.map church_veteran) c.veterans;
              story_veterans = Option.map (Array.map church_veteran) c.story_veterans;
              evacuated =
                Option.map
                  (Array.map (fun (e : V12.evacuated) -> { e with unit_ = church_unit e.unit_ }))
                  c.evacuated;
              dialogue =
                Array.map
                  (fun (line : V12.dialogue) ->
                    if
                      line.text
                      = "The vestry has been stripped of food, but two pieces of armor remain \
                         beneath a bench."
                    then
                      {
                        line with
                        text =
                          "A shirt of mail beneath the bench. No rust, and silver runes worked \
                           into the links.";
                      }
                    else line)
                  c.dialogue;
            })
          old.campaign
      in
      { old with units = Array.map church_unit old.units; ground_items; campaign }
  in
  finish12 value state

let v13 (value : V13.t) : V14.t =
  let old = value.checkpoint.state in
  let state =
    if old.map_id <> "lantern-bell" || old.map_revision <> 3. then old
    else
      let units =
        Array.map
          (fun (u : V13.unit_) ->
            if u.ai_group = Some "garrison" then
              { u with ai_group = None; wire = remove_fields u.wire [ "aiGroup" ] }
            else u)
          old.units
      in
      let villages = Array.to_list (J.array (Lazy.force Frozen_data.history_bell_villages_v13)) in
      let tiles =
        Array.map
          (fun (tile : V13.tile) ->
            let occupied =
              Array.exists (fun (u : V13.unit_) -> u.side = `Player && u.tile = tile.id) units
            in
            if tile.id = "11,12" && tile.terrain = "castle-bell" && occupied then
              { tile with owner = `String "player" }
            else
              match List.find_opt (fun v -> J.text v "id" = tile.id) villages with
              | None -> tile
              | Some village when tile.terrain = "plains" ->
                  let fresh = (C.Fragment.tiles11 (J.a [ village ])).(0) in
                  {
                    tile with
                    col = fresh.col;
                    row = fresh.row;
                    connections = fresh.connections;
                    terrain = fresh.terrain;
                    floor = fresh.floor;
                    owner = (if occupied then `String "player" else fresh.owner);
                    wire = merge_wire tile.wire fresh.wire;
                  }
              | Some _ -> tile)
          old.tiles
      in
      { old with tiles; units }
  in
  finish13 value state

let v14 (value : V14.t) : V15.t =
  let old = value.checkpoint.state in
  let units =
    if old.map_id <> "lantern-villages" then old.units
    else
      Array.map
        (fun (u : V14.unit_) ->
          if u.name = Some "The Tollkeeper" && u.side = `Enemy && u.role = Some `Leader then
            { u with ai_group = Some "tollkeeper"; wire = inserted u.wire [ "aiGroup" ] }
          else u)
        old.units
  in
  finish14 value { old with units }

let v15 (value : V15.t) : V16.t =
  let old = value.checkpoint.state in
  let tiles =
    if old.map_id <> "lantern-bell" || old.map_revision <> 3. then old.tiles
    else
      Array.map
        (fun (tile : V15.tile) ->
          if
            List.mem tile.id [ "9,11"; "13,12"; "11,15" ]
            && tile.terrain = "village" && tile.floor = Some "green-plain"
          then { tile with floor = Some "mountain-rock" }
          else tile)
        old.tiles
  in
  finish15 value { old with tiles }

let directions = [| `E; `Se; `Sw; `W; `Nw; `Ne |]

let edge_name = function
  | `E -> "e"
  | `Se -> "se"
  | `Sw -> "sw"
  | `W -> "w"
  | `Nw -> "nw"
  | `Ne -> "ne"

let edge value =
  match value with
  | "e" -> `E
  | "se" -> `Se
  | "sw" -> `Sw
  | "w" -> `W
  | "nw" -> `Nw
  | "ne" -> `Ne
  | _ -> invalid_arg "Invalid historical castle geometry"

let castle terrain = List.mem terrain [ "castle-wall"; "castle-gate"; "castle-bell" ]

let v16 (value : V16.t) : V17.t =
  let old = value.checkpoint.state in
  let tiles =
    Array.map
      (fun (tile : V16.tile) ->
        let connections, walls, gates, wire =
          if not (castle tile.terrain) then
            let wire =
              match tile.wall_corners with
              | None when List.mem "wallCorners" tile.wire.undefined_fields ->
                  { tile.wire with extra = tile.wire.extra @ [ ("wallCorners", J.undefined) ] }
              | None -> tile.wire
              | Some values ->
                  {
                    tile.wire with
                    extra =
                      tile.wire.extra
                      @ [ ("wallCorners", J.a (List.map J.n (Array.to_list values))) ];
                  }
            in
            let gates =
              match List.assoc_opt "gates" tile.wire.extra with
              | None -> None
              | Some value when J.is_undefined value -> None
              | Some value -> Some (C.read_fragments_native_state38_log value)
            in
            let wire = { wire with extra = List.remove_assoc "gates" wire.extra } in
            (tile.connections, tile.walls, gates, wire)
          else
            let connections =
              match tile.connections with
              | Some x -> x
              | None -> invalid_arg "Invalid historical castle geometry"
            in
            require
              (List.length (List.sort_uniq compare (Array.to_list connections))
               = Array.length connections
              && Option.fold ~none:false
                   ~some:(fun corners -> Array.length corners = 6)
                   tile.wall_corners)
              "Invalid historical castle geometry";
            let boundary = filter (fun edge -> not (Array.mem edge connections)) directions in
            let gates =
              if tile.terrain <> "castle-gate" then [||]
              else
                filter
                  (fun e ->
                    let i = Option.get (Array.find_index (( = ) e) directions) in
                    Array.mem directions.((i + 5) mod 6) boundary
                    && Array.mem directions.((i + 1) mod 6) boundary)
                  boundary
            in
            require
              (tile.terrain <> "castle-gate" || Array.length gates = 1)
              "Invalid historical gate orientation";
            ( None,
              Some (filter (fun e -> not (Array.mem e gates)) boundary),
              Some (Array.map edge_name gates),
              inserted
                (remove_fields tile.wire [ "connections"; "wallCorners" ])
                [ "walls"; "gates" ] )
        in
        ({
           owner = tile.owner;
           id = tile.id;
           location = tile.location;
           col = tile.col;
           row = tile.row;
           terrain = tile.terrain;
           connections;
           dead_ends = tile.dead_ends;
           walls;
           floor = tile.floor;
           fog = tile.fog;
           light = tile.light;
           gates;
           wire;
         }
          : V17.tile))
      old.tiles
  in
  let state : V17.state =
    {
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
      tiles;
      campaign = old.campaign;
      gold = old.gold;
      factions = old.factions;
      ground_items = old.ground_items;
      combat_rng = old.combat_rng;
      wire = old.wire;
    }
  in
  finish16 value state

let v17 (value : V17.t) : V18.t =
  let old = value.checkpoint.state in
  let tiles =
    Array.map
      (fun (tile : V17.tile) ->
        let terrain, wire =
          if not (castle tile.terrain) then
            let wire =
              match tile.gates with
              | None when List.mem "gates" tile.wire.undefined_fields ->
                  { tile.wire with extra = tile.wire.extra @ [ ("gates", J.undefined) ] }
              | None -> tile.wire
              | Some values ->
                  {
                    tile.wire with
                    extra =
                      tile.wire.extra @ [ ("gates", J.a (List.map J.s (Array.to_list values))) ];
                  }
            in
            (tile.terrain, wire)
          else
            let walls =
              match tile.walls with
              | Some x -> x
              | None -> invalid_arg "Invalid historical castle geometry"
            in
            let gate_names =
              match tile.gates with
              | Some x -> x
              | None -> invalid_arg "Invalid historical castle geometry"
            in
            let gates = Array.map edge gate_names in
            require
              (List.length (List.sort_uniq compare (Array.to_list walls)) = Array.length walls
              && List.length (List.sort_uniq compare (Array.to_list gates)) = Array.length gates
              && not (Array.exists (fun e -> Array.mem e gates) walls))
              "Invalid historical castle geometry";
            require
              (Array.length gates = if tile.terrain = "castle-gate" then 1 else 0)
              "Invalid historical gate count";
            ( (if tile.terrain = "castle-gate" then "castle-wall" else tile.terrain),
              remove_fields tile.wire [ "gates" ] )
        in
        ({
           owner = tile.owner;
           id = tile.id;
           location = tile.location;
           col = tile.col;
           row = tile.row;
           terrain;
           connections = tile.connections;
           dead_ends = tile.dead_ends;
           walls = tile.walls;
           floor = tile.floor;
           fog = tile.fog;
           light = tile.light;
           wire;
         }
          : V18.tile))
      old.tiles
  in
  let state : V18.state =
    {
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
      tiles;
      campaign = old.campaign;
      gold = old.gold;
      factions = old.factions;
      ground_items = old.ground_items;
      combat_rng = old.combat_rng;
      wire = old.wire;
    }
  in
  finish17 value state

let v1_javascript (value : Javascript1.t) : V2.t =
  let state = Legacy_migrations.import value in
  let map =
    match frozen_map state.map_id with
    | Some map -> map
    | None -> invalid_arg "Unknown historical map"
  in
  (* The published Ferry preview needs its explicit v5 layout conversion.
     Upstream overwrites its revision-one tag before expanding that board. *)
  let map_revision =
    if state.map_id = "lantern-ferry" && state.map_revision = 1. then state.map_revision
    else map_revision map
  in
  {
    version = `V2;
    checkpoint =
      {
        seed = 17.;
        state = { state with map_revision };
        wire = { empty_wire with order = [ "seed"; "state" ] };
      };
    pending_enemy = value.pending_enemy;
    planner = None;
    setup = None;
    progress = None;
    undo = None;
    wire = { empty_wire with order = [ "version"; "checkpoint"; "pendingEnemy" ] };
  }
