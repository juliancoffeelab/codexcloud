(* Frozen v38-native -> v39 structural conversion. Only declared coordinate
   fields change; strings in prose, identities, items and undo values stay opaque. *)
open Historical_types

let digits value =
  let first = if String.length value > 0 && value.[0] = '-' then 1 else 0 in
  String.length value > first
  &&
  let rec loop i =
    i = String.length value || (value.[i] >= '0' && value.[i] <= '9' && loop (i + 1))
  in
  loop first

let coordinate value =
  match String.split_on_char ',' value with [ col; row ] -> digits col && digits row | _ -> false

let parse_hex value =
  match String.split_on_char ',' value with
  | [ col; row ] when digits col && digits row ->
      let col = float_of_string col and row = float_of_string row in
      if not (Migration_json.safe_integer col && Migration_json.safe_integer row) then
        invalid_arg "Invalid historical hex coordinates";
      (col, row)
  | _ -> invalid_arg "Invalid historical hex address"

let rec convert_fragments_native_state38_to_save39_member0_checkpoint_state
    (value : fragments_native_state38) : save39_member0_checkpoint_state =
  {
    wire = value.wire;
    factions =
      convert_fragments_native_state38_factions_to_save39_member0_checkpoint_state_factions
        value.factions;
    ground_items =
      convert_fragments_native_state38_ground_items_to_save39_member0_checkpoint_state_ground_items
        value.ground_items;
    armies = value.armies;
    condition_memory = value.condition_memory;
    map_id = value.map_id;
    map_revision = value.map_revision;
    cols = value.cols;
    rows = value.rows;
    round = value.round;
    turn = value.turn;
    gold = value.gold;
    winner = value.winner;
    next_id = value.next_id;
    combat_rng = value.combat_rng;
    log = value.log;
    tiles =
      convert_fragments_native_state38_tiles_to_save39_member0_checkpoint_state_tiles value.tiles;
    units =
      convert_fragments_native_state38_units_to_save39_member0_checkpoint_state_units value.units;
    campaign =
      Option.map
        (fun value ->
          convert_fragments_native_state38_campaign_member1_to_save39_member0_checkpoint_state_campaign_member1
            value)
        value.campaign;
    last_combat = value.last_combat;
  }

and convert_fragments_native_state38_factions_to_save39_member0_checkpoint_state_factions
    (value : fragments_native_state38_factions) : save39_member0_checkpoint_state_factions =
  Array.map
    (fun value ->
      convert_fragments_native_state38_factions_item_to_save39_member0_checkpoint_state_factions_item
        value)
    value

and convert_fragments_native_state38_ground_items_to_save39_member0_checkpoint_state_ground_items
    (value : fragments_native_state38_ground_items) : save39_member0_checkpoint_state_ground_items =
  Array.map
    (fun value ->
      convert_fragments_native_state38_ground_items_item_to_save39_member0_checkpoint_state_ground_items_item
        value)
    value

and convert_fragments_native_state38_tiles_to_save39_member0_checkpoint_state_tiles
    (value : fragments_native_state38_tiles) : save39_member0_checkpoint_state_tiles =
  Array.map
    (fun value ->
      convert_fragments_native_state38_tiles_item_to_save39_member0_checkpoint_state_tiles_item
        value)
    value

and convert_fragments_native_state38_units_to_save39_member0_checkpoint_state_units
    (value : fragments_native_state38_units) : save39_member0_checkpoint_state_units =
  Array.map
    (fun value ->
      convert_fragments_native_state38_units_item_to_save39_member0_checkpoint_state_units_item
        value)
    value

and convert_fragments_native_state38_campaign_member1_to_save39_member0_checkpoint_state_campaign_member1
    (value : fragments_native_state38_campaign_member1) :
    save39_member0_checkpoint_state_campaign_member1 =
  {
    wire = value.wire;
    scripts =
      Option.map
        (fun value ->
          convert_fragments_native_state38_campaign_member1_scripts_member1_to_save39_member0_checkpoint_state_campaign_member1_scripts_member1
            value)
        value.scripts;
    evacuated =
      Option.map
        (fun value ->
          convert_fragments_native_state38_campaign_member1_evacuated_member1_to_save39_member0_checkpoint_state_campaign_member1_evacuated_member1
            value)
        value.evacuated;
    id = value.id;
    mission = value.mission;
    recall = value.recall;
    veterans = value.veterans;
    story_veterans = value.story_veterans;
    fallen_companions = value.fallen_companions;
    carry_gold = value.carry_gold;
    marks = value.marks;
    dialogue = value.dialogue;
    dialogue_index = value.dialogue_index;
    outcome = value.outcome;
    completed = value.completed;
    refuge = value.refuge;
    vigil = value.vigil;
    holds = value.holds;
    flags = value.flags;
  }

