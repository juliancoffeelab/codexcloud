(* Generated from frozen src/formats/readers.json. Do not edit. *)
[@@@warning "-27"]

open Historical_types
open Migration_json

(* Melange 7.0.1 may merge null and undefined constant arms; keep null as an opaque, once-parsed value. *)
let json_null : Js.Json.t = Js.Json.parseExn "null"

let map_of_json decode value =
  Array.fold_left
    (fun map (key, value) -> String_map.add key (decode value) map)
    String_map.empty (entries value)

let map_to_json encode value =
  obj (List.map (fun (key, value) -> (key, encode value)) (String_map.bindings value))

let wire_of_json known value =
  let fields = Array.to_list (entries value) in
  {
    order = List.map fst fields;
    extra = List.filter (fun (k, _) -> not (List.mem k known)) fields;
    undefined_fields =
      List.filter_map (fun (k, v) -> if is_undefined v then Some k else None) fields;
  }

let wire_to_json ~known wire fields =
  List.iter
    (fun (key, _) ->
      require
        ((not (List.mem key known)) || List.mem_assoc key fields)
        ("Unpromoted historical field " ^ key))
    wire.extra;
  let fields = fields @ List.filter (fun (k, _) -> not (List.mem_assoc k fields)) wire.extra in
  let ordered =
    List.filter_map
      (fun key -> Option.map (fun v -> (key, v)) (List.assoc_opt key fields))
      wire.order
  in
  obj (ordered @ List.filter (fun (k, _) -> not (List.mem k wire.order)) fields)

let rec decode_save2 value : save2 =
  {
    wire =
      wire_of_json
        [ "version"; "checkpoint"; "pendingEnemy"; "planner"; "setup"; "progress"; "undo" ]
        value;
    version = decode_literal_2 (field value "version");
    checkpoint = decode_save2_checkpoint (field value "checkpoint");
    pending_enemy = decode_json_boolean (field value "pendingEnemy");
    planner =
      (match get value "planner" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_unknown x));
    setup =
      (match get value "setup" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_setup_member1 x));
    progress =
      (match get value "progress" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_progress_member1 x));
    undo =
      (match get value "undo" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_unknown x));
  }

and decode_literal_2 value : literal_2 = `V2

and decode_save2_checkpoint value : save2_checkpoint =
  {
    wire = wire_of_json [ "seed"; "state" ] value;
    seed = decode_json_number (field value "seed");
    state = decode_save2_checkpoint_state (field value "state");
  }

and decode_json_number value : json_number = number value

and decode_save2_checkpoint_state value : save2_checkpoint_state =
  {
    wire =
      wire_of_json
        [
          "round";
          "turn";
          "winner";
          "armies";
          "conditionMemory";
          "mapId";
          "mapRevision";
          "cols";
          "rows";
          "nextId";
          "log";
          "lastCombat";
          "units";
          "tiles";
          "campaign";
          "gold";
          "factions";
          "groundItems";
          "combatRng";
        ]
        value;
    round = decode_json_number (field value "round");
    turn = decode_save35_setup_first (field value "turn");
    winner = decode_fragments_native_state38_winner (field value "winner");
    armies =
      (match get value "armies" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_armies_member1 x));
    condition_memory =
      (match get value "conditionMemory" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_log x));
    map_id = decode_json_string (field value "mapId");
    map_revision = decode_json_number (field value "mapRevision");
    cols = decode_json_number (field value "cols");
    rows = decode_json_number (field value "rows");
    next_id = decode_json_number (field value "nextId");
    log = decode_fragments_native_state38_log (field value "log");
    last_combat =
      (match get value "lastCombat" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_last_combat x));
    units = decode_save2_checkpoint_state_units (field value "units");
    tiles = decode_save2_checkpoint_state_tiles (field value "tiles");
    campaign =
      (match get value "campaign" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_checkpoint_state_campaign_member1 x));
    gold = decode_javascript1_state_gold (field value "gold");
    factions =
      (match get value "factions" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_checkpoint_state_factions_member1 x));
    ground_items =
      (match get value "groundItems" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_ground_items x));
    combat_rng =
      (match get value "combatRng" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_number x));
  }

and decode_save35_setup_first value : save35_setup_first =
  if Migration_reader.matches value 6 then `Player
  else if Migration_reader.matches value 7 then `Enemy
  else invalid "Invalid frozen union"

and decode_literal_player value : literal_player = `Player
and decode_literal_enemy value : literal_enemy = `Enemy

and decode_fragments_native_state38_winner value : fragments_native_state38_winner =
  if Migration_reader.matches value 9 then `Null
  else if Migration_reader.matches value 6 then `Player
  else if Migration_reader.matches value 7 then `Enemy
  else invalid "Invalid frozen union"

and decode_json_null value : json_null = `Null

and decode_fragments_native_state38_armies value : fragments_native_state38_armies =
  if Migration_reader.matches value 11 then `Undefined
  else if Migration_reader.matches value 12 then
    `Tuple (decode_fragments_native_state38_armies_member1 value)
  else invalid "Invalid frozen union"

and decode_json_undefined value : json_undefined = `Undefined

and decode_fragments_native_state38_armies_member1 value : fragments_native_state38_armies_member1 =
  let xs = array value in
  (decode_json_string xs.(0), decode_json_string xs.(1))

and decode_json_string value : json_string = string value

and decode_fragments_native_state38_condition_memory value :
    fragments_native_state38_condition_memory =
  if Migration_reader.matches value 11 then `Undefined
  else if Migration_reader.matches value 15 then `Array (decode_fragments_native_state38_log value)
  else invalid "Invalid frozen union"

and decode_fragments_native_state38_log value : fragments_native_state38_log =
  Array.map decode_json_string (array value)

and decode_fragments_native_state38_last_combat value : fragments_native_state38_last_combat =
  if Migration_reader.matches value 11 then `Undefined
  else if Migration_reader.matches value 9 then `Null
  else if Migration_reader.matches value 17 then
    `Object3 (decode_fragments_native_state38_last_combat_member2 value)
  else invalid "Invalid frozen union"

and decode_fragments_native_state38_last_combat_member2 value :
    fragments_native_state38_last_combat_member2 =
  {
    wire = wire_of_json [ "attackerId"; "attackId"; "sound" ] value;
    attacker_id = decode_json_string (field value "attackerId");
    attack_id = decode_json_string (field value "attackId");
    sound = decode_json_string (field value "sound");
  }

and decode_save2_checkpoint_state_units value : save2_checkpoint_state_units =
  Array.map decode_save2_checkpoint_state_units_item (array value)

and decode_save2_checkpoint_state_units_item value : save2_checkpoint_state_units_item =
  {
    wire =
      wire_of_json
        [
          "tile";
          "id";
          "side";
          "role";
          "faction";
          "items";
          "aiGroup";
          "type";
          "hp";
          "moved";
          "attacked";
          "retreating";
          "rebuilding";
          "name";
          "character";
          "experience";
        ]
        value;
    tile = decode_json_string (field value "tile");
    id = decode_json_string (field value "id");
    side = decode_save35_setup_first (field value "side");
    role =
      (match get value "role" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_units_item_role x));
    faction =
      (match get value "faction" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
    items =
      (match get value "items" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_units_item_items_member1 x));
    ai_group =
      (match get value "aiGroup" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
    type_ = decode_json_string (field value "type");
    hp = decode_json_number (field value "hp");
    moved = decode_json_boolean (field value "moved");
    attacked = decode_json_boolean (field value "attacked");
    retreating =
      (match get value "retreating" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_rule11_remember x));
    rebuilding =
      (match get value "rebuilding" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_rule11_remember x));
    name =
      (match get value "name" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
    character =
      (match get value "character" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
    experience =
      (match get value "experience" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_number x));
  }

and decode_fragments_native_state38_units_item_role value : fragments_native_state38_units_item_role
    =
  if Migration_reader.matches value 11 then `Undefined
  else if Migration_reader.matches value 21 then `Leader
  else if Migration_reader.matches value 22 then `Courier
  else if Migration_reader.matches value 23 then `Captive
  else invalid "Invalid frozen union"

and decode_literal_leader value : literal_leader = `Leader
and decode_literal_courier value : literal_courier = `Courier
and decode_literal_captive value : literal_captive = `Captive

and decode_javascript1_state_map_id value : javascript1_state_map_id =
  if Migration_reader.matches value 11 then `Undefined
  else if Migration_reader.matches value 13 then `String (decode_json_string value)
  else invalid "Invalid frozen union"

and decode_fragments_native_state38_units_item_items value :
    fragments_native_state38_units_item_items =
  if Migration_reader.matches value 11 then `Undefined
  else if Migration_reader.matches value 26 then
    `Array (decode_fragments_native_state38_units_item_items_member1 value)
  else invalid "Invalid frozen union"

and decode_fragments_native_state38_units_item_items_member1 value :
    fragments_native_state38_units_item_items_member1 =
  Array.map decode_fragments_native_state38_ground_items_item_item (array value)

and decode_fragments_native_state38_ground_items_item_item value :
    fragments_native_state38_ground_items_item_item =
  {
    wire = wire_of_json [ "armor"; "id"; "name"; "description"; "attacks"; "healing" ] value;
    armor =
      (match get value "armor" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_ground_items_item_item_armor_member1 x));
    id = decode_json_string (field value "id");
    name = decode_json_string (field value "name");
    description = decode_json_string (field value "description");
    attacks = decode_fragments_native_state38_ground_items_item_item_attacks (field value "attacks");
    healing =
      (match get value "healing" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_ground_items_item_item_healing_member1 x));
  }

and decode_fragments_native_state38_ground_items_item_item_armor value :
    fragments_native_state38_ground_items_item_item_armor =
  if Migration_reader.matches value 11 then `Undefined
  else if Migration_reader.matches value 29 then
    `Object2 (decode_fragments_native_state38_ground_items_item_item_armor_member1 value)
  else invalid "Invalid frozen union"

and decode_fragments_native_state38_ground_items_item_item_armor_member1 value :
    fragments_native_state38_ground_items_item_item_armor_member1 =
  {
    wire = wire_of_json [ "blade"; "pierce"; "impact" ] value;
    blade = decode_json_number (field value "blade");
    pierce = decode_json_number (field value "pierce");
    impact = decode_json_number (field value "impact");
  }

and decode_fragments_native_state38_ground_items_item_item_attacks value :
    fragments_native_state38_ground_items_item_item_attacks =
  Array.map decode_fragments_native_state38_ground_items_item_item_attacks_item (array value)

and decode_fragments_native_state38_ground_items_item_item_attacks_item value :
    fragments_native_state38_ground_items_item_item_attacks_item =
  {
    wire =
      wire_of_json
        [
          "id";
          "name";
          "damage";
          "spread";
          "count";
          "damageType";
          "category";
          "range";
          "sound";
          "drain";
        ]
        value;
    id = decode_json_string (field value "id");
    name = decode_json_string (field value "name");
    damage = decode_json_number (field value "damage");
    spread =
      (match get value "spread" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_number x));
    count =
      (match get value "count" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_number x));
    damage_type =
      decode_fragments_native_state38_ground_items_item_item_attacks_item_damage_type
        (field value "damageType");
    category =
      decode_fragments_native_state38_ground_items_item_item_attacks_item_category
        (field value "category");
    range = decode_json_number (field value "range");
    sound = decode_json_string (field value "sound");
    drain =
      (match get value "drain" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_number x));
  }

and decode_fragments_light23_dx value : fragments_light23_dx =
  if Migration_reader.matches value 11 then `Undefined
  else if Migration_reader.matches value 3 then `Number (decode_json_number value)
  else invalid "Invalid frozen union"

and decode_fragments_native_state38_ground_items_item_item_attacks_item_damage_type value :
    fragments_native_state38_ground_items_item_item_attacks_item_damage_type =
  if Migration_reader.matches value 34 then `Blade
  else if Migration_reader.matches value 35 then `Pierce
  else if Migration_reader.matches value 36 then `Impact
  else invalid "Invalid frozen union"

and decode_literal_blade value : literal_blade = `Blade
and decode_literal_pierce value : literal_pierce = `Pierce
and decode_literal_impact value : literal_impact = `Impact

and decode_fragments_native_state38_ground_items_item_item_attacks_item_category value :
    fragments_native_state38_ground_items_item_item_attacks_item_category =
  if Migration_reader.matches value 38 then `Melee
  else if Migration_reader.matches value 39 then `Ranged
  else invalid "Invalid frozen union"

and decode_literal_melee value : literal_melee = `Melee
and decode_literal_ranged value : literal_ranged = `Ranged

and decode_fragments_native_state38_ground_items_item_item_healing value :
    fragments_native_state38_ground_items_item_item_healing =
  if Migration_reader.matches value 11 then `Undefined
  else if Migration_reader.matches value 41 then
    `Object2 (decode_fragments_native_state38_ground_items_item_item_healing_member1 value)
  else invalid "Invalid frozen union"

and decode_fragments_native_state38_ground_items_item_item_healing_member1 value :
    fragments_native_state38_ground_items_item_item_healing_member1 =
  {
    wire = wire_of_json [ "amount"; "radius" ] value;
    amount = decode_json_number (field value "amount");
    radius = decode_json_number (field value "radius");
  }

and decode_json_boolean value : json_boolean = boolean value

and decode_fragments_rule11_remember value : fragments_rule11_remember =
  if Migration_reader.matches value 11 then `Undefined
  else if Migration_reader.matches value 44 then `False
  else if Migration_reader.matches value 45 then `True
  else invalid "Invalid frozen union"

and decode_literal_false value : literal_false = `False
and decode_literal_true value : literal_true = `True

and decode_save2_checkpoint_state_tiles value : save2_checkpoint_state_tiles =
  Array.map decode_save2_checkpoint_state_tiles_item (array value)

and decode_save2_checkpoint_state_tiles_item value : save2_checkpoint_state_tiles_item =
  {
    wire =
      wire_of_json
        [
          "owner";
          "id";
          "location";
          "col";
          "row";
          "terrain";
          "connections";
          "deadEnds";
          "walls";
          "floor";
          "fog";
          "light";
          "wallCorners";
        ]
        value;
    owner = decode_fragments_tiles11_item_owner (field value "owner");
    id = decode_json_string (field value "id");
    location =
      (match get value "location" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
    col = decode_json_number (field value "col");
    row = decode_json_number (field value "row");
    terrain = decode_json_string (field value "terrain");
    connections =
      (match get value "connections" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_tiles11_item_connections_member1 x));
    dead_ends =
      (match get value "deadEnds" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_tiles11_item_connections_member1 x));
    walls =
      (match get value "walls" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_tiles11_item_connections_member1 x));
    floor =
      (match get value "floor" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
    fog =
      (match get value "fog" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_checkpoint_state_tiles_item_fog_member1 x));
    light =
      (match get value "light" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_checkpoint_state_tiles_item_light_member1 x));
    wall_corners =
      (match get value "wallCorners" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_tiles11_item_wall_corners_member1 x));
  }

and decode_fragments_tiles11_item_owner value : fragments_tiles11_item_owner =
  if Migration_reader.matches value 9 then `Null
  else if Migration_reader.matches value 13 then `String (decode_json_string value)
  else invalid "Invalid frozen union"

and decode_fragments_tiles11_item_connections value : fragments_tiles11_item_connections =
  if Migration_reader.matches value 11 then `Undefined
  else if Migration_reader.matches value 50 then
    `Array (decode_fragments_tiles11_item_connections_member1 value)
  else invalid "Invalid frozen union"

and decode_fragments_tiles11_item_connections_member1 value :
    fragments_tiles11_item_connections_member1 =
  Array.map decode_fragments_tiles11_item_connections_member1_item (array value)

and decode_fragments_tiles11_item_connections_member1_item value :
    fragments_tiles11_item_connections_member1_item =
  if Migration_reader.matches value 52 then `E
  else if Migration_reader.matches value 53 then `Se
  else if Migration_reader.matches value 54 then `Sw
  else if Migration_reader.matches value 55 then `W
  else if Migration_reader.matches value 56 then `Nw
  else if Migration_reader.matches value 57 then `Ne
  else invalid "Invalid frozen union"

and decode_literal_e value : literal_e = `E
and decode_literal_se value : literal_se = `Se
and decode_literal_sw value : literal_sw = `Sw
and decode_literal_w value : literal_w = `W
and decode_literal_nw value : literal_nw = `Nw
and decode_literal_ne value : literal_ne = `Ne

and decode_save2_checkpoint_state_tiles_item_fog value : save2_checkpoint_state_tiles_item_fog =
  if Migration_reader.matches value 11 then `Undefined
  else if Migration_reader.matches value 59 then
    `Object2 (decode_save2_checkpoint_state_tiles_item_fog_member1 value)
  else invalid "Invalid frozen union"

and decode_save2_checkpoint_state_tiles_item_fog_member1 value :
    save2_checkpoint_state_tiles_item_fog_member1 =
  {
    wire = wire_of_json [ "condition"; "remember" ] value;
    condition =
      decode_save2_checkpoint_state_tiles_item_fog_member1_condition (field value "condition");
    remember =
      (match get value "remember" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_rule11_remember x));
  }

and decode_save2_checkpoint_state_tiles_item_fog_member1_condition value :
    save2_checkpoint_state_tiles_item_fog_member1_condition =
  if Migration_reader.matches value 61 then `Owner (decode_fragments_rule11_condition_member0 value)
  else if Migration_reader.matches value 63 then
    `Visible_units (decode_fragments_rule11_condition_member1 value)
  else if Migration_reader.matches value 68 then
    `Choice (decode_fragments_rule11_condition_member2 value)
  else if Migration_reader.matches value 70 then
    `Flag (decode_fragments_rule11_condition_member3 value)
  else if Migration_reader.matches value 72 then
    `Event (decode_fragments_rule11_condition_member4 value)
  else if Migration_reader.matches value 74 then
    `Round (decode_fragments_rule11_condition_member5 value)
  else if Migration_reader.matches value 76 then
    `Object7 (decode_save2_checkpoint_state_tiles_item_fog_member1_condition_member6 value)
  else if Migration_reader.matches value 80 then
    `Proximity (decode_fragments_rule11_condition_member6 value)
  else if Migration_reader.matches value 82 then
    `Has_units (decode_fragments_rule11_condition_member7 value)
  else if Migration_reader.matches value 84 then
    `No_units (decode_fragments_rule11_condition_member8 value)
  else invalid "Invalid frozen union"

and decode_fragments_rule11_condition_member0 value : fragments_rule11_condition_member0 =
  {
    wire = wire_of_json [ "kind"; "tile"; "side" ] value;
    kind = decode_literal_owner (field value "kind");
    tile =
      (match get value "tile" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
    side = decode_json_string (field value "side");
  }

and decode_literal_owner value : literal_owner = `Owner

and decode_fragments_rule11_condition_member1 value : fragments_rule11_condition_member1 =
  {
    wire = wire_of_json [ "kind"; "side"; "unitTypes" ] value;
    kind = decode_literal_visible_units (field value "kind");
    side = decode_fragments_rule11_condition_member1_side (field value "side");
    unit_types =
      (match get value "unitTypes" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_log x));
  }

and decode_literal_visible_units value : literal_visible_units = `Visible_units

and decode_fragments_rule11_condition_member1_side value : fragments_rule11_condition_member1_side =
  if Migration_reader.matches value 6 then `Player
  else if Migration_reader.matches value 7 then `Enemy
  else if Migration_reader.matches value 66 then `Neutral
  else if Migration_reader.matches value 67 then `Any
  else invalid "Invalid frozen union"

and decode_literal_neutral value : literal_neutral = `Neutral
and decode_literal_any value : literal_any = `Any

and decode_fragments_rule11_condition_member2 value : fragments_rule11_condition_member2 =
  {
    wire = wire_of_json [ "kind"; "choice" ] value;
    kind = decode_literal_choice (field value "kind");
    choice = decode_json_string (field value "choice");
  }

and decode_literal_choice value : literal_choice = `Choice

and decode_fragments_rule11_condition_member3 value : fragments_rule11_condition_member3 =
  {
    wire = wire_of_json [ "kind"; "flag" ] value;
    kind = decode_literal_flag (field value "kind");
    flag = decode_json_string (field value "flag");
  }

and decode_literal_flag value : literal_flag = `Flag

and decode_fragments_rule11_condition_member4 value : fragments_rule11_condition_member4 =
  {
    wire = wire_of_json [ "kind"; "event" ] value;
    kind = decode_literal_event (field value "kind");
    event = decode_json_string (field value "event");
  }

and decode_literal_event value : literal_event = `Event

and decode_fragments_rule11_condition_member5 value : fragments_rule11_condition_member5 =
  {
    wire = wire_of_json [ "kind"; "round" ] value;
    kind = decode_literal_round (field value "kind");
    round = decode_json_number (field value "round");
  }

and decode_literal_round value : literal_round = `Round

and decode_save2_checkpoint_state_tiles_item_fog_member1_condition_member6 value :
    save2_checkpoint_state_tiles_item_fog_member1_condition_member6 =
  {
    wire = wire_of_json [ "kind"; "conditions" ] value;
    kind = decode_fragments_rule11_condition_member9_kind (field value "kind");
    conditions =
      decode_save2_checkpoint_state_tiles_item_fog_member1_condition_member6_conditions
        (field value "conditions");
  }

and decode_fragments_rule11_condition_member9_kind value : fragments_rule11_condition_member9_kind =
  if Migration_reader.matches value 67 then `Any
  else if Migration_reader.matches value 78 then `All
  else invalid "Invalid frozen union"

and decode_literal_all value : literal_all = `All

and decode_save2_checkpoint_state_tiles_item_fog_member1_condition_member6_conditions value :
    save2_checkpoint_state_tiles_item_fog_member1_condition_member6_conditions =
  Array.map decode_save2_checkpoint_state_tiles_item_fog_member1_condition (array value)

and decode_fragments_rule11_condition_member6 value : fragments_rule11_condition_member6 =
  {
    wire =
      wire_of_json
        [ "kind"; "unitTypes"; "tile"; "radius"; "side"; "unitType"; "character"; "name" ]
        value;
    kind = decode_literal_proximity (field value "kind");
    unit_types =
      (match get value "unitTypes" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_log x));
    tile =
      (match get value "tile" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
    radius = decode_json_number (field value "radius");
    side = decode_fragments_rule11_condition_member1_side (field value "side");
    unit_type =
      (match get value "unitType" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
    character =
      (match get value "character" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
    name =
      (match get value "name" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
  }

and decode_literal_proximity value : literal_proximity = `Proximity

and decode_fragments_rule11_condition_member7 value : fragments_rule11_condition_member7 =
  {
    wire = wire_of_json [ "kind"; "aiGroup"; "side"; "unitType"; "character"; "name" ] value;
    kind = decode_literal_has_units (field value "kind");
    ai_group =
      (match get value "aiGroup" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
    side = decode_fragments_rule11_condition_member1_side (field value "side");
    unit_type =
      (match get value "unitType" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
    character =
      (match get value "character" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
    name =
      (match get value "name" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
  }

and decode_literal_has_units value : literal_has_units = `Has_units

and decode_fragments_rule11_condition_member8 value : fragments_rule11_condition_member8 =
  {
    wire = wire_of_json [ "kind"; "side"; "unitType"; "character"; "name" ] value;
    kind = decode_literal_no_units (field value "kind");
    side = decode_fragments_rule11_condition_member1_side (field value "side");
    unit_type =
      (match get value "unitType" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
    character =
      (match get value "character" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
    name =
      (match get value "name" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
  }

and decode_literal_no_units value : literal_no_units = `No_units

and decode_save2_checkpoint_state_tiles_item_light value : save2_checkpoint_state_tiles_item_light =
  if Migration_reader.matches value 11 then `Undefined
  else if Migration_reader.matches value 87 then
    `Object2 (decode_save2_checkpoint_state_tiles_item_light_member1 value)
  else invalid "Invalid frozen union"

and decode_save2_checkpoint_state_tiles_item_light_member1 value :
    save2_checkpoint_state_tiles_item_light_member1 =
  {
    wire = wire_of_json [ "radius"; "color"; "fixture"; "dx"; "dy"; "power"; "when" ] value;
    radius = decode_json_number (field value "radius");
    color =
      (match get value "color" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_light23_color_member1 x));
    fixture =
      (match get value "fixture" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_rule11_remember x));
    dx =
      (match get value "dx" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_number x));
    dy =
      (match get value "dy" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_number x));
    power = decode_json_number (field value "power");
    when_ =
      (match get value "when" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_checkpoint_state_tiles_item_fog_member1 x));
  }

and decode_fragments_light23_color value : fragments_light23_color =
  if Migration_reader.matches value 11 then `Undefined
  else if Migration_reader.matches value 89 then
    `Tuple (decode_fragments_light23_color_member1 value)
  else invalid "Invalid frozen union"

and decode_fragments_light23_color_member1 value : fragments_light23_color_member1 =
  let xs = array value in
  (decode_json_number xs.(0), decode_json_number xs.(1), decode_json_number xs.(2))

and decode_fragments_tiles11_item_wall_corners value : fragments_tiles11_item_wall_corners =
  if Migration_reader.matches value 11 then `Undefined
  else if Migration_reader.matches value 91 then
    `Array (decode_fragments_tiles11_item_wall_corners_member1 value)
  else invalid "Invalid frozen union"

and decode_fragments_tiles11_item_wall_corners_member1 value :
    fragments_tiles11_item_wall_corners_member1 =
  Array.map decode_json_number (array value)

and decode_save2_checkpoint_state_campaign value : save2_checkpoint_state_campaign =
  if Migration_reader.matches value 11 then `Undefined
  else if Migration_reader.matches value 93 then
    `Object2 (decode_save2_checkpoint_state_campaign_member1 value)
  else invalid "Invalid frozen union"

and decode_save2_checkpoint_state_campaign_member1 value : save2_checkpoint_state_campaign_member1 =
  {
    wire =
      wire_of_json
        [
          "mission";
          "id";
          "vigil";
          "refuge";
          "completed";
          "marks";
          "holds";
          "recall";
          "fallenCompanions";
          "carryGold";
          "dialogue";
          "dialogueIndex";
          "outcome";
          "veterans";
          "storyVeterans";
          "evacuated";
          "choices";
        ]
        value;
    mission = decode_json_number (field value "mission");
    id = decode_json_string (field value "id");
    vigil =
      (match get value "vigil" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_campaign_member1_vigil_member1 x));
    refuge =
      (match get value "refuge" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_campaign_member1_refuge_member1 x));
    completed = decode_json_boolean (field value "completed");
    marks = decode_fragments_native_state38_log (field value "marks");
    holds =
      (match get value "holds" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_campaign_member1_holds_member1 x));
    recall = decode_fragments_native_state38_log (field value "recall");
    fallen_companions =
      (match get value "fallenCompanions" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_log x));
    carry_gold = decode_json_number (field value "carryGold");
    dialogue = decode_fragments_native_state38_campaign_member1_dialogue (field value "dialogue");
    dialogue_index = decode_json_number (field value "dialogueIndex");
    outcome = decode_fragments_tiles11_item_owner (field value "outcome");
    veterans =
      (match get value "veterans" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_checkpoint_state_campaign_member1_veterans_member1 x));
    story_veterans =
      (match get value "storyVeterans" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_checkpoint_state_campaign_member1_veterans_member1 x));
    evacuated =
      (match get value "evacuated" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_checkpoint_state_campaign_member1_evacuated_member1 x));
    choices =
      (match get value "choices" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_log x));
  }

and decode_fragments_native_state38_campaign_member1_vigil value :
    fragments_native_state38_campaign_member1_vigil =
  if Migration_reader.matches value 11 then `Undefined
  else if Migration_reader.matches value 95 then
    `Object2 (decode_fragments_native_state38_campaign_member1_vigil_member1 value)
  else invalid "Invalid frozen union"

and decode_fragments_native_state38_campaign_member1_vigil_member1 value :
    fragments_native_state38_campaign_member1_vigil_member1 =
  {
    wire = wire_of_json [ "heldTurns"; "lastCountedRound" ] value;
    held_turns = decode_json_number (field value "heldTurns");
    last_counted_round = decode_json_number (field value "lastCountedRound");
  }

and decode_fragments_native_state38_campaign_member1_refuge value :
    fragments_native_state38_campaign_member1_refuge =
  if Migration_reader.matches value 11 then `Undefined
  else if Migration_reader.matches value 97 then
    `Object2 (decode_fragments_native_state38_campaign_member1_refuge_member1 value)
  else invalid "Invalid frozen union"

and decode_fragments_native_state38_campaign_member1_refuge_member1 value :
    fragments_native_state38_campaign_member1_refuge_member1 =
  {
    wire = wire_of_json [ "heldTurns"; "lastCountedRound"; "escalated" ] value;
    held_turns = decode_json_number (field value "heldTurns");
    last_counted_round = decode_json_number (field value "lastCountedRound");
    escalated = decode_json_boolean (field value "escalated");
  }

and decode_fragments_native_state38_campaign_member1_holds value :
    fragments_native_state38_campaign_member1_holds =
  if Migration_reader.matches value 11 then `Undefined
  else if Migration_reader.matches value 99 then
    `Object2 (decode_fragments_native_state38_campaign_member1_holds_member1 value)
  else invalid "Invalid frozen union"

and decode_fragments_native_state38_campaign_member1_holds_member1 value :
    fragments_native_state38_campaign_member1_holds_member1 =
  map_of_json decode_fragments_native_state38_campaign_member1_vigil_member1 value

and decode_fragments_native_state38_campaign_member1_dialogue value :
    fragments_native_state38_campaign_member1_dialogue =
  Array.map decode_fragments_native_state38_campaign_member1_dialogue_item (array value)

and decode_fragments_native_state38_campaign_member1_dialogue_item value :
    fragments_native_state38_campaign_member1_dialogue_item =
  {
    wire = wire_of_json [ "sound"; "speaker"; "character"; "portrait"; "text" ] value;
    sound =
      (match get value "sound" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
    speaker =
      (match get value "speaker" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
    character =
      (match get value "character" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
    portrait = decode_json_string (field value "portrait");
    text = decode_json_string (field value "text");
  }

and decode_save2_checkpoint_state_campaign_member1_veterans value :
    save2_checkpoint_state_campaign_member1_veterans =
  if Migration_reader.matches value 11 then `Undefined
  else if Migration_reader.matches value 103 then
    `Array (decode_save2_checkpoint_state_campaign_member1_veterans_member1 value)
  else invalid "Invalid frozen union"

and decode_save2_checkpoint_state_campaign_member1_veterans_member1 value :
    save2_checkpoint_state_campaign_member1_veterans_member1 =
  Array.map decode_save2_checkpoint_state_campaign_member1_veterans_member1_item (array value)

and decode_save2_checkpoint_state_campaign_member1_veterans_member1_item value :
    save2_checkpoint_state_campaign_member1_veterans_member1_item =
  {
    wire = wire_of_json [ "role"; "experience"; "items"; "type"; "name"; "character" ] value;
    role =
      (match get value "role" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_units_item_role x));
    experience = decode_json_number (field value "experience");
    items =
      (match get value "items" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_units_item_items_member1 x));
    type_ = decode_json_string (field value "type");
    name =
      (match get value "name" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
    character =
      (match get value "character" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
  }

and decode_save2_checkpoint_state_campaign_member1_evacuated value :
    save2_checkpoint_state_campaign_member1_evacuated =
  if Migration_reader.matches value 11 then `Undefined
  else if Migration_reader.matches value 106 then
    `Array (decode_save2_checkpoint_state_campaign_member1_evacuated_member1 value)
  else invalid "Invalid frozen union"

and decode_save2_checkpoint_state_campaign_member1_evacuated_member1 value :
    save2_checkpoint_state_campaign_member1_evacuated_member1 =
  Array.map decode_save2_checkpoint_state_campaign_member1_evacuated_member1_item (array value)

and decode_save2_checkpoint_state_campaign_member1_evacuated_member1_item value :
    save2_checkpoint_state_campaign_member1_evacuated_member1_item =
  {
    wire = wire_of_json [ "event"; "unit" ] value;
    event = decode_json_string (field value "event");
    unit_ = decode_save2_checkpoint_state_units_item (field value "unit");
  }

and decode_javascript1_state_gold value : javascript1_state_gold =
  {
    wire = wire_of_json [ "player"; "enemy" ] value;
    player = decode_json_number (field value "player");
    enemy = decode_json_number (field value "enemy");
  }

and decode_save2_checkpoint_state_factions value : save2_checkpoint_state_factions =
  if Migration_reader.matches value 11 then `Undefined
  else if Migration_reader.matches value 110 then
    `Array (decode_save2_checkpoint_state_factions_member1 value)
  else invalid "Invalid frozen union"

and decode_save2_checkpoint_state_factions_member1 value : save2_checkpoint_state_factions_member1 =
  Array.map decode_save2_checkpoint_state_factions_member1_item (array value)

and decode_save2_checkpoint_state_factions_member1_item value :
    save2_checkpoint_state_factions_member1_item =
  {
    wire =
      wire_of_json
        [ "id"; "name"; "phase"; "controller"; "palette"; "recruits"; "relations"; "transfer" ]
        value;
    id = decode_json_string (field value "id");
    name =
      (match get value "name" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
    phase = decode_save35_setup_first (field value "phase");
    controller = decode_fragments_factions7_item_controller (field value "controller");
    palette = decode_json_string (field value "palette");
    recruits =
      (match get value "recruits" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_factions7_item_recruits_member1 x));
    relations =
      (match get value "relations" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_factions7_item_relations_member1 x));
    transfer =
      (match get value "transfer" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_checkpoint_state_factions_member1_item_transfer_member1 x));
  }

and decode_fragments_factions7_item_controller value : fragments_factions7_item_controller =
  if Migration_reader.matches value 113 then
    `Human (decode_fragments_factions7_item_controller_member0 value)
  else if Migration_reader.matches value 115 then
    `Passive (decode_fragments_factions7_item_controller_member1 value)
  else if Migration_reader.matches value 117 then
    `Ai (decode_fragments_factions7_item_controller_member2 value)
  else invalid "Invalid frozen union"

and decode_fragments_factions7_item_controller_member0 value :
    fragments_factions7_item_controller_member0 =
  { wire = wire_of_json [ "kind" ] value; kind = decode_literal_human (field value "kind") }

and decode_literal_human value : literal_human = `Human

and decode_fragments_factions7_item_controller_member1 value :
    fragments_factions7_item_controller_member1 =
  { wire = wire_of_json [ "kind" ] value; kind = decode_literal_passive (field value "kind") }

and decode_literal_passive value : literal_passive = `Passive

and decode_fragments_factions7_item_controller_member2 value :
    fragments_factions7_item_controller_member2 =
  {
    wire = wire_of_json [ "kind"; "policy" ] value;
    kind = decode_literal_ai (field value "kind");
    policy =
      (match get value "policy" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_factions7_item_controller_member2_policy x));
  }

and decode_literal_ai value : literal_ai = `Ai

and decode_fragments_factions7_item_controller_member2_policy value :
    fragments_factions7_item_controller_member2_policy =
  if Migration_reader.matches value 11 then `Undefined
  else if Migration_reader.matches value 120 then `Classic
  else if Migration_reader.matches value 121 then `Rush
  else if Migration_reader.matches value 122 then `Balanced
  else if Migration_reader.matches value 123 then `Rotated
  else if Migration_reader.matches value 124 then `Situational
  else if Migration_reader.matches value 125 then `Predictor
  else invalid "Invalid frozen union"

and decode_literal_classic value : literal_classic = `Classic
and decode_literal_rush value : literal_rush = `Rush
and decode_literal_balanced value : literal_balanced = `Balanced
and decode_literal_rotated value : literal_rotated = `Rotated
and decode_literal_situational value : literal_situational = `Situational
and decode_literal_predictor value : literal_predictor = `Predictor

and decode_fragments_factions7_item_recruits value : fragments_factions7_item_recruits =
  if Migration_reader.matches value 11 then `Undefined
  else if Migration_reader.matches value 127 then
    `Array (decode_fragments_factions7_item_recruits_member1 value)
  else invalid "Invalid frozen union"

and decode_fragments_factions7_item_recruits_member1 value :
    fragments_factions7_item_recruits_member1 =
  Array.map decode_fragments_factions7_item_recruits_member1_item (array value)

and decode_fragments_factions7_item_recruits_member1_item value :
    fragments_factions7_item_recruits_member1_item =
  {
    wire = wire_of_json [ "unit"; "cost" ] value;
    unit_ = decode_json_string (field value "unit");
    cost =
      (match get value "cost" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_number x));
  }

and decode_fragments_factions7_item_relations value : fragments_factions7_item_relations =
  if Migration_reader.matches value 11 then `Undefined
  else if Migration_reader.matches value 130 then
    `Object2 (decode_fragments_factions7_item_relations_member1 value)
  else invalid "Invalid frozen union"

and decode_fragments_factions7_item_relations_member1 value :
    fragments_factions7_item_relations_member1 =
  map_of_json decode_fragments_factions7_item_relations_member1_value value

and decode_fragments_factions7_item_relations_member1_value value :
    fragments_factions7_item_relations_member1_value =
  if Migration_reader.matches value 66 then `Neutral
  else if Migration_reader.matches value 132 then `Allied
  else if Migration_reader.matches value 133 then `Hostile
  else invalid "Invalid frozen union"

and decode_literal_allied value : literal_allied = `Allied
and decode_literal_hostile value : literal_hostile = `Hostile

and decode_save2_checkpoint_state_factions_member1_item_transfer value :
    save2_checkpoint_state_factions_member1_item_transfer =
  if Migration_reader.matches value 11 then `Undefined
  else if Migration_reader.matches value 135 then
    `Object2 (decode_save2_checkpoint_state_factions_member1_item_transfer_member1 value)
  else invalid "Invalid frozen union"

and decode_save2_checkpoint_state_factions_member1_item_transfer_member1 value :
    save2_checkpoint_state_factions_member1_item_transfer_member1 =
  {
    wire = wire_of_json [ "to"; "when" ] value;
    to_ = decode_json_string (field value "to");
    when_ = decode_save2_checkpoint_state_tiles_item_fog_member1 (field value "when");
  }

and decode_save2_checkpoint_state_ground_items value : save2_checkpoint_state_ground_items =
  if Migration_reader.matches value 11 then `Undefined
  else if Migration_reader.matches value 137 then
    `Array (decode_fragments_native_state38_ground_items value)
  else invalid "Invalid frozen union"

and decode_fragments_native_state38_ground_items value : fragments_native_state38_ground_items =
  Array.map decode_fragments_native_state38_ground_items_item (array value)

and decode_fragments_native_state38_ground_items_item value :
    fragments_native_state38_ground_items_item =
  {
    wire = wire_of_json [ "tile"; "item" ] value;
    tile = decode_json_string (field value "tile");
    item = decode_fragments_native_state38_ground_items_item_item (field value "item");
  }

and decode_json_unknown value : json_unknown = value

and decode_save2_setup value : save2_setup =
  if Migration_reader.matches value 11 then `Undefined
  else if Migration_reader.matches value 141 then `Object2 (decode_save2_setup_member1 value)
  else invalid "Invalid frozen union"

and decode_save2_setup_member1 value : save2_setup_member1 =
  {
    wire = wire_of_json [ "first"; "enemy" ] value;
    first = decode_save35_setup_first (field value "first");
    enemy = decode_save2_setup_member1_enemy (field value "enemy");
  }

and decode_save2_setup_member1_enemy value : save2_setup_member1_enemy =
  {
    wire = wire_of_json [ "policy"; "blocks"; "recruitmentLimit"; "diplomacy" ] value;
    policy = decode_save35_setup_enemy_policy (field value "policy");
    blocks =
      (match get value "blocks" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save35_setup_enemy_blocks_member1 x));
    recruitment_limit =
      (match get value "recruitmentLimit" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_number x));
    diplomacy = decode_save2_setup_member1_enemy_diplomacy (field value "diplomacy");
  }

and decode_save35_setup_enemy_policy value : save35_setup_enemy_policy =
  if Migration_reader.matches value 120 then `Classic
  else if Migration_reader.matches value 121 then `Rush
  else if Migration_reader.matches value 122 then `Balanced
  else if Migration_reader.matches value 123 then `Rotated
  else if Migration_reader.matches value 124 then `Situational
  else if Migration_reader.matches value 125 then `Predictor
  else invalid "Invalid frozen union"

and decode_save35_setup_enemy_blocks value : save35_setup_enemy_blocks =
  if Migration_reader.matches value 11 then `Undefined
  else if Migration_reader.matches value 146 then
    `Object2 (decode_save35_setup_enemy_blocks_member1 value)
  else invalid "Invalid frozen union"

and decode_save35_setup_enemy_blocks_member1 value : save35_setup_enemy_blocks_member1 =
  {
    wire = wire_of_json [ "recruiter"; "campaign"; "planner" ] value;
    recruiter = decode_save35_setup_enemy_blocks_member1_recruiter (field value "recruiter");
    campaign = decode_save35_setup_enemy_blocks_member1_campaign (field value "campaign");
    planner = decode_save35_setup_enemy_blocks_member1_planner (field value "planner");
  }

and decode_save35_setup_enemy_blocks_member1_recruiter value :
    save35_setup_enemy_blocks_member1_recruiter =
  if Migration_reader.matches value 120 then `Classic
  else if Migration_reader.matches value 122 then `Balanced
  else if Migration_reader.matches value 124 then `Situational
  else if Migration_reader.matches value 125 then `Predictor
  else if Migration_reader.matches value 148 then `Rotation
  else if Migration_reader.matches value 149 then `Frontline
  else if Migration_reader.matches value 150 then `Aggressive_predictor
  else invalid "Invalid frozen union"

and decode_literal_rotation value : literal_rotation = `Rotation
and decode_literal_frontline value : literal_frontline = `Frontline
and decode_literal_aggressive_predictor value : literal_aggressive_predictor = `Aggressive_predictor

and decode_save35_setup_enemy_blocks_member1_campaign value :
    save35_setup_enemy_blocks_member1_campaign =
  if Migration_reader.matches value 152 then `Arena
  else if Migration_reader.matches value 153 then `Mission
  else invalid "Invalid frozen union"

and decode_literal_arena value : literal_arena = `Arena
and decode_literal_mission value : literal_mission = `Mission

and decode_save35_setup_enemy_blocks_member1_planner value :
    save35_setup_enemy_blocks_member1_planner =
  if Migration_reader.matches value 120 then `Classic
  else if Migration_reader.matches value 155 then `Full
  else if Migration_reader.matches value 156 then `Queued
  else invalid "Invalid frozen union"

and decode_literal_full value : literal_full = `Full
and decode_literal_queued value : literal_queued = `Queued

and decode_save2_setup_member1_enemy_diplomacy value : save2_setup_member1_enemy_diplomacy =
  if Migration_reader.matches value 158 then `Diplomatic
  else if Migration_reader.matches value 159 then `Unrestricted
  else if Migration_reader.matches value 160 then `Pragmatic
  else if Migration_reader.matches value 161 then `Restrained
  else invalid "Invalid frozen union"

and decode_literal_diplomatic value : literal_diplomatic = `Diplomatic
and decode_literal_unrestricted value : literal_unrestricted = `Unrestricted
and decode_literal_pragmatic value : literal_pragmatic = `Pragmatic
and decode_literal_restrained value : literal_restrained = `Restrained

and decode_save2_progress value : save2_progress =
  if Migration_reader.matches value 11 then `Undefined
  else if Migration_reader.matches value 163 then `Object2 (decode_save2_progress_member1 value)
  else invalid "Invalid frozen union"

and decode_save2_progress_member1 value : save2_progress_member1 =
  {
    wire = wire_of_json [ "entry" ] value;
    entry = decode_history1_entries_item (field value "entry");
  }

and decode_history1_entries_item value : history1_entries_item =
  {
    wire =
      wire_of_json
        [
          "id";
          "savedAt";
          "run";
          "mission";
          "round";
          "side";
          "missionStart";
          "runId";
          "branchId";
          "checkpointId";
          "ordinal";
          "kind";
          "parentId";
        ]
        value;
    id = decode_json_string (field value "id");
    saved_at = decode_json_number (field value "savedAt");
    run = decode_json_string (field value "run");
    mission = decode_json_string (field value "mission");
    round = decode_json_number (field value "round");
    side = decode_json_string (field value "side");
    mission_start = decode_json_boolean (field value "missionStart");
    run_id = decode_json_string (field value "runId");
    branch_id = decode_json_string (field value "branchId");
    checkpoint_id = decode_json_string (field value "checkpointId");
    ordinal = decode_json_number (field value "ordinal");
    kind = decode_history1_entries_item_kind (field value "kind");
    parent_id =
      (match get value "parentId" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
  }

and decode_history1_entries_item_kind value : history1_entries_item_kind =
  if Migration_reader.matches value 166 then `Turn
  else if Migration_reader.matches value 167 then `Recovery
  else if Migration_reader.matches value 168 then `Resume
  else invalid "Invalid frozen union"

and decode_literal_turn value : literal_turn = `Turn
and decode_literal_recovery value : literal_recovery = `Recovery
and decode_literal_resume value : literal_resume = `Resume

and decode_save3 value : save3 =
  {
    wire =
      wire_of_json
        [ "version"; "checkpoint"; "pendingEnemy"; "planner"; "setup"; "progress"; "undo" ]
        value;
    version = decode_literal_3 (field value "version");
    checkpoint = decode_save3_checkpoint (field value "checkpoint");
    pending_enemy = decode_json_boolean (field value "pendingEnemy");
    planner =
      (match get value "planner" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_unknown x));
    setup =
      (match get value "setup" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_setup_member1 x));
    progress =
      (match get value "progress" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_progress_member1 x));
    undo =
      (match get value "undo" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_unknown x));
  }

and decode_literal_3 value : literal_3 = `V3

and decode_save3_checkpoint value : save3_checkpoint =
  {
    wire = wire_of_json [ "seed"; "state" ] value;
    seed = decode_json_number (field value "seed");
    state = decode_save3_checkpoint_state (field value "state");
  }

and decode_save3_checkpoint_state value : save3_checkpoint_state =
  {
    wire =
      wire_of_json
        [
          "round";
          "turn";
          "winner";
          "armies";
          "conditionMemory";
          "mapId";
          "mapRevision";
          "cols";
          "rows";
          "nextId";
          "log";
          "lastCombat";
          "units";
          "tiles";
          "campaign";
          "gold";
          "factions";
          "groundItems";
          "combatRng";
        ]
        value;
    round = decode_json_number (field value "round");
    turn = decode_save35_setup_first (field value "turn");
    winner = decode_fragments_native_state38_winner (field value "winner");
    armies =
      (match get value "armies" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_armies_member1 x));
    condition_memory =
      (match get value "conditionMemory" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_log x));
    map_id = decode_json_string (field value "mapId");
    map_revision = decode_json_number (field value "mapRevision");
    cols = decode_json_number (field value "cols");
    rows = decode_json_number (field value "rows");
    next_id = decode_json_number (field value "nextId");
    log = decode_fragments_native_state38_log (field value "log");
    last_combat =
      (match get value "lastCombat" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_last_combat x));
    units = decode_save2_checkpoint_state_units (field value "units");
    tiles = decode_fragments_tiles11 (field value "tiles");
    campaign =
      (match get value "campaign" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_checkpoint_state_campaign_member1 x));
    gold = decode_javascript1_state_gold (field value "gold");
    factions =
      (match get value "factions" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_factions7 x));
    ground_items =
      (match get value "groundItems" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_ground_items x));
    combat_rng =
      (match get value "combatRng" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_number x));
  }

and decode_fragments_tiles11 value : fragments_tiles11 =
  Array.map decode_fragments_tiles11_item (array value)

and decode_fragments_tiles11_item value : fragments_tiles11_item =
  {
    wire =
      wire_of_json
        [
          "owner";
          "id";
          "location";
          "col";
          "row";
          "terrain";
          "connections";
          "deadEnds";
          "walls";
          "floor";
          "fog";
          "light";
          "wallCorners";
        ]
        value;
    owner = decode_fragments_tiles11_item_owner (field value "owner");
    id = decode_json_string (field value "id");
    location =
      (match get value "location" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
    col = decode_json_number (field value "col");
    row = decode_json_number (field value "row");
    terrain = decode_json_string (field value "terrain");
    connections =
      (match get value "connections" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_tiles11_item_connections_member1 x));
    dead_ends =
      (match get value "deadEnds" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_tiles11_item_connections_member1 x));
    walls =
      (match get value "walls" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_tiles11_item_connections_member1 x));
    floor =
      (match get value "floor" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
    fog =
      (match get value "fog" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_rule11 x));
    light =
      (match get value "light" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_light23 x));
    wall_corners =
      (match get value "wallCorners" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_tiles11_item_wall_corners_member1 x));
  }

and decode_fragments_light23_when_ value : fragments_light23_when_ =
  if Migration_reader.matches value 11 then `Undefined
  else if Migration_reader.matches value 178 then `Object2 (decode_fragments_rule11 value)
  else invalid "Invalid frozen union"

and decode_fragments_rule11 value : fragments_rule11 =
  {
    wire = wire_of_json [ "condition"; "remember" ] value;
    condition = decode_fragments_rule11_condition (field value "condition");
    remember =
      (match get value "remember" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_rule11_remember x));
  }

and decode_fragments_rule11_condition value : fragments_rule11_condition =
  if Migration_reader.matches value 61 then `Owner (decode_fragments_rule11_condition_member0 value)
  else if Migration_reader.matches value 63 then
    `Visible_units (decode_fragments_rule11_condition_member1 value)
  else if Migration_reader.matches value 68 then
    `Choice (decode_fragments_rule11_condition_member2 value)
  else if Migration_reader.matches value 70 then
    `Flag (decode_fragments_rule11_condition_member3 value)
  else if Migration_reader.matches value 72 then
    `Event (decode_fragments_rule11_condition_member4 value)
  else if Migration_reader.matches value 74 then
    `Round (decode_fragments_rule11_condition_member5 value)
  else if Migration_reader.matches value 80 then
    `Proximity (decode_fragments_rule11_condition_member6 value)
  else if Migration_reader.matches value 82 then
    `Has_units (decode_fragments_rule11_condition_member7 value)
  else if Migration_reader.matches value 84 then
    `No_units (decode_fragments_rule11_condition_member8 value)
  else if Migration_reader.matches value 180 then
    `Object10 (decode_fragments_rule11_condition_member9 value)
  else invalid "Invalid frozen union"

and decode_fragments_rule11_condition_member9 value : fragments_rule11_condition_member9 =
  {
    wire = wire_of_json [ "kind"; "conditions" ] value;
    kind = decode_fragments_rule11_condition_member9_kind (field value "kind");
    conditions = decode_fragments_rule11_condition_member9_conditions (field value "conditions");
  }

and decode_fragments_rule11_condition_member9_conditions value :
    fragments_rule11_condition_member9_conditions =
  Array.map decode_fragments_rule11_condition (array value)

and decode_fragments_tiles11_item_light value : fragments_tiles11_item_light =
  if Migration_reader.matches value 11 then `Undefined
  else if Migration_reader.matches value 183 then `Object2 (decode_fragments_light23 value)
  else invalid "Invalid frozen union"

and decode_fragments_light23 value : fragments_light23 =
  {
    wire = wire_of_json [ "radius"; "color"; "fixture"; "dx"; "dy"; "power"; "when" ] value;
    radius = decode_json_number (field value "radius");
    color =
      (match get value "color" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_light23_color_member1 x));
    fixture =
      (match get value "fixture" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_rule11_remember x));
    dx =
      (match get value "dx" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_number x));
    dy =
      (match get value "dy" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_number x));
    power = decode_json_number (field value "power");
    when_ =
      (match get value "when" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_rule11 x));
  }

and decode_save3_checkpoint_state_factions value : save3_checkpoint_state_factions =
  if Migration_reader.matches value 11 then `Undefined
  else if Migration_reader.matches value 193 then `Array (decode_fragments_factions7 value)
  else invalid "Invalid frozen union"

and decode_fragments_factions7 value : fragments_factions7 =
  Array.map decode_fragments_factions7_item (array value)

and decode_fragments_factions7_item value : fragments_factions7_item =
  {
    wire =
      wire_of_json
        [ "id"; "name"; "phase"; "controller"; "palette"; "recruits"; "relations"; "transfer" ]
        value;
    id = decode_json_string (field value "id");
    name =
      (match get value "name" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
    phase = decode_save35_setup_first (field value "phase");
    controller = decode_fragments_factions7_item_controller (field value "controller");
    palette = decode_json_string (field value "palette");
    recruits =
      (match get value "recruits" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_factions7_item_recruits_member1 x));
    relations =
      (match get value "relations" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_factions7_item_relations_member1 x));
    transfer =
      (match get value "transfer" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_factions7_item_transfer_member1 x));
  }

and decode_fragments_factions7_item_transfer value : fragments_factions7_item_transfer =
  if Migration_reader.matches value 11 then `Undefined
  else if Migration_reader.matches value 196 then
    `Object2 (decode_fragments_factions7_item_transfer_member1 value)
  else invalid "Invalid frozen union"

and decode_fragments_factions7_item_transfer_member1 value :
    fragments_factions7_item_transfer_member1 =
  {
    wire = wire_of_json [ "to"; "when" ] value;
    to_ = decode_json_string (field value "to");
    when_ = decode_fragments_rule11 (field value "when");
  }

and decode_save4 value : save4 =
  {
    wire =
      wire_of_json
        [ "version"; "checkpoint"; "pendingEnemy"; "planner"; "setup"; "progress"; "undo" ]
        value;
    version = decode_literal_4 (field value "version");
    checkpoint = decode_save3_checkpoint (field value "checkpoint");
    pending_enemy = decode_json_boolean (field value "pendingEnemy");
    planner =
      (match get value "planner" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_unknown x));
    setup =
      (match get value "setup" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_setup_member1 x));
    progress =
      (match get value "progress" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_progress_member1 x));
    undo =
      (match get value "undo" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_unknown x));
  }

and decode_literal_4 value : literal_4 = `V4

and decode_save5 value : save5 =
  {
    wire =
      wire_of_json [ "version"; "checkpoint"; "pendingEnemy"; "setup"; "progress"; "undo" ] value;
    version = decode_literal_5 (field value "version");
    checkpoint = decode_save3_checkpoint (field value "checkpoint");
    pending_enemy = decode_json_boolean (field value "pendingEnemy");
    setup =
      (match get value "setup" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_setup_member1 x));
    progress =
      (match get value "progress" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_progress_member1 x));
    undo =
      (match get value "undo" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_unknown x));
  }

and decode_literal_5 value : literal_5 = `V5

and decode_save6 value : save6 =
  {
    wire =
      wire_of_json [ "version"; "checkpoint"; "pendingEnemy"; "setup"; "progress"; "undo" ] value;
    version = decode_literal_6 (field value "version");
    checkpoint = decode_save3_checkpoint (field value "checkpoint");
    pending_enemy = decode_json_boolean (field value "pendingEnemy");
    setup =
      (match get value "setup" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_setup_member1 x));
    progress =
      (match get value "progress" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_progress_member1 x));
    undo =
      (match get value "undo" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_unknown x));
  }

and decode_literal_6 value : literal_6 = `V6

and decode_save7 value : save7 =
  {
    wire =
      wire_of_json [ "version"; "checkpoint"; "pendingEnemy"; "setup"; "progress"; "undo" ] value;
    version = decode_literal_7 (field value "version");
    checkpoint = decode_save7_checkpoint (field value "checkpoint");
    pending_enemy = decode_json_boolean (field value "pendingEnemy");
    setup =
      (match get value "setup" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_setup_member1 x));
    progress =
      (match get value "progress" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_progress_member1 x));
    undo =
      (match get value "undo" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_unknown x));
  }

and decode_literal_7 value : literal_7 = `V7

and decode_save7_checkpoint value : save7_checkpoint =
  {
    wire = wire_of_json [ "seed"; "state" ] value;
    seed = decode_json_number (field value "seed");
    state = decode_save7_checkpoint_state (field value "state");
  }

and decode_save7_checkpoint_state value : save7_checkpoint_state =
  {
    wire =
      wire_of_json
        [
          "round";
          "turn";
          "winner";
          "armies";
          "conditionMemory";
          "mapId";
          "mapRevision";
          "cols";
          "rows";
          "nextId";
          "log";
          "lastCombat";
          "units";
          "tiles";
          "campaign";
          "gold";
          "factions";
          "groundItems";
          "combatRng";
        ]
        value;
    round = decode_json_number (field value "round");
    turn = decode_save35_setup_first (field value "turn");
    winner = decode_fragments_native_state38_winner (field value "winner");
    armies =
      (match get value "armies" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_armies_member1 x));
    condition_memory =
      (match get value "conditionMemory" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_log x));
    map_id = decode_json_string (field value "mapId");
    map_revision = decode_json_number (field value "mapRevision");
    cols = decode_json_number (field value "cols");
    rows = decode_json_number (field value "rows");
    next_id = decode_json_number (field value "nextId");
    log = decode_fragments_native_state38_log (field value "log");
    last_combat =
      (match get value "lastCombat" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_last_combat x));
    units = decode_save2_checkpoint_state_units (field value "units");
    tiles = decode_fragments_tiles11 (field value "tiles");
    campaign =
      (match get value "campaign" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_checkpoint_state_campaign_member1 x));
    gold = decode_javascript1_state_gold (field value "gold");
    factions = decode_fragments_factions7 (field value "factions");
    ground_items =
      (match get value "groundItems" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_ground_items x));
    combat_rng =
      (match get value "combatRng" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_number x));
  }

and decode_save8 value : save8 =
  {
    wire =
      wire_of_json [ "version"; "checkpoint"; "pendingEnemy"; "setup"; "progress"; "undo" ] value;
    version = decode_literal_8 (field value "version");
    checkpoint = decode_save8_checkpoint (field value "checkpoint");
    pending_enemy = decode_json_boolean (field value "pendingEnemy");
    setup =
      (match get value "setup" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_setup_member1 x));
    progress =
      (match get value "progress" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_progress_member1 x));
    undo =
      (match get value "undo" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_unknown x));
  }

and decode_literal_8 value : literal_8 = `V8

and decode_save8_checkpoint value : save8_checkpoint =
  {
    wire = wire_of_json [ "seed"; "state" ] value;
    seed = decode_json_number (field value "seed");
    state = decode_save8_checkpoint_state (field value "state");
  }

and decode_save8_checkpoint_state value : save8_checkpoint_state =
  {
    wire =
      wire_of_json
        [
          "round";
          "turn";
          "winner";
          "armies";
          "conditionMemory";
          "mapId";
          "mapRevision";
          "cols";
          "rows";
          "nextId";
          "log";
          "lastCombat";
          "units";
          "tiles";
          "campaign";
          "gold";
          "factions";
          "groundItems";
          "combatRng";
        ]
        value;
    round = decode_json_number (field value "round");
    turn = decode_save35_setup_first (field value "turn");
    winner = decode_fragments_native_state38_winner (field value "winner");
    armies =
      (match get value "armies" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_armies_member1 x));
    condition_memory =
      (match get value "conditionMemory" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_log x));
    map_id = decode_json_string (field value "mapId");
    map_revision = decode_json_number (field value "mapRevision");
    cols = decode_json_number (field value "cols");
    rows = decode_json_number (field value "rows");
    next_id = decode_json_number (field value "nextId");
    log = decode_fragments_native_state38_log (field value "log");
    last_combat =
      (match get value "lastCombat" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_last_combat x));
    units = decode_save2_checkpoint_state_units (field value "units");
    tiles = decode_fragments_tiles11 (field value "tiles");
    campaign =
      (match get value "campaign" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_checkpoint_state_campaign_member1 x));
    gold = decode_javascript1_state_gold (field value "gold");
    factions = decode_fragments_factions7 (field value "factions");
    ground_items = decode_fragments_native_state38_ground_items (field value "groundItems");
    combat_rng =
      (match get value "combatRng" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_number x));
  }

and decode_save9 value : save9 =
  {
    wire =
      wire_of_json [ "version"; "checkpoint"; "pendingEnemy"; "setup"; "progress"; "undo" ] value;
    version = decode_literal_9 (field value "version");
    checkpoint = decode_save8_checkpoint (field value "checkpoint");
    pending_enemy = decode_json_boolean (field value "pendingEnemy");
    setup =
      (match get value "setup" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_setup_member1 x));
    progress =
      (match get value "progress" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_progress_member1 x));
    undo =
      (match get value "undo" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_unknown x));
  }

and decode_literal_9 value : literal_9 = `V9

and decode_save10 value : save10 =
  {
    wire =
      wire_of_json [ "version"; "checkpoint"; "pendingEnemy"; "setup"; "progress"; "undo" ] value;
    version = decode_literal_10 (field value "version");
    checkpoint = decode_save8_checkpoint (field value "checkpoint");
    pending_enemy = decode_json_boolean (field value "pendingEnemy");
    setup =
      (match get value "setup" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_setup_member1 x));
    progress =
      (match get value "progress" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_progress_member1 x));
    undo =
      (match get value "undo" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_unknown x));
  }

and decode_literal_10 value : literal_10 = `V10

and decode_save11 value : save11 =
  {
    wire =
      wire_of_json [ "version"; "checkpoint"; "pendingEnemy"; "setup"; "progress"; "undo" ] value;
    version = decode_literal_11 (field value "version");
    checkpoint = decode_save8_checkpoint (field value "checkpoint");
    pending_enemy = decode_json_boolean (field value "pendingEnemy");
    setup =
      (match get value "setup" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_setup_member1 x));
    progress =
      (match get value "progress" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_progress_member1 x));
    undo =
      (match get value "undo" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_unknown x));
  }

and decode_literal_11 value : literal_11 = `V11

and decode_save12 value : save12 =
  {
    wire =
      wire_of_json [ "version"; "checkpoint"; "pendingEnemy"; "setup"; "progress"; "undo" ] value;
    version = decode_literal_12 (field value "version");
    checkpoint = decode_save8_checkpoint (field value "checkpoint");
    pending_enemy = decode_json_boolean (field value "pendingEnemy");
    setup =
      (match get value "setup" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_setup_member1 x));
    progress =
      (match get value "progress" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_progress_member1 x));
    undo =
      (match get value "undo" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_unknown x));
  }

and decode_literal_12 value : literal_12 = `V12

and decode_save13 value : save13 =
  {
    wire =
      wire_of_json [ "version"; "checkpoint"; "pendingEnemy"; "setup"; "progress"; "undo" ] value;
    version = decode_literal_13 (field value "version");
    checkpoint = decode_save8_checkpoint (field value "checkpoint");
    pending_enemy = decode_json_boolean (field value "pendingEnemy");
    setup =
      (match get value "setup" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_setup_member1 x));
    progress =
      (match get value "progress" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_progress_member1 x));
    undo =
      (match get value "undo" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_unknown x));
  }

and decode_literal_13 value : literal_13 = `V13

and decode_save14 value : save14 =
  {
    wire =
      wire_of_json [ "version"; "checkpoint"; "pendingEnemy"; "setup"; "progress"; "undo" ] value;
    version = decode_literal_14 (field value "version");
    checkpoint = decode_save8_checkpoint (field value "checkpoint");
    pending_enemy = decode_json_boolean (field value "pendingEnemy");
    setup =
      (match get value "setup" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_setup_member1 x));
    progress =
      (match get value "progress" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_progress_member1 x));
    undo =
      (match get value "undo" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_unknown x));
  }

and decode_literal_14 value : literal_14 = `V14

and decode_save15 value : save15 =
  {
    wire =
      wire_of_json [ "version"; "checkpoint"; "pendingEnemy"; "setup"; "progress"; "undo" ] value;
    version = decode_literal_15 (field value "version");
    checkpoint = decode_save8_checkpoint (field value "checkpoint");
    pending_enemy = decode_json_boolean (field value "pendingEnemy");
    setup =
      (match get value "setup" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_setup_member1 x));
    progress =
      (match get value "progress" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_progress_member1 x));
    undo =
      (match get value "undo" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_unknown x));
  }

and decode_literal_15 value : literal_15 = `V15

and decode_save16 value : save16 =
  {
    wire =
      wire_of_json [ "version"; "checkpoint"; "pendingEnemy"; "setup"; "progress"; "undo" ] value;
    version = decode_literal_16 (field value "version");
    checkpoint = decode_save8_checkpoint (field value "checkpoint");
    pending_enemy = decode_json_boolean (field value "pendingEnemy");
    setup =
      (match get value "setup" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_setup_member1 x));
    progress =
      (match get value "progress" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_progress_member1 x));
    undo =
      (match get value "undo" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_unknown x));
  }

and decode_literal_16 value : literal_16 = `V16

and decode_save17 value : save17 =
  {
    wire =
      wire_of_json [ "version"; "checkpoint"; "pendingEnemy"; "setup"; "progress"; "undo" ] value;
    version = decode_literal_17 (field value "version");
    checkpoint = decode_save17_checkpoint (field value "checkpoint");
    pending_enemy = decode_json_boolean (field value "pendingEnemy");
    setup =
      (match get value "setup" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_setup_member1 x));
    progress =
      (match get value "progress" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_progress_member1 x));
    undo =
      (match get value "undo" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_unknown x));
  }

and decode_literal_17 value : literal_17 = `V17

and decode_save17_checkpoint value : save17_checkpoint =
  {
    wire = wire_of_json [ "seed"; "state" ] value;
    seed = decode_json_number (field value "seed");
    state = decode_save17_checkpoint_state (field value "state");
  }

and decode_save17_checkpoint_state value : save17_checkpoint_state =
  {
    wire =
      wire_of_json
        [
          "round";
          "turn";
          "winner";
          "armies";
          "conditionMemory";
          "mapId";
          "mapRevision";
          "cols";
          "rows";
          "nextId";
          "log";
          "lastCombat";
          "units";
          "tiles";
          "campaign";
          "gold";
          "factions";
          "groundItems";
          "combatRng";
        ]
        value;
    round = decode_json_number (field value "round");
    turn = decode_save35_setup_first (field value "turn");
    winner = decode_fragments_native_state38_winner (field value "winner");
    armies =
      (match get value "armies" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_armies_member1 x));
    condition_memory =
      (match get value "conditionMemory" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_log x));
    map_id = decode_json_string (field value "mapId");
    map_revision = decode_json_number (field value "mapRevision");
    cols = decode_json_number (field value "cols");
    rows = decode_json_number (field value "rows");
    next_id = decode_json_number (field value "nextId");
    log = decode_fragments_native_state38_log (field value "log");
    last_combat =
      (match get value "lastCombat" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_last_combat x));
    units = decode_save2_checkpoint_state_units (field value "units");
    tiles = decode_save17_checkpoint_state_tiles (field value "tiles");
    campaign =
      (match get value "campaign" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_checkpoint_state_campaign_member1 x));
    gold = decode_javascript1_state_gold (field value "gold");
    factions = decode_fragments_factions7 (field value "factions");
    ground_items = decode_fragments_native_state38_ground_items (field value "groundItems");
    combat_rng =
      (match get value "combatRng" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_number x));
  }

and decode_save17_checkpoint_state_tiles value : save17_checkpoint_state_tiles =
  Array.map decode_save17_checkpoint_state_tiles_item (array value)

and decode_save17_checkpoint_state_tiles_item value : save17_checkpoint_state_tiles_item =
  {
    wire =
      wire_of_json
        [
          "owner";
          "id";
          "location";
          "col";
          "row";
          "terrain";
          "connections";
          "deadEnds";
          "walls";
          "floor";
          "fog";
          "light";
          "gates";
        ]
        value;
    owner = decode_fragments_tiles11_item_owner (field value "owner");
    id = decode_json_string (field value "id");
    location =
      (match get value "location" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
    col = decode_json_number (field value "col");
    row = decode_json_number (field value "row");
    terrain = decode_json_string (field value "terrain");
    connections =
      (match get value "connections" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_tiles11_item_connections_member1 x));
    dead_ends =
      (match get value "deadEnds" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_tiles11_item_connections_member1 x));
    walls =
      (match get value "walls" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_tiles11_item_connections_member1 x));
    floor =
      (match get value "floor" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
    fog =
      (match get value "fog" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_rule11 x));
    light =
      (match get value "light" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_light23 x));
    gates =
      (match get value "gates" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_log x));
  }

and decode_save18 value : save18 =
  {
    wire =
      wire_of_json [ "version"; "checkpoint"; "pendingEnemy"; "setup"; "progress"; "undo" ] value;
    version = decode_literal_18 (field value "version");
    checkpoint = decode_save18_checkpoint (field value "checkpoint");
    pending_enemy = decode_json_boolean (field value "pendingEnemy");
    setup =
      (match get value "setup" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_setup_member1 x));
    progress =
      (match get value "progress" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_progress_member1 x));
    undo =
      (match get value "undo" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_unknown x));
  }

and decode_literal_18 value : literal_18 = `V18

and decode_save18_checkpoint value : save18_checkpoint =
  {
    wire = wire_of_json [ "seed"; "state" ] value;
    seed = decode_json_number (field value "seed");
    state = decode_save18_checkpoint_state (field value "state");
  }

and decode_save18_checkpoint_state value : save18_checkpoint_state =
  {
    wire =
      wire_of_json
        [
          "round";
          "turn";
          "winner";
          "armies";
          "conditionMemory";
          "mapId";
          "mapRevision";
          "cols";
          "rows";
          "nextId";
          "log";
          "lastCombat";
          "units";
          "tiles";
          "campaign";
          "gold";
          "factions";
          "groundItems";
          "combatRng";
        ]
        value;
    round = decode_json_number (field value "round");
    turn = decode_save35_setup_first (field value "turn");
    winner = decode_fragments_native_state38_winner (field value "winner");
    armies =
      (match get value "armies" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_armies_member1 x));
    condition_memory =
      (match get value "conditionMemory" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_log x));
    map_id = decode_json_string (field value "mapId");
    map_revision = decode_json_number (field value "mapRevision");
    cols = decode_json_number (field value "cols");
    rows = decode_json_number (field value "rows");
    next_id = decode_json_number (field value "nextId");
    log = decode_fragments_native_state38_log (field value "log");
    last_combat =
      (match get value "lastCombat" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_last_combat x));
    units = decode_save2_checkpoint_state_units (field value "units");
    tiles = decode_save18_checkpoint_state_tiles (field value "tiles");
    campaign =
      (match get value "campaign" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_checkpoint_state_campaign_member1 x));
    gold = decode_javascript1_state_gold (field value "gold");
    factions = decode_fragments_factions7 (field value "factions");
    ground_items = decode_fragments_native_state38_ground_items (field value "groundItems");
    combat_rng =
      (match get value "combatRng" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_number x));
  }

and decode_save18_checkpoint_state_tiles value : save18_checkpoint_state_tiles =
  Array.map decode_save18_checkpoint_state_tiles_item (array value)

and decode_save18_checkpoint_state_tiles_item value : save18_checkpoint_state_tiles_item =
  {
    wire =
      wire_of_json
        [
          "owner";
          "id";
          "location";
          "col";
          "row";
          "terrain";
          "connections";
          "deadEnds";
          "walls";
          "floor";
          "fog";
          "light";
        ]
        value;
    owner = decode_fragments_tiles11_item_owner (field value "owner");
    id = decode_json_string (field value "id");
    location =
      (match get value "location" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
    col = decode_json_number (field value "col");
    row = decode_json_number (field value "row");
    terrain = decode_json_string (field value "terrain");
    connections =
      (match get value "connections" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_tiles11_item_connections_member1 x));
    dead_ends =
      (match get value "deadEnds" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_tiles11_item_connections_member1 x));
    walls =
      (match get value "walls" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_tiles11_item_connections_member1 x));
    floor =
      (match get value "floor" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
    fog =
      (match get value "fog" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_rule11 x));
    light =
      (match get value "light" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_light23 x));
  }

and decode_save19 value : save19 =
  {
    wire =
      wire_of_json [ "version"; "checkpoint"; "pendingEnemy"; "setup"; "progress"; "undo" ] value;
    version = decode_literal_19 (field value "version");
    checkpoint = decode_save18_checkpoint (field value "checkpoint");
    pending_enemy = decode_json_boolean (field value "pendingEnemy");
    setup =
      (match get value "setup" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_setup_member1 x));
    progress =
      (match get value "progress" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_progress_member1 x));
    undo =
      (match get value "undo" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_unknown x));
  }

and decode_literal_19 value : literal_19 = `V19

and decode_save20 value : save20 =
  {
    wire =
      wire_of_json [ "version"; "checkpoint"; "pendingEnemy"; "setup"; "progress"; "undo" ] value;
    version = decode_literal_20 (field value "version");
    checkpoint = decode_save18_checkpoint (field value "checkpoint");
    pending_enemy = decode_json_boolean (field value "pendingEnemy");
    setup =
      (match get value "setup" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_setup_member1 x));
    progress =
      (match get value "progress" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_progress_member1 x));
    undo =
      (match get value "undo" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_unknown x));
  }

and decode_literal_20 value : literal_20 = `V20

and decode_save21 value : save21 =
  {
    wire =
      wire_of_json [ "version"; "checkpoint"; "pendingEnemy"; "setup"; "progress"; "undo" ] value;
    version = decode_literal_21 (field value "version");
    checkpoint = decode_save21_checkpoint (field value "checkpoint");
    pending_enemy = decode_json_boolean (field value "pendingEnemy");
    setup =
      (match get value "setup" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_setup_member1 x));
    progress =
      (match get value "progress" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_progress_member1 x));
    undo =
      (match get value "undo" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_unknown x));
  }

and decode_literal_21 value : literal_21 = `V21

and decode_save21_checkpoint value : save21_checkpoint =
  {
    wire = wire_of_json [ "seed"; "state" ] value;
    seed = decode_json_number (field value "seed");
    state = decode_save21_checkpoint_state (field value "state");
  }

and decode_save21_checkpoint_state value : save21_checkpoint_state =
  {
    wire =
      wire_of_json
        [
          "round";
          "turn";
          "winner";
          "armies";
          "conditionMemory";
          "mapId";
          "mapRevision";
          "cols";
          "rows";
          "nextId";
          "log";
          "lastCombat";
          "units";
          "tiles";
          "campaign";
          "gold";
          "factions";
          "groundItems";
          "combatRng";
        ]
        value;
    round = decode_json_number (field value "round");
    turn = decode_save35_setup_first (field value "turn");
    winner = decode_fragments_native_state38_winner (field value "winner");
    armies =
      (match get value "armies" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_armies_member1 x));
    condition_memory =
      (match get value "conditionMemory" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_log x));
    map_id = decode_json_string (field value "mapId");
    map_revision = decode_json_number (field value "mapRevision");
    cols = decode_json_number (field value "cols");
    rows = decode_json_number (field value "rows");
    next_id = decode_json_number (field value "nextId");
    log = decode_fragments_native_state38_log (field value "log");
    last_combat =
      (match get value "lastCombat" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_last_combat x));
    units = decode_save2_checkpoint_state_units (field value "units");
    tiles = decode_save18_checkpoint_state_tiles (field value "tiles");
    campaign =
      (match get value "campaign" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_checkpoint_state_campaign_member1 x));
    gold = decode_fragments_native_state38_gold (field value "gold");
    factions = decode_fragments_factions7 (field value "factions");
    ground_items = decode_fragments_native_state38_ground_items (field value "groundItems");
    combat_rng =
      (match get value "combatRng" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_number x));
  }

and decode_fragments_native_state38_gold value : fragments_native_state38_gold =
  map_of_json decode_json_number value

and decode_save22 value : save22 =
  {
    wire =
      wire_of_json [ "version"; "checkpoint"; "pendingEnemy"; "setup"; "progress"; "undo" ] value;
    version = decode_literal_22 (field value "version");
    checkpoint = decode_save21_checkpoint (field value "checkpoint");
    pending_enemy = decode_json_boolean (field value "pendingEnemy");
    setup =
      (match get value "setup" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_setup_member1 x));
    progress =
      (match get value "progress" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_progress_member1 x));
    undo =
      (match get value "undo" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_unknown x));
  }

and decode_literal_22 value : literal_22 = `V22

and decode_save23 value : save23 =
  {
    wire =
      wire_of_json [ "version"; "checkpoint"; "pendingEnemy"; "setup"; "progress"; "undo" ] value;
    version = decode_literal_23 (field value "version");
    checkpoint = decode_save21_checkpoint (field value "checkpoint");
    pending_enemy = decode_json_boolean (field value "pendingEnemy");
    setup =
      (match get value "setup" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_setup_member1 x));
    progress =
      (match get value "progress" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_progress_member1 x));
    undo =
      (match get value "undo" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_unknown x));
  }

and decode_literal_23 value : literal_23 = `V23

and decode_save24 value : save24 =
  {
    wire =
      wire_of_json [ "version"; "checkpoint"; "pendingEnemy"; "setup"; "progress"; "undo" ] value;
    version = decode_literal_24 (field value "version");
    checkpoint = decode_save21_checkpoint (field value "checkpoint");
    pending_enemy = decode_json_boolean (field value "pendingEnemy");
    setup =
      (match get value "setup" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_setup_member1 x));
    progress =
      (match get value "progress" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_progress_member1 x));
    undo =
      (match get value "undo" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_unknown x));
  }

and decode_literal_24 value : literal_24 = `V24

and decode_save25 value : save25 =
  {
    wire =
      wire_of_json [ "version"; "checkpoint"; "pendingEnemy"; "setup"; "progress"; "undo" ] value;
    version = decode_literal_25 (field value "version");
    checkpoint = decode_save21_checkpoint (field value "checkpoint");
    pending_enemy = decode_json_boolean (field value "pendingEnemy");
    setup =
      (match get value "setup" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_setup_member1 x));
    progress =
      (match get value "progress" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_progress_member1 x));
    undo =
      (match get value "undo" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_unknown x));
  }

and decode_literal_25 value : literal_25 = `V25

and decode_save26 value : save26 =
  {
    wire =
      wire_of_json [ "version"; "checkpoint"; "pendingEnemy"; "setup"; "progress"; "undo" ] value;
    version = decode_literal_26 (field value "version");
    checkpoint = decode_save26_checkpoint (field value "checkpoint");
    pending_enemy = decode_json_boolean (field value "pendingEnemy");
    setup =
      (match get value "setup" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_setup_member1 x));
    progress =
      (match get value "progress" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_progress_member1 x));
    undo =
      (match get value "undo" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_unknown x));
  }

and decode_literal_26 value : literal_26 = `V26

and decode_save26_checkpoint value : save26_checkpoint =
  {
    wire = wire_of_json [ "seed"; "state" ] value;
    seed = decode_json_number (field value "seed");
    state = decode_save26_checkpoint_state (field value "state");
  }

and decode_save26_checkpoint_state value : save26_checkpoint_state =
  {
    wire =
      wire_of_json
        [
          "round";
          "turn";
          "winner";
          "armies";
          "conditionMemory";
          "mapId";
          "mapRevision";
          "cols";
          "rows";
          "nextId";
          "log";
          "lastCombat";
          "units";
          "tiles";
          "campaign";
          "gold";
          "factions";
          "groundItems";
          "combatRng";
        ]
        value;
    round = decode_json_number (field value "round");
    turn = decode_save35_setup_first (field value "turn");
    winner = decode_fragments_native_state38_winner (field value "winner");
    armies =
      (match get value "armies" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_armies_member1 x));
    condition_memory =
      (match get value "conditionMemory" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_log x));
    map_id = decode_json_string (field value "mapId");
    map_revision = decode_json_number (field value "mapRevision");
    cols = decode_json_number (field value "cols");
    rows = decode_json_number (field value "rows");
    next_id = decode_json_number (field value "nextId");
    log = decode_fragments_native_state38_log (field value "log");
    last_combat =
      (match get value "lastCombat" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_last_combat x));
    units = decode_save26_checkpoint_state_units (field value "units");
    tiles = decode_save26_checkpoint_state_tiles (field value "tiles");
    campaign =
      (match get value "campaign" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save26_checkpoint_state_campaign_member1 x));
    gold = decode_fragments_native_state38_gold (field value "gold");
    factions = decode_save26_checkpoint_state_factions (field value "factions");
    ground_items = decode_fragments_native_state38_ground_items (field value "groundItems");
    combat_rng =
      (match get value "combatRng" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_number x));
  }

and decode_save26_checkpoint_state_units value : save26_checkpoint_state_units =
  Array.map decode_save26_checkpoint_state_units_item (array value)

and decode_save26_checkpoint_state_units_item value : save26_checkpoint_state_units_item =
  {
    wire =
      wire_of_json
        [
          "tile";
          "id";
          "side";
          "role";
          "faction";
          "items";
          "aiGroup";
          "type";
          "hp";
          "moved";
          "attacked";
          "retreating";
          "rebuilding";
          "character";
          "experience";
        ]
        value;
    tile = decode_json_string (field value "tile");
    id = decode_json_string (field value "id");
    side = decode_save35_setup_first (field value "side");
    role =
      (match get value "role" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_units_item_role x));
    faction =
      (match get value "faction" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
    items =
      (match get value "items" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_units_item_items_member1 x));
    ai_group =
      (match get value "aiGroup" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
    type_ = decode_json_string (field value "type");
    hp = decode_json_number (field value "hp");
    moved = decode_json_boolean (field value "moved");
    attacked = decode_json_boolean (field value "attacked");
    retreating =
      (match get value "retreating" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_rule11_remember x));
    rebuilding =
      (match get value "rebuilding" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_rule11_remember x));
    character =
      (match get value "character" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
    experience =
      (match get value "experience" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_number x));
  }

and decode_save26_checkpoint_state_tiles value : save26_checkpoint_state_tiles =
  Array.map decode_save26_checkpoint_state_tiles_item (array value)

and decode_save26_checkpoint_state_tiles_item value : save26_checkpoint_state_tiles_item =
  {
    wire =
      wire_of_json
        [
          "owner";
          "id";
          "location";
          "col";
          "row";
          "terrain";
          "connections";
          "deadEnds";
          "walls";
          "floor";
          "fog";
          "light";
        ]
        value;
    owner = decode_fragments_tiles11_item_owner (field value "owner");
    id = decode_json_string (field value "id");
    location =
      (match get value "location" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
    col = decode_json_number (field value "col");
    row = decode_json_number (field value "row");
    terrain = decode_json_string (field value "terrain");
    connections =
      (match get value "connections" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_tiles11_item_connections_member1 x));
    dead_ends =
      (match get value "deadEnds" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_tiles11_item_connections_member1 x));
    walls =
      (match get value "walls" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_tiles11_item_connections_member1 x));
    floor =
      (match get value "floor" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
    fog =
      (match get value "fog" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_light28_member1_when__member1 x));
    light =
      (match get value "light" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_light28_member1 x));
  }

and decode_fragments_light28_member1_when_ value : fragments_light28_member1_when_ =
  if Migration_reader.matches value 11 then `Undefined
  else if Migration_reader.matches value 919 then
    `Object2 (decode_fragments_light28_member1_when__member1 value)
  else invalid "Invalid frozen union"

and decode_fragments_light28_member1_when__member1 value : fragments_light28_member1_when__member1 =
  {
    wire = wire_of_json [ "condition"; "remember" ] value;
    condition = decode_fragments_light28_member1_when__member1_condition (field value "condition");
    remember =
      (match get value "remember" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_rule11_remember x));
  }

and decode_fragments_light28_member1_when__member1_condition value :
    fragments_light28_member1_when__member1_condition =
  if Migration_reader.matches value 61 then `Owner (decode_fragments_rule11_condition_member0 value)
  else if Migration_reader.matches value 921 then
    `Proximity (decode_fragments_light28_member1_when__member1_condition_member1 value)
  else if Migration_reader.matches value 63 then
    `Visible_units (decode_fragments_rule11_condition_member1 value)
  else if Migration_reader.matches value 922 then
    `Has_units (decode_fragments_light28_member1_when__member1_condition_member3 value)
  else if Migration_reader.matches value 923 then
    `No_units (decode_fragments_light28_member1_when__member1_condition_member4 value)
  else if Migration_reader.matches value 68 then
    `Choice (decode_fragments_rule11_condition_member2 value)
  else if Migration_reader.matches value 70 then
    `Flag (decode_fragments_rule11_condition_member3 value)
  else if Migration_reader.matches value 72 then
    `Event (decode_fragments_rule11_condition_member4 value)
  else if Migration_reader.matches value 74 then
    `Round (decode_fragments_rule11_condition_member5 value)
  else if Migration_reader.matches value 924 then
    `Object10 (decode_fragments_light28_member1_when__member1_condition_member9 value)
  else invalid "Invalid frozen union"

and decode_fragments_light28_member1_when__member1_condition_member1 value :
    fragments_light28_member1_when__member1_condition_member1 =
  {
    wire =
      wire_of_json [ "kind"; "unitTypes"; "tile"; "radius"; "side"; "unitType"; "character" ] value;
    kind = decode_literal_proximity (field value "kind");
    unit_types =
      (match get value "unitTypes" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_log x));
    tile =
      (match get value "tile" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
    radius = decode_json_number (field value "radius");
    side = decode_fragments_rule11_condition_member1_side (field value "side");
    unit_type =
      (match get value "unitType" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
    character =
      (match get value "character" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
  }

and decode_fragments_light28_member1_when__member1_condition_member3 value :
    fragments_light28_member1_when__member1_condition_member3 =
  {
    wire = wire_of_json [ "kind"; "aiGroup"; "side"; "unitType"; "character" ] value;
    kind = decode_literal_has_units (field value "kind");
    ai_group =
      (match get value "aiGroup" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
    side = decode_fragments_rule11_condition_member1_side (field value "side");
    unit_type =
      (match get value "unitType" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
    character =
      (match get value "character" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
  }

and decode_fragments_light28_member1_when__member1_condition_member4 value :
    fragments_light28_member1_when__member1_condition_member4 =
  {
    wire = wire_of_json [ "kind"; "side"; "unitType"; "character" ] value;
    kind = decode_literal_no_units (field value "kind");
    side = decode_fragments_rule11_condition_member1_side (field value "side");
    unit_type =
      (match get value "unitType" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
    character =
      (match get value "character" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
  }

and decode_fragments_light28_member1_when__member1_condition_member9 value :
    fragments_light28_member1_when__member1_condition_member9 =
  {
    wire = wire_of_json [ "kind"; "conditions" ] value;
    kind = decode_fragments_rule11_condition_member9_kind (field value "kind");
    conditions =
      decode_fragments_light28_member1_when__member1_condition_member9_conditions
        (field value "conditions");
  }

and decode_fragments_light28_member1_when__member1_condition_member9_conditions value :
    fragments_light28_member1_when__member1_condition_member9_conditions =
  Array.map decode_fragments_light28_member1_when__member1_condition (array value)

and decode_save26_checkpoint_state_tiles_item_light value : save26_checkpoint_state_tiles_item_light
    =
  if Migration_reader.matches value 11 then `Undefined
  else if Migration_reader.matches value 927 then `Object2 (decode_fragments_light28_member1 value)
  else invalid "Invalid frozen union"

and decode_fragments_light28_member1 value : fragments_light28_member1 =
  {
    wire = wire_of_json [ "radius"; "color"; "fixture"; "dx"; "dy"; "power"; "when" ] value;
    radius = decode_json_number (field value "radius");
    color =
      (match get value "color" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_light23_color_member1 x));
    fixture =
      (match get value "fixture" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_rule11_remember x));
    dx =
      (match get value "dx" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_number x));
    dy =
      (match get value "dy" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_number x));
    power = decode_json_number (field value "power");
    when_ =
      (match get value "when" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_light28_member1_when__member1 x));
  }

and decode_save26_checkpoint_state_campaign value : save26_checkpoint_state_campaign =
  if Migration_reader.matches value 11 then `Undefined
  else if Migration_reader.matches value 929 then
    `Object2 (decode_save26_checkpoint_state_campaign_member1 value)
  else invalid "Invalid frozen union"

and decode_save26_checkpoint_state_campaign_member1 value : save26_checkpoint_state_campaign_member1
    =
  {
    wire =
      wire_of_json
        [
          "mission";
          "id";
          "vigil";
          "refuge";
          "completed";
          "marks";
          "holds";
          "recall";
          "fallenCompanions";
          "carryGold";
          "dialogue";
          "dialogueIndex";
          "outcome";
          "veterans";
          "storyVeterans";
          "evacuated";
          "choices";
        ]
        value;
    mission = decode_json_number (field value "mission");
    id = decode_json_string (field value "id");
    vigil =
      (match get value "vigil" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_campaign_member1_vigil_member1 x));
    refuge =
      (match get value "refuge" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_campaign_member1_refuge_member1 x));
    completed = decode_json_boolean (field value "completed");
    marks = decode_fragments_native_state38_log (field value "marks");
    holds =
      (match get value "holds" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_campaign_member1_holds_member1 x));
    recall = decode_fragments_native_state38_log (field value "recall");
    fallen_companions =
      (match get value "fallenCompanions" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_log x));
    carry_gold = decode_json_number (field value "carryGold");
    dialogue = decode_fragments_native_state38_campaign_member1_dialogue (field value "dialogue");
    dialogue_index = decode_json_number (field value "dialogueIndex");
    outcome = decode_fragments_tiles11_item_owner (field value "outcome");
    veterans =
      (match get value "veterans" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save26_checkpoint_state_campaign_member1_veterans_member1 x));
    story_veterans =
      (match get value "storyVeterans" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save26_checkpoint_state_campaign_member1_veterans_member1 x));
    evacuated =
      (match get value "evacuated" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save26_checkpoint_state_campaign_member1_evacuated_member1 x));
    choices =
      (match get value "choices" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_log x));
  }

and decode_save26_checkpoint_state_campaign_member1_veterans value :
    save26_checkpoint_state_campaign_member1_veterans =
  if Migration_reader.matches value 11 then `Undefined
  else if Migration_reader.matches value 931 then
    `Array (decode_save26_checkpoint_state_campaign_member1_veterans_member1 value)
  else invalid "Invalid frozen union"

and decode_save26_checkpoint_state_campaign_member1_veterans_member1 value :
    save26_checkpoint_state_campaign_member1_veterans_member1 =
  Array.map decode_save26_checkpoint_state_campaign_member1_veterans_member1_item (array value)

and decode_save26_checkpoint_state_campaign_member1_veterans_member1_item value :
    save26_checkpoint_state_campaign_member1_veterans_member1_item =
  {
    wire = wire_of_json [ "role"; "experience"; "items"; "type"; "character" ] value;
    role =
      (match get value "role" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_units_item_role x));
    experience = decode_json_number (field value "experience");
    items =
      (match get value "items" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_units_item_items_member1 x));
    type_ = decode_json_string (field value "type");
    character =
      (match get value "character" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
  }

and decode_save26_checkpoint_state_campaign_member1_evacuated value :
    save26_checkpoint_state_campaign_member1_evacuated =
  if Migration_reader.matches value 11 then `Undefined
  else if Migration_reader.matches value 934 then
    `Array (decode_save26_checkpoint_state_campaign_member1_evacuated_member1 value)
  else invalid "Invalid frozen union"

and decode_save26_checkpoint_state_campaign_member1_evacuated_member1 value :
    save26_checkpoint_state_campaign_member1_evacuated_member1 =
  Array.map decode_save26_checkpoint_state_campaign_member1_evacuated_member1_item (array value)

and decode_save26_checkpoint_state_campaign_member1_evacuated_member1_item value :
    save26_checkpoint_state_campaign_member1_evacuated_member1_item =
  {
    wire = wire_of_json [ "event"; "unit" ] value;
    event = decode_json_string (field value "event");
    unit_ = decode_save26_checkpoint_state_units_item (field value "unit");
  }

and decode_save26_checkpoint_state_factions value : save26_checkpoint_state_factions =
  Array.map decode_save26_checkpoint_state_factions_item (array value)

and decode_save26_checkpoint_state_factions_item value : save26_checkpoint_state_factions_item =
  {
    wire =
      wire_of_json
        [ "id"; "name"; "phase"; "controller"; "palette"; "recruits"; "relations"; "transfer" ]
        value;
    id = decode_json_string (field value "id");
    name =
      (match get value "name" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
    phase = decode_save35_setup_first (field value "phase");
    controller = decode_fragments_factions7_item_controller (field value "controller");
    palette = decode_json_string (field value "palette");
    recruits =
      (match get value "recruits" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_factions7_item_recruits_member1 x));
    relations =
      (match get value "relations" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_factions7_item_relations_member1 x));
    transfer =
      (match get value "transfer" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save26_checkpoint_state_factions_item_transfer_member1 x));
  }

and decode_save26_checkpoint_state_factions_item_transfer value :
    save26_checkpoint_state_factions_item_transfer =
  if Migration_reader.matches value 11 then `Undefined
  else if Migration_reader.matches value 939 then
    `Object2 (decode_save26_checkpoint_state_factions_item_transfer_member1 value)
  else invalid "Invalid frozen union"

and decode_save26_checkpoint_state_factions_item_transfer_member1 value :
    save26_checkpoint_state_factions_item_transfer_member1 =
  {
    wire = wire_of_json [ "to"; "when" ] value;
    to_ = decode_json_string (field value "to");
    when_ = decode_fragments_light28_member1_when__member1 (field value "when");
  }

and decode_save27 value : save27 =
  {
    wire =
      wire_of_json [ "version"; "checkpoint"; "pendingEnemy"; "setup"; "progress"; "undo" ] value;
    version = decode_literal_27 (field value "version");
    checkpoint = decode_save26_checkpoint (field value "checkpoint");
    pending_enemy = decode_json_boolean (field value "pendingEnemy");
    setup =
      (match get value "setup" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_setup_member1 x));
    progress =
      (match get value "progress" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_progress_member1 x));
    undo =
      (match get value "undo" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_unknown x));
  }

and decode_literal_27 value : literal_27 = `V27

and decode_save28 value : save28 =
  {
    wire =
      wire_of_json [ "version"; "checkpoint"; "pendingEnemy"; "setup"; "progress"; "undo" ] value;
    version = decode_literal_28 (field value "version");
    checkpoint = decode_save28_checkpoint (field value "checkpoint");
    pending_enemy = decode_json_boolean (field value "pendingEnemy");
    setup =
      (match get value "setup" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_setup_member1 x));
    progress =
      (match get value "progress" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_progress_member1 x));
    undo =
      (match get value "undo" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_unknown x));
  }

and decode_literal_28 value : literal_28 = `V28

and decode_save28_checkpoint value : save28_checkpoint =
  {
    wire = wire_of_json [ "seed"; "state" ] value;
    seed = decode_json_number (field value "seed");
    state = decode_save28_checkpoint_state (field value "state");
  }

and decode_save28_checkpoint_state value : save28_checkpoint_state =
  {
    wire =
      wire_of_json
        [
          "round";
          "turn";
          "winner";
          "armies";
          "conditionMemory";
          "mapId";
          "mapRevision";
          "cols";
          "rows";
          "nextId";
          "log";
          "lastCombat";
          "units";
          "tiles";
          "campaign";
          "gold";
          "factions";
          "groundItems";
          "combatRng";
        ]
        value;
    round = decode_json_number (field value "round");
    turn = decode_save35_setup_first (field value "turn");
    winner = decode_fragments_native_state38_winner (field value "winner");
    armies =
      (match get value "armies" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_armies_member1 x));
    condition_memory =
      (match get value "conditionMemory" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_log x));
    map_id = decode_json_string (field value "mapId");
    map_revision = decode_json_number (field value "mapRevision");
    cols = decode_json_number (field value "cols");
    rows = decode_json_number (field value "rows");
    next_id = decode_json_number (field value "nextId");
    log = decode_fragments_native_state38_log (field value "log");
    last_combat =
      (match get value "lastCombat" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_last_combat x));
    units = decode_save26_checkpoint_state_units (field value "units");
    tiles = decode_save26_checkpoint_state_tiles (field value "tiles");
    campaign =
      (match get value "campaign" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save28_checkpoint_state_campaign_member1 x));
    gold = decode_fragments_native_state38_gold (field value "gold");
    factions = decode_save26_checkpoint_state_factions (field value "factions");
    ground_items = decode_fragments_native_state38_ground_items (field value "groundItems");
    combat_rng =
      (match get value "combatRng" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_number x));
  }

and decode_save28_checkpoint_state_campaign value : save28_checkpoint_state_campaign =
  if Migration_reader.matches value 11 then `Undefined
  else if Migration_reader.matches value 993 then
    `Object2 (decode_save28_checkpoint_state_campaign_member1 value)
  else invalid "Invalid frozen union"

and decode_save28_checkpoint_state_campaign_member1 value : save28_checkpoint_state_campaign_member1
    =
  {
    wire =
      wire_of_json
        [
          "mission";
          "id";
          "vigil";
          "refuge";
          "completed";
          "marks";
          "holds";
          "recall";
          "fallenCompanions";
          "carryGold";
          "dialogue";
          "dialogueIndex";
          "outcome";
          "veterans";
          "storyVeterans";
          "evacuated";
          "scripts";
          "choices";
        ]
        value;
    mission = decode_json_number (field value "mission");
    id = decode_json_string (field value "id");
    vigil =
      (match get value "vigil" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_campaign_member1_vigil_member1 x));
    refuge =
      (match get value "refuge" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_campaign_member1_refuge_member1 x));
    completed = decode_json_boolean (field value "completed");
    marks = decode_fragments_native_state38_log (field value "marks");
    holds =
      (match get value "holds" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_campaign_member1_holds_member1 x));
    recall = decode_fragments_native_state38_log (field value "recall");
    fallen_companions =
      (match get value "fallenCompanions" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_log x));
    carry_gold = decode_json_number (field value "carryGold");
    dialogue = decode_fragments_native_state38_campaign_member1_dialogue (field value "dialogue");
    dialogue_index = decode_json_number (field value "dialogueIndex");
    outcome = decode_fragments_tiles11_item_owner (field value "outcome");
    veterans =
      (match get value "veterans" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save26_checkpoint_state_campaign_member1_veterans_member1 x));
    story_veterans =
      (match get value "storyVeterans" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save26_checkpoint_state_campaign_member1_veterans_member1 x));
    evacuated =
      (match get value "evacuated" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save26_checkpoint_state_campaign_member1_evacuated_member1 x));
    scripts =
      (match get value "scripts" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save28_checkpoint_state_campaign_member1_scripts_member1 x));
    choices =
      (match get value "choices" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_log x));
  }

and decode_save28_checkpoint_state_campaign_member1_scripts value :
    save28_checkpoint_state_campaign_member1_scripts =
  if Migration_reader.matches value 11 then `Undefined
  else if Migration_reader.matches value 1001 then
    `Object2 (decode_save28_checkpoint_state_campaign_member1_scripts_member1 value)
  else invalid "Invalid frozen union"

and decode_save28_checkpoint_state_campaign_member1_scripts_member1 value :
    save28_checkpoint_state_campaign_member1_scripts_member1 =
  {
    wire =
      wire_of_json
        [
          "recruitment";
          "failed";
          "active";
          "holds";
          "nextMark";
          "pending";
          "marks";
          "completed";
          "revealed";
        ]
        value;
    recruitment =
      decode_fragments_native_state38_campaign_member1_scripts_member1_recruitment
        (field value "recruitment");
    failed =
      (match get value "failed" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_log x));
    active = decode_fragments_native_state38_log (field value "active");
    holds =
      decode_fragments_native_state38_campaign_member1_scripts_member1_holds (field value "holds");
    next_mark = decode_json_number (field value "nextMark");
    pending =
      decode_fragments_native_state38_campaign_member1_scripts_member1_pending
        (field value "pending");
    marks =
      decode_save28_checkpoint_state_campaign_member1_scripts_member1_marks (field value "marks");
    completed =
      (match get value "completed" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_log x));
    revealed =
      (match get value "revealed" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_log x));
  }

and decode_fragments_native_state38_campaign_member1_scripts_member1_recruitment value :
    fragments_native_state38_campaign_member1_scripts_member1_recruitment =
  map_of_json decode_fragments_factions7_item_recruits_member1 value

and decode_fragments_native_state38_campaign_member1_scripts_member1_holds value :
    fragments_native_state38_campaign_member1_scripts_member1_holds =
  Array.map decode_fragments_native_state38_campaign_member1_scripts_member1_holds_item
    (array value)

and decode_fragments_native_state38_campaign_member1_scripts_member1_holds_item value :
    fragments_native_state38_campaign_member1_scripts_member1_holds_item =
  {
    wire = wire_of_json [ "event"; "predicate"; "turns"; "lastRound" ] value;
    event = decode_json_string (field value "event");
    predicate = decode_json_number (field value "predicate");
    turns = decode_json_number (field value "turns");
    last_round = decode_json_number (field value "lastRound");
  }

and decode_fragments_native_state38_campaign_member1_scripts_member1_pending value :
    fragments_native_state38_campaign_member1_scripts_member1_pending =
  Array.map decode_fragments_native_state38_campaign_member1_scripts_member1_pending_item
    (array value)

and decode_fragments_native_state38_campaign_member1_scripts_member1_pending_item value :
    fragments_native_state38_campaign_member1_scripts_member1_pending_item =
  {
    wire = wire_of_json [ "area"; "faction"; "aiGroup"; "units" ] value;
    area = decode_fragments_native_state38_log (field value "area");
    faction = decode_json_string (field value "faction");
    ai_group =
      (match get value "aiGroup" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_delta19_item_owner x));
    units =
      decode_fragments_native_state38_campaign_member1_scripts_member1_pending_item_units
        (field value "units");
  }

and decode_fragments_delta19_item_owner value : fragments_delta19_item_owner =
  if Migration_reader.matches value 11 then `Undefined
  else if Migration_reader.matches value 9 then `Null
  else if Migration_reader.matches value 13 then `String (decode_json_string value)
  else invalid "Invalid frozen union"

and decode_fragments_native_state38_campaign_member1_scripts_member1_pending_item_units value :
    fragments_native_state38_campaign_member1_scripts_member1_pending_item_units =
  Array.map decode_fragments_native_state38_campaign_member1_scripts_member1_pending_item_units_item
    (array value)

and decode_fragments_native_state38_campaign_member1_scripts_member1_pending_item_units_item value :
    fragments_native_state38_campaign_member1_scripts_member1_pending_item_units_item =
  {
    wire = wire_of_json [ "type"; "character"; "role"; "items" ] value;
    type_ = decode_json_string (field value "type");
    character =
      (match get value "character" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_delta19_item_owner x));
    role =
      (match get value "role" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x ->
          Some
            (decode_fragments_native_state38_campaign_member1_scripts_member1_pending_item_units_item_role
               x));
    items =
      (match get value "items" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_log x));
  }

and decode_fragments_native_state38_campaign_member1_scripts_member1_pending_item_units_item_role
    value : fragments_native_state38_campaign_member1_scripts_member1_pending_item_units_item_role =
  if Migration_reader.matches value 11 then `Undefined
  else if Migration_reader.matches value 9 then `Null
  else if Migration_reader.matches value 21 then `Leader
  else if Migration_reader.matches value 22 then `Courier
  else if Migration_reader.matches value 23 then `Captive
  else invalid "Invalid frozen union"

and decode_save28_checkpoint_state_campaign_member1_scripts_member1_marks value :
    save28_checkpoint_state_campaign_member1_scripts_member1_marks =
  Array.map decode_save28_checkpoint_state_campaign_member1_scripts_member1_marks_item (array value)

and decode_save28_checkpoint_state_campaign_member1_scripts_member1_marks_item value :
    save28_checkpoint_state_campaign_member1_scripts_member1_marks_item =
  {
    wire = wire_of_json [ "id"; "type"; "tile"; "light"; "lightRemembered" ] value;
    id = decode_json_number (field value "id");
    type_ = decode_json_string (field value "type");
    tile = decode_json_string (field value "tile");
    light =
      (match get value "light" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_light28_member1 x));
    light_remembered =
      (match get value "lightRemembered" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_rule11_remember x));
  }

and decode_save29 value : save29 =
  {
    wire =
      wire_of_json [ "version"; "checkpoint"; "pendingEnemy"; "setup"; "progress"; "undo" ] value;
    version = decode_literal_29 (field value "version");
    checkpoint = decode_save29_checkpoint (field value "checkpoint");
    pending_enemy = decode_json_boolean (field value "pendingEnemy");
    setup =
      (match get value "setup" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_setup_member1 x));
    progress =
      (match get value "progress" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_progress_member1 x));
    undo =
      (match get value "undo" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_unknown x));
  }

and decode_literal_29 value : literal_29 = `V29

and decode_save29_checkpoint value : save29_checkpoint =
  {
    wire = wire_of_json [ "seed"; "state" ] value;
    seed = decode_json_number (field value "seed");
    state = decode_save29_checkpoint_state (field value "state");
  }

and decode_save29_checkpoint_state value : save29_checkpoint_state =
  {
    wire =
      wire_of_json
        [
          "round";
          "turn";
          "winner";
          "armies";
          "conditionMemory";
          "mapId";
          "mapRevision";
          "cols";
          "rows";
          "nextId";
          "log";
          "lastCombat";
          "units";
          "tiles";
          "campaign";
          "gold";
          "factions";
          "groundItems";
          "combatRng";
        ]
        value;
    round = decode_json_number (field value "round");
    turn = decode_save35_setup_first (field value "turn");
    winner = decode_fragments_native_state38_winner (field value "winner");
    armies =
      (match get value "armies" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_armies_member1 x));
    condition_memory =
      (match get value "conditionMemory" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_log x));
    map_id = decode_json_string (field value "mapId");
    map_revision = decode_json_number (field value "mapRevision");
    cols = decode_json_number (field value "cols");
    rows = decode_json_number (field value "rows");
    next_id = decode_json_number (field value "nextId");
    log = decode_fragments_native_state38_log (field value "log");
    last_combat =
      (match get value "lastCombat" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_last_combat x));
    units = decode_save26_checkpoint_state_units (field value "units");
    tiles = decode_save26_checkpoint_state_tiles (field value "tiles");
    campaign =
      (match get value "campaign" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save29_checkpoint_state_campaign_member1 x));
    gold = decode_fragments_native_state38_gold (field value "gold");
    factions = decode_save26_checkpoint_state_factions (field value "factions");
    ground_items = decode_fragments_native_state38_ground_items (field value "groundItems");
    combat_rng =
      (match get value "combatRng" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_number x));
  }

and decode_save29_checkpoint_state_campaign value : save29_checkpoint_state_campaign =
  if Migration_reader.matches value 11 then `Undefined
  else if Migration_reader.matches value 1038 then
    `Object2 (decode_save29_checkpoint_state_campaign_member1 value)
  else invalid "Invalid frozen union"

and decode_save29_checkpoint_state_campaign_member1 value : save29_checkpoint_state_campaign_member1
    =
  {
    wire =
      wire_of_json
        [
          "mission";
          "id";
          "vigil";
          "refuge";
          "completed";
          "marks";
          "holds";
          "recall";
          "fallenCompanions";
          "carryGold";
          "dialogue";
          "dialogueIndex";
          "outcome";
          "veterans";
          "storyVeterans";
          "evacuated";
          "scripts";
          "flags";
        ]
        value;
    mission = decode_json_number (field value "mission");
    id = decode_json_string (field value "id");
    vigil =
      (match get value "vigil" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_campaign_member1_vigil_member1 x));
    refuge =
      (match get value "refuge" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_campaign_member1_refuge_member1 x));
    completed = decode_json_boolean (field value "completed");
    marks = decode_fragments_native_state38_log (field value "marks");
    holds =
      (match get value "holds" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_campaign_member1_holds_member1 x));
    recall = decode_fragments_native_state38_log (field value "recall");
    fallen_companions =
      (match get value "fallenCompanions" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_log x));
    carry_gold = decode_json_number (field value "carryGold");
    dialogue = decode_fragments_native_state38_campaign_member1_dialogue (field value "dialogue");
    dialogue_index = decode_json_number (field value "dialogueIndex");
    outcome = decode_fragments_tiles11_item_owner (field value "outcome");
    veterans =
      (match get value "veterans" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save26_checkpoint_state_campaign_member1_veterans_member1 x));
    story_veterans =
      (match get value "storyVeterans" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save26_checkpoint_state_campaign_member1_veterans_member1 x));
    evacuated =
      (match get value "evacuated" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save26_checkpoint_state_campaign_member1_evacuated_member1 x));
    scripts =
      (match get value "scripts" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save28_checkpoint_state_campaign_member1_scripts_member1 x));
    flags = decode_fragments_native_state38_campaign_member1_flags (field value "flags");
  }

and decode_fragments_native_state38_campaign_member1_flags value :
    fragments_native_state38_campaign_member1_flags =
  map_of_json decode_fragments_native_state38_campaign_member1_flags_value value

and decode_fragments_native_state38_campaign_member1_flags_value value :
    fragments_native_state38_campaign_member1_flags_value =
  Array.map decode_fragments_native_state38_campaign_member1_flags_value_item (array value)

and decode_fragments_native_state38_campaign_member1_flags_value_item value :
    fragments_native_state38_campaign_member1_flags_value_item =
  {
    wire = wire_of_json [ "state"; "stage"; "turn" ] value;
    state = decode_json_boolean (field value "state");
    stage = decode_json_number (field value "stage");
    turn = decode_json_number (field value "turn");
  }

and decode_save30 value : save30 =
  {
    wire =
      wire_of_json [ "version"; "checkpoint"; "pendingEnemy"; "setup"; "progress"; "undo" ] value;
    version = decode_literal_30 (field value "version");
    checkpoint = decode_save29_checkpoint (field value "checkpoint");
    pending_enemy = decode_json_boolean (field value "pendingEnemy");
    setup =
      (match get value "setup" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save35_setup x));
    progress =
      (match get value "progress" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_progress_member1 x));
    undo =
      (match get value "undo" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_unknown x));
  }

and decode_literal_30 value : literal_30 = `V30

and decode_save30_setup value : save30_setup =
  if Migration_reader.matches value 11 then `Undefined
  else if Migration_reader.matches value 1093 then `Object2 (decode_save35_setup value)
  else invalid "Invalid frozen union"

and decode_save35_setup value : save35_setup =
  {
    wire = wire_of_json [ "first"; "enemy" ] value;
    first = decode_save35_setup_first (field value "first");
    enemy = decode_save35_setup_enemy (field value "enemy");
  }

and decode_save35_setup_enemy value : save35_setup_enemy =
  {
    wire = wire_of_json [ "policy"; "blocks"; "recruitmentLimit"; "diplomacy" ] value;
    policy = decode_save35_setup_enemy_policy (field value "policy");
    blocks =
      (match get value "blocks" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save35_setup_enemy_blocks_member1 x));
    recruitment_limit =
      (match get value "recruitmentLimit" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_number x));
    diplomacy = decode_save35_setup_enemy_diplomacy (field value "diplomacy");
  }

and decode_save35_setup_enemy_diplomacy value : save35_setup_enemy_diplomacy =
  if Migration_reader.matches value 158 then `Diplomatic
  else if Migration_reader.matches value 159 then `Unrestricted
  else invalid "Invalid frozen union"

and decode_save31 value : save31 =
  {
    wire =
      wire_of_json [ "version"; "checkpoint"; "pendingEnemy"; "setup"; "progress"; "undo" ] value;
    version = decode_literal_31 (field value "version");
    checkpoint = decode_save29_checkpoint (field value "checkpoint");
    pending_enemy = decode_json_boolean (field value "pendingEnemy");
    setup =
      (match get value "setup" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save35_setup x));
    progress =
      (match get value "progress" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_progress_member1 x));
    undo =
      (match get value "undo" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_unknown x));
  }

and decode_literal_31 value : literal_31 = `V31

and decode_save32 value : save32 =
  {
    wire =
      wire_of_json [ "version"; "checkpoint"; "pendingEnemy"; "setup"; "progress"; "undo" ] value;
    version = decode_literal_32 (field value "version");
    checkpoint = decode_save29_checkpoint (field value "checkpoint");
    pending_enemy = decode_json_boolean (field value "pendingEnemy");
    setup =
      (match get value "setup" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save35_setup x));
    progress =
      (match get value "progress" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_progress_member1 x));
    undo =
      (match get value "undo" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_unknown x));
  }

and decode_literal_32 value : literal_32 = `V32

and decode_save33 value : save33 =
  {
    wire =
      wire_of_json [ "version"; "checkpoint"; "pendingEnemy"; "setup"; "progress"; "undo" ] value;
    version = decode_literal_33 (field value "version");
    checkpoint = decode_save29_checkpoint (field value "checkpoint");
    pending_enemy = decode_json_boolean (field value "pendingEnemy");
    setup =
      (match get value "setup" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save35_setup x));
    progress =
      (match get value "progress" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_progress_member1 x));
    undo =
      (match get value "undo" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_unknown x));
  }

and decode_literal_33 value : literal_33 = `V33

and decode_save34 value : save34 =
  {
    wire =
      wire_of_json [ "version"; "checkpoint"; "pendingEnemy"; "setup"; "progress"; "undo" ] value;
    version = decode_literal_34 (field value "version");
    checkpoint = decode_save34_checkpoint (field value "checkpoint");
    pending_enemy = decode_json_boolean (field value "pendingEnemy");
    setup =
      (match get value "setup" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save35_setup x));
    progress =
      (match get value "progress" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save2_progress_member1 x));
    undo =
      (match get value "undo" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save34_undo_member1 x));
  }

and decode_literal_34 value : literal_34 = `V34

and decode_save34_checkpoint value : save34_checkpoint =
  {
    wire = wire_of_json [ "seed"; "state" ] value;
    seed = decode_json_number (field value "seed");
    state = decode_save34_checkpoint_state (field value "state");
  }

and decode_save34_checkpoint_state value : save34_checkpoint_state =
  {
    wire =
      wire_of_json
        [
          "round";
          "turn";
          "winner";
          "armies";
          "conditionMemory";
          "mapId";
          "mapRevision";
          "cols";
          "rows";
          "nextId";
          "log";
          "lastCombat";
          "units";
          "tiles";
          "campaign";
          "gold";
          "factions";
          "groundItems";
          "combatRng";
        ]
        value;
    round = decode_json_number (field value "round");
    turn = decode_save35_setup_first (field value "turn");
    winner = decode_fragments_native_state38_winner (field value "winner");
    armies =
      (match get value "armies" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_armies_member1 x));
    condition_memory =
      (match get value "conditionMemory" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_log x));
    map_id = decode_json_string (field value "mapId");
    map_revision = decode_json_number (field value "mapRevision");
    cols = decode_json_number (field value "cols");
    rows = decode_json_number (field value "rows");
    next_id = decode_json_number (field value "nextId");
    log = decode_fragments_native_state38_log (field value "log");
    last_combat =
      (match get value "lastCombat" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_last_combat x));
    units = decode_save26_checkpoint_state_units (field value "units");
    tiles = decode_save26_checkpoint_state_tiles (field value "tiles");
    campaign =
      (match get value "campaign" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save34_checkpoint_state_campaign_member1 x));
    gold = decode_fragments_native_state38_gold (field value "gold");
    factions = decode_save26_checkpoint_state_factions (field value "factions");
    ground_items = decode_fragments_native_state38_ground_items (field value "groundItems");
    combat_rng =
      (match get value "combatRng" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_number x));
  }

and decode_save34_checkpoint_state_campaign value : save34_checkpoint_state_campaign =
  if Migration_reader.matches value 11 then `Undefined
  else if Migration_reader.matches value 1222 then
    `Object2 (decode_save34_checkpoint_state_campaign_member1 value)
  else invalid "Invalid frozen union"

and decode_save34_checkpoint_state_campaign_member1 value : save34_checkpoint_state_campaign_member1
    =
  {
    wire =
      wire_of_json
        [
          "mission";
          "id";
          "vigil";
          "refuge";
          "completed";
          "marks";
          "holds";
          "recall";
          "fallenCompanions";
          "carryGold";
          "dialogue";
          "dialogueIndex";
          "outcome";
          "veterans";
          "storyVeterans";
          "evacuated";
          "scripts";
          "flags";
        ]
        value;
    mission = decode_json_number (field value "mission");
    id = decode_json_string (field value "id");
    vigil =
      (match get value "vigil" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_campaign_member1_vigil_member1 x));
    refuge =
      (match get value "refuge" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_campaign_member1_refuge_member1 x));
    completed = decode_json_boolean (field value "completed");
    marks = decode_fragments_native_state38_log (field value "marks");
    holds =
      (match get value "holds" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_campaign_member1_holds_member1 x));
    recall = decode_fragments_native_state38_log (field value "recall");
    fallen_companions =
      (match get value "fallenCompanions" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_log x));
    carry_gold = decode_json_number (field value "carryGold");
    dialogue = decode_fragments_native_state38_campaign_member1_dialogue (field value "dialogue");
    dialogue_index = decode_json_number (field value "dialogueIndex");
    outcome = decode_fragments_tiles11_item_owner (field value "outcome");
    veterans =
      (match get value "veterans" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save26_checkpoint_state_campaign_member1_veterans_member1 x));
    story_veterans =
      (match get value "storyVeterans" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save26_checkpoint_state_campaign_member1_veterans_member1 x));
    evacuated =
      (match get value "evacuated" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save26_checkpoint_state_campaign_member1_evacuated_member1 x));
    scripts =
      (match get value "scripts" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save34_checkpoint_state_campaign_member1_scripts_member1 x));
    flags = decode_fragments_native_state38_campaign_member1_flags (field value "flags");
  }

and decode_save34_checkpoint_state_campaign_member1_scripts value :
    save34_checkpoint_state_campaign_member1_scripts =
  if Migration_reader.matches value 11 then `Undefined
  else if Migration_reader.matches value 1230 then
    `Object2 (decode_save34_checkpoint_state_campaign_member1_scripts_member1 value)
  else invalid "Invalid frozen union"

and decode_save34_checkpoint_state_campaign_member1_scripts_member1 value :
    save34_checkpoint_state_campaign_member1_scripts_member1 =
  {
    wire =
      wire_of_json
        [
          "recruitment";
          "failed";
          "active";
          "holds";
          "nextMark";
          "pending";
          "marks";
          "completed";
          "revealed";
        ]
        value;
    recruitment =
      decode_fragments_native_state38_campaign_member1_scripts_member1_recruitment
        (field value "recruitment");
    failed =
      (match get value "failed" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_log x));
    active = decode_fragments_native_state38_log (field value "active");
    holds =
      decode_fragments_native_state38_campaign_member1_scripts_member1_holds (field value "holds");
    next_mark = decode_json_number (field value "nextMark");
    pending =
      decode_fragments_native_state38_campaign_member1_scripts_member1_pending
        (field value "pending");
    marks =
      decode_save28_checkpoint_state_campaign_member1_scripts_member1_marks (field value "marks");
    completed = decode_fragments_native_state38_log (field value "completed");
    revealed = decode_fragments_native_state38_log (field value "revealed");
  }

and decode_save34_undo value : save34_undo =
  if Migration_reader.matches value 11 then `Undefined
  else if Migration_reader.matches value 1243 then `Array (decode_save34_undo_member1 value)
  else invalid "Invalid frozen union"

and decode_save34_undo_member1 value : save34_undo_member1 =
  Array.map decode_save34_undo_member1_item (array value)

and decode_save34_undo_member1_item value : save34_undo_member1_item =
  {
    wire = wire_of_json [ "round"; "unitId"; "changes" ] value;
    round = decode_json_number (field value "round");
    unit_id = decode_json_string (field value "unitId");
    changes = decode_save34_undo_member1_item_changes (field value "changes");
  }

and decode_save34_undo_member1_item_changes value : save34_undo_member1_item_changes =
  Array.map decode_save34_undo_member1_item_changes_item (array value)

and decode_save34_undo_member1_item_changes_item value : save34_undo_member1_item_changes_item =
  if Migration_reader.matches value 1247 then
    `Set (decode_save34_undo_member1_item_changes_item_member0 value)
  else if Migration_reader.matches value 1249 then
    `Remove (decode_save34_undo_member1_item_changes_item_member1 value)
  else invalid "Invalid frozen union"

and decode_save34_undo_member1_item_changes_item_member0 value :
    save34_undo_member1_item_changes_item_member0 =
  {
    wire = wire_of_json [ "op"; "path"; "value" ] value;
    op = decode_literal_set (field value "op");
    path = decode_fragments_native_state38_log (field value "path");
    value = decode_json_unknown (field value "value");
  }

and decode_literal_set value : literal_set = `Set

and decode_save34_undo_member1_item_changes_item_member1 value :
    save34_undo_member1_item_changes_item_member1 =
  {
    wire = wire_of_json [ "op"; "path" ] value;
    op = decode_literal_remove (field value "op");
    path = decode_fragments_native_state38_log (field value "path");
  }

and decode_literal_remove value : literal_remove = `Remove

and decode_save35 value : save35 =
  {
    wire =
      wire_of_json [ "version"; "checkpoint"; "pendingEnemy"; "setup"; "progress"; "undo" ] value;
    version = decode_literal_35 (field value "version");
    checkpoint = decode_save35_checkpoint (field value "checkpoint");
    pending_enemy = decode_json_boolean (field value "pendingEnemy");
    setup = decode_save35_setup (field value "setup");
    progress = decode_save35_progress (field value "progress");
    undo =
      (match get value "undo" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save34_undo_member1 x));
  }

and decode_literal_35 value : literal_35 = `V35

and decode_save35_checkpoint value : save35_checkpoint =
  {
    wire = wire_of_json [ "seed"; "state" ] value;
    seed = decode_json_number (field value "seed");
    state = decode_save35_checkpoint_state (field value "state");
  }

and decode_save35_checkpoint_state value : save35_checkpoint_state =
  {
    wire =
      wire_of_json
        [
          "round";
          "turn";
          "winner";
          "armies";
          "conditionMemory";
          "mapId";
          "mapRevision";
          "cols";
          "rows";
          "nextId";
          "log";
          "lastCombat";
          "units";
          "tiles";
          "campaign";
          "gold";
          "factions";
          "groundItems";
          "combatRng";
        ]
        value;
    round = decode_json_number (field value "round");
    turn = decode_save35_setup_first (field value "turn");
    winner = decode_fragments_native_state38_winner (field value "winner");
    armies =
      (match get value "armies" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_armies_member1 x));
    condition_memory =
      (match get value "conditionMemory" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_log x));
    map_id = decode_json_string (field value "mapId");
    map_revision = decode_json_number (field value "mapRevision");
    cols = decode_json_number (field value "cols");
    rows = decode_json_number (field value "rows");
    next_id = decode_json_number (field value "nextId");
    log = decode_fragments_native_state38_log (field value "log");
    last_combat =
      (match get value "lastCombat" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_last_combat x));
    units = decode_save26_checkpoint_state_units (field value "units");
    tiles = decode_save26_checkpoint_state_tiles (field value "tiles");
    campaign =
      (match get value "campaign" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save34_checkpoint_state_campaign_member1 x));
    gold = decode_fragments_native_state38_gold (field value "gold");
    factions = decode_save26_checkpoint_state_factions (field value "factions");
    ground_items = decode_fragments_native_state38_ground_items (field value "groundItems");
    combat_rng = decode_json_number (field value "combatRng");
  }

and decode_save35_progress value : save35_progress =
  {
    wire = wire_of_json [ "entry" ] value;
    entry = decode_save35_progress_entry (field value "entry");
  }

and decode_save35_progress_entry value : save35_progress_entry =
  {
    wire =
      wire_of_json
        [
          "id";
          "savedAt";
          "run";
          "mission";
          "round";
          "side";
          "missionStart";
          "runId";
          "branchId";
          "checkpointId";
          "ordinal";
          "kind";
          "parentId";
        ]
        value;
    id = decode_json_string (field value "id");
    saved_at = decode_json_number (field value "savedAt");
    run = decode_json_string (field value "run");
    mission = decode_json_string (field value "mission");
    round = decode_json_number (field value "round");
    side = decode_json_string (field value "side");
    mission_start = decode_json_boolean (field value "missionStart");
    run_id = decode_json_string (field value "runId");
    branch_id = decode_json_string (field value "branchId");
    checkpoint_id = decode_json_string (field value "checkpointId");
    ordinal = decode_json_number (field value "ordinal");
    kind = decode_save35_progress_entry_kind (field value "kind");
    parent_id =
      (match get value "parentId" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
  }

and decode_save35_progress_entry_kind value : save35_progress_entry_kind =
  if Migration_reader.matches value 166 then `Turn
  else if Migration_reader.matches value 1287 then `Manual
  else if Migration_reader.matches value 167 then `Recovery
  else invalid "Invalid frozen union"

and decode_literal_manual value : literal_manual = `Manual

and decode_save36 value : save36 =
  {
    wire =
      wire_of_json [ "version"; "checkpoint"; "pendingEnemy"; "setup"; "progress"; "undo" ] value;
    version = decode_literal_36 (field value "version");
    checkpoint = decode_save35_checkpoint (field value "checkpoint");
    pending_enemy = decode_json_boolean (field value "pendingEnemy");
    setup = decode_save35_setup (field value "setup");
    progress = decode_save35_progress (field value "progress");
    undo =
      (match get value "undo" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save34_undo_member1 x));
  }

and decode_literal_36 value : literal_36 = `V36

and decode_save37 value : save37 =
  {
    wire =
      wire_of_json [ "version"; "checkpoint"; "pendingEnemy"; "setup"; "progress"; "undo" ] value;
    version = decode_literal_37 (field value "version");
    checkpoint = decode_save35_checkpoint (field value "checkpoint");
    pending_enemy = decode_json_boolean (field value "pendingEnemy");
    setup = decode_save35_setup (field value "setup");
    progress = decode_save37_progress (field value "progress");
    undo =
      (match get value "undo" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save34_undo_member1 x));
  }

and decode_literal_37 value : literal_37 = `V37

and decode_save37_progress value : save37_progress =
  {
    wire = wire_of_json [ "entry" ] value;
    entry = decode_save37_progress_entry (field value "entry");
  }

and decode_save37_progress_entry value : save37_progress_entry =
  if Migration_reader.matches value 1357 then `Flash (decode_save37_progress_entry_member0 value)
  else if Migration_reader.matches value 1359 then `Turn (decode_history4_entries_item value)
  else invalid "Invalid frozen union"

and decode_save37_progress_entry_member0 value : save37_progress_entry_member0 =
  {
    wire =
      wire_of_json
        [
          "kind";
          "id";
          "savedAt";
          "run";
          "mission";
          "round";
          "side";
          "missionStart";
          "runId";
          "branchId";
          "checkpointId";
          "ordinal";
        ]
        value;
    kind = decode_literal_flash (field value "kind");
    id = decode_json_string (field value "id");
    saved_at = decode_json_number (field value "savedAt");
    run = decode_json_string (field value "run");
    mission = decode_json_string (field value "mission");
    round = decode_json_number (field value "round");
    side = decode_json_string (field value "side");
    mission_start = decode_json_boolean (field value "missionStart");
    run_id = decode_json_string (field value "runId");
    branch_id = decode_json_string (field value "branchId");
    checkpoint_id = decode_json_string (field value "checkpointId");
    ordinal = decode_json_number (field value "ordinal");
  }

and decode_literal_flash value : literal_flash = `Flash

and decode_history4_entries_item value : history4_entries_item =
  {
    wire =
      wire_of_json
        [
          "kind";
          "manual";
          "parentId";
          "id";
          "savedAt";
          "run";
          "mission";
          "round";
          "side";
          "missionStart";
          "runId";
          "branchId";
          "checkpointId";
          "ordinal";
        ]
        value;
    kind = decode_literal_turn (field value "kind");
    manual = decode_json_boolean (field value "manual");
    parent_id =
      (match get value "parentId" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
    id = decode_json_string (field value "id");
    saved_at = decode_json_number (field value "savedAt");
    run = decode_json_string (field value "run");
    mission = decode_json_string (field value "mission");
    round = decode_json_number (field value "round");
    side = decode_json_string (field value "side");
    mission_start = decode_json_boolean (field value "missionStart");
    run_id = decode_json_string (field value "runId");
    branch_id = decode_json_string (field value "branchId");
    checkpoint_id = decode_json_string (field value "checkpointId");
    ordinal = decode_json_number (field value "ordinal");
  }

and decode_save38 value : save38 =
  {
    wire =
      wire_of_json [ "version"; "checkpoint"; "pendingEnemy"; "setup"; "progress"; "undo" ] value;
    version = decode_literal_38 (field value "version");
    checkpoint = decode_save35_checkpoint (field value "checkpoint");
    pending_enemy = decode_json_boolean (field value "pendingEnemy");
    setup = decode_save35_setup (field value "setup");
    progress = decode_save37_progress (field value "progress");
    undo =
      (match get value "undo" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save34_undo_member1 x));
  }

and decode_literal_38 value : literal_38 = `V38

and decode_save39 value : save39 =
  if Migration_reader.matches value 1395 then `Current (decode_save39_member0 value)
  else if Migration_reader.matches value 1514 then `Turn (decode_save39_member1 value)
  else invalid "Invalid frozen union"

and decode_save39_member0 value : save39_member0 =
  {
    wire =
      wire_of_json [ "pendingEnemy"; "undo"; "progress"; "version"; "checkpoint"; "setup" ] value;
    pending_enemy = decode_json_boolean (field value "pendingEnemy");
    undo = decode_save34_undo_member1 (field value "undo");
    progress = decode_save39_member0_progress (field value "progress");
    version = decode_literal_39 (field value "version");
    checkpoint = decode_save39_member0_checkpoint (field value "checkpoint");
    setup = decode_save39_member0_setup (field value "setup");
  }

and decode_save39_member0_progress value : save39_member0_progress =
  {
    wire = wire_of_json [ "entry" ] value;
    entry = decode_save37_progress_entry_member0 (field value "entry");
  }

and decode_literal_39 value : literal_39 = `V39

and decode_save39_member0_checkpoint value : save39_member0_checkpoint =
  {
    wire = wire_of_json [ "seed"; "state" ] value;
    seed = decode_json_number (field value "seed");
    state = decode_save39_member0_checkpoint_state (field value "state");
  }

and decode_save39_member0_checkpoint_state value : save39_member0_checkpoint_state =
  {
    wire =
      wire_of_json
        [
          "factions";
          "groundItems";
          "armies";
          "conditionMemory";
          "mapId";
          "mapRevision";
          "cols";
          "rows";
          "round";
          "turn";
          "gold";
          "winner";
          "nextId";
          "combatRng";
          "log";
          "tiles";
          "units";
          "campaign";
          "lastCombat";
        ]
        value;
    factions = decode_save39_member0_checkpoint_state_factions (field value "factions");
    ground_items = decode_save39_member0_checkpoint_state_ground_items (field value "groundItems");
    armies =
      (match get value "armies" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_armies_member1 x));
    condition_memory =
      (match get value "conditionMemory" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_log x));
    map_id = decode_json_string (field value "mapId");
    map_revision = decode_json_number (field value "mapRevision");
    cols = decode_json_number (field value "cols");
    rows = decode_json_number (field value "rows");
    round = decode_json_number (field value "round");
    turn = decode_save35_setup_first (field value "turn");
    gold = decode_fragments_native_state38_gold (field value "gold");
    winner = decode_fragments_native_state38_winner (field value "winner");
    next_id = decode_json_number (field value "nextId");
    combat_rng = decode_json_number (field value "combatRng");
    log = decode_fragments_native_state38_log (field value "log");
    tiles = decode_save39_member0_checkpoint_state_tiles (field value "tiles");
    units = decode_save39_member0_checkpoint_state_units (field value "units");
    campaign =
      (match get value "campaign" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save39_member0_checkpoint_state_campaign_member1 x));
    last_combat =
      (match get value "lastCombat" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_last_combat x));
  }

and decode_save39_member0_checkpoint_state_factions value : save39_member0_checkpoint_state_factions
    =
  Array.map decode_save39_member0_checkpoint_state_factions_item (array value)

and decode_save39_member0_checkpoint_state_factions_item value :
    save39_member0_checkpoint_state_factions_item =
  {
    wire =
      wire_of_json
        [ "id"; "phase"; "controller"; "palette"; "name"; "recruits"; "relations"; "transfer" ]
        value;
    id = decode_json_string (field value "id");
    phase = decode_save35_setup_first (field value "phase");
    controller = decode_fragments_factions7_item_controller (field value "controller");
    palette = decode_json_string (field value "palette");
    name =
      (match get value "name" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
    recruits =
      (match get value "recruits" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_factions7_item_recruits_member1 x));
    relations =
      (match get value "relations" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_factions7_item_relations_member1 x));
    transfer =
      (match get value "transfer" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save39_member0_checkpoint_state_factions_item_transfer_member1 x));
  }

and decode_save39_member0_checkpoint_state_factions_item_transfer value :
    save39_member0_checkpoint_state_factions_item_transfer =
  if Migration_reader.matches value 11 then `Undefined
  else if Migration_reader.matches value 1420 then
    `Object2 (decode_save39_member0_checkpoint_state_factions_item_transfer_member1 value)
  else invalid "Invalid frozen union"

and decode_save39_member0_checkpoint_state_factions_item_transfer_member1 value :
    save39_member0_checkpoint_state_factions_item_transfer_member1 =
  {
    wire = wire_of_json [ "to"; "when" ] value;
    to_ = decode_json_string (field value "to");
    when_ = decode_save39_member0_checkpoint_state_tiles_item_fog_member1 (field value "when");
  }

and decode_save39_member0_checkpoint_state_tiles_item_fog_member1 value :
    save39_member0_checkpoint_state_tiles_item_fog_member1 =
  {
    wire = wire_of_json [ "condition"; "remember" ] value;
    condition =
      decode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition
        (field value "condition");
    remember =
      (match get value "remember" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_rule11_remember x));
  }

and decode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition value :
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition =
  if Migration_reader.matches value 1423 then
    `Owner (decode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0 value)
  else if Migration_reader.matches value 1433 then
    `Proximity
      (decode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member1 value)
  else if Migration_reader.matches value 1435 then
    `Visible_units (decode_fragments_rule11_condition_member1 value)
  else if Migration_reader.matches value 1436 then
    `Has_units (decode_fragments_light28_member1_when__member1_condition_member3 value)
  else if Migration_reader.matches value 1437 then
    `No_units (decode_fragments_light28_member1_when__member1_condition_member4 value)
  else if Migration_reader.matches value 1438 then
    `Choice (decode_fragments_rule11_condition_member2 value)
  else if Migration_reader.matches value 1439 then
    `Flag (decode_fragments_rule11_condition_member3 value)
  else if Migration_reader.matches value 1440 then
    `Event (decode_fragments_rule11_condition_member4 value)
  else if Migration_reader.matches value 1441 then
    `Round (decode_fragments_rule11_condition_member5 value)
  else if Migration_reader.matches value 1442 then
    `All (decode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member9 value)
  else if Migration_reader.matches value 1444 then
    `Any (decode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member10 value)
  else invalid "Invalid frozen union"

and decode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0 value :
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0 =
  {
    wire = wire_of_json [ "kind"; "tile"; "side" ] value;
    kind = decode_literal_owner (field value "kind");
    tile =
      (match get value "tile" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x ->
          Some
            (decode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile x));
    side = decode_json_string (field value "side");
  }

and decode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile value :
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile =
  if Migration_reader.matches value 11 then `Undefined
  else if Migration_reader.matches value 1425 then
    `Location
      (decode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member1
         value)
  else if Migration_reader.matches value 1426 then
    `Object3
      (decode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member2
         value)
  else if Migration_reader.matches value 1428 then
    `Object4
      (decode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member3
         value)
  else invalid "Invalid frozen union"

and decode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member1
    value : save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member1 =
  {
    wire = wire_of_json [ "location" ] value;
    location = decode_json_string (field value "location");
  }

and decode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member2
    value : save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member2 =
  {
    wire = wire_of_json [ "hex" ] value;
    hex = decode_save39_member0_checkpoint_state_ground_items_item_tile (field value "hex");
  }

and decode_save39_member0_checkpoint_state_ground_items_item_tile value :
    save39_member0_checkpoint_state_ground_items_item_tile =
  {
    wire = wire_of_json [ "col"; "row" ] value;
    col = decode_json_number (field value "col");
    row = decode_json_number (field value "row");
  }

and decode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member3
    value : save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member3 =
  {
    wire = wire_of_json [ "binding" ] value;
    binding =
      decode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member3_binding
        (field value "binding");
  }

and decode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member3_binding
    value :
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member3_binding =
  if Migration_reader.matches value 1430 then `Unit
  else if Migration_reader.matches value 1431 then `Mark
  else if Migration_reader.matches value 1432 then `Death
  else invalid "Invalid frozen union"

and decode_literal_unit value : literal_unit = `Unit
and decode_literal_mark value : literal_mark = `Mark
and decode_literal_death value : literal_death = `Death

and decode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member1 value :
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member1 =
  {
    wire =
      wire_of_json [ "kind"; "unitTypes"; "tile"; "radius"; "side"; "unitType"; "character" ] value;
    kind = decode_literal_proximity (field value "kind");
    unit_types =
      (match get value "unitTypes" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_log x));
    tile =
      (match get value "tile" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x ->
          Some
            (decode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile x));
    radius = decode_json_number (field value "radius");
    side = decode_fragments_rule11_condition_member1_side (field value "side");
    unit_type =
      (match get value "unitType" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
    character =
      (match get value "character" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
  }

and decode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member9 value :
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member9 =
  {
    wire = wire_of_json [ "kind"; "conditions" ] value;
    kind = decode_literal_all (field value "kind");
    conditions =
      decode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member9_conditions
        (field value "conditions");
  }

and decode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member9_conditions value
    : save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member9_conditions =
  Array.map decode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition (array value)

and decode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member10 value :
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member10 =
  {
    wire = wire_of_json [ "kind"; "conditions" ] value;
    kind = decode_literal_any (field value "kind");
    conditions =
      decode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member9_conditions
        (field value "conditions");
  }

and decode_save39_member0_checkpoint_state_ground_items value :
    save39_member0_checkpoint_state_ground_items =
  Array.map decode_save39_member0_checkpoint_state_ground_items_item (array value)

and decode_save39_member0_checkpoint_state_ground_items_item value :
    save39_member0_checkpoint_state_ground_items_item =
  {
    wire = wire_of_json [ "tile"; "item" ] value;
    tile = decode_save39_member0_checkpoint_state_ground_items_item_tile (field value "tile");
    item = decode_fragments_native_state38_ground_items_item_item (field value "item");
  }

and decode_save39_member0_checkpoint_state_tiles value : save39_member0_checkpoint_state_tiles =
  Array.map decode_save39_member0_checkpoint_state_tiles_item (array value)

and decode_save39_member0_checkpoint_state_tiles_item value :
    save39_member0_checkpoint_state_tiles_item =
  {
    wire =
      wire_of_json
        [
          "id";
          "location";
          "col";
          "row";
          "terrain";
          "owner";
          "connections";
          "deadEnds";
          "walls";
          "floor";
          "fog";
          "light";
        ]
        value;
    id = decode_json_string (field value "id");
    location =
      (match get value "location" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
    col = decode_json_number (field value "col");
    row = decode_json_number (field value "row");
    terrain = decode_json_string (field value "terrain");
    owner = decode_fragments_tiles11_item_owner (field value "owner");
    connections =
      (match get value "connections" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_tiles11_item_connections_member1 x));
    dead_ends =
      (match get value "deadEnds" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_tiles11_item_connections_member1 x));
    walls =
      (match get value "walls" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_tiles11_item_connections_member1 x));
    floor =
      (match get value "floor" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
    fog =
      (match get value "fog" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save39_member0_checkpoint_state_tiles_item_fog_member1 x));
    light =
      (match get value "light" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save39_member0_checkpoint_state_tiles_item_light_member1 x));
  }

and decode_save39_member0_checkpoint_state_tiles_item_fog value :
    save39_member0_checkpoint_state_tiles_item_fog =
  if Migration_reader.matches value 11 then `Undefined
  else if Migration_reader.matches value 1421 then
    `Object2 (decode_save39_member0_checkpoint_state_tiles_item_fog_member1 value)
  else invalid "Invalid frozen union"

and decode_save39_member0_checkpoint_state_tiles_item_light value :
    save39_member0_checkpoint_state_tiles_item_light =
  if Migration_reader.matches value 11 then `Undefined
  else if Migration_reader.matches value 1465 then
    `Object2 (decode_save39_member0_checkpoint_state_tiles_item_light_member1 value)
  else invalid "Invalid frozen union"

and decode_save39_member0_checkpoint_state_tiles_item_light_member1 value :
    save39_member0_checkpoint_state_tiles_item_light_member1 =
  {
    wire = wire_of_json [ "color"; "fixture"; "dx"; "dy"; "radius"; "power"; "when" ] value;
    color =
      (match get value "color" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_light23_color_member1 x));
    fixture =
      (match get value "fixture" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_rule11_remember x));
    dx =
      (match get value "dx" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_number x));
    dy =
      (match get value "dy" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_number x));
    radius = decode_json_number (field value "radius");
    power = decode_json_number (field value "power");
    when_ =
      (match get value "when" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save39_member0_checkpoint_state_tiles_item_fog_member1 x));
  }

and decode_save39_member0_checkpoint_state_units value : save39_member0_checkpoint_state_units =
  Array.map decode_save39_member0_checkpoint_state_units_item (array value)

and decode_save39_member0_checkpoint_state_units_item value :
    save39_member0_checkpoint_state_units_item =
  {
    wire =
      wire_of_json
        [
          "faction";
          "items";
          "aiGroup";
          "id";
          "type";
          "side";
          "tile";
          "hp";
          "moved";
          "attacked";
          "experience";
          "character";
          "role";
          "retreating";
          "rebuilding";
        ]
        value;
    faction =
      (match get value "faction" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
    items =
      (match get value "items" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_units_item_items_member1 x));
    ai_group =
      (match get value "aiGroup" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
    id = decode_json_string (field value "id");
    type_ = decode_json_string (field value "type");
    side = decode_save35_setup_first (field value "side");
    tile = decode_save39_member0_checkpoint_state_ground_items_item_tile (field value "tile");
    hp = decode_json_number (field value "hp");
    moved = decode_json_boolean (field value "moved");
    attacked = decode_json_boolean (field value "attacked");
    experience =
      (match get value "experience" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_number x));
    character =
      (match get value "character" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
    role =
      (match get value "role" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_units_item_role x));
    retreating =
      (match get value "retreating" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_rule11_remember x));
    rebuilding =
      (match get value "rebuilding" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_rule11_remember x));
  }

and decode_save39_member0_checkpoint_state_campaign value : save39_member0_checkpoint_state_campaign
    =
  if Migration_reader.matches value 11 then `Undefined
  else if Migration_reader.matches value 1472 then
    `Object2 (decode_save39_member0_checkpoint_state_campaign_member1 value)
  else invalid "Invalid frozen union"

and decode_save39_member0_checkpoint_state_campaign_member1 value :
    save39_member0_checkpoint_state_campaign_member1 =
  {
    wire =
      wire_of_json
        [
          "scripts";
          "evacuated";
          "id";
          "mission";
          "recall";
          "veterans";
          "storyVeterans";
          "fallenCompanions";
          "carryGold";
          "marks";
          "dialogue";
          "dialogueIndex";
          "outcome";
          "completed";
          "refuge";
          "vigil";
          "holds";
          "flags";
        ]
        value;
    scripts =
      (match get value "scripts" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save39_member0_checkpoint_state_campaign_member1_scripts_member1 x));
    evacuated =
      (match get value "evacuated" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save39_member0_checkpoint_state_campaign_member1_evacuated_member1 x));
    id = decode_json_string (field value "id");
    mission = decode_json_number (field value "mission");
    recall = decode_fragments_native_state38_log (field value "recall");
    veterans =
      (match get value "veterans" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_campaign_member1_veterans_member1 x));
    story_veterans =
      (match get value "storyVeterans" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_campaign_member1_veterans_member1 x));
    fallen_companions =
      (match get value "fallenCompanions" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_log x));
    carry_gold = decode_json_number (field value "carryGold");
    marks = decode_fragments_native_state38_log (field value "marks");
    dialogue = decode_fragments_native_state38_campaign_member1_dialogue (field value "dialogue");
    dialogue_index = decode_json_number (field value "dialogueIndex");
    outcome = decode_fragments_tiles11_item_owner (field value "outcome");
    completed = decode_json_boolean (field value "completed");
    refuge =
      (match get value "refuge" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_campaign_member1_refuge_member1 x));
    vigil =
      (match get value "vigil" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_campaign_member1_vigil_member1 x));
    holds =
      (match get value "holds" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_campaign_member1_holds_member1 x));
    flags = decode_fragments_native_state38_campaign_member1_flags (field value "flags");
  }

and decode_save39_member0_checkpoint_state_campaign_member1_scripts value :
    save39_member0_checkpoint_state_campaign_member1_scripts =
  if Migration_reader.matches value 11 then `Undefined
  else if Migration_reader.matches value 1474 then
    `Object2 (decode_save39_member0_checkpoint_state_campaign_member1_scripts_member1 value)
  else invalid "Invalid frozen union"

and decode_save39_member0_checkpoint_state_campaign_member1_scripts_member1 value :
    save39_member0_checkpoint_state_campaign_member1_scripts_member1 =
  {
    wire =
      wire_of_json
        [
          "failed";
          "completed";
          "revealed";
          "recruitment";
          "active";
          "holds";
          "marks";
          "pending";
          "nextMark";
        ]
        value;
    failed =
      (match get value "failed" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_log x));
    completed = decode_fragments_native_state38_log (field value "completed");
    revealed = decode_fragments_native_state38_log (field value "revealed");
    recruitment =
      decode_fragments_native_state38_campaign_member1_scripts_member1_recruitment
        (field value "recruitment");
    active = decode_fragments_native_state38_log (field value "active");
    holds =
      decode_fragments_native_state38_campaign_member1_scripts_member1_holds (field value "holds");
    marks =
      decode_save39_member0_checkpoint_state_campaign_member1_scripts_member1_marks
        (field value "marks");
    pending =
      decode_save39_member0_checkpoint_state_campaign_member1_scripts_member1_pending
        (field value "pending");
    next_mark = decode_json_number (field value "nextMark");
  }

and decode_save39_member0_checkpoint_state_campaign_member1_scripts_member1_marks value :
    save39_member0_checkpoint_state_campaign_member1_scripts_member1_marks =
  Array.map decode_save39_member0_checkpoint_state_campaign_member1_scripts_member1_marks_item
    (array value)

and decode_save39_member0_checkpoint_state_campaign_member1_scripts_member1_marks_item value :
    save39_member0_checkpoint_state_campaign_member1_scripts_member1_marks_item =
  {
    wire = wire_of_json [ "lightRemembered"; "id"; "type"; "tile"; "light" ] value;
    light_remembered =
      (match get value "lightRemembered" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_rule11_remember x));
    id = decode_json_number (field value "id");
    type_ = decode_json_string (field value "type");
    tile = decode_save39_member0_checkpoint_state_ground_items_item_tile (field value "tile");
    light =
      (match get value "light" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save39_member0_checkpoint_state_tiles_item_light_member1 x));
  }

and decode_save39_member0_checkpoint_state_campaign_member1_scripts_member1_pending value :
    save39_member0_checkpoint_state_campaign_member1_scripts_member1_pending =
  Array.map decode_save39_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item
    (array value)

and decode_save39_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item value :
    save39_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item =
  {
    wire = wire_of_json [ "area"; "faction"; "aiGroup"; "units" ] value;
    area =
      decode_save39_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item_area
        (field value "area");
    faction = decode_json_string (field value "faction");
    ai_group =
      (match get value "aiGroup" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_delta19_item_owner x));
    units =
      decode_fragments_native_state38_campaign_member1_scripts_member1_pending_item_units
        (field value "units");
  }

and decode_save39_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item_area value
    : save39_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item_area =
  Array.map decode_save39_member0_checkpoint_state_ground_items_item_tile (array value)

and decode_save39_member0_checkpoint_state_campaign_member1_evacuated value :
    save39_member0_checkpoint_state_campaign_member1_evacuated =
  if Migration_reader.matches value 11 then `Undefined
  else if Migration_reader.matches value 1487 then
    `Array (decode_save39_member0_checkpoint_state_campaign_member1_evacuated_member1 value)
  else invalid "Invalid frozen union"

and decode_save39_member0_checkpoint_state_campaign_member1_evacuated_member1 value :
    save39_member0_checkpoint_state_campaign_member1_evacuated_member1 =
  Array.map decode_save39_member0_checkpoint_state_campaign_member1_evacuated_member1_item
    (array value)

and decode_save39_member0_checkpoint_state_campaign_member1_evacuated_member1_item value :
    save39_member0_checkpoint_state_campaign_member1_evacuated_member1_item =
  {
    wire = wire_of_json [ "event"; "unit" ] value;
    event = decode_json_string (field value "event");
    unit_ = decode_save39_member0_checkpoint_state_units_item (field value "unit");
  }

and decode_fragments_native_state38_campaign_member1_veterans value :
    fragments_native_state38_campaign_member1_veterans =
  if Migration_reader.matches value 11 then `Undefined
  else if Migration_reader.matches value 1490 then
    `Array (decode_fragments_native_state38_campaign_member1_veterans_member1 value)
  else invalid "Invalid frozen union"

and decode_fragments_native_state38_campaign_member1_veterans_member1 value :
    fragments_native_state38_campaign_member1_veterans_member1 =
  Array.map decode_fragments_native_state38_campaign_member1_veterans_member1_item (array value)

and decode_fragments_native_state38_campaign_member1_veterans_member1_item value :
    fragments_native_state38_campaign_member1_veterans_member1_item =
  {
    wire = wire_of_json [ "items"; "type"; "experience"; "character"; "role" ] value;
    items =
      (match get value "items" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_units_item_items_member1 x));
    type_ = decode_json_string (field value "type");
    experience = decode_json_number (field value "experience");
    character =
      (match get value "character" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
    role =
      (match get value "role" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_units_item_role x));
  }

and decode_save39_member0_setup value : save39_member0_setup =
  {
    wire = wire_of_json [ "first"; "enemy" ] value;
    first = decode_save35_setup_first (field value "first");
    enemy = decode_save39_member0_setup_enemy (field value "enemy");
  }

and decode_save39_member0_setup_enemy value : save39_member0_setup_enemy =
  {
    wire = wire_of_json [ "blocks"; "policy"; "diplomacy"; "recruitmentLimit" ] value;
    blocks =
      (match get value "blocks" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_save35_setup_enemy_blocks_member1 x));
    policy = decode_save35_setup_enemy_policy (field value "policy");
    diplomacy = decode_save35_setup_enemy_diplomacy (field value "diplomacy");
    recruitment_limit =
      (match get value "recruitmentLimit" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_number x));
  }

and decode_save39_member1 value : save39_member1 =
  {
    wire =
      wire_of_json [ "pendingEnemy"; "undo"; "progress"; "version"; "checkpoint"; "setup" ] value;
    pending_enemy = decode_literal_false (field value "pendingEnemy");
    undo =
      (match get value "undo" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_undefined x));
    progress = decode_save39_member1_progress (field value "progress");
    version = decode_literal_39 (field value "version");
    checkpoint = decode_save39_member0_checkpoint (field value "checkpoint");
    setup = decode_save39_member0_setup (field value "setup");
  }

and decode_save39_member1_progress value : save39_member1_progress =
  {
    wire = wire_of_json [ "entry" ] value;
    entry = decode_history4_entries_item (field value "entry");
  }

and decode_history1 value : history1 =
  {
    wire = wire_of_json [ "version"; "entries" ] value;
    version = decode_literal_1 (field value "version");
    entries = decode_history1_entries (field value "entries");
  }

and decode_literal_1 value : literal_1 = `V1

and decode_history1_entries value : history1_entries =
  Array.map decode_history1_entries_item (array value)

and decode_history2 value : history2 =
  {
    wire = wire_of_json [ "version"; "entries" ] value;
    version = decode_literal_2 (field value "version");
    entries = decode_history2_entries (field value "entries");
  }

and decode_history2_entries value : history2_entries =
  Array.map decode_history2_entries_item (array value)

and decode_history2_entries_item value : history2_entries_item =
  {
    wire =
      wire_of_json
        [
          "id";
          "savedAt";
          "run";
          "mission";
          "round";
          "side";
          "missionStart";
          "runId";
          "branchId";
          "checkpointId";
          "ordinal";
          "kind";
          "parentId";
        ]
        value;
    id = decode_json_string (field value "id");
    saved_at = decode_json_number (field value "savedAt");
    run = decode_json_string (field value "run");
    mission = decode_json_string (field value "mission");
    round = decode_json_number (field value "round");
    side = decode_json_string (field value "side");
    mission_start = decode_json_boolean (field value "missionStart");
    run_id = decode_json_string (field value "runId");
    branch_id = decode_json_string (field value "branchId");
    checkpoint_id = decode_json_string (field value "checkpointId");
    ordinal = decode_json_number (field value "ordinal");
    kind = decode_literal_turn (field value "kind");
    parent_id =
      (match get value "parentId" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
  }

and decode_history3 value : history3 =
  {
    wire = wire_of_json [ "version"; "entries" ] value;
    version = decode_literal_3 (field value "version");
    entries = decode_history3_entries (field value "entries");
  }

and decode_history3_entries value : history3_entries =
  Array.map decode_history3_entries_item (array value)

and decode_history3_entries_item value : history3_entries_item =
  {
    wire =
      wire_of_json
        [
          "id";
          "savedAt";
          "run";
          "mission";
          "round";
          "side";
          "missionStart";
          "runId";
          "branchId";
          "checkpointId";
          "ordinal";
          "kind";
          "parentId";
        ]
        value;
    id = decode_json_string (field value "id");
    saved_at = decode_json_number (field value "savedAt");
    run = decode_json_string (field value "run");
    mission = decode_json_string (field value "mission");
    round = decode_json_number (field value "round");
    side = decode_json_string (field value "side");
    mission_start = decode_json_boolean (field value "missionStart");
    run_id = decode_json_string (field value "runId");
    branch_id = decode_json_string (field value "branchId");
    checkpoint_id = decode_json_string (field value "checkpointId");
    ordinal = decode_json_number (field value "ordinal");
    kind = decode_history3_entries_item_kind (field value "kind");
    parent_id =
      (match get value "parentId" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
  }

and decode_history3_entries_item_kind value : history3_entries_item_kind =
  if Migration_reader.matches value 166 then `Turn
  else if Migration_reader.matches value 1287 then `Manual
  else invalid "Invalid frozen union"

and decode_history4 value : history4 =
  {
    wire = wire_of_json [ "version"; "entries" ] value;
    version = decode_literal_4 (field value "version");
    entries = decode_history4_entries (field value "entries");
  }

and decode_history4_entries value : history4_entries =
  Array.map decode_history4_entries_item (array value)

and decode_fragments value : fragments =
  {
    wire =
      wire_of_json
        [
          "nativeState38";
          "factions7";
          "factions11";
          "factions21";
          "factions24";
          "tiles11";
          "rule11";
          "delta19";
          "delta20";
          "delta21";
          "light23";
          "light28";
        ]
        value;
    native_state38 = decode_fragments_native_state38 (field value "nativeState38");
    factions7 = decode_fragments_factions7 (field value "factions7");
    factions11 = decode_fragments_factions7 (field value "factions11");
    factions21 = decode_fragments_factions7 (field value "factions21");
    factions24 = decode_fragments_factions7 (field value "factions24");
    tiles11 = decode_fragments_tiles11 (field value "tiles11");
    rule11 = decode_fragments_rule11 (field value "rule11");
    delta19 = decode_fragments_delta19 (field value "delta19");
    delta20 = decode_fragments_delta19 (field value "delta20");
    delta21 = decode_fragments_delta19 (field value "delta21");
    light23 = decode_fragments_light23 (field value "light23");
    light28 = decode_fragments_light28 (field value "light28");
  }

and decode_fragments_native_state38 value : fragments_native_state38 =
  {
    wire =
      wire_of_json
        [
          "factions";
          "groundItems";
          "armies";
          "conditionMemory";
          "mapId";
          "mapRevision";
          "cols";
          "rows";
          "round";
          "turn";
          "gold";
          "winner";
          "nextId";
          "combatRng";
          "log";
          "tiles";
          "units";
          "campaign";
          "lastCombat";
        ]
        value;
    factions = decode_fragments_native_state38_factions (field value "factions");
    ground_items = decode_fragments_native_state38_ground_items (field value "groundItems");
    armies =
      (match get value "armies" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_armies_member1 x));
    condition_memory =
      (match get value "conditionMemory" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_log x));
    map_id = decode_json_string (field value "mapId");
    map_revision = decode_json_number (field value "mapRevision");
    cols = decode_json_number (field value "cols");
    rows = decode_json_number (field value "rows");
    round = decode_json_number (field value "round");
    turn = decode_save35_setup_first (field value "turn");
    gold = decode_fragments_native_state38_gold (field value "gold");
    winner = decode_fragments_native_state38_winner (field value "winner");
    next_id = decode_json_number (field value "nextId");
    combat_rng = decode_json_number (field value "combatRng");
    log = decode_fragments_native_state38_log (field value "log");
    tiles = decode_fragments_native_state38_tiles (field value "tiles");
    units = decode_fragments_native_state38_units (field value "units");
    campaign =
      (match get value "campaign" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_campaign_member1 x));
    last_combat =
      (match get value "lastCombat" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_last_combat x));
  }

and decode_fragments_native_state38_factions value : fragments_native_state38_factions =
  Array.map decode_fragments_native_state38_factions_item (array value)

and decode_fragments_native_state38_factions_item value : fragments_native_state38_factions_item =
  {
    wire =
      wire_of_json
        [ "id"; "phase"; "controller"; "palette"; "name"; "recruits"; "relations"; "transfer" ]
        value;
    id = decode_json_string (field value "id");
    phase = decode_save35_setup_first (field value "phase");
    controller = decode_fragments_factions7_item_controller (field value "controller");
    palette = decode_json_string (field value "palette");
    name =
      (match get value "name" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
    recruits =
      (match get value "recruits" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_factions7_item_recruits_member1 x));
    relations =
      (match get value "relations" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_factions7_item_relations_member1 x));
    transfer =
      (match get value "transfer" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_factions_item_transfer_member1 x));
  }

and decode_fragments_native_state38_factions_item_transfer value :
    fragments_native_state38_factions_item_transfer =
  if Migration_reader.matches value 11 then `Undefined
  else if Migration_reader.matches value 1533 then
    `Object2 (decode_fragments_native_state38_factions_item_transfer_member1 value)
  else invalid "Invalid frozen union"

and decode_fragments_native_state38_factions_item_transfer_member1 value :
    fragments_native_state38_factions_item_transfer_member1 =
  {
    wire = wire_of_json [ "to"; "when" ] value;
    to_ = decode_json_string (field value "to");
    when_ = decode_fragments_native_state38_tiles_item_fog_member1 (field value "when");
  }

and decode_fragments_native_state38_tiles_item_fog_member1 value :
    fragments_native_state38_tiles_item_fog_member1 =
  {
    wire = wire_of_json [ "condition"; "remember" ] value;
    condition =
      decode_fragments_native_state38_tiles_item_fog_member1_condition (field value "condition");
    remember =
      (match get value "remember" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_rule11_remember x));
  }

and decode_fragments_native_state38_tiles_item_fog_member1_condition value :
    fragments_native_state38_tiles_item_fog_member1_condition =
  if Migration_reader.matches value 61 then `Owner (decode_fragments_rule11_condition_member0 value)
  else if Migration_reader.matches value 921 then
    `Proximity (decode_fragments_light28_member1_when__member1_condition_member1 value)
  else if Migration_reader.matches value 63 then
    `Visible_units (decode_fragments_rule11_condition_member1 value)
  else if Migration_reader.matches value 922 then
    `Has_units (decode_fragments_light28_member1_when__member1_condition_member3 value)
  else if Migration_reader.matches value 923 then
    `No_units (decode_fragments_light28_member1_when__member1_condition_member4 value)
  else if Migration_reader.matches value 68 then
    `Choice (decode_fragments_rule11_condition_member2 value)
  else if Migration_reader.matches value 70 then
    `Flag (decode_fragments_rule11_condition_member3 value)
  else if Migration_reader.matches value 72 then
    `Event (decode_fragments_rule11_condition_member4 value)
  else if Migration_reader.matches value 74 then
    `Round (decode_fragments_rule11_condition_member5 value)
  else if Migration_reader.matches value 1536 then
    `All (decode_fragments_native_state38_tiles_item_fog_member1_condition_member9 value)
  else if Migration_reader.matches value 1538 then
    `Any (decode_fragments_native_state38_tiles_item_fog_member1_condition_member10 value)
  else invalid "Invalid frozen union"

and decode_fragments_native_state38_tiles_item_fog_member1_condition_member9 value :
    fragments_native_state38_tiles_item_fog_member1_condition_member9 =
  {
    wire = wire_of_json [ "kind"; "conditions" ] value;
    kind = decode_literal_all (field value "kind");
    conditions =
      decode_fragments_native_state38_tiles_item_fog_member1_condition_member9_conditions
        (field value "conditions");
  }

and decode_fragments_native_state38_tiles_item_fog_member1_condition_member9_conditions value :
    fragments_native_state38_tiles_item_fog_member1_condition_member9_conditions =
  Array.map decode_fragments_native_state38_tiles_item_fog_member1_condition (array value)

and decode_fragments_native_state38_tiles_item_fog_member1_condition_member10 value :
    fragments_native_state38_tiles_item_fog_member1_condition_member10 =
  {
    wire = wire_of_json [ "kind"; "conditions" ] value;
    kind = decode_literal_any (field value "kind");
    conditions =
      decode_fragments_native_state38_tiles_item_fog_member1_condition_member9_conditions
        (field value "conditions");
  }

and decode_fragments_native_state38_tiles value : fragments_native_state38_tiles =
  Array.map decode_fragments_native_state38_tiles_item (array value)

and decode_fragments_native_state38_tiles_item value : fragments_native_state38_tiles_item =
  {
    wire =
      wire_of_json
        [
          "id";
          "location";
          "col";
          "row";
          "terrain";
          "owner";
          "connections";
          "deadEnds";
          "walls";
          "floor";
          "fog";
          "light";
        ]
        value;
    id = decode_json_string (field value "id");
    location =
      (match get value "location" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
    col = decode_json_number (field value "col");
    row = decode_json_number (field value "row");
    terrain = decode_json_string (field value "terrain");
    owner = decode_fragments_tiles11_item_owner (field value "owner");
    connections =
      (match get value "connections" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_tiles11_item_connections_member1 x));
    dead_ends =
      (match get value "deadEnds" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_tiles11_item_connections_member1 x));
    walls =
      (match get value "walls" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_tiles11_item_connections_member1 x));
    floor =
      (match get value "floor" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
    fog =
      (match get value "fog" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_tiles_item_fog_member1 x));
    light =
      (match get value "light" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_tiles_item_light_member1 x));
  }

and decode_fragments_native_state38_tiles_item_fog value : fragments_native_state38_tiles_item_fog =
  if Migration_reader.matches value 11 then `Undefined
  else if Migration_reader.matches value 1534 then
    `Object2 (decode_fragments_native_state38_tiles_item_fog_member1 value)
  else invalid "Invalid frozen union"

and decode_fragments_native_state38_tiles_item_light value :
    fragments_native_state38_tiles_item_light =
  if Migration_reader.matches value 11 then `Undefined
  else if Migration_reader.matches value 1544 then
    `Object2 (decode_fragments_native_state38_tiles_item_light_member1 value)
  else invalid "Invalid frozen union"

and decode_fragments_native_state38_tiles_item_light_member1 value :
    fragments_native_state38_tiles_item_light_member1 =
  {
    wire = wire_of_json [ "color"; "fixture"; "dx"; "dy"; "radius"; "power"; "when" ] value;
    color =
      (match get value "color" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_light23_color_member1 x));
    fixture =
      (match get value "fixture" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_rule11_remember x));
    dx =
      (match get value "dx" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_number x));
    dy =
      (match get value "dy" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_number x));
    radius = decode_json_number (field value "radius");
    power = decode_json_number (field value "power");
    when_ =
      (match get value "when" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_tiles_item_fog_member1 x));
  }

and decode_fragments_native_state38_units value : fragments_native_state38_units =
  Array.map decode_fragments_native_state38_units_item (array value)

and decode_fragments_native_state38_units_item value : fragments_native_state38_units_item =
  {
    wire =
      wire_of_json
        [
          "faction";
          "items";
          "aiGroup";
          "id";
          "type";
          "side";
          "tile";
          "hp";
          "moved";
          "attacked";
          "experience";
          "character";
          "role";
          "retreating";
          "rebuilding";
        ]
        value;
    faction =
      (match get value "faction" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
    items =
      (match get value "items" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_units_item_items_member1 x));
    ai_group =
      (match get value "aiGroup" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
    id = decode_json_string (field value "id");
    type_ = decode_json_string (field value "type");
    side = decode_save35_setup_first (field value "side");
    tile = decode_json_string (field value "tile");
    hp = decode_json_number (field value "hp");
    moved = decode_json_boolean (field value "moved");
    attacked = decode_json_boolean (field value "attacked");
    experience =
      (match get value "experience" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_number x));
    character =
      (match get value "character" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
    role =
      (match get value "role" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_units_item_role x));
    retreating =
      (match get value "retreating" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_rule11_remember x));
    rebuilding =
      (match get value "rebuilding" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_rule11_remember x));
  }

and decode_fragments_native_state38_campaign value : fragments_native_state38_campaign =
  if Migration_reader.matches value 11 then `Undefined
  else if Migration_reader.matches value 1548 then
    `Object2 (decode_fragments_native_state38_campaign_member1 value)
  else invalid "Invalid frozen union"

and decode_fragments_native_state38_campaign_member1 value :
    fragments_native_state38_campaign_member1 =
  {
    wire =
      wire_of_json
        [
          "scripts";
          "evacuated";
          "id";
          "mission";
          "recall";
          "veterans";
          "storyVeterans";
          "fallenCompanions";
          "carryGold";
          "marks";
          "dialogue";
          "dialogueIndex";
          "outcome";
          "completed";
          "refuge";
          "vigil";
          "holds";
          "flags";
        ]
        value;
    scripts =
      (match get value "scripts" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_campaign_member1_scripts_member1 x));
    evacuated =
      (match get value "evacuated" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_campaign_member1_evacuated_member1 x));
    id = decode_json_string (field value "id");
    mission = decode_json_number (field value "mission");
    recall = decode_fragments_native_state38_log (field value "recall");
    veterans =
      (match get value "veterans" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_campaign_member1_veterans_member1 x));
    story_veterans =
      (match get value "storyVeterans" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_campaign_member1_veterans_member1 x));
    fallen_companions =
      (match get value "fallenCompanions" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_log x));
    carry_gold = decode_json_number (field value "carryGold");
    marks = decode_fragments_native_state38_log (field value "marks");
    dialogue = decode_fragments_native_state38_campaign_member1_dialogue (field value "dialogue");
    dialogue_index = decode_json_number (field value "dialogueIndex");
    outcome = decode_fragments_tiles11_item_owner (field value "outcome");
    completed = decode_json_boolean (field value "completed");
    refuge =
      (match get value "refuge" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_campaign_member1_refuge_member1 x));
    vigil =
      (match get value "vigil" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_campaign_member1_vigil_member1 x));
    holds =
      (match get value "holds" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_campaign_member1_holds_member1 x));
    flags = decode_fragments_native_state38_campaign_member1_flags (field value "flags");
  }

and decode_fragments_native_state38_campaign_member1_scripts value :
    fragments_native_state38_campaign_member1_scripts =
  if Migration_reader.matches value 11 then `Undefined
  else if Migration_reader.matches value 1550 then
    `Object2 (decode_fragments_native_state38_campaign_member1_scripts_member1 value)
  else invalid "Invalid frozen union"

and decode_fragments_native_state38_campaign_member1_scripts_member1 value :
    fragments_native_state38_campaign_member1_scripts_member1 =
  {
    wire =
      wire_of_json
        [
          "failed";
          "completed";
          "revealed";
          "recruitment";
          "active";
          "holds";
          "marks";
          "pending";
          "nextMark";
        ]
        value;
    failed =
      (match get value "failed" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_log x));
    completed = decode_fragments_native_state38_log (field value "completed");
    revealed = decode_fragments_native_state38_log (field value "revealed");
    recruitment =
      decode_fragments_native_state38_campaign_member1_scripts_member1_recruitment
        (field value "recruitment");
    active = decode_fragments_native_state38_log (field value "active");
    holds =
      decode_fragments_native_state38_campaign_member1_scripts_member1_holds (field value "holds");
    marks =
      decode_fragments_native_state38_campaign_member1_scripts_member1_marks (field value "marks");
    pending =
      decode_fragments_native_state38_campaign_member1_scripts_member1_pending
        (field value "pending");
    next_mark = decode_json_number (field value "nextMark");
  }

and decode_fragments_native_state38_campaign_member1_scripts_member1_marks value :
    fragments_native_state38_campaign_member1_scripts_member1_marks =
  Array.map decode_fragments_native_state38_campaign_member1_scripts_member1_marks_item
    (array value)

and decode_fragments_native_state38_campaign_member1_scripts_member1_marks_item value :
    fragments_native_state38_campaign_member1_scripts_member1_marks_item =
  {
    wire = wire_of_json [ "lightRemembered"; "id"; "type"; "tile"; "light" ] value;
    light_remembered =
      (match get value "lightRemembered" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_rule11_remember x));
    id = decode_json_number (field value "id");
    type_ = decode_json_string (field value "type");
    tile = decode_json_string (field value "tile");
    light =
      (match get value "light" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_tiles_item_light_member1 x));
  }

and decode_fragments_native_state38_campaign_member1_evacuated value :
    fragments_native_state38_campaign_member1_evacuated =
  if Migration_reader.matches value 11 then `Undefined
  else if Migration_reader.matches value 1556 then
    `Array (decode_fragments_native_state38_campaign_member1_evacuated_member1 value)
  else invalid "Invalid frozen union"

and decode_fragments_native_state38_campaign_member1_evacuated_member1 value :
    fragments_native_state38_campaign_member1_evacuated_member1 =
  Array.map decode_fragments_native_state38_campaign_member1_evacuated_member1_item (array value)

and decode_fragments_native_state38_campaign_member1_evacuated_member1_item value :
    fragments_native_state38_campaign_member1_evacuated_member1_item =
  {
    wire = wire_of_json [ "event"; "unit" ] value;
    event = decode_json_string (field value "event");
    unit_ = decode_fragments_native_state38_units_item (field value "unit");
  }

and decode_fragments_delta19 value : fragments_delta19 =
  Array.map decode_fragments_delta19_item (array value)

and decode_fragments_delta19_item value : fragments_delta19_item =
  {
    wire =
      wire_of_json
        [
          "owner";
          "id";
          "location";
          "col";
          "row";
          "terrain";
          "connections";
          "deadEnds";
          "walls";
          "floor";
          "fog";
          "light";
        ]
        value;
    owner =
      (match get value "owner" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_delta19_item_owner x));
    id = decode_json_string (field value "id");
    location =
      (match get value "location" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
    col =
      (match get value "col" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_number x));
    row =
      (match get value "row" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_number x));
    terrain =
      (match get value "terrain" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
    connections =
      (match get value "connections" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_tiles11_item_connections_member1 x));
    dead_ends =
      (match get value "deadEnds" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_tiles11_item_connections_member1 x));
    walls =
      (match get value "walls" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_tiles11_item_connections_member1 x));
    floor =
      (match get value "floor" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
    fog =
      (match get value "fog" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_rule11 x));
    light =
      (match get value "light" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_light23 x));
  }

and decode_fragments_light28 value : fragments_light28 =
  if Migration_reader.matches value 9 then `Null
  else if Migration_reader.matches value 991 then `Object2 (decode_fragments_light28_member1 value)
  else invalid "Invalid frozen union"

and decode_native1 value : native1 =
  {
    wire = wire_of_json [ "engine"; "version"; "pendingEnemy"; "saved" ] value;
    engine = decode_literal_rust_wasm (field value "engine");
    version =
      (match get value "version" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_literal_1 x));
    pending_enemy = decode_json_boolean (field value "pendingEnemy");
    saved = decode_native1_saved (field value "saved");
  }

and decode_literal_rust_wasm value : literal_rust_wasm = `Rust_wasm

and decode_native1_version value : native1_version =
  if Migration_reader.matches value 11 then `Undefined
  else if Migration_reader.matches value 1517 then `V1
  else invalid "Invalid frozen union"

and decode_native1_saved value : native1_saved =
  {
    wire = wire_of_json [ "seed"; "state"; "version" ] value;
    seed = decode_json_number (field value "seed");
    state = decode_save2_checkpoint_state (field value "state");
    version = decode_literal_1 (field value "version");
  }

and decode_javascript1 value : javascript1 =
  {
    wire = wire_of_json [ "version"; "pendingEnemy"; "campaignDeploymentFormat"; "state" ] value;
    version = decode_literal_1 (field value "version");
    pending_enemy = decode_json_boolean (field value "pendingEnemy");
    campaign_deployment_format =
      (match get value "campaignDeploymentFormat" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_literal_named_waves_v1 x));
    state = decode_javascript1_state (field value "state");
  }

and decode_javascript1_campaign_deployment_format value : javascript1_campaign_deployment_format =
  if Migration_reader.matches value 11 then `Undefined
  else if Migration_reader.matches value 1574 then `Named_waves_v1
  else invalid "Invalid frozen union"

and decode_literal_named_waves_v1 value : literal_named_waves_v1 = `Named_waves_v1

and decode_javascript1_state value : javascript1_state =
  {
    wire =
      wire_of_json
        [
          "mapId";
          "mapRevision";
          "round";
          "turn";
          "winner";
          "gold";
          "nextId";
          "log";
          "units";
          "villages";
          "sites";
          "campaign";
          "lastCombat";
        ]
        value;
    map_id =
      (match get value "mapId" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
    map_revision =
      (match get value "mapRevision" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_javascript1_state_map_revision x));
    round = decode_json_number (field value "round");
    turn = decode_save35_setup_first (field value "turn");
    winner = decode_fragments_native_state38_winner (field value "winner");
    gold = decode_javascript1_state_gold (field value "gold");
    next_id = decode_json_number (field value "nextId");
    log = decode_fragments_native_state38_log (field value "log");
    units = decode_javascript1_state_units (field value "units");
    villages = decode_javascript1_state_villages (field value "villages");
    sites =
      (match get value "sites" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_javascript1_state_villages x));
    campaign =
      (match get value "campaign" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_javascript1_state_campaign_member1 x));
    last_combat =
      (match get value "lastCombat" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_last_combat x));
  }

and decode_javascript1_state_map_revision value : javascript1_state_map_revision =
  if Migration_reader.matches value 11 then `Undefined
  else if Migration_reader.matches value 9 then `Null
  else if Migration_reader.matches value 3 then `Number (decode_json_number value)
  else invalid "Invalid frozen union"

and decode_javascript1_state_units value : javascript1_state_units =
  Array.map decode_javascript1_state_units_item (array value)

and decode_javascript1_state_units_item value : javascript1_state_units_item =
  {
    wire =
      wire_of_json
        [
          "tile";
          "id";
          "side";
          "name";
          "role";
          "character";
          "faction";
          "items";
          "aiGroup";
          "type";
          "hp";
          "moved";
          "attacked";
          "retreating";
          "rebuilding";
          "experience";
        ]
        value;
    tile = decode_json_string (field value "tile");
    id = decode_json_string (field value "id");
    side = decode_save35_setup_first (field value "side");
    name =
      (match get value "name" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
    role =
      (match get value "role" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_units_item_role x));
    character =
      (match get value "character" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
    faction =
      (match get value "faction" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
    items =
      (match get value "items" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_javascript1_state_units_item_items_member1 x));
    ai_group =
      (match get value "aiGroup" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
    type_ = decode_javascript1_state_units_item_type_ (field value "type");
    hp = decode_json_number (field value "hp");
    moved = decode_json_boolean (field value "moved");
    attacked = decode_json_boolean (field value "attacked");
    retreating =
      (match get value "retreating" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_rule11_remember x));
    rebuilding =
      (match get value "rebuilding" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_rule11_remember x));
    experience =
      (match get value "experience" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_number x));
  }

and decode_javascript1_state_units_item_items value : javascript1_state_units_item_items =
  if Migration_reader.matches value 11 then `Undefined
  else if Migration_reader.matches value 1583 then
    `Array (decode_javascript1_state_units_item_items_member1 value)
  else invalid "Invalid frozen union"

and decode_javascript1_state_units_item_items_member1 value :
    javascript1_state_units_item_items_member1 =
  Array.map decode_javascript1_state_units_item_items_member1_item (array value)

and decode_javascript1_state_units_item_items_member1_item value :
    javascript1_state_units_item_items_member1_item =
  {
    wire = wire_of_json [ "id"; "armor"; "name"; "description"; "healing"; "attacks" ] value;
    id = decode_json_string (field value "id");
    armor =
      (match get value "armor" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_ground_items_item_item_armor_member1 x));
    name = decode_json_string (field value "name");
    description = decode_json_string (field value "description");
    healing =
      (match get value "healing" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_ground_items_item_item_healing_member1 x));
    attacks = decode_javascript1_state_units_item_items_member1_item_attacks (field value "attacks");
  }

and decode_javascript1_state_units_item_items_member1_item_attacks value :
    javascript1_state_units_item_items_member1_item_attacks =
  Array.map decode_javascript1_state_units_item_items_member1_item_attacks_item (array value)

and decode_javascript1_state_units_item_items_member1_item_attacks_item value :
    javascript1_state_units_item_items_member1_item_attacks_item =
  {
    wire =
      wire_of_json
        [
          "spread";
          "drain";
          "count";
          "id";
          "name";
          "damage";
          "damageType";
          "category";
          "range";
          "sound";
        ]
        value;
    spread =
      (match get value "spread" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_number x));
    drain =
      (match get value "drain" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_number x));
    count = decode_json_number (field value "count");
    id = decode_json_string (field value "id");
    name = decode_json_string (field value "name");
    damage = decode_json_number (field value "damage");
    damage_type =
      decode_fragments_native_state38_ground_items_item_item_attacks_item_damage_type
        (field value "damageType");
    category =
      decode_fragments_native_state38_ground_items_item_item_attacks_item_category
        (field value "category");
    range = decode_json_number (field value "range");
    sound = decode_json_string (field value "sound");
  }

and decode_javascript1_state_units_item_type_ value : javascript1_state_units_item_type_ =
  if Migration_reader.matches value 1594 then `Ranger
  else if Migration_reader.matches value 1595 then `Healer
  else if Migration_reader.matches value 1596 then `Briarweaver
  else if Migration_reader.matches value 1597 then `Deatharcher
  else if Migration_reader.matches value 1598 then `Direwolf
  else if Migration_reader.matches value 1599 then `Greatboar
  else if Migration_reader.matches value 1600 then `Wraith
  else if Migration_reader.matches value 1601 then `Spectre
  else if Migration_reader.matches value 1602 then `Lancer
  else if Migration_reader.matches value 1603 then `Mage
  else if Migration_reader.matches value 1604 then `Marksman
  else if Migration_reader.matches value 1605 then `Merfolk_maceguard_l2
  else if Migration_reader.matches value 1606 then `Revenant
  else if Migration_reader.matches value 1607 then `Skeleton_impact_caster_l2
  else if Migration_reader.matches value 1608 then `Soldier
  else if Migration_reader.matches value 1609 then `Tidehunter
  else if Migration_reader.matches value 1610 then `Tidesage
  else if Migration_reader.matches value 1611 then `Tridentguard
  else if Migration_reader.matches value 1612 then `Warden
  else if Migration_reader.matches value 1613 then `Champion
  else if Migration_reader.matches value 1614 then `Hammerman
  else if Migration_reader.matches value 1615 then `Warhammer
  else if Migration_reader.matches value 1616 then `Sentinel
  else if Migration_reader.matches value 1617 then `Halberdier
  else if Migration_reader.matches value 1618 then `Halberd_master
  else if Migration_reader.matches value 1619 then `Shadowblade
  else if Migration_reader.matches value 1620 then `Slinger
  else if Migration_reader.matches value 1621 then `Fireslinger
  else if Migration_reader.matches value 1622 then `Sharpshooter
  else if Migration_reader.matches value 1623 then `Crossbowman
  else if Migration_reader.matches value 1624 then `Arbalester
  else if Migration_reader.matches value 1625 then `Archmage
  else if Migration_reader.matches value 1626 then `High_healer
  else if Migration_reader.matches value 1627 then `Knight
  else if Migration_reader.matches value 1628 then `Horse_bowman
  else if Migration_reader.matches value 1629 then `Horse_marksman
  else if Migration_reader.matches value 1630 then `Skeleton_dreadguard
  else if Migration_reader.matches value 1631 then `Skeleton_flanker_l2
  else if Migration_reader.matches value 1632 then `Skeleton_flanker_l3
  else if Migration_reader.matches value 1633 then `Skeleton_impact_caster_l3
  else if Migration_reader.matches value 1634 then `Skeleton_scythe_caster_l2
  else if Migration_reader.matches value 1635 then `Skeleton_scythe_caster_l3
  else if Migration_reader.matches value 1636 then `Skeleton_rider_l2
  else if Migration_reader.matches value 1637 then `Skeleton_rider_l3
  else if Migration_reader.matches value 1638 then `Skeleton_bow_master
  else if Migration_reader.matches value 1639 then `Skeleton_axeman_l2
  else if Migration_reader.matches value 1640 then `Skeleton_axeman_l3
  else if Migration_reader.matches value 1641 then `Woodland_alpha_wolf
  else if Migration_reader.matches value 1642 then `Woodland_ancient_boar
  else if Migration_reader.matches value 1643 then `Woodland_thorn_oracle
  else if Migration_reader.matches value 1644 then `Woodland_bramble_warden
  else if Migration_reader.matches value 1645 then `Woodland_thornbinder
  else if Migration_reader.matches value 1646 then `Fox_spearguard_l2
  else if Migration_reader.matches value 1647 then `Fox_spearguard_l3
  else if Migration_reader.matches value 1648 then `Treant_l2
  else if Migration_reader.matches value 1649 then `Treant_l3
  else if Migration_reader.matches value 1650 then `Merfolk_harpoon_master
  else if Migration_reader.matches value 1651 then `Merfolk_netter_l2
  else if Migration_reader.matches value 1652 then `Merfolk_netter_l3
  else if Migration_reader.matches value 1653 then `Merfolk_reef_champion
  else if Migration_reader.matches value 1654 then `Merfolk_trident_l2
  else if Migration_reader.matches value 1655 then `Merfolk_trident_l3
  else if Migration_reader.matches value 1656 then `Merfolk_tide_oracle
  else if Migration_reader.matches value 1657 then `Merfolk_tidecrusher_l2
  else if Migration_reader.matches value 1658 then `Merfolk_tidecrusher_l3
  else if Migration_reader.matches value 1659 then `Merfolk_maceguard_l3
  else if Migration_reader.matches value 1660 then `Dragon
  else if Migration_reader.matches value 1661 then `Skeletal_hound
  else if Migration_reader.matches value 1662 then `Captives
  else if Migration_reader.matches value 1663 then `Zombie_walker
  else if Migration_reader.matches value 1664 then `Zombie_decay_caster
  else if Migration_reader.matches value 1665 then `Bear
  else if Migration_reader.matches value 1666 then `Giant_bat
  else if Migration_reader.matches value 1667 then `Giant_spider
  else if Migration_reader.matches value 1668 then `Commander
  else if Migration_reader.matches value 1669 then `Grovewarden
  else if Migration_reader.matches value 1670 then `Lich
  else if Migration_reader.matches value 1671 then `Marshal
  else if Migration_reader.matches value 1672 then `Necromancer
  else if Migration_reader.matches value 1673 then `Tidekeeper
  else if Migration_reader.matches value 1674 then `Tideseer
  else if Migration_reader.matches value 1675 then `Wildkeeper
  else if Migration_reader.matches value 1676 then `Grand_marshal
  else if Migration_reader.matches value 1677 then `Lich_commander_l3
  else if Migration_reader.matches value 1678 then `Woodland_grove_oracle
  else if Migration_reader.matches value 1679 then `Merfolk_tide_sovereign
  else if Migration_reader.matches value 1680 then `Adept
  else if Migration_reader.matches value 1681 then `Archer
  else if Migration_reader.matches value 1682 then `Boar
  else if Migration_reader.matches value 1683 then `Bonearcher
  else if Migration_reader.matches value 1684 then `Ghost
  else if Migration_reader.matches value 1685 then `Harpooner
  else if Migration_reader.matches value 1686 then `Merfolk_maceguard
  else if Migration_reader.matches value 1687 then `Reefguard
  else if Migration_reader.matches value 1688 then `Rider
  else if Migration_reader.matches value 1689 then `Scout
  else if Migration_reader.matches value 1690 then `Skeleton_impact_caster
  else if Migration_reader.matches value 1691 then `Skeleton
  else if Migration_reader.matches value 1692 then `Spearman
  else if Migration_reader.matches value 1693 then `Swordsman
  else if Migration_reader.matches value 1694 then `Thorncaller
  else if Migration_reader.matches value 1695 then `Wavemender
  else if Migration_reader.matches value 1696 then `Wolf
  else if Migration_reader.matches value 1697 then `Horse_archer
  else if Migration_reader.matches value 1698 then `Skeleton_rider
  else if Migration_reader.matches value 1699 then `Skeleton_axeman
  else if Migration_reader.matches value 1700 then `Fox_spearguard
  else if Migration_reader.matches value 1701 then `Treant
  else invalid "Invalid frozen union"

and decode_literal_ranger value : literal_ranger = `Ranger
and decode_literal_healer value : literal_healer = `Healer
and decode_literal_briarweaver value : literal_briarweaver = `Briarweaver
and decode_literal_deatharcher value : literal_deatharcher = `Deatharcher
and decode_literal_direwolf value : literal_direwolf = `Direwolf
and decode_literal_greatboar value : literal_greatboar = `Greatboar
and decode_literal_wraith value : literal_wraith = `Wraith
and decode_literal_spectre value : literal_spectre = `Spectre
and decode_literal_lancer value : literal_lancer = `Lancer
and decode_literal_mage value : literal_mage = `Mage
and decode_literal_marksman value : literal_marksman = `Marksman
and decode_literal_merfolk_maceguard_l2 value : literal_merfolk_maceguard_l2 = `Merfolk_maceguard_l2
and decode_literal_revenant value : literal_revenant = `Revenant

and decode_literal_skeleton_impact_caster_l2 value : literal_skeleton_impact_caster_l2 =
  `Skeleton_impact_caster_l2

and decode_literal_soldier value : literal_soldier = `Soldier
and decode_literal_tidehunter value : literal_tidehunter = `Tidehunter
and decode_literal_tidesage value : literal_tidesage = `Tidesage
and decode_literal_tridentguard value : literal_tridentguard = `Tridentguard
and decode_literal_warden value : literal_warden = `Warden
and decode_literal_champion value : literal_champion = `Champion
and decode_literal_hammerman value : literal_hammerman = `Hammerman
and decode_literal_warhammer value : literal_warhammer = `Warhammer
and decode_literal_sentinel value : literal_sentinel = `Sentinel
and decode_literal_halberdier value : literal_halberdier = `Halberdier
and decode_literal_halberd_master value : literal_halberd_master = `Halberd_master
and decode_literal_shadowblade value : literal_shadowblade = `Shadowblade
and decode_literal_slinger value : literal_slinger = `Slinger
and decode_literal_fireslinger value : literal_fireslinger = `Fireslinger
and decode_literal_sharpshooter value : literal_sharpshooter = `Sharpshooter
and decode_literal_crossbowman value : literal_crossbowman = `Crossbowman
and decode_literal_arbalester value : literal_arbalester = `Arbalester
and decode_literal_archmage value : literal_archmage = `Archmage
and decode_literal_high_healer value : literal_high_healer = `High_healer
and decode_literal_knight value : literal_knight = `Knight
and decode_literal_horse_bowman value : literal_horse_bowman = `Horse_bowman
and decode_literal_horse_marksman value : literal_horse_marksman = `Horse_marksman
and decode_literal_skeleton_dreadguard value : literal_skeleton_dreadguard = `Skeleton_dreadguard
and decode_literal_skeleton_flanker_l2 value : literal_skeleton_flanker_l2 = `Skeleton_flanker_l2
and decode_literal_skeleton_flanker_l3 value : literal_skeleton_flanker_l3 = `Skeleton_flanker_l3

and decode_literal_skeleton_impact_caster_l3 value : literal_skeleton_impact_caster_l3 =
  `Skeleton_impact_caster_l3

and decode_literal_skeleton_scythe_caster_l2 value : literal_skeleton_scythe_caster_l2 =
  `Skeleton_scythe_caster_l2

and decode_literal_skeleton_scythe_caster_l3 value : literal_skeleton_scythe_caster_l3 =
  `Skeleton_scythe_caster_l3

and decode_literal_skeleton_rider_l2 value : literal_skeleton_rider_l2 = `Skeleton_rider_l2
and decode_literal_skeleton_rider_l3 value : literal_skeleton_rider_l3 = `Skeleton_rider_l3
and decode_literal_skeleton_bow_master value : literal_skeleton_bow_master = `Skeleton_bow_master
and decode_literal_skeleton_axeman_l2 value : literal_skeleton_axeman_l2 = `Skeleton_axeman_l2
and decode_literal_skeleton_axeman_l3 value : literal_skeleton_axeman_l3 = `Skeleton_axeman_l3
and decode_literal_woodland_alpha_wolf value : literal_woodland_alpha_wolf = `Woodland_alpha_wolf

and decode_literal_woodland_ancient_boar value : literal_woodland_ancient_boar =
  `Woodland_ancient_boar

and decode_literal_woodland_thorn_oracle value : literal_woodland_thorn_oracle =
  `Woodland_thorn_oracle

and decode_literal_woodland_bramble_warden value : literal_woodland_bramble_warden =
  `Woodland_bramble_warden

and decode_literal_woodland_thornbinder value : literal_woodland_thornbinder = `Woodland_thornbinder
and decode_literal_fox_spearguard_l2 value : literal_fox_spearguard_l2 = `Fox_spearguard_l2
and decode_literal_fox_spearguard_l3 value : literal_fox_spearguard_l3 = `Fox_spearguard_l3
and decode_literal_treant_l2 value : literal_treant_l2 = `Treant_l2
and decode_literal_treant_l3 value : literal_treant_l3 = `Treant_l3

and decode_literal_merfolk_harpoon_master value : literal_merfolk_harpoon_master =
  `Merfolk_harpoon_master

and decode_literal_merfolk_netter_l2 value : literal_merfolk_netter_l2 = `Merfolk_netter_l2
and decode_literal_merfolk_netter_l3 value : literal_merfolk_netter_l3 = `Merfolk_netter_l3

and decode_literal_merfolk_reef_champion value : literal_merfolk_reef_champion =
  `Merfolk_reef_champion

and decode_literal_merfolk_trident_l2 value : literal_merfolk_trident_l2 = `Merfolk_trident_l2
and decode_literal_merfolk_trident_l3 value : literal_merfolk_trident_l3 = `Merfolk_trident_l3
and decode_literal_merfolk_tide_oracle value : literal_merfolk_tide_oracle = `Merfolk_tide_oracle

and decode_literal_merfolk_tidecrusher_l2 value : literal_merfolk_tidecrusher_l2 =
  `Merfolk_tidecrusher_l2

and decode_literal_merfolk_tidecrusher_l3 value : literal_merfolk_tidecrusher_l3 =
  `Merfolk_tidecrusher_l3

and decode_literal_merfolk_maceguard_l3 value : literal_merfolk_maceguard_l3 = `Merfolk_maceguard_l3
and decode_literal_dragon value : literal_dragon = `Dragon
and decode_literal_skeletal_hound value : literal_skeletal_hound = `Skeletal_hound
and decode_literal_captives value : literal_captives = `Captives
and decode_literal_zombie_walker value : literal_zombie_walker = `Zombie_walker
and decode_literal_zombie_decay_caster value : literal_zombie_decay_caster = `Zombie_decay_caster
and decode_literal_bear value : literal_bear = `Bear
and decode_literal_giant_bat value : literal_giant_bat = `Giant_bat
and decode_literal_giant_spider value : literal_giant_spider = `Giant_spider
and decode_literal_commander value : literal_commander = `Commander
and decode_literal_grovewarden value : literal_grovewarden = `Grovewarden
and decode_literal_lich value : literal_lich = `Lich
and decode_literal_marshal value : literal_marshal = `Marshal
and decode_literal_necromancer value : literal_necromancer = `Necromancer
and decode_literal_tidekeeper value : literal_tidekeeper = `Tidekeeper
and decode_literal_tideseer value : literal_tideseer = `Tideseer
and decode_literal_wildkeeper value : literal_wildkeeper = `Wildkeeper
and decode_literal_grand_marshal value : literal_grand_marshal = `Grand_marshal
and decode_literal_lich_commander_l3 value : literal_lich_commander_l3 = `Lich_commander_l3

and decode_literal_woodland_grove_oracle value : literal_woodland_grove_oracle =
  `Woodland_grove_oracle

and decode_literal_merfolk_tide_sovereign value : literal_merfolk_tide_sovereign =
  `Merfolk_tide_sovereign

and decode_literal_adept value : literal_adept = `Adept
and decode_literal_archer value : literal_archer = `Archer
and decode_literal_boar value : literal_boar = `Boar
and decode_literal_bonearcher value : literal_bonearcher = `Bonearcher
and decode_literal_ghost value : literal_ghost = `Ghost
and decode_literal_harpooner value : literal_harpooner = `Harpooner
and decode_literal_merfolk_maceguard value : literal_merfolk_maceguard = `Merfolk_maceguard
and decode_literal_reefguard value : literal_reefguard = `Reefguard
and decode_literal_rider value : literal_rider = `Rider
and decode_literal_scout value : literal_scout = `Scout

and decode_literal_skeleton_impact_caster value : literal_skeleton_impact_caster =
  `Skeleton_impact_caster

and decode_literal_skeleton value : literal_skeleton = `Skeleton
and decode_literal_spearman value : literal_spearman = `Spearman
and decode_literal_swordsman value : literal_swordsman = `Swordsman
and decode_literal_thorncaller value : literal_thorncaller = `Thorncaller
and decode_literal_wavemender value : literal_wavemender = `Wavemender
and decode_literal_wolf value : literal_wolf = `Wolf
and decode_literal_horse_archer value : literal_horse_archer = `Horse_archer
and decode_literal_skeleton_rider value : literal_skeleton_rider = `Skeleton_rider
and decode_literal_skeleton_axeman value : literal_skeleton_axeman = `Skeleton_axeman
and decode_literal_fox_spearguard value : literal_fox_spearguard = `Fox_spearguard
and decode_literal_treant value : literal_treant = `Treant

and decode_javascript1_state_villages value : javascript1_state_villages =
  Array.map decode_javascript1_state_villages_item (array value)

and decode_javascript1_state_villages_item value : javascript1_state_villages_item =
  {
    wire = wire_of_json [ "id"; "owner" ] value;
    id = decode_json_string (field value "id");
    owner = decode_fragments_tiles11_item_owner (field value "owner");
  }

and decode_javascript1_state_sites value : javascript1_state_sites =
  if Migration_reader.matches value 11 then `Undefined
  else if Migration_reader.matches value 1702 then `Array (decode_javascript1_state_villages value)
  else invalid "Invalid frozen union"

and decode_javascript1_state_campaign value : javascript1_state_campaign =
  if Migration_reader.matches value 11 then `Undefined
  else if Migration_reader.matches value 1706 then
    `Object2 (decode_javascript1_state_campaign_member1 value)
  else invalid "Invalid frozen union"

and decode_javascript1_state_campaign_member1 value : javascript1_state_campaign_member1 =
  {
    wire =
      wire_of_json
        [
          "id";
          "mission";
          "recall";
          "veterans";
          "storyVeterans";
          "fallenCompanions";
          "carryGold";
          "marks";
          "dialogue";
          "dialogueIndex";
          "outcome";
          "completed";
          "refuge";
          "vigil";
          "holds";
          "choices";
        ]
        value;
    id = decode_javascript1_state_campaign_member1_id (field value "id");
    mission = decode_json_number (field value "mission");
    recall = decode_javascript1_state_campaign_member1_recall (field value "recall");
    veterans =
      (match get value "veterans" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_javascript1_state_campaign_member1_veterans_member1 x));
    story_veterans =
      (match get value "storyVeterans" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_javascript1_state_campaign_member1_veterans_member1 x));
    fallen_companions =
      (match get value "fallenCompanions" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_log x));
    carry_gold = decode_json_number (field value "carryGold");
    marks = decode_fragments_native_state38_log (field value "marks");
    dialogue = decode_javascript1_state_campaign_member1_dialogue (field value "dialogue");
    dialogue_index = decode_json_number (field value "dialogueIndex");
    outcome = decode_fragments_tiles11_item_owner (field value "outcome");
    completed = decode_json_boolean (field value "completed");
    refuge =
      (match get value "refuge" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_campaign_member1_refuge_member1 x));
    vigil =
      (match get value "vigil" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_campaign_member1_vigil_member1 x));
    holds =
      (match get value "holds" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_campaign_member1_holds_member1 x));
    choices =
      (match get value "choices" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_log x));
  }

and decode_javascript1_state_campaign_member1_id value : javascript1_state_campaign_member1_id =
  if Migration_reader.matches value 1708 then `Lantern_road
  else if Migration_reader.matches value 1709 then `Green_accord
  else if Migration_reader.matches value 1710 then `Ashen_oath
  else if Migration_reader.matches value 1711 then `Borrowed_tide
  else if Migration_reader.matches value 1712 then `Mirror_night
  else if Migration_reader.matches value 1713 then `Bell_night
  else invalid "Invalid frozen union"

and decode_literal_lantern_road value : literal_lantern_road = `Lantern_road
and decode_literal_green_accord value : literal_green_accord = `Green_accord
and decode_literal_ashen_oath value : literal_ashen_oath = `Ashen_oath
and decode_literal_borrowed_tide value : literal_borrowed_tide = `Borrowed_tide
and decode_literal_mirror_night value : literal_mirror_night = `Mirror_night
and decode_literal_bell_night value : literal_bell_night = `Bell_night

and decode_javascript1_state_campaign_member1_recall value :
    javascript1_state_campaign_member1_recall =
  Array.map decode_javascript1_state_units_item_type_ (array value)

and decode_javascript1_state_campaign_member1_veterans value :
    javascript1_state_campaign_member1_veterans =
  if Migration_reader.matches value 11 then `Undefined
  else if Migration_reader.matches value 1716 then
    `Array (decode_javascript1_state_campaign_member1_veterans_member1 value)
  else invalid "Invalid frozen union"

and decode_javascript1_state_campaign_member1_veterans_member1 value :
    javascript1_state_campaign_member1_veterans_member1 =
  Array.map decode_javascript1_state_campaign_member1_veterans_member1_item (array value)

and decode_javascript1_state_campaign_member1_veterans_member1_item value :
    javascript1_state_campaign_member1_veterans_member1_item =
  {
    wire = wire_of_json [ "type"; "experience"; "name"; "role" ] value;
    type_ = decode_javascript1_state_units_item_type_ (field value "type");
    experience = decode_json_number (field value "experience");
    name =
      (match get value "name" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
    role =
      (match get value "role" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_fragments_native_state38_units_item_role x));
  }

and decode_javascript1_state_campaign_member1_dialogue value :
    javascript1_state_campaign_member1_dialogue =
  Array.map decode_javascript1_state_campaign_member1_dialogue_item (array value)

and decode_javascript1_state_campaign_member1_dialogue_item value :
    javascript1_state_campaign_member1_dialogue_item =
  {
    wire = wire_of_json [ "sound"; "speaker"; "character"; "portrait"; "text" ] value;
    sound =
      (match get value "sound" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
    speaker = decode_json_string (field value "speaker");
    character =
      (match get value "character" with
      | None -> None
      | Some x when is_undefined x -> None
      | Some x -> Some (decode_json_string x));
    portrait =
      decode_javascript1_state_campaign_member1_dialogue_item_portrait (field value "portrait");
    text = decode_json_string (field value "text");
  }

and decode_javascript1_state_campaign_member1_dialogue_item_portrait value :
    javascript1_state_campaign_member1_dialogue_item_portrait =
  if Migration_reader.matches value 1594 then `Ranger
  else if Migration_reader.matches value 1595 then `Healer
  else if Migration_reader.matches value 1596 then `Briarweaver
  else if Migration_reader.matches value 1597 then `Deatharcher
  else if Migration_reader.matches value 1598 then `Direwolf
  else if Migration_reader.matches value 1599 then `Greatboar
  else if Migration_reader.matches value 1600 then `Wraith
  else if Migration_reader.matches value 1601 then `Spectre
  else if Migration_reader.matches value 1602 then `Lancer
  else if Migration_reader.matches value 1603 then `Mage
  else if Migration_reader.matches value 1604 then `Marksman
  else if Migration_reader.matches value 1605 then `Merfolk_maceguard_l2
  else if Migration_reader.matches value 1606 then `Revenant
  else if Migration_reader.matches value 1607 then `Skeleton_impact_caster_l2
  else if Migration_reader.matches value 1608 then `Soldier
  else if Migration_reader.matches value 1609 then `Tidehunter
  else if Migration_reader.matches value 1610 then `Tidesage
  else if Migration_reader.matches value 1611 then `Tridentguard
  else if Migration_reader.matches value 1612 then `Warden
  else if Migration_reader.matches value 1613 then `Champion
  else if Migration_reader.matches value 1614 then `Hammerman
  else if Migration_reader.matches value 1615 then `Warhammer
  else if Migration_reader.matches value 1616 then `Sentinel
  else if Migration_reader.matches value 1617 then `Halberdier
  else if Migration_reader.matches value 1618 then `Halberd_master
  else if Migration_reader.matches value 1619 then `Shadowblade
  else if Migration_reader.matches value 1620 then `Slinger
  else if Migration_reader.matches value 1621 then `Fireslinger
  else if Migration_reader.matches value 1622 then `Sharpshooter
  else if Migration_reader.matches value 1623 then `Crossbowman
  else if Migration_reader.matches value 1624 then `Arbalester
  else if Migration_reader.matches value 1625 then `Archmage
  else if Migration_reader.matches value 1626 then `High_healer
  else if Migration_reader.matches value 1627 then `Knight
  else if Migration_reader.matches value 1628 then `Horse_bowman
  else if Migration_reader.matches value 1629 then `Horse_marksman
  else if Migration_reader.matches value 1630 then `Skeleton_dreadguard
  else if Migration_reader.matches value 1631 then `Skeleton_flanker_l2
  else if Migration_reader.matches value 1632 then `Skeleton_flanker_l3
  else if Migration_reader.matches value 1633 then `Skeleton_impact_caster_l3
  else if Migration_reader.matches value 1634 then `Skeleton_scythe_caster_l2
  else if Migration_reader.matches value 1635 then `Skeleton_scythe_caster_l3
  else if Migration_reader.matches value 1636 then `Skeleton_rider_l2
  else if Migration_reader.matches value 1637 then `Skeleton_rider_l3
  else if Migration_reader.matches value 1638 then `Skeleton_bow_master
  else if Migration_reader.matches value 1639 then `Skeleton_axeman_l2
  else if Migration_reader.matches value 1640 then `Skeleton_axeman_l3
  else if Migration_reader.matches value 1641 then `Woodland_alpha_wolf
  else if Migration_reader.matches value 1642 then `Woodland_ancient_boar
  else if Migration_reader.matches value 1643 then `Woodland_thorn_oracle
  else if Migration_reader.matches value 1644 then `Woodland_bramble_warden
  else if Migration_reader.matches value 1645 then `Woodland_thornbinder
  else if Migration_reader.matches value 1646 then `Fox_spearguard_l2
  else if Migration_reader.matches value 1647 then `Fox_spearguard_l3
  else if Migration_reader.matches value 1648 then `Treant_l2
  else if Migration_reader.matches value 1649 then `Treant_l3
  else if Migration_reader.matches value 1650 then `Merfolk_harpoon_master
  else if Migration_reader.matches value 1651 then `Merfolk_netter_l2
  else if Migration_reader.matches value 1652 then `Merfolk_netter_l3
  else if Migration_reader.matches value 1653 then `Merfolk_reef_champion
  else if Migration_reader.matches value 1654 then `Merfolk_trident_l2
  else if Migration_reader.matches value 1655 then `Merfolk_trident_l3
  else if Migration_reader.matches value 1656 then `Merfolk_tide_oracle
  else if Migration_reader.matches value 1657 then `Merfolk_tidecrusher_l2
  else if Migration_reader.matches value 1658 then `Merfolk_tidecrusher_l3
  else if Migration_reader.matches value 1659 then `Merfolk_maceguard_l3
  else if Migration_reader.matches value 1660 then `Dragon
  else if Migration_reader.matches value 1661 then `Skeletal_hound
  else if Migration_reader.matches value 1662 then `Captives
  else if Migration_reader.matches value 1663 then `Zombie_walker
  else if Migration_reader.matches value 1664 then `Zombie_decay_caster
  else if Migration_reader.matches value 1665 then `Bear
  else if Migration_reader.matches value 1666 then `Giant_bat
  else if Migration_reader.matches value 1667 then `Giant_spider
  else if Migration_reader.matches value 1668 then `Commander
  else if Migration_reader.matches value 1669 then `Grovewarden
  else if Migration_reader.matches value 1670 then `Lich
  else if Migration_reader.matches value 1671 then `Marshal
  else if Migration_reader.matches value 1672 then `Necromancer
  else if Migration_reader.matches value 1673 then `Tidekeeper
  else if Migration_reader.matches value 1674 then `Tideseer
  else if Migration_reader.matches value 1675 then `Wildkeeper
  else if Migration_reader.matches value 1676 then `Grand_marshal
  else if Migration_reader.matches value 1677 then `Lich_commander_l3
  else if Migration_reader.matches value 1678 then `Woodland_grove_oracle
  else if Migration_reader.matches value 1679 then `Merfolk_tide_sovereign
  else if Migration_reader.matches value 1680 then `Adept
  else if Migration_reader.matches value 1681 then `Archer
  else if Migration_reader.matches value 1682 then `Boar
  else if Migration_reader.matches value 1683 then `Bonearcher
  else if Migration_reader.matches value 1684 then `Ghost
  else if Migration_reader.matches value 1685 then `Harpooner
  else if Migration_reader.matches value 1686 then `Merfolk_maceguard
  else if Migration_reader.matches value 1687 then `Reefguard
  else if Migration_reader.matches value 1688 then `Rider
  else if Migration_reader.matches value 1689 then `Scout
  else if Migration_reader.matches value 1690 then `Skeleton_impact_caster
  else if Migration_reader.matches value 1691 then `Skeleton
  else if Migration_reader.matches value 1692 then `Spearman
  else if Migration_reader.matches value 1693 then `Swordsman
  else if Migration_reader.matches value 1694 then `Thorncaller
  else if Migration_reader.matches value 1695 then `Wavemender
  else if Migration_reader.matches value 1696 then `Wolf
  else if Migration_reader.matches value 1697 then `Horse_archer
  else if Migration_reader.matches value 1698 then `Skeleton_rider
  else if Migration_reader.matches value 1699 then `Skeleton_axeman
  else if Migration_reader.matches value 1700 then `Fox_spearguard
  else if Migration_reader.matches value 1701 then `Treant
  else if Migration_reader.matches value 1721 then `Worker
  else invalid "Invalid frozen union"

and decode_literal_worker value : literal_worker = `Worker

let rec encode_save2 (value : save2) : Js.Json.t =
  wire_to_json
    ~known:[ "version"; "checkpoint"; "pendingEnemy"; "planner"; "setup"; "progress"; "undo" ]
    value.wire
    ([
       ("version", encode_literal_2 value.version);
       ("checkpoint", encode_save2_checkpoint value.checkpoint);
       ("pendingEnemy", encode_json_boolean value.pending_enemy);
     ]
    @ (match value.planner with
      | None ->
          if List.mem "planner" value.wire.undefined_fields then [ ("planner", undefined) ] else []
      | Some x -> [ ("planner", encode_json_unknown x) ])
    @ (match value.setup with
      | None ->
          if List.mem "setup" value.wire.undefined_fields then [ ("setup", undefined) ] else []
      | Some x -> [ ("setup", encode_save2_setup_member1 x) ])
    @ (match value.progress with
      | None ->
          if List.mem "progress" value.wire.undefined_fields then [ ("progress", undefined) ]
          else []
      | Some x -> [ ("progress", encode_save2_progress_member1 x) ])
    @
    match value.undo with
    | None -> if List.mem "undo" value.wire.undefined_fields then [ ("undo", undefined) ] else []
    | Some x -> [ ("undo", encode_json_unknown x) ])

and encode_literal_2 (value : literal_2) : Js.Json.t = Js.Json.parseExn "2"

and encode_save2_checkpoint (value : save2_checkpoint) : Js.Json.t =
  wire_to_json ~known:[ "seed"; "state" ] value.wire
    [
      ("seed", encode_json_number value.seed); ("state", encode_save2_checkpoint_state value.state);
    ]

and encode_json_number (value : json_number) : Js.Json.t = n value

and encode_save2_checkpoint_state (value : save2_checkpoint_state) : Js.Json.t =
  wire_to_json
    ~known:
      [
        "round";
        "turn";
        "winner";
        "armies";
        "conditionMemory";
        "mapId";
        "mapRevision";
        "cols";
        "rows";
        "nextId";
        "log";
        "lastCombat";
        "units";
        "tiles";
        "campaign";
        "gold";
        "factions";
        "groundItems";
        "combatRng";
      ]
    value.wire
    ([
       ("round", encode_json_number value.round);
       ("turn", encode_save35_setup_first value.turn);
       ("winner", encode_fragments_native_state38_winner value.winner);
       ("mapId", encode_json_string value.map_id);
       ("mapRevision", encode_json_number value.map_revision);
       ("cols", encode_json_number value.cols);
       ("rows", encode_json_number value.rows);
       ("nextId", encode_json_number value.next_id);
       ("log", encode_fragments_native_state38_log value.log);
       ("units", encode_save2_checkpoint_state_units value.units);
       ("tiles", encode_save2_checkpoint_state_tiles value.tiles);
       ("gold", encode_javascript1_state_gold value.gold);
     ]
    @ (match value.armies with
      | None ->
          if List.mem "armies" value.wire.undefined_fields then [ ("armies", undefined) ] else []
      | Some x -> [ ("armies", encode_fragments_native_state38_armies_member1 x) ])
    @ (match value.condition_memory with
      | None ->
          if List.mem "conditionMemory" value.wire.undefined_fields then
            [ ("conditionMemory", undefined) ]
          else []
      | Some x -> [ ("conditionMemory", encode_fragments_native_state38_log x) ])
    @ (match value.last_combat with
      | None ->
          if List.mem "lastCombat" value.wire.undefined_fields then [ ("lastCombat", undefined) ]
          else []
      | Some x -> [ ("lastCombat", encode_fragments_native_state38_last_combat x) ])
    @ (match value.campaign with
      | None ->
          if List.mem "campaign" value.wire.undefined_fields then [ ("campaign", undefined) ]
          else []
      | Some x -> [ ("campaign", encode_save2_checkpoint_state_campaign_member1 x) ])
    @ (match value.factions with
      | None ->
          if List.mem "factions" value.wire.undefined_fields then [ ("factions", undefined) ]
          else []
      | Some x -> [ ("factions", encode_save2_checkpoint_state_factions_member1 x) ])
    @ (match value.ground_items with
      | None ->
          if List.mem "groundItems" value.wire.undefined_fields then [ ("groundItems", undefined) ]
          else []
      | Some x -> [ ("groundItems", encode_fragments_native_state38_ground_items x) ])
    @
    match value.combat_rng with
    | None ->
        if List.mem "combatRng" value.wire.undefined_fields then [ ("combatRng", undefined) ]
        else []
    | Some x -> [ ("combatRng", encode_json_number x) ])

and encode_save35_setup_first (value : save35_setup_first) : Js.Json.t =
  match value with
  | `Player -> Js.Json.parseExn "\"player\""
  | `Enemy -> Js.Json.parseExn "\"enemy\""

and encode_literal_player (value : literal_player) : Js.Json.t = Js.Json.parseExn "\"player\""
and encode_literal_enemy (value : literal_enemy) : Js.Json.t = Js.Json.parseExn "\"enemy\""

and encode_fragments_native_state38_winner (value : fragments_native_state38_winner) : Js.Json.t =
  match value with
  | `Null -> json_null
  | `Player -> Js.Json.parseExn "\"player\""
  | `Enemy -> Js.Json.parseExn "\"enemy\""

and encode_json_null (value : json_null) : Js.Json.t = json_null

and encode_fragments_native_state38_armies (value : fragments_native_state38_armies) : Js.Json.t =
  match value with
  | `Undefined -> undefined
  | `Tuple x -> encode_fragments_native_state38_armies_member1 x

and encode_json_undefined (value : json_undefined) : Js.Json.t = undefined

and encode_fragments_native_state38_armies_member1 (value : fragments_native_state38_armies_member1)
    : Js.Json.t =
  let v0, v1 = value in
  a [ encode_json_string v0; encode_json_string v1 ]

and encode_json_string (value : json_string) : Js.Json.t = s value

and encode_fragments_native_state38_condition_memory
    (value : fragments_native_state38_condition_memory) : Js.Json.t =
  match value with `Undefined -> undefined | `Array x -> encode_fragments_native_state38_log x

and encode_fragments_native_state38_log (value : fragments_native_state38_log) : Js.Json.t =
  Js.Json.array (Array.map encode_json_string value)

and encode_fragments_native_state38_last_combat (value : fragments_native_state38_last_combat) :
    Js.Json.t =
  match value with
  | `Undefined -> undefined
  | `Null -> json_null
  | `Object3 x -> encode_fragments_native_state38_last_combat_member2 x

and encode_fragments_native_state38_last_combat_member2
    (value : fragments_native_state38_last_combat_member2) : Js.Json.t =
  wire_to_json
    ~known:[ "attackerId"; "attackId"; "sound" ]
    value.wire
    [
      ("attackerId", encode_json_string value.attacker_id);
      ("attackId", encode_json_string value.attack_id);
      ("sound", encode_json_string value.sound);
    ]

and encode_save2_checkpoint_state_units (value : save2_checkpoint_state_units) : Js.Json.t =
  Js.Json.array (Array.map encode_save2_checkpoint_state_units_item value)

and encode_save2_checkpoint_state_units_item (value : save2_checkpoint_state_units_item) : Js.Json.t
    =
  wire_to_json
    ~known:
      [
        "tile";
        "id";
        "side";
        "role";
        "faction";
        "items";
        "aiGroup";
        "type";
        "hp";
        "moved";
        "attacked";
        "retreating";
        "rebuilding";
        "name";
        "character";
        "experience";
      ]
    value.wire
    ([
       ("tile", encode_json_string value.tile);
       ("id", encode_json_string value.id);
       ("side", encode_save35_setup_first value.side);
       ("type", encode_json_string value.type_);
       ("hp", encode_json_number value.hp);
       ("moved", encode_json_boolean value.moved);
       ("attacked", encode_json_boolean value.attacked);
     ]
    @ (match value.role with
      | None -> if List.mem "role" value.wire.undefined_fields then [ ("role", undefined) ] else []
      | Some x -> [ ("role", encode_fragments_native_state38_units_item_role x) ])
    @ (match value.faction with
      | None ->
          if List.mem "faction" value.wire.undefined_fields then [ ("faction", undefined) ] else []
      | Some x -> [ ("faction", encode_json_string x) ])
    @ (match value.items with
      | None ->
          if List.mem "items" value.wire.undefined_fields then [ ("items", undefined) ] else []
      | Some x -> [ ("items", encode_fragments_native_state38_units_item_items_member1 x) ])
    @ (match value.ai_group with
      | None ->
          if List.mem "aiGroup" value.wire.undefined_fields then [ ("aiGroup", undefined) ] else []
      | Some x -> [ ("aiGroup", encode_json_string x) ])
    @ (match value.retreating with
      | None ->
          if List.mem "retreating" value.wire.undefined_fields then [ ("retreating", undefined) ]
          else []
      | Some x -> [ ("retreating", encode_fragments_rule11_remember x) ])
    @ (match value.rebuilding with
      | None ->
          if List.mem "rebuilding" value.wire.undefined_fields then [ ("rebuilding", undefined) ]
          else []
      | Some x -> [ ("rebuilding", encode_fragments_rule11_remember x) ])
    @ (match value.name with
      | None -> if List.mem "name" value.wire.undefined_fields then [ ("name", undefined) ] else []
      | Some x -> [ ("name", encode_json_string x) ])
    @ (match value.character with
      | None ->
          if List.mem "character" value.wire.undefined_fields then [ ("character", undefined) ]
          else []
      | Some x -> [ ("character", encode_json_string x) ])
    @
    match value.experience with
    | None ->
        if List.mem "experience" value.wire.undefined_fields then [ ("experience", undefined) ]
        else []
    | Some x -> [ ("experience", encode_json_number x) ])

and encode_fragments_native_state38_units_item_role
    (value : fragments_native_state38_units_item_role) : Js.Json.t =
  match value with
  | `Undefined -> undefined
  | `Leader -> Js.Json.parseExn "\"leader\""
  | `Courier -> Js.Json.parseExn "\"courier\""
  | `Captive -> Js.Json.parseExn "\"captive\""

and encode_literal_leader (value : literal_leader) : Js.Json.t = Js.Json.parseExn "\"leader\""
and encode_literal_courier (value : literal_courier) : Js.Json.t = Js.Json.parseExn "\"courier\""
and encode_literal_captive (value : literal_captive) : Js.Json.t = Js.Json.parseExn "\"captive\""

and encode_javascript1_state_map_id (value : javascript1_state_map_id) : Js.Json.t =
  match value with `Undefined -> undefined | `String x -> encode_json_string x

and encode_fragments_native_state38_units_item_items
    (value : fragments_native_state38_units_item_items) : Js.Json.t =
  match value with
  | `Undefined -> undefined
  | `Array x -> encode_fragments_native_state38_units_item_items_member1 x

and encode_fragments_native_state38_units_item_items_member1
    (value : fragments_native_state38_units_item_items_member1) : Js.Json.t =
  Js.Json.array (Array.map encode_fragments_native_state38_ground_items_item_item value)

and encode_fragments_native_state38_ground_items_item_item
    (value : fragments_native_state38_ground_items_item_item) : Js.Json.t =
  wire_to_json
    ~known:[ "armor"; "id"; "name"; "description"; "attacks"; "healing" ]
    value.wire
    ([
       ("id", encode_json_string value.id);
       ("name", encode_json_string value.name);
       ("description", encode_json_string value.description);
       ("attacks", encode_fragments_native_state38_ground_items_item_item_attacks value.attacks);
     ]
    @ (match value.armor with
      | None ->
          if List.mem "armor" value.wire.undefined_fields then [ ("armor", undefined) ] else []
      | Some x ->
          [ ("armor", encode_fragments_native_state38_ground_items_item_item_armor_member1 x) ])
    @
    match value.healing with
    | None ->
        if List.mem "healing" value.wire.undefined_fields then [ ("healing", undefined) ] else []
    | Some x ->
        [ ("healing", encode_fragments_native_state38_ground_items_item_item_healing_member1 x) ])

and encode_fragments_native_state38_ground_items_item_item_armor
    (value : fragments_native_state38_ground_items_item_item_armor) : Js.Json.t =
  match value with
  | `Undefined -> undefined
  | `Object2 x -> encode_fragments_native_state38_ground_items_item_item_armor_member1 x

and encode_fragments_native_state38_ground_items_item_item_armor_member1
    (value : fragments_native_state38_ground_items_item_item_armor_member1) : Js.Json.t =
  wire_to_json ~known:[ "blade"; "pierce"; "impact" ] value.wire
    [
      ("blade", encode_json_number value.blade);
      ("pierce", encode_json_number value.pierce);
      ("impact", encode_json_number value.impact);
    ]

and encode_fragments_native_state38_ground_items_item_item_attacks
    (value : fragments_native_state38_ground_items_item_item_attacks) : Js.Json.t =
  Js.Json.array
    (Array.map encode_fragments_native_state38_ground_items_item_item_attacks_item value)

and encode_fragments_native_state38_ground_items_item_item_attacks_item
    (value : fragments_native_state38_ground_items_item_item_attacks_item) : Js.Json.t =
  wire_to_json
    ~known:
      [
        "id";
        "name";
        "damage";
        "spread";
        "count";
        "damageType";
        "category";
        "range";
        "sound";
        "drain";
      ]
    value.wire
    ([
       ("id", encode_json_string value.id);
       ("name", encode_json_string value.name);
       ("damage", encode_json_number value.damage);
       ( "damageType",
         encode_fragments_native_state38_ground_items_item_item_attacks_item_damage_type
           value.damage_type );
       ( "category",
         encode_fragments_native_state38_ground_items_item_item_attacks_item_category value.category
       );
       ("range", encode_json_number value.range);
       ("sound", encode_json_string value.sound);
     ]
    @ (match value.spread with
      | None ->
          if List.mem "spread" value.wire.undefined_fields then [ ("spread", undefined) ] else []
      | Some x -> [ ("spread", encode_json_number x) ])
    @ (match value.count with
      | None ->
          if List.mem "count" value.wire.undefined_fields then [ ("count", undefined) ] else []
      | Some x -> [ ("count", encode_json_number x) ])
    @
    match value.drain with
    | None -> if List.mem "drain" value.wire.undefined_fields then [ ("drain", undefined) ] else []
    | Some x -> [ ("drain", encode_json_number x) ])

and encode_fragments_light23_dx (value : fragments_light23_dx) : Js.Json.t =
  match value with `Undefined -> undefined | `Number x -> encode_json_number x

and encode_fragments_native_state38_ground_items_item_item_attacks_item_damage_type
    (value : fragments_native_state38_ground_items_item_item_attacks_item_damage_type) : Js.Json.t =
  match value with
  | `Blade -> Js.Json.parseExn "\"blade\""
  | `Pierce -> Js.Json.parseExn "\"pierce\""
  | `Impact -> Js.Json.parseExn "\"impact\""

and encode_literal_blade (value : literal_blade) : Js.Json.t = Js.Json.parseExn "\"blade\""
and encode_literal_pierce (value : literal_pierce) : Js.Json.t = Js.Json.parseExn "\"pierce\""
and encode_literal_impact (value : literal_impact) : Js.Json.t = Js.Json.parseExn "\"impact\""

and encode_fragments_native_state38_ground_items_item_item_attacks_item_category
    (value : fragments_native_state38_ground_items_item_item_attacks_item_category) : Js.Json.t =
  match value with
  | `Melee -> Js.Json.parseExn "\"melee\""
  | `Ranged -> Js.Json.parseExn "\"ranged\""

and encode_literal_melee (value : literal_melee) : Js.Json.t = Js.Json.parseExn "\"melee\""
and encode_literal_ranged (value : literal_ranged) : Js.Json.t = Js.Json.parseExn "\"ranged\""

and encode_fragments_native_state38_ground_items_item_item_healing
    (value : fragments_native_state38_ground_items_item_item_healing) : Js.Json.t =
  match value with
  | `Undefined -> undefined
  | `Object2 x -> encode_fragments_native_state38_ground_items_item_item_healing_member1 x

and encode_fragments_native_state38_ground_items_item_item_healing_member1
    (value : fragments_native_state38_ground_items_item_item_healing_member1) : Js.Json.t =
  wire_to_json ~known:[ "amount"; "radius" ] value.wire
    [ ("amount", encode_json_number value.amount); ("radius", encode_json_number value.radius) ]

and encode_json_boolean (value : json_boolean) : Js.Json.t = b value

and encode_fragments_rule11_remember (value : fragments_rule11_remember) : Js.Json.t =
  match value with
  | `Undefined -> undefined
  | `False -> Js.Json.parseExn "false"
  | `True -> Js.Json.parseExn "true"

and encode_literal_false (value : literal_false) : Js.Json.t = Js.Json.parseExn "false"
and encode_literal_true (value : literal_true) : Js.Json.t = Js.Json.parseExn "true"

and encode_save2_checkpoint_state_tiles (value : save2_checkpoint_state_tiles) : Js.Json.t =
  Js.Json.array (Array.map encode_save2_checkpoint_state_tiles_item value)

and encode_save2_checkpoint_state_tiles_item (value : save2_checkpoint_state_tiles_item) : Js.Json.t
    =
  wire_to_json
    ~known:
      [
        "owner";
        "id";
        "location";
        "col";
        "row";
        "terrain";
        "connections";
        "deadEnds";
        "walls";
        "floor";
        "fog";
        "light";
        "wallCorners";
      ]
    value.wire
    ([
       ("owner", encode_fragments_tiles11_item_owner value.owner);
       ("id", encode_json_string value.id);
       ("col", encode_json_number value.col);
       ("row", encode_json_number value.row);
       ("terrain", encode_json_string value.terrain);
     ]
    @ (match value.location with
      | None ->
          if List.mem "location" value.wire.undefined_fields then [ ("location", undefined) ]
          else []
      | Some x -> [ ("location", encode_json_string x) ])
    @ (match value.connections with
      | None ->
          if List.mem "connections" value.wire.undefined_fields then [ ("connections", undefined) ]
          else []
      | Some x -> [ ("connections", encode_fragments_tiles11_item_connections_member1 x) ])
    @ (match value.dead_ends with
      | None ->
          if List.mem "deadEnds" value.wire.undefined_fields then [ ("deadEnds", undefined) ]
          else []
      | Some x -> [ ("deadEnds", encode_fragments_tiles11_item_connections_member1 x) ])
    @ (match value.walls with
      | None ->
          if List.mem "walls" value.wire.undefined_fields then [ ("walls", undefined) ] else []
      | Some x -> [ ("walls", encode_fragments_tiles11_item_connections_member1 x) ])
    @ (match value.floor with
      | None ->
          if List.mem "floor" value.wire.undefined_fields then [ ("floor", undefined) ] else []
      | Some x -> [ ("floor", encode_json_string x) ])
    @ (match value.fog with
      | None -> if List.mem "fog" value.wire.undefined_fields then [ ("fog", undefined) ] else []
      | Some x -> [ ("fog", encode_save2_checkpoint_state_tiles_item_fog_member1 x) ])
    @ (match value.light with
      | None ->
          if List.mem "light" value.wire.undefined_fields then [ ("light", undefined) ] else []
      | Some x -> [ ("light", encode_save2_checkpoint_state_tiles_item_light_member1 x) ])
    @
    match value.wall_corners with
    | None ->
        if List.mem "wallCorners" value.wire.undefined_fields then [ ("wallCorners", undefined) ]
        else []
    | Some x -> [ ("wallCorners", encode_fragments_tiles11_item_wall_corners_member1 x) ])

and encode_fragments_tiles11_item_owner (value : fragments_tiles11_item_owner) : Js.Json.t =
  match value with `Null -> json_null | `String x -> encode_json_string x

and encode_fragments_tiles11_item_connections (value : fragments_tiles11_item_connections) :
    Js.Json.t =
  match value with
  | `Undefined -> undefined
  | `Array x -> encode_fragments_tiles11_item_connections_member1 x

and encode_fragments_tiles11_item_connections_member1
    (value : fragments_tiles11_item_connections_member1) : Js.Json.t =
  Js.Json.array (Array.map encode_fragments_tiles11_item_connections_member1_item value)

and encode_fragments_tiles11_item_connections_member1_item
    (value : fragments_tiles11_item_connections_member1_item) : Js.Json.t =
  match value with
  | `E -> Js.Json.parseExn "\"e\""
  | `Se -> Js.Json.parseExn "\"se\""
  | `Sw -> Js.Json.parseExn "\"sw\""
  | `W -> Js.Json.parseExn "\"w\""
  | `Nw -> Js.Json.parseExn "\"nw\""
  | `Ne -> Js.Json.parseExn "\"ne\""

and encode_literal_e (value : literal_e) : Js.Json.t = Js.Json.parseExn "\"e\""
and encode_literal_se (value : literal_se) : Js.Json.t = Js.Json.parseExn "\"se\""
and encode_literal_sw (value : literal_sw) : Js.Json.t = Js.Json.parseExn "\"sw\""
and encode_literal_w (value : literal_w) : Js.Json.t = Js.Json.parseExn "\"w\""
and encode_literal_nw (value : literal_nw) : Js.Json.t = Js.Json.parseExn "\"nw\""
and encode_literal_ne (value : literal_ne) : Js.Json.t = Js.Json.parseExn "\"ne\""

and encode_save2_checkpoint_state_tiles_item_fog (value : save2_checkpoint_state_tiles_item_fog) :
    Js.Json.t =
  match value with
  | `Undefined -> undefined
  | `Object2 x -> encode_save2_checkpoint_state_tiles_item_fog_member1 x

and encode_save2_checkpoint_state_tiles_item_fog_member1
    (value : save2_checkpoint_state_tiles_item_fog_member1) : Js.Json.t =
  wire_to_json ~known:[ "condition"; "remember" ] value.wire
    ([
       ("condition", encode_save2_checkpoint_state_tiles_item_fog_member1_condition value.condition);
     ]
    @
    match value.remember with
    | None ->
        if List.mem "remember" value.wire.undefined_fields then [ ("remember", undefined) ] else []
    | Some x -> [ ("remember", encode_fragments_rule11_remember x) ])

and encode_save2_checkpoint_state_tiles_item_fog_member1_condition
    (value : save2_checkpoint_state_tiles_item_fog_member1_condition) : Js.Json.t =
  match value with
  | `Owner x -> encode_fragments_rule11_condition_member0 x
  | `Visible_units x -> encode_fragments_rule11_condition_member1 x
  | `Choice x -> encode_fragments_rule11_condition_member2 x
  | `Flag x -> encode_fragments_rule11_condition_member3 x
  | `Event x -> encode_fragments_rule11_condition_member4 x
  | `Round x -> encode_fragments_rule11_condition_member5 x
  | `Object7 x -> encode_save2_checkpoint_state_tiles_item_fog_member1_condition_member6 x
  | `Proximity x -> encode_fragments_rule11_condition_member6 x
  | `Has_units x -> encode_fragments_rule11_condition_member7 x
  | `No_units x -> encode_fragments_rule11_condition_member8 x

and encode_fragments_rule11_condition_member0 (value : fragments_rule11_condition_member0) :
    Js.Json.t =
  wire_to_json ~known:[ "kind"; "tile"; "side" ] value.wire
    ([ ("kind", encode_literal_owner value.kind); ("side", encode_json_string value.side) ]
    @
    match value.tile with
    | None -> if List.mem "tile" value.wire.undefined_fields then [ ("tile", undefined) ] else []
    | Some x -> [ ("tile", encode_json_string x) ])

and encode_literal_owner (value : literal_owner) : Js.Json.t = Js.Json.parseExn "\"owner\""

and encode_fragments_rule11_condition_member1 (value : fragments_rule11_condition_member1) :
    Js.Json.t =
  wire_to_json ~known:[ "kind"; "side"; "unitTypes" ] value.wire
    ([
       ("kind", encode_literal_visible_units value.kind);
       ("side", encode_fragments_rule11_condition_member1_side value.side);
     ]
    @
    match value.unit_types with
    | None ->
        if List.mem "unitTypes" value.wire.undefined_fields then [ ("unitTypes", undefined) ]
        else []
    | Some x -> [ ("unitTypes", encode_fragments_native_state38_log x) ])

and encode_literal_visible_units (value : literal_visible_units) : Js.Json.t =
  Js.Json.parseExn "\"visibleUnits\""

and encode_fragments_rule11_condition_member1_side (value : fragments_rule11_condition_member1_side)
    : Js.Json.t =
  match value with
  | `Player -> Js.Json.parseExn "\"player\""
  | `Enemy -> Js.Json.parseExn "\"enemy\""
  | `Neutral -> Js.Json.parseExn "\"neutral\""
  | `Any -> Js.Json.parseExn "\"any\""

and encode_literal_neutral (value : literal_neutral) : Js.Json.t = Js.Json.parseExn "\"neutral\""
and encode_literal_any (value : literal_any) : Js.Json.t = Js.Json.parseExn "\"any\""

and encode_fragments_rule11_condition_member2 (value : fragments_rule11_condition_member2) :
    Js.Json.t =
  wire_to_json ~known:[ "kind"; "choice" ] value.wire
    [ ("kind", encode_literal_choice value.kind); ("choice", encode_json_string value.choice) ]

and encode_literal_choice (value : literal_choice) : Js.Json.t = Js.Json.parseExn "\"choice\""

and encode_fragments_rule11_condition_member3 (value : fragments_rule11_condition_member3) :
    Js.Json.t =
  wire_to_json ~known:[ "kind"; "flag" ] value.wire
    [ ("kind", encode_literal_flag value.kind); ("flag", encode_json_string value.flag) ]

and encode_literal_flag (value : literal_flag) : Js.Json.t = Js.Json.parseExn "\"flag\""

and encode_fragments_rule11_condition_member4 (value : fragments_rule11_condition_member4) :
    Js.Json.t =
  wire_to_json ~known:[ "kind"; "event" ] value.wire
    [ ("kind", encode_literal_event value.kind); ("event", encode_json_string value.event) ]

and encode_literal_event (value : literal_event) : Js.Json.t = Js.Json.parseExn "\"event\""

and encode_fragments_rule11_condition_member5 (value : fragments_rule11_condition_member5) :
    Js.Json.t =
  wire_to_json ~known:[ "kind"; "round" ] value.wire
    [ ("kind", encode_literal_round value.kind); ("round", encode_json_number value.round) ]

and encode_literal_round (value : literal_round) : Js.Json.t = Js.Json.parseExn "\"round\""

and encode_save2_checkpoint_state_tiles_item_fog_member1_condition_member6
    (value : save2_checkpoint_state_tiles_item_fog_member1_condition_member6) : Js.Json.t =
  wire_to_json ~known:[ "kind"; "conditions" ] value.wire
    [
      ("kind", encode_fragments_rule11_condition_member9_kind value.kind);
      ( "conditions",
        encode_save2_checkpoint_state_tiles_item_fog_member1_condition_member6_conditions
          value.conditions );
    ]

and encode_fragments_rule11_condition_member9_kind (value : fragments_rule11_condition_member9_kind)
    : Js.Json.t =
  match value with `Any -> Js.Json.parseExn "\"any\"" | `All -> Js.Json.parseExn "\"all\""

and encode_literal_all (value : literal_all) : Js.Json.t = Js.Json.parseExn "\"all\""

and encode_save2_checkpoint_state_tiles_item_fog_member1_condition_member6_conditions
    (value : save2_checkpoint_state_tiles_item_fog_member1_condition_member6_conditions) : Js.Json.t
    =
  Js.Json.array (Array.map encode_save2_checkpoint_state_tiles_item_fog_member1_condition value)

and encode_fragments_rule11_condition_member6 (value : fragments_rule11_condition_member6) :
    Js.Json.t =
  wire_to_json
    ~known:[ "kind"; "unitTypes"; "tile"; "radius"; "side"; "unitType"; "character"; "name" ]
    value.wire
    ([
       ("kind", encode_literal_proximity value.kind);
       ("radius", encode_json_number value.radius);
       ("side", encode_fragments_rule11_condition_member1_side value.side);
     ]
    @ (match value.unit_types with
      | None ->
          if List.mem "unitTypes" value.wire.undefined_fields then [ ("unitTypes", undefined) ]
          else []
      | Some x -> [ ("unitTypes", encode_fragments_native_state38_log x) ])
    @ (match value.tile with
      | None -> if List.mem "tile" value.wire.undefined_fields then [ ("tile", undefined) ] else []
      | Some x -> [ ("tile", encode_json_string x) ])
    @ (match value.unit_type with
      | None ->
          if List.mem "unitType" value.wire.undefined_fields then [ ("unitType", undefined) ]
          else []
      | Some x -> [ ("unitType", encode_json_string x) ])
    @ (match value.character with
      | None ->
          if List.mem "character" value.wire.undefined_fields then [ ("character", undefined) ]
          else []
      | Some x -> [ ("character", encode_json_string x) ])
    @
    match value.name with
    | None -> if List.mem "name" value.wire.undefined_fields then [ ("name", undefined) ] else []
    | Some x -> [ ("name", encode_json_string x) ])

and encode_literal_proximity (value : literal_proximity) : Js.Json.t =
  Js.Json.parseExn "\"proximity\""

and encode_fragments_rule11_condition_member7 (value : fragments_rule11_condition_member7) :
    Js.Json.t =
  wire_to_json
    ~known:[ "kind"; "aiGroup"; "side"; "unitType"; "character"; "name" ]
    value.wire
    ([
       ("kind", encode_literal_has_units value.kind);
       ("side", encode_fragments_rule11_condition_member1_side value.side);
     ]
    @ (match value.ai_group with
      | None ->
          if List.mem "aiGroup" value.wire.undefined_fields then [ ("aiGroup", undefined) ] else []
      | Some x -> [ ("aiGroup", encode_json_string x) ])
    @ (match value.unit_type with
      | None ->
          if List.mem "unitType" value.wire.undefined_fields then [ ("unitType", undefined) ]
          else []
      | Some x -> [ ("unitType", encode_json_string x) ])
    @ (match value.character with
      | None ->
          if List.mem "character" value.wire.undefined_fields then [ ("character", undefined) ]
          else []
      | Some x -> [ ("character", encode_json_string x) ])
    @
    match value.name with
    | None -> if List.mem "name" value.wire.undefined_fields then [ ("name", undefined) ] else []
    | Some x -> [ ("name", encode_json_string x) ])

and encode_literal_has_units (value : literal_has_units) : Js.Json.t =
  Js.Json.parseExn "\"hasUnits\""

and encode_fragments_rule11_condition_member8 (value : fragments_rule11_condition_member8) :
    Js.Json.t =
  wire_to_json
    ~known:[ "kind"; "side"; "unitType"; "character"; "name" ]
    value.wire
    ([
       ("kind", encode_literal_no_units value.kind);
       ("side", encode_fragments_rule11_condition_member1_side value.side);
     ]
    @ (match value.unit_type with
      | None ->
          if List.mem "unitType" value.wire.undefined_fields then [ ("unitType", undefined) ]
          else []
      | Some x -> [ ("unitType", encode_json_string x) ])
    @ (match value.character with
      | None ->
          if List.mem "character" value.wire.undefined_fields then [ ("character", undefined) ]
          else []
      | Some x -> [ ("character", encode_json_string x) ])
    @
    match value.name with
    | None -> if List.mem "name" value.wire.undefined_fields then [ ("name", undefined) ] else []
    | Some x -> [ ("name", encode_json_string x) ])

and encode_literal_no_units (value : literal_no_units) : Js.Json.t = Js.Json.parseExn "\"noUnits\""

and encode_save2_checkpoint_state_tiles_item_light (value : save2_checkpoint_state_tiles_item_light)
    : Js.Json.t =
  match value with
  | `Undefined -> undefined
  | `Object2 x -> encode_save2_checkpoint_state_tiles_item_light_member1 x

and encode_save2_checkpoint_state_tiles_item_light_member1
    (value : save2_checkpoint_state_tiles_item_light_member1) : Js.Json.t =
  wire_to_json
    ~known:[ "radius"; "color"; "fixture"; "dx"; "dy"; "power"; "when" ]
    value.wire
    ([ ("radius", encode_json_number value.radius); ("power", encode_json_number value.power) ]
    @ (match value.color with
      | None ->
          if List.mem "color" value.wire.undefined_fields then [ ("color", undefined) ] else []
      | Some x -> [ ("color", encode_fragments_light23_color_member1 x) ])
    @ (match value.fixture with
      | None ->
          if List.mem "fixture" value.wire.undefined_fields then [ ("fixture", undefined) ] else []
      | Some x -> [ ("fixture", encode_fragments_rule11_remember x) ])
    @ (match value.dx with
      | None -> if List.mem "dx" value.wire.undefined_fields then [ ("dx", undefined) ] else []
      | Some x -> [ ("dx", encode_json_number x) ])
    @ (match value.dy with
      | None -> if List.mem "dy" value.wire.undefined_fields then [ ("dy", undefined) ] else []
      | Some x -> [ ("dy", encode_json_number x) ])
    @
    match value.when_ with
    | None -> if List.mem "when" value.wire.undefined_fields then [ ("when", undefined) ] else []
    | Some x -> [ ("when", encode_save2_checkpoint_state_tiles_item_fog_member1 x) ])

and encode_fragments_light23_color (value : fragments_light23_color) : Js.Json.t =
  match value with `Undefined -> undefined | `Tuple x -> encode_fragments_light23_color_member1 x

and encode_fragments_light23_color_member1 (value : fragments_light23_color_member1) : Js.Json.t =
  let v0, v1, v2 = value in
  a [ encode_json_number v0; encode_json_number v1; encode_json_number v2 ]

and encode_fragments_tiles11_item_wall_corners (value : fragments_tiles11_item_wall_corners) :
    Js.Json.t =
  match value with
  | `Undefined -> undefined
  | `Array x -> encode_fragments_tiles11_item_wall_corners_member1 x

and encode_fragments_tiles11_item_wall_corners_member1
    (value : fragments_tiles11_item_wall_corners_member1) : Js.Json.t =
  Js.Json.array (Array.map encode_json_number value)

and encode_save2_checkpoint_state_campaign (value : save2_checkpoint_state_campaign) : Js.Json.t =
  match value with
  | `Undefined -> undefined
  | `Object2 x -> encode_save2_checkpoint_state_campaign_member1 x

and encode_save2_checkpoint_state_campaign_member1 (value : save2_checkpoint_state_campaign_member1)
    : Js.Json.t =
  wire_to_json
    ~known:
      [
        "mission";
        "id";
        "vigil";
        "refuge";
        "completed";
        "marks";
        "holds";
        "recall";
        "fallenCompanions";
        "carryGold";
        "dialogue";
        "dialogueIndex";
        "outcome";
        "veterans";
        "storyVeterans";
        "evacuated";
        "choices";
      ]
    value.wire
    ([
       ("mission", encode_json_number value.mission);
       ("id", encode_json_string value.id);
       ("completed", encode_json_boolean value.completed);
       ("marks", encode_fragments_native_state38_log value.marks);
       ("recall", encode_fragments_native_state38_log value.recall);
       ("carryGold", encode_json_number value.carry_gold);
       ("dialogue", encode_fragments_native_state38_campaign_member1_dialogue value.dialogue);
       ("dialogueIndex", encode_json_number value.dialogue_index);
       ("outcome", encode_fragments_tiles11_item_owner value.outcome);
     ]
    @ (match value.vigil with
      | None ->
          if List.mem "vigil" value.wire.undefined_fields then [ ("vigil", undefined) ] else []
      | Some x -> [ ("vigil", encode_fragments_native_state38_campaign_member1_vigil_member1 x) ])
    @ (match value.refuge with
      | None ->
          if List.mem "refuge" value.wire.undefined_fields then [ ("refuge", undefined) ] else []
      | Some x -> [ ("refuge", encode_fragments_native_state38_campaign_member1_refuge_member1 x) ])
    @ (match value.holds with
      | None ->
          if List.mem "holds" value.wire.undefined_fields then [ ("holds", undefined) ] else []
      | Some x -> [ ("holds", encode_fragments_native_state38_campaign_member1_holds_member1 x) ])
    @ (match value.fallen_companions with
      | None ->
          if List.mem "fallenCompanions" value.wire.undefined_fields then
            [ ("fallenCompanions", undefined) ]
          else []
      | Some x -> [ ("fallenCompanions", encode_fragments_native_state38_log x) ])
    @ (match value.veterans with
      | None ->
          if List.mem "veterans" value.wire.undefined_fields then [ ("veterans", undefined) ]
          else []
      | Some x ->
          [ ("veterans", encode_save2_checkpoint_state_campaign_member1_veterans_member1 x) ])
    @ (match value.story_veterans with
      | None ->
          if List.mem "storyVeterans" value.wire.undefined_fields then
            [ ("storyVeterans", undefined) ]
          else []
      | Some x ->
          [ ("storyVeterans", encode_save2_checkpoint_state_campaign_member1_veterans_member1 x) ])
    @ (match value.evacuated with
      | None ->
          if List.mem "evacuated" value.wire.undefined_fields then [ ("evacuated", undefined) ]
          else []
      | Some x ->
          [ ("evacuated", encode_save2_checkpoint_state_campaign_member1_evacuated_member1 x) ])
    @
    match value.choices with
    | None ->
        if List.mem "choices" value.wire.undefined_fields then [ ("choices", undefined) ] else []
    | Some x -> [ ("choices", encode_fragments_native_state38_log x) ])

and encode_fragments_native_state38_campaign_member1_vigil
    (value : fragments_native_state38_campaign_member1_vigil) : Js.Json.t =
  match value with
  | `Undefined -> undefined
  | `Object2 x -> encode_fragments_native_state38_campaign_member1_vigil_member1 x

and encode_fragments_native_state38_campaign_member1_vigil_member1
    (value : fragments_native_state38_campaign_member1_vigil_member1) : Js.Json.t =
  wire_to_json
    ~known:[ "heldTurns"; "lastCountedRound" ]
    value.wire
    [
      ("heldTurns", encode_json_number value.held_turns);
      ("lastCountedRound", encode_json_number value.last_counted_round);
    ]

and encode_fragments_native_state38_campaign_member1_refuge
    (value : fragments_native_state38_campaign_member1_refuge) : Js.Json.t =
  match value with
  | `Undefined -> undefined
  | `Object2 x -> encode_fragments_native_state38_campaign_member1_refuge_member1 x

and encode_fragments_native_state38_campaign_member1_refuge_member1
    (value : fragments_native_state38_campaign_member1_refuge_member1) : Js.Json.t =
  wire_to_json
    ~known:[ "heldTurns"; "lastCountedRound"; "escalated" ]
    value.wire
    [
      ("heldTurns", encode_json_number value.held_turns);
      ("lastCountedRound", encode_json_number value.last_counted_round);
      ("escalated", encode_json_boolean value.escalated);
    ]

and encode_fragments_native_state38_campaign_member1_holds
    (value : fragments_native_state38_campaign_member1_holds) : Js.Json.t =
  match value with
  | `Undefined -> undefined
  | `Object2 x -> encode_fragments_native_state38_campaign_member1_holds_member1 x

and encode_fragments_native_state38_campaign_member1_holds_member1
    (value : fragments_native_state38_campaign_member1_holds_member1) : Js.Json.t =
  map_to_json encode_fragments_native_state38_campaign_member1_vigil_member1 value

and encode_fragments_native_state38_campaign_member1_dialogue
    (value : fragments_native_state38_campaign_member1_dialogue) : Js.Json.t =
  Js.Json.array (Array.map encode_fragments_native_state38_campaign_member1_dialogue_item value)

and encode_fragments_native_state38_campaign_member1_dialogue_item
    (value : fragments_native_state38_campaign_member1_dialogue_item) : Js.Json.t =
  wire_to_json
    ~known:[ "sound"; "speaker"; "character"; "portrait"; "text" ]
    value.wire
    ([ ("portrait", encode_json_string value.portrait); ("text", encode_json_string value.text) ]
    @ (match value.sound with
      | None ->
          if List.mem "sound" value.wire.undefined_fields then [ ("sound", undefined) ] else []
      | Some x -> [ ("sound", encode_json_string x) ])
    @ (match value.speaker with
      | None ->
          if List.mem "speaker" value.wire.undefined_fields then [ ("speaker", undefined) ] else []
      | Some x -> [ ("speaker", encode_json_string x) ])
    @
    match value.character with
    | None ->
        if List.mem "character" value.wire.undefined_fields then [ ("character", undefined) ]
        else []
    | Some x -> [ ("character", encode_json_string x) ])

and encode_save2_checkpoint_state_campaign_member1_veterans
    (value : save2_checkpoint_state_campaign_member1_veterans) : Js.Json.t =
  match value with
  | `Undefined -> undefined
  | `Array x -> encode_save2_checkpoint_state_campaign_member1_veterans_member1 x

and encode_save2_checkpoint_state_campaign_member1_veterans_member1
    (value : save2_checkpoint_state_campaign_member1_veterans_member1) : Js.Json.t =
  Js.Json.array
    (Array.map encode_save2_checkpoint_state_campaign_member1_veterans_member1_item value)

and encode_save2_checkpoint_state_campaign_member1_veterans_member1_item
    (value : save2_checkpoint_state_campaign_member1_veterans_member1_item) : Js.Json.t =
  wire_to_json
    ~known:[ "role"; "experience"; "items"; "type"; "name"; "character" ]
    value.wire
    ([
       ("experience", encode_json_number value.experience); ("type", encode_json_string value.type_);
     ]
    @ (match value.role with
      | None -> if List.mem "role" value.wire.undefined_fields then [ ("role", undefined) ] else []
      | Some x -> [ ("role", encode_fragments_native_state38_units_item_role x) ])
    @ (match value.items with
      | None ->
          if List.mem "items" value.wire.undefined_fields then [ ("items", undefined) ] else []
      | Some x -> [ ("items", encode_fragments_native_state38_units_item_items_member1 x) ])
    @ (match value.name with
      | None -> if List.mem "name" value.wire.undefined_fields then [ ("name", undefined) ] else []
      | Some x -> [ ("name", encode_json_string x) ])
    @
    match value.character with
    | None ->
        if List.mem "character" value.wire.undefined_fields then [ ("character", undefined) ]
        else []
    | Some x -> [ ("character", encode_json_string x) ])

and encode_save2_checkpoint_state_campaign_member1_evacuated
    (value : save2_checkpoint_state_campaign_member1_evacuated) : Js.Json.t =
  match value with
  | `Undefined -> undefined
  | `Array x -> encode_save2_checkpoint_state_campaign_member1_evacuated_member1 x

and encode_save2_checkpoint_state_campaign_member1_evacuated_member1
    (value : save2_checkpoint_state_campaign_member1_evacuated_member1) : Js.Json.t =
  Js.Json.array
    (Array.map encode_save2_checkpoint_state_campaign_member1_evacuated_member1_item value)

and encode_save2_checkpoint_state_campaign_member1_evacuated_member1_item
    (value : save2_checkpoint_state_campaign_member1_evacuated_member1_item) : Js.Json.t =
  wire_to_json ~known:[ "event"; "unit" ] value.wire
    [
      ("event", encode_json_string value.event);
      ("unit", encode_save2_checkpoint_state_units_item value.unit_);
    ]

and encode_javascript1_state_gold (value : javascript1_state_gold) : Js.Json.t =
  wire_to_json ~known:[ "player"; "enemy" ] value.wire
    [ ("player", encode_json_number value.player); ("enemy", encode_json_number value.enemy) ]

and encode_save2_checkpoint_state_factions (value : save2_checkpoint_state_factions) : Js.Json.t =
  match value with
  | `Undefined -> undefined
  | `Array x -> encode_save2_checkpoint_state_factions_member1 x

and encode_save2_checkpoint_state_factions_member1 (value : save2_checkpoint_state_factions_member1)
    : Js.Json.t =
  Js.Json.array (Array.map encode_save2_checkpoint_state_factions_member1_item value)

and encode_save2_checkpoint_state_factions_member1_item
    (value : save2_checkpoint_state_factions_member1_item) : Js.Json.t =
  wire_to_json
    ~known:[ "id"; "name"; "phase"; "controller"; "palette"; "recruits"; "relations"; "transfer" ]
    value.wire
    ([
       ("id", encode_json_string value.id);
       ("phase", encode_save35_setup_first value.phase);
       ("controller", encode_fragments_factions7_item_controller value.controller);
       ("palette", encode_json_string value.palette);
     ]
    @ (match value.name with
      | None -> if List.mem "name" value.wire.undefined_fields then [ ("name", undefined) ] else []
      | Some x -> [ ("name", encode_json_string x) ])
    @ (match value.recruits with
      | None ->
          if List.mem "recruits" value.wire.undefined_fields then [ ("recruits", undefined) ]
          else []
      | Some x -> [ ("recruits", encode_fragments_factions7_item_recruits_member1 x) ])
    @ (match value.relations with
      | None ->
          if List.mem "relations" value.wire.undefined_fields then [ ("relations", undefined) ]
          else []
      | Some x -> [ ("relations", encode_fragments_factions7_item_relations_member1 x) ])
    @
    match value.transfer with
    | None ->
        if List.mem "transfer" value.wire.undefined_fields then [ ("transfer", undefined) ] else []
    | Some x ->
        [ ("transfer", encode_save2_checkpoint_state_factions_member1_item_transfer_member1 x) ])

and encode_fragments_factions7_item_controller (value : fragments_factions7_item_controller) :
    Js.Json.t =
  match value with
  | `Human x -> encode_fragments_factions7_item_controller_member0 x
  | `Passive x -> encode_fragments_factions7_item_controller_member1 x
  | `Ai x -> encode_fragments_factions7_item_controller_member2 x

and encode_fragments_factions7_item_controller_member0
    (value : fragments_factions7_item_controller_member0) : Js.Json.t =
  wire_to_json ~known:[ "kind" ] value.wire [ ("kind", encode_literal_human value.kind) ]

and encode_literal_human (value : literal_human) : Js.Json.t = Js.Json.parseExn "\"human\""

and encode_fragments_factions7_item_controller_member1
    (value : fragments_factions7_item_controller_member1) : Js.Json.t =
  wire_to_json ~known:[ "kind" ] value.wire [ ("kind", encode_literal_passive value.kind) ]

and encode_literal_passive (value : literal_passive) : Js.Json.t = Js.Json.parseExn "\"passive\""

and encode_fragments_factions7_item_controller_member2
    (value : fragments_factions7_item_controller_member2) : Js.Json.t =
  wire_to_json ~known:[ "kind"; "policy" ] value.wire
    ([ ("kind", encode_literal_ai value.kind) ]
    @
    match value.policy with
    | None ->
        if List.mem "policy" value.wire.undefined_fields then [ ("policy", undefined) ] else []
    | Some x -> [ ("policy", encode_fragments_factions7_item_controller_member2_policy x) ])

and encode_literal_ai (value : literal_ai) : Js.Json.t = Js.Json.parseExn "\"ai\""

and encode_fragments_factions7_item_controller_member2_policy
    (value : fragments_factions7_item_controller_member2_policy) : Js.Json.t =
  match value with
  | `Undefined -> undefined
  | `Classic -> Js.Json.parseExn "\"classic\""
  | `Rush -> Js.Json.parseExn "\"rush\""
  | `Balanced -> Js.Json.parseExn "\"balanced\""
  | `Rotated -> Js.Json.parseExn "\"rotated\""
  | `Situational -> Js.Json.parseExn "\"situational\""
  | `Predictor -> Js.Json.parseExn "\"predictor\""

and encode_literal_classic (value : literal_classic) : Js.Json.t = Js.Json.parseExn "\"classic\""
and encode_literal_rush (value : literal_rush) : Js.Json.t = Js.Json.parseExn "\"rush\""
and encode_literal_balanced (value : literal_balanced) : Js.Json.t = Js.Json.parseExn "\"balanced\""
and encode_literal_rotated (value : literal_rotated) : Js.Json.t = Js.Json.parseExn "\"rotated\""

and encode_literal_situational (value : literal_situational) : Js.Json.t =
  Js.Json.parseExn "\"situational\""

and encode_literal_predictor (value : literal_predictor) : Js.Json.t =
  Js.Json.parseExn "\"predictor\""

and encode_fragments_factions7_item_recruits (value : fragments_factions7_item_recruits) : Js.Json.t
    =
  match value with
  | `Undefined -> undefined
  | `Array x -> encode_fragments_factions7_item_recruits_member1 x

and encode_fragments_factions7_item_recruits_member1
    (value : fragments_factions7_item_recruits_member1) : Js.Json.t =
  Js.Json.array (Array.map encode_fragments_factions7_item_recruits_member1_item value)

and encode_fragments_factions7_item_recruits_member1_item
    (value : fragments_factions7_item_recruits_member1_item) : Js.Json.t =
  wire_to_json ~known:[ "unit"; "cost" ] value.wire
    ([ ("unit", encode_json_string value.unit_) ]
    @
    match value.cost with
    | None -> if List.mem "cost" value.wire.undefined_fields then [ ("cost", undefined) ] else []
    | Some x -> [ ("cost", encode_json_number x) ])

and encode_fragments_factions7_item_relations (value : fragments_factions7_item_relations) :
    Js.Json.t =
  match value with
  | `Undefined -> undefined
  | `Object2 x -> encode_fragments_factions7_item_relations_member1 x

and encode_fragments_factions7_item_relations_member1
    (value : fragments_factions7_item_relations_member1) : Js.Json.t =
  map_to_json encode_fragments_factions7_item_relations_member1_value value

and encode_fragments_factions7_item_relations_member1_value
    (value : fragments_factions7_item_relations_member1_value) : Js.Json.t =
  match value with
  | `Neutral -> Js.Json.parseExn "\"neutral\""
  | `Allied -> Js.Json.parseExn "\"allied\""
  | `Hostile -> Js.Json.parseExn "\"hostile\""

and encode_literal_allied (value : literal_allied) : Js.Json.t = Js.Json.parseExn "\"allied\""
and encode_literal_hostile (value : literal_hostile) : Js.Json.t = Js.Json.parseExn "\"hostile\""

and encode_save2_checkpoint_state_factions_member1_item_transfer
    (value : save2_checkpoint_state_factions_member1_item_transfer) : Js.Json.t =
  match value with
  | `Undefined -> undefined
  | `Object2 x -> encode_save2_checkpoint_state_factions_member1_item_transfer_member1 x

and encode_save2_checkpoint_state_factions_member1_item_transfer_member1
    (value : save2_checkpoint_state_factions_member1_item_transfer_member1) : Js.Json.t =
  wire_to_json ~known:[ "to"; "when" ] value.wire
    [
      ("to", encode_json_string value.to_);
      ("when", encode_save2_checkpoint_state_tiles_item_fog_member1 value.when_);
    ]

and encode_save2_checkpoint_state_ground_items (value : save2_checkpoint_state_ground_items) :
    Js.Json.t =
  match value with
  | `Undefined -> undefined
  | `Array x -> encode_fragments_native_state38_ground_items x

and encode_fragments_native_state38_ground_items (value : fragments_native_state38_ground_items) :
    Js.Json.t =
  Js.Json.array (Array.map encode_fragments_native_state38_ground_items_item value)

and encode_fragments_native_state38_ground_items_item
    (value : fragments_native_state38_ground_items_item) : Js.Json.t =
  wire_to_json ~known:[ "tile"; "item" ] value.wire
    [
      ("tile", encode_json_string value.tile);
      ("item", encode_fragments_native_state38_ground_items_item_item value.item);
    ]

and encode_json_unknown (value : json_unknown) : Js.Json.t = value

and encode_save2_setup (value : save2_setup) : Js.Json.t =
  match value with `Undefined -> undefined | `Object2 x -> encode_save2_setup_member1 x

and encode_save2_setup_member1 (value : save2_setup_member1) : Js.Json.t =
  wire_to_json ~known:[ "first"; "enemy" ] value.wire
    [
      ("first", encode_save35_setup_first value.first);
      ("enemy", encode_save2_setup_member1_enemy value.enemy);
    ]

and encode_save2_setup_member1_enemy (value : save2_setup_member1_enemy) : Js.Json.t =
  wire_to_json
    ~known:[ "policy"; "blocks"; "recruitmentLimit"; "diplomacy" ]
    value.wire
    ([
       ("policy", encode_save35_setup_enemy_policy value.policy);
       ("diplomacy", encode_save2_setup_member1_enemy_diplomacy value.diplomacy);
     ]
    @ (match value.blocks with
      | None ->
          if List.mem "blocks" value.wire.undefined_fields then [ ("blocks", undefined) ] else []
      | Some x -> [ ("blocks", encode_save35_setup_enemy_blocks_member1 x) ])
    @
    match value.recruitment_limit with
    | None ->
        if List.mem "recruitmentLimit" value.wire.undefined_fields then
          [ ("recruitmentLimit", undefined) ]
        else []
    | Some x -> [ ("recruitmentLimit", encode_json_number x) ])

and encode_save35_setup_enemy_policy (value : save35_setup_enemy_policy) : Js.Json.t =
  match value with
  | `Classic -> Js.Json.parseExn "\"classic\""
  | `Rush -> Js.Json.parseExn "\"rush\""
  | `Balanced -> Js.Json.parseExn "\"balanced\""
  | `Rotated -> Js.Json.parseExn "\"rotated\""
  | `Situational -> Js.Json.parseExn "\"situational\""
  | `Predictor -> Js.Json.parseExn "\"predictor\""

and encode_save35_setup_enemy_blocks (value : save35_setup_enemy_blocks) : Js.Json.t =
  match value with
  | `Undefined -> undefined
  | `Object2 x -> encode_save35_setup_enemy_blocks_member1 x

and encode_save35_setup_enemy_blocks_member1 (value : save35_setup_enemy_blocks_member1) : Js.Json.t
    =
  wire_to_json
    ~known:[ "recruiter"; "campaign"; "planner" ]
    value.wire
    [
      ("recruiter", encode_save35_setup_enemy_blocks_member1_recruiter value.recruiter);
      ("campaign", encode_save35_setup_enemy_blocks_member1_campaign value.campaign);
      ("planner", encode_save35_setup_enemy_blocks_member1_planner value.planner);
    ]

and encode_save35_setup_enemy_blocks_member1_recruiter
    (value : save35_setup_enemy_blocks_member1_recruiter) : Js.Json.t =
  match value with
  | `Classic -> Js.Json.parseExn "\"classic\""
  | `Balanced -> Js.Json.parseExn "\"balanced\""
  | `Situational -> Js.Json.parseExn "\"situational\""
  | `Predictor -> Js.Json.parseExn "\"predictor\""
  | `Rotation -> Js.Json.parseExn "\"rotation\""
  | `Frontline -> Js.Json.parseExn "\"frontline\""
  | `Aggressive_predictor -> Js.Json.parseExn "\"aggressive-predictor\""

and encode_literal_rotation (value : literal_rotation) : Js.Json.t = Js.Json.parseExn "\"rotation\""

and encode_literal_frontline (value : literal_frontline) : Js.Json.t =
  Js.Json.parseExn "\"frontline\""

and encode_literal_aggressive_predictor (value : literal_aggressive_predictor) : Js.Json.t =
  Js.Json.parseExn "\"aggressive-predictor\""

and encode_save35_setup_enemy_blocks_member1_campaign
    (value : save35_setup_enemy_blocks_member1_campaign) : Js.Json.t =
  match value with
  | `Arena -> Js.Json.parseExn "\"arena\""
  | `Mission -> Js.Json.parseExn "\"mission\""

and encode_literal_arena (value : literal_arena) : Js.Json.t = Js.Json.parseExn "\"arena\""
and encode_literal_mission (value : literal_mission) : Js.Json.t = Js.Json.parseExn "\"mission\""

and encode_save35_setup_enemy_blocks_member1_planner
    (value : save35_setup_enemy_blocks_member1_planner) : Js.Json.t =
  match value with
  | `Classic -> Js.Json.parseExn "\"classic\""
  | `Full -> Js.Json.parseExn "\"full\""
  | `Queued -> Js.Json.parseExn "\"queued\""

and encode_literal_full (value : literal_full) : Js.Json.t = Js.Json.parseExn "\"full\""
and encode_literal_queued (value : literal_queued) : Js.Json.t = Js.Json.parseExn "\"queued\""

and encode_save2_setup_member1_enemy_diplomacy (value : save2_setup_member1_enemy_diplomacy) :
    Js.Json.t =
  match value with
  | `Diplomatic -> Js.Json.parseExn "\"diplomatic\""
  | `Unrestricted -> Js.Json.parseExn "\"unrestricted\""
  | `Pragmatic -> Js.Json.parseExn "\"pragmatic\""
  | `Restrained -> Js.Json.parseExn "\"restrained\""

and encode_literal_diplomatic (value : literal_diplomatic) : Js.Json.t =
  Js.Json.parseExn "\"diplomatic\""

and encode_literal_unrestricted (value : literal_unrestricted) : Js.Json.t =
  Js.Json.parseExn "\"unrestricted\""

and encode_literal_pragmatic (value : literal_pragmatic) : Js.Json.t =
  Js.Json.parseExn "\"pragmatic\""

and encode_literal_restrained (value : literal_restrained) : Js.Json.t =
  Js.Json.parseExn "\"restrained\""

and encode_save2_progress (value : save2_progress) : Js.Json.t =
  match value with `Undefined -> undefined | `Object2 x -> encode_save2_progress_member1 x

and encode_save2_progress_member1 (value : save2_progress_member1) : Js.Json.t =
  wire_to_json ~known:[ "entry" ] value.wire [ ("entry", encode_history1_entries_item value.entry) ]

and encode_history1_entries_item (value : history1_entries_item) : Js.Json.t =
  wire_to_json
    ~known:
      [
        "id";
        "savedAt";
        "run";
        "mission";
        "round";
        "side";
        "missionStart";
        "runId";
        "branchId";
        "checkpointId";
        "ordinal";
        "kind";
        "parentId";
      ]
    value.wire
    ([
       ("id", encode_json_string value.id);
       ("savedAt", encode_json_number value.saved_at);
       ("run", encode_json_string value.run);
       ("mission", encode_json_string value.mission);
       ("round", encode_json_number value.round);
       ("side", encode_json_string value.side);
       ("missionStart", encode_json_boolean value.mission_start);
       ("runId", encode_json_string value.run_id);
       ("branchId", encode_json_string value.branch_id);
       ("checkpointId", encode_json_string value.checkpoint_id);
       ("ordinal", encode_json_number value.ordinal);
       ("kind", encode_history1_entries_item_kind value.kind);
     ]
    @
    match value.parent_id with
    | None ->
        if List.mem "parentId" value.wire.undefined_fields then [ ("parentId", undefined) ] else []
    | Some x -> [ ("parentId", encode_json_string x) ])

and encode_history1_entries_item_kind (value : history1_entries_item_kind) : Js.Json.t =
  match value with
  | `Turn -> Js.Json.parseExn "\"turn\""
  | `Recovery -> Js.Json.parseExn "\"recovery\""
  | `Resume -> Js.Json.parseExn "\"resume\""

and encode_literal_turn (value : literal_turn) : Js.Json.t = Js.Json.parseExn "\"turn\""
and encode_literal_recovery (value : literal_recovery) : Js.Json.t = Js.Json.parseExn "\"recovery\""
and encode_literal_resume (value : literal_resume) : Js.Json.t = Js.Json.parseExn "\"resume\""

and encode_save3 (value : save3) : Js.Json.t =
  wire_to_json
    ~known:[ "version"; "checkpoint"; "pendingEnemy"; "planner"; "setup"; "progress"; "undo" ]
    value.wire
    ([
       ("version", encode_literal_3 value.version);
       ("checkpoint", encode_save3_checkpoint value.checkpoint);
       ("pendingEnemy", encode_json_boolean value.pending_enemy);
     ]
    @ (match value.planner with
      | None ->
          if List.mem "planner" value.wire.undefined_fields then [ ("planner", undefined) ] else []
      | Some x -> [ ("planner", encode_json_unknown x) ])
    @ (match value.setup with
      | None ->
          if List.mem "setup" value.wire.undefined_fields then [ ("setup", undefined) ] else []
      | Some x -> [ ("setup", encode_save2_setup_member1 x) ])
    @ (match value.progress with
      | None ->
          if List.mem "progress" value.wire.undefined_fields then [ ("progress", undefined) ]
          else []
      | Some x -> [ ("progress", encode_save2_progress_member1 x) ])
    @
    match value.undo with
    | None -> if List.mem "undo" value.wire.undefined_fields then [ ("undo", undefined) ] else []
    | Some x -> [ ("undo", encode_json_unknown x) ])

and encode_literal_3 (value : literal_3) : Js.Json.t = Js.Json.parseExn "3"

and encode_save3_checkpoint (value : save3_checkpoint) : Js.Json.t =
  wire_to_json ~known:[ "seed"; "state" ] value.wire
    [
      ("seed", encode_json_number value.seed); ("state", encode_save3_checkpoint_state value.state);
    ]

and encode_save3_checkpoint_state (value : save3_checkpoint_state) : Js.Json.t =
  wire_to_json
    ~known:
      [
        "round";
        "turn";
        "winner";
        "armies";
        "conditionMemory";
        "mapId";
        "mapRevision";
        "cols";
        "rows";
        "nextId";
        "log";
        "lastCombat";
        "units";
        "tiles";
        "campaign";
        "gold";
        "factions";
        "groundItems";
        "combatRng";
      ]
    value.wire
    ([
       ("round", encode_json_number value.round);
       ("turn", encode_save35_setup_first value.turn);
       ("winner", encode_fragments_native_state38_winner value.winner);
       ("mapId", encode_json_string value.map_id);
       ("mapRevision", encode_json_number value.map_revision);
       ("cols", encode_json_number value.cols);
       ("rows", encode_json_number value.rows);
       ("nextId", encode_json_number value.next_id);
       ("log", encode_fragments_native_state38_log value.log);
       ("units", encode_save2_checkpoint_state_units value.units);
       ("tiles", encode_fragments_tiles11 value.tiles);
       ("gold", encode_javascript1_state_gold value.gold);
     ]
    @ (match value.armies with
      | None ->
          if List.mem "armies" value.wire.undefined_fields then [ ("armies", undefined) ] else []
      | Some x -> [ ("armies", encode_fragments_native_state38_armies_member1 x) ])
    @ (match value.condition_memory with
      | None ->
          if List.mem "conditionMemory" value.wire.undefined_fields then
            [ ("conditionMemory", undefined) ]
          else []
      | Some x -> [ ("conditionMemory", encode_fragments_native_state38_log x) ])
    @ (match value.last_combat with
      | None ->
          if List.mem "lastCombat" value.wire.undefined_fields then [ ("lastCombat", undefined) ]
          else []
      | Some x -> [ ("lastCombat", encode_fragments_native_state38_last_combat x) ])
    @ (match value.campaign with
      | None ->
          if List.mem "campaign" value.wire.undefined_fields then [ ("campaign", undefined) ]
          else []
      | Some x -> [ ("campaign", encode_save2_checkpoint_state_campaign_member1 x) ])
    @ (match value.factions with
      | None ->
          if List.mem "factions" value.wire.undefined_fields then [ ("factions", undefined) ]
          else []
      | Some x -> [ ("factions", encode_fragments_factions7 x) ])
    @ (match value.ground_items with
      | None ->
          if List.mem "groundItems" value.wire.undefined_fields then [ ("groundItems", undefined) ]
          else []
      | Some x -> [ ("groundItems", encode_fragments_native_state38_ground_items x) ])
    @
    match value.combat_rng with
    | None ->
        if List.mem "combatRng" value.wire.undefined_fields then [ ("combatRng", undefined) ]
        else []
    | Some x -> [ ("combatRng", encode_json_number x) ])

and encode_fragments_tiles11 (value : fragments_tiles11) : Js.Json.t =
  Js.Json.array (Array.map encode_fragments_tiles11_item value)

and encode_fragments_tiles11_item (value : fragments_tiles11_item) : Js.Json.t =
  wire_to_json
    ~known:
      [
        "owner";
        "id";
        "location";
        "col";
        "row";
        "terrain";
        "connections";
        "deadEnds";
        "walls";
        "floor";
        "fog";
        "light";
        "wallCorners";
      ]
    value.wire
    ([
       ("owner", encode_fragments_tiles11_item_owner value.owner);
       ("id", encode_json_string value.id);
       ("col", encode_json_number value.col);
       ("row", encode_json_number value.row);
       ("terrain", encode_json_string value.terrain);
     ]
    @ (match value.location with
      | None ->
          if List.mem "location" value.wire.undefined_fields then [ ("location", undefined) ]
          else []
      | Some x -> [ ("location", encode_json_string x) ])
    @ (match value.connections with
      | None ->
          if List.mem "connections" value.wire.undefined_fields then [ ("connections", undefined) ]
          else []
      | Some x -> [ ("connections", encode_fragments_tiles11_item_connections_member1 x) ])
    @ (match value.dead_ends with
      | None ->
          if List.mem "deadEnds" value.wire.undefined_fields then [ ("deadEnds", undefined) ]
          else []
      | Some x -> [ ("deadEnds", encode_fragments_tiles11_item_connections_member1 x) ])
    @ (match value.walls with
      | None ->
          if List.mem "walls" value.wire.undefined_fields then [ ("walls", undefined) ] else []
      | Some x -> [ ("walls", encode_fragments_tiles11_item_connections_member1 x) ])
    @ (match value.floor with
      | None ->
          if List.mem "floor" value.wire.undefined_fields then [ ("floor", undefined) ] else []
      | Some x -> [ ("floor", encode_json_string x) ])
    @ (match value.fog with
      | None -> if List.mem "fog" value.wire.undefined_fields then [ ("fog", undefined) ] else []
      | Some x -> [ ("fog", encode_fragments_rule11 x) ])
    @ (match value.light with
      | None ->
          if List.mem "light" value.wire.undefined_fields then [ ("light", undefined) ] else []
      | Some x -> [ ("light", encode_fragments_light23 x) ])
    @
    match value.wall_corners with
    | None ->
        if List.mem "wallCorners" value.wire.undefined_fields then [ ("wallCorners", undefined) ]
        else []
    | Some x -> [ ("wallCorners", encode_fragments_tiles11_item_wall_corners_member1 x) ])

and encode_fragments_light23_when_ (value : fragments_light23_when_) : Js.Json.t =
  match value with `Undefined -> undefined | `Object2 x -> encode_fragments_rule11 x

and encode_fragments_rule11 (value : fragments_rule11) : Js.Json.t =
  wire_to_json ~known:[ "condition"; "remember" ] value.wire
    ([ ("condition", encode_fragments_rule11_condition value.condition) ]
    @
    match value.remember with
    | None ->
        if List.mem "remember" value.wire.undefined_fields then [ ("remember", undefined) ] else []
    | Some x -> [ ("remember", encode_fragments_rule11_remember x) ])

and encode_fragments_rule11_condition (value : fragments_rule11_condition) : Js.Json.t =
  match value with
  | `Owner x -> encode_fragments_rule11_condition_member0 x
  | `Visible_units x -> encode_fragments_rule11_condition_member1 x
  | `Choice x -> encode_fragments_rule11_condition_member2 x
  | `Flag x -> encode_fragments_rule11_condition_member3 x
  | `Event x -> encode_fragments_rule11_condition_member4 x
  | `Round x -> encode_fragments_rule11_condition_member5 x
  | `Proximity x -> encode_fragments_rule11_condition_member6 x
  | `Has_units x -> encode_fragments_rule11_condition_member7 x
  | `No_units x -> encode_fragments_rule11_condition_member8 x
  | `Object10 x -> encode_fragments_rule11_condition_member9 x

and encode_fragments_rule11_condition_member9 (value : fragments_rule11_condition_member9) :
    Js.Json.t =
  wire_to_json ~known:[ "kind"; "conditions" ] value.wire
    [
      ("kind", encode_fragments_rule11_condition_member9_kind value.kind);
      ("conditions", encode_fragments_rule11_condition_member9_conditions value.conditions);
    ]

and encode_fragments_rule11_condition_member9_conditions
    (value : fragments_rule11_condition_member9_conditions) : Js.Json.t =
  Js.Json.array (Array.map encode_fragments_rule11_condition value)

and encode_fragments_tiles11_item_light (value : fragments_tiles11_item_light) : Js.Json.t =
  match value with `Undefined -> undefined | `Object2 x -> encode_fragments_light23 x

and encode_fragments_light23 (value : fragments_light23) : Js.Json.t =
  wire_to_json
    ~known:[ "radius"; "color"; "fixture"; "dx"; "dy"; "power"; "when" ]
    value.wire
    ([ ("radius", encode_json_number value.radius); ("power", encode_json_number value.power) ]
    @ (match value.color with
      | None ->
          if List.mem "color" value.wire.undefined_fields then [ ("color", undefined) ] else []
      | Some x -> [ ("color", encode_fragments_light23_color_member1 x) ])
    @ (match value.fixture with
      | None ->
          if List.mem "fixture" value.wire.undefined_fields then [ ("fixture", undefined) ] else []
      | Some x -> [ ("fixture", encode_fragments_rule11_remember x) ])
    @ (match value.dx with
      | None -> if List.mem "dx" value.wire.undefined_fields then [ ("dx", undefined) ] else []
      | Some x -> [ ("dx", encode_json_number x) ])
    @ (match value.dy with
      | None -> if List.mem "dy" value.wire.undefined_fields then [ ("dy", undefined) ] else []
      | Some x -> [ ("dy", encode_json_number x) ])
    @
    match value.when_ with
    | None -> if List.mem "when" value.wire.undefined_fields then [ ("when", undefined) ] else []
    | Some x -> [ ("when", encode_fragments_rule11 x) ])

and encode_save3_checkpoint_state_factions (value : save3_checkpoint_state_factions) : Js.Json.t =
  match value with `Undefined -> undefined | `Array x -> encode_fragments_factions7 x

and encode_fragments_factions7 (value : fragments_factions7) : Js.Json.t =
  Js.Json.array (Array.map encode_fragments_factions7_item value)

and encode_fragments_factions7_item (value : fragments_factions7_item) : Js.Json.t =
  wire_to_json
    ~known:[ "id"; "name"; "phase"; "controller"; "palette"; "recruits"; "relations"; "transfer" ]
    value.wire
    ([
       ("id", encode_json_string value.id);
       ("phase", encode_save35_setup_first value.phase);
       ("controller", encode_fragments_factions7_item_controller value.controller);
       ("palette", encode_json_string value.palette);
     ]
    @ (match value.name with
      | None -> if List.mem "name" value.wire.undefined_fields then [ ("name", undefined) ] else []
      | Some x -> [ ("name", encode_json_string x) ])
    @ (match value.recruits with
      | None ->
          if List.mem "recruits" value.wire.undefined_fields then [ ("recruits", undefined) ]
          else []
      | Some x -> [ ("recruits", encode_fragments_factions7_item_recruits_member1 x) ])
    @ (match value.relations with
      | None ->
          if List.mem "relations" value.wire.undefined_fields then [ ("relations", undefined) ]
          else []
      | Some x -> [ ("relations", encode_fragments_factions7_item_relations_member1 x) ])
    @
    match value.transfer with
    | None ->
        if List.mem "transfer" value.wire.undefined_fields then [ ("transfer", undefined) ] else []
    | Some x -> [ ("transfer", encode_fragments_factions7_item_transfer_member1 x) ])

and encode_fragments_factions7_item_transfer (value : fragments_factions7_item_transfer) : Js.Json.t
    =
  match value with
  | `Undefined -> undefined
  | `Object2 x -> encode_fragments_factions7_item_transfer_member1 x

and encode_fragments_factions7_item_transfer_member1
    (value : fragments_factions7_item_transfer_member1) : Js.Json.t =
  wire_to_json ~known:[ "to"; "when" ] value.wire
    [ ("to", encode_json_string value.to_); ("when", encode_fragments_rule11 value.when_) ]

and encode_save4 (value : save4) : Js.Json.t =
  wire_to_json
    ~known:[ "version"; "checkpoint"; "pendingEnemy"; "planner"; "setup"; "progress"; "undo" ]
    value.wire
    ([
       ("version", encode_literal_4 value.version);
       ("checkpoint", encode_save3_checkpoint value.checkpoint);
       ("pendingEnemy", encode_json_boolean value.pending_enemy);
     ]
    @ (match value.planner with
      | None ->
          if List.mem "planner" value.wire.undefined_fields then [ ("planner", undefined) ] else []
      | Some x -> [ ("planner", encode_json_unknown x) ])
    @ (match value.setup with
      | None ->
          if List.mem "setup" value.wire.undefined_fields then [ ("setup", undefined) ] else []
      | Some x -> [ ("setup", encode_save2_setup_member1 x) ])
    @ (match value.progress with
      | None ->
          if List.mem "progress" value.wire.undefined_fields then [ ("progress", undefined) ]
          else []
      | Some x -> [ ("progress", encode_save2_progress_member1 x) ])
    @
    match value.undo with
    | None -> if List.mem "undo" value.wire.undefined_fields then [ ("undo", undefined) ] else []
    | Some x -> [ ("undo", encode_json_unknown x) ])

and encode_literal_4 (value : literal_4) : Js.Json.t = Js.Json.parseExn "4"

and encode_save5 (value : save5) : Js.Json.t =
  wire_to_json
    ~known:[ "version"; "checkpoint"; "pendingEnemy"; "setup"; "progress"; "undo" ]
    value.wire
    ([
       ("version", encode_literal_5 value.version);
       ("checkpoint", encode_save3_checkpoint value.checkpoint);
       ("pendingEnemy", encode_json_boolean value.pending_enemy);
     ]
    @ (match value.setup with
      | None ->
          if List.mem "setup" value.wire.undefined_fields then [ ("setup", undefined) ] else []
      | Some x -> [ ("setup", encode_save2_setup_member1 x) ])
    @ (match value.progress with
      | None ->
          if List.mem "progress" value.wire.undefined_fields then [ ("progress", undefined) ]
          else []
      | Some x -> [ ("progress", encode_save2_progress_member1 x) ])
    @
    match value.undo with
    | None -> if List.mem "undo" value.wire.undefined_fields then [ ("undo", undefined) ] else []
    | Some x -> [ ("undo", encode_json_unknown x) ])

and encode_literal_5 (value : literal_5) : Js.Json.t = Js.Json.parseExn "5"

and encode_save6 (value : save6) : Js.Json.t =
  wire_to_json
    ~known:[ "version"; "checkpoint"; "pendingEnemy"; "setup"; "progress"; "undo" ]
    value.wire
    ([
       ("version", encode_literal_6 value.version);
       ("checkpoint", encode_save3_checkpoint value.checkpoint);
       ("pendingEnemy", encode_json_boolean value.pending_enemy);
     ]
    @ (match value.setup with
      | None ->
          if List.mem "setup" value.wire.undefined_fields then [ ("setup", undefined) ] else []
      | Some x -> [ ("setup", encode_save2_setup_member1 x) ])
    @ (match value.progress with
      | None ->
          if List.mem "progress" value.wire.undefined_fields then [ ("progress", undefined) ]
          else []
      | Some x -> [ ("progress", encode_save2_progress_member1 x) ])
    @
    match value.undo with
    | None -> if List.mem "undo" value.wire.undefined_fields then [ ("undo", undefined) ] else []
    | Some x -> [ ("undo", encode_json_unknown x) ])

and encode_literal_6 (value : literal_6) : Js.Json.t = Js.Json.parseExn "6"

and encode_save7 (value : save7) : Js.Json.t =
  wire_to_json
    ~known:[ "version"; "checkpoint"; "pendingEnemy"; "setup"; "progress"; "undo" ]
    value.wire
    ([
       ("version", encode_literal_7 value.version);
       ("checkpoint", encode_save7_checkpoint value.checkpoint);
       ("pendingEnemy", encode_json_boolean value.pending_enemy);
     ]
    @ (match value.setup with
      | None ->
          if List.mem "setup" value.wire.undefined_fields then [ ("setup", undefined) ] else []
      | Some x -> [ ("setup", encode_save2_setup_member1 x) ])
    @ (match value.progress with
      | None ->
          if List.mem "progress" value.wire.undefined_fields then [ ("progress", undefined) ]
          else []
      | Some x -> [ ("progress", encode_save2_progress_member1 x) ])
    @
    match value.undo with
    | None -> if List.mem "undo" value.wire.undefined_fields then [ ("undo", undefined) ] else []
    | Some x -> [ ("undo", encode_json_unknown x) ])

and encode_literal_7 (value : literal_7) : Js.Json.t = Js.Json.parseExn "7"

and encode_save7_checkpoint (value : save7_checkpoint) : Js.Json.t =
  wire_to_json ~known:[ "seed"; "state" ] value.wire
    [
      ("seed", encode_json_number value.seed); ("state", encode_save7_checkpoint_state value.state);
    ]

and encode_save7_checkpoint_state (value : save7_checkpoint_state) : Js.Json.t =
  wire_to_json
    ~known:
      [
        "round";
        "turn";
        "winner";
        "armies";
        "conditionMemory";
        "mapId";
        "mapRevision";
        "cols";
        "rows";
        "nextId";
        "log";
        "lastCombat";
        "units";
        "tiles";
        "campaign";
        "gold";
        "factions";
        "groundItems";
        "combatRng";
      ]
    value.wire
    ([
       ("round", encode_json_number value.round);
       ("turn", encode_save35_setup_first value.turn);
       ("winner", encode_fragments_native_state38_winner value.winner);
       ("mapId", encode_json_string value.map_id);
       ("mapRevision", encode_json_number value.map_revision);
       ("cols", encode_json_number value.cols);
       ("rows", encode_json_number value.rows);
       ("nextId", encode_json_number value.next_id);
       ("log", encode_fragments_native_state38_log value.log);
       ("units", encode_save2_checkpoint_state_units value.units);
       ("tiles", encode_fragments_tiles11 value.tiles);
       ("gold", encode_javascript1_state_gold value.gold);
       ("factions", encode_fragments_factions7 value.factions);
     ]
    @ (match value.armies with
      | None ->
          if List.mem "armies" value.wire.undefined_fields then [ ("armies", undefined) ] else []
      | Some x -> [ ("armies", encode_fragments_native_state38_armies_member1 x) ])
    @ (match value.condition_memory with
      | None ->
          if List.mem "conditionMemory" value.wire.undefined_fields then
            [ ("conditionMemory", undefined) ]
          else []
      | Some x -> [ ("conditionMemory", encode_fragments_native_state38_log x) ])
    @ (match value.last_combat with
      | None ->
          if List.mem "lastCombat" value.wire.undefined_fields then [ ("lastCombat", undefined) ]
          else []
      | Some x -> [ ("lastCombat", encode_fragments_native_state38_last_combat x) ])
    @ (match value.campaign with
      | None ->
          if List.mem "campaign" value.wire.undefined_fields then [ ("campaign", undefined) ]
          else []
      | Some x -> [ ("campaign", encode_save2_checkpoint_state_campaign_member1 x) ])
    @ (match value.ground_items with
      | None ->
          if List.mem "groundItems" value.wire.undefined_fields then [ ("groundItems", undefined) ]
          else []
      | Some x -> [ ("groundItems", encode_fragments_native_state38_ground_items x) ])
    @
    match value.combat_rng with
    | None ->
        if List.mem "combatRng" value.wire.undefined_fields then [ ("combatRng", undefined) ]
        else []
    | Some x -> [ ("combatRng", encode_json_number x) ])

and encode_save8 (value : save8) : Js.Json.t =
  wire_to_json
    ~known:[ "version"; "checkpoint"; "pendingEnemy"; "setup"; "progress"; "undo" ]
    value.wire
    ([
       ("version", encode_literal_8 value.version);
       ("checkpoint", encode_save8_checkpoint value.checkpoint);
       ("pendingEnemy", encode_json_boolean value.pending_enemy);
     ]
    @ (match value.setup with
      | None ->
          if List.mem "setup" value.wire.undefined_fields then [ ("setup", undefined) ] else []
      | Some x -> [ ("setup", encode_save2_setup_member1 x) ])
    @ (match value.progress with
      | None ->
          if List.mem "progress" value.wire.undefined_fields then [ ("progress", undefined) ]
          else []
      | Some x -> [ ("progress", encode_save2_progress_member1 x) ])
    @
    match value.undo with
    | None -> if List.mem "undo" value.wire.undefined_fields then [ ("undo", undefined) ] else []
    | Some x -> [ ("undo", encode_json_unknown x) ])

and encode_literal_8 (value : literal_8) : Js.Json.t = Js.Json.parseExn "8"

and encode_save8_checkpoint (value : save8_checkpoint) : Js.Json.t =
  wire_to_json ~known:[ "seed"; "state" ] value.wire
    [
      ("seed", encode_json_number value.seed); ("state", encode_save8_checkpoint_state value.state);
    ]

and encode_save8_checkpoint_state (value : save8_checkpoint_state) : Js.Json.t =
  wire_to_json
    ~known:
      [
        "round";
        "turn";
        "winner";
        "armies";
        "conditionMemory";
        "mapId";
        "mapRevision";
        "cols";
        "rows";
        "nextId";
        "log";
        "lastCombat";
        "units";
        "tiles";
        "campaign";
        "gold";
        "factions";
        "groundItems";
        "combatRng";
      ]
    value.wire
    ([
       ("round", encode_json_number value.round);
       ("turn", encode_save35_setup_first value.turn);
       ("winner", encode_fragments_native_state38_winner value.winner);
       ("mapId", encode_json_string value.map_id);
       ("mapRevision", encode_json_number value.map_revision);
       ("cols", encode_json_number value.cols);
       ("rows", encode_json_number value.rows);
       ("nextId", encode_json_number value.next_id);
       ("log", encode_fragments_native_state38_log value.log);
       ("units", encode_save2_checkpoint_state_units value.units);
       ("tiles", encode_fragments_tiles11 value.tiles);
       ("gold", encode_javascript1_state_gold value.gold);
       ("factions", encode_fragments_factions7 value.factions);
       ("groundItems", encode_fragments_native_state38_ground_items value.ground_items);
     ]
    @ (match value.armies with
      | None ->
          if List.mem "armies" value.wire.undefined_fields then [ ("armies", undefined) ] else []
      | Some x -> [ ("armies", encode_fragments_native_state38_armies_member1 x) ])
    @ (match value.condition_memory with
      | None ->
          if List.mem "conditionMemory" value.wire.undefined_fields then
            [ ("conditionMemory", undefined) ]
          else []
      | Some x -> [ ("conditionMemory", encode_fragments_native_state38_log x) ])
    @ (match value.last_combat with
      | None ->
          if List.mem "lastCombat" value.wire.undefined_fields then [ ("lastCombat", undefined) ]
          else []
      | Some x -> [ ("lastCombat", encode_fragments_native_state38_last_combat x) ])
    @ (match value.campaign with
      | None ->
          if List.mem "campaign" value.wire.undefined_fields then [ ("campaign", undefined) ]
          else []
      | Some x -> [ ("campaign", encode_save2_checkpoint_state_campaign_member1 x) ])
    @
    match value.combat_rng with
    | None ->
        if List.mem "combatRng" value.wire.undefined_fields then [ ("combatRng", undefined) ]
        else []
    | Some x -> [ ("combatRng", encode_json_number x) ])

and encode_save9 (value : save9) : Js.Json.t =
  wire_to_json
    ~known:[ "version"; "checkpoint"; "pendingEnemy"; "setup"; "progress"; "undo" ]
    value.wire
    ([
       ("version", encode_literal_9 value.version);
       ("checkpoint", encode_save8_checkpoint value.checkpoint);
       ("pendingEnemy", encode_json_boolean value.pending_enemy);
     ]
    @ (match value.setup with
      | None ->
          if List.mem "setup" value.wire.undefined_fields then [ ("setup", undefined) ] else []
      | Some x -> [ ("setup", encode_save2_setup_member1 x) ])
    @ (match value.progress with
      | None ->
          if List.mem "progress" value.wire.undefined_fields then [ ("progress", undefined) ]
          else []
      | Some x -> [ ("progress", encode_save2_progress_member1 x) ])
    @
    match value.undo with
    | None -> if List.mem "undo" value.wire.undefined_fields then [ ("undo", undefined) ] else []
    | Some x -> [ ("undo", encode_json_unknown x) ])

and encode_literal_9 (value : literal_9) : Js.Json.t = Js.Json.parseExn "9"

and encode_save10 (value : save10) : Js.Json.t =
  wire_to_json
    ~known:[ "version"; "checkpoint"; "pendingEnemy"; "setup"; "progress"; "undo" ]
    value.wire
    ([
       ("version", encode_literal_10 value.version);
       ("checkpoint", encode_save8_checkpoint value.checkpoint);
       ("pendingEnemy", encode_json_boolean value.pending_enemy);
     ]
    @ (match value.setup with
      | None ->
          if List.mem "setup" value.wire.undefined_fields then [ ("setup", undefined) ] else []
      | Some x -> [ ("setup", encode_save2_setup_member1 x) ])
    @ (match value.progress with
      | None ->
          if List.mem "progress" value.wire.undefined_fields then [ ("progress", undefined) ]
          else []
      | Some x -> [ ("progress", encode_save2_progress_member1 x) ])
    @
    match value.undo with
    | None -> if List.mem "undo" value.wire.undefined_fields then [ ("undo", undefined) ] else []
    | Some x -> [ ("undo", encode_json_unknown x) ])

and encode_literal_10 (value : literal_10) : Js.Json.t = Js.Json.parseExn "10"

and encode_save11 (value : save11) : Js.Json.t =
  wire_to_json
    ~known:[ "version"; "checkpoint"; "pendingEnemy"; "setup"; "progress"; "undo" ]
    value.wire
    ([
       ("version", encode_literal_11 value.version);
       ("checkpoint", encode_save8_checkpoint value.checkpoint);
       ("pendingEnemy", encode_json_boolean value.pending_enemy);
     ]
    @ (match value.setup with
      | None ->
          if List.mem "setup" value.wire.undefined_fields then [ ("setup", undefined) ] else []
      | Some x -> [ ("setup", encode_save2_setup_member1 x) ])
    @ (match value.progress with
      | None ->
          if List.mem "progress" value.wire.undefined_fields then [ ("progress", undefined) ]
          else []
      | Some x -> [ ("progress", encode_save2_progress_member1 x) ])
    @
    match value.undo with
    | None -> if List.mem "undo" value.wire.undefined_fields then [ ("undo", undefined) ] else []
    | Some x -> [ ("undo", encode_json_unknown x) ])

and encode_literal_11 (value : literal_11) : Js.Json.t = Js.Json.parseExn "11"

and encode_save12 (value : save12) : Js.Json.t =
  wire_to_json
    ~known:[ "version"; "checkpoint"; "pendingEnemy"; "setup"; "progress"; "undo" ]
    value.wire
    ([
       ("version", encode_literal_12 value.version);
       ("checkpoint", encode_save8_checkpoint value.checkpoint);
       ("pendingEnemy", encode_json_boolean value.pending_enemy);
     ]
    @ (match value.setup with
      | None ->
          if List.mem "setup" value.wire.undefined_fields then [ ("setup", undefined) ] else []
      | Some x -> [ ("setup", encode_save2_setup_member1 x) ])
    @ (match value.progress with
      | None ->
          if List.mem "progress" value.wire.undefined_fields then [ ("progress", undefined) ]
          else []
      | Some x -> [ ("progress", encode_save2_progress_member1 x) ])
    @
    match value.undo with
    | None -> if List.mem "undo" value.wire.undefined_fields then [ ("undo", undefined) ] else []
    | Some x -> [ ("undo", encode_json_unknown x) ])

and encode_literal_12 (value : literal_12) : Js.Json.t = Js.Json.parseExn "12"

and encode_save13 (value : save13) : Js.Json.t =
  wire_to_json
    ~known:[ "version"; "checkpoint"; "pendingEnemy"; "setup"; "progress"; "undo" ]
    value.wire
    ([
       ("version", encode_literal_13 value.version);
       ("checkpoint", encode_save8_checkpoint value.checkpoint);
       ("pendingEnemy", encode_json_boolean value.pending_enemy);
     ]
    @ (match value.setup with
      | None ->
          if List.mem "setup" value.wire.undefined_fields then [ ("setup", undefined) ] else []
      | Some x -> [ ("setup", encode_save2_setup_member1 x) ])
    @ (match value.progress with
      | None ->
          if List.mem "progress" value.wire.undefined_fields then [ ("progress", undefined) ]
          else []
      | Some x -> [ ("progress", encode_save2_progress_member1 x) ])
    @
    match value.undo with
    | None -> if List.mem "undo" value.wire.undefined_fields then [ ("undo", undefined) ] else []
    | Some x -> [ ("undo", encode_json_unknown x) ])

and encode_literal_13 (value : literal_13) : Js.Json.t = Js.Json.parseExn "13"

and encode_save14 (value : save14) : Js.Json.t =
  wire_to_json
    ~known:[ "version"; "checkpoint"; "pendingEnemy"; "setup"; "progress"; "undo" ]
    value.wire
    ([
       ("version", encode_literal_14 value.version);
       ("checkpoint", encode_save8_checkpoint value.checkpoint);
       ("pendingEnemy", encode_json_boolean value.pending_enemy);
     ]
    @ (match value.setup with
      | None ->
          if List.mem "setup" value.wire.undefined_fields then [ ("setup", undefined) ] else []
      | Some x -> [ ("setup", encode_save2_setup_member1 x) ])
    @ (match value.progress with
      | None ->
          if List.mem "progress" value.wire.undefined_fields then [ ("progress", undefined) ]
          else []
      | Some x -> [ ("progress", encode_save2_progress_member1 x) ])
    @
    match value.undo with
    | None -> if List.mem "undo" value.wire.undefined_fields then [ ("undo", undefined) ] else []
    | Some x -> [ ("undo", encode_json_unknown x) ])

and encode_literal_14 (value : literal_14) : Js.Json.t = Js.Json.parseExn "14"

and encode_save15 (value : save15) : Js.Json.t =
  wire_to_json
    ~known:[ "version"; "checkpoint"; "pendingEnemy"; "setup"; "progress"; "undo" ]
    value.wire
    ([
       ("version", encode_literal_15 value.version);
       ("checkpoint", encode_save8_checkpoint value.checkpoint);
       ("pendingEnemy", encode_json_boolean value.pending_enemy);
     ]
    @ (match value.setup with
      | None ->
          if List.mem "setup" value.wire.undefined_fields then [ ("setup", undefined) ] else []
      | Some x -> [ ("setup", encode_save2_setup_member1 x) ])
    @ (match value.progress with
      | None ->
          if List.mem "progress" value.wire.undefined_fields then [ ("progress", undefined) ]
          else []
      | Some x -> [ ("progress", encode_save2_progress_member1 x) ])
    @
    match value.undo with
    | None -> if List.mem "undo" value.wire.undefined_fields then [ ("undo", undefined) ] else []
    | Some x -> [ ("undo", encode_json_unknown x) ])

and encode_literal_15 (value : literal_15) : Js.Json.t = Js.Json.parseExn "15"

and encode_save16 (value : save16) : Js.Json.t =
  wire_to_json
    ~known:[ "version"; "checkpoint"; "pendingEnemy"; "setup"; "progress"; "undo" ]
    value.wire
    ([
       ("version", encode_literal_16 value.version);
       ("checkpoint", encode_save8_checkpoint value.checkpoint);
       ("pendingEnemy", encode_json_boolean value.pending_enemy);
     ]
    @ (match value.setup with
      | None ->
          if List.mem "setup" value.wire.undefined_fields then [ ("setup", undefined) ] else []
      | Some x -> [ ("setup", encode_save2_setup_member1 x) ])
    @ (match value.progress with
      | None ->
          if List.mem "progress" value.wire.undefined_fields then [ ("progress", undefined) ]
          else []
      | Some x -> [ ("progress", encode_save2_progress_member1 x) ])
    @
    match value.undo with
    | None -> if List.mem "undo" value.wire.undefined_fields then [ ("undo", undefined) ] else []
    | Some x -> [ ("undo", encode_json_unknown x) ])

and encode_literal_16 (value : literal_16) : Js.Json.t = Js.Json.parseExn "16"

and encode_save17 (value : save17) : Js.Json.t =
  wire_to_json
    ~known:[ "version"; "checkpoint"; "pendingEnemy"; "setup"; "progress"; "undo" ]
    value.wire
    ([
       ("version", encode_literal_17 value.version);
       ("checkpoint", encode_save17_checkpoint value.checkpoint);
       ("pendingEnemy", encode_json_boolean value.pending_enemy);
     ]
    @ (match value.setup with
      | None ->
          if List.mem "setup" value.wire.undefined_fields then [ ("setup", undefined) ] else []
      | Some x -> [ ("setup", encode_save2_setup_member1 x) ])
    @ (match value.progress with
      | None ->
          if List.mem "progress" value.wire.undefined_fields then [ ("progress", undefined) ]
          else []
      | Some x -> [ ("progress", encode_save2_progress_member1 x) ])
    @
    match value.undo with
    | None -> if List.mem "undo" value.wire.undefined_fields then [ ("undo", undefined) ] else []
    | Some x -> [ ("undo", encode_json_unknown x) ])

and encode_literal_17 (value : literal_17) : Js.Json.t = Js.Json.parseExn "17"

and encode_save17_checkpoint (value : save17_checkpoint) : Js.Json.t =
  wire_to_json ~known:[ "seed"; "state" ] value.wire
    [
      ("seed", encode_json_number value.seed); ("state", encode_save17_checkpoint_state value.state);
    ]

and encode_save17_checkpoint_state (value : save17_checkpoint_state) : Js.Json.t =
  wire_to_json
    ~known:
      [
        "round";
        "turn";
        "winner";
        "armies";
        "conditionMemory";
        "mapId";
        "mapRevision";
        "cols";
        "rows";
        "nextId";
        "log";
        "lastCombat";
        "units";
        "tiles";
        "campaign";
        "gold";
        "factions";
        "groundItems";
        "combatRng";
      ]
    value.wire
    ([
       ("round", encode_json_number value.round);
       ("turn", encode_save35_setup_first value.turn);
       ("winner", encode_fragments_native_state38_winner value.winner);
       ("mapId", encode_json_string value.map_id);
       ("mapRevision", encode_json_number value.map_revision);
       ("cols", encode_json_number value.cols);
       ("rows", encode_json_number value.rows);
       ("nextId", encode_json_number value.next_id);
       ("log", encode_fragments_native_state38_log value.log);
       ("units", encode_save2_checkpoint_state_units value.units);
       ("tiles", encode_save17_checkpoint_state_tiles value.tiles);
       ("gold", encode_javascript1_state_gold value.gold);
       ("factions", encode_fragments_factions7 value.factions);
       ("groundItems", encode_fragments_native_state38_ground_items value.ground_items);
     ]
    @ (match value.armies with
      | None ->
          if List.mem "armies" value.wire.undefined_fields then [ ("armies", undefined) ] else []
      | Some x -> [ ("armies", encode_fragments_native_state38_armies_member1 x) ])
    @ (match value.condition_memory with
      | None ->
          if List.mem "conditionMemory" value.wire.undefined_fields then
            [ ("conditionMemory", undefined) ]
          else []
      | Some x -> [ ("conditionMemory", encode_fragments_native_state38_log x) ])
    @ (match value.last_combat with
      | None ->
          if List.mem "lastCombat" value.wire.undefined_fields then [ ("lastCombat", undefined) ]
          else []
      | Some x -> [ ("lastCombat", encode_fragments_native_state38_last_combat x) ])
    @ (match value.campaign with
      | None ->
          if List.mem "campaign" value.wire.undefined_fields then [ ("campaign", undefined) ]
          else []
      | Some x -> [ ("campaign", encode_save2_checkpoint_state_campaign_member1 x) ])
    @
    match value.combat_rng with
    | None ->
        if List.mem "combatRng" value.wire.undefined_fields then [ ("combatRng", undefined) ]
        else []
    | Some x -> [ ("combatRng", encode_json_number x) ])

and encode_save17_checkpoint_state_tiles (value : save17_checkpoint_state_tiles) : Js.Json.t =
  Js.Json.array (Array.map encode_save17_checkpoint_state_tiles_item value)

and encode_save17_checkpoint_state_tiles_item (value : save17_checkpoint_state_tiles_item) :
    Js.Json.t =
  wire_to_json
    ~known:
      [
        "owner";
        "id";
        "location";
        "col";
        "row";
        "terrain";
        "connections";
        "deadEnds";
        "walls";
        "floor";
        "fog";
        "light";
        "gates";
      ]
    value.wire
    ([
       ("owner", encode_fragments_tiles11_item_owner value.owner);
       ("id", encode_json_string value.id);
       ("col", encode_json_number value.col);
       ("row", encode_json_number value.row);
       ("terrain", encode_json_string value.terrain);
     ]
    @ (match value.location with
      | None ->
          if List.mem "location" value.wire.undefined_fields then [ ("location", undefined) ]
          else []
      | Some x -> [ ("location", encode_json_string x) ])
    @ (match value.connections with
      | None ->
          if List.mem "connections" value.wire.undefined_fields then [ ("connections", undefined) ]
          else []
      | Some x -> [ ("connections", encode_fragments_tiles11_item_connections_member1 x) ])
    @ (match value.dead_ends with
      | None ->
          if List.mem "deadEnds" value.wire.undefined_fields then [ ("deadEnds", undefined) ]
          else []
      | Some x -> [ ("deadEnds", encode_fragments_tiles11_item_connections_member1 x) ])
    @ (match value.walls with
      | None ->
          if List.mem "walls" value.wire.undefined_fields then [ ("walls", undefined) ] else []
      | Some x -> [ ("walls", encode_fragments_tiles11_item_connections_member1 x) ])
    @ (match value.floor with
      | None ->
          if List.mem "floor" value.wire.undefined_fields then [ ("floor", undefined) ] else []
      | Some x -> [ ("floor", encode_json_string x) ])
    @ (match value.fog with
      | None -> if List.mem "fog" value.wire.undefined_fields then [ ("fog", undefined) ] else []
      | Some x -> [ ("fog", encode_fragments_rule11 x) ])
    @ (match value.light with
      | None ->
          if List.mem "light" value.wire.undefined_fields then [ ("light", undefined) ] else []
      | Some x -> [ ("light", encode_fragments_light23 x) ])
    @
    match value.gates with
    | None -> if List.mem "gates" value.wire.undefined_fields then [ ("gates", undefined) ] else []
    | Some x -> [ ("gates", encode_fragments_native_state38_log x) ])

and encode_save18 (value : save18) : Js.Json.t =
  wire_to_json
    ~known:[ "version"; "checkpoint"; "pendingEnemy"; "setup"; "progress"; "undo" ]
    value.wire
    ([
       ("version", encode_literal_18 value.version);
       ("checkpoint", encode_save18_checkpoint value.checkpoint);
       ("pendingEnemy", encode_json_boolean value.pending_enemy);
     ]
    @ (match value.setup with
      | None ->
          if List.mem "setup" value.wire.undefined_fields then [ ("setup", undefined) ] else []
      | Some x -> [ ("setup", encode_save2_setup_member1 x) ])
    @ (match value.progress with
      | None ->
          if List.mem "progress" value.wire.undefined_fields then [ ("progress", undefined) ]
          else []
      | Some x -> [ ("progress", encode_save2_progress_member1 x) ])
    @
    match value.undo with
    | None -> if List.mem "undo" value.wire.undefined_fields then [ ("undo", undefined) ] else []
    | Some x -> [ ("undo", encode_json_unknown x) ])

and encode_literal_18 (value : literal_18) : Js.Json.t = Js.Json.parseExn "18"

and encode_save18_checkpoint (value : save18_checkpoint) : Js.Json.t =
  wire_to_json ~known:[ "seed"; "state" ] value.wire
    [
      ("seed", encode_json_number value.seed); ("state", encode_save18_checkpoint_state value.state);
    ]

and encode_save18_checkpoint_state (value : save18_checkpoint_state) : Js.Json.t =
  wire_to_json
    ~known:
      [
        "round";
        "turn";
        "winner";
        "armies";
        "conditionMemory";
        "mapId";
        "mapRevision";
        "cols";
        "rows";
        "nextId";
        "log";
        "lastCombat";
        "units";
        "tiles";
        "campaign";
        "gold";
        "factions";
        "groundItems";
        "combatRng";
      ]
    value.wire
    ([
       ("round", encode_json_number value.round);
       ("turn", encode_save35_setup_first value.turn);
       ("winner", encode_fragments_native_state38_winner value.winner);
       ("mapId", encode_json_string value.map_id);
       ("mapRevision", encode_json_number value.map_revision);
       ("cols", encode_json_number value.cols);
       ("rows", encode_json_number value.rows);
       ("nextId", encode_json_number value.next_id);
       ("log", encode_fragments_native_state38_log value.log);
       ("units", encode_save2_checkpoint_state_units value.units);
       ("tiles", encode_save18_checkpoint_state_tiles value.tiles);
       ("gold", encode_javascript1_state_gold value.gold);
       ("factions", encode_fragments_factions7 value.factions);
       ("groundItems", encode_fragments_native_state38_ground_items value.ground_items);
     ]
    @ (match value.armies with
      | None ->
          if List.mem "armies" value.wire.undefined_fields then [ ("armies", undefined) ] else []
      | Some x -> [ ("armies", encode_fragments_native_state38_armies_member1 x) ])
    @ (match value.condition_memory with
      | None ->
          if List.mem "conditionMemory" value.wire.undefined_fields then
            [ ("conditionMemory", undefined) ]
          else []
      | Some x -> [ ("conditionMemory", encode_fragments_native_state38_log x) ])
    @ (match value.last_combat with
      | None ->
          if List.mem "lastCombat" value.wire.undefined_fields then [ ("lastCombat", undefined) ]
          else []
      | Some x -> [ ("lastCombat", encode_fragments_native_state38_last_combat x) ])
    @ (match value.campaign with
      | None ->
          if List.mem "campaign" value.wire.undefined_fields then [ ("campaign", undefined) ]
          else []
      | Some x -> [ ("campaign", encode_save2_checkpoint_state_campaign_member1 x) ])
    @
    match value.combat_rng with
    | None ->
        if List.mem "combatRng" value.wire.undefined_fields then [ ("combatRng", undefined) ]
        else []
    | Some x -> [ ("combatRng", encode_json_number x) ])

and encode_save18_checkpoint_state_tiles (value : save18_checkpoint_state_tiles) : Js.Json.t =
  Js.Json.array (Array.map encode_save18_checkpoint_state_tiles_item value)

and encode_save18_checkpoint_state_tiles_item (value : save18_checkpoint_state_tiles_item) :
    Js.Json.t =
  wire_to_json
    ~known:
      [
        "owner";
        "id";
        "location";
        "col";
        "row";
        "terrain";
        "connections";
        "deadEnds";
        "walls";
        "floor";
        "fog";
        "light";
      ]
    value.wire
    ([
       ("owner", encode_fragments_tiles11_item_owner value.owner);
       ("id", encode_json_string value.id);
       ("col", encode_json_number value.col);
       ("row", encode_json_number value.row);
       ("terrain", encode_json_string value.terrain);
     ]
    @ (match value.location with
      | None ->
          if List.mem "location" value.wire.undefined_fields then [ ("location", undefined) ]
          else []
      | Some x -> [ ("location", encode_json_string x) ])
    @ (match value.connections with
      | None ->
          if List.mem "connections" value.wire.undefined_fields then [ ("connections", undefined) ]
          else []
      | Some x -> [ ("connections", encode_fragments_tiles11_item_connections_member1 x) ])
    @ (match value.dead_ends with
      | None ->
          if List.mem "deadEnds" value.wire.undefined_fields then [ ("deadEnds", undefined) ]
          else []
      | Some x -> [ ("deadEnds", encode_fragments_tiles11_item_connections_member1 x) ])
    @ (match value.walls with
      | None ->
          if List.mem "walls" value.wire.undefined_fields then [ ("walls", undefined) ] else []
      | Some x -> [ ("walls", encode_fragments_tiles11_item_connections_member1 x) ])
    @ (match value.floor with
      | None ->
          if List.mem "floor" value.wire.undefined_fields then [ ("floor", undefined) ] else []
      | Some x -> [ ("floor", encode_json_string x) ])
    @ (match value.fog with
      | None -> if List.mem "fog" value.wire.undefined_fields then [ ("fog", undefined) ] else []
      | Some x -> [ ("fog", encode_fragments_rule11 x) ])
    @
    match value.light with
    | None -> if List.mem "light" value.wire.undefined_fields then [ ("light", undefined) ] else []
    | Some x -> [ ("light", encode_fragments_light23 x) ])

and encode_save19 (value : save19) : Js.Json.t =
  wire_to_json
    ~known:[ "version"; "checkpoint"; "pendingEnemy"; "setup"; "progress"; "undo" ]
    value.wire
    ([
       ("version", encode_literal_19 value.version);
       ("checkpoint", encode_save18_checkpoint value.checkpoint);
       ("pendingEnemy", encode_json_boolean value.pending_enemy);
     ]
    @ (match value.setup with
      | None ->
          if List.mem "setup" value.wire.undefined_fields then [ ("setup", undefined) ] else []
      | Some x -> [ ("setup", encode_save2_setup_member1 x) ])
    @ (match value.progress with
      | None ->
          if List.mem "progress" value.wire.undefined_fields then [ ("progress", undefined) ]
          else []
      | Some x -> [ ("progress", encode_save2_progress_member1 x) ])
    @
    match value.undo with
    | None -> if List.mem "undo" value.wire.undefined_fields then [ ("undo", undefined) ] else []
    | Some x -> [ ("undo", encode_json_unknown x) ])

and encode_literal_19 (value : literal_19) : Js.Json.t = Js.Json.parseExn "19"

and encode_save20 (value : save20) : Js.Json.t =
  wire_to_json
    ~known:[ "version"; "checkpoint"; "pendingEnemy"; "setup"; "progress"; "undo" ]
    value.wire
    ([
       ("version", encode_literal_20 value.version);
       ("checkpoint", encode_save18_checkpoint value.checkpoint);
       ("pendingEnemy", encode_json_boolean value.pending_enemy);
     ]
    @ (match value.setup with
      | None ->
          if List.mem "setup" value.wire.undefined_fields then [ ("setup", undefined) ] else []
      | Some x -> [ ("setup", encode_save2_setup_member1 x) ])
    @ (match value.progress with
      | None ->
          if List.mem "progress" value.wire.undefined_fields then [ ("progress", undefined) ]
          else []
      | Some x -> [ ("progress", encode_save2_progress_member1 x) ])
    @
    match value.undo with
    | None -> if List.mem "undo" value.wire.undefined_fields then [ ("undo", undefined) ] else []
    | Some x -> [ ("undo", encode_json_unknown x) ])

and encode_literal_20 (value : literal_20) : Js.Json.t = Js.Json.parseExn "20"

and encode_save21 (value : save21) : Js.Json.t =
  wire_to_json
    ~known:[ "version"; "checkpoint"; "pendingEnemy"; "setup"; "progress"; "undo" ]
    value.wire
    ([
       ("version", encode_literal_21 value.version);
       ("checkpoint", encode_save21_checkpoint value.checkpoint);
       ("pendingEnemy", encode_json_boolean value.pending_enemy);
     ]
    @ (match value.setup with
      | None ->
          if List.mem "setup" value.wire.undefined_fields then [ ("setup", undefined) ] else []
      | Some x -> [ ("setup", encode_save2_setup_member1 x) ])
    @ (match value.progress with
      | None ->
          if List.mem "progress" value.wire.undefined_fields then [ ("progress", undefined) ]
          else []
      | Some x -> [ ("progress", encode_save2_progress_member1 x) ])
    @
    match value.undo with
    | None -> if List.mem "undo" value.wire.undefined_fields then [ ("undo", undefined) ] else []
    | Some x -> [ ("undo", encode_json_unknown x) ])

and encode_literal_21 (value : literal_21) : Js.Json.t = Js.Json.parseExn "21"

and encode_save21_checkpoint (value : save21_checkpoint) : Js.Json.t =
  wire_to_json ~known:[ "seed"; "state" ] value.wire
    [
      ("seed", encode_json_number value.seed); ("state", encode_save21_checkpoint_state value.state);
    ]

and encode_save21_checkpoint_state (value : save21_checkpoint_state) : Js.Json.t =
  wire_to_json
    ~known:
      [
        "round";
        "turn";
        "winner";
        "armies";
        "conditionMemory";
        "mapId";
        "mapRevision";
        "cols";
        "rows";
        "nextId";
        "log";
        "lastCombat";
        "units";
        "tiles";
        "campaign";
        "gold";
        "factions";
        "groundItems";
        "combatRng";
      ]
    value.wire
    ([
       ("round", encode_json_number value.round);
       ("turn", encode_save35_setup_first value.turn);
       ("winner", encode_fragments_native_state38_winner value.winner);
       ("mapId", encode_json_string value.map_id);
       ("mapRevision", encode_json_number value.map_revision);
       ("cols", encode_json_number value.cols);
       ("rows", encode_json_number value.rows);
       ("nextId", encode_json_number value.next_id);
       ("log", encode_fragments_native_state38_log value.log);
       ("units", encode_save2_checkpoint_state_units value.units);
       ("tiles", encode_save18_checkpoint_state_tiles value.tiles);
       ("gold", encode_fragments_native_state38_gold value.gold);
       ("factions", encode_fragments_factions7 value.factions);
       ("groundItems", encode_fragments_native_state38_ground_items value.ground_items);
     ]
    @ (match value.armies with
      | None ->
          if List.mem "armies" value.wire.undefined_fields then [ ("armies", undefined) ] else []
      | Some x -> [ ("armies", encode_fragments_native_state38_armies_member1 x) ])
    @ (match value.condition_memory with
      | None ->
          if List.mem "conditionMemory" value.wire.undefined_fields then
            [ ("conditionMemory", undefined) ]
          else []
      | Some x -> [ ("conditionMemory", encode_fragments_native_state38_log x) ])
    @ (match value.last_combat with
      | None ->
          if List.mem "lastCombat" value.wire.undefined_fields then [ ("lastCombat", undefined) ]
          else []
      | Some x -> [ ("lastCombat", encode_fragments_native_state38_last_combat x) ])
    @ (match value.campaign with
      | None ->
          if List.mem "campaign" value.wire.undefined_fields then [ ("campaign", undefined) ]
          else []
      | Some x -> [ ("campaign", encode_save2_checkpoint_state_campaign_member1 x) ])
    @
    match value.combat_rng with
    | None ->
        if List.mem "combatRng" value.wire.undefined_fields then [ ("combatRng", undefined) ]
        else []
    | Some x -> [ ("combatRng", encode_json_number x) ])

and encode_fragments_native_state38_gold (value : fragments_native_state38_gold) : Js.Json.t =
  map_to_json encode_json_number value

and encode_save22 (value : save22) : Js.Json.t =
  wire_to_json
    ~known:[ "version"; "checkpoint"; "pendingEnemy"; "setup"; "progress"; "undo" ]
    value.wire
    ([
       ("version", encode_literal_22 value.version);
       ("checkpoint", encode_save21_checkpoint value.checkpoint);
       ("pendingEnemy", encode_json_boolean value.pending_enemy);
     ]
    @ (match value.setup with
      | None ->
          if List.mem "setup" value.wire.undefined_fields then [ ("setup", undefined) ] else []
      | Some x -> [ ("setup", encode_save2_setup_member1 x) ])
    @ (match value.progress with
      | None ->
          if List.mem "progress" value.wire.undefined_fields then [ ("progress", undefined) ]
          else []
      | Some x -> [ ("progress", encode_save2_progress_member1 x) ])
    @
    match value.undo with
    | None -> if List.mem "undo" value.wire.undefined_fields then [ ("undo", undefined) ] else []
    | Some x -> [ ("undo", encode_json_unknown x) ])

and encode_literal_22 (value : literal_22) : Js.Json.t = Js.Json.parseExn "22"

and encode_save23 (value : save23) : Js.Json.t =
  wire_to_json
    ~known:[ "version"; "checkpoint"; "pendingEnemy"; "setup"; "progress"; "undo" ]
    value.wire
    ([
       ("version", encode_literal_23 value.version);
       ("checkpoint", encode_save21_checkpoint value.checkpoint);
       ("pendingEnemy", encode_json_boolean value.pending_enemy);
     ]
    @ (match value.setup with
      | None ->
          if List.mem "setup" value.wire.undefined_fields then [ ("setup", undefined) ] else []
      | Some x -> [ ("setup", encode_save2_setup_member1 x) ])
    @ (match value.progress with
      | None ->
          if List.mem "progress" value.wire.undefined_fields then [ ("progress", undefined) ]
          else []
      | Some x -> [ ("progress", encode_save2_progress_member1 x) ])
    @
    match value.undo with
    | None -> if List.mem "undo" value.wire.undefined_fields then [ ("undo", undefined) ] else []
    | Some x -> [ ("undo", encode_json_unknown x) ])

and encode_literal_23 (value : literal_23) : Js.Json.t = Js.Json.parseExn "23"

and encode_save24 (value : save24) : Js.Json.t =
  wire_to_json
    ~known:[ "version"; "checkpoint"; "pendingEnemy"; "setup"; "progress"; "undo" ]
    value.wire
    ([
       ("version", encode_literal_24 value.version);
       ("checkpoint", encode_save21_checkpoint value.checkpoint);
       ("pendingEnemy", encode_json_boolean value.pending_enemy);
     ]
    @ (match value.setup with
      | None ->
          if List.mem "setup" value.wire.undefined_fields then [ ("setup", undefined) ] else []
      | Some x -> [ ("setup", encode_save2_setup_member1 x) ])
    @ (match value.progress with
      | None ->
          if List.mem "progress" value.wire.undefined_fields then [ ("progress", undefined) ]
          else []
      | Some x -> [ ("progress", encode_save2_progress_member1 x) ])
    @
    match value.undo with
    | None -> if List.mem "undo" value.wire.undefined_fields then [ ("undo", undefined) ] else []
    | Some x -> [ ("undo", encode_json_unknown x) ])

and encode_literal_24 (value : literal_24) : Js.Json.t = Js.Json.parseExn "24"

and encode_save25 (value : save25) : Js.Json.t =
  wire_to_json
    ~known:[ "version"; "checkpoint"; "pendingEnemy"; "setup"; "progress"; "undo" ]
    value.wire
    ([
       ("version", encode_literal_25 value.version);
       ("checkpoint", encode_save21_checkpoint value.checkpoint);
       ("pendingEnemy", encode_json_boolean value.pending_enemy);
     ]
    @ (match value.setup with
      | None ->
          if List.mem "setup" value.wire.undefined_fields then [ ("setup", undefined) ] else []
      | Some x -> [ ("setup", encode_save2_setup_member1 x) ])
    @ (match value.progress with
      | None ->
          if List.mem "progress" value.wire.undefined_fields then [ ("progress", undefined) ]
          else []
      | Some x -> [ ("progress", encode_save2_progress_member1 x) ])
    @
    match value.undo with
    | None -> if List.mem "undo" value.wire.undefined_fields then [ ("undo", undefined) ] else []
    | Some x -> [ ("undo", encode_json_unknown x) ])

and encode_literal_25 (value : literal_25) : Js.Json.t = Js.Json.parseExn "25"

and encode_save26 (value : save26) : Js.Json.t =
  wire_to_json
    ~known:[ "version"; "checkpoint"; "pendingEnemy"; "setup"; "progress"; "undo" ]
    value.wire
    ([
       ("version", encode_literal_26 value.version);
       ("checkpoint", encode_save26_checkpoint value.checkpoint);
       ("pendingEnemy", encode_json_boolean value.pending_enemy);
     ]
    @ (match value.setup with
      | None ->
          if List.mem "setup" value.wire.undefined_fields then [ ("setup", undefined) ] else []
      | Some x -> [ ("setup", encode_save2_setup_member1 x) ])
    @ (match value.progress with
      | None ->
          if List.mem "progress" value.wire.undefined_fields then [ ("progress", undefined) ]
          else []
      | Some x -> [ ("progress", encode_save2_progress_member1 x) ])
    @
    match value.undo with
    | None -> if List.mem "undo" value.wire.undefined_fields then [ ("undo", undefined) ] else []
    | Some x -> [ ("undo", encode_json_unknown x) ])

and encode_literal_26 (value : literal_26) : Js.Json.t = Js.Json.parseExn "26"

and encode_save26_checkpoint (value : save26_checkpoint) : Js.Json.t =
  wire_to_json ~known:[ "seed"; "state" ] value.wire
    [
      ("seed", encode_json_number value.seed); ("state", encode_save26_checkpoint_state value.state);
    ]

and encode_save26_checkpoint_state (value : save26_checkpoint_state) : Js.Json.t =
  wire_to_json
    ~known:
      [
        "round";
        "turn";
        "winner";
        "armies";
        "conditionMemory";
        "mapId";
        "mapRevision";
        "cols";
        "rows";
        "nextId";
        "log";
        "lastCombat";
        "units";
        "tiles";
        "campaign";
        "gold";
        "factions";
        "groundItems";
        "combatRng";
      ]
    value.wire
    ([
       ("round", encode_json_number value.round);
       ("turn", encode_save35_setup_first value.turn);
       ("winner", encode_fragments_native_state38_winner value.winner);
       ("mapId", encode_json_string value.map_id);
       ("mapRevision", encode_json_number value.map_revision);
       ("cols", encode_json_number value.cols);
       ("rows", encode_json_number value.rows);
       ("nextId", encode_json_number value.next_id);
       ("log", encode_fragments_native_state38_log value.log);
       ("units", encode_save26_checkpoint_state_units value.units);
       ("tiles", encode_save26_checkpoint_state_tiles value.tiles);
       ("gold", encode_fragments_native_state38_gold value.gold);
       ("factions", encode_save26_checkpoint_state_factions value.factions);
       ("groundItems", encode_fragments_native_state38_ground_items value.ground_items);
     ]
    @ (match value.armies with
      | None ->
          if List.mem "armies" value.wire.undefined_fields then [ ("armies", undefined) ] else []
      | Some x -> [ ("armies", encode_fragments_native_state38_armies_member1 x) ])
    @ (match value.condition_memory with
      | None ->
          if List.mem "conditionMemory" value.wire.undefined_fields then
            [ ("conditionMemory", undefined) ]
          else []
      | Some x -> [ ("conditionMemory", encode_fragments_native_state38_log x) ])
    @ (match value.last_combat with
      | None ->
          if List.mem "lastCombat" value.wire.undefined_fields then [ ("lastCombat", undefined) ]
          else []
      | Some x -> [ ("lastCombat", encode_fragments_native_state38_last_combat x) ])
    @ (match value.campaign with
      | None ->
          if List.mem "campaign" value.wire.undefined_fields then [ ("campaign", undefined) ]
          else []
      | Some x -> [ ("campaign", encode_save26_checkpoint_state_campaign_member1 x) ])
    @
    match value.combat_rng with
    | None ->
        if List.mem "combatRng" value.wire.undefined_fields then [ ("combatRng", undefined) ]
        else []
    | Some x -> [ ("combatRng", encode_json_number x) ])

and encode_save26_checkpoint_state_units (value : save26_checkpoint_state_units) : Js.Json.t =
  Js.Json.array (Array.map encode_save26_checkpoint_state_units_item value)

and encode_save26_checkpoint_state_units_item (value : save26_checkpoint_state_units_item) :
    Js.Json.t =
  wire_to_json
    ~known:
      [
        "tile";
        "id";
        "side";
        "role";
        "faction";
        "items";
        "aiGroup";
        "type";
        "hp";
        "moved";
        "attacked";
        "retreating";
        "rebuilding";
        "character";
        "experience";
      ]
    value.wire
    ([
       ("tile", encode_json_string value.tile);
       ("id", encode_json_string value.id);
       ("side", encode_save35_setup_first value.side);
       ("type", encode_json_string value.type_);
       ("hp", encode_json_number value.hp);
       ("moved", encode_json_boolean value.moved);
       ("attacked", encode_json_boolean value.attacked);
     ]
    @ (match value.role with
      | None -> if List.mem "role" value.wire.undefined_fields then [ ("role", undefined) ] else []
      | Some x -> [ ("role", encode_fragments_native_state38_units_item_role x) ])
    @ (match value.faction with
      | None ->
          if List.mem "faction" value.wire.undefined_fields then [ ("faction", undefined) ] else []
      | Some x -> [ ("faction", encode_json_string x) ])
    @ (match value.items with
      | None ->
          if List.mem "items" value.wire.undefined_fields then [ ("items", undefined) ] else []
      | Some x -> [ ("items", encode_fragments_native_state38_units_item_items_member1 x) ])
    @ (match value.ai_group with
      | None ->
          if List.mem "aiGroup" value.wire.undefined_fields then [ ("aiGroup", undefined) ] else []
      | Some x -> [ ("aiGroup", encode_json_string x) ])
    @ (match value.retreating with
      | None ->
          if List.mem "retreating" value.wire.undefined_fields then [ ("retreating", undefined) ]
          else []
      | Some x -> [ ("retreating", encode_fragments_rule11_remember x) ])
    @ (match value.rebuilding with
      | None ->
          if List.mem "rebuilding" value.wire.undefined_fields then [ ("rebuilding", undefined) ]
          else []
      | Some x -> [ ("rebuilding", encode_fragments_rule11_remember x) ])
    @ (match value.character with
      | None ->
          if List.mem "character" value.wire.undefined_fields then [ ("character", undefined) ]
          else []
      | Some x -> [ ("character", encode_json_string x) ])
    @
    match value.experience with
    | None ->
        if List.mem "experience" value.wire.undefined_fields then [ ("experience", undefined) ]
        else []
    | Some x -> [ ("experience", encode_json_number x) ])

and encode_save26_checkpoint_state_tiles (value : save26_checkpoint_state_tiles) : Js.Json.t =
  Js.Json.array (Array.map encode_save26_checkpoint_state_tiles_item value)

and encode_save26_checkpoint_state_tiles_item (value : save26_checkpoint_state_tiles_item) :
    Js.Json.t =
  wire_to_json
    ~known:
      [
        "owner";
        "id";
        "location";
        "col";
        "row";
        "terrain";
        "connections";
        "deadEnds";
        "walls";
        "floor";
        "fog";
        "light";
      ]
    value.wire
    ([
       ("owner", encode_fragments_tiles11_item_owner value.owner);
       ("id", encode_json_string value.id);
       ("col", encode_json_number value.col);
       ("row", encode_json_number value.row);
       ("terrain", encode_json_string value.terrain);
     ]
    @ (match value.location with
      | None ->
          if List.mem "location" value.wire.undefined_fields then [ ("location", undefined) ]
          else []
      | Some x -> [ ("location", encode_json_string x) ])
    @ (match value.connections with
      | None ->
          if List.mem "connections" value.wire.undefined_fields then [ ("connections", undefined) ]
          else []
      | Some x -> [ ("connections", encode_fragments_tiles11_item_connections_member1 x) ])
    @ (match value.dead_ends with
      | None ->
          if List.mem "deadEnds" value.wire.undefined_fields then [ ("deadEnds", undefined) ]
          else []
      | Some x -> [ ("deadEnds", encode_fragments_tiles11_item_connections_member1 x) ])
    @ (match value.walls with
      | None ->
          if List.mem "walls" value.wire.undefined_fields then [ ("walls", undefined) ] else []
      | Some x -> [ ("walls", encode_fragments_tiles11_item_connections_member1 x) ])
    @ (match value.floor with
      | None ->
          if List.mem "floor" value.wire.undefined_fields then [ ("floor", undefined) ] else []
      | Some x -> [ ("floor", encode_json_string x) ])
    @ (match value.fog with
      | None -> if List.mem "fog" value.wire.undefined_fields then [ ("fog", undefined) ] else []
      | Some x -> [ ("fog", encode_fragments_light28_member1_when__member1 x) ])
    @
    match value.light with
    | None -> if List.mem "light" value.wire.undefined_fields then [ ("light", undefined) ] else []
    | Some x -> [ ("light", encode_fragments_light28_member1 x) ])

and encode_fragments_light28_member1_when_ (value : fragments_light28_member1_when_) : Js.Json.t =
  match value with
  | `Undefined -> undefined
  | `Object2 x -> encode_fragments_light28_member1_when__member1 x

and encode_fragments_light28_member1_when__member1 (value : fragments_light28_member1_when__member1)
    : Js.Json.t =
  wire_to_json ~known:[ "condition"; "remember" ] value.wire
    ([ ("condition", encode_fragments_light28_member1_when__member1_condition value.condition) ]
    @
    match value.remember with
    | None ->
        if List.mem "remember" value.wire.undefined_fields then [ ("remember", undefined) ] else []
    | Some x -> [ ("remember", encode_fragments_rule11_remember x) ])

and encode_fragments_light28_member1_when__member1_condition
    (value : fragments_light28_member1_when__member1_condition) : Js.Json.t =
  match value with
  | `Owner x -> encode_fragments_rule11_condition_member0 x
  | `Proximity x -> encode_fragments_light28_member1_when__member1_condition_member1 x
  | `Visible_units x -> encode_fragments_rule11_condition_member1 x
  | `Has_units x -> encode_fragments_light28_member1_when__member1_condition_member3 x
  | `No_units x -> encode_fragments_light28_member1_when__member1_condition_member4 x
  | `Choice x -> encode_fragments_rule11_condition_member2 x
  | `Flag x -> encode_fragments_rule11_condition_member3 x
  | `Event x -> encode_fragments_rule11_condition_member4 x
  | `Round x -> encode_fragments_rule11_condition_member5 x
  | `Object10 x -> encode_fragments_light28_member1_when__member1_condition_member9 x

and encode_fragments_light28_member1_when__member1_condition_member1
    (value : fragments_light28_member1_when__member1_condition_member1) : Js.Json.t =
  wire_to_json
    ~known:[ "kind"; "unitTypes"; "tile"; "radius"; "side"; "unitType"; "character" ]
    value.wire
    ([
       ("kind", encode_literal_proximity value.kind);
       ("radius", encode_json_number value.radius);
       ("side", encode_fragments_rule11_condition_member1_side value.side);
     ]
    @ (match value.unit_types with
      | None ->
          if List.mem "unitTypes" value.wire.undefined_fields then [ ("unitTypes", undefined) ]
          else []
      | Some x -> [ ("unitTypes", encode_fragments_native_state38_log x) ])
    @ (match value.tile with
      | None -> if List.mem "tile" value.wire.undefined_fields then [ ("tile", undefined) ] else []
      | Some x -> [ ("tile", encode_json_string x) ])
    @ (match value.unit_type with
      | None ->
          if List.mem "unitType" value.wire.undefined_fields then [ ("unitType", undefined) ]
          else []
      | Some x -> [ ("unitType", encode_json_string x) ])
    @
    match value.character with
    | None ->
        if List.mem "character" value.wire.undefined_fields then [ ("character", undefined) ]
        else []
    | Some x -> [ ("character", encode_json_string x) ])

and encode_fragments_light28_member1_when__member1_condition_member3
    (value : fragments_light28_member1_when__member1_condition_member3) : Js.Json.t =
  wire_to_json
    ~known:[ "kind"; "aiGroup"; "side"; "unitType"; "character" ]
    value.wire
    ([
       ("kind", encode_literal_has_units value.kind);
       ("side", encode_fragments_rule11_condition_member1_side value.side);
     ]
    @ (match value.ai_group with
      | None ->
          if List.mem "aiGroup" value.wire.undefined_fields then [ ("aiGroup", undefined) ] else []
      | Some x -> [ ("aiGroup", encode_json_string x) ])
    @ (match value.unit_type with
      | None ->
          if List.mem "unitType" value.wire.undefined_fields then [ ("unitType", undefined) ]
          else []
      | Some x -> [ ("unitType", encode_json_string x) ])
    @
    match value.character with
    | None ->
        if List.mem "character" value.wire.undefined_fields then [ ("character", undefined) ]
        else []
    | Some x -> [ ("character", encode_json_string x) ])

and encode_fragments_light28_member1_when__member1_condition_member4
    (value : fragments_light28_member1_when__member1_condition_member4) : Js.Json.t =
  wire_to_json
    ~known:[ "kind"; "side"; "unitType"; "character" ]
    value.wire
    ([
       ("kind", encode_literal_no_units value.kind);
       ("side", encode_fragments_rule11_condition_member1_side value.side);
     ]
    @ (match value.unit_type with
      | None ->
          if List.mem "unitType" value.wire.undefined_fields then [ ("unitType", undefined) ]
          else []
      | Some x -> [ ("unitType", encode_json_string x) ])
    @
    match value.character with
    | None ->
        if List.mem "character" value.wire.undefined_fields then [ ("character", undefined) ]
        else []
    | Some x -> [ ("character", encode_json_string x) ])

and encode_fragments_light28_member1_when__member1_condition_member9
    (value : fragments_light28_member1_when__member1_condition_member9) : Js.Json.t =
  wire_to_json ~known:[ "kind"; "conditions" ] value.wire
    [
      ("kind", encode_fragments_rule11_condition_member9_kind value.kind);
      ( "conditions",
        encode_fragments_light28_member1_when__member1_condition_member9_conditions value.conditions
      );
    ]

and encode_fragments_light28_member1_when__member1_condition_member9_conditions
    (value : fragments_light28_member1_when__member1_condition_member9_conditions) : Js.Json.t =
  Js.Json.array (Array.map encode_fragments_light28_member1_when__member1_condition value)

and encode_save26_checkpoint_state_tiles_item_light
    (value : save26_checkpoint_state_tiles_item_light) : Js.Json.t =
  match value with `Undefined -> undefined | `Object2 x -> encode_fragments_light28_member1 x

and encode_fragments_light28_member1 (value : fragments_light28_member1) : Js.Json.t =
  wire_to_json
    ~known:[ "radius"; "color"; "fixture"; "dx"; "dy"; "power"; "when" ]
    value.wire
    ([ ("radius", encode_json_number value.radius); ("power", encode_json_number value.power) ]
    @ (match value.color with
      | None ->
          if List.mem "color" value.wire.undefined_fields then [ ("color", undefined) ] else []
      | Some x -> [ ("color", encode_fragments_light23_color_member1 x) ])
    @ (match value.fixture with
      | None ->
          if List.mem "fixture" value.wire.undefined_fields then [ ("fixture", undefined) ] else []
      | Some x -> [ ("fixture", encode_fragments_rule11_remember x) ])
    @ (match value.dx with
      | None -> if List.mem "dx" value.wire.undefined_fields then [ ("dx", undefined) ] else []
      | Some x -> [ ("dx", encode_json_number x) ])
    @ (match value.dy with
      | None -> if List.mem "dy" value.wire.undefined_fields then [ ("dy", undefined) ] else []
      | Some x -> [ ("dy", encode_json_number x) ])
    @
    match value.when_ with
    | None -> if List.mem "when" value.wire.undefined_fields then [ ("when", undefined) ] else []
    | Some x -> [ ("when", encode_fragments_light28_member1_when__member1 x) ])

and encode_save26_checkpoint_state_campaign (value : save26_checkpoint_state_campaign) : Js.Json.t =
  match value with
  | `Undefined -> undefined
  | `Object2 x -> encode_save26_checkpoint_state_campaign_member1 x

and encode_save26_checkpoint_state_campaign_member1
    (value : save26_checkpoint_state_campaign_member1) : Js.Json.t =
  wire_to_json
    ~known:
      [
        "mission";
        "id";
        "vigil";
        "refuge";
        "completed";
        "marks";
        "holds";
        "recall";
        "fallenCompanions";
        "carryGold";
        "dialogue";
        "dialogueIndex";
        "outcome";
        "veterans";
        "storyVeterans";
        "evacuated";
        "choices";
      ]
    value.wire
    ([
       ("mission", encode_json_number value.mission);
       ("id", encode_json_string value.id);
       ("completed", encode_json_boolean value.completed);
       ("marks", encode_fragments_native_state38_log value.marks);
       ("recall", encode_fragments_native_state38_log value.recall);
       ("carryGold", encode_json_number value.carry_gold);
       ("dialogue", encode_fragments_native_state38_campaign_member1_dialogue value.dialogue);
       ("dialogueIndex", encode_json_number value.dialogue_index);
       ("outcome", encode_fragments_tiles11_item_owner value.outcome);
     ]
    @ (match value.vigil with
      | None ->
          if List.mem "vigil" value.wire.undefined_fields then [ ("vigil", undefined) ] else []
      | Some x -> [ ("vigil", encode_fragments_native_state38_campaign_member1_vigil_member1 x) ])
    @ (match value.refuge with
      | None ->
          if List.mem "refuge" value.wire.undefined_fields then [ ("refuge", undefined) ] else []
      | Some x -> [ ("refuge", encode_fragments_native_state38_campaign_member1_refuge_member1 x) ])
    @ (match value.holds with
      | None ->
          if List.mem "holds" value.wire.undefined_fields then [ ("holds", undefined) ] else []
      | Some x -> [ ("holds", encode_fragments_native_state38_campaign_member1_holds_member1 x) ])
    @ (match value.fallen_companions with
      | None ->
          if List.mem "fallenCompanions" value.wire.undefined_fields then
            [ ("fallenCompanions", undefined) ]
          else []
      | Some x -> [ ("fallenCompanions", encode_fragments_native_state38_log x) ])
    @ (match value.veterans with
      | None ->
          if List.mem "veterans" value.wire.undefined_fields then [ ("veterans", undefined) ]
          else []
      | Some x ->
          [ ("veterans", encode_save26_checkpoint_state_campaign_member1_veterans_member1 x) ])
    @ (match value.story_veterans with
      | None ->
          if List.mem "storyVeterans" value.wire.undefined_fields then
            [ ("storyVeterans", undefined) ]
          else []
      | Some x ->
          [ ("storyVeterans", encode_save26_checkpoint_state_campaign_member1_veterans_member1 x) ])
    @ (match value.evacuated with
      | None ->
          if List.mem "evacuated" value.wire.undefined_fields then [ ("evacuated", undefined) ]
          else []
      | Some x ->
          [ ("evacuated", encode_save26_checkpoint_state_campaign_member1_evacuated_member1 x) ])
    @
    match value.choices with
    | None ->
        if List.mem "choices" value.wire.undefined_fields then [ ("choices", undefined) ] else []
    | Some x -> [ ("choices", encode_fragments_native_state38_log x) ])

and encode_save26_checkpoint_state_campaign_member1_veterans
    (value : save26_checkpoint_state_campaign_member1_veterans) : Js.Json.t =
  match value with
  | `Undefined -> undefined
  | `Array x -> encode_save26_checkpoint_state_campaign_member1_veterans_member1 x

and encode_save26_checkpoint_state_campaign_member1_veterans_member1
    (value : save26_checkpoint_state_campaign_member1_veterans_member1) : Js.Json.t =
  Js.Json.array
    (Array.map encode_save26_checkpoint_state_campaign_member1_veterans_member1_item value)

and encode_save26_checkpoint_state_campaign_member1_veterans_member1_item
    (value : save26_checkpoint_state_campaign_member1_veterans_member1_item) : Js.Json.t =
  wire_to_json
    ~known:[ "role"; "experience"; "items"; "type"; "character" ]
    value.wire
    ([
       ("experience", encode_json_number value.experience); ("type", encode_json_string value.type_);
     ]
    @ (match value.role with
      | None -> if List.mem "role" value.wire.undefined_fields then [ ("role", undefined) ] else []
      | Some x -> [ ("role", encode_fragments_native_state38_units_item_role x) ])
    @ (match value.items with
      | None ->
          if List.mem "items" value.wire.undefined_fields then [ ("items", undefined) ] else []
      | Some x -> [ ("items", encode_fragments_native_state38_units_item_items_member1 x) ])
    @
    match value.character with
    | None ->
        if List.mem "character" value.wire.undefined_fields then [ ("character", undefined) ]
        else []
    | Some x -> [ ("character", encode_json_string x) ])

and encode_save26_checkpoint_state_campaign_member1_evacuated
    (value : save26_checkpoint_state_campaign_member1_evacuated) : Js.Json.t =
  match value with
  | `Undefined -> undefined
  | `Array x -> encode_save26_checkpoint_state_campaign_member1_evacuated_member1 x

and encode_save26_checkpoint_state_campaign_member1_evacuated_member1
    (value : save26_checkpoint_state_campaign_member1_evacuated_member1) : Js.Json.t =
  Js.Json.array
    (Array.map encode_save26_checkpoint_state_campaign_member1_evacuated_member1_item value)

and encode_save26_checkpoint_state_campaign_member1_evacuated_member1_item
    (value : save26_checkpoint_state_campaign_member1_evacuated_member1_item) : Js.Json.t =
  wire_to_json ~known:[ "event"; "unit" ] value.wire
    [
      ("event", encode_json_string value.event);
      ("unit", encode_save26_checkpoint_state_units_item value.unit_);
    ]

and encode_save26_checkpoint_state_factions (value : save26_checkpoint_state_factions) : Js.Json.t =
  Js.Json.array (Array.map encode_save26_checkpoint_state_factions_item value)

and encode_save26_checkpoint_state_factions_item (value : save26_checkpoint_state_factions_item) :
    Js.Json.t =
  wire_to_json
    ~known:[ "id"; "name"; "phase"; "controller"; "palette"; "recruits"; "relations"; "transfer" ]
    value.wire
    ([
       ("id", encode_json_string value.id);
       ("phase", encode_save35_setup_first value.phase);
       ("controller", encode_fragments_factions7_item_controller value.controller);
       ("palette", encode_json_string value.palette);
     ]
    @ (match value.name with
      | None -> if List.mem "name" value.wire.undefined_fields then [ ("name", undefined) ] else []
      | Some x -> [ ("name", encode_json_string x) ])
    @ (match value.recruits with
      | None ->
          if List.mem "recruits" value.wire.undefined_fields then [ ("recruits", undefined) ]
          else []
      | Some x -> [ ("recruits", encode_fragments_factions7_item_recruits_member1 x) ])
    @ (match value.relations with
      | None ->
          if List.mem "relations" value.wire.undefined_fields then [ ("relations", undefined) ]
          else []
      | Some x -> [ ("relations", encode_fragments_factions7_item_relations_member1 x) ])
    @
    match value.transfer with
    | None ->
        if List.mem "transfer" value.wire.undefined_fields then [ ("transfer", undefined) ] else []
    | Some x -> [ ("transfer", encode_save26_checkpoint_state_factions_item_transfer_member1 x) ])

and encode_save26_checkpoint_state_factions_item_transfer
    (value : save26_checkpoint_state_factions_item_transfer) : Js.Json.t =
  match value with
  | `Undefined -> undefined
  | `Object2 x -> encode_save26_checkpoint_state_factions_item_transfer_member1 x

and encode_save26_checkpoint_state_factions_item_transfer_member1
    (value : save26_checkpoint_state_factions_item_transfer_member1) : Js.Json.t =
  wire_to_json ~known:[ "to"; "when" ] value.wire
    [
      ("to", encode_json_string value.to_);
      ("when", encode_fragments_light28_member1_when__member1 value.when_);
    ]

and encode_save27 (value : save27) : Js.Json.t =
  wire_to_json
    ~known:[ "version"; "checkpoint"; "pendingEnemy"; "setup"; "progress"; "undo" ]
    value.wire
    ([
       ("version", encode_literal_27 value.version);
       ("checkpoint", encode_save26_checkpoint value.checkpoint);
       ("pendingEnemy", encode_json_boolean value.pending_enemy);
     ]
    @ (match value.setup with
      | None ->
          if List.mem "setup" value.wire.undefined_fields then [ ("setup", undefined) ] else []
      | Some x -> [ ("setup", encode_save2_setup_member1 x) ])
    @ (match value.progress with
      | None ->
          if List.mem "progress" value.wire.undefined_fields then [ ("progress", undefined) ]
          else []
      | Some x -> [ ("progress", encode_save2_progress_member1 x) ])
    @
    match value.undo with
    | None -> if List.mem "undo" value.wire.undefined_fields then [ ("undo", undefined) ] else []
    | Some x -> [ ("undo", encode_json_unknown x) ])

and encode_literal_27 (value : literal_27) : Js.Json.t = Js.Json.parseExn "27"

and encode_save28 (value : save28) : Js.Json.t =
  wire_to_json
    ~known:[ "version"; "checkpoint"; "pendingEnemy"; "setup"; "progress"; "undo" ]
    value.wire
    ([
       ("version", encode_literal_28 value.version);
       ("checkpoint", encode_save28_checkpoint value.checkpoint);
       ("pendingEnemy", encode_json_boolean value.pending_enemy);
     ]
    @ (match value.setup with
      | None ->
          if List.mem "setup" value.wire.undefined_fields then [ ("setup", undefined) ] else []
      | Some x -> [ ("setup", encode_save2_setup_member1 x) ])
    @ (match value.progress with
      | None ->
          if List.mem "progress" value.wire.undefined_fields then [ ("progress", undefined) ]
          else []
      | Some x -> [ ("progress", encode_save2_progress_member1 x) ])
    @
    match value.undo with
    | None -> if List.mem "undo" value.wire.undefined_fields then [ ("undo", undefined) ] else []
    | Some x -> [ ("undo", encode_json_unknown x) ])

and encode_literal_28 (value : literal_28) : Js.Json.t = Js.Json.parseExn "28"

and encode_save28_checkpoint (value : save28_checkpoint) : Js.Json.t =
  wire_to_json ~known:[ "seed"; "state" ] value.wire
    [
      ("seed", encode_json_number value.seed); ("state", encode_save28_checkpoint_state value.state);
    ]

and encode_save28_checkpoint_state (value : save28_checkpoint_state) : Js.Json.t =
  wire_to_json
    ~known:
      [
        "round";
        "turn";
        "winner";
        "armies";
        "conditionMemory";
        "mapId";
        "mapRevision";
        "cols";
        "rows";
        "nextId";
        "log";
        "lastCombat";
        "units";
        "tiles";
        "campaign";
        "gold";
        "factions";
        "groundItems";
        "combatRng";
      ]
    value.wire
    ([
       ("round", encode_json_number value.round);
       ("turn", encode_save35_setup_first value.turn);
       ("winner", encode_fragments_native_state38_winner value.winner);
       ("mapId", encode_json_string value.map_id);
       ("mapRevision", encode_json_number value.map_revision);
       ("cols", encode_json_number value.cols);
       ("rows", encode_json_number value.rows);
       ("nextId", encode_json_number value.next_id);
       ("log", encode_fragments_native_state38_log value.log);
       ("units", encode_save26_checkpoint_state_units value.units);
       ("tiles", encode_save26_checkpoint_state_tiles value.tiles);
       ("gold", encode_fragments_native_state38_gold value.gold);
       ("factions", encode_save26_checkpoint_state_factions value.factions);
       ("groundItems", encode_fragments_native_state38_ground_items value.ground_items);
     ]
    @ (match value.armies with
      | None ->
          if List.mem "armies" value.wire.undefined_fields then [ ("armies", undefined) ] else []
      | Some x -> [ ("armies", encode_fragments_native_state38_armies_member1 x) ])
    @ (match value.condition_memory with
      | None ->
          if List.mem "conditionMemory" value.wire.undefined_fields then
            [ ("conditionMemory", undefined) ]
          else []
      | Some x -> [ ("conditionMemory", encode_fragments_native_state38_log x) ])
    @ (match value.last_combat with
      | None ->
          if List.mem "lastCombat" value.wire.undefined_fields then [ ("lastCombat", undefined) ]
          else []
      | Some x -> [ ("lastCombat", encode_fragments_native_state38_last_combat x) ])
    @ (match value.campaign with
      | None ->
          if List.mem "campaign" value.wire.undefined_fields then [ ("campaign", undefined) ]
          else []
      | Some x -> [ ("campaign", encode_save28_checkpoint_state_campaign_member1 x) ])
    @
    match value.combat_rng with
    | None ->
        if List.mem "combatRng" value.wire.undefined_fields then [ ("combatRng", undefined) ]
        else []
    | Some x -> [ ("combatRng", encode_json_number x) ])

and encode_save28_checkpoint_state_campaign (value : save28_checkpoint_state_campaign) : Js.Json.t =
  match value with
  | `Undefined -> undefined
  | `Object2 x -> encode_save28_checkpoint_state_campaign_member1 x

and encode_save28_checkpoint_state_campaign_member1
    (value : save28_checkpoint_state_campaign_member1) : Js.Json.t =
  wire_to_json
    ~known:
      [
        "mission";
        "id";
        "vigil";
        "refuge";
        "completed";
        "marks";
        "holds";
        "recall";
        "fallenCompanions";
        "carryGold";
        "dialogue";
        "dialogueIndex";
        "outcome";
        "veterans";
        "storyVeterans";
        "evacuated";
        "scripts";
        "choices";
      ]
    value.wire
    ([
       ("mission", encode_json_number value.mission);
       ("id", encode_json_string value.id);
       ("completed", encode_json_boolean value.completed);
       ("marks", encode_fragments_native_state38_log value.marks);
       ("recall", encode_fragments_native_state38_log value.recall);
       ("carryGold", encode_json_number value.carry_gold);
       ("dialogue", encode_fragments_native_state38_campaign_member1_dialogue value.dialogue);
       ("dialogueIndex", encode_json_number value.dialogue_index);
       ("outcome", encode_fragments_tiles11_item_owner value.outcome);
     ]
    @ (match value.vigil with
      | None ->
          if List.mem "vigil" value.wire.undefined_fields then [ ("vigil", undefined) ] else []
      | Some x -> [ ("vigil", encode_fragments_native_state38_campaign_member1_vigil_member1 x) ])
    @ (match value.refuge with
      | None ->
          if List.mem "refuge" value.wire.undefined_fields then [ ("refuge", undefined) ] else []
      | Some x -> [ ("refuge", encode_fragments_native_state38_campaign_member1_refuge_member1 x) ])
    @ (match value.holds with
      | None ->
          if List.mem "holds" value.wire.undefined_fields then [ ("holds", undefined) ] else []
      | Some x -> [ ("holds", encode_fragments_native_state38_campaign_member1_holds_member1 x) ])
    @ (match value.fallen_companions with
      | None ->
          if List.mem "fallenCompanions" value.wire.undefined_fields then
            [ ("fallenCompanions", undefined) ]
          else []
      | Some x -> [ ("fallenCompanions", encode_fragments_native_state38_log x) ])
    @ (match value.veterans with
      | None ->
          if List.mem "veterans" value.wire.undefined_fields then [ ("veterans", undefined) ]
          else []
      | Some x ->
          [ ("veterans", encode_save26_checkpoint_state_campaign_member1_veterans_member1 x) ])
    @ (match value.story_veterans with
      | None ->
          if List.mem "storyVeterans" value.wire.undefined_fields then
            [ ("storyVeterans", undefined) ]
          else []
      | Some x ->
          [ ("storyVeterans", encode_save26_checkpoint_state_campaign_member1_veterans_member1 x) ])
    @ (match value.evacuated with
      | None ->
          if List.mem "evacuated" value.wire.undefined_fields then [ ("evacuated", undefined) ]
          else []
      | Some x ->
          [ ("evacuated", encode_save26_checkpoint_state_campaign_member1_evacuated_member1 x) ])
    @ (match value.scripts with
      | None ->
          if List.mem "scripts" value.wire.undefined_fields then [ ("scripts", undefined) ] else []
      | Some x -> [ ("scripts", encode_save28_checkpoint_state_campaign_member1_scripts_member1 x) ])
    @
    match value.choices with
    | None ->
        if List.mem "choices" value.wire.undefined_fields then [ ("choices", undefined) ] else []
    | Some x -> [ ("choices", encode_fragments_native_state38_log x) ])

and encode_save28_checkpoint_state_campaign_member1_scripts
    (value : save28_checkpoint_state_campaign_member1_scripts) : Js.Json.t =
  match value with
  | `Undefined -> undefined
  | `Object2 x -> encode_save28_checkpoint_state_campaign_member1_scripts_member1 x

and encode_save28_checkpoint_state_campaign_member1_scripts_member1
    (value : save28_checkpoint_state_campaign_member1_scripts_member1) : Js.Json.t =
  wire_to_json
    ~known:
      [
        "recruitment";
        "failed";
        "active";
        "holds";
        "nextMark";
        "pending";
        "marks";
        "completed";
        "revealed";
      ]
    value.wire
    ([
       ( "recruitment",
         encode_fragments_native_state38_campaign_member1_scripts_member1_recruitment
           value.recruitment );
       ("active", encode_fragments_native_state38_log value.active);
       ("holds", encode_fragments_native_state38_campaign_member1_scripts_member1_holds value.holds);
       ("nextMark", encode_json_number value.next_mark);
       ( "pending",
         encode_fragments_native_state38_campaign_member1_scripts_member1_pending value.pending );
       ("marks", encode_save28_checkpoint_state_campaign_member1_scripts_member1_marks value.marks);
     ]
    @ (match value.failed with
      | None ->
          if List.mem "failed" value.wire.undefined_fields then [ ("failed", undefined) ] else []
      | Some x -> [ ("failed", encode_fragments_native_state38_log x) ])
    @ (match value.completed with
      | None ->
          if List.mem "completed" value.wire.undefined_fields then [ ("completed", undefined) ]
          else []
      | Some x -> [ ("completed", encode_fragments_native_state38_log x) ])
    @
    match value.revealed with
    | None ->
        if List.mem "revealed" value.wire.undefined_fields then [ ("revealed", undefined) ] else []
    | Some x -> [ ("revealed", encode_fragments_native_state38_log x) ])

and encode_fragments_native_state38_campaign_member1_scripts_member1_recruitment
    (value : fragments_native_state38_campaign_member1_scripts_member1_recruitment) : Js.Json.t =
  map_to_json encode_fragments_factions7_item_recruits_member1 value

and encode_fragments_native_state38_campaign_member1_scripts_member1_holds
    (value : fragments_native_state38_campaign_member1_scripts_member1_holds) : Js.Json.t =
  Js.Json.array
    (Array.map encode_fragments_native_state38_campaign_member1_scripts_member1_holds_item value)

and encode_fragments_native_state38_campaign_member1_scripts_member1_holds_item
    (value : fragments_native_state38_campaign_member1_scripts_member1_holds_item) : Js.Json.t =
  wire_to_json
    ~known:[ "event"; "predicate"; "turns"; "lastRound" ]
    value.wire
    [
      ("event", encode_json_string value.event);
      ("predicate", encode_json_number value.predicate);
      ("turns", encode_json_number value.turns);
      ("lastRound", encode_json_number value.last_round);
    ]

and encode_fragments_native_state38_campaign_member1_scripts_member1_pending
    (value : fragments_native_state38_campaign_member1_scripts_member1_pending) : Js.Json.t =
  Js.Json.array
    (Array.map encode_fragments_native_state38_campaign_member1_scripts_member1_pending_item value)

and encode_fragments_native_state38_campaign_member1_scripts_member1_pending_item
    (value : fragments_native_state38_campaign_member1_scripts_member1_pending_item) : Js.Json.t =
  wire_to_json
    ~known:[ "area"; "faction"; "aiGroup"; "units" ]
    value.wire
    ([
       ("area", encode_fragments_native_state38_log value.area);
       ("faction", encode_json_string value.faction);
       ( "units",
         encode_fragments_native_state38_campaign_member1_scripts_member1_pending_item_units
           value.units );
     ]
    @
    match value.ai_group with
    | None ->
        if List.mem "aiGroup" value.wire.undefined_fields then [ ("aiGroup", undefined) ] else []
    | Some x -> [ ("aiGroup", encode_fragments_delta19_item_owner x) ])

and encode_fragments_delta19_item_owner (value : fragments_delta19_item_owner) : Js.Json.t =
  match value with
  | `Undefined -> undefined
  | `Null -> json_null
  | `String x -> encode_json_string x

and encode_fragments_native_state38_campaign_member1_scripts_member1_pending_item_units
    (value : fragments_native_state38_campaign_member1_scripts_member1_pending_item_units) :
    Js.Json.t =
  Js.Json.array
    (Array.map
       encode_fragments_native_state38_campaign_member1_scripts_member1_pending_item_units_item
       value)

and encode_fragments_native_state38_campaign_member1_scripts_member1_pending_item_units_item
    (value : fragments_native_state38_campaign_member1_scripts_member1_pending_item_units_item) :
    Js.Json.t =
  wire_to_json
    ~known:[ "type"; "character"; "role"; "items" ]
    value.wire
    ([ ("type", encode_json_string value.type_) ]
    @ (match value.character with
      | None ->
          if List.mem "character" value.wire.undefined_fields then [ ("character", undefined) ]
          else []
      | Some x -> [ ("character", encode_fragments_delta19_item_owner x) ])
    @ (match value.role with
      | None -> if List.mem "role" value.wire.undefined_fields then [ ("role", undefined) ] else []
      | Some x ->
          [
            ( "role",
              encode_fragments_native_state38_campaign_member1_scripts_member1_pending_item_units_item_role
                x );
          ])
    @
    match value.items with
    | None -> if List.mem "items" value.wire.undefined_fields then [ ("items", undefined) ] else []
    | Some x -> [ ("items", encode_fragments_native_state38_log x) ])

and encode_fragments_native_state38_campaign_member1_scripts_member1_pending_item_units_item_role
    (value : fragments_native_state38_campaign_member1_scripts_member1_pending_item_units_item_role)
    : Js.Json.t =
  match value with
  | `Undefined -> undefined
  | `Null -> json_null
  | `Leader -> Js.Json.parseExn "\"leader\""
  | `Courier -> Js.Json.parseExn "\"courier\""
  | `Captive -> Js.Json.parseExn "\"captive\""

and encode_save28_checkpoint_state_campaign_member1_scripts_member1_marks
    (value : save28_checkpoint_state_campaign_member1_scripts_member1_marks) : Js.Json.t =
  Js.Json.array
    (Array.map encode_save28_checkpoint_state_campaign_member1_scripts_member1_marks_item value)

and encode_save28_checkpoint_state_campaign_member1_scripts_member1_marks_item
    (value : save28_checkpoint_state_campaign_member1_scripts_member1_marks_item) : Js.Json.t =
  wire_to_json
    ~known:[ "id"; "type"; "tile"; "light"; "lightRemembered" ]
    value.wire
    ([
       ("id", encode_json_number value.id);
       ("type", encode_json_string value.type_);
       ("tile", encode_json_string value.tile);
     ]
    @ (match value.light with
      | None ->
          if List.mem "light" value.wire.undefined_fields then [ ("light", undefined) ] else []
      | Some x -> [ ("light", encode_fragments_light28_member1 x) ])
    @
    match value.light_remembered with
    | None ->
        if List.mem "lightRemembered" value.wire.undefined_fields then
          [ ("lightRemembered", undefined) ]
        else []
    | Some x -> [ ("lightRemembered", encode_fragments_rule11_remember x) ])

and encode_save29 (value : save29) : Js.Json.t =
  wire_to_json
    ~known:[ "version"; "checkpoint"; "pendingEnemy"; "setup"; "progress"; "undo" ]
    value.wire
    ([
       ("version", encode_literal_29 value.version);
       ("checkpoint", encode_save29_checkpoint value.checkpoint);
       ("pendingEnemy", encode_json_boolean value.pending_enemy);
     ]
    @ (match value.setup with
      | None ->
          if List.mem "setup" value.wire.undefined_fields then [ ("setup", undefined) ] else []
      | Some x -> [ ("setup", encode_save2_setup_member1 x) ])
    @ (match value.progress with
      | None ->
          if List.mem "progress" value.wire.undefined_fields then [ ("progress", undefined) ]
          else []
      | Some x -> [ ("progress", encode_save2_progress_member1 x) ])
    @
    match value.undo with
    | None -> if List.mem "undo" value.wire.undefined_fields then [ ("undo", undefined) ] else []
    | Some x -> [ ("undo", encode_json_unknown x) ])

and encode_literal_29 (value : literal_29) : Js.Json.t = Js.Json.parseExn "29"

and encode_save29_checkpoint (value : save29_checkpoint) : Js.Json.t =
  wire_to_json ~known:[ "seed"; "state" ] value.wire
    [
      ("seed", encode_json_number value.seed); ("state", encode_save29_checkpoint_state value.state);
    ]

and encode_save29_checkpoint_state (value : save29_checkpoint_state) : Js.Json.t =
  wire_to_json
    ~known:
      [
        "round";
        "turn";
        "winner";
        "armies";
        "conditionMemory";
        "mapId";
        "mapRevision";
        "cols";
        "rows";
        "nextId";
        "log";
        "lastCombat";
        "units";
        "tiles";
        "campaign";
        "gold";
        "factions";
        "groundItems";
        "combatRng";
      ]
    value.wire
    ([
       ("round", encode_json_number value.round);
       ("turn", encode_save35_setup_first value.turn);
       ("winner", encode_fragments_native_state38_winner value.winner);
       ("mapId", encode_json_string value.map_id);
       ("mapRevision", encode_json_number value.map_revision);
       ("cols", encode_json_number value.cols);
       ("rows", encode_json_number value.rows);
       ("nextId", encode_json_number value.next_id);
       ("log", encode_fragments_native_state38_log value.log);
       ("units", encode_save26_checkpoint_state_units value.units);
       ("tiles", encode_save26_checkpoint_state_tiles value.tiles);
       ("gold", encode_fragments_native_state38_gold value.gold);
       ("factions", encode_save26_checkpoint_state_factions value.factions);
       ("groundItems", encode_fragments_native_state38_ground_items value.ground_items);
     ]
    @ (match value.armies with
      | None ->
          if List.mem "armies" value.wire.undefined_fields then [ ("armies", undefined) ] else []
      | Some x -> [ ("armies", encode_fragments_native_state38_armies_member1 x) ])
    @ (match value.condition_memory with
      | None ->
          if List.mem "conditionMemory" value.wire.undefined_fields then
            [ ("conditionMemory", undefined) ]
          else []
      | Some x -> [ ("conditionMemory", encode_fragments_native_state38_log x) ])
    @ (match value.last_combat with
      | None ->
          if List.mem "lastCombat" value.wire.undefined_fields then [ ("lastCombat", undefined) ]
          else []
      | Some x -> [ ("lastCombat", encode_fragments_native_state38_last_combat x) ])
    @ (match value.campaign with
      | None ->
          if List.mem "campaign" value.wire.undefined_fields then [ ("campaign", undefined) ]
          else []
      | Some x -> [ ("campaign", encode_save29_checkpoint_state_campaign_member1 x) ])
    @
    match value.combat_rng with
    | None ->
        if List.mem "combatRng" value.wire.undefined_fields then [ ("combatRng", undefined) ]
        else []
    | Some x -> [ ("combatRng", encode_json_number x) ])

and encode_save29_checkpoint_state_campaign (value : save29_checkpoint_state_campaign) : Js.Json.t =
  match value with
  | `Undefined -> undefined
  | `Object2 x -> encode_save29_checkpoint_state_campaign_member1 x

and encode_save29_checkpoint_state_campaign_member1
    (value : save29_checkpoint_state_campaign_member1) : Js.Json.t =
  wire_to_json
    ~known:
      [
        "mission";
        "id";
        "vigil";
        "refuge";
        "completed";
        "marks";
        "holds";
        "recall";
        "fallenCompanions";
        "carryGold";
        "dialogue";
        "dialogueIndex";
        "outcome";
        "veterans";
        "storyVeterans";
        "evacuated";
        "scripts";
        "flags";
      ]
    value.wire
    ([
       ("mission", encode_json_number value.mission);
       ("id", encode_json_string value.id);
       ("completed", encode_json_boolean value.completed);
       ("marks", encode_fragments_native_state38_log value.marks);
       ("recall", encode_fragments_native_state38_log value.recall);
       ("carryGold", encode_json_number value.carry_gold);
       ("dialogue", encode_fragments_native_state38_campaign_member1_dialogue value.dialogue);
       ("dialogueIndex", encode_json_number value.dialogue_index);
       ("outcome", encode_fragments_tiles11_item_owner value.outcome);
       ("flags", encode_fragments_native_state38_campaign_member1_flags value.flags);
     ]
    @ (match value.vigil with
      | None ->
          if List.mem "vigil" value.wire.undefined_fields then [ ("vigil", undefined) ] else []
      | Some x -> [ ("vigil", encode_fragments_native_state38_campaign_member1_vigil_member1 x) ])
    @ (match value.refuge with
      | None ->
          if List.mem "refuge" value.wire.undefined_fields then [ ("refuge", undefined) ] else []
      | Some x -> [ ("refuge", encode_fragments_native_state38_campaign_member1_refuge_member1 x) ])
    @ (match value.holds with
      | None ->
          if List.mem "holds" value.wire.undefined_fields then [ ("holds", undefined) ] else []
      | Some x -> [ ("holds", encode_fragments_native_state38_campaign_member1_holds_member1 x) ])
    @ (match value.fallen_companions with
      | None ->
          if List.mem "fallenCompanions" value.wire.undefined_fields then
            [ ("fallenCompanions", undefined) ]
          else []
      | Some x -> [ ("fallenCompanions", encode_fragments_native_state38_log x) ])
    @ (match value.veterans with
      | None ->
          if List.mem "veterans" value.wire.undefined_fields then [ ("veterans", undefined) ]
          else []
      | Some x ->
          [ ("veterans", encode_save26_checkpoint_state_campaign_member1_veterans_member1 x) ])
    @ (match value.story_veterans with
      | None ->
          if List.mem "storyVeterans" value.wire.undefined_fields then
            [ ("storyVeterans", undefined) ]
          else []
      | Some x ->
          [ ("storyVeterans", encode_save26_checkpoint_state_campaign_member1_veterans_member1 x) ])
    @ (match value.evacuated with
      | None ->
          if List.mem "evacuated" value.wire.undefined_fields then [ ("evacuated", undefined) ]
          else []
      | Some x ->
          [ ("evacuated", encode_save26_checkpoint_state_campaign_member1_evacuated_member1 x) ])
    @
    match value.scripts with
    | None ->
        if List.mem "scripts" value.wire.undefined_fields then [ ("scripts", undefined) ] else []
    | Some x -> [ ("scripts", encode_save28_checkpoint_state_campaign_member1_scripts_member1 x) ])

and encode_fragments_native_state38_campaign_member1_flags
    (value : fragments_native_state38_campaign_member1_flags) : Js.Json.t =
  map_to_json encode_fragments_native_state38_campaign_member1_flags_value value

and encode_fragments_native_state38_campaign_member1_flags_value
    (value : fragments_native_state38_campaign_member1_flags_value) : Js.Json.t =
  Js.Json.array (Array.map encode_fragments_native_state38_campaign_member1_flags_value_item value)

and encode_fragments_native_state38_campaign_member1_flags_value_item
    (value : fragments_native_state38_campaign_member1_flags_value_item) : Js.Json.t =
  wire_to_json ~known:[ "state"; "stage"; "turn" ] value.wire
    [
      ("state", encode_json_boolean value.state);
      ("stage", encode_json_number value.stage);
      ("turn", encode_json_number value.turn);
    ]

and encode_save30 (value : save30) : Js.Json.t =
  wire_to_json
    ~known:[ "version"; "checkpoint"; "pendingEnemy"; "setup"; "progress"; "undo" ]
    value.wire
    ([
       ("version", encode_literal_30 value.version);
       ("checkpoint", encode_save29_checkpoint value.checkpoint);
       ("pendingEnemy", encode_json_boolean value.pending_enemy);
     ]
    @ (match value.setup with
      | None ->
          if List.mem "setup" value.wire.undefined_fields then [ ("setup", undefined) ] else []
      | Some x -> [ ("setup", encode_save35_setup x) ])
    @ (match value.progress with
      | None ->
          if List.mem "progress" value.wire.undefined_fields then [ ("progress", undefined) ]
          else []
      | Some x -> [ ("progress", encode_save2_progress_member1 x) ])
    @
    match value.undo with
    | None -> if List.mem "undo" value.wire.undefined_fields then [ ("undo", undefined) ] else []
    | Some x -> [ ("undo", encode_json_unknown x) ])

and encode_literal_30 (value : literal_30) : Js.Json.t = Js.Json.parseExn "30"

and encode_save30_setup (value : save30_setup) : Js.Json.t =
  match value with `Undefined -> undefined | `Object2 x -> encode_save35_setup x

and encode_save35_setup (value : save35_setup) : Js.Json.t =
  wire_to_json ~known:[ "first"; "enemy" ] value.wire
    [
      ("first", encode_save35_setup_first value.first);
      ("enemy", encode_save35_setup_enemy value.enemy);
    ]

and encode_save35_setup_enemy (value : save35_setup_enemy) : Js.Json.t =
  wire_to_json
    ~known:[ "policy"; "blocks"; "recruitmentLimit"; "diplomacy" ]
    value.wire
    ([
       ("policy", encode_save35_setup_enemy_policy value.policy);
       ("diplomacy", encode_save35_setup_enemy_diplomacy value.diplomacy);
     ]
    @ (match value.blocks with
      | None ->
          if List.mem "blocks" value.wire.undefined_fields then [ ("blocks", undefined) ] else []
      | Some x -> [ ("blocks", encode_save35_setup_enemy_blocks_member1 x) ])
    @
    match value.recruitment_limit with
    | None ->
        if List.mem "recruitmentLimit" value.wire.undefined_fields then
          [ ("recruitmentLimit", undefined) ]
        else []
    | Some x -> [ ("recruitmentLimit", encode_json_number x) ])

and encode_save35_setup_enemy_diplomacy (value : save35_setup_enemy_diplomacy) : Js.Json.t =
  match value with
  | `Diplomatic -> Js.Json.parseExn "\"diplomatic\""
  | `Unrestricted -> Js.Json.parseExn "\"unrestricted\""

and encode_save31 (value : save31) : Js.Json.t =
  wire_to_json
    ~known:[ "version"; "checkpoint"; "pendingEnemy"; "setup"; "progress"; "undo" ]
    value.wire
    ([
       ("version", encode_literal_31 value.version);
       ("checkpoint", encode_save29_checkpoint value.checkpoint);
       ("pendingEnemy", encode_json_boolean value.pending_enemy);
     ]
    @ (match value.setup with
      | None ->
          if List.mem "setup" value.wire.undefined_fields then [ ("setup", undefined) ] else []
      | Some x -> [ ("setup", encode_save35_setup x) ])
    @ (match value.progress with
      | None ->
          if List.mem "progress" value.wire.undefined_fields then [ ("progress", undefined) ]
          else []
      | Some x -> [ ("progress", encode_save2_progress_member1 x) ])
    @
    match value.undo with
    | None -> if List.mem "undo" value.wire.undefined_fields then [ ("undo", undefined) ] else []
    | Some x -> [ ("undo", encode_json_unknown x) ])

and encode_literal_31 (value : literal_31) : Js.Json.t = Js.Json.parseExn "31"

and encode_save32 (value : save32) : Js.Json.t =
  wire_to_json
    ~known:[ "version"; "checkpoint"; "pendingEnemy"; "setup"; "progress"; "undo" ]
    value.wire
    ([
       ("version", encode_literal_32 value.version);
       ("checkpoint", encode_save29_checkpoint value.checkpoint);
       ("pendingEnemy", encode_json_boolean value.pending_enemy);
     ]
    @ (match value.setup with
      | None ->
          if List.mem "setup" value.wire.undefined_fields then [ ("setup", undefined) ] else []
      | Some x -> [ ("setup", encode_save35_setup x) ])
    @ (match value.progress with
      | None ->
          if List.mem "progress" value.wire.undefined_fields then [ ("progress", undefined) ]
          else []
      | Some x -> [ ("progress", encode_save2_progress_member1 x) ])
    @
    match value.undo with
    | None -> if List.mem "undo" value.wire.undefined_fields then [ ("undo", undefined) ] else []
    | Some x -> [ ("undo", encode_json_unknown x) ])

and encode_literal_32 (value : literal_32) : Js.Json.t = Js.Json.parseExn "32"

and encode_save33 (value : save33) : Js.Json.t =
  wire_to_json
    ~known:[ "version"; "checkpoint"; "pendingEnemy"; "setup"; "progress"; "undo" ]
    value.wire
    ([
       ("version", encode_literal_33 value.version);
       ("checkpoint", encode_save29_checkpoint value.checkpoint);
       ("pendingEnemy", encode_json_boolean value.pending_enemy);
     ]
    @ (match value.setup with
      | None ->
          if List.mem "setup" value.wire.undefined_fields then [ ("setup", undefined) ] else []
      | Some x -> [ ("setup", encode_save35_setup x) ])
    @ (match value.progress with
      | None ->
          if List.mem "progress" value.wire.undefined_fields then [ ("progress", undefined) ]
          else []
      | Some x -> [ ("progress", encode_save2_progress_member1 x) ])
    @
    match value.undo with
    | None -> if List.mem "undo" value.wire.undefined_fields then [ ("undo", undefined) ] else []
    | Some x -> [ ("undo", encode_json_unknown x) ])

and encode_literal_33 (value : literal_33) : Js.Json.t = Js.Json.parseExn "33"

and encode_save34 (value : save34) : Js.Json.t =
  wire_to_json
    ~known:[ "version"; "checkpoint"; "pendingEnemy"; "setup"; "progress"; "undo" ]
    value.wire
    ([
       ("version", encode_literal_34 value.version);
       ("checkpoint", encode_save34_checkpoint value.checkpoint);
       ("pendingEnemy", encode_json_boolean value.pending_enemy);
     ]
    @ (match value.setup with
      | None ->
          if List.mem "setup" value.wire.undefined_fields then [ ("setup", undefined) ] else []
      | Some x -> [ ("setup", encode_save35_setup x) ])
    @ (match value.progress with
      | None ->
          if List.mem "progress" value.wire.undefined_fields then [ ("progress", undefined) ]
          else []
      | Some x -> [ ("progress", encode_save2_progress_member1 x) ])
    @
    match value.undo with
    | None -> if List.mem "undo" value.wire.undefined_fields then [ ("undo", undefined) ] else []
    | Some x -> [ ("undo", encode_save34_undo_member1 x) ])

and encode_literal_34 (value : literal_34) : Js.Json.t = Js.Json.parseExn "34"

and encode_save34_checkpoint (value : save34_checkpoint) : Js.Json.t =
  wire_to_json ~known:[ "seed"; "state" ] value.wire
    [
      ("seed", encode_json_number value.seed); ("state", encode_save34_checkpoint_state value.state);
    ]

and encode_save34_checkpoint_state (value : save34_checkpoint_state) : Js.Json.t =
  wire_to_json
    ~known:
      [
        "round";
        "turn";
        "winner";
        "armies";
        "conditionMemory";
        "mapId";
        "mapRevision";
        "cols";
        "rows";
        "nextId";
        "log";
        "lastCombat";
        "units";
        "tiles";
        "campaign";
        "gold";
        "factions";
        "groundItems";
        "combatRng";
      ]
    value.wire
    ([
       ("round", encode_json_number value.round);
       ("turn", encode_save35_setup_first value.turn);
       ("winner", encode_fragments_native_state38_winner value.winner);
       ("mapId", encode_json_string value.map_id);
       ("mapRevision", encode_json_number value.map_revision);
       ("cols", encode_json_number value.cols);
       ("rows", encode_json_number value.rows);
       ("nextId", encode_json_number value.next_id);
       ("log", encode_fragments_native_state38_log value.log);
       ("units", encode_save26_checkpoint_state_units value.units);
       ("tiles", encode_save26_checkpoint_state_tiles value.tiles);
       ("gold", encode_fragments_native_state38_gold value.gold);
       ("factions", encode_save26_checkpoint_state_factions value.factions);
       ("groundItems", encode_fragments_native_state38_ground_items value.ground_items);
     ]
    @ (match value.armies with
      | None ->
          if List.mem "armies" value.wire.undefined_fields then [ ("armies", undefined) ] else []
      | Some x -> [ ("armies", encode_fragments_native_state38_armies_member1 x) ])
    @ (match value.condition_memory with
      | None ->
          if List.mem "conditionMemory" value.wire.undefined_fields then
            [ ("conditionMemory", undefined) ]
          else []
      | Some x -> [ ("conditionMemory", encode_fragments_native_state38_log x) ])
    @ (match value.last_combat with
      | None ->
          if List.mem "lastCombat" value.wire.undefined_fields then [ ("lastCombat", undefined) ]
          else []
      | Some x -> [ ("lastCombat", encode_fragments_native_state38_last_combat x) ])
    @ (match value.campaign with
      | None ->
          if List.mem "campaign" value.wire.undefined_fields then [ ("campaign", undefined) ]
          else []
      | Some x -> [ ("campaign", encode_save34_checkpoint_state_campaign_member1 x) ])
    @
    match value.combat_rng with
    | None ->
        if List.mem "combatRng" value.wire.undefined_fields then [ ("combatRng", undefined) ]
        else []
    | Some x -> [ ("combatRng", encode_json_number x) ])

and encode_save34_checkpoint_state_campaign (value : save34_checkpoint_state_campaign) : Js.Json.t =
  match value with
  | `Undefined -> undefined
  | `Object2 x -> encode_save34_checkpoint_state_campaign_member1 x

and encode_save34_checkpoint_state_campaign_member1
    (value : save34_checkpoint_state_campaign_member1) : Js.Json.t =
  wire_to_json
    ~known:
      [
        "mission";
        "id";
        "vigil";
        "refuge";
        "completed";
        "marks";
        "holds";
        "recall";
        "fallenCompanions";
        "carryGold";
        "dialogue";
        "dialogueIndex";
        "outcome";
        "veterans";
        "storyVeterans";
        "evacuated";
        "scripts";
        "flags";
      ]
    value.wire
    ([
       ("mission", encode_json_number value.mission);
       ("id", encode_json_string value.id);
       ("completed", encode_json_boolean value.completed);
       ("marks", encode_fragments_native_state38_log value.marks);
       ("recall", encode_fragments_native_state38_log value.recall);
       ("carryGold", encode_json_number value.carry_gold);
       ("dialogue", encode_fragments_native_state38_campaign_member1_dialogue value.dialogue);
       ("dialogueIndex", encode_json_number value.dialogue_index);
       ("outcome", encode_fragments_tiles11_item_owner value.outcome);
       ("flags", encode_fragments_native_state38_campaign_member1_flags value.flags);
     ]
    @ (match value.vigil with
      | None ->
          if List.mem "vigil" value.wire.undefined_fields then [ ("vigil", undefined) ] else []
      | Some x -> [ ("vigil", encode_fragments_native_state38_campaign_member1_vigil_member1 x) ])
    @ (match value.refuge with
      | None ->
          if List.mem "refuge" value.wire.undefined_fields then [ ("refuge", undefined) ] else []
      | Some x -> [ ("refuge", encode_fragments_native_state38_campaign_member1_refuge_member1 x) ])
    @ (match value.holds with
      | None ->
          if List.mem "holds" value.wire.undefined_fields then [ ("holds", undefined) ] else []
      | Some x -> [ ("holds", encode_fragments_native_state38_campaign_member1_holds_member1 x) ])
    @ (match value.fallen_companions with
      | None ->
          if List.mem "fallenCompanions" value.wire.undefined_fields then
            [ ("fallenCompanions", undefined) ]
          else []
      | Some x -> [ ("fallenCompanions", encode_fragments_native_state38_log x) ])
    @ (match value.veterans with
      | None ->
          if List.mem "veterans" value.wire.undefined_fields then [ ("veterans", undefined) ]
          else []
      | Some x ->
          [ ("veterans", encode_save26_checkpoint_state_campaign_member1_veterans_member1 x) ])
    @ (match value.story_veterans with
      | None ->
          if List.mem "storyVeterans" value.wire.undefined_fields then
            [ ("storyVeterans", undefined) ]
          else []
      | Some x ->
          [ ("storyVeterans", encode_save26_checkpoint_state_campaign_member1_veterans_member1 x) ])
    @ (match value.evacuated with
      | None ->
          if List.mem "evacuated" value.wire.undefined_fields then [ ("evacuated", undefined) ]
          else []
      | Some x ->
          [ ("evacuated", encode_save26_checkpoint_state_campaign_member1_evacuated_member1 x) ])
    @
    match value.scripts with
    | None ->
        if List.mem "scripts" value.wire.undefined_fields then [ ("scripts", undefined) ] else []
    | Some x -> [ ("scripts", encode_save34_checkpoint_state_campaign_member1_scripts_member1 x) ])

and encode_save34_checkpoint_state_campaign_member1_scripts
    (value : save34_checkpoint_state_campaign_member1_scripts) : Js.Json.t =
  match value with
  | `Undefined -> undefined
  | `Object2 x -> encode_save34_checkpoint_state_campaign_member1_scripts_member1 x

and encode_save34_checkpoint_state_campaign_member1_scripts_member1
    (value : save34_checkpoint_state_campaign_member1_scripts_member1) : Js.Json.t =
  wire_to_json
    ~known:
      [
        "recruitment";
        "failed";
        "active";
        "holds";
        "nextMark";
        "pending";
        "marks";
        "completed";
        "revealed";
      ]
    value.wire
    ([
       ( "recruitment",
         encode_fragments_native_state38_campaign_member1_scripts_member1_recruitment
           value.recruitment );
       ("active", encode_fragments_native_state38_log value.active);
       ("holds", encode_fragments_native_state38_campaign_member1_scripts_member1_holds value.holds);
       ("nextMark", encode_json_number value.next_mark);
       ( "pending",
         encode_fragments_native_state38_campaign_member1_scripts_member1_pending value.pending );
       ("marks", encode_save28_checkpoint_state_campaign_member1_scripts_member1_marks value.marks);
       ("completed", encode_fragments_native_state38_log value.completed);
       ("revealed", encode_fragments_native_state38_log value.revealed);
     ]
    @
    match value.failed with
    | None ->
        if List.mem "failed" value.wire.undefined_fields then [ ("failed", undefined) ] else []
    | Some x -> [ ("failed", encode_fragments_native_state38_log x) ])

and encode_save34_undo (value : save34_undo) : Js.Json.t =
  match value with `Undefined -> undefined | `Array x -> encode_save34_undo_member1 x

and encode_save34_undo_member1 (value : save34_undo_member1) : Js.Json.t =
  Js.Json.array (Array.map encode_save34_undo_member1_item value)

and encode_save34_undo_member1_item (value : save34_undo_member1_item) : Js.Json.t =
  wire_to_json ~known:[ "round"; "unitId"; "changes" ] value.wire
    [
      ("round", encode_json_number value.round);
      ("unitId", encode_json_string value.unit_id);
      ("changes", encode_save34_undo_member1_item_changes value.changes);
    ]

and encode_save34_undo_member1_item_changes (value : save34_undo_member1_item_changes) : Js.Json.t =
  Js.Json.array (Array.map encode_save34_undo_member1_item_changes_item value)

and encode_save34_undo_member1_item_changes_item (value : save34_undo_member1_item_changes_item) :
    Js.Json.t =
  match value with
  | `Set x -> encode_save34_undo_member1_item_changes_item_member0 x
  | `Remove x -> encode_save34_undo_member1_item_changes_item_member1 x

and encode_save34_undo_member1_item_changes_item_member0
    (value : save34_undo_member1_item_changes_item_member0) : Js.Json.t =
  wire_to_json ~known:[ "op"; "path"; "value" ] value.wire
    [
      ("op", encode_literal_set value.op);
      ("path", encode_fragments_native_state38_log value.path);
      ("value", encode_json_unknown value.value);
    ]

and encode_literal_set (value : literal_set) : Js.Json.t = Js.Json.parseExn "\"set\""

and encode_save34_undo_member1_item_changes_item_member1
    (value : save34_undo_member1_item_changes_item_member1) : Js.Json.t =
  wire_to_json ~known:[ "op"; "path" ] value.wire
    [
      ("op", encode_literal_remove value.op);
      ("path", encode_fragments_native_state38_log value.path);
    ]

and encode_literal_remove (value : literal_remove) : Js.Json.t = Js.Json.parseExn "\"remove\""

and encode_save35 (value : save35) : Js.Json.t =
  wire_to_json
    ~known:[ "version"; "checkpoint"; "pendingEnemy"; "setup"; "progress"; "undo" ]
    value.wire
    ([
       ("version", encode_literal_35 value.version);
       ("checkpoint", encode_save35_checkpoint value.checkpoint);
       ("pendingEnemy", encode_json_boolean value.pending_enemy);
       ("setup", encode_save35_setup value.setup);
       ("progress", encode_save35_progress value.progress);
     ]
    @
    match value.undo with
    | None -> if List.mem "undo" value.wire.undefined_fields then [ ("undo", undefined) ] else []
    | Some x -> [ ("undo", encode_save34_undo_member1 x) ])

and encode_literal_35 (value : literal_35) : Js.Json.t = Js.Json.parseExn "35"

and encode_save35_checkpoint (value : save35_checkpoint) : Js.Json.t =
  wire_to_json ~known:[ "seed"; "state" ] value.wire
    [
      ("seed", encode_json_number value.seed); ("state", encode_save35_checkpoint_state value.state);
    ]

and encode_save35_checkpoint_state (value : save35_checkpoint_state) : Js.Json.t =
  wire_to_json
    ~known:
      [
        "round";
        "turn";
        "winner";
        "armies";
        "conditionMemory";
        "mapId";
        "mapRevision";
        "cols";
        "rows";
        "nextId";
        "log";
        "lastCombat";
        "units";
        "tiles";
        "campaign";
        "gold";
        "factions";
        "groundItems";
        "combatRng";
      ]
    value.wire
    ([
       ("round", encode_json_number value.round);
       ("turn", encode_save35_setup_first value.turn);
       ("winner", encode_fragments_native_state38_winner value.winner);
       ("mapId", encode_json_string value.map_id);
       ("mapRevision", encode_json_number value.map_revision);
       ("cols", encode_json_number value.cols);
       ("rows", encode_json_number value.rows);
       ("nextId", encode_json_number value.next_id);
       ("log", encode_fragments_native_state38_log value.log);
       ("units", encode_save26_checkpoint_state_units value.units);
       ("tiles", encode_save26_checkpoint_state_tiles value.tiles);
       ("gold", encode_fragments_native_state38_gold value.gold);
       ("factions", encode_save26_checkpoint_state_factions value.factions);
       ("groundItems", encode_fragments_native_state38_ground_items value.ground_items);
       ("combatRng", encode_json_number value.combat_rng);
     ]
    @ (match value.armies with
      | None ->
          if List.mem "armies" value.wire.undefined_fields then [ ("armies", undefined) ] else []
      | Some x -> [ ("armies", encode_fragments_native_state38_armies_member1 x) ])
    @ (match value.condition_memory with
      | None ->
          if List.mem "conditionMemory" value.wire.undefined_fields then
            [ ("conditionMemory", undefined) ]
          else []
      | Some x -> [ ("conditionMemory", encode_fragments_native_state38_log x) ])
    @ (match value.last_combat with
      | None ->
          if List.mem "lastCombat" value.wire.undefined_fields then [ ("lastCombat", undefined) ]
          else []
      | Some x -> [ ("lastCombat", encode_fragments_native_state38_last_combat x) ])
    @
    match value.campaign with
    | None ->
        if List.mem "campaign" value.wire.undefined_fields then [ ("campaign", undefined) ] else []
    | Some x -> [ ("campaign", encode_save34_checkpoint_state_campaign_member1 x) ])

and encode_save35_progress (value : save35_progress) : Js.Json.t =
  wire_to_json ~known:[ "entry" ] value.wire [ ("entry", encode_save35_progress_entry value.entry) ]

and encode_save35_progress_entry (value : save35_progress_entry) : Js.Json.t =
  wire_to_json
    ~known:
      [
        "id";
        "savedAt";
        "run";
        "mission";
        "round";
        "side";
        "missionStart";
        "runId";
        "branchId";
        "checkpointId";
        "ordinal";
        "kind";
        "parentId";
      ]
    value.wire
    ([
       ("id", encode_json_string value.id);
       ("savedAt", encode_json_number value.saved_at);
       ("run", encode_json_string value.run);
       ("mission", encode_json_string value.mission);
       ("round", encode_json_number value.round);
       ("side", encode_json_string value.side);
       ("missionStart", encode_json_boolean value.mission_start);
       ("runId", encode_json_string value.run_id);
       ("branchId", encode_json_string value.branch_id);
       ("checkpointId", encode_json_string value.checkpoint_id);
       ("ordinal", encode_json_number value.ordinal);
       ("kind", encode_save35_progress_entry_kind value.kind);
     ]
    @
    match value.parent_id with
    | None ->
        if List.mem "parentId" value.wire.undefined_fields then [ ("parentId", undefined) ] else []
    | Some x -> [ ("parentId", encode_json_string x) ])

and encode_save35_progress_entry_kind (value : save35_progress_entry_kind) : Js.Json.t =
  match value with
  | `Turn -> Js.Json.parseExn "\"turn\""
  | `Manual -> Js.Json.parseExn "\"manual\""
  | `Recovery -> Js.Json.parseExn "\"recovery\""

and encode_literal_manual (value : literal_manual) : Js.Json.t = Js.Json.parseExn "\"manual\""

and encode_save36 (value : save36) : Js.Json.t =
  wire_to_json
    ~known:[ "version"; "checkpoint"; "pendingEnemy"; "setup"; "progress"; "undo" ]
    value.wire
    ([
       ("version", encode_literal_36 value.version);
       ("checkpoint", encode_save35_checkpoint value.checkpoint);
       ("pendingEnemy", encode_json_boolean value.pending_enemy);
       ("setup", encode_save35_setup value.setup);
       ("progress", encode_save35_progress value.progress);
     ]
    @
    match value.undo with
    | None -> if List.mem "undo" value.wire.undefined_fields then [ ("undo", undefined) ] else []
    | Some x -> [ ("undo", encode_save34_undo_member1 x) ])

and encode_literal_36 (value : literal_36) : Js.Json.t = Js.Json.parseExn "36"

and encode_save37 (value : save37) : Js.Json.t =
  wire_to_json
    ~known:[ "version"; "checkpoint"; "pendingEnemy"; "setup"; "progress"; "undo" ]
    value.wire
    ([
       ("version", encode_literal_37 value.version);
       ("checkpoint", encode_save35_checkpoint value.checkpoint);
       ("pendingEnemy", encode_json_boolean value.pending_enemy);
       ("setup", encode_save35_setup value.setup);
       ("progress", encode_save37_progress value.progress);
     ]
    @
    match value.undo with
    | None -> if List.mem "undo" value.wire.undefined_fields then [ ("undo", undefined) ] else []
    | Some x -> [ ("undo", encode_save34_undo_member1 x) ])

and encode_literal_37 (value : literal_37) : Js.Json.t = Js.Json.parseExn "37"

and encode_save37_progress (value : save37_progress) : Js.Json.t =
  wire_to_json ~known:[ "entry" ] value.wire [ ("entry", encode_save37_progress_entry value.entry) ]

and encode_save37_progress_entry (value : save37_progress_entry) : Js.Json.t =
  match value with
  | `Flash x -> encode_save37_progress_entry_member0 x
  | `Turn x -> encode_history4_entries_item x

and encode_save37_progress_entry_member0 (value : save37_progress_entry_member0) : Js.Json.t =
  wire_to_json
    ~known:
      [
        "kind";
        "id";
        "savedAt";
        "run";
        "mission";
        "round";
        "side";
        "missionStart";
        "runId";
        "branchId";
        "checkpointId";
        "ordinal";
      ]
    value.wire
    [
      ("kind", encode_literal_flash value.kind);
      ("id", encode_json_string value.id);
      ("savedAt", encode_json_number value.saved_at);
      ("run", encode_json_string value.run);
      ("mission", encode_json_string value.mission);
      ("round", encode_json_number value.round);
      ("side", encode_json_string value.side);
      ("missionStart", encode_json_boolean value.mission_start);
      ("runId", encode_json_string value.run_id);
      ("branchId", encode_json_string value.branch_id);
      ("checkpointId", encode_json_string value.checkpoint_id);
      ("ordinal", encode_json_number value.ordinal);
    ]

and encode_literal_flash (value : literal_flash) : Js.Json.t = Js.Json.parseExn "\"flash\""

and encode_history4_entries_item (value : history4_entries_item) : Js.Json.t =
  wire_to_json
    ~known:
      [
        "kind";
        "manual";
        "parentId";
        "id";
        "savedAt";
        "run";
        "mission";
        "round";
        "side";
        "missionStart";
        "runId";
        "branchId";
        "checkpointId";
        "ordinal";
      ]
    value.wire
    ([
       ("kind", encode_literal_turn value.kind);
       ("manual", encode_json_boolean value.manual);
       ("id", encode_json_string value.id);
       ("savedAt", encode_json_number value.saved_at);
       ("run", encode_json_string value.run);
       ("mission", encode_json_string value.mission);
       ("round", encode_json_number value.round);
       ("side", encode_json_string value.side);
       ("missionStart", encode_json_boolean value.mission_start);
       ("runId", encode_json_string value.run_id);
       ("branchId", encode_json_string value.branch_id);
       ("checkpointId", encode_json_string value.checkpoint_id);
       ("ordinal", encode_json_number value.ordinal);
     ]
    @
    match value.parent_id with
    | None ->
        if List.mem "parentId" value.wire.undefined_fields then [ ("parentId", undefined) ] else []
    | Some x -> [ ("parentId", encode_json_string x) ])

and encode_save38 (value : save38) : Js.Json.t =
  wire_to_json
    ~known:[ "version"; "checkpoint"; "pendingEnemy"; "setup"; "progress"; "undo" ]
    value.wire
    ([
       ("version", encode_literal_38 value.version);
       ("checkpoint", encode_save35_checkpoint value.checkpoint);
       ("pendingEnemy", encode_json_boolean value.pending_enemy);
       ("setup", encode_save35_setup value.setup);
       ("progress", encode_save37_progress value.progress);
     ]
    @
    match value.undo with
    | None -> if List.mem "undo" value.wire.undefined_fields then [ ("undo", undefined) ] else []
    | Some x -> [ ("undo", encode_save34_undo_member1 x) ])

and encode_literal_38 (value : literal_38) : Js.Json.t = Js.Json.parseExn "38"

and encode_save39 (value : save39) : Js.Json.t =
  match value with `Current x -> encode_save39_member0 x | `Turn x -> encode_save39_member1 x

and encode_save39_member0 (value : save39_member0) : Js.Json.t =
  wire_to_json
    ~known:[ "pendingEnemy"; "undo"; "progress"; "version"; "checkpoint"; "setup" ]
    value.wire
    [
      ("pendingEnemy", encode_json_boolean value.pending_enemy);
      ("undo", encode_save34_undo_member1 value.undo);
      ("progress", encode_save39_member0_progress value.progress);
      ("version", encode_literal_39 value.version);
      ("checkpoint", encode_save39_member0_checkpoint value.checkpoint);
      ("setup", encode_save39_member0_setup value.setup);
    ]

and encode_save39_member0_progress (value : save39_member0_progress) : Js.Json.t =
  wire_to_json ~known:[ "entry" ] value.wire
    [ ("entry", encode_save37_progress_entry_member0 value.entry) ]

and encode_literal_39 (value : literal_39) : Js.Json.t = Js.Json.parseExn "39"

and encode_save39_member0_checkpoint (value : save39_member0_checkpoint) : Js.Json.t =
  wire_to_json ~known:[ "seed"; "state" ] value.wire
    [
      ("seed", encode_json_number value.seed);
      ("state", encode_save39_member0_checkpoint_state value.state);
    ]

and encode_save39_member0_checkpoint_state (value : save39_member0_checkpoint_state) : Js.Json.t =
  wire_to_json
    ~known:
      [
        "factions";
        "groundItems";
        "armies";
        "conditionMemory";
        "mapId";
        "mapRevision";
        "cols";
        "rows";
        "round";
        "turn";
        "gold";
        "winner";
        "nextId";
        "combatRng";
        "log";
        "tiles";
        "units";
        "campaign";
        "lastCombat";
      ]
    value.wire
    ([
       ("factions", encode_save39_member0_checkpoint_state_factions value.factions);
       ("groundItems", encode_save39_member0_checkpoint_state_ground_items value.ground_items);
       ("mapId", encode_json_string value.map_id);
       ("mapRevision", encode_json_number value.map_revision);
       ("cols", encode_json_number value.cols);
       ("rows", encode_json_number value.rows);
       ("round", encode_json_number value.round);
       ("turn", encode_save35_setup_first value.turn);
       ("gold", encode_fragments_native_state38_gold value.gold);
       ("winner", encode_fragments_native_state38_winner value.winner);
       ("nextId", encode_json_number value.next_id);
       ("combatRng", encode_json_number value.combat_rng);
       ("log", encode_fragments_native_state38_log value.log);
       ("tiles", encode_save39_member0_checkpoint_state_tiles value.tiles);
       ("units", encode_save39_member0_checkpoint_state_units value.units);
     ]
    @ (match value.armies with
      | None ->
          if List.mem "armies" value.wire.undefined_fields then [ ("armies", undefined) ] else []
      | Some x -> [ ("armies", encode_fragments_native_state38_armies_member1 x) ])
    @ (match value.condition_memory with
      | None ->
          if List.mem "conditionMemory" value.wire.undefined_fields then
            [ ("conditionMemory", undefined) ]
          else []
      | Some x -> [ ("conditionMemory", encode_fragments_native_state38_log x) ])
    @ (match value.campaign with
      | None ->
          if List.mem "campaign" value.wire.undefined_fields then [ ("campaign", undefined) ]
          else []
      | Some x -> [ ("campaign", encode_save39_member0_checkpoint_state_campaign_member1 x) ])
    @
    match value.last_combat with
    | None ->
        if List.mem "lastCombat" value.wire.undefined_fields then [ ("lastCombat", undefined) ]
        else []
    | Some x -> [ ("lastCombat", encode_fragments_native_state38_last_combat x) ])

and encode_save39_member0_checkpoint_state_factions
    (value : save39_member0_checkpoint_state_factions) : Js.Json.t =
  Js.Json.array (Array.map encode_save39_member0_checkpoint_state_factions_item value)

and encode_save39_member0_checkpoint_state_factions_item
    (value : save39_member0_checkpoint_state_factions_item) : Js.Json.t =
  wire_to_json
    ~known:[ "id"; "phase"; "controller"; "palette"; "name"; "recruits"; "relations"; "transfer" ]
    value.wire
    ([
       ("id", encode_json_string value.id);
       ("phase", encode_save35_setup_first value.phase);
       ("controller", encode_fragments_factions7_item_controller value.controller);
       ("palette", encode_json_string value.palette);
     ]
    @ (match value.name with
      | None -> if List.mem "name" value.wire.undefined_fields then [ ("name", undefined) ] else []
      | Some x -> [ ("name", encode_json_string x) ])
    @ (match value.recruits with
      | None ->
          if List.mem "recruits" value.wire.undefined_fields then [ ("recruits", undefined) ]
          else []
      | Some x -> [ ("recruits", encode_fragments_factions7_item_recruits_member1 x) ])
    @ (match value.relations with
      | None ->
          if List.mem "relations" value.wire.undefined_fields then [ ("relations", undefined) ]
          else []
      | Some x -> [ ("relations", encode_fragments_factions7_item_relations_member1 x) ])
    @
    match value.transfer with
    | None ->
        if List.mem "transfer" value.wire.undefined_fields then [ ("transfer", undefined) ] else []
    | Some x ->
        [ ("transfer", encode_save39_member0_checkpoint_state_factions_item_transfer_member1 x) ])

and encode_save39_member0_checkpoint_state_factions_item_transfer
    (value : save39_member0_checkpoint_state_factions_item_transfer) : Js.Json.t =
  match value with
  | `Undefined -> undefined
  | `Object2 x -> encode_save39_member0_checkpoint_state_factions_item_transfer_member1 x

and encode_save39_member0_checkpoint_state_factions_item_transfer_member1
    (value : save39_member0_checkpoint_state_factions_item_transfer_member1) : Js.Json.t =
  wire_to_json ~known:[ "to"; "when" ] value.wire
    [
      ("to", encode_json_string value.to_);
      ("when", encode_save39_member0_checkpoint_state_tiles_item_fog_member1 value.when_);
    ]

and encode_save39_member0_checkpoint_state_tiles_item_fog_member1
    (value : save39_member0_checkpoint_state_tiles_item_fog_member1) : Js.Json.t =
  wire_to_json ~known:[ "condition"; "remember" ] value.wire
    ([
       ( "condition",
         encode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition value.condition );
     ]
    @
    match value.remember with
    | None ->
        if List.mem "remember" value.wire.undefined_fields then [ ("remember", undefined) ] else []
    | Some x -> [ ("remember", encode_fragments_rule11_remember x) ])

and encode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition
    (value : save39_member0_checkpoint_state_tiles_item_fog_member1_condition) : Js.Json.t =
  match value with
  | `Owner x -> encode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0 x
  | `Proximity x ->
      encode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member1 x
  | `Visible_units x -> encode_fragments_rule11_condition_member1 x
  | `Has_units x -> encode_fragments_light28_member1_when__member1_condition_member3 x
  | `No_units x -> encode_fragments_light28_member1_when__member1_condition_member4 x
  | `Choice x -> encode_fragments_rule11_condition_member2 x
  | `Flag x -> encode_fragments_rule11_condition_member3 x
  | `Event x -> encode_fragments_rule11_condition_member4 x
  | `Round x -> encode_fragments_rule11_condition_member5 x
  | `All x -> encode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member9 x
  | `Any x -> encode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member10 x

and encode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0
    (value : save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0) : Js.Json.t =
  wire_to_json ~known:[ "kind"; "tile"; "side" ] value.wire
    ([ ("kind", encode_literal_owner value.kind); ("side", encode_json_string value.side) ]
    @
    match value.tile with
    | None -> if List.mem "tile" value.wire.undefined_fields then [ ("tile", undefined) ] else []
    | Some x ->
        [
          ( "tile",
            encode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile x
          );
        ])

and encode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile
    (value : save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile) :
    Js.Json.t =
  match value with
  | `Undefined -> undefined
  | `Location x ->
      encode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member1 x
  | `Object3 x ->
      encode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member2 x
  | `Object4 x ->
      encode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member3 x

and encode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member1
    (value : save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member1)
    : Js.Json.t =
  wire_to_json ~known:[ "location" ] value.wire [ ("location", encode_json_string value.location) ]

and encode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member2
    (value : save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member2)
    : Js.Json.t =
  wire_to_json ~known:[ "hex" ] value.wire
    [ ("hex", encode_save39_member0_checkpoint_state_ground_items_item_tile value.hex) ]

and encode_save39_member0_checkpoint_state_ground_items_item_tile
    (value : save39_member0_checkpoint_state_ground_items_item_tile) : Js.Json.t =
  wire_to_json ~known:[ "col"; "row" ] value.wire
    [ ("col", encode_json_number value.col); ("row", encode_json_number value.row) ]

and encode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member3
    (value : save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member3)
    : Js.Json.t =
  wire_to_json ~known:[ "binding" ] value.wire
    [
      ( "binding",
        encode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member3_binding
          value.binding );
    ]

and encode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member3_binding
    (value :
      save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member3_binding)
    : Js.Json.t =
  match value with
  | `Unit -> Js.Json.parseExn "\"unit\""
  | `Mark -> Js.Json.parseExn "\"mark\""
  | `Death -> Js.Json.parseExn "\"death\""

and encode_literal_unit (value : literal_unit) : Js.Json.t = Js.Json.parseExn "\"unit\""
and encode_literal_mark (value : literal_mark) : Js.Json.t = Js.Json.parseExn "\"mark\""
and encode_literal_death (value : literal_death) : Js.Json.t = Js.Json.parseExn "\"death\""

and encode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member1
    (value : save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member1) : Js.Json.t =
  wire_to_json
    ~known:[ "kind"; "unitTypes"; "tile"; "radius"; "side"; "unitType"; "character" ]
    value.wire
    ([
       ("kind", encode_literal_proximity value.kind);
       ("radius", encode_json_number value.radius);
       ("side", encode_fragments_rule11_condition_member1_side value.side);
     ]
    @ (match value.unit_types with
      | None ->
          if List.mem "unitTypes" value.wire.undefined_fields then [ ("unitTypes", undefined) ]
          else []
      | Some x -> [ ("unitTypes", encode_fragments_native_state38_log x) ])
    @ (match value.tile with
      | None -> if List.mem "tile" value.wire.undefined_fields then [ ("tile", undefined) ] else []
      | Some x ->
          [
            ( "tile",
              encode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile x
            );
          ])
    @ (match value.unit_type with
      | None ->
          if List.mem "unitType" value.wire.undefined_fields then [ ("unitType", undefined) ]
          else []
      | Some x -> [ ("unitType", encode_json_string x) ])
    @
    match value.character with
    | None ->
        if List.mem "character" value.wire.undefined_fields then [ ("character", undefined) ]
        else []
    | Some x -> [ ("character", encode_json_string x) ])

and encode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member9
    (value : save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member9) : Js.Json.t =
  wire_to_json ~known:[ "kind"; "conditions" ] value.wire
    [
      ("kind", encode_literal_all value.kind);
      ( "conditions",
        encode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member9_conditions
          value.conditions );
    ]

and encode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member9_conditions
    (value : save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member9_conditions) :
    Js.Json.t =
  Js.Json.array
    (Array.map encode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition value)

and encode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member10
    (value : save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member10) : Js.Json.t
    =
  wire_to_json ~known:[ "kind"; "conditions" ] value.wire
    [
      ("kind", encode_literal_any value.kind);
      ( "conditions",
        encode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member9_conditions
          value.conditions );
    ]

and encode_save39_member0_checkpoint_state_ground_items
    (value : save39_member0_checkpoint_state_ground_items) : Js.Json.t =
  Js.Json.array (Array.map encode_save39_member0_checkpoint_state_ground_items_item value)

and encode_save39_member0_checkpoint_state_ground_items_item
    (value : save39_member0_checkpoint_state_ground_items_item) : Js.Json.t =
  wire_to_json ~known:[ "tile"; "item" ] value.wire
    [
      ("tile", encode_save39_member0_checkpoint_state_ground_items_item_tile value.tile);
      ("item", encode_fragments_native_state38_ground_items_item_item value.item);
    ]

and encode_save39_member0_checkpoint_state_tiles (value : save39_member0_checkpoint_state_tiles) :
    Js.Json.t =
  Js.Json.array (Array.map encode_save39_member0_checkpoint_state_tiles_item value)

and encode_save39_member0_checkpoint_state_tiles_item
    (value : save39_member0_checkpoint_state_tiles_item) : Js.Json.t =
  wire_to_json
    ~known:
      [
        "id";
        "location";
        "col";
        "row";
        "terrain";
        "owner";
        "connections";
        "deadEnds";
        "walls";
        "floor";
        "fog";
        "light";
      ]
    value.wire
    ([
       ("id", encode_json_string value.id);
       ("col", encode_json_number value.col);
       ("row", encode_json_number value.row);
       ("terrain", encode_json_string value.terrain);
       ("owner", encode_fragments_tiles11_item_owner value.owner);
     ]
    @ (match value.location with
      | None ->
          if List.mem "location" value.wire.undefined_fields then [ ("location", undefined) ]
          else []
      | Some x -> [ ("location", encode_json_string x) ])
    @ (match value.connections with
      | None ->
          if List.mem "connections" value.wire.undefined_fields then [ ("connections", undefined) ]
          else []
      | Some x -> [ ("connections", encode_fragments_tiles11_item_connections_member1 x) ])
    @ (match value.dead_ends with
      | None ->
          if List.mem "deadEnds" value.wire.undefined_fields then [ ("deadEnds", undefined) ]
          else []
      | Some x -> [ ("deadEnds", encode_fragments_tiles11_item_connections_member1 x) ])
    @ (match value.walls with
      | None ->
          if List.mem "walls" value.wire.undefined_fields then [ ("walls", undefined) ] else []
      | Some x -> [ ("walls", encode_fragments_tiles11_item_connections_member1 x) ])
    @ (match value.floor with
      | None ->
          if List.mem "floor" value.wire.undefined_fields then [ ("floor", undefined) ] else []
      | Some x -> [ ("floor", encode_json_string x) ])
    @ (match value.fog with
      | None -> if List.mem "fog" value.wire.undefined_fields then [ ("fog", undefined) ] else []
      | Some x -> [ ("fog", encode_save39_member0_checkpoint_state_tiles_item_fog_member1 x) ])
    @
    match value.light with
    | None -> if List.mem "light" value.wire.undefined_fields then [ ("light", undefined) ] else []
    | Some x -> [ ("light", encode_save39_member0_checkpoint_state_tiles_item_light_member1 x) ])

and encode_save39_member0_checkpoint_state_tiles_item_fog
    (value : save39_member0_checkpoint_state_tiles_item_fog) : Js.Json.t =
  match value with
  | `Undefined -> undefined
  | `Object2 x -> encode_save39_member0_checkpoint_state_tiles_item_fog_member1 x

and encode_save39_member0_checkpoint_state_tiles_item_light
    (value : save39_member0_checkpoint_state_tiles_item_light) : Js.Json.t =
  match value with
  | `Undefined -> undefined
  | `Object2 x -> encode_save39_member0_checkpoint_state_tiles_item_light_member1 x

and encode_save39_member0_checkpoint_state_tiles_item_light_member1
    (value : save39_member0_checkpoint_state_tiles_item_light_member1) : Js.Json.t =
  wire_to_json
    ~known:[ "color"; "fixture"; "dx"; "dy"; "radius"; "power"; "when" ]
    value.wire
    ([ ("radius", encode_json_number value.radius); ("power", encode_json_number value.power) ]
    @ (match value.color with
      | None ->
          if List.mem "color" value.wire.undefined_fields then [ ("color", undefined) ] else []
      | Some x -> [ ("color", encode_fragments_light23_color_member1 x) ])
    @ (match value.fixture with
      | None ->
          if List.mem "fixture" value.wire.undefined_fields then [ ("fixture", undefined) ] else []
      | Some x -> [ ("fixture", encode_fragments_rule11_remember x) ])
    @ (match value.dx with
      | None -> if List.mem "dx" value.wire.undefined_fields then [ ("dx", undefined) ] else []
      | Some x -> [ ("dx", encode_json_number x) ])
    @ (match value.dy with
      | None -> if List.mem "dy" value.wire.undefined_fields then [ ("dy", undefined) ] else []
      | Some x -> [ ("dy", encode_json_number x) ])
    @
    match value.when_ with
    | None -> if List.mem "when" value.wire.undefined_fields then [ ("when", undefined) ] else []
    | Some x -> [ ("when", encode_save39_member0_checkpoint_state_tiles_item_fog_member1 x) ])

and encode_save39_member0_checkpoint_state_units (value : save39_member0_checkpoint_state_units) :
    Js.Json.t =
  Js.Json.array (Array.map encode_save39_member0_checkpoint_state_units_item value)

and encode_save39_member0_checkpoint_state_units_item
    (value : save39_member0_checkpoint_state_units_item) : Js.Json.t =
  wire_to_json
    ~known:
      [
        "faction";
        "items";
        "aiGroup";
        "id";
        "type";
        "side";
        "tile";
        "hp";
        "moved";
        "attacked";
        "experience";
        "character";
        "role";
        "retreating";
        "rebuilding";
      ]
    value.wire
    ([
       ("id", encode_json_string value.id);
       ("type", encode_json_string value.type_);
       ("side", encode_save35_setup_first value.side);
       ("tile", encode_save39_member0_checkpoint_state_ground_items_item_tile value.tile);
       ("hp", encode_json_number value.hp);
       ("moved", encode_json_boolean value.moved);
       ("attacked", encode_json_boolean value.attacked);
     ]
    @ (match value.faction with
      | None ->
          if List.mem "faction" value.wire.undefined_fields then [ ("faction", undefined) ] else []
      | Some x -> [ ("faction", encode_json_string x) ])
    @ (match value.items with
      | None ->
          if List.mem "items" value.wire.undefined_fields then [ ("items", undefined) ] else []
      | Some x -> [ ("items", encode_fragments_native_state38_units_item_items_member1 x) ])
    @ (match value.ai_group with
      | None ->
          if List.mem "aiGroup" value.wire.undefined_fields then [ ("aiGroup", undefined) ] else []
      | Some x -> [ ("aiGroup", encode_json_string x) ])
    @ (match value.experience with
      | None ->
          if List.mem "experience" value.wire.undefined_fields then [ ("experience", undefined) ]
          else []
      | Some x -> [ ("experience", encode_json_number x) ])
    @ (match value.character with
      | None ->
          if List.mem "character" value.wire.undefined_fields then [ ("character", undefined) ]
          else []
      | Some x -> [ ("character", encode_json_string x) ])
    @ (match value.role with
      | None -> if List.mem "role" value.wire.undefined_fields then [ ("role", undefined) ] else []
      | Some x -> [ ("role", encode_fragments_native_state38_units_item_role x) ])
    @ (match value.retreating with
      | None ->
          if List.mem "retreating" value.wire.undefined_fields then [ ("retreating", undefined) ]
          else []
      | Some x -> [ ("retreating", encode_fragments_rule11_remember x) ])
    @
    match value.rebuilding with
    | None ->
        if List.mem "rebuilding" value.wire.undefined_fields then [ ("rebuilding", undefined) ]
        else []
    | Some x -> [ ("rebuilding", encode_fragments_rule11_remember x) ])

and encode_save39_member0_checkpoint_state_campaign
    (value : save39_member0_checkpoint_state_campaign) : Js.Json.t =
  match value with
  | `Undefined -> undefined
  | `Object2 x -> encode_save39_member0_checkpoint_state_campaign_member1 x

and encode_save39_member0_checkpoint_state_campaign_member1
    (value : save39_member0_checkpoint_state_campaign_member1) : Js.Json.t =
  wire_to_json
    ~known:
      [
        "scripts";
        "evacuated";
        "id";
        "mission";
        "recall";
        "veterans";
        "storyVeterans";
        "fallenCompanions";
        "carryGold";
        "marks";
        "dialogue";
        "dialogueIndex";
        "outcome";
        "completed";
        "refuge";
        "vigil";
        "holds";
        "flags";
      ]
    value.wire
    ([
       ("id", encode_json_string value.id);
       ("mission", encode_json_number value.mission);
       ("recall", encode_fragments_native_state38_log value.recall);
       ("carryGold", encode_json_number value.carry_gold);
       ("marks", encode_fragments_native_state38_log value.marks);
       ("dialogue", encode_fragments_native_state38_campaign_member1_dialogue value.dialogue);
       ("dialogueIndex", encode_json_number value.dialogue_index);
       ("outcome", encode_fragments_tiles11_item_owner value.outcome);
       ("completed", encode_json_boolean value.completed);
       ("flags", encode_fragments_native_state38_campaign_member1_flags value.flags);
     ]
    @ (match value.scripts with
      | None ->
          if List.mem "scripts" value.wire.undefined_fields then [ ("scripts", undefined) ] else []
      | Some x ->
          [ ("scripts", encode_save39_member0_checkpoint_state_campaign_member1_scripts_member1 x) ])
    @ (match value.evacuated with
      | None ->
          if List.mem "evacuated" value.wire.undefined_fields then [ ("evacuated", undefined) ]
          else []
      | Some x ->
          [
            ( "evacuated",
              encode_save39_member0_checkpoint_state_campaign_member1_evacuated_member1 x );
          ])
    @ (match value.veterans with
      | None ->
          if List.mem "veterans" value.wire.undefined_fields then [ ("veterans", undefined) ]
          else []
      | Some x ->
          [ ("veterans", encode_fragments_native_state38_campaign_member1_veterans_member1 x) ])
    @ (match value.story_veterans with
      | None ->
          if List.mem "storyVeterans" value.wire.undefined_fields then
            [ ("storyVeterans", undefined) ]
          else []
      | Some x ->
          [ ("storyVeterans", encode_fragments_native_state38_campaign_member1_veterans_member1 x) ])
    @ (match value.fallen_companions with
      | None ->
          if List.mem "fallenCompanions" value.wire.undefined_fields then
            [ ("fallenCompanions", undefined) ]
          else []
      | Some x -> [ ("fallenCompanions", encode_fragments_native_state38_log x) ])
    @ (match value.refuge with
      | None ->
          if List.mem "refuge" value.wire.undefined_fields then [ ("refuge", undefined) ] else []
      | Some x -> [ ("refuge", encode_fragments_native_state38_campaign_member1_refuge_member1 x) ])
    @ (match value.vigil with
      | None ->
          if List.mem "vigil" value.wire.undefined_fields then [ ("vigil", undefined) ] else []
      | Some x -> [ ("vigil", encode_fragments_native_state38_campaign_member1_vigil_member1 x) ])
    @
    match value.holds with
    | None -> if List.mem "holds" value.wire.undefined_fields then [ ("holds", undefined) ] else []
    | Some x -> [ ("holds", encode_fragments_native_state38_campaign_member1_holds_member1 x) ])

and encode_save39_member0_checkpoint_state_campaign_member1_scripts
    (value : save39_member0_checkpoint_state_campaign_member1_scripts) : Js.Json.t =
  match value with
  | `Undefined -> undefined
  | `Object2 x -> encode_save39_member0_checkpoint_state_campaign_member1_scripts_member1 x

and encode_save39_member0_checkpoint_state_campaign_member1_scripts_member1
    (value : save39_member0_checkpoint_state_campaign_member1_scripts_member1) : Js.Json.t =
  wire_to_json
    ~known:
      [
        "failed";
        "completed";
        "revealed";
        "recruitment";
        "active";
        "holds";
        "marks";
        "pending";
        "nextMark";
      ]
    value.wire
    ([
       ("completed", encode_fragments_native_state38_log value.completed);
       ("revealed", encode_fragments_native_state38_log value.revealed);
       ( "recruitment",
         encode_fragments_native_state38_campaign_member1_scripts_member1_recruitment
           value.recruitment );
       ("active", encode_fragments_native_state38_log value.active);
       ("holds", encode_fragments_native_state38_campaign_member1_scripts_member1_holds value.holds);
       ( "marks",
         encode_save39_member0_checkpoint_state_campaign_member1_scripts_member1_marks value.marks
       );
       ( "pending",
         encode_save39_member0_checkpoint_state_campaign_member1_scripts_member1_pending
           value.pending );
       ("nextMark", encode_json_number value.next_mark);
     ]
    @
    match value.failed with
    | None ->
        if List.mem "failed" value.wire.undefined_fields then [ ("failed", undefined) ] else []
    | Some x -> [ ("failed", encode_fragments_native_state38_log x) ])

and encode_save39_member0_checkpoint_state_campaign_member1_scripts_member1_marks
    (value : save39_member0_checkpoint_state_campaign_member1_scripts_member1_marks) : Js.Json.t =
  Js.Json.array
    (Array.map encode_save39_member0_checkpoint_state_campaign_member1_scripts_member1_marks_item
       value)

and encode_save39_member0_checkpoint_state_campaign_member1_scripts_member1_marks_item
    (value : save39_member0_checkpoint_state_campaign_member1_scripts_member1_marks_item) :
    Js.Json.t =
  wire_to_json
    ~known:[ "lightRemembered"; "id"; "type"; "tile"; "light" ]
    value.wire
    ([
       ("id", encode_json_number value.id);
       ("type", encode_json_string value.type_);
       ("tile", encode_save39_member0_checkpoint_state_ground_items_item_tile value.tile);
     ]
    @ (match value.light_remembered with
      | None ->
          if List.mem "lightRemembered" value.wire.undefined_fields then
            [ ("lightRemembered", undefined) ]
          else []
      | Some x -> [ ("lightRemembered", encode_fragments_rule11_remember x) ])
    @
    match value.light with
    | None -> if List.mem "light" value.wire.undefined_fields then [ ("light", undefined) ] else []
    | Some x -> [ ("light", encode_save39_member0_checkpoint_state_tiles_item_light_member1 x) ])

and encode_save39_member0_checkpoint_state_campaign_member1_scripts_member1_pending
    (value : save39_member0_checkpoint_state_campaign_member1_scripts_member1_pending) : Js.Json.t =
  Js.Json.array
    (Array.map encode_save39_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item
       value)

and encode_save39_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item
    (value : save39_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item) :
    Js.Json.t =
  wire_to_json
    ~known:[ "area"; "faction"; "aiGroup"; "units" ]
    value.wire
    ([
       ( "area",
         encode_save39_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item_area
           value.area );
       ("faction", encode_json_string value.faction);
       ( "units",
         encode_fragments_native_state38_campaign_member1_scripts_member1_pending_item_units
           value.units );
     ]
    @
    match value.ai_group with
    | None ->
        if List.mem "aiGroup" value.wire.undefined_fields then [ ("aiGroup", undefined) ] else []
    | Some x -> [ ("aiGroup", encode_fragments_delta19_item_owner x) ])

and encode_save39_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item_area
    (value : save39_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item_area) :
    Js.Json.t =
  Js.Json.array (Array.map encode_save39_member0_checkpoint_state_ground_items_item_tile value)

and encode_save39_member0_checkpoint_state_campaign_member1_evacuated
    (value : save39_member0_checkpoint_state_campaign_member1_evacuated) : Js.Json.t =
  match value with
  | `Undefined -> undefined
  | `Array x -> encode_save39_member0_checkpoint_state_campaign_member1_evacuated_member1 x

and encode_save39_member0_checkpoint_state_campaign_member1_evacuated_member1
    (value : save39_member0_checkpoint_state_campaign_member1_evacuated_member1) : Js.Json.t =
  Js.Json.array
    (Array.map encode_save39_member0_checkpoint_state_campaign_member1_evacuated_member1_item value)

and encode_save39_member0_checkpoint_state_campaign_member1_evacuated_member1_item
    (value : save39_member0_checkpoint_state_campaign_member1_evacuated_member1_item) : Js.Json.t =
  wire_to_json ~known:[ "event"; "unit" ] value.wire
    [
      ("event", encode_json_string value.event);
      ("unit", encode_save39_member0_checkpoint_state_units_item value.unit_);
    ]

and encode_fragments_native_state38_campaign_member1_veterans
    (value : fragments_native_state38_campaign_member1_veterans) : Js.Json.t =
  match value with
  | `Undefined -> undefined
  | `Array x -> encode_fragments_native_state38_campaign_member1_veterans_member1 x

and encode_fragments_native_state38_campaign_member1_veterans_member1
    (value : fragments_native_state38_campaign_member1_veterans_member1) : Js.Json.t =
  Js.Json.array
    (Array.map encode_fragments_native_state38_campaign_member1_veterans_member1_item value)

and encode_fragments_native_state38_campaign_member1_veterans_member1_item
    (value : fragments_native_state38_campaign_member1_veterans_member1_item) : Js.Json.t =
  wire_to_json
    ~known:[ "items"; "type"; "experience"; "character"; "role" ]
    value.wire
    ([
       ("type", encode_json_string value.type_); ("experience", encode_json_number value.experience);
     ]
    @ (match value.items with
      | None ->
          if List.mem "items" value.wire.undefined_fields then [ ("items", undefined) ] else []
      | Some x -> [ ("items", encode_fragments_native_state38_units_item_items_member1 x) ])
    @ (match value.character with
      | None ->
          if List.mem "character" value.wire.undefined_fields then [ ("character", undefined) ]
          else []
      | Some x -> [ ("character", encode_json_string x) ])
    @
    match value.role with
    | None -> if List.mem "role" value.wire.undefined_fields then [ ("role", undefined) ] else []
    | Some x -> [ ("role", encode_fragments_native_state38_units_item_role x) ])

and encode_save39_member0_setup (value : save39_member0_setup) : Js.Json.t =
  wire_to_json ~known:[ "first"; "enemy" ] value.wire
    [
      ("first", encode_save35_setup_first value.first);
      ("enemy", encode_save39_member0_setup_enemy value.enemy);
    ]

and encode_save39_member0_setup_enemy (value : save39_member0_setup_enemy) : Js.Json.t =
  wire_to_json
    ~known:[ "blocks"; "policy"; "diplomacy"; "recruitmentLimit" ]
    value.wire
    ([
       ("policy", encode_save35_setup_enemy_policy value.policy);
       ("diplomacy", encode_save35_setup_enemy_diplomacy value.diplomacy);
     ]
    @ (match value.blocks with
      | None ->
          if List.mem "blocks" value.wire.undefined_fields then [ ("blocks", undefined) ] else []
      | Some x -> [ ("blocks", encode_save35_setup_enemy_blocks_member1 x) ])
    @
    match value.recruitment_limit with
    | None ->
        if List.mem "recruitmentLimit" value.wire.undefined_fields then
          [ ("recruitmentLimit", undefined) ]
        else []
    | Some x -> [ ("recruitmentLimit", encode_json_number x) ])

and encode_save39_member1 (value : save39_member1) : Js.Json.t =
  wire_to_json
    ~known:[ "pendingEnemy"; "undo"; "progress"; "version"; "checkpoint"; "setup" ]
    value.wire
    ([
       ("pendingEnemy", encode_literal_false value.pending_enemy);
       ("progress", encode_save39_member1_progress value.progress);
       ("version", encode_literal_39 value.version);
       ("checkpoint", encode_save39_member0_checkpoint value.checkpoint);
       ("setup", encode_save39_member0_setup value.setup);
     ]
    @
    match value.undo with
    | None -> if List.mem "undo" value.wire.undefined_fields then [ ("undo", undefined) ] else []
    | Some x -> [ ("undo", encode_json_undefined x) ])

and encode_save39_member1_progress (value : save39_member1_progress) : Js.Json.t =
  wire_to_json ~known:[ "entry" ] value.wire [ ("entry", encode_history4_entries_item value.entry) ]

and encode_history1 (value : history1) : Js.Json.t =
  wire_to_json ~known:[ "version"; "entries" ] value.wire
    [
      ("version", encode_literal_1 value.version); ("entries", encode_history1_entries value.entries);
    ]

and encode_literal_1 (value : literal_1) : Js.Json.t = Js.Json.parseExn "1"

and encode_history1_entries (value : history1_entries) : Js.Json.t =
  Js.Json.array (Array.map encode_history1_entries_item value)

and encode_history2 (value : history2) : Js.Json.t =
  wire_to_json ~known:[ "version"; "entries" ] value.wire
    [
      ("version", encode_literal_2 value.version); ("entries", encode_history2_entries value.entries);
    ]

and encode_history2_entries (value : history2_entries) : Js.Json.t =
  Js.Json.array (Array.map encode_history2_entries_item value)

and encode_history2_entries_item (value : history2_entries_item) : Js.Json.t =
  wire_to_json
    ~known:
      [
        "id";
        "savedAt";
        "run";
        "mission";
        "round";
        "side";
        "missionStart";
        "runId";
        "branchId";
        "checkpointId";
        "ordinal";
        "kind";
        "parentId";
      ]
    value.wire
    ([
       ("id", encode_json_string value.id);
       ("savedAt", encode_json_number value.saved_at);
       ("run", encode_json_string value.run);
       ("mission", encode_json_string value.mission);
       ("round", encode_json_number value.round);
       ("side", encode_json_string value.side);
       ("missionStart", encode_json_boolean value.mission_start);
       ("runId", encode_json_string value.run_id);
       ("branchId", encode_json_string value.branch_id);
       ("checkpointId", encode_json_string value.checkpoint_id);
       ("ordinal", encode_json_number value.ordinal);
       ("kind", encode_literal_turn value.kind);
     ]
    @
    match value.parent_id with
    | None ->
        if List.mem "parentId" value.wire.undefined_fields then [ ("parentId", undefined) ] else []
    | Some x -> [ ("parentId", encode_json_string x) ])

and encode_history3 (value : history3) : Js.Json.t =
  wire_to_json ~known:[ "version"; "entries" ] value.wire
    [
      ("version", encode_literal_3 value.version); ("entries", encode_history3_entries value.entries);
    ]

and encode_history3_entries (value : history3_entries) : Js.Json.t =
  Js.Json.array (Array.map encode_history3_entries_item value)

and encode_history3_entries_item (value : history3_entries_item) : Js.Json.t =
  wire_to_json
    ~known:
      [
        "id";
        "savedAt";
        "run";
        "mission";
        "round";
        "side";
        "missionStart";
        "runId";
        "branchId";
        "checkpointId";
        "ordinal";
        "kind";
        "parentId";
      ]
    value.wire
    ([
       ("id", encode_json_string value.id);
       ("savedAt", encode_json_number value.saved_at);
       ("run", encode_json_string value.run);
       ("mission", encode_json_string value.mission);
       ("round", encode_json_number value.round);
       ("side", encode_json_string value.side);
       ("missionStart", encode_json_boolean value.mission_start);
       ("runId", encode_json_string value.run_id);
       ("branchId", encode_json_string value.branch_id);
       ("checkpointId", encode_json_string value.checkpoint_id);
       ("ordinal", encode_json_number value.ordinal);
       ("kind", encode_history3_entries_item_kind value.kind);
     ]
    @
    match value.parent_id with
    | None ->
        if List.mem "parentId" value.wire.undefined_fields then [ ("parentId", undefined) ] else []
    | Some x -> [ ("parentId", encode_json_string x) ])

and encode_history3_entries_item_kind (value : history3_entries_item_kind) : Js.Json.t =
  match value with `Turn -> Js.Json.parseExn "\"turn\"" | `Manual -> Js.Json.parseExn "\"manual\""

and encode_history4 (value : history4) : Js.Json.t =
  wire_to_json ~known:[ "version"; "entries" ] value.wire
    [
      ("version", encode_literal_4 value.version); ("entries", encode_history4_entries value.entries);
    ]

and encode_history4_entries (value : history4_entries) : Js.Json.t =
  Js.Json.array (Array.map encode_history4_entries_item value)

and encode_fragments (value : fragments) : Js.Json.t =
  wire_to_json
    ~known:
      [
        "nativeState38";
        "factions7";
        "factions11";
        "factions21";
        "factions24";
        "tiles11";
        "rule11";
        "delta19";
        "delta20";
        "delta21";
        "light23";
        "light28";
      ]
    value.wire
    [
      ("nativeState38", encode_fragments_native_state38 value.native_state38);
      ("factions7", encode_fragments_factions7 value.factions7);
      ("factions11", encode_fragments_factions7 value.factions11);
      ("factions21", encode_fragments_factions7 value.factions21);
      ("factions24", encode_fragments_factions7 value.factions24);
      ("tiles11", encode_fragments_tiles11 value.tiles11);
      ("rule11", encode_fragments_rule11 value.rule11);
      ("delta19", encode_fragments_delta19 value.delta19);
      ("delta20", encode_fragments_delta19 value.delta20);
      ("delta21", encode_fragments_delta19 value.delta21);
      ("light23", encode_fragments_light23 value.light23);
      ("light28", encode_fragments_light28 value.light28);
    ]

and encode_fragments_native_state38 (value : fragments_native_state38) : Js.Json.t =
  wire_to_json
    ~known:
      [
        "factions";
        "groundItems";
        "armies";
        "conditionMemory";
        "mapId";
        "mapRevision";
        "cols";
        "rows";
        "round";
        "turn";
        "gold";
        "winner";
        "nextId";
        "combatRng";
        "log";
        "tiles";
        "units";
        "campaign";
        "lastCombat";
      ]
    value.wire
    ([
       ("factions", encode_fragments_native_state38_factions value.factions);
       ("groundItems", encode_fragments_native_state38_ground_items value.ground_items);
       ("mapId", encode_json_string value.map_id);
       ("mapRevision", encode_json_number value.map_revision);
       ("cols", encode_json_number value.cols);
       ("rows", encode_json_number value.rows);
       ("round", encode_json_number value.round);
       ("turn", encode_save35_setup_first value.turn);
       ("gold", encode_fragments_native_state38_gold value.gold);
       ("winner", encode_fragments_native_state38_winner value.winner);
       ("nextId", encode_json_number value.next_id);
       ("combatRng", encode_json_number value.combat_rng);
       ("log", encode_fragments_native_state38_log value.log);
       ("tiles", encode_fragments_native_state38_tiles value.tiles);
       ("units", encode_fragments_native_state38_units value.units);
     ]
    @ (match value.armies with
      | None ->
          if List.mem "armies" value.wire.undefined_fields then [ ("armies", undefined) ] else []
      | Some x -> [ ("armies", encode_fragments_native_state38_armies_member1 x) ])
    @ (match value.condition_memory with
      | None ->
          if List.mem "conditionMemory" value.wire.undefined_fields then
            [ ("conditionMemory", undefined) ]
          else []
      | Some x -> [ ("conditionMemory", encode_fragments_native_state38_log x) ])
    @ (match value.campaign with
      | None ->
          if List.mem "campaign" value.wire.undefined_fields then [ ("campaign", undefined) ]
          else []
      | Some x -> [ ("campaign", encode_fragments_native_state38_campaign_member1 x) ])
    @
    match value.last_combat with
    | None ->
        if List.mem "lastCombat" value.wire.undefined_fields then [ ("lastCombat", undefined) ]
        else []
    | Some x -> [ ("lastCombat", encode_fragments_native_state38_last_combat x) ])

and encode_fragments_native_state38_factions (value : fragments_native_state38_factions) : Js.Json.t
    =
  Js.Json.array (Array.map encode_fragments_native_state38_factions_item value)

and encode_fragments_native_state38_factions_item (value : fragments_native_state38_factions_item) :
    Js.Json.t =
  wire_to_json
    ~known:[ "id"; "phase"; "controller"; "palette"; "name"; "recruits"; "relations"; "transfer" ]
    value.wire
    ([
       ("id", encode_json_string value.id);
       ("phase", encode_save35_setup_first value.phase);
       ("controller", encode_fragments_factions7_item_controller value.controller);
       ("palette", encode_json_string value.palette);
     ]
    @ (match value.name with
      | None -> if List.mem "name" value.wire.undefined_fields then [ ("name", undefined) ] else []
      | Some x -> [ ("name", encode_json_string x) ])
    @ (match value.recruits with
      | None ->
          if List.mem "recruits" value.wire.undefined_fields then [ ("recruits", undefined) ]
          else []
      | Some x -> [ ("recruits", encode_fragments_factions7_item_recruits_member1 x) ])
    @ (match value.relations with
      | None ->
          if List.mem "relations" value.wire.undefined_fields then [ ("relations", undefined) ]
          else []
      | Some x -> [ ("relations", encode_fragments_factions7_item_relations_member1 x) ])
    @
    match value.transfer with
    | None ->
        if List.mem "transfer" value.wire.undefined_fields then [ ("transfer", undefined) ] else []
    | Some x -> [ ("transfer", encode_fragments_native_state38_factions_item_transfer_member1 x) ])

and encode_fragments_native_state38_factions_item_transfer
    (value : fragments_native_state38_factions_item_transfer) : Js.Json.t =
  match value with
  | `Undefined -> undefined
  | `Object2 x -> encode_fragments_native_state38_factions_item_transfer_member1 x

and encode_fragments_native_state38_factions_item_transfer_member1
    (value : fragments_native_state38_factions_item_transfer_member1) : Js.Json.t =
  wire_to_json ~known:[ "to"; "when" ] value.wire
    [
      ("to", encode_json_string value.to_);
      ("when", encode_fragments_native_state38_tiles_item_fog_member1 value.when_);
    ]

and encode_fragments_native_state38_tiles_item_fog_member1
    (value : fragments_native_state38_tiles_item_fog_member1) : Js.Json.t =
  wire_to_json ~known:[ "condition"; "remember" ] value.wire
    ([
       ( "condition",
         encode_fragments_native_state38_tiles_item_fog_member1_condition value.condition );
     ]
    @
    match value.remember with
    | None ->
        if List.mem "remember" value.wire.undefined_fields then [ ("remember", undefined) ] else []
    | Some x -> [ ("remember", encode_fragments_rule11_remember x) ])

and encode_fragments_native_state38_tiles_item_fog_member1_condition
    (value : fragments_native_state38_tiles_item_fog_member1_condition) : Js.Json.t =
  match value with
  | `Owner x -> encode_fragments_rule11_condition_member0 x
  | `Proximity x -> encode_fragments_light28_member1_when__member1_condition_member1 x
  | `Visible_units x -> encode_fragments_rule11_condition_member1 x
  | `Has_units x -> encode_fragments_light28_member1_when__member1_condition_member3 x
  | `No_units x -> encode_fragments_light28_member1_when__member1_condition_member4 x
  | `Choice x -> encode_fragments_rule11_condition_member2 x
  | `Flag x -> encode_fragments_rule11_condition_member3 x
  | `Event x -> encode_fragments_rule11_condition_member4 x
  | `Round x -> encode_fragments_rule11_condition_member5 x
  | `All x -> encode_fragments_native_state38_tiles_item_fog_member1_condition_member9 x
  | `Any x -> encode_fragments_native_state38_tiles_item_fog_member1_condition_member10 x

and encode_fragments_native_state38_tiles_item_fog_member1_condition_member9
    (value : fragments_native_state38_tiles_item_fog_member1_condition_member9) : Js.Json.t =
  wire_to_json ~known:[ "kind"; "conditions" ] value.wire
    [
      ("kind", encode_literal_all value.kind);
      ( "conditions",
        encode_fragments_native_state38_tiles_item_fog_member1_condition_member9_conditions
          value.conditions );
    ]

and encode_fragments_native_state38_tiles_item_fog_member1_condition_member9_conditions
    (value : fragments_native_state38_tiles_item_fog_member1_condition_member9_conditions) :
    Js.Json.t =
  Js.Json.array (Array.map encode_fragments_native_state38_tiles_item_fog_member1_condition value)

and encode_fragments_native_state38_tiles_item_fog_member1_condition_member10
    (value : fragments_native_state38_tiles_item_fog_member1_condition_member10) : Js.Json.t =
  wire_to_json ~known:[ "kind"; "conditions" ] value.wire
    [
      ("kind", encode_literal_any value.kind);
      ( "conditions",
        encode_fragments_native_state38_tiles_item_fog_member1_condition_member9_conditions
          value.conditions );
    ]

and encode_fragments_native_state38_tiles (value : fragments_native_state38_tiles) : Js.Json.t =
  Js.Json.array (Array.map encode_fragments_native_state38_tiles_item value)

and encode_fragments_native_state38_tiles_item (value : fragments_native_state38_tiles_item) :
    Js.Json.t =
  wire_to_json
    ~known:
      [
        "id";
        "location";
        "col";
        "row";
        "terrain";
        "owner";
        "connections";
        "deadEnds";
        "walls";
        "floor";
        "fog";
        "light";
      ]
    value.wire
    ([
       ("id", encode_json_string value.id);
       ("col", encode_json_number value.col);
       ("row", encode_json_number value.row);
       ("terrain", encode_json_string value.terrain);
       ("owner", encode_fragments_tiles11_item_owner value.owner);
     ]
    @ (match value.location with
      | None ->
          if List.mem "location" value.wire.undefined_fields then [ ("location", undefined) ]
          else []
      | Some x -> [ ("location", encode_json_string x) ])
    @ (match value.connections with
      | None ->
          if List.mem "connections" value.wire.undefined_fields then [ ("connections", undefined) ]
          else []
      | Some x -> [ ("connections", encode_fragments_tiles11_item_connections_member1 x) ])
    @ (match value.dead_ends with
      | None ->
          if List.mem "deadEnds" value.wire.undefined_fields then [ ("deadEnds", undefined) ]
          else []
      | Some x -> [ ("deadEnds", encode_fragments_tiles11_item_connections_member1 x) ])
    @ (match value.walls with
      | None ->
          if List.mem "walls" value.wire.undefined_fields then [ ("walls", undefined) ] else []
      | Some x -> [ ("walls", encode_fragments_tiles11_item_connections_member1 x) ])
    @ (match value.floor with
      | None ->
          if List.mem "floor" value.wire.undefined_fields then [ ("floor", undefined) ] else []
      | Some x -> [ ("floor", encode_json_string x) ])
    @ (match value.fog with
      | None -> if List.mem "fog" value.wire.undefined_fields then [ ("fog", undefined) ] else []
      | Some x -> [ ("fog", encode_fragments_native_state38_tiles_item_fog_member1 x) ])
    @
    match value.light with
    | None -> if List.mem "light" value.wire.undefined_fields then [ ("light", undefined) ] else []
    | Some x -> [ ("light", encode_fragments_native_state38_tiles_item_light_member1 x) ])

and encode_fragments_native_state38_tiles_item_fog (value : fragments_native_state38_tiles_item_fog)
    : Js.Json.t =
  match value with
  | `Undefined -> undefined
  | `Object2 x -> encode_fragments_native_state38_tiles_item_fog_member1 x

and encode_fragments_native_state38_tiles_item_light
    (value : fragments_native_state38_tiles_item_light) : Js.Json.t =
  match value with
  | `Undefined -> undefined
  | `Object2 x -> encode_fragments_native_state38_tiles_item_light_member1 x

and encode_fragments_native_state38_tiles_item_light_member1
    (value : fragments_native_state38_tiles_item_light_member1) : Js.Json.t =
  wire_to_json
    ~known:[ "color"; "fixture"; "dx"; "dy"; "radius"; "power"; "when" ]
    value.wire
    ([ ("radius", encode_json_number value.radius); ("power", encode_json_number value.power) ]
    @ (match value.color with
      | None ->
          if List.mem "color" value.wire.undefined_fields then [ ("color", undefined) ] else []
      | Some x -> [ ("color", encode_fragments_light23_color_member1 x) ])
    @ (match value.fixture with
      | None ->
          if List.mem "fixture" value.wire.undefined_fields then [ ("fixture", undefined) ] else []
      | Some x -> [ ("fixture", encode_fragments_rule11_remember x) ])
    @ (match value.dx with
      | None -> if List.mem "dx" value.wire.undefined_fields then [ ("dx", undefined) ] else []
      | Some x -> [ ("dx", encode_json_number x) ])
    @ (match value.dy with
      | None -> if List.mem "dy" value.wire.undefined_fields then [ ("dy", undefined) ] else []
      | Some x -> [ ("dy", encode_json_number x) ])
    @
    match value.when_ with
    | None -> if List.mem "when" value.wire.undefined_fields then [ ("when", undefined) ] else []
    | Some x -> [ ("when", encode_fragments_native_state38_tiles_item_fog_member1 x) ])

and encode_fragments_native_state38_units (value : fragments_native_state38_units) : Js.Json.t =
  Js.Json.array (Array.map encode_fragments_native_state38_units_item value)

and encode_fragments_native_state38_units_item (value : fragments_native_state38_units_item) :
    Js.Json.t =
  wire_to_json
    ~known:
      [
        "faction";
        "items";
        "aiGroup";
        "id";
        "type";
        "side";
        "tile";
        "hp";
        "moved";
        "attacked";
        "experience";
        "character";
        "role";
        "retreating";
        "rebuilding";
      ]
    value.wire
    ([
       ("id", encode_json_string value.id);
       ("type", encode_json_string value.type_);
       ("side", encode_save35_setup_first value.side);
       ("tile", encode_json_string value.tile);
       ("hp", encode_json_number value.hp);
       ("moved", encode_json_boolean value.moved);
       ("attacked", encode_json_boolean value.attacked);
     ]
    @ (match value.faction with
      | None ->
          if List.mem "faction" value.wire.undefined_fields then [ ("faction", undefined) ] else []
      | Some x -> [ ("faction", encode_json_string x) ])
    @ (match value.items with
      | None ->
          if List.mem "items" value.wire.undefined_fields then [ ("items", undefined) ] else []
      | Some x -> [ ("items", encode_fragments_native_state38_units_item_items_member1 x) ])
    @ (match value.ai_group with
      | None ->
          if List.mem "aiGroup" value.wire.undefined_fields then [ ("aiGroup", undefined) ] else []
      | Some x -> [ ("aiGroup", encode_json_string x) ])
    @ (match value.experience with
      | None ->
          if List.mem "experience" value.wire.undefined_fields then [ ("experience", undefined) ]
          else []
      | Some x -> [ ("experience", encode_json_number x) ])
    @ (match value.character with
      | None ->
          if List.mem "character" value.wire.undefined_fields then [ ("character", undefined) ]
          else []
      | Some x -> [ ("character", encode_json_string x) ])
    @ (match value.role with
      | None -> if List.mem "role" value.wire.undefined_fields then [ ("role", undefined) ] else []
      | Some x -> [ ("role", encode_fragments_native_state38_units_item_role x) ])
    @ (match value.retreating with
      | None ->
          if List.mem "retreating" value.wire.undefined_fields then [ ("retreating", undefined) ]
          else []
      | Some x -> [ ("retreating", encode_fragments_rule11_remember x) ])
    @
    match value.rebuilding with
    | None ->
        if List.mem "rebuilding" value.wire.undefined_fields then [ ("rebuilding", undefined) ]
        else []
    | Some x -> [ ("rebuilding", encode_fragments_rule11_remember x) ])

and encode_fragments_native_state38_campaign (value : fragments_native_state38_campaign) : Js.Json.t
    =
  match value with
  | `Undefined -> undefined
  | `Object2 x -> encode_fragments_native_state38_campaign_member1 x

and encode_fragments_native_state38_campaign_member1
    (value : fragments_native_state38_campaign_member1) : Js.Json.t =
  wire_to_json
    ~known:
      [
        "scripts";
        "evacuated";
        "id";
        "mission";
        "recall";
        "veterans";
        "storyVeterans";
        "fallenCompanions";
        "carryGold";
        "marks";
        "dialogue";
        "dialogueIndex";
        "outcome";
        "completed";
        "refuge";
        "vigil";
        "holds";
        "flags";
      ]
    value.wire
    ([
       ("id", encode_json_string value.id);
       ("mission", encode_json_number value.mission);
       ("recall", encode_fragments_native_state38_log value.recall);
       ("carryGold", encode_json_number value.carry_gold);
       ("marks", encode_fragments_native_state38_log value.marks);
       ("dialogue", encode_fragments_native_state38_campaign_member1_dialogue value.dialogue);
       ("dialogueIndex", encode_json_number value.dialogue_index);
       ("outcome", encode_fragments_tiles11_item_owner value.outcome);
       ("completed", encode_json_boolean value.completed);
       ("flags", encode_fragments_native_state38_campaign_member1_flags value.flags);
     ]
    @ (match value.scripts with
      | None ->
          if List.mem "scripts" value.wire.undefined_fields then [ ("scripts", undefined) ] else []
      | Some x ->
          [ ("scripts", encode_fragments_native_state38_campaign_member1_scripts_member1 x) ])
    @ (match value.evacuated with
      | None ->
          if List.mem "evacuated" value.wire.undefined_fields then [ ("evacuated", undefined) ]
          else []
      | Some x ->
          [ ("evacuated", encode_fragments_native_state38_campaign_member1_evacuated_member1 x) ])
    @ (match value.veterans with
      | None ->
          if List.mem "veterans" value.wire.undefined_fields then [ ("veterans", undefined) ]
          else []
      | Some x ->
          [ ("veterans", encode_fragments_native_state38_campaign_member1_veterans_member1 x) ])
    @ (match value.story_veterans with
      | None ->
          if List.mem "storyVeterans" value.wire.undefined_fields then
            [ ("storyVeterans", undefined) ]
          else []
      | Some x ->
          [ ("storyVeterans", encode_fragments_native_state38_campaign_member1_veterans_member1 x) ])
    @ (match value.fallen_companions with
      | None ->
          if List.mem "fallenCompanions" value.wire.undefined_fields then
            [ ("fallenCompanions", undefined) ]
          else []
      | Some x -> [ ("fallenCompanions", encode_fragments_native_state38_log x) ])
    @ (match value.refuge with
      | None ->
          if List.mem "refuge" value.wire.undefined_fields then [ ("refuge", undefined) ] else []
      | Some x -> [ ("refuge", encode_fragments_native_state38_campaign_member1_refuge_member1 x) ])
    @ (match value.vigil with
      | None ->
          if List.mem "vigil" value.wire.undefined_fields then [ ("vigil", undefined) ] else []
      | Some x -> [ ("vigil", encode_fragments_native_state38_campaign_member1_vigil_member1 x) ])
    @
    match value.holds with
    | None -> if List.mem "holds" value.wire.undefined_fields then [ ("holds", undefined) ] else []
    | Some x -> [ ("holds", encode_fragments_native_state38_campaign_member1_holds_member1 x) ])

and encode_fragments_native_state38_campaign_member1_scripts
    (value : fragments_native_state38_campaign_member1_scripts) : Js.Json.t =
  match value with
  | `Undefined -> undefined
  | `Object2 x -> encode_fragments_native_state38_campaign_member1_scripts_member1 x

and encode_fragments_native_state38_campaign_member1_scripts_member1
    (value : fragments_native_state38_campaign_member1_scripts_member1) : Js.Json.t =
  wire_to_json
    ~known:
      [
        "failed";
        "completed";
        "revealed";
        "recruitment";
        "active";
        "holds";
        "marks";
        "pending";
        "nextMark";
      ]
    value.wire
    ([
       ("completed", encode_fragments_native_state38_log value.completed);
       ("revealed", encode_fragments_native_state38_log value.revealed);
       ( "recruitment",
         encode_fragments_native_state38_campaign_member1_scripts_member1_recruitment
           value.recruitment );
       ("active", encode_fragments_native_state38_log value.active);
       ("holds", encode_fragments_native_state38_campaign_member1_scripts_member1_holds value.holds);
       ("marks", encode_fragments_native_state38_campaign_member1_scripts_member1_marks value.marks);
       ( "pending",
         encode_fragments_native_state38_campaign_member1_scripts_member1_pending value.pending );
       ("nextMark", encode_json_number value.next_mark);
     ]
    @
    match value.failed with
    | None ->
        if List.mem "failed" value.wire.undefined_fields then [ ("failed", undefined) ] else []
    | Some x -> [ ("failed", encode_fragments_native_state38_log x) ])

and encode_fragments_native_state38_campaign_member1_scripts_member1_marks
    (value : fragments_native_state38_campaign_member1_scripts_member1_marks) : Js.Json.t =
  Js.Json.array
    (Array.map encode_fragments_native_state38_campaign_member1_scripts_member1_marks_item value)

and encode_fragments_native_state38_campaign_member1_scripts_member1_marks_item
    (value : fragments_native_state38_campaign_member1_scripts_member1_marks_item) : Js.Json.t =
  wire_to_json
    ~known:[ "lightRemembered"; "id"; "type"; "tile"; "light" ]
    value.wire
    ([
       ("id", encode_json_number value.id);
       ("type", encode_json_string value.type_);
       ("tile", encode_json_string value.tile);
     ]
    @ (match value.light_remembered with
      | None ->
          if List.mem "lightRemembered" value.wire.undefined_fields then
            [ ("lightRemembered", undefined) ]
          else []
      | Some x -> [ ("lightRemembered", encode_fragments_rule11_remember x) ])
    @
    match value.light with
    | None -> if List.mem "light" value.wire.undefined_fields then [ ("light", undefined) ] else []
    | Some x -> [ ("light", encode_fragments_native_state38_tiles_item_light_member1 x) ])

and encode_fragments_native_state38_campaign_member1_evacuated
    (value : fragments_native_state38_campaign_member1_evacuated) : Js.Json.t =
  match value with
  | `Undefined -> undefined
  | `Array x -> encode_fragments_native_state38_campaign_member1_evacuated_member1 x

and encode_fragments_native_state38_campaign_member1_evacuated_member1
    (value : fragments_native_state38_campaign_member1_evacuated_member1) : Js.Json.t =
  Js.Json.array
    (Array.map encode_fragments_native_state38_campaign_member1_evacuated_member1_item value)

and encode_fragments_native_state38_campaign_member1_evacuated_member1_item
    (value : fragments_native_state38_campaign_member1_evacuated_member1_item) : Js.Json.t =
  wire_to_json ~known:[ "event"; "unit" ] value.wire
    [
      ("event", encode_json_string value.event);
      ("unit", encode_fragments_native_state38_units_item value.unit_);
    ]

and encode_fragments_delta19 (value : fragments_delta19) : Js.Json.t =
  Js.Json.array (Array.map encode_fragments_delta19_item value)

and encode_fragments_delta19_item (value : fragments_delta19_item) : Js.Json.t =
  wire_to_json
    ~known:
      [
        "owner";
        "id";
        "location";
        "col";
        "row";
        "terrain";
        "connections";
        "deadEnds";
        "walls";
        "floor";
        "fog";
        "light";
      ]
    value.wire
    ([ ("id", encode_json_string value.id) ]
    @ (match value.owner with
      | None ->
          if List.mem "owner" value.wire.undefined_fields then [ ("owner", undefined) ] else []
      | Some x -> [ ("owner", encode_fragments_delta19_item_owner x) ])
    @ (match value.location with
      | None ->
          if List.mem "location" value.wire.undefined_fields then [ ("location", undefined) ]
          else []
      | Some x -> [ ("location", encode_json_string x) ])
    @ (match value.col with
      | None -> if List.mem "col" value.wire.undefined_fields then [ ("col", undefined) ] else []
      | Some x -> [ ("col", encode_json_number x) ])
    @ (match value.row with
      | None -> if List.mem "row" value.wire.undefined_fields then [ ("row", undefined) ] else []
      | Some x -> [ ("row", encode_json_number x) ])
    @ (match value.terrain with
      | None ->
          if List.mem "terrain" value.wire.undefined_fields then [ ("terrain", undefined) ] else []
      | Some x -> [ ("terrain", encode_json_string x) ])
    @ (match value.connections with
      | None ->
          if List.mem "connections" value.wire.undefined_fields then [ ("connections", undefined) ]
          else []
      | Some x -> [ ("connections", encode_fragments_tiles11_item_connections_member1 x) ])
    @ (match value.dead_ends with
      | None ->
          if List.mem "deadEnds" value.wire.undefined_fields then [ ("deadEnds", undefined) ]
          else []
      | Some x -> [ ("deadEnds", encode_fragments_tiles11_item_connections_member1 x) ])
    @ (match value.walls with
      | None ->
          if List.mem "walls" value.wire.undefined_fields then [ ("walls", undefined) ] else []
      | Some x -> [ ("walls", encode_fragments_tiles11_item_connections_member1 x) ])
    @ (match value.floor with
      | None ->
          if List.mem "floor" value.wire.undefined_fields then [ ("floor", undefined) ] else []
      | Some x -> [ ("floor", encode_json_string x) ])
    @ (match value.fog with
      | None -> if List.mem "fog" value.wire.undefined_fields then [ ("fog", undefined) ] else []
      | Some x -> [ ("fog", encode_fragments_rule11 x) ])
    @
    match value.light with
    | None -> if List.mem "light" value.wire.undefined_fields then [ ("light", undefined) ] else []
    | Some x -> [ ("light", encode_fragments_light23 x) ])

and encode_fragments_light28 (value : fragments_light28) : Js.Json.t =
  match value with `Null -> json_null | `Object2 x -> encode_fragments_light28_member1 x

and encode_native1 (value : native1) : Js.Json.t =
  wire_to_json
    ~known:[ "engine"; "version"; "pendingEnemy"; "saved" ]
    value.wire
    ([
       ("engine", encode_literal_rust_wasm value.engine);
       ("pendingEnemy", encode_json_boolean value.pending_enemy);
       ("saved", encode_native1_saved value.saved);
     ]
    @
    match value.version with
    | None ->
        if List.mem "version" value.wire.undefined_fields then [ ("version", undefined) ] else []
    | Some x -> [ ("version", encode_literal_1 x) ])

and encode_literal_rust_wasm (value : literal_rust_wasm) : Js.Json.t =
  Js.Json.parseExn "\"rust-wasm\""

and encode_native1_version (value : native1_version) : Js.Json.t =
  match value with `Undefined -> undefined | `V1 -> Js.Json.parseExn "1"

and encode_native1_saved (value : native1_saved) : Js.Json.t =
  wire_to_json ~known:[ "seed"; "state"; "version" ] value.wire
    [
      ("seed", encode_json_number value.seed);
      ("state", encode_save2_checkpoint_state value.state);
      ("version", encode_literal_1 value.version);
    ]

and encode_javascript1 (value : javascript1) : Js.Json.t =
  wire_to_json
    ~known:[ "version"; "pendingEnemy"; "campaignDeploymentFormat"; "state" ]
    value.wire
    ([
       ("version", encode_literal_1 value.version);
       ("pendingEnemy", encode_json_boolean value.pending_enemy);
       ("state", encode_javascript1_state value.state);
     ]
    @
    match value.campaign_deployment_format with
    | None ->
        if List.mem "campaignDeploymentFormat" value.wire.undefined_fields then
          [ ("campaignDeploymentFormat", undefined) ]
        else []
    | Some x -> [ ("campaignDeploymentFormat", encode_literal_named_waves_v1 x) ])

and encode_javascript1_campaign_deployment_format (value : javascript1_campaign_deployment_format) :
    Js.Json.t =
  match value with
  | `Undefined -> undefined
  | `Named_waves_v1 -> Js.Json.parseExn "\"named-waves-v1\""

and encode_literal_named_waves_v1 (value : literal_named_waves_v1) : Js.Json.t =
  Js.Json.parseExn "\"named-waves-v1\""

and encode_javascript1_state (value : javascript1_state) : Js.Json.t =
  wire_to_json
    ~known:
      [
        "mapId";
        "mapRevision";
        "round";
        "turn";
        "winner";
        "gold";
        "nextId";
        "log";
        "units";
        "villages";
        "sites";
        "campaign";
        "lastCombat";
      ]
    value.wire
    ([
       ("round", encode_json_number value.round);
       ("turn", encode_save35_setup_first value.turn);
       ("winner", encode_fragments_native_state38_winner value.winner);
       ("gold", encode_javascript1_state_gold value.gold);
       ("nextId", encode_json_number value.next_id);
       ("log", encode_fragments_native_state38_log value.log);
       ("units", encode_javascript1_state_units value.units);
       ("villages", encode_javascript1_state_villages value.villages);
     ]
    @ (match value.map_id with
      | None ->
          if List.mem "mapId" value.wire.undefined_fields then [ ("mapId", undefined) ] else []
      | Some x -> [ ("mapId", encode_json_string x) ])
    @ (match value.map_revision with
      | None ->
          if List.mem "mapRevision" value.wire.undefined_fields then [ ("mapRevision", undefined) ]
          else []
      | Some x -> [ ("mapRevision", encode_javascript1_state_map_revision x) ])
    @ (match value.sites with
      | None ->
          if List.mem "sites" value.wire.undefined_fields then [ ("sites", undefined) ] else []
      | Some x -> [ ("sites", encode_javascript1_state_villages x) ])
    @ (match value.campaign with
      | None ->
          if List.mem "campaign" value.wire.undefined_fields then [ ("campaign", undefined) ]
          else []
      | Some x -> [ ("campaign", encode_javascript1_state_campaign_member1 x) ])
    @
    match value.last_combat with
    | None ->
        if List.mem "lastCombat" value.wire.undefined_fields then [ ("lastCombat", undefined) ]
        else []
    | Some x -> [ ("lastCombat", encode_fragments_native_state38_last_combat x) ])

and encode_javascript1_state_map_revision (value : javascript1_state_map_revision) : Js.Json.t =
  match value with
  | `Undefined -> undefined
  | `Null -> json_null
  | `Number x -> encode_json_number x

and encode_javascript1_state_units (value : javascript1_state_units) : Js.Json.t =
  Js.Json.array (Array.map encode_javascript1_state_units_item value)

and encode_javascript1_state_units_item (value : javascript1_state_units_item) : Js.Json.t =
  wire_to_json
    ~known:
      [
        "tile";
        "id";
        "side";
        "name";
        "role";
        "character";
        "faction";
        "items";
        "aiGroup";
        "type";
        "hp";
        "moved";
        "attacked";
        "retreating";
        "rebuilding";
        "experience";
      ]
    value.wire
    ([
       ("tile", encode_json_string value.tile);
       ("id", encode_json_string value.id);
       ("side", encode_save35_setup_first value.side);
       ("type", encode_javascript1_state_units_item_type_ value.type_);
       ("hp", encode_json_number value.hp);
       ("moved", encode_json_boolean value.moved);
       ("attacked", encode_json_boolean value.attacked);
     ]
    @ (match value.name with
      | None -> if List.mem "name" value.wire.undefined_fields then [ ("name", undefined) ] else []
      | Some x -> [ ("name", encode_json_string x) ])
    @ (match value.role with
      | None -> if List.mem "role" value.wire.undefined_fields then [ ("role", undefined) ] else []
      | Some x -> [ ("role", encode_fragments_native_state38_units_item_role x) ])
    @ (match value.character with
      | None ->
          if List.mem "character" value.wire.undefined_fields then [ ("character", undefined) ]
          else []
      | Some x -> [ ("character", encode_json_string x) ])
    @ (match value.faction with
      | None ->
          if List.mem "faction" value.wire.undefined_fields then [ ("faction", undefined) ] else []
      | Some x -> [ ("faction", encode_json_string x) ])
    @ (match value.items with
      | None ->
          if List.mem "items" value.wire.undefined_fields then [ ("items", undefined) ] else []
      | Some x -> [ ("items", encode_javascript1_state_units_item_items_member1 x) ])
    @ (match value.ai_group with
      | None ->
          if List.mem "aiGroup" value.wire.undefined_fields then [ ("aiGroup", undefined) ] else []
      | Some x -> [ ("aiGroup", encode_json_string x) ])
    @ (match value.retreating with
      | None ->
          if List.mem "retreating" value.wire.undefined_fields then [ ("retreating", undefined) ]
          else []
      | Some x -> [ ("retreating", encode_fragments_rule11_remember x) ])
    @ (match value.rebuilding with
      | None ->
          if List.mem "rebuilding" value.wire.undefined_fields then [ ("rebuilding", undefined) ]
          else []
      | Some x -> [ ("rebuilding", encode_fragments_rule11_remember x) ])
    @
    match value.experience with
    | None ->
        if List.mem "experience" value.wire.undefined_fields then [ ("experience", undefined) ]
        else []
    | Some x -> [ ("experience", encode_json_number x) ])

and encode_javascript1_state_units_item_items (value : javascript1_state_units_item_items) :
    Js.Json.t =
  match value with
  | `Undefined -> undefined
  | `Array x -> encode_javascript1_state_units_item_items_member1 x

and encode_javascript1_state_units_item_items_member1
    (value : javascript1_state_units_item_items_member1) : Js.Json.t =
  Js.Json.array (Array.map encode_javascript1_state_units_item_items_member1_item value)

and encode_javascript1_state_units_item_items_member1_item
    (value : javascript1_state_units_item_items_member1_item) : Js.Json.t =
  wire_to_json
    ~known:[ "id"; "armor"; "name"; "description"; "healing"; "attacks" ]
    value.wire
    ([
       ("id", encode_json_string value.id);
       ("name", encode_json_string value.name);
       ("description", encode_json_string value.description);
       ("attacks", encode_javascript1_state_units_item_items_member1_item_attacks value.attacks);
     ]
    @ (match value.armor with
      | None ->
          if List.mem "armor" value.wire.undefined_fields then [ ("armor", undefined) ] else []
      | Some x ->
          [ ("armor", encode_fragments_native_state38_ground_items_item_item_armor_member1 x) ])
    @
    match value.healing with
    | None ->
        if List.mem "healing" value.wire.undefined_fields then [ ("healing", undefined) ] else []
    | Some x ->
        [ ("healing", encode_fragments_native_state38_ground_items_item_item_healing_member1 x) ])

and encode_javascript1_state_units_item_items_member1_item_attacks
    (value : javascript1_state_units_item_items_member1_item_attacks) : Js.Json.t =
  Js.Json.array
    (Array.map encode_javascript1_state_units_item_items_member1_item_attacks_item value)

and encode_javascript1_state_units_item_items_member1_item_attacks_item
    (value : javascript1_state_units_item_items_member1_item_attacks_item) : Js.Json.t =
  wire_to_json
    ~known:
      [
        "spread";
        "drain";
        "count";
        "id";
        "name";
        "damage";
        "damageType";
        "category";
        "range";
        "sound";
      ]
    value.wire
    ([
       ("count", encode_json_number value.count);
       ("id", encode_json_string value.id);
       ("name", encode_json_string value.name);
       ("damage", encode_json_number value.damage);
       ( "damageType",
         encode_fragments_native_state38_ground_items_item_item_attacks_item_damage_type
           value.damage_type );
       ( "category",
         encode_fragments_native_state38_ground_items_item_item_attacks_item_category value.category
       );
       ("range", encode_json_number value.range);
       ("sound", encode_json_string value.sound);
     ]
    @ (match value.spread with
      | None ->
          if List.mem "spread" value.wire.undefined_fields then [ ("spread", undefined) ] else []
      | Some x -> [ ("spread", encode_json_number x) ])
    @
    match value.drain with
    | None -> if List.mem "drain" value.wire.undefined_fields then [ ("drain", undefined) ] else []
    | Some x -> [ ("drain", encode_json_number x) ])

and encode_javascript1_state_units_item_type_ (value : javascript1_state_units_item_type_) :
    Js.Json.t =
  match value with
  | `Ranger -> Js.Json.parseExn "\"ranger\""
  | `Healer -> Js.Json.parseExn "\"healer\""
  | `Briarweaver -> Js.Json.parseExn "\"briarweaver\""
  | `Deatharcher -> Js.Json.parseExn "\"deatharcher\""
  | `Direwolf -> Js.Json.parseExn "\"direwolf\""
  | `Greatboar -> Js.Json.parseExn "\"greatboar\""
  | `Wraith -> Js.Json.parseExn "\"wraith\""
  | `Spectre -> Js.Json.parseExn "\"spectre\""
  | `Lancer -> Js.Json.parseExn "\"lancer\""
  | `Mage -> Js.Json.parseExn "\"mage\""
  | `Marksman -> Js.Json.parseExn "\"marksman\""
  | `Merfolk_maceguard_l2 -> Js.Json.parseExn "\"merfolk-maceguard-l2\""
  | `Revenant -> Js.Json.parseExn "\"revenant\""
  | `Skeleton_impact_caster_l2 -> Js.Json.parseExn "\"skeleton-impact-caster-l2\""
  | `Soldier -> Js.Json.parseExn "\"soldier\""
  | `Tidehunter -> Js.Json.parseExn "\"tidehunter\""
  | `Tidesage -> Js.Json.parseExn "\"tidesage\""
  | `Tridentguard -> Js.Json.parseExn "\"tridentguard\""
  | `Warden -> Js.Json.parseExn "\"warden\""
  | `Champion -> Js.Json.parseExn "\"champion\""
  | `Hammerman -> Js.Json.parseExn "\"hammerman\""
  | `Warhammer -> Js.Json.parseExn "\"warhammer\""
  | `Sentinel -> Js.Json.parseExn "\"sentinel\""
  | `Halberdier -> Js.Json.parseExn "\"halberdier\""
  | `Halberd_master -> Js.Json.parseExn "\"halberd-master\""
  | `Shadowblade -> Js.Json.parseExn "\"shadowblade\""
  | `Slinger -> Js.Json.parseExn "\"slinger\""
  | `Fireslinger -> Js.Json.parseExn "\"fireslinger\""
  | `Sharpshooter -> Js.Json.parseExn "\"sharpshooter\""
  | `Crossbowman -> Js.Json.parseExn "\"crossbowman\""
  | `Arbalester -> Js.Json.parseExn "\"arbalester\""
  | `Archmage -> Js.Json.parseExn "\"archmage\""
  | `High_healer -> Js.Json.parseExn "\"high-healer\""
  | `Knight -> Js.Json.parseExn "\"knight\""
  | `Horse_bowman -> Js.Json.parseExn "\"horse-bowman\""
  | `Horse_marksman -> Js.Json.parseExn "\"horse-marksman\""
  | `Skeleton_dreadguard -> Js.Json.parseExn "\"skeleton-dreadguard\""
  | `Skeleton_flanker_l2 -> Js.Json.parseExn "\"skeleton-flanker-l2\""
  | `Skeleton_flanker_l3 -> Js.Json.parseExn "\"skeleton-flanker-l3\""
  | `Skeleton_impact_caster_l3 -> Js.Json.parseExn "\"skeleton-impact-caster-l3\""
  | `Skeleton_scythe_caster_l2 -> Js.Json.parseExn "\"skeleton-scythe-caster-l2\""
  | `Skeleton_scythe_caster_l3 -> Js.Json.parseExn "\"skeleton-scythe-caster-l3\""
  | `Skeleton_rider_l2 -> Js.Json.parseExn "\"skeleton-rider-l2\""
  | `Skeleton_rider_l3 -> Js.Json.parseExn "\"skeleton-rider-l3\""
  | `Skeleton_bow_master -> Js.Json.parseExn "\"skeleton-bow-master\""
  | `Skeleton_axeman_l2 -> Js.Json.parseExn "\"skeleton-axeman-l2\""
  | `Skeleton_axeman_l3 -> Js.Json.parseExn "\"skeleton-axeman-l3\""
  | `Woodland_alpha_wolf -> Js.Json.parseExn "\"woodland-alpha-wolf\""
  | `Woodland_ancient_boar -> Js.Json.parseExn "\"woodland-ancient-boar\""
  | `Woodland_thorn_oracle -> Js.Json.parseExn "\"woodland-thorn-oracle\""
  | `Woodland_bramble_warden -> Js.Json.parseExn "\"woodland-bramble-warden\""
  | `Woodland_thornbinder -> Js.Json.parseExn "\"woodland-thornbinder\""
  | `Fox_spearguard_l2 -> Js.Json.parseExn "\"fox-spearguard-l2\""
  | `Fox_spearguard_l3 -> Js.Json.parseExn "\"fox-spearguard-l3\""
  | `Treant_l2 -> Js.Json.parseExn "\"treant-l2\""
  | `Treant_l3 -> Js.Json.parseExn "\"treant-l3\""
  | `Merfolk_harpoon_master -> Js.Json.parseExn "\"merfolk-harpoon-master\""
  | `Merfolk_netter_l2 -> Js.Json.parseExn "\"merfolk-netter-l2\""
  | `Merfolk_netter_l3 -> Js.Json.parseExn "\"merfolk-netter-l3\""
  | `Merfolk_reef_champion -> Js.Json.parseExn "\"merfolk-reef-champion\""
  | `Merfolk_trident_l2 -> Js.Json.parseExn "\"merfolk-trident-l2\""
  | `Merfolk_trident_l3 -> Js.Json.parseExn "\"merfolk-trident-l3\""
  | `Merfolk_tide_oracle -> Js.Json.parseExn "\"merfolk-tide-oracle\""
  | `Merfolk_tidecrusher_l2 -> Js.Json.parseExn "\"merfolk-tidecrusher-l2\""
  | `Merfolk_tidecrusher_l3 -> Js.Json.parseExn "\"merfolk-tidecrusher-l3\""
  | `Merfolk_maceguard_l3 -> Js.Json.parseExn "\"merfolk-maceguard-l3\""
  | `Dragon -> Js.Json.parseExn "\"dragon\""
  | `Skeletal_hound -> Js.Json.parseExn "\"skeletal-hound\""
  | `Captives -> Js.Json.parseExn "\"captives\""
  | `Zombie_walker -> Js.Json.parseExn "\"zombie-walker\""
  | `Zombie_decay_caster -> Js.Json.parseExn "\"zombie-decay-caster\""
  | `Bear -> Js.Json.parseExn "\"bear\""
  | `Giant_bat -> Js.Json.parseExn "\"giant-bat\""
  | `Giant_spider -> Js.Json.parseExn "\"giant-spider\""
  | `Commander -> Js.Json.parseExn "\"commander\""
  | `Grovewarden -> Js.Json.parseExn "\"grovewarden\""
  | `Lich -> Js.Json.parseExn "\"lich\""
  | `Marshal -> Js.Json.parseExn "\"marshal\""
  | `Necromancer -> Js.Json.parseExn "\"necromancer\""
  | `Tidekeeper -> Js.Json.parseExn "\"tidekeeper\""
  | `Tideseer -> Js.Json.parseExn "\"tideseer\""
  | `Wildkeeper -> Js.Json.parseExn "\"wildkeeper\""
  | `Grand_marshal -> Js.Json.parseExn "\"grand-marshal\""
  | `Lich_commander_l3 -> Js.Json.parseExn "\"lich-commander-l3\""
  | `Woodland_grove_oracle -> Js.Json.parseExn "\"woodland-grove-oracle\""
  | `Merfolk_tide_sovereign -> Js.Json.parseExn "\"merfolk-tide-sovereign\""
  | `Adept -> Js.Json.parseExn "\"adept\""
  | `Archer -> Js.Json.parseExn "\"archer\""
  | `Boar -> Js.Json.parseExn "\"boar\""
  | `Bonearcher -> Js.Json.parseExn "\"bonearcher\""
  | `Ghost -> Js.Json.parseExn "\"ghost\""
  | `Harpooner -> Js.Json.parseExn "\"harpooner\""
  | `Merfolk_maceguard -> Js.Json.parseExn "\"merfolk-maceguard\""
  | `Reefguard -> Js.Json.parseExn "\"reefguard\""
  | `Rider -> Js.Json.parseExn "\"rider\""
  | `Scout -> Js.Json.parseExn "\"scout\""
  | `Skeleton_impact_caster -> Js.Json.parseExn "\"skeleton-impact-caster\""
  | `Skeleton -> Js.Json.parseExn "\"skeleton\""
  | `Spearman -> Js.Json.parseExn "\"spearman\""
  | `Swordsman -> Js.Json.parseExn "\"swordsman\""
  | `Thorncaller -> Js.Json.parseExn "\"thorncaller\""
  | `Wavemender -> Js.Json.parseExn "\"wavemender\""
  | `Wolf -> Js.Json.parseExn "\"wolf\""
  | `Horse_archer -> Js.Json.parseExn "\"horse-archer\""
  | `Skeleton_rider -> Js.Json.parseExn "\"skeleton-rider\""
  | `Skeleton_axeman -> Js.Json.parseExn "\"skeleton-axeman\""
  | `Fox_spearguard -> Js.Json.parseExn "\"fox-spearguard\""
  | `Treant -> Js.Json.parseExn "\"treant\""

and encode_literal_ranger (value : literal_ranger) : Js.Json.t = Js.Json.parseExn "\"ranger\""
and encode_literal_healer (value : literal_healer) : Js.Json.t = Js.Json.parseExn "\"healer\""

and encode_literal_briarweaver (value : literal_briarweaver) : Js.Json.t =
  Js.Json.parseExn "\"briarweaver\""

and encode_literal_deatharcher (value : literal_deatharcher) : Js.Json.t =
  Js.Json.parseExn "\"deatharcher\""

and encode_literal_direwolf (value : literal_direwolf) : Js.Json.t = Js.Json.parseExn "\"direwolf\""

and encode_literal_greatboar (value : literal_greatboar) : Js.Json.t =
  Js.Json.parseExn "\"greatboar\""

and encode_literal_wraith (value : literal_wraith) : Js.Json.t = Js.Json.parseExn "\"wraith\""
and encode_literal_spectre (value : literal_spectre) : Js.Json.t = Js.Json.parseExn "\"spectre\""
and encode_literal_lancer (value : literal_lancer) : Js.Json.t = Js.Json.parseExn "\"lancer\""
and encode_literal_mage (value : literal_mage) : Js.Json.t = Js.Json.parseExn "\"mage\""
and encode_literal_marksman (value : literal_marksman) : Js.Json.t = Js.Json.parseExn "\"marksman\""

and encode_literal_merfolk_maceguard_l2 (value : literal_merfolk_maceguard_l2) : Js.Json.t =
  Js.Json.parseExn "\"merfolk-maceguard-l2\""

and encode_literal_revenant (value : literal_revenant) : Js.Json.t = Js.Json.parseExn "\"revenant\""

and encode_literal_skeleton_impact_caster_l2 (value : literal_skeleton_impact_caster_l2) : Js.Json.t
    =
  Js.Json.parseExn "\"skeleton-impact-caster-l2\""

and encode_literal_soldier (value : literal_soldier) : Js.Json.t = Js.Json.parseExn "\"soldier\""

and encode_literal_tidehunter (value : literal_tidehunter) : Js.Json.t =
  Js.Json.parseExn "\"tidehunter\""

and encode_literal_tidesage (value : literal_tidesage) : Js.Json.t = Js.Json.parseExn "\"tidesage\""

and encode_literal_tridentguard (value : literal_tridentguard) : Js.Json.t =
  Js.Json.parseExn "\"tridentguard\""

and encode_literal_warden (value : literal_warden) : Js.Json.t = Js.Json.parseExn "\"warden\""
and encode_literal_champion (value : literal_champion) : Js.Json.t = Js.Json.parseExn "\"champion\""

and encode_literal_hammerman (value : literal_hammerman) : Js.Json.t =
  Js.Json.parseExn "\"hammerman\""

and encode_literal_warhammer (value : literal_warhammer) : Js.Json.t =
  Js.Json.parseExn "\"warhammer\""

and encode_literal_sentinel (value : literal_sentinel) : Js.Json.t = Js.Json.parseExn "\"sentinel\""

and encode_literal_halberdier (value : literal_halberdier) : Js.Json.t =
  Js.Json.parseExn "\"halberdier\""

and encode_literal_halberd_master (value : literal_halberd_master) : Js.Json.t =
  Js.Json.parseExn "\"halberd-master\""

and encode_literal_shadowblade (value : literal_shadowblade) : Js.Json.t =
  Js.Json.parseExn "\"shadowblade\""

and encode_literal_slinger (value : literal_slinger) : Js.Json.t = Js.Json.parseExn "\"slinger\""

and encode_literal_fireslinger (value : literal_fireslinger) : Js.Json.t =
  Js.Json.parseExn "\"fireslinger\""

and encode_literal_sharpshooter (value : literal_sharpshooter) : Js.Json.t =
  Js.Json.parseExn "\"sharpshooter\""

and encode_literal_crossbowman (value : literal_crossbowman) : Js.Json.t =
  Js.Json.parseExn "\"crossbowman\""

and encode_literal_arbalester (value : literal_arbalester) : Js.Json.t =
  Js.Json.parseExn "\"arbalester\""

and encode_literal_archmage (value : literal_archmage) : Js.Json.t = Js.Json.parseExn "\"archmage\""

and encode_literal_high_healer (value : literal_high_healer) : Js.Json.t =
  Js.Json.parseExn "\"high-healer\""

and encode_literal_knight (value : literal_knight) : Js.Json.t = Js.Json.parseExn "\"knight\""

and encode_literal_horse_bowman (value : literal_horse_bowman) : Js.Json.t =
  Js.Json.parseExn "\"horse-bowman\""

and encode_literal_horse_marksman (value : literal_horse_marksman) : Js.Json.t =
  Js.Json.parseExn "\"horse-marksman\""

and encode_literal_skeleton_dreadguard (value : literal_skeleton_dreadguard) : Js.Json.t =
  Js.Json.parseExn "\"skeleton-dreadguard\""

and encode_literal_skeleton_flanker_l2 (value : literal_skeleton_flanker_l2) : Js.Json.t =
  Js.Json.parseExn "\"skeleton-flanker-l2\""

and encode_literal_skeleton_flanker_l3 (value : literal_skeleton_flanker_l3) : Js.Json.t =
  Js.Json.parseExn "\"skeleton-flanker-l3\""

and encode_literal_skeleton_impact_caster_l3 (value : literal_skeleton_impact_caster_l3) : Js.Json.t
    =
  Js.Json.parseExn "\"skeleton-impact-caster-l3\""

and encode_literal_skeleton_scythe_caster_l2 (value : literal_skeleton_scythe_caster_l2) : Js.Json.t
    =
  Js.Json.parseExn "\"skeleton-scythe-caster-l2\""

and encode_literal_skeleton_scythe_caster_l3 (value : literal_skeleton_scythe_caster_l3) : Js.Json.t
    =
  Js.Json.parseExn "\"skeleton-scythe-caster-l3\""

and encode_literal_skeleton_rider_l2 (value : literal_skeleton_rider_l2) : Js.Json.t =
  Js.Json.parseExn "\"skeleton-rider-l2\""

and encode_literal_skeleton_rider_l3 (value : literal_skeleton_rider_l3) : Js.Json.t =
  Js.Json.parseExn "\"skeleton-rider-l3\""

and encode_literal_skeleton_bow_master (value : literal_skeleton_bow_master) : Js.Json.t =
  Js.Json.parseExn "\"skeleton-bow-master\""

and encode_literal_skeleton_axeman_l2 (value : literal_skeleton_axeman_l2) : Js.Json.t =
  Js.Json.parseExn "\"skeleton-axeman-l2\""

and encode_literal_skeleton_axeman_l3 (value : literal_skeleton_axeman_l3) : Js.Json.t =
  Js.Json.parseExn "\"skeleton-axeman-l3\""

and encode_literal_woodland_alpha_wolf (value : literal_woodland_alpha_wolf) : Js.Json.t =
  Js.Json.parseExn "\"woodland-alpha-wolf\""

and encode_literal_woodland_ancient_boar (value : literal_woodland_ancient_boar) : Js.Json.t =
  Js.Json.parseExn "\"woodland-ancient-boar\""

and encode_literal_woodland_thorn_oracle (value : literal_woodland_thorn_oracle) : Js.Json.t =
  Js.Json.parseExn "\"woodland-thorn-oracle\""

and encode_literal_woodland_bramble_warden (value : literal_woodland_bramble_warden) : Js.Json.t =
  Js.Json.parseExn "\"woodland-bramble-warden\""

and encode_literal_woodland_thornbinder (value : literal_woodland_thornbinder) : Js.Json.t =
  Js.Json.parseExn "\"woodland-thornbinder\""

and encode_literal_fox_spearguard_l2 (value : literal_fox_spearguard_l2) : Js.Json.t =
  Js.Json.parseExn "\"fox-spearguard-l2\""

and encode_literal_fox_spearguard_l3 (value : literal_fox_spearguard_l3) : Js.Json.t =
  Js.Json.parseExn "\"fox-spearguard-l3\""

and encode_literal_treant_l2 (value : literal_treant_l2) : Js.Json.t =
  Js.Json.parseExn "\"treant-l2\""

and encode_literal_treant_l3 (value : literal_treant_l3) : Js.Json.t =
  Js.Json.parseExn "\"treant-l3\""

and encode_literal_merfolk_harpoon_master (value : literal_merfolk_harpoon_master) : Js.Json.t =
  Js.Json.parseExn "\"merfolk-harpoon-master\""

and encode_literal_merfolk_netter_l2 (value : literal_merfolk_netter_l2) : Js.Json.t =
  Js.Json.parseExn "\"merfolk-netter-l2\""

and encode_literal_merfolk_netter_l3 (value : literal_merfolk_netter_l3) : Js.Json.t =
  Js.Json.parseExn "\"merfolk-netter-l3\""

and encode_literal_merfolk_reef_champion (value : literal_merfolk_reef_champion) : Js.Json.t =
  Js.Json.parseExn "\"merfolk-reef-champion\""

and encode_literal_merfolk_trident_l2 (value : literal_merfolk_trident_l2) : Js.Json.t =
  Js.Json.parseExn "\"merfolk-trident-l2\""

and encode_literal_merfolk_trident_l3 (value : literal_merfolk_trident_l3) : Js.Json.t =
  Js.Json.parseExn "\"merfolk-trident-l3\""

and encode_literal_merfolk_tide_oracle (value : literal_merfolk_tide_oracle) : Js.Json.t =
  Js.Json.parseExn "\"merfolk-tide-oracle\""

and encode_literal_merfolk_tidecrusher_l2 (value : literal_merfolk_tidecrusher_l2) : Js.Json.t =
  Js.Json.parseExn "\"merfolk-tidecrusher-l2\""

and encode_literal_merfolk_tidecrusher_l3 (value : literal_merfolk_tidecrusher_l3) : Js.Json.t =
  Js.Json.parseExn "\"merfolk-tidecrusher-l3\""

and encode_literal_merfolk_maceguard_l3 (value : literal_merfolk_maceguard_l3) : Js.Json.t =
  Js.Json.parseExn "\"merfolk-maceguard-l3\""

and encode_literal_dragon (value : literal_dragon) : Js.Json.t = Js.Json.parseExn "\"dragon\""

and encode_literal_skeletal_hound (value : literal_skeletal_hound) : Js.Json.t =
  Js.Json.parseExn "\"skeletal-hound\""

and encode_literal_captives (value : literal_captives) : Js.Json.t = Js.Json.parseExn "\"captives\""

and encode_literal_zombie_walker (value : literal_zombie_walker) : Js.Json.t =
  Js.Json.parseExn "\"zombie-walker\""

and encode_literal_zombie_decay_caster (value : literal_zombie_decay_caster) : Js.Json.t =
  Js.Json.parseExn "\"zombie-decay-caster\""

and encode_literal_bear (value : literal_bear) : Js.Json.t = Js.Json.parseExn "\"bear\""

and encode_literal_giant_bat (value : literal_giant_bat) : Js.Json.t =
  Js.Json.parseExn "\"giant-bat\""

and encode_literal_giant_spider (value : literal_giant_spider) : Js.Json.t =
  Js.Json.parseExn "\"giant-spider\""

and encode_literal_commander (value : literal_commander) : Js.Json.t =
  Js.Json.parseExn "\"commander\""

and encode_literal_grovewarden (value : literal_grovewarden) : Js.Json.t =
  Js.Json.parseExn "\"grovewarden\""

and encode_literal_lich (value : literal_lich) : Js.Json.t = Js.Json.parseExn "\"lich\""
and encode_literal_marshal (value : literal_marshal) : Js.Json.t = Js.Json.parseExn "\"marshal\""

and encode_literal_necromancer (value : literal_necromancer) : Js.Json.t =
  Js.Json.parseExn "\"necromancer\""

and encode_literal_tidekeeper (value : literal_tidekeeper) : Js.Json.t =
  Js.Json.parseExn "\"tidekeeper\""

and encode_literal_tideseer (value : literal_tideseer) : Js.Json.t = Js.Json.parseExn "\"tideseer\""

and encode_literal_wildkeeper (value : literal_wildkeeper) : Js.Json.t =
  Js.Json.parseExn "\"wildkeeper\""

and encode_literal_grand_marshal (value : literal_grand_marshal) : Js.Json.t =
  Js.Json.parseExn "\"grand-marshal\""

and encode_literal_lich_commander_l3 (value : literal_lich_commander_l3) : Js.Json.t =
  Js.Json.parseExn "\"lich-commander-l3\""

and encode_literal_woodland_grove_oracle (value : literal_woodland_grove_oracle) : Js.Json.t =
  Js.Json.parseExn "\"woodland-grove-oracle\""

and encode_literal_merfolk_tide_sovereign (value : literal_merfolk_tide_sovereign) : Js.Json.t =
  Js.Json.parseExn "\"merfolk-tide-sovereign\""

and encode_literal_adept (value : literal_adept) : Js.Json.t = Js.Json.parseExn "\"adept\""
and encode_literal_archer (value : literal_archer) : Js.Json.t = Js.Json.parseExn "\"archer\""
and encode_literal_boar (value : literal_boar) : Js.Json.t = Js.Json.parseExn "\"boar\""

and encode_literal_bonearcher (value : literal_bonearcher) : Js.Json.t =
  Js.Json.parseExn "\"bonearcher\""

and encode_literal_ghost (value : literal_ghost) : Js.Json.t = Js.Json.parseExn "\"ghost\""

and encode_literal_harpooner (value : literal_harpooner) : Js.Json.t =
  Js.Json.parseExn "\"harpooner\""

and encode_literal_merfolk_maceguard (value : literal_merfolk_maceguard) : Js.Json.t =
  Js.Json.parseExn "\"merfolk-maceguard\""

and encode_literal_reefguard (value : literal_reefguard) : Js.Json.t =
  Js.Json.parseExn "\"reefguard\""

and encode_literal_rider (value : literal_rider) : Js.Json.t = Js.Json.parseExn "\"rider\""
and encode_literal_scout (value : literal_scout) : Js.Json.t = Js.Json.parseExn "\"scout\""

and encode_literal_skeleton_impact_caster (value : literal_skeleton_impact_caster) : Js.Json.t =
  Js.Json.parseExn "\"skeleton-impact-caster\""

and encode_literal_skeleton (value : literal_skeleton) : Js.Json.t = Js.Json.parseExn "\"skeleton\""
and encode_literal_spearman (value : literal_spearman) : Js.Json.t = Js.Json.parseExn "\"spearman\""

and encode_literal_swordsman (value : literal_swordsman) : Js.Json.t =
  Js.Json.parseExn "\"swordsman\""

and encode_literal_thorncaller (value : literal_thorncaller) : Js.Json.t =
  Js.Json.parseExn "\"thorncaller\""

and encode_literal_wavemender (value : literal_wavemender) : Js.Json.t =
  Js.Json.parseExn "\"wavemender\""

and encode_literal_wolf (value : literal_wolf) : Js.Json.t = Js.Json.parseExn "\"wolf\""

and encode_literal_horse_archer (value : literal_horse_archer) : Js.Json.t =
  Js.Json.parseExn "\"horse-archer\""

and encode_literal_skeleton_rider (value : literal_skeleton_rider) : Js.Json.t =
  Js.Json.parseExn "\"skeleton-rider\""

and encode_literal_skeleton_axeman (value : literal_skeleton_axeman) : Js.Json.t =
  Js.Json.parseExn "\"skeleton-axeman\""

and encode_literal_fox_spearguard (value : literal_fox_spearguard) : Js.Json.t =
  Js.Json.parseExn "\"fox-spearguard\""

and encode_literal_treant (value : literal_treant) : Js.Json.t = Js.Json.parseExn "\"treant\""

and encode_javascript1_state_villages (value : javascript1_state_villages) : Js.Json.t =
  Js.Json.array (Array.map encode_javascript1_state_villages_item value)

and encode_javascript1_state_villages_item (value : javascript1_state_villages_item) : Js.Json.t =
  wire_to_json ~known:[ "id"; "owner" ] value.wire
    [
      ("id", encode_json_string value.id); ("owner", encode_fragments_tiles11_item_owner value.owner);
    ]

and encode_javascript1_state_sites (value : javascript1_state_sites) : Js.Json.t =
  match value with `Undefined -> undefined | `Array x -> encode_javascript1_state_villages x

and encode_javascript1_state_campaign (value : javascript1_state_campaign) : Js.Json.t =
  match value with
  | `Undefined -> undefined
  | `Object2 x -> encode_javascript1_state_campaign_member1 x

and encode_javascript1_state_campaign_member1 (value : javascript1_state_campaign_member1) :
    Js.Json.t =
  wire_to_json
    ~known:
      [
        "id";
        "mission";
        "recall";
        "veterans";
        "storyVeterans";
        "fallenCompanions";
        "carryGold";
        "marks";
        "dialogue";
        "dialogueIndex";
        "outcome";
        "completed";
        "refuge";
        "vigil";
        "holds";
        "choices";
      ]
    value.wire
    ([
       ("id", encode_javascript1_state_campaign_member1_id value.id);
       ("mission", encode_json_number value.mission);
       ("recall", encode_javascript1_state_campaign_member1_recall value.recall);
       ("carryGold", encode_json_number value.carry_gold);
       ("marks", encode_fragments_native_state38_log value.marks);
       ("dialogue", encode_javascript1_state_campaign_member1_dialogue value.dialogue);
       ("dialogueIndex", encode_json_number value.dialogue_index);
       ("outcome", encode_fragments_tiles11_item_owner value.outcome);
       ("completed", encode_json_boolean value.completed);
     ]
    @ (match value.veterans with
      | None ->
          if List.mem "veterans" value.wire.undefined_fields then [ ("veterans", undefined) ]
          else []
      | Some x -> [ ("veterans", encode_javascript1_state_campaign_member1_veterans_member1 x) ])
    @ (match value.story_veterans with
      | None ->
          if List.mem "storyVeterans" value.wire.undefined_fields then
            [ ("storyVeterans", undefined) ]
          else []
      | Some x ->
          [ ("storyVeterans", encode_javascript1_state_campaign_member1_veterans_member1 x) ])
    @ (match value.fallen_companions with
      | None ->
          if List.mem "fallenCompanions" value.wire.undefined_fields then
            [ ("fallenCompanions", undefined) ]
          else []
      | Some x -> [ ("fallenCompanions", encode_fragments_native_state38_log x) ])
    @ (match value.refuge with
      | None ->
          if List.mem "refuge" value.wire.undefined_fields then [ ("refuge", undefined) ] else []
      | Some x -> [ ("refuge", encode_fragments_native_state38_campaign_member1_refuge_member1 x) ])
    @ (match value.vigil with
      | None ->
          if List.mem "vigil" value.wire.undefined_fields then [ ("vigil", undefined) ] else []
      | Some x -> [ ("vigil", encode_fragments_native_state38_campaign_member1_vigil_member1 x) ])
    @ (match value.holds with
      | None ->
          if List.mem "holds" value.wire.undefined_fields then [ ("holds", undefined) ] else []
      | Some x -> [ ("holds", encode_fragments_native_state38_campaign_member1_holds_member1 x) ])
    @
    match value.choices with
    | None ->
        if List.mem "choices" value.wire.undefined_fields then [ ("choices", undefined) ] else []
    | Some x -> [ ("choices", encode_fragments_native_state38_log x) ])

and encode_javascript1_state_campaign_member1_id (value : javascript1_state_campaign_member1_id) :
    Js.Json.t =
  match value with
  | `Lantern_road -> Js.Json.parseExn "\"lantern-road\""
  | `Green_accord -> Js.Json.parseExn "\"green-accord\""
  | `Ashen_oath -> Js.Json.parseExn "\"ashen-oath\""
  | `Borrowed_tide -> Js.Json.parseExn "\"borrowed-tide\""
  | `Mirror_night -> Js.Json.parseExn "\"mirror-night\""
  | `Bell_night -> Js.Json.parseExn "\"bell-night\""

and encode_literal_lantern_road (value : literal_lantern_road) : Js.Json.t =
  Js.Json.parseExn "\"lantern-road\""

and encode_literal_green_accord (value : literal_green_accord) : Js.Json.t =
  Js.Json.parseExn "\"green-accord\""

and encode_literal_ashen_oath (value : literal_ashen_oath) : Js.Json.t =
  Js.Json.parseExn "\"ashen-oath\""

and encode_literal_borrowed_tide (value : literal_borrowed_tide) : Js.Json.t =
  Js.Json.parseExn "\"borrowed-tide\""

and encode_literal_mirror_night (value : literal_mirror_night) : Js.Json.t =
  Js.Json.parseExn "\"mirror-night\""

and encode_literal_bell_night (value : literal_bell_night) : Js.Json.t =
  Js.Json.parseExn "\"bell-night\""

and encode_javascript1_state_campaign_member1_recall
    (value : javascript1_state_campaign_member1_recall) : Js.Json.t =
  Js.Json.array (Array.map encode_javascript1_state_units_item_type_ value)

and encode_javascript1_state_campaign_member1_veterans
    (value : javascript1_state_campaign_member1_veterans) : Js.Json.t =
  match value with
  | `Undefined -> undefined
  | `Array x -> encode_javascript1_state_campaign_member1_veterans_member1 x

and encode_javascript1_state_campaign_member1_veterans_member1
    (value : javascript1_state_campaign_member1_veterans_member1) : Js.Json.t =
  Js.Json.array (Array.map encode_javascript1_state_campaign_member1_veterans_member1_item value)

and encode_javascript1_state_campaign_member1_veterans_member1_item
    (value : javascript1_state_campaign_member1_veterans_member1_item) : Js.Json.t =
  wire_to_json
    ~known:[ "type"; "experience"; "name"; "role" ]
    value.wire
    ([
       ("type", encode_javascript1_state_units_item_type_ value.type_);
       ("experience", encode_json_number value.experience);
     ]
    @ (match value.name with
      | None -> if List.mem "name" value.wire.undefined_fields then [ ("name", undefined) ] else []
      | Some x -> [ ("name", encode_json_string x) ])
    @
    match value.role with
    | None -> if List.mem "role" value.wire.undefined_fields then [ ("role", undefined) ] else []
    | Some x -> [ ("role", encode_fragments_native_state38_units_item_role x) ])

and encode_javascript1_state_campaign_member1_dialogue
    (value : javascript1_state_campaign_member1_dialogue) : Js.Json.t =
  Js.Json.array (Array.map encode_javascript1_state_campaign_member1_dialogue_item value)

and encode_javascript1_state_campaign_member1_dialogue_item
    (value : javascript1_state_campaign_member1_dialogue_item) : Js.Json.t =
  wire_to_json
    ~known:[ "sound"; "speaker"; "character"; "portrait"; "text" ]
    value.wire
    ([
       ("speaker", encode_json_string value.speaker);
       ("portrait", encode_javascript1_state_campaign_member1_dialogue_item_portrait value.portrait);
       ("text", encode_json_string value.text);
     ]
    @ (match value.sound with
      | None ->
          if List.mem "sound" value.wire.undefined_fields then [ ("sound", undefined) ] else []
      | Some x -> [ ("sound", encode_json_string x) ])
    @
    match value.character with
    | None ->
        if List.mem "character" value.wire.undefined_fields then [ ("character", undefined) ]
        else []
    | Some x -> [ ("character", encode_json_string x) ])

and encode_javascript1_state_campaign_member1_dialogue_item_portrait
    (value : javascript1_state_campaign_member1_dialogue_item_portrait) : Js.Json.t =
  match value with
  | `Ranger -> Js.Json.parseExn "\"ranger\""
  | `Healer -> Js.Json.parseExn "\"healer\""
  | `Briarweaver -> Js.Json.parseExn "\"briarweaver\""
  | `Deatharcher -> Js.Json.parseExn "\"deatharcher\""
  | `Direwolf -> Js.Json.parseExn "\"direwolf\""
  | `Greatboar -> Js.Json.parseExn "\"greatboar\""
  | `Wraith -> Js.Json.parseExn "\"wraith\""
  | `Spectre -> Js.Json.parseExn "\"spectre\""
  | `Lancer -> Js.Json.parseExn "\"lancer\""
  | `Mage -> Js.Json.parseExn "\"mage\""
  | `Marksman -> Js.Json.parseExn "\"marksman\""
  | `Merfolk_maceguard_l2 -> Js.Json.parseExn "\"merfolk-maceguard-l2\""
  | `Revenant -> Js.Json.parseExn "\"revenant\""
  | `Skeleton_impact_caster_l2 -> Js.Json.parseExn "\"skeleton-impact-caster-l2\""
  | `Soldier -> Js.Json.parseExn "\"soldier\""
  | `Tidehunter -> Js.Json.parseExn "\"tidehunter\""
  | `Tidesage -> Js.Json.parseExn "\"tidesage\""
  | `Tridentguard -> Js.Json.parseExn "\"tridentguard\""
  | `Warden -> Js.Json.parseExn "\"warden\""
  | `Champion -> Js.Json.parseExn "\"champion\""
  | `Hammerman -> Js.Json.parseExn "\"hammerman\""
  | `Warhammer -> Js.Json.parseExn "\"warhammer\""
  | `Sentinel -> Js.Json.parseExn "\"sentinel\""
  | `Halberdier -> Js.Json.parseExn "\"halberdier\""
  | `Halberd_master -> Js.Json.parseExn "\"halberd-master\""
  | `Shadowblade -> Js.Json.parseExn "\"shadowblade\""
  | `Slinger -> Js.Json.parseExn "\"slinger\""
  | `Fireslinger -> Js.Json.parseExn "\"fireslinger\""
  | `Sharpshooter -> Js.Json.parseExn "\"sharpshooter\""
  | `Crossbowman -> Js.Json.parseExn "\"crossbowman\""
  | `Arbalester -> Js.Json.parseExn "\"arbalester\""
  | `Archmage -> Js.Json.parseExn "\"archmage\""
  | `High_healer -> Js.Json.parseExn "\"high-healer\""
  | `Knight -> Js.Json.parseExn "\"knight\""
  | `Horse_bowman -> Js.Json.parseExn "\"horse-bowman\""
  | `Horse_marksman -> Js.Json.parseExn "\"horse-marksman\""
  | `Skeleton_dreadguard -> Js.Json.parseExn "\"skeleton-dreadguard\""
  | `Skeleton_flanker_l2 -> Js.Json.parseExn "\"skeleton-flanker-l2\""
  | `Skeleton_flanker_l3 -> Js.Json.parseExn "\"skeleton-flanker-l3\""
  | `Skeleton_impact_caster_l3 -> Js.Json.parseExn "\"skeleton-impact-caster-l3\""
  | `Skeleton_scythe_caster_l2 -> Js.Json.parseExn "\"skeleton-scythe-caster-l2\""
  | `Skeleton_scythe_caster_l3 -> Js.Json.parseExn "\"skeleton-scythe-caster-l3\""
  | `Skeleton_rider_l2 -> Js.Json.parseExn "\"skeleton-rider-l2\""
  | `Skeleton_rider_l3 -> Js.Json.parseExn "\"skeleton-rider-l3\""
  | `Skeleton_bow_master -> Js.Json.parseExn "\"skeleton-bow-master\""
  | `Skeleton_axeman_l2 -> Js.Json.parseExn "\"skeleton-axeman-l2\""
  | `Skeleton_axeman_l3 -> Js.Json.parseExn "\"skeleton-axeman-l3\""
  | `Woodland_alpha_wolf -> Js.Json.parseExn "\"woodland-alpha-wolf\""
  | `Woodland_ancient_boar -> Js.Json.parseExn "\"woodland-ancient-boar\""
  | `Woodland_thorn_oracle -> Js.Json.parseExn "\"woodland-thorn-oracle\""
  | `Woodland_bramble_warden -> Js.Json.parseExn "\"woodland-bramble-warden\""
  | `Woodland_thornbinder -> Js.Json.parseExn "\"woodland-thornbinder\""
  | `Fox_spearguard_l2 -> Js.Json.parseExn "\"fox-spearguard-l2\""
  | `Fox_spearguard_l3 -> Js.Json.parseExn "\"fox-spearguard-l3\""
  | `Treant_l2 -> Js.Json.parseExn "\"treant-l2\""
  | `Treant_l3 -> Js.Json.parseExn "\"treant-l3\""
  | `Merfolk_harpoon_master -> Js.Json.parseExn "\"merfolk-harpoon-master\""
  | `Merfolk_netter_l2 -> Js.Json.parseExn "\"merfolk-netter-l2\""
  | `Merfolk_netter_l3 -> Js.Json.parseExn "\"merfolk-netter-l3\""
  | `Merfolk_reef_champion -> Js.Json.parseExn "\"merfolk-reef-champion\""
  | `Merfolk_trident_l2 -> Js.Json.parseExn "\"merfolk-trident-l2\""
  | `Merfolk_trident_l3 -> Js.Json.parseExn "\"merfolk-trident-l3\""
  | `Merfolk_tide_oracle -> Js.Json.parseExn "\"merfolk-tide-oracle\""
  | `Merfolk_tidecrusher_l2 -> Js.Json.parseExn "\"merfolk-tidecrusher-l2\""
  | `Merfolk_tidecrusher_l3 -> Js.Json.parseExn "\"merfolk-tidecrusher-l3\""
  | `Merfolk_maceguard_l3 -> Js.Json.parseExn "\"merfolk-maceguard-l3\""
  | `Dragon -> Js.Json.parseExn "\"dragon\""
  | `Skeletal_hound -> Js.Json.parseExn "\"skeletal-hound\""
  | `Captives -> Js.Json.parseExn "\"captives\""
  | `Zombie_walker -> Js.Json.parseExn "\"zombie-walker\""
  | `Zombie_decay_caster -> Js.Json.parseExn "\"zombie-decay-caster\""
  | `Bear -> Js.Json.parseExn "\"bear\""
  | `Giant_bat -> Js.Json.parseExn "\"giant-bat\""
  | `Giant_spider -> Js.Json.parseExn "\"giant-spider\""
  | `Commander -> Js.Json.parseExn "\"commander\""
  | `Grovewarden -> Js.Json.parseExn "\"grovewarden\""
  | `Lich -> Js.Json.parseExn "\"lich\""
  | `Marshal -> Js.Json.parseExn "\"marshal\""
  | `Necromancer -> Js.Json.parseExn "\"necromancer\""
  | `Tidekeeper -> Js.Json.parseExn "\"tidekeeper\""
  | `Tideseer -> Js.Json.parseExn "\"tideseer\""
  | `Wildkeeper -> Js.Json.parseExn "\"wildkeeper\""
  | `Grand_marshal -> Js.Json.parseExn "\"grand-marshal\""
  | `Lich_commander_l3 -> Js.Json.parseExn "\"lich-commander-l3\""
  | `Woodland_grove_oracle -> Js.Json.parseExn "\"woodland-grove-oracle\""
  | `Merfolk_tide_sovereign -> Js.Json.parseExn "\"merfolk-tide-sovereign\""
  | `Adept -> Js.Json.parseExn "\"adept\""
  | `Archer -> Js.Json.parseExn "\"archer\""
  | `Boar -> Js.Json.parseExn "\"boar\""
  | `Bonearcher -> Js.Json.parseExn "\"bonearcher\""
  | `Ghost -> Js.Json.parseExn "\"ghost\""
  | `Harpooner -> Js.Json.parseExn "\"harpooner\""
  | `Merfolk_maceguard -> Js.Json.parseExn "\"merfolk-maceguard\""
  | `Reefguard -> Js.Json.parseExn "\"reefguard\""
  | `Rider -> Js.Json.parseExn "\"rider\""
  | `Scout -> Js.Json.parseExn "\"scout\""
  | `Skeleton_impact_caster -> Js.Json.parseExn "\"skeleton-impact-caster\""
  | `Skeleton -> Js.Json.parseExn "\"skeleton\""
  | `Spearman -> Js.Json.parseExn "\"spearman\""
  | `Swordsman -> Js.Json.parseExn "\"swordsman\""
  | `Thorncaller -> Js.Json.parseExn "\"thorncaller\""
  | `Wavemender -> Js.Json.parseExn "\"wavemender\""
  | `Wolf -> Js.Json.parseExn "\"wolf\""
  | `Horse_archer -> Js.Json.parseExn "\"horse-archer\""
  | `Skeleton_rider -> Js.Json.parseExn "\"skeleton-rider\""
  | `Skeleton_axeman -> Js.Json.parseExn "\"skeleton-axeman\""
  | `Fox_spearguard -> Js.Json.parseExn "\"fox-spearguard\""
  | `Treant -> Js.Json.parseExn "\"treant\""
  | `Worker -> Js.Json.parseExn "\"worker\""

and encode_literal_worker (value : literal_worker) : Js.Json.t = Js.Json.parseExn "\"worker\""

let read_save2 value : save2 =
  require (Migration_reader.matches value 0) "Invalid frozen save2";
  decode_save2 value

let read_literal_2 value : literal_2 =
  require (Migration_reader.matches value 1) "Invalid frozen literal_2";
  decode_literal_2 value

let read_save2_checkpoint value : save2_checkpoint =
  require (Migration_reader.matches value 2) "Invalid frozen save2_checkpoint";
  decode_save2_checkpoint value

let read_json_number value : json_number =
  require (Migration_reader.matches value 3) "Invalid frozen json_number";
  decode_json_number value

let read_save2_checkpoint_state value : save2_checkpoint_state =
  require (Migration_reader.matches value 4) "Invalid frozen save2_checkpoint_state";
  decode_save2_checkpoint_state value

let read_save35_setup_first value : save35_setup_first =
  require (Migration_reader.matches value 5) "Invalid frozen save35_setup_first";
  decode_save35_setup_first value

let read_literal_player value : literal_player =
  require (Migration_reader.matches value 6) "Invalid frozen literal_player";
  decode_literal_player value

let read_literal_enemy value : literal_enemy =
  require (Migration_reader.matches value 7) "Invalid frozen literal_enemy";
  decode_literal_enemy value

let read_fragments_native_state38_winner value : fragments_native_state38_winner =
  require (Migration_reader.matches value 8) "Invalid frozen fragments_native_state38_winner";
  decode_fragments_native_state38_winner value

let read_json_null value : json_null =
  require (Migration_reader.matches value 9) "Invalid frozen json_null";
  decode_json_null value

let read_fragments_native_state38_armies value : fragments_native_state38_armies =
  require (Migration_reader.matches value 10) "Invalid frozen fragments_native_state38_armies";
  decode_fragments_native_state38_armies value

let read_json_undefined value : json_undefined =
  require (Migration_reader.matches value 11) "Invalid frozen json_undefined";
  decode_json_undefined value

let read_fragments_native_state38_armies_member1 value : fragments_native_state38_armies_member1 =
  require
    (Migration_reader.matches value 12)
    "Invalid frozen fragments_native_state38_armies_member1";
  decode_fragments_native_state38_armies_member1 value

let read_json_string value : json_string =
  require (Migration_reader.matches value 13) "Invalid frozen json_string";
  decode_json_string value

let read_fragments_native_state38_condition_memory value : fragments_native_state38_condition_memory
    =
  require
    (Migration_reader.matches value 14)
    "Invalid frozen fragments_native_state38_condition_memory";
  decode_fragments_native_state38_condition_memory value

let read_fragments_native_state38_log value : fragments_native_state38_log =
  require (Migration_reader.matches value 15) "Invalid frozen fragments_native_state38_log";
  decode_fragments_native_state38_log value

let read_fragments_native_state38_last_combat value : fragments_native_state38_last_combat =
  require (Migration_reader.matches value 16) "Invalid frozen fragments_native_state38_last_combat";
  decode_fragments_native_state38_last_combat value

let read_fragments_native_state38_last_combat_member2 value :
    fragments_native_state38_last_combat_member2 =
  require
    (Migration_reader.matches value 17)
    "Invalid frozen fragments_native_state38_last_combat_member2";
  decode_fragments_native_state38_last_combat_member2 value

let read_save2_checkpoint_state_units value : save2_checkpoint_state_units =
  require (Migration_reader.matches value 18) "Invalid frozen save2_checkpoint_state_units";
  decode_save2_checkpoint_state_units value

let read_save2_checkpoint_state_units_item value : save2_checkpoint_state_units_item =
  require (Migration_reader.matches value 19) "Invalid frozen save2_checkpoint_state_units_item";
  decode_save2_checkpoint_state_units_item value

let read_fragments_native_state38_units_item_role value : fragments_native_state38_units_item_role =
  require
    (Migration_reader.matches value 20)
    "Invalid frozen fragments_native_state38_units_item_role";
  decode_fragments_native_state38_units_item_role value

let read_literal_leader value : literal_leader =
  require (Migration_reader.matches value 21) "Invalid frozen literal_leader";
  decode_literal_leader value

let read_literal_courier value : literal_courier =
  require (Migration_reader.matches value 22) "Invalid frozen literal_courier";
  decode_literal_courier value

let read_literal_captive value : literal_captive =
  require (Migration_reader.matches value 23) "Invalid frozen literal_captive";
  decode_literal_captive value

let read_javascript1_state_map_id value : javascript1_state_map_id =
  require (Migration_reader.matches value 24) "Invalid frozen javascript1_state_map_id";
  decode_javascript1_state_map_id value

let read_fragments_native_state38_units_item_items value : fragments_native_state38_units_item_items
    =
  require
    (Migration_reader.matches value 25)
    "Invalid frozen fragments_native_state38_units_item_items";
  decode_fragments_native_state38_units_item_items value

let read_fragments_native_state38_units_item_items_member1 value :
    fragments_native_state38_units_item_items_member1 =
  require
    (Migration_reader.matches value 26)
    "Invalid frozen fragments_native_state38_units_item_items_member1";
  decode_fragments_native_state38_units_item_items_member1 value

let read_fragments_native_state38_ground_items_item_item value :
    fragments_native_state38_ground_items_item_item =
  require
    (Migration_reader.matches value 27)
    "Invalid frozen fragments_native_state38_ground_items_item_item";
  decode_fragments_native_state38_ground_items_item_item value

let read_fragments_native_state38_ground_items_item_item_armor value :
    fragments_native_state38_ground_items_item_item_armor =
  require
    (Migration_reader.matches value 28)
    "Invalid frozen fragments_native_state38_ground_items_item_item_armor";
  decode_fragments_native_state38_ground_items_item_item_armor value

let read_fragments_native_state38_ground_items_item_item_armor_member1 value :
    fragments_native_state38_ground_items_item_item_armor_member1 =
  require
    (Migration_reader.matches value 29)
    "Invalid frozen fragments_native_state38_ground_items_item_item_armor_member1";
  decode_fragments_native_state38_ground_items_item_item_armor_member1 value

let read_fragments_native_state38_ground_items_item_item_attacks value :
    fragments_native_state38_ground_items_item_item_attacks =
  require
    (Migration_reader.matches value 30)
    "Invalid frozen fragments_native_state38_ground_items_item_item_attacks";
  decode_fragments_native_state38_ground_items_item_item_attacks value

let read_fragments_native_state38_ground_items_item_item_attacks_item value :
    fragments_native_state38_ground_items_item_item_attacks_item =
  require
    (Migration_reader.matches value 31)
    "Invalid frozen fragments_native_state38_ground_items_item_item_attacks_item";
  decode_fragments_native_state38_ground_items_item_item_attacks_item value

let read_fragments_light23_dx value : fragments_light23_dx =
  require (Migration_reader.matches value 32) "Invalid frozen fragments_light23_dx";
  decode_fragments_light23_dx value

let read_fragments_native_state38_ground_items_item_item_attacks_item_damage_type value :
    fragments_native_state38_ground_items_item_item_attacks_item_damage_type =
  require
    (Migration_reader.matches value 33)
    "Invalid frozen fragments_native_state38_ground_items_item_item_attacks_item_damage_type";
  decode_fragments_native_state38_ground_items_item_item_attacks_item_damage_type value

let read_literal_blade value : literal_blade =
  require (Migration_reader.matches value 34) "Invalid frozen literal_blade";
  decode_literal_blade value

let read_literal_pierce value : literal_pierce =
  require (Migration_reader.matches value 35) "Invalid frozen literal_pierce";
  decode_literal_pierce value

let read_literal_impact value : literal_impact =
  require (Migration_reader.matches value 36) "Invalid frozen literal_impact";
  decode_literal_impact value

let read_fragments_native_state38_ground_items_item_item_attacks_item_category value :
    fragments_native_state38_ground_items_item_item_attacks_item_category =
  require
    (Migration_reader.matches value 37)
    "Invalid frozen fragments_native_state38_ground_items_item_item_attacks_item_category";
  decode_fragments_native_state38_ground_items_item_item_attacks_item_category value

let read_literal_melee value : literal_melee =
  require (Migration_reader.matches value 38) "Invalid frozen literal_melee";
  decode_literal_melee value

let read_literal_ranged value : literal_ranged =
  require (Migration_reader.matches value 39) "Invalid frozen literal_ranged";
  decode_literal_ranged value

let read_fragments_native_state38_ground_items_item_item_healing value :
    fragments_native_state38_ground_items_item_item_healing =
  require
    (Migration_reader.matches value 40)
    "Invalid frozen fragments_native_state38_ground_items_item_item_healing";
  decode_fragments_native_state38_ground_items_item_item_healing value

let read_fragments_native_state38_ground_items_item_item_healing_member1 value :
    fragments_native_state38_ground_items_item_item_healing_member1 =
  require
    (Migration_reader.matches value 41)
    "Invalid frozen fragments_native_state38_ground_items_item_item_healing_member1";
  decode_fragments_native_state38_ground_items_item_item_healing_member1 value

let read_json_boolean value : json_boolean =
  require (Migration_reader.matches value 42) "Invalid frozen json_boolean";
  decode_json_boolean value

let read_fragments_rule11_remember value : fragments_rule11_remember =
  require (Migration_reader.matches value 43) "Invalid frozen fragments_rule11_remember";
  decode_fragments_rule11_remember value

let read_literal_false value : literal_false =
  require (Migration_reader.matches value 44) "Invalid frozen literal_false";
  decode_literal_false value

let read_literal_true value : literal_true =
  require (Migration_reader.matches value 45) "Invalid frozen literal_true";
  decode_literal_true value

let read_save2_checkpoint_state_tiles value : save2_checkpoint_state_tiles =
  require (Migration_reader.matches value 46) "Invalid frozen save2_checkpoint_state_tiles";
  decode_save2_checkpoint_state_tiles value

let read_save2_checkpoint_state_tiles_item value : save2_checkpoint_state_tiles_item =
  require (Migration_reader.matches value 47) "Invalid frozen save2_checkpoint_state_tiles_item";
  decode_save2_checkpoint_state_tiles_item value

let read_fragments_tiles11_item_owner value : fragments_tiles11_item_owner =
  require (Migration_reader.matches value 48) "Invalid frozen fragments_tiles11_item_owner";
  decode_fragments_tiles11_item_owner value

let read_fragments_tiles11_item_connections value : fragments_tiles11_item_connections =
  require (Migration_reader.matches value 49) "Invalid frozen fragments_tiles11_item_connections";
  decode_fragments_tiles11_item_connections value

let read_fragments_tiles11_item_connections_member1 value :
    fragments_tiles11_item_connections_member1 =
  require
    (Migration_reader.matches value 50)
    "Invalid frozen fragments_tiles11_item_connections_member1";
  decode_fragments_tiles11_item_connections_member1 value

let read_fragments_tiles11_item_connections_member1_item value :
    fragments_tiles11_item_connections_member1_item =
  require
    (Migration_reader.matches value 51)
    "Invalid frozen fragments_tiles11_item_connections_member1_item";
  decode_fragments_tiles11_item_connections_member1_item value

let read_literal_e value : literal_e =
  require (Migration_reader.matches value 52) "Invalid frozen literal_e";
  decode_literal_e value

let read_literal_se value : literal_se =
  require (Migration_reader.matches value 53) "Invalid frozen literal_se";
  decode_literal_se value

let read_literal_sw value : literal_sw =
  require (Migration_reader.matches value 54) "Invalid frozen literal_sw";
  decode_literal_sw value

let read_literal_w value : literal_w =
  require (Migration_reader.matches value 55) "Invalid frozen literal_w";
  decode_literal_w value

let read_literal_nw value : literal_nw =
  require (Migration_reader.matches value 56) "Invalid frozen literal_nw";
  decode_literal_nw value

let read_literal_ne value : literal_ne =
  require (Migration_reader.matches value 57) "Invalid frozen literal_ne";
  decode_literal_ne value

let read_save2_checkpoint_state_tiles_item_fog value : save2_checkpoint_state_tiles_item_fog =
  require (Migration_reader.matches value 58) "Invalid frozen save2_checkpoint_state_tiles_item_fog";
  decode_save2_checkpoint_state_tiles_item_fog value

let read_save2_checkpoint_state_tiles_item_fog_member1 value :
    save2_checkpoint_state_tiles_item_fog_member1 =
  require
    (Migration_reader.matches value 59)
    "Invalid frozen save2_checkpoint_state_tiles_item_fog_member1";
  decode_save2_checkpoint_state_tiles_item_fog_member1 value

let read_save2_checkpoint_state_tiles_item_fog_member1_condition value :
    save2_checkpoint_state_tiles_item_fog_member1_condition =
  require
    (Migration_reader.matches value 60)
    "Invalid frozen save2_checkpoint_state_tiles_item_fog_member1_condition";
  decode_save2_checkpoint_state_tiles_item_fog_member1_condition value

let read_fragments_rule11_condition_member0 value : fragments_rule11_condition_member0 =
  require (Migration_reader.matches value 61) "Invalid frozen fragments_rule11_condition_member0";
  decode_fragments_rule11_condition_member0 value

let read_literal_owner value : literal_owner =
  require (Migration_reader.matches value 62) "Invalid frozen literal_owner";
  decode_literal_owner value

let read_fragments_rule11_condition_member1 value : fragments_rule11_condition_member1 =
  require (Migration_reader.matches value 63) "Invalid frozen fragments_rule11_condition_member1";
  decode_fragments_rule11_condition_member1 value

let read_literal_visible_units value : literal_visible_units =
  require (Migration_reader.matches value 64) "Invalid frozen literal_visible_units";
  decode_literal_visible_units value

let read_fragments_rule11_condition_member1_side value : fragments_rule11_condition_member1_side =
  require
    (Migration_reader.matches value 65)
    "Invalid frozen fragments_rule11_condition_member1_side";
  decode_fragments_rule11_condition_member1_side value

let read_literal_neutral value : literal_neutral =
  require (Migration_reader.matches value 66) "Invalid frozen literal_neutral";
  decode_literal_neutral value

let read_literal_any value : literal_any =
  require (Migration_reader.matches value 67) "Invalid frozen literal_any";
  decode_literal_any value

let read_fragments_rule11_condition_member2 value : fragments_rule11_condition_member2 =
  require (Migration_reader.matches value 68) "Invalid frozen fragments_rule11_condition_member2";
  decode_fragments_rule11_condition_member2 value

let read_literal_choice value : literal_choice =
  require (Migration_reader.matches value 69) "Invalid frozen literal_choice";
  decode_literal_choice value

let read_fragments_rule11_condition_member3 value : fragments_rule11_condition_member3 =
  require (Migration_reader.matches value 70) "Invalid frozen fragments_rule11_condition_member3";
  decode_fragments_rule11_condition_member3 value

let read_literal_flag value : literal_flag =
  require (Migration_reader.matches value 71) "Invalid frozen literal_flag";
  decode_literal_flag value

let read_fragments_rule11_condition_member4 value : fragments_rule11_condition_member4 =
  require (Migration_reader.matches value 72) "Invalid frozen fragments_rule11_condition_member4";
  decode_fragments_rule11_condition_member4 value

let read_literal_event value : literal_event =
  require (Migration_reader.matches value 73) "Invalid frozen literal_event";
  decode_literal_event value

let read_fragments_rule11_condition_member5 value : fragments_rule11_condition_member5 =
  require (Migration_reader.matches value 74) "Invalid frozen fragments_rule11_condition_member5";
  decode_fragments_rule11_condition_member5 value

let read_literal_round value : literal_round =
  require (Migration_reader.matches value 75) "Invalid frozen literal_round";
  decode_literal_round value

let read_save2_checkpoint_state_tiles_item_fog_member1_condition_member6 value :
    save2_checkpoint_state_tiles_item_fog_member1_condition_member6 =
  require
    (Migration_reader.matches value 76)
    "Invalid frozen save2_checkpoint_state_tiles_item_fog_member1_condition_member6";
  decode_save2_checkpoint_state_tiles_item_fog_member1_condition_member6 value

let read_fragments_rule11_condition_member9_kind value : fragments_rule11_condition_member9_kind =
  require
    (Migration_reader.matches value 77)
    "Invalid frozen fragments_rule11_condition_member9_kind";
  decode_fragments_rule11_condition_member9_kind value

let read_literal_all value : literal_all =
  require (Migration_reader.matches value 78) "Invalid frozen literal_all";
  decode_literal_all value

let read_save2_checkpoint_state_tiles_item_fog_member1_condition_member6_conditions value :
    save2_checkpoint_state_tiles_item_fog_member1_condition_member6_conditions =
  require
    (Migration_reader.matches value 79)
    "Invalid frozen save2_checkpoint_state_tiles_item_fog_member1_condition_member6_conditions";
  decode_save2_checkpoint_state_tiles_item_fog_member1_condition_member6_conditions value

let read_fragments_rule11_condition_member6 value : fragments_rule11_condition_member6 =
  require (Migration_reader.matches value 80) "Invalid frozen fragments_rule11_condition_member6";
  decode_fragments_rule11_condition_member6 value

let read_literal_proximity value : literal_proximity =
  require (Migration_reader.matches value 81) "Invalid frozen literal_proximity";
  decode_literal_proximity value

let read_fragments_rule11_condition_member7 value : fragments_rule11_condition_member7 =
  require (Migration_reader.matches value 82) "Invalid frozen fragments_rule11_condition_member7";
  decode_fragments_rule11_condition_member7 value

let read_literal_has_units value : literal_has_units =
  require (Migration_reader.matches value 83) "Invalid frozen literal_has_units";
  decode_literal_has_units value

let read_fragments_rule11_condition_member8 value : fragments_rule11_condition_member8 =
  require (Migration_reader.matches value 84) "Invalid frozen fragments_rule11_condition_member8";
  decode_fragments_rule11_condition_member8 value

let read_literal_no_units value : literal_no_units =
  require (Migration_reader.matches value 85) "Invalid frozen literal_no_units";
  decode_literal_no_units value

let read_save2_checkpoint_state_tiles_item_light value : save2_checkpoint_state_tiles_item_light =
  require
    (Migration_reader.matches value 86)
    "Invalid frozen save2_checkpoint_state_tiles_item_light";
  decode_save2_checkpoint_state_tiles_item_light value

let read_save2_checkpoint_state_tiles_item_light_member1 value :
    save2_checkpoint_state_tiles_item_light_member1 =
  require
    (Migration_reader.matches value 87)
    "Invalid frozen save2_checkpoint_state_tiles_item_light_member1";
  decode_save2_checkpoint_state_tiles_item_light_member1 value

let read_fragments_light23_color value : fragments_light23_color =
  require (Migration_reader.matches value 88) "Invalid frozen fragments_light23_color";
  decode_fragments_light23_color value

let read_fragments_light23_color_member1 value : fragments_light23_color_member1 =
  require (Migration_reader.matches value 89) "Invalid frozen fragments_light23_color_member1";
  decode_fragments_light23_color_member1 value

let read_fragments_tiles11_item_wall_corners value : fragments_tiles11_item_wall_corners =
  require (Migration_reader.matches value 90) "Invalid frozen fragments_tiles11_item_wall_corners";
  decode_fragments_tiles11_item_wall_corners value

let read_fragments_tiles11_item_wall_corners_member1 value :
    fragments_tiles11_item_wall_corners_member1 =
  require
    (Migration_reader.matches value 91)
    "Invalid frozen fragments_tiles11_item_wall_corners_member1";
  decode_fragments_tiles11_item_wall_corners_member1 value

let read_save2_checkpoint_state_campaign value : save2_checkpoint_state_campaign =
  require (Migration_reader.matches value 92) "Invalid frozen save2_checkpoint_state_campaign";
  decode_save2_checkpoint_state_campaign value

let read_save2_checkpoint_state_campaign_member1 value : save2_checkpoint_state_campaign_member1 =
  require
    (Migration_reader.matches value 93)
    "Invalid frozen save2_checkpoint_state_campaign_member1";
  decode_save2_checkpoint_state_campaign_member1 value

let read_fragments_native_state38_campaign_member1_vigil value :
    fragments_native_state38_campaign_member1_vigil =
  require
    (Migration_reader.matches value 94)
    "Invalid frozen fragments_native_state38_campaign_member1_vigil";
  decode_fragments_native_state38_campaign_member1_vigil value

let read_fragments_native_state38_campaign_member1_vigil_member1 value :
    fragments_native_state38_campaign_member1_vigil_member1 =
  require
    (Migration_reader.matches value 95)
    "Invalid frozen fragments_native_state38_campaign_member1_vigil_member1";
  decode_fragments_native_state38_campaign_member1_vigil_member1 value

let read_fragments_native_state38_campaign_member1_refuge value :
    fragments_native_state38_campaign_member1_refuge =
  require
    (Migration_reader.matches value 96)
    "Invalid frozen fragments_native_state38_campaign_member1_refuge";
  decode_fragments_native_state38_campaign_member1_refuge value

let read_fragments_native_state38_campaign_member1_refuge_member1 value :
    fragments_native_state38_campaign_member1_refuge_member1 =
  require
    (Migration_reader.matches value 97)
    "Invalid frozen fragments_native_state38_campaign_member1_refuge_member1";
  decode_fragments_native_state38_campaign_member1_refuge_member1 value

let read_fragments_native_state38_campaign_member1_holds value :
    fragments_native_state38_campaign_member1_holds =
  require
    (Migration_reader.matches value 98)
    "Invalid frozen fragments_native_state38_campaign_member1_holds";
  decode_fragments_native_state38_campaign_member1_holds value

let read_fragments_native_state38_campaign_member1_holds_member1 value :
    fragments_native_state38_campaign_member1_holds_member1 =
  require
    (Migration_reader.matches value 99)
    "Invalid frozen fragments_native_state38_campaign_member1_holds_member1";
  decode_fragments_native_state38_campaign_member1_holds_member1 value

let read_fragments_native_state38_campaign_member1_dialogue value :
    fragments_native_state38_campaign_member1_dialogue =
  require
    (Migration_reader.matches value 100)
    "Invalid frozen fragments_native_state38_campaign_member1_dialogue";
  decode_fragments_native_state38_campaign_member1_dialogue value

let read_fragments_native_state38_campaign_member1_dialogue_item value :
    fragments_native_state38_campaign_member1_dialogue_item =
  require
    (Migration_reader.matches value 101)
    "Invalid frozen fragments_native_state38_campaign_member1_dialogue_item";
  decode_fragments_native_state38_campaign_member1_dialogue_item value

let read_save2_checkpoint_state_campaign_member1_veterans value :
    save2_checkpoint_state_campaign_member1_veterans =
  require
    (Migration_reader.matches value 102)
    "Invalid frozen save2_checkpoint_state_campaign_member1_veterans";
  decode_save2_checkpoint_state_campaign_member1_veterans value

let read_save2_checkpoint_state_campaign_member1_veterans_member1 value :
    save2_checkpoint_state_campaign_member1_veterans_member1 =
  require
    (Migration_reader.matches value 103)
    "Invalid frozen save2_checkpoint_state_campaign_member1_veterans_member1";
  decode_save2_checkpoint_state_campaign_member1_veterans_member1 value

let read_save2_checkpoint_state_campaign_member1_veterans_member1_item value :
    save2_checkpoint_state_campaign_member1_veterans_member1_item =
  require
    (Migration_reader.matches value 104)
    "Invalid frozen save2_checkpoint_state_campaign_member1_veterans_member1_item";
  decode_save2_checkpoint_state_campaign_member1_veterans_member1_item value

let read_save2_checkpoint_state_campaign_member1_evacuated value :
    save2_checkpoint_state_campaign_member1_evacuated =
  require
    (Migration_reader.matches value 105)
    "Invalid frozen save2_checkpoint_state_campaign_member1_evacuated";
  decode_save2_checkpoint_state_campaign_member1_evacuated value

let read_save2_checkpoint_state_campaign_member1_evacuated_member1 value :
    save2_checkpoint_state_campaign_member1_evacuated_member1 =
  require
    (Migration_reader.matches value 106)
    "Invalid frozen save2_checkpoint_state_campaign_member1_evacuated_member1";
  decode_save2_checkpoint_state_campaign_member1_evacuated_member1 value

let read_save2_checkpoint_state_campaign_member1_evacuated_member1_item value :
    save2_checkpoint_state_campaign_member1_evacuated_member1_item =
  require
    (Migration_reader.matches value 107)
    "Invalid frozen save2_checkpoint_state_campaign_member1_evacuated_member1_item";
  decode_save2_checkpoint_state_campaign_member1_evacuated_member1_item value

let read_javascript1_state_gold value : javascript1_state_gold =
  require (Migration_reader.matches value 108) "Invalid frozen javascript1_state_gold";
  decode_javascript1_state_gold value

let read_save2_checkpoint_state_factions value : save2_checkpoint_state_factions =
  require (Migration_reader.matches value 109) "Invalid frozen save2_checkpoint_state_factions";
  decode_save2_checkpoint_state_factions value

let read_save2_checkpoint_state_factions_member1 value : save2_checkpoint_state_factions_member1 =
  require
    (Migration_reader.matches value 110)
    "Invalid frozen save2_checkpoint_state_factions_member1";
  decode_save2_checkpoint_state_factions_member1 value

let read_save2_checkpoint_state_factions_member1_item value :
    save2_checkpoint_state_factions_member1_item =
  require
    (Migration_reader.matches value 111)
    "Invalid frozen save2_checkpoint_state_factions_member1_item";
  decode_save2_checkpoint_state_factions_member1_item value

let read_fragments_factions7_item_controller value : fragments_factions7_item_controller =
  require (Migration_reader.matches value 112) "Invalid frozen fragments_factions7_item_controller";
  decode_fragments_factions7_item_controller value

let read_fragments_factions7_item_controller_member0 value :
    fragments_factions7_item_controller_member0 =
  require
    (Migration_reader.matches value 113)
    "Invalid frozen fragments_factions7_item_controller_member0";
  decode_fragments_factions7_item_controller_member0 value

let read_literal_human value : literal_human =
  require (Migration_reader.matches value 114) "Invalid frozen literal_human";
  decode_literal_human value

let read_fragments_factions7_item_controller_member1 value :
    fragments_factions7_item_controller_member1 =
  require
    (Migration_reader.matches value 115)
    "Invalid frozen fragments_factions7_item_controller_member1";
  decode_fragments_factions7_item_controller_member1 value

let read_literal_passive value : literal_passive =
  require (Migration_reader.matches value 116) "Invalid frozen literal_passive";
  decode_literal_passive value

let read_fragments_factions7_item_controller_member2 value :
    fragments_factions7_item_controller_member2 =
  require
    (Migration_reader.matches value 117)
    "Invalid frozen fragments_factions7_item_controller_member2";
  decode_fragments_factions7_item_controller_member2 value

let read_literal_ai value : literal_ai =
  require (Migration_reader.matches value 118) "Invalid frozen literal_ai";
  decode_literal_ai value

let read_fragments_factions7_item_controller_member2_policy value :
    fragments_factions7_item_controller_member2_policy =
  require
    (Migration_reader.matches value 119)
    "Invalid frozen fragments_factions7_item_controller_member2_policy";
  decode_fragments_factions7_item_controller_member2_policy value

let read_literal_classic value : literal_classic =
  require (Migration_reader.matches value 120) "Invalid frozen literal_classic";
  decode_literal_classic value

let read_literal_rush value : literal_rush =
  require (Migration_reader.matches value 121) "Invalid frozen literal_rush";
  decode_literal_rush value

let read_literal_balanced value : literal_balanced =
  require (Migration_reader.matches value 122) "Invalid frozen literal_balanced";
  decode_literal_balanced value

let read_literal_rotated value : literal_rotated =
  require (Migration_reader.matches value 123) "Invalid frozen literal_rotated";
  decode_literal_rotated value

let read_literal_situational value : literal_situational =
  require (Migration_reader.matches value 124) "Invalid frozen literal_situational";
  decode_literal_situational value

let read_literal_predictor value : literal_predictor =
  require (Migration_reader.matches value 125) "Invalid frozen literal_predictor";
  decode_literal_predictor value

let read_fragments_factions7_item_recruits value : fragments_factions7_item_recruits =
  require (Migration_reader.matches value 126) "Invalid frozen fragments_factions7_item_recruits";
  decode_fragments_factions7_item_recruits value

let read_fragments_factions7_item_recruits_member1 value : fragments_factions7_item_recruits_member1
    =
  require
    (Migration_reader.matches value 127)
    "Invalid frozen fragments_factions7_item_recruits_member1";
  decode_fragments_factions7_item_recruits_member1 value

let read_fragments_factions7_item_recruits_member1_item value :
    fragments_factions7_item_recruits_member1_item =
  require
    (Migration_reader.matches value 128)
    "Invalid frozen fragments_factions7_item_recruits_member1_item";
  decode_fragments_factions7_item_recruits_member1_item value

let read_fragments_factions7_item_relations value : fragments_factions7_item_relations =
  require (Migration_reader.matches value 129) "Invalid frozen fragments_factions7_item_relations";
  decode_fragments_factions7_item_relations value

let read_fragments_factions7_item_relations_member1 value :
    fragments_factions7_item_relations_member1 =
  require
    (Migration_reader.matches value 130)
    "Invalid frozen fragments_factions7_item_relations_member1";
  decode_fragments_factions7_item_relations_member1 value

let read_fragments_factions7_item_relations_member1_value value :
    fragments_factions7_item_relations_member1_value =
  require
    (Migration_reader.matches value 131)
    "Invalid frozen fragments_factions7_item_relations_member1_value";
  decode_fragments_factions7_item_relations_member1_value value

let read_literal_allied value : literal_allied =
  require (Migration_reader.matches value 132) "Invalid frozen literal_allied";
  decode_literal_allied value

let read_literal_hostile value : literal_hostile =
  require (Migration_reader.matches value 133) "Invalid frozen literal_hostile";
  decode_literal_hostile value

let read_save2_checkpoint_state_factions_member1_item_transfer value :
    save2_checkpoint_state_factions_member1_item_transfer =
  require
    (Migration_reader.matches value 134)
    "Invalid frozen save2_checkpoint_state_factions_member1_item_transfer";
  decode_save2_checkpoint_state_factions_member1_item_transfer value

let read_save2_checkpoint_state_factions_member1_item_transfer_member1 value :
    save2_checkpoint_state_factions_member1_item_transfer_member1 =
  require
    (Migration_reader.matches value 135)
    "Invalid frozen save2_checkpoint_state_factions_member1_item_transfer_member1";
  decode_save2_checkpoint_state_factions_member1_item_transfer_member1 value

let read_save2_checkpoint_state_ground_items value : save2_checkpoint_state_ground_items =
  require (Migration_reader.matches value 136) "Invalid frozen save2_checkpoint_state_ground_items";
  decode_save2_checkpoint_state_ground_items value

let read_fragments_native_state38_ground_items value : fragments_native_state38_ground_items =
  require
    (Migration_reader.matches value 137)
    "Invalid frozen fragments_native_state38_ground_items";
  decode_fragments_native_state38_ground_items value

let read_fragments_native_state38_ground_items_item value :
    fragments_native_state38_ground_items_item =
  require
    (Migration_reader.matches value 138)
    "Invalid frozen fragments_native_state38_ground_items_item";
  decode_fragments_native_state38_ground_items_item value

let read_json_unknown value : json_unknown =
  require (Migration_reader.matches value 139) "Invalid frozen json_unknown";
  decode_json_unknown value

let read_save2_setup value : save2_setup =
  require (Migration_reader.matches value 140) "Invalid frozen save2_setup";
  decode_save2_setup value

let read_save2_setup_member1 value : save2_setup_member1 =
  require (Migration_reader.matches value 141) "Invalid frozen save2_setup_member1";
  decode_save2_setup_member1 value

let read_save2_setup_member1_enemy value : save2_setup_member1_enemy =
  require (Migration_reader.matches value 143) "Invalid frozen save2_setup_member1_enemy";
  decode_save2_setup_member1_enemy value

let read_save35_setup_enemy_policy value : save35_setup_enemy_policy =
  require (Migration_reader.matches value 144) "Invalid frozen save35_setup_enemy_policy";
  decode_save35_setup_enemy_policy value

let read_save35_setup_enemy_blocks value : save35_setup_enemy_blocks =
  require (Migration_reader.matches value 145) "Invalid frozen save35_setup_enemy_blocks";
  decode_save35_setup_enemy_blocks value

let read_save35_setup_enemy_blocks_member1 value : save35_setup_enemy_blocks_member1 =
  require (Migration_reader.matches value 146) "Invalid frozen save35_setup_enemy_blocks_member1";
  decode_save35_setup_enemy_blocks_member1 value

let read_save35_setup_enemy_blocks_member1_recruiter value :
    save35_setup_enemy_blocks_member1_recruiter =
  require
    (Migration_reader.matches value 147)
    "Invalid frozen save35_setup_enemy_blocks_member1_recruiter";
  decode_save35_setup_enemy_blocks_member1_recruiter value

let read_literal_rotation value : literal_rotation =
  require (Migration_reader.matches value 148) "Invalid frozen literal_rotation";
  decode_literal_rotation value

let read_literal_frontline value : literal_frontline =
  require (Migration_reader.matches value 149) "Invalid frozen literal_frontline";
  decode_literal_frontline value

let read_literal_aggressive_predictor value : literal_aggressive_predictor =
  require (Migration_reader.matches value 150) "Invalid frozen literal_aggressive_predictor";
  decode_literal_aggressive_predictor value

let read_save35_setup_enemy_blocks_member1_campaign value :
    save35_setup_enemy_blocks_member1_campaign =
  require
    (Migration_reader.matches value 151)
    "Invalid frozen save35_setup_enemy_blocks_member1_campaign";
  decode_save35_setup_enemy_blocks_member1_campaign value

let read_literal_arena value : literal_arena =
  require (Migration_reader.matches value 152) "Invalid frozen literal_arena";
  decode_literal_arena value

let read_literal_mission value : literal_mission =
  require (Migration_reader.matches value 153) "Invalid frozen literal_mission";
  decode_literal_mission value

let read_save35_setup_enemy_blocks_member1_planner value : save35_setup_enemy_blocks_member1_planner
    =
  require
    (Migration_reader.matches value 154)
    "Invalid frozen save35_setup_enemy_blocks_member1_planner";
  decode_save35_setup_enemy_blocks_member1_planner value

let read_literal_full value : literal_full =
  require (Migration_reader.matches value 155) "Invalid frozen literal_full";
  decode_literal_full value

let read_literal_queued value : literal_queued =
  require (Migration_reader.matches value 156) "Invalid frozen literal_queued";
  decode_literal_queued value

let read_save2_setup_member1_enemy_diplomacy value : save2_setup_member1_enemy_diplomacy =
  require (Migration_reader.matches value 157) "Invalid frozen save2_setup_member1_enemy_diplomacy";
  decode_save2_setup_member1_enemy_diplomacy value

let read_literal_diplomatic value : literal_diplomatic =
  require (Migration_reader.matches value 158) "Invalid frozen literal_diplomatic";
  decode_literal_diplomatic value

let read_literal_unrestricted value : literal_unrestricted =
  require (Migration_reader.matches value 159) "Invalid frozen literal_unrestricted";
  decode_literal_unrestricted value

let read_literal_pragmatic value : literal_pragmatic =
  require (Migration_reader.matches value 160) "Invalid frozen literal_pragmatic";
  decode_literal_pragmatic value

let read_literal_restrained value : literal_restrained =
  require (Migration_reader.matches value 161) "Invalid frozen literal_restrained";
  decode_literal_restrained value

let read_save2_progress value : save2_progress =
  require (Migration_reader.matches value 162) "Invalid frozen save2_progress";
  decode_save2_progress value

let read_save2_progress_member1 value : save2_progress_member1 =
  require (Migration_reader.matches value 163) "Invalid frozen save2_progress_member1";
  decode_save2_progress_member1 value

let read_history1_entries_item value : history1_entries_item =
  require (Migration_reader.matches value 164) "Invalid frozen history1_entries_item";
  decode_history1_entries_item value

let read_history1_entries_item_kind value : history1_entries_item_kind =
  require (Migration_reader.matches value 165) "Invalid frozen history1_entries_item_kind";
  decode_history1_entries_item_kind value

let read_literal_turn value : literal_turn =
  require (Migration_reader.matches value 166) "Invalid frozen literal_turn";
  decode_literal_turn value

let read_literal_recovery value : literal_recovery =
  require (Migration_reader.matches value 167) "Invalid frozen literal_recovery";
  decode_literal_recovery value

let read_literal_resume value : literal_resume =
  require (Migration_reader.matches value 168) "Invalid frozen literal_resume";
  decode_literal_resume value

let read_save3 value : save3 =
  require (Migration_reader.matches value 169) "Invalid frozen save3";
  decode_save3 value

let read_literal_3 value : literal_3 =
  require (Migration_reader.matches value 170) "Invalid frozen literal_3";
  decode_literal_3 value

let read_save3_checkpoint value : save3_checkpoint =
  require (Migration_reader.matches value 171) "Invalid frozen save3_checkpoint";
  decode_save3_checkpoint value

let read_save3_checkpoint_state value : save3_checkpoint_state =
  require (Migration_reader.matches value 172) "Invalid frozen save3_checkpoint_state";
  decode_save3_checkpoint_state value

let read_fragments_tiles11 value : fragments_tiles11 =
  require (Migration_reader.matches value 175) "Invalid frozen fragments_tiles11";
  decode_fragments_tiles11 value

let read_fragments_tiles11_item value : fragments_tiles11_item =
  require (Migration_reader.matches value 176) "Invalid frozen fragments_tiles11_item";
  decode_fragments_tiles11_item value

let read_fragments_light23_when_ value : fragments_light23_when_ =
  require (Migration_reader.matches value 177) "Invalid frozen fragments_light23_when_";
  decode_fragments_light23_when_ value

let read_fragments_rule11 value : fragments_rule11 =
  require (Migration_reader.matches value 178) "Invalid frozen fragments_rule11";
  decode_fragments_rule11 value

let read_fragments_rule11_condition value : fragments_rule11_condition =
  require (Migration_reader.matches value 179) "Invalid frozen fragments_rule11_condition";
  decode_fragments_rule11_condition value

let read_fragments_rule11_condition_member9 value : fragments_rule11_condition_member9 =
  require (Migration_reader.matches value 180) "Invalid frozen fragments_rule11_condition_member9";
  decode_fragments_rule11_condition_member9 value

let read_fragments_rule11_condition_member9_conditions value :
    fragments_rule11_condition_member9_conditions =
  require
    (Migration_reader.matches value 181)
    "Invalid frozen fragments_rule11_condition_member9_conditions";
  decode_fragments_rule11_condition_member9_conditions value

let read_fragments_tiles11_item_light value : fragments_tiles11_item_light =
  require (Migration_reader.matches value 182) "Invalid frozen fragments_tiles11_item_light";
  decode_fragments_tiles11_item_light value

let read_fragments_light23 value : fragments_light23 =
  require (Migration_reader.matches value 183) "Invalid frozen fragments_light23";
  decode_fragments_light23 value

let read_save3_checkpoint_state_factions value : save3_checkpoint_state_factions =
  require (Migration_reader.matches value 192) "Invalid frozen save3_checkpoint_state_factions";
  decode_save3_checkpoint_state_factions value

let read_fragments_factions7 value : fragments_factions7 =
  require (Migration_reader.matches value 193) "Invalid frozen fragments_factions7";
  decode_fragments_factions7 value

let read_fragments_factions7_item value : fragments_factions7_item =
  require (Migration_reader.matches value 194) "Invalid frozen fragments_factions7_item";
  decode_fragments_factions7_item value

let read_fragments_factions7_item_transfer value : fragments_factions7_item_transfer =
  require (Migration_reader.matches value 195) "Invalid frozen fragments_factions7_item_transfer";
  decode_fragments_factions7_item_transfer value

let read_fragments_factions7_item_transfer_member1 value : fragments_factions7_item_transfer_member1
    =
  require
    (Migration_reader.matches value 196)
    "Invalid frozen fragments_factions7_item_transfer_member1";
  decode_fragments_factions7_item_transfer_member1 value

let read_save4 value : save4 =
  require (Migration_reader.matches value 202) "Invalid frozen save4";
  decode_save4 value

let read_literal_4 value : literal_4 =
  require (Migration_reader.matches value 203) "Invalid frozen literal_4";
  decode_literal_4 value

let read_save5 value : save5 =
  require (Migration_reader.matches value 235) "Invalid frozen save5";
  decode_save5 value

let read_literal_5 value : literal_5 =
  require (Migration_reader.matches value 236) "Invalid frozen literal_5";
  decode_literal_5 value

let read_save6 value : save6 =
  require (Migration_reader.matches value 268) "Invalid frozen save6";
  decode_save6 value

let read_literal_6 value : literal_6 =
  require (Migration_reader.matches value 269) "Invalid frozen literal_6";
  decode_literal_6 value

let read_save7 value : save7 =
  require (Migration_reader.matches value 301) "Invalid frozen save7";
  decode_save7 value

let read_literal_7 value : literal_7 =
  require (Migration_reader.matches value 302) "Invalid frozen literal_7";
  decode_literal_7 value

let read_save7_checkpoint value : save7_checkpoint =
  require (Migration_reader.matches value 303) "Invalid frozen save7_checkpoint";
  decode_save7_checkpoint value

let read_save7_checkpoint_state value : save7_checkpoint_state =
  require (Migration_reader.matches value 304) "Invalid frozen save7_checkpoint_state";
  decode_save7_checkpoint_state value

let read_save8 value : save8 =
  require (Migration_reader.matches value 333) "Invalid frozen save8";
  decode_save8 value

let read_literal_8 value : literal_8 =
  require (Migration_reader.matches value 334) "Invalid frozen literal_8";
  decode_literal_8 value

let read_save8_checkpoint value : save8_checkpoint =
  require (Migration_reader.matches value 335) "Invalid frozen save8_checkpoint";
  decode_save8_checkpoint value

let read_save8_checkpoint_state value : save8_checkpoint_state =
  require (Migration_reader.matches value 336) "Invalid frozen save8_checkpoint_state";
  decode_save8_checkpoint_state value

let read_save9 value : save9 =
  require (Migration_reader.matches value 365) "Invalid frozen save9";
  decode_save9 value

let read_literal_9 value : literal_9 =
  require (Migration_reader.matches value 366) "Invalid frozen literal_9";
  decode_literal_9 value

let read_save10 value : save10 =
  require (Migration_reader.matches value 397) "Invalid frozen save10";
  decode_save10 value

let read_literal_10 value : literal_10 =
  require (Migration_reader.matches value 398) "Invalid frozen literal_10";
  decode_literal_10 value

let read_save11 value : save11 =
  require (Migration_reader.matches value 429) "Invalid frozen save11";
  decode_save11 value

let read_literal_11 value : literal_11 =
  require (Migration_reader.matches value 430) "Invalid frozen literal_11";
  decode_literal_11 value

let read_save12 value : save12 =
  require (Migration_reader.matches value 461) "Invalid frozen save12";
  decode_save12 value

let read_literal_12 value : literal_12 =
  require (Migration_reader.matches value 462) "Invalid frozen literal_12";
  decode_literal_12 value

let read_save13 value : save13 =
  require (Migration_reader.matches value 493) "Invalid frozen save13";
  decode_save13 value

let read_literal_13 value : literal_13 =
  require (Migration_reader.matches value 494) "Invalid frozen literal_13";
  decode_literal_13 value

let read_save14 value : save14 =
  require (Migration_reader.matches value 525) "Invalid frozen save14";
  decode_save14 value

let read_literal_14 value : literal_14 =
  require (Migration_reader.matches value 526) "Invalid frozen literal_14";
  decode_literal_14 value

let read_save15 value : save15 =
  require (Migration_reader.matches value 557) "Invalid frozen save15";
  decode_save15 value

let read_literal_15 value : literal_15 =
  require (Migration_reader.matches value 558) "Invalid frozen literal_15";
  decode_literal_15 value

let read_save16 value : save16 =
  require (Migration_reader.matches value 589) "Invalid frozen save16";
  decode_save16 value

let read_literal_16 value : literal_16 =
  require (Migration_reader.matches value 590) "Invalid frozen literal_16";
  decode_literal_16 value

let read_save17 value : save17 =
  require (Migration_reader.matches value 621) "Invalid frozen save17";
  decode_save17 value

let read_literal_17 value : literal_17 =
  require (Migration_reader.matches value 622) "Invalid frozen literal_17";
  decode_literal_17 value

let read_save17_checkpoint value : save17_checkpoint =
  require (Migration_reader.matches value 623) "Invalid frozen save17_checkpoint";
  decode_save17_checkpoint value

let read_save17_checkpoint_state value : save17_checkpoint_state =
  require (Migration_reader.matches value 624) "Invalid frozen save17_checkpoint_state";
  decode_save17_checkpoint_state value

let read_save17_checkpoint_state_tiles value : save17_checkpoint_state_tiles =
  require (Migration_reader.matches value 627) "Invalid frozen save17_checkpoint_state_tiles";
  decode_save17_checkpoint_state_tiles value

let read_save17_checkpoint_state_tiles_item value : save17_checkpoint_state_tiles_item =
  require (Migration_reader.matches value 628) "Invalid frozen save17_checkpoint_state_tiles_item";
  decode_save17_checkpoint_state_tiles_item value

let read_save18 value : save18 =
  require (Migration_reader.matches value 653) "Invalid frozen save18";
  decode_save18 value

let read_literal_18 value : literal_18 =
  require (Migration_reader.matches value 654) "Invalid frozen literal_18";
  decode_literal_18 value

let read_save18_checkpoint value : save18_checkpoint =
  require (Migration_reader.matches value 655) "Invalid frozen save18_checkpoint";
  decode_save18_checkpoint value

let read_save18_checkpoint_state value : save18_checkpoint_state =
  require (Migration_reader.matches value 656) "Invalid frozen save18_checkpoint_state";
  decode_save18_checkpoint_state value

let read_save18_checkpoint_state_tiles value : save18_checkpoint_state_tiles =
  require (Migration_reader.matches value 659) "Invalid frozen save18_checkpoint_state_tiles";
  decode_save18_checkpoint_state_tiles value

let read_save18_checkpoint_state_tiles_item value : save18_checkpoint_state_tiles_item =
  require (Migration_reader.matches value 660) "Invalid frozen save18_checkpoint_state_tiles_item";
  decode_save18_checkpoint_state_tiles_item value

let read_save19 value : save19 =
  require (Migration_reader.matches value 685) "Invalid frozen save19";
  decode_save19 value

let read_literal_19 value : literal_19 =
  require (Migration_reader.matches value 686) "Invalid frozen literal_19";
  decode_literal_19 value

let read_save20 value : save20 =
  require (Migration_reader.matches value 717) "Invalid frozen save20";
  decode_save20 value

let read_literal_20 value : literal_20 =
  require (Migration_reader.matches value 718) "Invalid frozen literal_20";
  decode_literal_20 value

let read_save21 value : save21 =
  require (Migration_reader.matches value 749) "Invalid frozen save21";
  decode_save21 value

let read_literal_21 value : literal_21 =
  require (Migration_reader.matches value 750) "Invalid frozen literal_21";
  decode_literal_21 value

let read_save21_checkpoint value : save21_checkpoint =
  require (Migration_reader.matches value 751) "Invalid frozen save21_checkpoint";
  decode_save21_checkpoint value

let read_save21_checkpoint_state value : save21_checkpoint_state =
  require (Migration_reader.matches value 752) "Invalid frozen save21_checkpoint_state";
  decode_save21_checkpoint_state value

let read_fragments_native_state38_gold value : fragments_native_state38_gold =
  require (Migration_reader.matches value 772) "Invalid frozen fragments_native_state38_gold";
  decode_fragments_native_state38_gold value

let read_save22 value : save22 =
  require (Migration_reader.matches value 782) "Invalid frozen save22";
  decode_save22 value

let read_literal_22 value : literal_22 =
  require (Migration_reader.matches value 783) "Invalid frozen literal_22";
  decode_literal_22 value

let read_save23 value : save23 =
  require (Migration_reader.matches value 814) "Invalid frozen save23";
  decode_save23 value

let read_literal_23 value : literal_23 =
  require (Migration_reader.matches value 815) "Invalid frozen literal_23";
  decode_literal_23 value

let read_save24 value : save24 =
  require (Migration_reader.matches value 846) "Invalid frozen save24";
  decode_save24 value

let read_literal_24 value : literal_24 =
  require (Migration_reader.matches value 847) "Invalid frozen literal_24";
  decode_literal_24 value

let read_save25 value : save25 =
  require (Migration_reader.matches value 878) "Invalid frozen save25";
  decode_save25 value

let read_literal_25 value : literal_25 =
  require (Migration_reader.matches value 879) "Invalid frozen literal_25";
  decode_literal_25 value

let read_save26 value : save26 =
  require (Migration_reader.matches value 910) "Invalid frozen save26";
  decode_save26 value

let read_literal_26 value : literal_26 =
  require (Migration_reader.matches value 911) "Invalid frozen literal_26";
  decode_literal_26 value

let read_save26_checkpoint value : save26_checkpoint =
  require (Migration_reader.matches value 912) "Invalid frozen save26_checkpoint";
  decode_save26_checkpoint value

let read_save26_checkpoint_state value : save26_checkpoint_state =
  require (Migration_reader.matches value 913) "Invalid frozen save26_checkpoint_state";
  decode_save26_checkpoint_state value

let read_save26_checkpoint_state_units value : save26_checkpoint_state_units =
  require (Migration_reader.matches value 914) "Invalid frozen save26_checkpoint_state_units";
  decode_save26_checkpoint_state_units value

let read_save26_checkpoint_state_units_item value : save26_checkpoint_state_units_item =
  require (Migration_reader.matches value 915) "Invalid frozen save26_checkpoint_state_units_item";
  decode_save26_checkpoint_state_units_item value

let read_save26_checkpoint_state_tiles value : save26_checkpoint_state_tiles =
  require (Migration_reader.matches value 916) "Invalid frozen save26_checkpoint_state_tiles";
  decode_save26_checkpoint_state_tiles value

let read_save26_checkpoint_state_tiles_item value : save26_checkpoint_state_tiles_item =
  require (Migration_reader.matches value 917) "Invalid frozen save26_checkpoint_state_tiles_item";
  decode_save26_checkpoint_state_tiles_item value

let read_fragments_light28_member1_when_ value : fragments_light28_member1_when_ =
  require (Migration_reader.matches value 918) "Invalid frozen fragments_light28_member1_when_";
  decode_fragments_light28_member1_when_ value

let read_fragments_light28_member1_when__member1 value : fragments_light28_member1_when__member1 =
  require
    (Migration_reader.matches value 919)
    "Invalid frozen fragments_light28_member1_when__member1";
  decode_fragments_light28_member1_when__member1 value

let read_fragments_light28_member1_when__member1_condition value :
    fragments_light28_member1_when__member1_condition =
  require
    (Migration_reader.matches value 920)
    "Invalid frozen fragments_light28_member1_when__member1_condition";
  decode_fragments_light28_member1_when__member1_condition value

let read_fragments_light28_member1_when__member1_condition_member1 value :
    fragments_light28_member1_when__member1_condition_member1 =
  require
    (Migration_reader.matches value 921)
    "Invalid frozen fragments_light28_member1_when__member1_condition_member1";
  decode_fragments_light28_member1_when__member1_condition_member1 value

let read_fragments_light28_member1_when__member1_condition_member3 value :
    fragments_light28_member1_when__member1_condition_member3 =
  require
    (Migration_reader.matches value 922)
    "Invalid frozen fragments_light28_member1_when__member1_condition_member3";
  decode_fragments_light28_member1_when__member1_condition_member3 value

let read_fragments_light28_member1_when__member1_condition_member4 value :
    fragments_light28_member1_when__member1_condition_member4 =
  require
    (Migration_reader.matches value 923)
    "Invalid frozen fragments_light28_member1_when__member1_condition_member4";
  decode_fragments_light28_member1_when__member1_condition_member4 value

let read_fragments_light28_member1_when__member1_condition_member9 value :
    fragments_light28_member1_when__member1_condition_member9 =
  require
    (Migration_reader.matches value 924)
    "Invalid frozen fragments_light28_member1_when__member1_condition_member9";
  decode_fragments_light28_member1_when__member1_condition_member9 value

let read_fragments_light28_member1_when__member1_condition_member9_conditions value :
    fragments_light28_member1_when__member1_condition_member9_conditions =
  require
    (Migration_reader.matches value 925)
    "Invalid frozen fragments_light28_member1_when__member1_condition_member9_conditions";
  decode_fragments_light28_member1_when__member1_condition_member9_conditions value

let read_save26_checkpoint_state_tiles_item_light value : save26_checkpoint_state_tiles_item_light =
  require
    (Migration_reader.matches value 926)
    "Invalid frozen save26_checkpoint_state_tiles_item_light";
  decode_save26_checkpoint_state_tiles_item_light value

let read_fragments_light28_member1 value : fragments_light28_member1 =
  require (Migration_reader.matches value 927) "Invalid frozen fragments_light28_member1";
  decode_fragments_light28_member1 value

let read_save26_checkpoint_state_campaign value : save26_checkpoint_state_campaign =
  require (Migration_reader.matches value 928) "Invalid frozen save26_checkpoint_state_campaign";
  decode_save26_checkpoint_state_campaign value

let read_save26_checkpoint_state_campaign_member1 value : save26_checkpoint_state_campaign_member1 =
  require
    (Migration_reader.matches value 929)
    "Invalid frozen save26_checkpoint_state_campaign_member1";
  decode_save26_checkpoint_state_campaign_member1 value

let read_save26_checkpoint_state_campaign_member1_veterans value :
    save26_checkpoint_state_campaign_member1_veterans =
  require
    (Migration_reader.matches value 930)
    "Invalid frozen save26_checkpoint_state_campaign_member1_veterans";
  decode_save26_checkpoint_state_campaign_member1_veterans value

let read_save26_checkpoint_state_campaign_member1_veterans_member1 value :
    save26_checkpoint_state_campaign_member1_veterans_member1 =
  require
    (Migration_reader.matches value 931)
    "Invalid frozen save26_checkpoint_state_campaign_member1_veterans_member1";
  decode_save26_checkpoint_state_campaign_member1_veterans_member1 value

let read_save26_checkpoint_state_campaign_member1_veterans_member1_item value :
    save26_checkpoint_state_campaign_member1_veterans_member1_item =
  require
    (Migration_reader.matches value 932)
    "Invalid frozen save26_checkpoint_state_campaign_member1_veterans_member1_item";
  decode_save26_checkpoint_state_campaign_member1_veterans_member1_item value

let read_save26_checkpoint_state_campaign_member1_evacuated value :
    save26_checkpoint_state_campaign_member1_evacuated =
  require
    (Migration_reader.matches value 933)
    "Invalid frozen save26_checkpoint_state_campaign_member1_evacuated";
  decode_save26_checkpoint_state_campaign_member1_evacuated value

let read_save26_checkpoint_state_campaign_member1_evacuated_member1 value :
    save26_checkpoint_state_campaign_member1_evacuated_member1 =
  require
    (Migration_reader.matches value 934)
    "Invalid frozen save26_checkpoint_state_campaign_member1_evacuated_member1";
  decode_save26_checkpoint_state_campaign_member1_evacuated_member1 value

let read_save26_checkpoint_state_campaign_member1_evacuated_member1_item value :
    save26_checkpoint_state_campaign_member1_evacuated_member1_item =
  require
    (Migration_reader.matches value 935)
    "Invalid frozen save26_checkpoint_state_campaign_member1_evacuated_member1_item";
  decode_save26_checkpoint_state_campaign_member1_evacuated_member1_item value

let read_save26_checkpoint_state_factions value : save26_checkpoint_state_factions =
  require (Migration_reader.matches value 936) "Invalid frozen save26_checkpoint_state_factions";
  decode_save26_checkpoint_state_factions value

let read_save26_checkpoint_state_factions_item value : save26_checkpoint_state_factions_item =
  require
    (Migration_reader.matches value 937)
    "Invalid frozen save26_checkpoint_state_factions_item";
  decode_save26_checkpoint_state_factions_item value

let read_save26_checkpoint_state_factions_item_transfer value :
    save26_checkpoint_state_factions_item_transfer =
  require
    (Migration_reader.matches value 938)
    "Invalid frozen save26_checkpoint_state_factions_item_transfer";
  decode_save26_checkpoint_state_factions_item_transfer value

let read_save26_checkpoint_state_factions_item_transfer_member1 value :
    save26_checkpoint_state_factions_item_transfer_member1 =
  require
    (Migration_reader.matches value 939)
    "Invalid frozen save26_checkpoint_state_factions_item_transfer_member1";
  decode_save26_checkpoint_state_factions_item_transfer_member1 value

let read_save27 value : save27 =
  require (Migration_reader.matches value 945) "Invalid frozen save27";
  decode_save27 value

let read_literal_27 value : literal_27 =
  require (Migration_reader.matches value 946) "Invalid frozen literal_27";
  decode_literal_27 value

let read_save28 value : save28 =
  require (Migration_reader.matches value 977) "Invalid frozen save28";
  decode_save28 value

let read_literal_28 value : literal_28 =
  require (Migration_reader.matches value 978) "Invalid frozen literal_28";
  decode_literal_28 value

let read_save28_checkpoint value : save28_checkpoint =
  require (Migration_reader.matches value 979) "Invalid frozen save28_checkpoint";
  decode_save28_checkpoint value

let read_save28_checkpoint_state value : save28_checkpoint_state =
  require (Migration_reader.matches value 980) "Invalid frozen save28_checkpoint_state";
  decode_save28_checkpoint_state value

let read_save28_checkpoint_state_campaign value : save28_checkpoint_state_campaign =
  require (Migration_reader.matches value 992) "Invalid frozen save28_checkpoint_state_campaign";
  decode_save28_checkpoint_state_campaign value

let read_save28_checkpoint_state_campaign_member1 value : save28_checkpoint_state_campaign_member1 =
  require
    (Migration_reader.matches value 993)
    "Invalid frozen save28_checkpoint_state_campaign_member1";
  decode_save28_checkpoint_state_campaign_member1 value

let read_save28_checkpoint_state_campaign_member1_scripts value :
    save28_checkpoint_state_campaign_member1_scripts =
  require
    (Migration_reader.matches value 1000)
    "Invalid frozen save28_checkpoint_state_campaign_member1_scripts";
  decode_save28_checkpoint_state_campaign_member1_scripts value

let read_save28_checkpoint_state_campaign_member1_scripts_member1 value :
    save28_checkpoint_state_campaign_member1_scripts_member1 =
  require
    (Migration_reader.matches value 1001)
    "Invalid frozen save28_checkpoint_state_campaign_member1_scripts_member1";
  decode_save28_checkpoint_state_campaign_member1_scripts_member1 value

let read_fragments_native_state38_campaign_member1_scripts_member1_recruitment value :
    fragments_native_state38_campaign_member1_scripts_member1_recruitment =
  require
    (Migration_reader.matches value 1002)
    "Invalid frozen fragments_native_state38_campaign_member1_scripts_member1_recruitment";
  decode_fragments_native_state38_campaign_member1_scripts_member1_recruitment value

let read_fragments_native_state38_campaign_member1_scripts_member1_holds value :
    fragments_native_state38_campaign_member1_scripts_member1_holds =
  require
    (Migration_reader.matches value 1003)
    "Invalid frozen fragments_native_state38_campaign_member1_scripts_member1_holds";
  decode_fragments_native_state38_campaign_member1_scripts_member1_holds value

let read_fragments_native_state38_campaign_member1_scripts_member1_holds_item value :
    fragments_native_state38_campaign_member1_scripts_member1_holds_item =
  require
    (Migration_reader.matches value 1004)
    "Invalid frozen fragments_native_state38_campaign_member1_scripts_member1_holds_item";
  decode_fragments_native_state38_campaign_member1_scripts_member1_holds_item value

let read_fragments_native_state38_campaign_member1_scripts_member1_pending value :
    fragments_native_state38_campaign_member1_scripts_member1_pending =
  require
    (Migration_reader.matches value 1005)
    "Invalid frozen fragments_native_state38_campaign_member1_scripts_member1_pending";
  decode_fragments_native_state38_campaign_member1_scripts_member1_pending value

let read_fragments_native_state38_campaign_member1_scripts_member1_pending_item value :
    fragments_native_state38_campaign_member1_scripts_member1_pending_item =
  require
    (Migration_reader.matches value 1006)
    "Invalid frozen fragments_native_state38_campaign_member1_scripts_member1_pending_item";
  decode_fragments_native_state38_campaign_member1_scripts_member1_pending_item value

let read_fragments_delta19_item_owner value : fragments_delta19_item_owner =
  require (Migration_reader.matches value 1007) "Invalid frozen fragments_delta19_item_owner";
  decode_fragments_delta19_item_owner value

let read_fragments_native_state38_campaign_member1_scripts_member1_pending_item_units value :
    fragments_native_state38_campaign_member1_scripts_member1_pending_item_units =
  require
    (Migration_reader.matches value 1008)
    "Invalid frozen fragments_native_state38_campaign_member1_scripts_member1_pending_item_units";
  decode_fragments_native_state38_campaign_member1_scripts_member1_pending_item_units value

let read_fragments_native_state38_campaign_member1_scripts_member1_pending_item_units_item value :
    fragments_native_state38_campaign_member1_scripts_member1_pending_item_units_item =
  require
    (Migration_reader.matches value 1009)
    "Invalid frozen \
     fragments_native_state38_campaign_member1_scripts_member1_pending_item_units_item";
  decode_fragments_native_state38_campaign_member1_scripts_member1_pending_item_units_item value

let read_fragments_native_state38_campaign_member1_scripts_member1_pending_item_units_item_role
    value : fragments_native_state38_campaign_member1_scripts_member1_pending_item_units_item_role =
  require
    (Migration_reader.matches value 1010)
    "Invalid frozen \
     fragments_native_state38_campaign_member1_scripts_member1_pending_item_units_item_role";
  decode_fragments_native_state38_campaign_member1_scripts_member1_pending_item_units_item_role
    value

let read_save28_checkpoint_state_campaign_member1_scripts_member1_marks value :
    save28_checkpoint_state_campaign_member1_scripts_member1_marks =
  require
    (Migration_reader.matches value 1011)
    "Invalid frozen save28_checkpoint_state_campaign_member1_scripts_member1_marks";
  decode_save28_checkpoint_state_campaign_member1_scripts_member1_marks value

let read_save28_checkpoint_state_campaign_member1_scripts_member1_marks_item value :
    save28_checkpoint_state_campaign_member1_scripts_member1_marks_item =
  require
    (Migration_reader.matches value 1012)
    "Invalid frozen save28_checkpoint_state_campaign_member1_scripts_member1_marks_item";
  decode_save28_checkpoint_state_campaign_member1_scripts_member1_marks_item value

let read_save29 value : save29 =
  require (Migration_reader.matches value 1022) "Invalid frozen save29";
  decode_save29 value

let read_literal_29 value : literal_29 =
  require (Migration_reader.matches value 1023) "Invalid frozen literal_29";
  decode_literal_29 value

let read_save29_checkpoint value : save29_checkpoint =
  require (Migration_reader.matches value 1024) "Invalid frozen save29_checkpoint";
  decode_save29_checkpoint value

let read_save29_checkpoint_state value : save29_checkpoint_state =
  require (Migration_reader.matches value 1025) "Invalid frozen save29_checkpoint_state";
  decode_save29_checkpoint_state value

let read_save29_checkpoint_state_campaign value : save29_checkpoint_state_campaign =
  require (Migration_reader.matches value 1037) "Invalid frozen save29_checkpoint_state_campaign";
  decode_save29_checkpoint_state_campaign value

let read_save29_checkpoint_state_campaign_member1 value : save29_checkpoint_state_campaign_member1 =
  require
    (Migration_reader.matches value 1038)
    "Invalid frozen save29_checkpoint_state_campaign_member1";
  decode_save29_checkpoint_state_campaign_member1 value

let read_fragments_native_state38_campaign_member1_flags value :
    fragments_native_state38_campaign_member1_flags =
  require
    (Migration_reader.matches value 1049)
    "Invalid frozen fragments_native_state38_campaign_member1_flags";
  decode_fragments_native_state38_campaign_member1_flags value

let read_fragments_native_state38_campaign_member1_flags_value value :
    fragments_native_state38_campaign_member1_flags_value =
  require
    (Migration_reader.matches value 1050)
    "Invalid frozen fragments_native_state38_campaign_member1_flags_value";
  decode_fragments_native_state38_campaign_member1_flags_value value

let read_fragments_native_state38_campaign_member1_flags_value_item value :
    fragments_native_state38_campaign_member1_flags_value_item =
  require
    (Migration_reader.matches value 1051)
    "Invalid frozen fragments_native_state38_campaign_member1_flags_value_item";
  decode_fragments_native_state38_campaign_member1_flags_value_item value

let read_save30 value : save30 =
  require (Migration_reader.matches value 1061) "Invalid frozen save30";
  decode_save30 value

let read_literal_30 value : literal_30 =
  require (Migration_reader.matches value 1062) "Invalid frozen literal_30";
  decode_literal_30 value

let read_save30_setup value : save30_setup =
  require (Migration_reader.matches value 1092) "Invalid frozen save30_setup";
  decode_save30_setup value

let read_save35_setup value : save35_setup =
  require (Migration_reader.matches value 1093) "Invalid frozen save35_setup";
  decode_save35_setup value

let read_save35_setup_enemy value : save35_setup_enemy =
  require (Migration_reader.matches value 1094) "Invalid frozen save35_setup_enemy";
  decode_save35_setup_enemy value

let read_save35_setup_enemy_diplomacy value : save35_setup_enemy_diplomacy =
  require (Migration_reader.matches value 1095) "Invalid frozen save35_setup_enemy_diplomacy";
  decode_save35_setup_enemy_diplomacy value

let read_save31 value : save31 =
  require (Migration_reader.matches value 1098) "Invalid frozen save31";
  decode_save31 value

let read_literal_31 value : literal_31 =
  require (Migration_reader.matches value 1099) "Invalid frozen literal_31";
  decode_literal_31 value

let read_save32 value : save32 =
  require (Migration_reader.matches value 1134) "Invalid frozen save32";
  decode_save32 value

let read_literal_32 value : literal_32 =
  require (Migration_reader.matches value 1135) "Invalid frozen literal_32";
  decode_literal_32 value

let read_save33 value : save33 =
  require (Migration_reader.matches value 1170) "Invalid frozen save33";
  decode_save33 value

let read_literal_33 value : literal_33 =
  require (Migration_reader.matches value 1171) "Invalid frozen literal_33";
  decode_literal_33 value

let read_save34 value : save34 =
  require (Migration_reader.matches value 1206) "Invalid frozen save34";
  decode_save34 value

let read_literal_34 value : literal_34 =
  require (Migration_reader.matches value 1207) "Invalid frozen literal_34";
  decode_literal_34 value

let read_save34_checkpoint value : save34_checkpoint =
  require (Migration_reader.matches value 1208) "Invalid frozen save34_checkpoint";
  decode_save34_checkpoint value

let read_save34_checkpoint_state value : save34_checkpoint_state =
  require (Migration_reader.matches value 1209) "Invalid frozen save34_checkpoint_state";
  decode_save34_checkpoint_state value

let read_save34_checkpoint_state_campaign value : save34_checkpoint_state_campaign =
  require (Migration_reader.matches value 1221) "Invalid frozen save34_checkpoint_state_campaign";
  decode_save34_checkpoint_state_campaign value

let read_save34_checkpoint_state_campaign_member1 value : save34_checkpoint_state_campaign_member1 =
  require
    (Migration_reader.matches value 1222)
    "Invalid frozen save34_checkpoint_state_campaign_member1";
  decode_save34_checkpoint_state_campaign_member1 value

let read_save34_checkpoint_state_campaign_member1_scripts value :
    save34_checkpoint_state_campaign_member1_scripts =
  require
    (Migration_reader.matches value 1229)
    "Invalid frozen save34_checkpoint_state_campaign_member1_scripts";
  decode_save34_checkpoint_state_campaign_member1_scripts value

let read_save34_checkpoint_state_campaign_member1_scripts_member1 value :
    save34_checkpoint_state_campaign_member1_scripts_member1 =
  require
    (Migration_reader.matches value 1230)
    "Invalid frozen save34_checkpoint_state_campaign_member1_scripts_member1";
  decode_save34_checkpoint_state_campaign_member1_scripts_member1 value

let read_save34_undo value : save34_undo =
  require (Migration_reader.matches value 1242) "Invalid frozen save34_undo";
  decode_save34_undo value

let read_save34_undo_member1 value : save34_undo_member1 =
  require (Migration_reader.matches value 1243) "Invalid frozen save34_undo_member1";
  decode_save34_undo_member1 value

let read_save34_undo_member1_item value : save34_undo_member1_item =
  require (Migration_reader.matches value 1244) "Invalid frozen save34_undo_member1_item";
  decode_save34_undo_member1_item value

let read_save34_undo_member1_item_changes value : save34_undo_member1_item_changes =
  require (Migration_reader.matches value 1245) "Invalid frozen save34_undo_member1_item_changes";
  decode_save34_undo_member1_item_changes value

let read_save34_undo_member1_item_changes_item value : save34_undo_member1_item_changes_item =
  require
    (Migration_reader.matches value 1246)
    "Invalid frozen save34_undo_member1_item_changes_item";
  decode_save34_undo_member1_item_changes_item value

let read_save34_undo_member1_item_changes_item_member0 value :
    save34_undo_member1_item_changes_item_member0 =
  require
    (Migration_reader.matches value 1247)
    "Invalid frozen save34_undo_member1_item_changes_item_member0";
  decode_save34_undo_member1_item_changes_item_member0 value

let read_literal_set value : literal_set =
  require (Migration_reader.matches value 1248) "Invalid frozen literal_set";
  decode_literal_set value

let read_save34_undo_member1_item_changes_item_member1 value :
    save34_undo_member1_item_changes_item_member1 =
  require
    (Migration_reader.matches value 1249)
    "Invalid frozen save34_undo_member1_item_changes_item_member1";
  decode_save34_undo_member1_item_changes_item_member1 value

let read_literal_remove value : literal_remove =
  require (Migration_reader.matches value 1250) "Invalid frozen literal_remove";
  decode_literal_remove value

let read_save35 value : save35 =
  require (Migration_reader.matches value 1251) "Invalid frozen save35";
  decode_save35 value

let read_literal_35 value : literal_35 =
  require (Migration_reader.matches value 1252) "Invalid frozen literal_35";
  decode_literal_35 value

let read_save35_checkpoint value : save35_checkpoint =
  require (Migration_reader.matches value 1253) "Invalid frozen save35_checkpoint";
  decode_save35_checkpoint value

let read_save35_checkpoint_state value : save35_checkpoint_state =
  require (Migration_reader.matches value 1254) "Invalid frozen save35_checkpoint_state";
  decode_save35_checkpoint_state value

let read_save35_progress value : save35_progress =
  require (Migration_reader.matches value 1284) "Invalid frozen save35_progress";
  decode_save35_progress value

let read_save35_progress_entry value : save35_progress_entry =
  require (Migration_reader.matches value 1285) "Invalid frozen save35_progress_entry";
  decode_save35_progress_entry value

let read_save35_progress_entry_kind value : save35_progress_entry_kind =
  require (Migration_reader.matches value 1286) "Invalid frozen save35_progress_entry_kind";
  decode_save35_progress_entry_kind value

let read_literal_manual value : literal_manual =
  require (Migration_reader.matches value 1287) "Invalid frozen literal_manual";
  decode_literal_manual value

let read_save36 value : save36 =
  require (Migration_reader.matches value 1288) "Invalid frozen save36";
  decode_save36 value

let read_literal_36 value : literal_36 =
  require (Migration_reader.matches value 1289) "Invalid frozen literal_36";
  decode_literal_36 value

let read_save37 value : save37 =
  require (Migration_reader.matches value 1322) "Invalid frozen save37";
  decode_save37 value

let read_literal_37 value : literal_37 =
  require (Migration_reader.matches value 1323) "Invalid frozen literal_37";
  decode_literal_37 value

let read_save37_progress value : save37_progress =
  require (Migration_reader.matches value 1355) "Invalid frozen save37_progress";
  decode_save37_progress value

let read_save37_progress_entry value : save37_progress_entry =
  require (Migration_reader.matches value 1356) "Invalid frozen save37_progress_entry";
  decode_save37_progress_entry value

let read_save37_progress_entry_member0 value : save37_progress_entry_member0 =
  require (Migration_reader.matches value 1357) "Invalid frozen save37_progress_entry_member0";
  decode_save37_progress_entry_member0 value

let read_literal_flash value : literal_flash =
  require (Migration_reader.matches value 1358) "Invalid frozen literal_flash";
  decode_literal_flash value

let read_history4_entries_item value : history4_entries_item =
  require (Migration_reader.matches value 1359) "Invalid frozen history4_entries_item";
  decode_history4_entries_item value

let read_save38 value : save38 =
  require (Migration_reader.matches value 1360) "Invalid frozen save38";
  decode_save38 value

let read_literal_38 value : literal_38 =
  require (Migration_reader.matches value 1361) "Invalid frozen literal_38";
  decode_literal_38 value

let read_save39 value : save39 =
  require (Migration_reader.matches value 1394) "Invalid frozen save39";
  decode_save39 value

let read_save39_member0 value : save39_member0 =
  require (Migration_reader.matches value 1395) "Invalid frozen save39_member0";
  decode_save39_member0 value

let read_save39_member0_progress value : save39_member0_progress =
  require (Migration_reader.matches value 1402) "Invalid frozen save39_member0_progress";
  decode_save39_member0_progress value

let read_literal_39 value : literal_39 =
  require (Migration_reader.matches value 1403) "Invalid frozen literal_39";
  decode_literal_39 value

let read_save39_member0_checkpoint value : save39_member0_checkpoint =
  require (Migration_reader.matches value 1404) "Invalid frozen save39_member0_checkpoint";
  decode_save39_member0_checkpoint value

let read_save39_member0_checkpoint_state value : save39_member0_checkpoint_state =
  require (Migration_reader.matches value 1405) "Invalid frozen save39_member0_checkpoint_state";
  decode_save39_member0_checkpoint_state value

let read_save39_member0_checkpoint_state_factions value : save39_member0_checkpoint_state_factions =
  require
    (Migration_reader.matches value 1406)
    "Invalid frozen save39_member0_checkpoint_state_factions";
  decode_save39_member0_checkpoint_state_factions value

let read_save39_member0_checkpoint_state_factions_item value :
    save39_member0_checkpoint_state_factions_item =
  require
    (Migration_reader.matches value 1407)
    "Invalid frozen save39_member0_checkpoint_state_factions_item";
  decode_save39_member0_checkpoint_state_factions_item value

let read_save39_member0_checkpoint_state_factions_item_transfer value :
    save39_member0_checkpoint_state_factions_item_transfer =
  require
    (Migration_reader.matches value 1419)
    "Invalid frozen save39_member0_checkpoint_state_factions_item_transfer";
  decode_save39_member0_checkpoint_state_factions_item_transfer value

let read_save39_member0_checkpoint_state_factions_item_transfer_member1 value :
    save39_member0_checkpoint_state_factions_item_transfer_member1 =
  require
    (Migration_reader.matches value 1420)
    "Invalid frozen save39_member0_checkpoint_state_factions_item_transfer_member1";
  decode_save39_member0_checkpoint_state_factions_item_transfer_member1 value

let read_save39_member0_checkpoint_state_tiles_item_fog_member1 value :
    save39_member0_checkpoint_state_tiles_item_fog_member1 =
  require
    (Migration_reader.matches value 1421)
    "Invalid frozen save39_member0_checkpoint_state_tiles_item_fog_member1";
  decode_save39_member0_checkpoint_state_tiles_item_fog_member1 value

let read_save39_member0_checkpoint_state_tiles_item_fog_member1_condition value :
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition =
  require
    (Migration_reader.matches value 1422)
    "Invalid frozen save39_member0_checkpoint_state_tiles_item_fog_member1_condition";
  decode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition value

let read_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0 value :
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0 =
  require
    (Migration_reader.matches value 1423)
    "Invalid frozen save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0";
  decode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0 value

let read_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile value :
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile =
  require
    (Migration_reader.matches value 1424)
    "Invalid frozen save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile";
  decode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile value

let read_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member1 value
    : save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member1 =
  require
    (Migration_reader.matches value 1425)
    "Invalid frozen \
     save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member1";
  decode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member1 value

let read_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member2 value
    : save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member2 =
  require
    (Migration_reader.matches value 1426)
    "Invalid frozen \
     save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member2";
  decode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member2 value

let read_save39_member0_checkpoint_state_ground_items_item_tile value :
    save39_member0_checkpoint_state_ground_items_item_tile =
  require
    (Migration_reader.matches value 1427)
    "Invalid frozen save39_member0_checkpoint_state_ground_items_item_tile";
  decode_save39_member0_checkpoint_state_ground_items_item_tile value

let read_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member3 value
    : save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member3 =
  require
    (Migration_reader.matches value 1428)
    "Invalid frozen \
     save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member3";
  decode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member3 value

let read_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member3_binding
    value :
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member3_binding =
  require
    (Migration_reader.matches value 1429)
    "Invalid frozen \
     save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member3_binding";
  decode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member3_binding
    value

let read_literal_unit value : literal_unit =
  require (Migration_reader.matches value 1430) "Invalid frozen literal_unit";
  decode_literal_unit value

let read_literal_mark value : literal_mark =
  require (Migration_reader.matches value 1431) "Invalid frozen literal_mark";
  decode_literal_mark value

let read_literal_death value : literal_death =
  require (Migration_reader.matches value 1432) "Invalid frozen literal_death";
  decode_literal_death value

let read_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member1 value :
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member1 =
  require
    (Migration_reader.matches value 1433)
    "Invalid frozen save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member1";
  decode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member1 value

let read_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member9 value :
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member9 =
  require
    (Migration_reader.matches value 1442)
    "Invalid frozen save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member9";
  decode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member9 value

let read_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member9_conditions value :
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member9_conditions =
  require
    (Migration_reader.matches value 1443)
    "Invalid frozen \
     save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member9_conditions";
  decode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member9_conditions value

let read_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member10 value :
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member10 =
  require
    (Migration_reader.matches value 1444)
    "Invalid frozen save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member10";
  decode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member10 value

let read_save39_member0_checkpoint_state_ground_items value :
    save39_member0_checkpoint_state_ground_items =
  require
    (Migration_reader.matches value 1445)
    "Invalid frozen save39_member0_checkpoint_state_ground_items";
  decode_save39_member0_checkpoint_state_ground_items value

let read_save39_member0_checkpoint_state_ground_items_item value :
    save39_member0_checkpoint_state_ground_items_item =
  require
    (Migration_reader.matches value 1446)
    "Invalid frozen save39_member0_checkpoint_state_ground_items_item";
  decode_save39_member0_checkpoint_state_ground_items_item value

let read_save39_member0_checkpoint_state_tiles value : save39_member0_checkpoint_state_tiles =
  require
    (Migration_reader.matches value 1458)
    "Invalid frozen save39_member0_checkpoint_state_tiles";
  decode_save39_member0_checkpoint_state_tiles value

let read_save39_member0_checkpoint_state_tiles_item value :
    save39_member0_checkpoint_state_tiles_item =
  require
    (Migration_reader.matches value 1459)
    "Invalid frozen save39_member0_checkpoint_state_tiles_item";
  decode_save39_member0_checkpoint_state_tiles_item value

let read_save39_member0_checkpoint_state_tiles_item_fog value :
    save39_member0_checkpoint_state_tiles_item_fog =
  require
    (Migration_reader.matches value 1463)
    "Invalid frozen save39_member0_checkpoint_state_tiles_item_fog";
  decode_save39_member0_checkpoint_state_tiles_item_fog value

let read_save39_member0_checkpoint_state_tiles_item_light value :
    save39_member0_checkpoint_state_tiles_item_light =
  require
    (Migration_reader.matches value 1464)
    "Invalid frozen save39_member0_checkpoint_state_tiles_item_light";
  decode_save39_member0_checkpoint_state_tiles_item_light value

let read_save39_member0_checkpoint_state_tiles_item_light_member1 value :
    save39_member0_checkpoint_state_tiles_item_light_member1 =
  require
    (Migration_reader.matches value 1465)
    "Invalid frozen save39_member0_checkpoint_state_tiles_item_light_member1";
  decode_save39_member0_checkpoint_state_tiles_item_light_member1 value

let read_save39_member0_checkpoint_state_units value : save39_member0_checkpoint_state_units =
  require
    (Migration_reader.matches value 1466)
    "Invalid frozen save39_member0_checkpoint_state_units";
  decode_save39_member0_checkpoint_state_units value

let read_save39_member0_checkpoint_state_units_item value :
    save39_member0_checkpoint_state_units_item =
  require
    (Migration_reader.matches value 1467)
    "Invalid frozen save39_member0_checkpoint_state_units_item";
  decode_save39_member0_checkpoint_state_units_item value

let read_save39_member0_checkpoint_state_campaign value : save39_member0_checkpoint_state_campaign =
  require
    (Migration_reader.matches value 1471)
    "Invalid frozen save39_member0_checkpoint_state_campaign";
  decode_save39_member0_checkpoint_state_campaign value

let read_save39_member0_checkpoint_state_campaign_member1 value :
    save39_member0_checkpoint_state_campaign_member1 =
  require
    (Migration_reader.matches value 1472)
    "Invalid frozen save39_member0_checkpoint_state_campaign_member1";
  decode_save39_member0_checkpoint_state_campaign_member1 value

let read_save39_member0_checkpoint_state_campaign_member1_scripts value :
    save39_member0_checkpoint_state_campaign_member1_scripts =
  require
    (Migration_reader.matches value 1473)
    "Invalid frozen save39_member0_checkpoint_state_campaign_member1_scripts";
  decode_save39_member0_checkpoint_state_campaign_member1_scripts value

let read_save39_member0_checkpoint_state_campaign_member1_scripts_member1 value :
    save39_member0_checkpoint_state_campaign_member1_scripts_member1 =
  require
    (Migration_reader.matches value 1474)
    "Invalid frozen save39_member0_checkpoint_state_campaign_member1_scripts_member1";
  decode_save39_member0_checkpoint_state_campaign_member1_scripts_member1 value

let read_save39_member0_checkpoint_state_campaign_member1_scripts_member1_marks value :
    save39_member0_checkpoint_state_campaign_member1_scripts_member1_marks =
  require
    (Migration_reader.matches value 1478)
    "Invalid frozen save39_member0_checkpoint_state_campaign_member1_scripts_member1_marks";
  decode_save39_member0_checkpoint_state_campaign_member1_scripts_member1_marks value

let read_save39_member0_checkpoint_state_campaign_member1_scripts_member1_marks_item value :
    save39_member0_checkpoint_state_campaign_member1_scripts_member1_marks_item =
  require
    (Migration_reader.matches value 1479)
    "Invalid frozen save39_member0_checkpoint_state_campaign_member1_scripts_member1_marks_item";
  decode_save39_member0_checkpoint_state_campaign_member1_scripts_member1_marks_item value

let read_save39_member0_checkpoint_state_campaign_member1_scripts_member1_pending value :
    save39_member0_checkpoint_state_campaign_member1_scripts_member1_pending =
  require
    (Migration_reader.matches value 1480)
    "Invalid frozen save39_member0_checkpoint_state_campaign_member1_scripts_member1_pending";
  decode_save39_member0_checkpoint_state_campaign_member1_scripts_member1_pending value

let read_save39_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item value :
    save39_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item =
  require
    (Migration_reader.matches value 1481)
    "Invalid frozen save39_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item";
  decode_save39_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item value

let read_save39_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item_area value :
    save39_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item_area =
  require
    (Migration_reader.matches value 1482)
    "Invalid frozen \
     save39_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item_area";
  decode_save39_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item_area value

let read_save39_member0_checkpoint_state_campaign_member1_evacuated value :
    save39_member0_checkpoint_state_campaign_member1_evacuated =
  require
    (Migration_reader.matches value 1486)
    "Invalid frozen save39_member0_checkpoint_state_campaign_member1_evacuated";
  decode_save39_member0_checkpoint_state_campaign_member1_evacuated value

let read_save39_member0_checkpoint_state_campaign_member1_evacuated_member1 value :
    save39_member0_checkpoint_state_campaign_member1_evacuated_member1 =
  require
    (Migration_reader.matches value 1487)
    "Invalid frozen save39_member0_checkpoint_state_campaign_member1_evacuated_member1";
  decode_save39_member0_checkpoint_state_campaign_member1_evacuated_member1 value

let read_save39_member0_checkpoint_state_campaign_member1_evacuated_member1_item value :
    save39_member0_checkpoint_state_campaign_member1_evacuated_member1_item =
  require
    (Migration_reader.matches value 1488)
    "Invalid frozen save39_member0_checkpoint_state_campaign_member1_evacuated_member1_item";
  decode_save39_member0_checkpoint_state_campaign_member1_evacuated_member1_item value

let read_fragments_native_state38_campaign_member1_veterans value :
    fragments_native_state38_campaign_member1_veterans =
  require
    (Migration_reader.matches value 1489)
    "Invalid frozen fragments_native_state38_campaign_member1_veterans";
  decode_fragments_native_state38_campaign_member1_veterans value

let read_fragments_native_state38_campaign_member1_veterans_member1 value :
    fragments_native_state38_campaign_member1_veterans_member1 =
  require
    (Migration_reader.matches value 1490)
    "Invalid frozen fragments_native_state38_campaign_member1_veterans_member1";
  decode_fragments_native_state38_campaign_member1_veterans_member1 value

let read_fragments_native_state38_campaign_member1_veterans_member1_item value :
    fragments_native_state38_campaign_member1_veterans_member1_item =
  require
    (Migration_reader.matches value 1491)
    "Invalid frozen fragments_native_state38_campaign_member1_veterans_member1_item";
  decode_fragments_native_state38_campaign_member1_veterans_member1_item value

let read_save39_member0_setup value : save39_member0_setup =
  require (Migration_reader.matches value 1505) "Invalid frozen save39_member0_setup";
  decode_save39_member0_setup value

let read_save39_member0_setup_enemy value : save39_member0_setup_enemy =
  require (Migration_reader.matches value 1506) "Invalid frozen save39_member0_setup_enemy";
  decode_save39_member0_setup_enemy value

let read_save39_member1 value : save39_member1 =
  require (Migration_reader.matches value 1514) "Invalid frozen save39_member1";
  decode_save39_member1 value

let read_save39_member1_progress value : save39_member1_progress =
  require (Migration_reader.matches value 1515) "Invalid frozen save39_member1_progress";
  decode_save39_member1_progress value

let read_history1 value : history1 =
  require (Migration_reader.matches value 1516) "Invalid frozen history1";
  decode_history1 value

let read_literal_1 value : literal_1 =
  require (Migration_reader.matches value 1517) "Invalid frozen literal_1";
  decode_literal_1 value

let read_history1_entries value : history1_entries =
  require (Migration_reader.matches value 1518) "Invalid frozen history1_entries";
  decode_history1_entries value

let read_history2 value : history2 =
  require (Migration_reader.matches value 1519) "Invalid frozen history2";
  decode_history2 value

let read_history2_entries value : history2_entries =
  require (Migration_reader.matches value 1520) "Invalid frozen history2_entries";
  decode_history2_entries value

let read_history2_entries_item value : history2_entries_item =
  require (Migration_reader.matches value 1521) "Invalid frozen history2_entries_item";
  decode_history2_entries_item value

let read_history3 value : history3 =
  require (Migration_reader.matches value 1522) "Invalid frozen history3";
  decode_history3 value

let read_history3_entries value : history3_entries =
  require (Migration_reader.matches value 1523) "Invalid frozen history3_entries";
  decode_history3_entries value

let read_history3_entries_item value : history3_entries_item =
  require (Migration_reader.matches value 1524) "Invalid frozen history3_entries_item";
  decode_history3_entries_item value

let read_history3_entries_item_kind value : history3_entries_item_kind =
  require (Migration_reader.matches value 1525) "Invalid frozen history3_entries_item_kind";
  decode_history3_entries_item_kind value

let read_history4 value : history4 =
  require (Migration_reader.matches value 1526) "Invalid frozen history4";
  decode_history4 value

let read_history4_entries value : history4_entries =
  require (Migration_reader.matches value 1527) "Invalid frozen history4_entries";
  decode_history4_entries value

let read_fragments value : fragments =
  require (Migration_reader.matches value 1528) "Invalid frozen fragments";
  decode_fragments value

let read_fragments_native_state38 value : fragments_native_state38 =
  require (Migration_reader.matches value 1529) "Invalid frozen fragments_native_state38";
  decode_fragments_native_state38 value

let read_fragments_native_state38_factions value : fragments_native_state38_factions =
  require (Migration_reader.matches value 1530) "Invalid frozen fragments_native_state38_factions";
  decode_fragments_native_state38_factions value

let read_fragments_native_state38_factions_item value : fragments_native_state38_factions_item =
  require
    (Migration_reader.matches value 1531)
    "Invalid frozen fragments_native_state38_factions_item";
  decode_fragments_native_state38_factions_item value

let read_fragments_native_state38_factions_item_transfer value :
    fragments_native_state38_factions_item_transfer =
  require
    (Migration_reader.matches value 1532)
    "Invalid frozen fragments_native_state38_factions_item_transfer";
  decode_fragments_native_state38_factions_item_transfer value

let read_fragments_native_state38_factions_item_transfer_member1 value :
    fragments_native_state38_factions_item_transfer_member1 =
  require
    (Migration_reader.matches value 1533)
    "Invalid frozen fragments_native_state38_factions_item_transfer_member1";
  decode_fragments_native_state38_factions_item_transfer_member1 value

let read_fragments_native_state38_tiles_item_fog_member1 value :
    fragments_native_state38_tiles_item_fog_member1 =
  require
    (Migration_reader.matches value 1534)
    "Invalid frozen fragments_native_state38_tiles_item_fog_member1";
  decode_fragments_native_state38_tiles_item_fog_member1 value

let read_fragments_native_state38_tiles_item_fog_member1_condition value :
    fragments_native_state38_tiles_item_fog_member1_condition =
  require
    (Migration_reader.matches value 1535)
    "Invalid frozen fragments_native_state38_tiles_item_fog_member1_condition";
  decode_fragments_native_state38_tiles_item_fog_member1_condition value

let read_fragments_native_state38_tiles_item_fog_member1_condition_member9 value :
    fragments_native_state38_tiles_item_fog_member1_condition_member9 =
  require
    (Migration_reader.matches value 1536)
    "Invalid frozen fragments_native_state38_tiles_item_fog_member1_condition_member9";
  decode_fragments_native_state38_tiles_item_fog_member1_condition_member9 value

let read_fragments_native_state38_tiles_item_fog_member1_condition_member9_conditions value :
    fragments_native_state38_tiles_item_fog_member1_condition_member9_conditions =
  require
    (Migration_reader.matches value 1537)
    "Invalid frozen fragments_native_state38_tiles_item_fog_member1_condition_member9_conditions";
  decode_fragments_native_state38_tiles_item_fog_member1_condition_member9_conditions value

let read_fragments_native_state38_tiles_item_fog_member1_condition_member10 value :
    fragments_native_state38_tiles_item_fog_member1_condition_member10 =
  require
    (Migration_reader.matches value 1538)
    "Invalid frozen fragments_native_state38_tiles_item_fog_member1_condition_member10";
  decode_fragments_native_state38_tiles_item_fog_member1_condition_member10 value

let read_fragments_native_state38_tiles value : fragments_native_state38_tiles =
  require (Migration_reader.matches value 1540) "Invalid frozen fragments_native_state38_tiles";
  decode_fragments_native_state38_tiles value

let read_fragments_native_state38_tiles_item value : fragments_native_state38_tiles_item =
  require (Migration_reader.matches value 1541) "Invalid frozen fragments_native_state38_tiles_item";
  decode_fragments_native_state38_tiles_item value

let read_fragments_native_state38_tiles_item_fog value : fragments_native_state38_tiles_item_fog =
  require
    (Migration_reader.matches value 1542)
    "Invalid frozen fragments_native_state38_tiles_item_fog";
  decode_fragments_native_state38_tiles_item_fog value

let read_fragments_native_state38_tiles_item_light value : fragments_native_state38_tiles_item_light
    =
  require
    (Migration_reader.matches value 1543)
    "Invalid frozen fragments_native_state38_tiles_item_light";
  decode_fragments_native_state38_tiles_item_light value

let read_fragments_native_state38_tiles_item_light_member1 value :
    fragments_native_state38_tiles_item_light_member1 =
  require
    (Migration_reader.matches value 1544)
    "Invalid frozen fragments_native_state38_tiles_item_light_member1";
  decode_fragments_native_state38_tiles_item_light_member1 value

let read_fragments_native_state38_units value : fragments_native_state38_units =
  require (Migration_reader.matches value 1545) "Invalid frozen fragments_native_state38_units";
  decode_fragments_native_state38_units value

let read_fragments_native_state38_units_item value : fragments_native_state38_units_item =
  require (Migration_reader.matches value 1546) "Invalid frozen fragments_native_state38_units_item";
  decode_fragments_native_state38_units_item value

let read_fragments_native_state38_campaign value : fragments_native_state38_campaign =
  require (Migration_reader.matches value 1547) "Invalid frozen fragments_native_state38_campaign";
  decode_fragments_native_state38_campaign value

let read_fragments_native_state38_campaign_member1 value : fragments_native_state38_campaign_member1
    =
  require
    (Migration_reader.matches value 1548)
    "Invalid frozen fragments_native_state38_campaign_member1";
  decode_fragments_native_state38_campaign_member1 value

let read_fragments_native_state38_campaign_member1_scripts value :
    fragments_native_state38_campaign_member1_scripts =
  require
    (Migration_reader.matches value 1549)
    "Invalid frozen fragments_native_state38_campaign_member1_scripts";
  decode_fragments_native_state38_campaign_member1_scripts value

let read_fragments_native_state38_campaign_member1_scripts_member1 value :
    fragments_native_state38_campaign_member1_scripts_member1 =
  require
    (Migration_reader.matches value 1550)
    "Invalid frozen fragments_native_state38_campaign_member1_scripts_member1";
  decode_fragments_native_state38_campaign_member1_scripts_member1 value

let read_fragments_native_state38_campaign_member1_scripts_member1_marks value :
    fragments_native_state38_campaign_member1_scripts_member1_marks =
  require
    (Migration_reader.matches value 1551)
    "Invalid frozen fragments_native_state38_campaign_member1_scripts_member1_marks";
  decode_fragments_native_state38_campaign_member1_scripts_member1_marks value

let read_fragments_native_state38_campaign_member1_scripts_member1_marks_item value :
    fragments_native_state38_campaign_member1_scripts_member1_marks_item =
  require
    (Migration_reader.matches value 1552)
    "Invalid frozen fragments_native_state38_campaign_member1_scripts_member1_marks_item";
  decode_fragments_native_state38_campaign_member1_scripts_member1_marks_item value

let read_fragments_native_state38_campaign_member1_evacuated value :
    fragments_native_state38_campaign_member1_evacuated =
  require
    (Migration_reader.matches value 1555)
    "Invalid frozen fragments_native_state38_campaign_member1_evacuated";
  decode_fragments_native_state38_campaign_member1_evacuated value

let read_fragments_native_state38_campaign_member1_evacuated_member1 value :
    fragments_native_state38_campaign_member1_evacuated_member1 =
  require
    (Migration_reader.matches value 1556)
    "Invalid frozen fragments_native_state38_campaign_member1_evacuated_member1";
  decode_fragments_native_state38_campaign_member1_evacuated_member1 value

let read_fragments_native_state38_campaign_member1_evacuated_member1_item value :
    fragments_native_state38_campaign_member1_evacuated_member1_item =
  require
    (Migration_reader.matches value 1557)
    "Invalid frozen fragments_native_state38_campaign_member1_evacuated_member1_item";
  decode_fragments_native_state38_campaign_member1_evacuated_member1_item value

let read_fragments_delta19 value : fragments_delta19 =
  require (Migration_reader.matches value 1561) "Invalid frozen fragments_delta19";
  decode_fragments_delta19 value

let read_fragments_delta19_item value : fragments_delta19_item =
  require (Migration_reader.matches value 1562) "Invalid frozen fragments_delta19_item";
  decode_fragments_delta19_item value

let read_fragments_light28 value : fragments_light28 =
  require (Migration_reader.matches value 1567) "Invalid frozen fragments_light28";
  decode_fragments_light28 value

let read_native1 value : native1 =
  require (Migration_reader.matches value 1568) "Invalid frozen native1";
  decode_native1 value

let read_literal_rust_wasm value : literal_rust_wasm =
  require (Migration_reader.matches value 1569) "Invalid frozen literal_rust_wasm";
  decode_literal_rust_wasm value

let read_native1_version value : native1_version =
  require (Migration_reader.matches value 1570) "Invalid frozen native1_version";
  decode_native1_version value

let read_native1_saved value : native1_saved =
  require (Migration_reader.matches value 1571) "Invalid frozen native1_saved";
  decode_native1_saved value

let read_javascript1 value : javascript1 =
  require (Migration_reader.matches value 1572) "Invalid frozen javascript1";
  decode_javascript1 value

let read_javascript1_campaign_deployment_format value : javascript1_campaign_deployment_format =
  require
    (Migration_reader.matches value 1573)
    "Invalid frozen javascript1_campaign_deployment_format";
  decode_javascript1_campaign_deployment_format value

let read_literal_named_waves_v1 value : literal_named_waves_v1 =
  require (Migration_reader.matches value 1574) "Invalid frozen literal_named_waves_v1";
  decode_literal_named_waves_v1 value

let read_javascript1_state value : javascript1_state =
  require (Migration_reader.matches value 1575) "Invalid frozen javascript1_state";
  decode_javascript1_state value

let read_javascript1_state_map_revision value : javascript1_state_map_revision =
  require (Migration_reader.matches value 1576) "Invalid frozen javascript1_state_map_revision";
  decode_javascript1_state_map_revision value

let read_javascript1_state_units value : javascript1_state_units =
  require (Migration_reader.matches value 1578) "Invalid frozen javascript1_state_units";
  decode_javascript1_state_units value

let read_javascript1_state_units_item value : javascript1_state_units_item =
  require (Migration_reader.matches value 1579) "Invalid frozen javascript1_state_units_item";
  decode_javascript1_state_units_item value

let read_javascript1_state_units_item_items value : javascript1_state_units_item_items =
  require (Migration_reader.matches value 1582) "Invalid frozen javascript1_state_units_item_items";
  decode_javascript1_state_units_item_items value

let read_javascript1_state_units_item_items_member1 value :
    javascript1_state_units_item_items_member1 =
  require
    (Migration_reader.matches value 1583)
    "Invalid frozen javascript1_state_units_item_items_member1";
  decode_javascript1_state_units_item_items_member1 value

let read_javascript1_state_units_item_items_member1_item value :
    javascript1_state_units_item_items_member1_item =
  require
    (Migration_reader.matches value 1584)
    "Invalid frozen javascript1_state_units_item_items_member1_item";
  decode_javascript1_state_units_item_items_member1_item value

let read_javascript1_state_units_item_items_member1_item_attacks value :
    javascript1_state_units_item_items_member1_item_attacks =
  require
    (Migration_reader.matches value 1589)
    "Invalid frozen javascript1_state_units_item_items_member1_item_attacks";
  decode_javascript1_state_units_item_items_member1_item_attacks value

let read_javascript1_state_units_item_items_member1_item_attacks_item value :
    javascript1_state_units_item_items_member1_item_attacks_item =
  require
    (Migration_reader.matches value 1590)
    "Invalid frozen javascript1_state_units_item_items_member1_item_attacks_item";
  decode_javascript1_state_units_item_items_member1_item_attacks_item value

let read_javascript1_state_units_item_type_ value : javascript1_state_units_item_type_ =
  require (Migration_reader.matches value 1593) "Invalid frozen javascript1_state_units_item_type_";
  decode_javascript1_state_units_item_type_ value

let read_literal_ranger value : literal_ranger =
  require (Migration_reader.matches value 1594) "Invalid frozen literal_ranger";
  decode_literal_ranger value

let read_literal_healer value : literal_healer =
  require (Migration_reader.matches value 1595) "Invalid frozen literal_healer";
  decode_literal_healer value

let read_literal_briarweaver value : literal_briarweaver =
  require (Migration_reader.matches value 1596) "Invalid frozen literal_briarweaver";
  decode_literal_briarweaver value

let read_literal_deatharcher value : literal_deatharcher =
  require (Migration_reader.matches value 1597) "Invalid frozen literal_deatharcher";
  decode_literal_deatharcher value

let read_literal_direwolf value : literal_direwolf =
  require (Migration_reader.matches value 1598) "Invalid frozen literal_direwolf";
  decode_literal_direwolf value

let read_literal_greatboar value : literal_greatboar =
  require (Migration_reader.matches value 1599) "Invalid frozen literal_greatboar";
  decode_literal_greatboar value

let read_literal_wraith value : literal_wraith =
  require (Migration_reader.matches value 1600) "Invalid frozen literal_wraith";
  decode_literal_wraith value

let read_literal_spectre value : literal_spectre =
  require (Migration_reader.matches value 1601) "Invalid frozen literal_spectre";
  decode_literal_spectre value

let read_literal_lancer value : literal_lancer =
  require (Migration_reader.matches value 1602) "Invalid frozen literal_lancer";
  decode_literal_lancer value

let read_literal_mage value : literal_mage =
  require (Migration_reader.matches value 1603) "Invalid frozen literal_mage";
  decode_literal_mage value

let read_literal_marksman value : literal_marksman =
  require (Migration_reader.matches value 1604) "Invalid frozen literal_marksman";
  decode_literal_marksman value

let read_literal_merfolk_maceguard_l2 value : literal_merfolk_maceguard_l2 =
  require (Migration_reader.matches value 1605) "Invalid frozen literal_merfolk_maceguard_l2";
  decode_literal_merfolk_maceguard_l2 value

let read_literal_revenant value : literal_revenant =
  require (Migration_reader.matches value 1606) "Invalid frozen literal_revenant";
  decode_literal_revenant value

let read_literal_skeleton_impact_caster_l2 value : literal_skeleton_impact_caster_l2 =
  require (Migration_reader.matches value 1607) "Invalid frozen literal_skeleton_impact_caster_l2";
  decode_literal_skeleton_impact_caster_l2 value

let read_literal_soldier value : literal_soldier =
  require (Migration_reader.matches value 1608) "Invalid frozen literal_soldier";
  decode_literal_soldier value

let read_literal_tidehunter value : literal_tidehunter =
  require (Migration_reader.matches value 1609) "Invalid frozen literal_tidehunter";
  decode_literal_tidehunter value

let read_literal_tidesage value : literal_tidesage =
  require (Migration_reader.matches value 1610) "Invalid frozen literal_tidesage";
  decode_literal_tidesage value

let read_literal_tridentguard value : literal_tridentguard =
  require (Migration_reader.matches value 1611) "Invalid frozen literal_tridentguard";
  decode_literal_tridentguard value

let read_literal_warden value : literal_warden =
  require (Migration_reader.matches value 1612) "Invalid frozen literal_warden";
  decode_literal_warden value

let read_literal_champion value : literal_champion =
  require (Migration_reader.matches value 1613) "Invalid frozen literal_champion";
  decode_literal_champion value

let read_literal_hammerman value : literal_hammerman =
  require (Migration_reader.matches value 1614) "Invalid frozen literal_hammerman";
  decode_literal_hammerman value

let read_literal_warhammer value : literal_warhammer =
  require (Migration_reader.matches value 1615) "Invalid frozen literal_warhammer";
  decode_literal_warhammer value

let read_literal_sentinel value : literal_sentinel =
  require (Migration_reader.matches value 1616) "Invalid frozen literal_sentinel";
  decode_literal_sentinel value

let read_literal_halberdier value : literal_halberdier =
  require (Migration_reader.matches value 1617) "Invalid frozen literal_halberdier";
  decode_literal_halberdier value

let read_literal_halberd_master value : literal_halberd_master =
  require (Migration_reader.matches value 1618) "Invalid frozen literal_halberd_master";
  decode_literal_halberd_master value

let read_literal_shadowblade value : literal_shadowblade =
  require (Migration_reader.matches value 1619) "Invalid frozen literal_shadowblade";
  decode_literal_shadowblade value

let read_literal_slinger value : literal_slinger =
  require (Migration_reader.matches value 1620) "Invalid frozen literal_slinger";
  decode_literal_slinger value

let read_literal_fireslinger value : literal_fireslinger =
  require (Migration_reader.matches value 1621) "Invalid frozen literal_fireslinger";
  decode_literal_fireslinger value

let read_literal_sharpshooter value : literal_sharpshooter =
  require (Migration_reader.matches value 1622) "Invalid frozen literal_sharpshooter";
  decode_literal_sharpshooter value

let read_literal_crossbowman value : literal_crossbowman =
  require (Migration_reader.matches value 1623) "Invalid frozen literal_crossbowman";
  decode_literal_crossbowman value

let read_literal_arbalester value : literal_arbalester =
  require (Migration_reader.matches value 1624) "Invalid frozen literal_arbalester";
  decode_literal_arbalester value

let read_literal_archmage value : literal_archmage =
  require (Migration_reader.matches value 1625) "Invalid frozen literal_archmage";
  decode_literal_archmage value

let read_literal_high_healer value : literal_high_healer =
  require (Migration_reader.matches value 1626) "Invalid frozen literal_high_healer";
  decode_literal_high_healer value

let read_literal_knight value : literal_knight =
  require (Migration_reader.matches value 1627) "Invalid frozen literal_knight";
  decode_literal_knight value

let read_literal_horse_bowman value : literal_horse_bowman =
  require (Migration_reader.matches value 1628) "Invalid frozen literal_horse_bowman";
  decode_literal_horse_bowman value

let read_literal_horse_marksman value : literal_horse_marksman =
  require (Migration_reader.matches value 1629) "Invalid frozen literal_horse_marksman";
  decode_literal_horse_marksman value

let read_literal_skeleton_dreadguard value : literal_skeleton_dreadguard =
  require (Migration_reader.matches value 1630) "Invalid frozen literal_skeleton_dreadguard";
  decode_literal_skeleton_dreadguard value

let read_literal_skeleton_flanker_l2 value : literal_skeleton_flanker_l2 =
  require (Migration_reader.matches value 1631) "Invalid frozen literal_skeleton_flanker_l2";
  decode_literal_skeleton_flanker_l2 value

let read_literal_skeleton_flanker_l3 value : literal_skeleton_flanker_l3 =
  require (Migration_reader.matches value 1632) "Invalid frozen literal_skeleton_flanker_l3";
  decode_literal_skeleton_flanker_l3 value

let read_literal_skeleton_impact_caster_l3 value : literal_skeleton_impact_caster_l3 =
  require (Migration_reader.matches value 1633) "Invalid frozen literal_skeleton_impact_caster_l3";
  decode_literal_skeleton_impact_caster_l3 value

let read_literal_skeleton_scythe_caster_l2 value : literal_skeleton_scythe_caster_l2 =
  require (Migration_reader.matches value 1634) "Invalid frozen literal_skeleton_scythe_caster_l2";
  decode_literal_skeleton_scythe_caster_l2 value

let read_literal_skeleton_scythe_caster_l3 value : literal_skeleton_scythe_caster_l3 =
  require (Migration_reader.matches value 1635) "Invalid frozen literal_skeleton_scythe_caster_l3";
  decode_literal_skeleton_scythe_caster_l3 value

let read_literal_skeleton_rider_l2 value : literal_skeleton_rider_l2 =
  require (Migration_reader.matches value 1636) "Invalid frozen literal_skeleton_rider_l2";
  decode_literal_skeleton_rider_l2 value

let read_literal_skeleton_rider_l3 value : literal_skeleton_rider_l3 =
  require (Migration_reader.matches value 1637) "Invalid frozen literal_skeleton_rider_l3";
  decode_literal_skeleton_rider_l3 value

let read_literal_skeleton_bow_master value : literal_skeleton_bow_master =
  require (Migration_reader.matches value 1638) "Invalid frozen literal_skeleton_bow_master";
  decode_literal_skeleton_bow_master value

let read_literal_skeleton_axeman_l2 value : literal_skeleton_axeman_l2 =
  require (Migration_reader.matches value 1639) "Invalid frozen literal_skeleton_axeman_l2";
  decode_literal_skeleton_axeman_l2 value

let read_literal_skeleton_axeman_l3 value : literal_skeleton_axeman_l3 =
  require (Migration_reader.matches value 1640) "Invalid frozen literal_skeleton_axeman_l3";
  decode_literal_skeleton_axeman_l3 value

let read_literal_woodland_alpha_wolf value : literal_woodland_alpha_wolf =
  require (Migration_reader.matches value 1641) "Invalid frozen literal_woodland_alpha_wolf";
  decode_literal_woodland_alpha_wolf value

let read_literal_woodland_ancient_boar value : literal_woodland_ancient_boar =
  require (Migration_reader.matches value 1642) "Invalid frozen literal_woodland_ancient_boar";
  decode_literal_woodland_ancient_boar value

let read_literal_woodland_thorn_oracle value : literal_woodland_thorn_oracle =
  require (Migration_reader.matches value 1643) "Invalid frozen literal_woodland_thorn_oracle";
  decode_literal_woodland_thorn_oracle value

let read_literal_woodland_bramble_warden value : literal_woodland_bramble_warden =
  require (Migration_reader.matches value 1644) "Invalid frozen literal_woodland_bramble_warden";
  decode_literal_woodland_bramble_warden value

let read_literal_woodland_thornbinder value : literal_woodland_thornbinder =
  require (Migration_reader.matches value 1645) "Invalid frozen literal_woodland_thornbinder";
  decode_literal_woodland_thornbinder value

let read_literal_fox_spearguard_l2 value : literal_fox_spearguard_l2 =
  require (Migration_reader.matches value 1646) "Invalid frozen literal_fox_spearguard_l2";
  decode_literal_fox_spearguard_l2 value

let read_literal_fox_spearguard_l3 value : literal_fox_spearguard_l3 =
  require (Migration_reader.matches value 1647) "Invalid frozen literal_fox_spearguard_l3";
  decode_literal_fox_spearguard_l3 value

let read_literal_treant_l2 value : literal_treant_l2 =
  require (Migration_reader.matches value 1648) "Invalid frozen literal_treant_l2";
  decode_literal_treant_l2 value

let read_literal_treant_l3 value : literal_treant_l3 =
  require (Migration_reader.matches value 1649) "Invalid frozen literal_treant_l3";
  decode_literal_treant_l3 value

let read_literal_merfolk_harpoon_master value : literal_merfolk_harpoon_master =
  require (Migration_reader.matches value 1650) "Invalid frozen literal_merfolk_harpoon_master";
  decode_literal_merfolk_harpoon_master value

let read_literal_merfolk_netter_l2 value : literal_merfolk_netter_l2 =
  require (Migration_reader.matches value 1651) "Invalid frozen literal_merfolk_netter_l2";
  decode_literal_merfolk_netter_l2 value

let read_literal_merfolk_netter_l3 value : literal_merfolk_netter_l3 =
  require (Migration_reader.matches value 1652) "Invalid frozen literal_merfolk_netter_l3";
  decode_literal_merfolk_netter_l3 value

let read_literal_merfolk_reef_champion value : literal_merfolk_reef_champion =
  require (Migration_reader.matches value 1653) "Invalid frozen literal_merfolk_reef_champion";
  decode_literal_merfolk_reef_champion value

let read_literal_merfolk_trident_l2 value : literal_merfolk_trident_l2 =
  require (Migration_reader.matches value 1654) "Invalid frozen literal_merfolk_trident_l2";
  decode_literal_merfolk_trident_l2 value

let read_literal_merfolk_trident_l3 value : literal_merfolk_trident_l3 =
  require (Migration_reader.matches value 1655) "Invalid frozen literal_merfolk_trident_l3";
  decode_literal_merfolk_trident_l3 value

let read_literal_merfolk_tide_oracle value : literal_merfolk_tide_oracle =
  require (Migration_reader.matches value 1656) "Invalid frozen literal_merfolk_tide_oracle";
  decode_literal_merfolk_tide_oracle value

let read_literal_merfolk_tidecrusher_l2 value : literal_merfolk_tidecrusher_l2 =
  require (Migration_reader.matches value 1657) "Invalid frozen literal_merfolk_tidecrusher_l2";
  decode_literal_merfolk_tidecrusher_l2 value

let read_literal_merfolk_tidecrusher_l3 value : literal_merfolk_tidecrusher_l3 =
  require (Migration_reader.matches value 1658) "Invalid frozen literal_merfolk_tidecrusher_l3";
  decode_literal_merfolk_tidecrusher_l3 value

let read_literal_merfolk_maceguard_l3 value : literal_merfolk_maceguard_l3 =
  require (Migration_reader.matches value 1659) "Invalid frozen literal_merfolk_maceguard_l3";
  decode_literal_merfolk_maceguard_l3 value

let read_literal_dragon value : literal_dragon =
  require (Migration_reader.matches value 1660) "Invalid frozen literal_dragon";
  decode_literal_dragon value

let read_literal_skeletal_hound value : literal_skeletal_hound =
  require (Migration_reader.matches value 1661) "Invalid frozen literal_skeletal_hound";
  decode_literal_skeletal_hound value

let read_literal_captives value : literal_captives =
  require (Migration_reader.matches value 1662) "Invalid frozen literal_captives";
  decode_literal_captives value

let read_literal_zombie_walker value : literal_zombie_walker =
  require (Migration_reader.matches value 1663) "Invalid frozen literal_zombie_walker";
  decode_literal_zombie_walker value

let read_literal_zombie_decay_caster value : literal_zombie_decay_caster =
  require (Migration_reader.matches value 1664) "Invalid frozen literal_zombie_decay_caster";
  decode_literal_zombie_decay_caster value

let read_literal_bear value : literal_bear =
  require (Migration_reader.matches value 1665) "Invalid frozen literal_bear";
  decode_literal_bear value

let read_literal_giant_bat value : literal_giant_bat =
  require (Migration_reader.matches value 1666) "Invalid frozen literal_giant_bat";
  decode_literal_giant_bat value

let read_literal_giant_spider value : literal_giant_spider =
  require (Migration_reader.matches value 1667) "Invalid frozen literal_giant_spider";
  decode_literal_giant_spider value

let read_literal_commander value : literal_commander =
  require (Migration_reader.matches value 1668) "Invalid frozen literal_commander";
  decode_literal_commander value

let read_literal_grovewarden value : literal_grovewarden =
  require (Migration_reader.matches value 1669) "Invalid frozen literal_grovewarden";
  decode_literal_grovewarden value

let read_literal_lich value : literal_lich =
  require (Migration_reader.matches value 1670) "Invalid frozen literal_lich";
  decode_literal_lich value

let read_literal_marshal value : literal_marshal =
  require (Migration_reader.matches value 1671) "Invalid frozen literal_marshal";
  decode_literal_marshal value

let read_literal_necromancer value : literal_necromancer =
  require (Migration_reader.matches value 1672) "Invalid frozen literal_necromancer";
  decode_literal_necromancer value

let read_literal_tidekeeper value : literal_tidekeeper =
  require (Migration_reader.matches value 1673) "Invalid frozen literal_tidekeeper";
  decode_literal_tidekeeper value

let read_literal_tideseer value : literal_tideseer =
  require (Migration_reader.matches value 1674) "Invalid frozen literal_tideseer";
  decode_literal_tideseer value

let read_literal_wildkeeper value : literal_wildkeeper =
  require (Migration_reader.matches value 1675) "Invalid frozen literal_wildkeeper";
  decode_literal_wildkeeper value

let read_literal_grand_marshal value : literal_grand_marshal =
  require (Migration_reader.matches value 1676) "Invalid frozen literal_grand_marshal";
  decode_literal_grand_marshal value

let read_literal_lich_commander_l3 value : literal_lich_commander_l3 =
  require (Migration_reader.matches value 1677) "Invalid frozen literal_lich_commander_l3";
  decode_literal_lich_commander_l3 value

let read_literal_woodland_grove_oracle value : literal_woodland_grove_oracle =
  require (Migration_reader.matches value 1678) "Invalid frozen literal_woodland_grove_oracle";
  decode_literal_woodland_grove_oracle value

let read_literal_merfolk_tide_sovereign value : literal_merfolk_tide_sovereign =
  require (Migration_reader.matches value 1679) "Invalid frozen literal_merfolk_tide_sovereign";
  decode_literal_merfolk_tide_sovereign value

let read_literal_adept value : literal_adept =
  require (Migration_reader.matches value 1680) "Invalid frozen literal_adept";
  decode_literal_adept value

let read_literal_archer value : literal_archer =
  require (Migration_reader.matches value 1681) "Invalid frozen literal_archer";
  decode_literal_archer value

let read_literal_boar value : literal_boar =
  require (Migration_reader.matches value 1682) "Invalid frozen literal_boar";
  decode_literal_boar value

let read_literal_bonearcher value : literal_bonearcher =
  require (Migration_reader.matches value 1683) "Invalid frozen literal_bonearcher";
  decode_literal_bonearcher value

let read_literal_ghost value : literal_ghost =
  require (Migration_reader.matches value 1684) "Invalid frozen literal_ghost";
  decode_literal_ghost value

let read_literal_harpooner value : literal_harpooner =
  require (Migration_reader.matches value 1685) "Invalid frozen literal_harpooner";
  decode_literal_harpooner value

let read_literal_merfolk_maceguard value : literal_merfolk_maceguard =
  require (Migration_reader.matches value 1686) "Invalid frozen literal_merfolk_maceguard";
  decode_literal_merfolk_maceguard value

let read_literal_reefguard value : literal_reefguard =
  require (Migration_reader.matches value 1687) "Invalid frozen literal_reefguard";
  decode_literal_reefguard value

let read_literal_rider value : literal_rider =
  require (Migration_reader.matches value 1688) "Invalid frozen literal_rider";
  decode_literal_rider value

let read_literal_scout value : literal_scout =
  require (Migration_reader.matches value 1689) "Invalid frozen literal_scout";
  decode_literal_scout value

let read_literal_skeleton_impact_caster value : literal_skeleton_impact_caster =
  require (Migration_reader.matches value 1690) "Invalid frozen literal_skeleton_impact_caster";
  decode_literal_skeleton_impact_caster value

let read_literal_skeleton value : literal_skeleton =
  require (Migration_reader.matches value 1691) "Invalid frozen literal_skeleton";
  decode_literal_skeleton value

let read_literal_spearman value : literal_spearman =
  require (Migration_reader.matches value 1692) "Invalid frozen literal_spearman";
  decode_literal_spearman value

let read_literal_swordsman value : literal_swordsman =
  require (Migration_reader.matches value 1693) "Invalid frozen literal_swordsman";
  decode_literal_swordsman value

let read_literal_thorncaller value : literal_thorncaller =
  require (Migration_reader.matches value 1694) "Invalid frozen literal_thorncaller";
  decode_literal_thorncaller value

let read_literal_wavemender value : literal_wavemender =
  require (Migration_reader.matches value 1695) "Invalid frozen literal_wavemender";
  decode_literal_wavemender value

let read_literal_wolf value : literal_wolf =
  require (Migration_reader.matches value 1696) "Invalid frozen literal_wolf";
  decode_literal_wolf value

let read_literal_horse_archer value : literal_horse_archer =
  require (Migration_reader.matches value 1697) "Invalid frozen literal_horse_archer";
  decode_literal_horse_archer value

let read_literal_skeleton_rider value : literal_skeleton_rider =
  require (Migration_reader.matches value 1698) "Invalid frozen literal_skeleton_rider";
  decode_literal_skeleton_rider value

let read_literal_skeleton_axeman value : literal_skeleton_axeman =
  require (Migration_reader.matches value 1699) "Invalid frozen literal_skeleton_axeman";
  decode_literal_skeleton_axeman value

let read_literal_fox_spearguard value : literal_fox_spearguard =
  require (Migration_reader.matches value 1700) "Invalid frozen literal_fox_spearguard";
  decode_literal_fox_spearguard value

let read_literal_treant value : literal_treant =
  require (Migration_reader.matches value 1701) "Invalid frozen literal_treant";
  decode_literal_treant value

let read_javascript1_state_villages value : javascript1_state_villages =
  require (Migration_reader.matches value 1702) "Invalid frozen javascript1_state_villages";
  decode_javascript1_state_villages value

let read_javascript1_state_villages_item value : javascript1_state_villages_item =
  require (Migration_reader.matches value 1703) "Invalid frozen javascript1_state_villages_item";
  decode_javascript1_state_villages_item value

let read_javascript1_state_sites value : javascript1_state_sites =
  require (Migration_reader.matches value 1704) "Invalid frozen javascript1_state_sites";
  decode_javascript1_state_sites value

let read_javascript1_state_campaign value : javascript1_state_campaign =
  require (Migration_reader.matches value 1705) "Invalid frozen javascript1_state_campaign";
  decode_javascript1_state_campaign value

let read_javascript1_state_campaign_member1 value : javascript1_state_campaign_member1 =
  require (Migration_reader.matches value 1706) "Invalid frozen javascript1_state_campaign_member1";
  decode_javascript1_state_campaign_member1 value

let read_javascript1_state_campaign_member1_id value : javascript1_state_campaign_member1_id =
  require
    (Migration_reader.matches value 1707)
    "Invalid frozen javascript1_state_campaign_member1_id";
  decode_javascript1_state_campaign_member1_id value

let read_literal_lantern_road value : literal_lantern_road =
  require (Migration_reader.matches value 1708) "Invalid frozen literal_lantern_road";
  decode_literal_lantern_road value

let read_literal_green_accord value : literal_green_accord =
  require (Migration_reader.matches value 1709) "Invalid frozen literal_green_accord";
  decode_literal_green_accord value

let read_literal_ashen_oath value : literal_ashen_oath =
  require (Migration_reader.matches value 1710) "Invalid frozen literal_ashen_oath";
  decode_literal_ashen_oath value

let read_literal_borrowed_tide value : literal_borrowed_tide =
  require (Migration_reader.matches value 1711) "Invalid frozen literal_borrowed_tide";
  decode_literal_borrowed_tide value

let read_literal_mirror_night value : literal_mirror_night =
  require (Migration_reader.matches value 1712) "Invalid frozen literal_mirror_night";
  decode_literal_mirror_night value

let read_literal_bell_night value : literal_bell_night =
  require (Migration_reader.matches value 1713) "Invalid frozen literal_bell_night";
  decode_literal_bell_night value

let read_javascript1_state_campaign_member1_recall value : javascript1_state_campaign_member1_recall
    =
  require
    (Migration_reader.matches value 1714)
    "Invalid frozen javascript1_state_campaign_member1_recall";
  decode_javascript1_state_campaign_member1_recall value

let read_javascript1_state_campaign_member1_veterans value :
    javascript1_state_campaign_member1_veterans =
  require
    (Migration_reader.matches value 1715)
    "Invalid frozen javascript1_state_campaign_member1_veterans";
  decode_javascript1_state_campaign_member1_veterans value

let read_javascript1_state_campaign_member1_veterans_member1 value :
    javascript1_state_campaign_member1_veterans_member1 =
  require
    (Migration_reader.matches value 1716)
    "Invalid frozen javascript1_state_campaign_member1_veterans_member1";
  decode_javascript1_state_campaign_member1_veterans_member1 value

let read_javascript1_state_campaign_member1_veterans_member1_item value :
    javascript1_state_campaign_member1_veterans_member1_item =
  require
    (Migration_reader.matches value 1717)
    "Invalid frozen javascript1_state_campaign_member1_veterans_member1_item";
  decode_javascript1_state_campaign_member1_veterans_member1_item value

let read_javascript1_state_campaign_member1_dialogue value :
    javascript1_state_campaign_member1_dialogue =
  require
    (Migration_reader.matches value 1718)
    "Invalid frozen javascript1_state_campaign_member1_dialogue";
  decode_javascript1_state_campaign_member1_dialogue value

let read_javascript1_state_campaign_member1_dialogue_item value :
    javascript1_state_campaign_member1_dialogue_item =
  require
    (Migration_reader.matches value 1719)
    "Invalid frozen javascript1_state_campaign_member1_dialogue_item";
  decode_javascript1_state_campaign_member1_dialogue_item value

let read_javascript1_state_campaign_member1_dialogue_item_portrait value :
    javascript1_state_campaign_member1_dialogue_item_portrait =
  require
    (Migration_reader.matches value 1720)
    "Invalid frozen javascript1_state_campaign_member1_dialogue_item_portrait";
  decode_javascript1_state_campaign_member1_dialogue_item_portrait value

let read_literal_worker value : literal_worker =
  require (Migration_reader.matches value 1721) "Invalid frozen literal_worker";
  decode_literal_worker value

module V2 = struct
  let of_json value : Historical_types.V2.t =
    let _ = Migration_reader.read value "Save2" in
    decode_save2 value

  let to_json (value : Historical_types.V2.t) = encode_save2 value
end

module V3 = struct
  let of_json value : Historical_types.V3.t =
    let _ = Migration_reader.read value "Save3" in
    decode_save3 value

  let to_json (value : Historical_types.V3.t) = encode_save3 value
end

module V4 = struct
  let of_json value : Historical_types.V4.t =
    let _ = Migration_reader.read value "Save4" in
    decode_save4 value

  let to_json (value : Historical_types.V4.t) = encode_save4 value
end

module V5 = struct
  let of_json value : Historical_types.V5.t =
    let _ = Migration_reader.read value "Save5" in
    decode_save5 value

  let to_json (value : Historical_types.V5.t) = encode_save5 value
end

module V6 = struct
  let of_json value : Historical_types.V6.t =
    let _ = Migration_reader.read value "Save6" in
    decode_save6 value

  let to_json (value : Historical_types.V6.t) = encode_save6 value
end

module V7 = struct
  let of_json value : Historical_types.V7.t =
    let _ = Migration_reader.read value "Save7" in
    decode_save7 value

  let to_json (value : Historical_types.V7.t) = encode_save7 value
end

module V8 = struct
  let of_json value : Historical_types.V8.t =
    let _ = Migration_reader.read value "Save8" in
    decode_save8 value

  let to_json (value : Historical_types.V8.t) = encode_save8 value
end

module V9 = struct
  let of_json value : Historical_types.V9.t =
    let _ = Migration_reader.read value "Save9" in
    decode_save9 value

  let to_json (value : Historical_types.V9.t) = encode_save9 value
end

module V10 = struct
  let of_json value : Historical_types.V10.t =
    let _ = Migration_reader.read value "Save10" in
    decode_save10 value

  let to_json (value : Historical_types.V10.t) = encode_save10 value
end

module V11 = struct
  let of_json value : Historical_types.V11.t =
    let _ = Migration_reader.read value "Save11" in
    decode_save11 value

  let to_json (value : Historical_types.V11.t) = encode_save11 value
end

module V12 = struct
  let of_json value : Historical_types.V12.t =
    let _ = Migration_reader.read value "Save12" in
    decode_save12 value

  let to_json (value : Historical_types.V12.t) = encode_save12 value
end

module V13 = struct
  let of_json value : Historical_types.V13.t =
    let _ = Migration_reader.read value "Save13" in
    decode_save13 value

  let to_json (value : Historical_types.V13.t) = encode_save13 value
end

module V14 = struct
  let of_json value : Historical_types.V14.t =
    let _ = Migration_reader.read value "Save14" in
    decode_save14 value

  let to_json (value : Historical_types.V14.t) = encode_save14 value
end

module V15 = struct
  let of_json value : Historical_types.V15.t =
    let _ = Migration_reader.read value "Save15" in
    decode_save15 value

  let to_json (value : Historical_types.V15.t) = encode_save15 value
end

module V16 = struct
  let of_json value : Historical_types.V16.t =
    let _ = Migration_reader.read value "Save16" in
    decode_save16 value

  let to_json (value : Historical_types.V16.t) = encode_save16 value
end

module V17 = struct
  let of_json value : Historical_types.V17.t =
    let _ = Migration_reader.read value "Save17" in
    decode_save17 value

  let to_json (value : Historical_types.V17.t) = encode_save17 value
end

module V18 = struct
  let of_json value : Historical_types.V18.t =
    let _ = Migration_reader.read value "Save18" in
    decode_save18 value

  let to_json (value : Historical_types.V18.t) = encode_save18 value
end

module V19 = struct
  let of_json value : Historical_types.V19.t =
    let _ = Migration_reader.read value "Save19" in
    decode_save19 value

  let to_json (value : Historical_types.V19.t) = encode_save19 value
end

module V20 = struct
  let of_json value : Historical_types.V20.t =
    let _ = Migration_reader.read value "Save20" in
    decode_save20 value

  let to_json (value : Historical_types.V20.t) = encode_save20 value
end

module V21 = struct
  let of_json value : Historical_types.V21.t =
    let _ = Migration_reader.read value "Save21" in
    decode_save21 value

  let to_json (value : Historical_types.V21.t) = encode_save21 value
end

module V22 = struct
  let of_json value : Historical_types.V22.t =
    let _ = Migration_reader.read value "Save22" in
    decode_save22 value

  let to_json (value : Historical_types.V22.t) = encode_save22 value
end

module V23 = struct
  let of_json value : Historical_types.V23.t =
    let _ = Migration_reader.read value "Save23" in
    decode_save23 value

  let to_json (value : Historical_types.V23.t) = encode_save23 value
end

module V24 = struct
  let of_json value : Historical_types.V24.t =
    let _ = Migration_reader.read value "Save24" in
    decode_save24 value

  let to_json (value : Historical_types.V24.t) = encode_save24 value
end

module V25 = struct
  let of_json value : Historical_types.V25.t =
    let _ = Migration_reader.read value "Save25" in
    decode_save25 value

  let to_json (value : Historical_types.V25.t) = encode_save25 value
end

module V26 = struct
  let of_json value : Historical_types.V26.t =
    let _ = Migration_reader.read value "Save26" in
    decode_save26 value

  let to_json (value : Historical_types.V26.t) = encode_save26 value
end

module V27 = struct
  let of_json value : Historical_types.V27.t =
    let _ = Migration_reader.read value "Save27" in
    decode_save27 value

  let to_json (value : Historical_types.V27.t) = encode_save27 value
end

module V28 = struct
  let of_json value : Historical_types.V28.t =
    let _ = Migration_reader.read value "Save28" in
    decode_save28 value

  let to_json (value : Historical_types.V28.t) = encode_save28 value
end

module V29 = struct
  let of_json value : Historical_types.V29.t =
    let _ = Migration_reader.read value "Save29" in
    decode_save29 value

  let to_json (value : Historical_types.V29.t) = encode_save29 value
end

module V30 = struct
  let of_json value : Historical_types.V30.t =
    let _ = Migration_reader.read value "Save30" in
    decode_save30 value

  let to_json (value : Historical_types.V30.t) = encode_save30 value
end

module V31 = struct
  let of_json value : Historical_types.V31.t =
    let _ = Migration_reader.read value "Save31" in
    decode_save31 value

  let to_json (value : Historical_types.V31.t) = encode_save31 value
end

module V32 = struct
  let of_json value : Historical_types.V32.t =
    let _ = Migration_reader.read value "Save32" in
    decode_save32 value

  let to_json (value : Historical_types.V32.t) = encode_save32 value
end

module V33 = struct
  let of_json value : Historical_types.V33.t =
    let _ = Migration_reader.read value "Save33" in
    decode_save33 value

  let to_json (value : Historical_types.V33.t) = encode_save33 value
end

module V34 = struct
  let of_json value : Historical_types.V34.t =
    let _ = Migration_reader.read value "Save34" in
    decode_save34 value

  let to_json (value : Historical_types.V34.t) = encode_save34 value
end

module V35 = struct
  let of_json value : Historical_types.V35.t =
    let _ = Migration_reader.read value "Save35" in
    decode_save35 value

  let to_json (value : Historical_types.V35.t) = encode_save35 value
end

module V36 = struct
  let of_json value : Historical_types.V36.t =
    let _ = Migration_reader.read value "Save36" in
    decode_save36 value

  let to_json (value : Historical_types.V36.t) = encode_save36 value
end

module V37 = struct
  let of_json value : Historical_types.V37.t =
    let _ = Migration_reader.read value "Save37" in
    decode_save37 value

  let to_json (value : Historical_types.V37.t) = encode_save37 value
end

module V38 = struct
  let of_json value : Historical_types.V38.t =
    let _ = Migration_reader.read value "Save38" in
    decode_save38 value

  let to_json (value : Historical_types.V38.t) = encode_save38 value
end

module V39 = struct
  let of_json value : Historical_types.V39.t =
    let _ = Migration_reader.read value "Save39" in
    decode_save39 value

  let to_json (value : Historical_types.V39.t) = encode_save39 value
end

module History1 = struct
  let of_json value : Historical_types.History1.t =
    let _ = Migration_reader.read value "History1" in
    decode_history1 value

  let to_json (value : Historical_types.History1.t) = encode_history1 value
end

module History2 = struct
  let of_json value : Historical_types.History2.t =
    let _ = Migration_reader.read value "History2" in
    decode_history2 value

  let to_json (value : Historical_types.History2.t) = encode_history2 value
end

module History3 = struct
  let of_json value : Historical_types.History3.t =
    let _ = Migration_reader.read value "History3" in
    decode_history3 value

  let to_json (value : Historical_types.History3.t) = encode_history3 value
end

module History4 = struct
  let of_json value : Historical_types.History4.t =
    let _ = Migration_reader.read value "History4" in
    decode_history4 value

  let to_json (value : Historical_types.History4.t) = encode_history4 value
end

module Fragments_document = struct
  let of_json value : Historical_types.Fragments.t =
    let _ = Migration_reader.read value "Fragments" in
    decode_fragments value

  let to_json (value : Historical_types.Fragments.t) = encode_fragments value
end

module Native1 = struct
  let of_json value : Historical_types.Native1.t =
    let _ = Migration_reader.read value "Native1" in
    decode_native1 value

  let to_json (value : Historical_types.Native1.t) = encode_native1 value
end

module Javascript1 = struct
  let of_json value : Historical_types.Javascript1.t =
    let _ = Migration_reader.read value "Javascript1" in
    decode_javascript1 value

  let to_json (value : Historical_types.Javascript1.t) = encode_javascript1 value
end

module Fragment = struct
  let native_state38 value =
    let _ = Migration_reader.fragment value "nativeState38" in
    decode_fragments_native_state38 value

  let factions7 value =
    let _ = Migration_reader.fragment value "factions7" in
    decode_fragments_factions7 value

  let factions11 value =
    let _ = Migration_reader.fragment value "factions11" in
    decode_fragments_factions7 value

  let factions21 value =
    let _ = Migration_reader.fragment value "factions21" in
    decode_fragments_factions7 value

  let factions24 value =
    let _ = Migration_reader.fragment value "factions24" in
    decode_fragments_factions7 value

  let tiles11 value =
    let _ = Migration_reader.fragment value "tiles11" in
    decode_fragments_tiles11 value

  let rule11 value =
    let _ = Migration_reader.fragment value "rule11" in
    decode_fragments_rule11 value

  let delta19 value =
    let _ = Migration_reader.fragment value "delta19" in
    decode_fragments_delta19 value

  let delta20 value =
    let _ = Migration_reader.fragment value "delta20" in
    decode_fragments_delta19 value

  let delta21 value =
    let _ = Migration_reader.fragment value "delta21" in
    decode_fragments_delta19 value

  let light23 value =
    let _ = Migration_reader.fragment value "light23" in
    decode_fragments_light23 value

  let light28 value =
    let _ = Migration_reader.fragment value "light28" in
    decode_fragments_light28 value
end

module Fragments = struct
  let native_state38_of_json = Fragment.native_state38
  let native_state38_to_json = encode_fragments_native_state38
  let factions7_of_json = Fragment.factions7
  let factions7_to_json = encode_fragments_factions7
  let factions11_of_json = Fragment.factions11
  let factions11_to_json = encode_fragments_factions7
  let factions21_of_json = Fragment.factions21
  let factions21_to_json = encode_fragments_factions7
  let factions24_of_json = Fragment.factions24
  let factions24_to_json = encode_fragments_factions7
  let tiles11_of_json = Fragment.tiles11
  let tiles11_to_json = encode_fragments_tiles11
  let rule11_of_json = Fragment.rule11
  let rule11_to_json = encode_fragments_rule11
  let delta19_of_json = Fragment.delta19
  let delta19_to_json = encode_fragments_delta19
  let delta20_of_json = Fragment.delta20
  let delta20_to_json = encode_fragments_delta19
  let delta21_of_json = Fragment.delta21
  let delta21_to_json = encode_fragments_delta19
  let light23_of_json = Fragment.light23
  let light23_to_json = encode_fragments_light23
  let light28_of_json = Fragment.light28
  let light28_to_json = encode_fragments_light28
end