and convert_fragments_native_state38_factions_item_to_save39_member0_checkpoint_state_factions_item
    (value : fragments_native_state38_factions_item) : save39_member0_checkpoint_state_factions_item
    =
  {
    wire = value.wire;
    id = value.id;
    phase = value.phase;
    controller = value.controller;
    palette = value.palette;
    name = value.name;
    recruits = value.recruits;
    relations = value.relations;
    transfer =
      Option.map
        (fun value ->
          convert_fragments_native_state38_factions_item_transfer_member1_to_save39_member0_checkpoint_state_factions_item_transfer_member1
            value)
        value.transfer;
  }

and convert_fragments_native_state38_ground_items_item_to_save39_member0_checkpoint_state_ground_items_item
    (value : fragments_native_state38_ground_items_item) :
    save39_member0_checkpoint_state_ground_items_item =
  {
    wire = value.wire;
    tile = convert_json_string_to_save39_member0_checkpoint_state_ground_items_item_tile value.tile;
    item = value.item;
  }

and convert_fragments_native_state38_tiles_item_to_save39_member0_checkpoint_state_tiles_item
    (value : fragments_native_state38_tiles_item) : save39_member0_checkpoint_state_tiles_item =
  {
    wire = value.wire;
    id = value.id;
    location = value.location;
    col = value.col;
    row = value.row;
    terrain = value.terrain;
    owner = value.owner;
    connections = value.connections;
    dead_ends = value.dead_ends;
    walls = value.walls;
    floor = value.floor;
    fog =
      Option.map
        (fun value ->
          convert_fragments_native_state38_tiles_item_fog_member1_to_save39_member0_checkpoint_state_tiles_item_fog_member1
            value)
        value.fog;
    light =
      Option.map
        (fun value ->
          convert_fragments_native_state38_tiles_item_light_member1_to_save39_member0_checkpoint_state_tiles_item_light_member1
            value)
        value.light;
  }

and convert_fragments_native_state38_units_item_to_save39_member0_checkpoint_state_units_item
    (value : fragments_native_state38_units_item) : save39_member0_checkpoint_state_units_item =
  {
    wire = value.wire;
    faction = value.faction;
    items = value.items;
    ai_group = value.ai_group;
    id = value.id;
    type_ = value.type_;
    side = value.side;
    tile = convert_json_string_to_save39_member0_checkpoint_state_ground_items_item_tile value.tile;
    hp = value.hp;
    moved = value.moved;
    attacked = value.attacked;
    experience = value.experience;
    character = value.character;
    role = value.role;
    retreating = value.retreating;
    rebuilding = value.rebuilding;
  }

and convert_fragments_native_state38_campaign_member1_scripts_member1_to_save39_member0_checkpoint_state_campaign_member1_scripts_member1
    (value : fragments_native_state38_campaign_member1_scripts_member1) :
    save39_member0_checkpoint_state_campaign_member1_scripts_member1 =
  {
    wire = value.wire;
    failed = value.failed;
    completed = value.completed;
    revealed = value.revealed;
    recruitment = value.recruitment;
    active = value.active;
    holds = value.holds;
    marks =
      convert_fragments_native_state38_campaign_member1_scripts_member1_marks_to_save39_member0_checkpoint_state_campaign_member1_scripts_member1_marks
        value.marks;
    pending =
      convert_fragments_native_state38_campaign_member1_scripts_member1_pending_to_save39_member0_checkpoint_state_campaign_member1_scripts_member1_pending
        value.pending;
    next_mark = value.next_mark;
  }

and convert_fragments_native_state38_campaign_member1_evacuated_member1_to_save39_member0_checkpoint_state_campaign_member1_evacuated_member1
    (value : fragments_native_state38_campaign_member1_evacuated_member1) :
    save39_member0_checkpoint_state_campaign_member1_evacuated_member1 =
  Array.map
    (fun value ->
      convert_fragments_native_state38_campaign_member1_evacuated_member1_item_to_save39_member0_checkpoint_state_campaign_member1_evacuated_member1_item
        value)
    value

and convert_fragments_native_state38_factions_item_transfer_member1_to_save39_member0_checkpoint_state_factions_item_transfer_member1
    (value : fragments_native_state38_factions_item_transfer_member1) :
    save39_member0_checkpoint_state_factions_item_transfer_member1 =
  {
    wire = value.wire;
    to_ = value.to_;
    when_ =
      convert_fragments_native_state38_tiles_item_fog_member1_to_save39_member0_checkpoint_state_tiles_item_fog_member1
        value.when_;
  }

and convert_json_string_to_save39_member0_checkpoint_state_ground_items_item_tile
    (value : json_string) : save39_member0_checkpoint_state_ground_items_item_tile =
  let col, row = parse_hex value in
  { wire = { empty_wire with order = [ "col"; "row" ] }; col; row }

and convert_fragments_native_state38_tiles_item_fog_member1_to_save39_member0_checkpoint_state_tiles_item_fog_member1
    (value : fragments_native_state38_tiles_item_fog_member1) :
    save39_member0_checkpoint_state_tiles_item_fog_member1 =
  {
    wire = value.wire;
    condition =
      convert_fragments_native_state38_tiles_item_fog_member1_condition_to_save39_member0_checkpoint_state_tiles_item_fog_member1_condition
        value.condition;
    remember = value.remember;
  }

and convert_fragments_native_state38_tiles_item_light_member1_to_save39_member0_checkpoint_state_tiles_item_light_member1
    (value : fragments_native_state38_tiles_item_light_member1) :
    save39_member0_checkpoint_state_tiles_item_light_member1 =
  {
    wire = value.wire;
    color = value.color;
    fixture = value.fixture;
    dx = value.dx;
    dy = value.dy;
    radius = value.radius;
    power = value.power;
    when_ =
      Option.map
        (fun value ->
          convert_fragments_native_state38_tiles_item_fog_member1_to_save39_member0_checkpoint_state_tiles_item_fog_member1
            value)
        value.when_;
  }

and convert_fragments_native_state38_campaign_member1_scripts_member1_marks_to_save39_member0_checkpoint_state_campaign_member1_scripts_member1_marks
    (value : fragments_native_state38_campaign_member1_scripts_member1_marks) :
    save39_member0_checkpoint_state_campaign_member1_scripts_member1_marks =
  Array.map
    (fun value ->
      convert_fragments_native_state38_campaign_member1_scripts_member1_marks_item_to_save39_member0_checkpoint_state_campaign_member1_scripts_member1_marks_item
        value)
    value

and convert_fragments_native_state38_campaign_member1_scripts_member1_pending_to_save39_member0_checkpoint_state_campaign_member1_scripts_member1_pending
    (value : fragments_native_state38_campaign_member1_scripts_member1_pending) :
    save39_member0_checkpoint_state_campaign_member1_scripts_member1_pending =
  Array.map
    (fun value ->
      convert_fragments_native_state38_campaign_member1_scripts_member1_pending_item_to_save39_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item
        value)
    value

and convert_fragments_native_state38_campaign_member1_evacuated_member1_item_to_save39_member0_checkpoint_state_campaign_member1_evacuated_member1_item
    (value : fragments_native_state38_campaign_member1_evacuated_member1_item) :
    save39_member0_checkpoint_state_campaign_member1_evacuated_member1_item =
  {
    wire = value.wire;
    event = value.event;
    unit_ =
      convert_fragments_native_state38_units_item_to_save39_member0_checkpoint_state_units_item
        value.unit_;
  }

and convert_fragments_native_state38_tiles_item_fog_member1_condition_to_save39_member0_checkpoint_state_tiles_item_fog_member1_condition
    (value : fragments_native_state38_tiles_item_fog_member1_condition) :
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition =
  match value with
  | `Owner value ->
      `Owner
        (convert_fragments_rule11_condition_member0_to_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0
           value)
  | `Proximity value ->
      `Proximity
        (convert_fragments_light28_member1_when__member1_condition_member1_to_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member1
           value)
  | `Visible_units value -> `Visible_units value
  | `Has_units value -> `Has_units value
  | `No_units value -> `No_units value
  | `Choice value -> `Choice value
  | `Flag value -> `Flag value
  | `Event value -> `Event value
  | `Round value -> `Round value
  | `All value ->
      `All
        (convert_fragments_native_state38_tiles_item_fog_member1_condition_member9_to_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member9
           value)
  | `Any value ->
      `Any
        (convert_fragments_native_state38_tiles_item_fog_member1_condition_member10_to_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member10
           value)

and convert_fragments_native_state38_campaign_member1_scripts_member1_marks_item_to_save39_member0_checkpoint_state_campaign_member1_scripts_member1_marks_item
    (value : fragments_native_state38_campaign_member1_scripts_member1_marks_item) :
    save39_member0_checkpoint_state_campaign_member1_scripts_member1_marks_item =
  {
    wire = value.wire;
    light_remembered = value.light_remembered;
    id = value.id;
    type_ = value.type_;
    tile = convert_json_string_to_save39_member0_checkpoint_state_ground_items_item_tile value.tile;
    light =
      Option.map
        (fun value ->
          convert_fragments_native_state38_tiles_item_light_member1_to_save39_member0_checkpoint_state_tiles_item_light_member1
            value)
        value.light;
  }

and convert_fragments_native_state38_campaign_member1_scripts_member1_pending_item_to_save39_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item
    (value : fragments_native_state38_campaign_member1_scripts_member1_pending_item) :
    save39_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item =
  {
    wire = value.wire;
    area =
      convert_fragments_native_state38_log_to_save39_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item_area
        value.area;
    faction = value.faction;
    ai_group = value.ai_group;
    units = value.units;
  }

and convert_fragments_rule11_condition_member0_to_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0
    (value : fragments_rule11_condition_member0) :
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0 =
  {
    wire = value.wire;
    kind = value.kind;
    tile =
      Option.map
        (fun value ->
          convert_json_string_to_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile
            value)
        value.tile;
    side = value.side;
  }

and convert_fragments_light28_member1_when__member1_condition_member1_to_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member1
    (value : fragments_light28_member1_when__member1_condition_member1) :
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member1 =
  {
    wire = value.wire;
    kind = value.kind;
    unit_types = value.unit_types;
    tile =
      Option.map
        (fun value ->
          convert_json_string_to_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile
            value)
        value.tile;
    radius = value.radius;
    side = value.side;
    unit_type = value.unit_type;
    character = value.character;
  }

and convert_fragments_native_state38_tiles_item_fog_member1_condition_member9_to_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member9
    (value : fragments_native_state38_tiles_item_fog_member1_condition_member9) :
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member9 =
  {
    wire = value.wire;
    kind = value.kind;
    conditions =
      convert_fragments_native_state38_tiles_item_fog_member1_condition_member9_conditions_to_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member9_conditions
        value.conditions;
  }

and convert_fragments_native_state38_tiles_item_fog_member1_condition_member10_to_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member10
    (value : fragments_native_state38_tiles_item_fog_member1_condition_member10) :
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member10 =
  {
    wire = value.wire;
    kind = value.kind;
    conditions =
      convert_fragments_native_state38_tiles_item_fog_member1_condition_member9_conditions_to_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member9_conditions
        value.conditions;
  }

and convert_fragments_native_state38_log_to_save39_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item_area
    (value : fragments_native_state38_log) :
    save39_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item_area =
  Array.map
    (fun value ->
      convert_json_string_to_save39_member0_checkpoint_state_ground_items_item_tile value)
    value

and convert_json_string_to_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile
    (value : json_string) :
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile =
  if value = "" then invalid_arg "Invalid historical tile reference";
  if coordinate value then
    `Object3
      {
        wire = { empty_wire with order = [ "hex" ] };
        hex = convert_json_string_to_save39_member0_checkpoint_state_ground_items_item_tile value;
      }
  else `Location { wire = { empty_wire with order = [ "location" ] }; location = value }

and convert_fragments_native_state38_tiles_item_fog_member1_condition_member9_conditions_to_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member9_conditions
    (value : fragments_native_state38_tiles_item_fog_member1_condition_member9_conditions) :
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member9_conditions =
  Array.map
    (fun value ->
      convert_fragments_native_state38_tiles_item_fog_member1_condition_to_save39_member0_checkpoint_state_tiles_item_fog_member1_condition
        value)
    value

let state value = convert_fragments_native_state38_to_save39_member0_checkpoint_state value
