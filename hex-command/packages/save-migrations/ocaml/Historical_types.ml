(* Generated from frozen src/formats/readers.json. Do not edit. *)
[@@@warning "-30"]

module String_map = struct
  type 'a t = (string * 'a) list

  let empty = []
  let is_empty value = value = []
  let mem key value = List.mem_assoc key value
  let find key value = List.assoc key value
  let find_opt key value = List.assoc_opt key value
  let bindings value = value
  let cardinal = List.length
  let singleton key value = [ (key, value) ]
  let remove key value = List.filter (fun (k, _) -> k <> key) value

  let add key item value =
    if mem key value then List.map (fun (k, v) -> if k = key then (k, item) else (k, v)) value
    else value @ [ (key, item) ]

  let update key f value =
    match f (find_opt key value) with None -> remove key value | Some item -> add key item value

  let iter f value = List.iter (fun (k, v) -> f k v) value
  let fold f value init = List.fold_left (fun acc (k, v) -> f k v acc) init value
  let map f value = List.map (fun (k, v) -> (k, f v)) value
  let mapi f value = List.map (fun (k, v) -> (k, f k v)) value
  let filter f value = List.filter (fun (k, v) -> f k v) value

  let filter_map f value =
    List.filter_map (fun (k, v) -> Option.map (fun v -> (k, v)) (f k v)) value

  let exists f value = List.exists (fun (k, v) -> f k v) value
  let for_all f value = List.for_all (fun (k, v) -> f k v) value
  let of_list value = List.fold_left (fun map (k, v) -> add k v map) empty value
  let to_list value = value
end

type empty_object = Empty_object

type wire = {
  order : string list;
  extra : (string * Js.Json.t) list;
  undefined_fields : string list;
}

let empty_wire = { order = []; extra = []; undefined_fields = [] }
let wire_has value key = List.mem key value.order

let wire_remove value key =
  {
    order = List.filter (( <> ) key) value.order;
    extra = List.filter (fun (k, _) -> k <> key) value.extra;
    undefined_fields = List.filter (( <> ) key) value.undefined_fields;
  }

let remove_fields wire keys = List.fold_left wire_remove wire keys

let merge_wire left right =
  {
    order = left.order @ List.filter (fun k -> not (List.mem k left.order)) right.order;
    extra = List.filter (fun (k, _) -> not (List.mem_assoc k right.extra)) left.extra @ right.extra;
    undefined_fields =
      List.filter (fun k -> not (List.mem k right.order)) left.undefined_fields
      @ right.undefined_fields;
  }

type save2 = {
  wire : wire;
  version : literal_2;
  checkpoint : save2_checkpoint;
  pending_enemy : json_boolean;
  planner : json_unknown option;
  setup : save2_setup_member1 option;
  progress : save2_progress_member1 option;
  undo : json_unknown option;
}

and literal_2 = [ `V2 ]
and save2_checkpoint = { wire : wire; seed : json_number; state : save2_checkpoint_state }
and json_number = float

and save2_checkpoint_state = {
  wire : wire;
  round : json_number;
  turn : save35_setup_first;
  winner : fragments_native_state38_winner;
  armies : fragments_native_state38_armies_member1 option;
  condition_memory : fragments_native_state38_log option;
  map_id : json_string;
  map_revision : json_number;
  cols : json_number;
  rows : json_number;
  next_id : json_number;
  log : fragments_native_state38_log;
  last_combat : fragments_native_state38_last_combat option;
  units : save2_checkpoint_state_units;
  tiles : save2_checkpoint_state_tiles;
  campaign : save2_checkpoint_state_campaign_member1 option;
  gold : javascript1_state_gold;
  factions : save2_checkpoint_state_factions_member1 option;
  ground_items : fragments_native_state38_ground_items option;
  combat_rng : json_number option;
}

and save35_setup_first = [ `Player | `Enemy ]
and literal_player = [ `Player ]
and literal_enemy = [ `Enemy ]
and fragments_native_state38_winner = [ `Null | `Player | `Enemy ]
and json_null = [ `Null ]

and fragments_native_state38_armies =
  [ `Undefined | `Tuple of fragments_native_state38_armies_member1 ]

and json_undefined = [ `Undefined ]
and fragments_native_state38_armies_member1 = json_string * json_string
and json_string = string

and fragments_native_state38_condition_memory =
  [ `Undefined | `Array of fragments_native_state38_log ]

and fragments_native_state38_log = json_string array

and fragments_native_state38_last_combat =
  [ `Undefined | `Null | `Object3 of fragments_native_state38_last_combat_member2 ]

and fragments_native_state38_last_combat_member2 = {
  wire : wire;
  attacker_id : json_string;
  attack_id : json_string;
  sound : json_string;
}

and save2_checkpoint_state_units = save2_checkpoint_state_units_item array

and save2_checkpoint_state_units_item = {
  wire : wire;
  tile : json_string;
  id : json_string;
  side : save35_setup_first;
  role : fragments_native_state38_units_item_role option;
  faction : json_string option;
  items : fragments_native_state38_units_item_items_member1 option;
  ai_group : json_string option;
  type_ : json_string;
  hp : json_number;
  moved : json_boolean;
  attacked : json_boolean;
  retreating : fragments_rule11_remember option;
  rebuilding : fragments_rule11_remember option;
  name : json_string option;
  character : json_string option;
  experience : json_number option;
}

and fragments_native_state38_units_item_role = [ `Undefined | `Leader | `Courier | `Captive ]
and literal_leader = [ `Leader ]
and literal_courier = [ `Courier ]
and literal_captive = [ `Captive ]
and javascript1_state_map_id = [ `Undefined | `String of json_string ]

and fragments_native_state38_units_item_items =
  [ `Undefined | `Array of fragments_native_state38_units_item_items_member1 ]

and fragments_native_state38_units_item_items_member1 =
  fragments_native_state38_ground_items_item_item array

and fragments_native_state38_ground_items_item_item = {
  wire : wire;
  armor : fragments_native_state38_ground_items_item_item_armor_member1 option;
  id : json_string;
  name : json_string;
  description : json_string;
  attacks : fragments_native_state38_ground_items_item_item_attacks;
  healing : fragments_native_state38_ground_items_item_item_healing_member1 option;
}

and fragments_native_state38_ground_items_item_item_armor =
  [ `Undefined | `Object2 of fragments_native_state38_ground_items_item_item_armor_member1 ]

and fragments_native_state38_ground_items_item_item_armor_member1 = {
  wire : wire;
  blade : json_number;
  pierce : json_number;
  impact : json_number;
}

and fragments_native_state38_ground_items_item_item_attacks =
  fragments_native_state38_ground_items_item_item_attacks_item array

and fragments_native_state38_ground_items_item_item_attacks_item = {
  wire : wire;
  id : json_string;
  name : json_string;
  damage : json_number;
  spread : json_number option;
  count : json_number option;
  damage_type : fragments_native_state38_ground_items_item_item_attacks_item_damage_type;
  category : fragments_native_state38_ground_items_item_item_attacks_item_category;
  range : json_number;
  sound : json_string;
  drain : json_number option;
}

and fragments_light23_dx = [ `Undefined | `Number of json_number ]

and fragments_native_state38_ground_items_item_item_attacks_item_damage_type =
  [ `Blade | `Pierce | `Impact ]

and literal_blade = [ `Blade ]
and literal_pierce = [ `Pierce ]
and literal_impact = [ `Impact ]
and fragments_native_state38_ground_items_item_item_attacks_item_category = [ `Melee | `Ranged ]
and literal_melee = [ `Melee ]
and literal_ranged = [ `Ranged ]

and fragments_native_state38_ground_items_item_item_healing =
  [ `Undefined | `Object2 of fragments_native_state38_ground_items_item_item_healing_member1 ]

and fragments_native_state38_ground_items_item_item_healing_member1 = {
  wire : wire;
  amount : json_number;
  radius : json_number;
}

and json_boolean = bool
and fragments_rule11_remember = [ `Undefined | `False | `True ]
and literal_false = [ `False ]
and literal_true = [ `True ]
and save2_checkpoint_state_tiles = save2_checkpoint_state_tiles_item array

and save2_checkpoint_state_tiles_item = {
  wire : wire;
  owner : fragments_tiles11_item_owner;
  id : json_string;
  location : json_string option;
  col : json_number;
  row : json_number;
  terrain : json_string;
  connections : fragments_tiles11_item_connections_member1 option;
  dead_ends : fragments_tiles11_item_connections_member1 option;
  walls : fragments_tiles11_item_connections_member1 option;
  floor : json_string option;
  fog : save2_checkpoint_state_tiles_item_fog_member1 option;
  light : save2_checkpoint_state_tiles_item_light_member1 option;
  wall_corners : fragments_tiles11_item_wall_corners_member1 option;
}

and fragments_tiles11_item_owner = [ `Null | `String of json_string ]

and fragments_tiles11_item_connections =
  [ `Undefined | `Array of fragments_tiles11_item_connections_member1 ]

and fragments_tiles11_item_connections_member1 =
  fragments_tiles11_item_connections_member1_item array

and fragments_tiles11_item_connections_member1_item = [ `E | `Se | `Sw | `W | `Nw | `Ne ]
and literal_e = [ `E ]
and literal_se = [ `Se ]
and literal_sw = [ `Sw ]
and literal_w = [ `W ]
and literal_nw = [ `Nw ]
and literal_ne = [ `Ne ]

and save2_checkpoint_state_tiles_item_fog =
  [ `Undefined | `Object2 of save2_checkpoint_state_tiles_item_fog_member1 ]

and save2_checkpoint_state_tiles_item_fog_member1 = {
  wire : wire;
  condition : save2_checkpoint_state_tiles_item_fog_member1_condition;
  remember : fragments_rule11_remember option;
}

and save2_checkpoint_state_tiles_item_fog_member1_condition =
  [ `Owner of fragments_rule11_condition_member0
  | `Visible_units of fragments_rule11_condition_member1
  | `Choice of fragments_rule11_condition_member2
  | `Flag of fragments_rule11_condition_member3
  | `Event of fragments_rule11_condition_member4
  | `Round of fragments_rule11_condition_member5
  | `Object7 of save2_checkpoint_state_tiles_item_fog_member1_condition_member6
  | `Proximity of fragments_rule11_condition_member6
  | `Has_units of fragments_rule11_condition_member7
  | `No_units of fragments_rule11_condition_member8 ]

and fragments_rule11_condition_member0 = {
  wire : wire;
  kind : literal_owner;
  tile : json_string option;
  side : json_string;
}

and literal_owner = [ `Owner ]

and fragments_rule11_condition_member1 = {
  wire : wire;
  kind : literal_visible_units;
  side : fragments_rule11_condition_member1_side;
  unit_types : fragments_native_state38_log option;
}

and literal_visible_units = [ `Visible_units ]
and fragments_rule11_condition_member1_side = [ `Player | `Enemy | `Neutral | `Any ]
and literal_neutral = [ `Neutral ]
and literal_any = [ `Any ]

and fragments_rule11_condition_member2 = {
  wire : wire;
  kind : literal_choice;
  choice : json_string;
}

and literal_choice = [ `Choice ]
and fragments_rule11_condition_member3 = { wire : wire; kind : literal_flag; flag : json_string }
and literal_flag = [ `Flag ]
and fragments_rule11_condition_member4 = { wire : wire; kind : literal_event; event : json_string }
and literal_event = [ `Event ]
and fragments_rule11_condition_member5 = { wire : wire; kind : literal_round; round : json_number }
and literal_round = [ `Round ]

and save2_checkpoint_state_tiles_item_fog_member1_condition_member6 = {
  wire : wire;
  kind : fragments_rule11_condition_member9_kind;
  conditions : save2_checkpoint_state_tiles_item_fog_member1_condition_member6_conditions;
}

and fragments_rule11_condition_member9_kind = [ `Any | `All ]
and literal_all = [ `All ]

and save2_checkpoint_state_tiles_item_fog_member1_condition_member6_conditions =
  save2_checkpoint_state_tiles_item_fog_member1_condition array

and fragments_rule11_condition_member6 = {
  wire : wire;
  kind : literal_proximity;
  unit_types : fragments_native_state38_log option;
  tile : json_string option;
  radius : json_number;
  side : fragments_rule11_condition_member1_side;
  unit_type : json_string option;
  character : json_string option;
  name : json_string option;
}

and literal_proximity = [ `Proximity ]

and fragments_rule11_condition_member7 = {
  wire : wire;
  kind : literal_has_units;
  ai_group : json_string option;
  side : fragments_rule11_condition_member1_side;
  unit_type : json_string option;
  character : json_string option;
  name : json_string option;
}

and literal_has_units = [ `Has_units ]

and fragments_rule11_condition_member8 = {
  wire : wire;
  kind : literal_no_units;
  side : fragments_rule11_condition_member1_side;
  unit_type : json_string option;
  character : json_string option;
  name : json_string option;
}

and literal_no_units = [ `No_units ]

and save2_checkpoint_state_tiles_item_light =
  [ `Undefined | `Object2 of save2_checkpoint_state_tiles_item_light_member1 ]

and save2_checkpoint_state_tiles_item_light_member1 = {
  wire : wire;
  radius : json_number;
  color : fragments_light23_color_member1 option;
  fixture : fragments_rule11_remember option;
  dx : json_number option;
  dy : json_number option;
  power : json_number;
  when_ : save2_checkpoint_state_tiles_item_fog_member1 option;
}

and fragments_light23_color = [ `Undefined | `Tuple of fragments_light23_color_member1 ]
and fragments_light23_color_member1 = json_number * json_number * json_number

and fragments_tiles11_item_wall_corners =
  [ `Undefined | `Array of fragments_tiles11_item_wall_corners_member1 ]

and fragments_tiles11_item_wall_corners_member1 = json_number array

and save2_checkpoint_state_campaign =
  [ `Undefined | `Object2 of save2_checkpoint_state_campaign_member1 ]

and save2_checkpoint_state_campaign_member1 = {
  wire : wire;
  mission : json_number;
  id : json_string;
  vigil : fragments_native_state38_campaign_member1_vigil_member1 option;
  refuge : fragments_native_state38_campaign_member1_refuge_member1 option;
  completed : json_boolean;
  marks : fragments_native_state38_log;
  holds : fragments_native_state38_campaign_member1_holds_member1 option;
  recall : fragments_native_state38_log;
  fallen_companions : fragments_native_state38_log option;
  carry_gold : json_number;
  dialogue : fragments_native_state38_campaign_member1_dialogue;
  dialogue_index : json_number;
  outcome : fragments_tiles11_item_owner;
  veterans : save2_checkpoint_state_campaign_member1_veterans_member1 option;
  story_veterans : save2_checkpoint_state_campaign_member1_veterans_member1 option;
  evacuated : save2_checkpoint_state_campaign_member1_evacuated_member1 option;
  choices : fragments_native_state38_log option;
}

and fragments_native_state38_campaign_member1_vigil =
  [ `Undefined | `Object2 of fragments_native_state38_campaign_member1_vigil_member1 ]

and fragments_native_state38_campaign_member1_vigil_member1 = {
  wire : wire;
  held_turns : json_number;
  last_counted_round : json_number;
}

and fragments_native_state38_campaign_member1_refuge =
  [ `Undefined | `Object2 of fragments_native_state38_campaign_member1_refuge_member1 ]

and fragments_native_state38_campaign_member1_refuge_member1 = {
  wire : wire;
  held_turns : json_number;
  last_counted_round : json_number;
  escalated : json_boolean;
}

and fragments_native_state38_campaign_member1_holds =
  [ `Undefined | `Object2 of fragments_native_state38_campaign_member1_holds_member1 ]

and fragments_native_state38_campaign_member1_holds_member1 =
  fragments_native_state38_campaign_member1_vigil_member1 String_map.t

and fragments_native_state38_campaign_member1_dialogue =
  fragments_native_state38_campaign_member1_dialogue_item array

and fragments_native_state38_campaign_member1_dialogue_item = {
  wire : wire;
  sound : json_string option;
  speaker : json_string option;
  character : json_string option;
  portrait : json_string;
  text : json_string;
}

and save2_checkpoint_state_campaign_member1_veterans =
  [ `Undefined | `Array of save2_checkpoint_state_campaign_member1_veterans_member1 ]

and save2_checkpoint_state_campaign_member1_veterans_member1 =
  save2_checkpoint_state_campaign_member1_veterans_member1_item array

and save2_checkpoint_state_campaign_member1_veterans_member1_item = {
  wire : wire;
  role : fragments_native_state38_units_item_role option;
  experience : json_number;
  items : fragments_native_state38_units_item_items_member1 option;
  type_ : json_string;
  name : json_string option;
  character : json_string option;
}

and save2_checkpoint_state_campaign_member1_evacuated =
  [ `Undefined | `Array of save2_checkpoint_state_campaign_member1_evacuated_member1 ]

and save2_checkpoint_state_campaign_member1_evacuated_member1 =
  save2_checkpoint_state_campaign_member1_evacuated_member1_item array

and save2_checkpoint_state_campaign_member1_evacuated_member1_item = {
  wire : wire;
  event : json_string;
  unit_ : save2_checkpoint_state_units_item;
}

and javascript1_state_gold = { wire : wire; player : json_number; enemy : json_number }

and save2_checkpoint_state_factions =
  [ `Undefined | `Array of save2_checkpoint_state_factions_member1 ]

and save2_checkpoint_state_factions_member1 = save2_checkpoint_state_factions_member1_item array

and save2_checkpoint_state_factions_member1_item = {
  wire : wire;
  id : json_string;
  name : json_string option;
  phase : save35_setup_first;
  controller : fragments_factions7_item_controller;
  palette : json_string;
  recruits : fragments_factions7_item_recruits_member1 option;
  relations : fragments_factions7_item_relations_member1 option;
  transfer : save2_checkpoint_state_factions_member1_item_transfer_member1 option;
}

and fragments_factions7_item_controller =
  [ `Human of fragments_factions7_item_controller_member0
  | `Passive of fragments_factions7_item_controller_member1
  | `Ai of fragments_factions7_item_controller_member2 ]

and fragments_factions7_item_controller_member0 = { wire : wire; kind : literal_human }
and literal_human = [ `Human ]
and fragments_factions7_item_controller_member1 = { wire : wire; kind : literal_passive }
and literal_passive = [ `Passive ]

and fragments_factions7_item_controller_member2 = {
  wire : wire;
  kind : literal_ai;
  policy : fragments_factions7_item_controller_member2_policy option;
}

and literal_ai = [ `Ai ]

and fragments_factions7_item_controller_member2_policy =
  [ `Undefined | `Classic | `Rush | `Balanced | `Rotated | `Situational | `Predictor ]

and literal_classic = [ `Classic ]
and literal_rush = [ `Rush ]
and literal_balanced = [ `Balanced ]
and literal_rotated = [ `Rotated ]
and literal_situational = [ `Situational ]
and literal_predictor = [ `Predictor ]

and fragments_factions7_item_recruits =
  [ `Undefined | `Array of fragments_factions7_item_recruits_member1 ]

and fragments_factions7_item_recruits_member1 = fragments_factions7_item_recruits_member1_item array

and fragments_factions7_item_recruits_member1_item = {
  wire : wire;
  unit_ : json_string;
  cost : json_number option;
}

and fragments_factions7_item_relations =
  [ `Undefined | `Object2 of fragments_factions7_item_relations_member1 ]

and fragments_factions7_item_relations_member1 =
  fragments_factions7_item_relations_member1_value String_map.t

and fragments_factions7_item_relations_member1_value = [ `Neutral | `Allied | `Hostile ]
and literal_allied = [ `Allied ]
and literal_hostile = [ `Hostile ]

and save2_checkpoint_state_factions_member1_item_transfer =
  [ `Undefined | `Object2 of save2_checkpoint_state_factions_member1_item_transfer_member1 ]

and save2_checkpoint_state_factions_member1_item_transfer_member1 = {
  wire : wire;
  to_ : json_string;
  when_ : save2_checkpoint_state_tiles_item_fog_member1;
}

and save2_checkpoint_state_ground_items =
  [ `Undefined | `Array of fragments_native_state38_ground_items ]

and fragments_native_state38_ground_items = fragments_native_state38_ground_items_item array

and fragments_native_state38_ground_items_item = {
  wire : wire;
  tile : json_string;
  item : fragments_native_state38_ground_items_item_item;
}

and json_unknown = Js.Json.t
and save2_setup = [ `Undefined | `Object2 of save2_setup_member1 ]

and save2_setup_member1 = {
  wire : wire;
  first : save35_setup_first;
  enemy : save2_setup_member1_enemy;
}

and save2_setup_member1_enemy = {
  wire : wire;
  policy : save35_setup_enemy_policy;
  blocks : save35_setup_enemy_blocks_member1 option;
  recruitment_limit : json_number option;
  diplomacy : save2_setup_member1_enemy_diplomacy;
}

and save35_setup_enemy_policy =
  [ `Classic | `Rush | `Balanced | `Rotated | `Situational | `Predictor ]

and save35_setup_enemy_blocks = [ `Undefined | `Object2 of save35_setup_enemy_blocks_member1 ]

and save35_setup_enemy_blocks_member1 = {
  wire : wire;
  recruiter : save35_setup_enemy_blocks_member1_recruiter;
  campaign : save35_setup_enemy_blocks_member1_campaign;
  planner : save35_setup_enemy_blocks_member1_planner;
}

and save35_setup_enemy_blocks_member1_recruiter =
  [ `Classic
  | `Balanced
  | `Situational
  | `Predictor
  | `Rotation
  | `Frontline
  | `Aggressive_predictor ]

and literal_rotation = [ `Rotation ]
and literal_frontline = [ `Frontline ]
and literal_aggressive_predictor = [ `Aggressive_predictor ]
and save35_setup_enemy_blocks_member1_campaign = [ `Arena | `Mission ]
and literal_arena = [ `Arena ]
and literal_mission = [ `Mission ]
and save35_setup_enemy_blocks_member1_planner = [ `Classic | `Full | `Queued ]
and literal_full = [ `Full ]
and literal_queued = [ `Queued ]
and save2_setup_member1_enemy_diplomacy = [ `Diplomatic | `Unrestricted | `Pragmatic | `Restrained ]
and literal_diplomatic = [ `Diplomatic ]
and literal_unrestricted = [ `Unrestricted ]
and literal_pragmatic = [ `Pragmatic ]
and literal_restrained = [ `Restrained ]
and save2_progress = [ `Undefined | `Object2 of save2_progress_member1 ]
and save2_progress_member1 = { wire : wire; entry : history1_entries_item }

and history1_entries_item = {
  wire : wire;
  id : json_string;
  saved_at : json_number;
  run : json_string;
  mission : json_string;
  round : json_number;
  side : json_string;
  mission_start : json_boolean;
  run_id : json_string;
  branch_id : json_string;
  checkpoint_id : json_string;
  ordinal : json_number;
  kind : history1_entries_item_kind;
  parent_id : json_string option;
}

and history1_entries_item_kind = [ `Turn | `Recovery | `Resume ]
and literal_turn = [ `Turn ]
and literal_recovery = [ `Recovery ]
and literal_resume = [ `Resume ]

and save3 = {
  wire : wire;
  version : literal_3;
  checkpoint : save3_checkpoint;
  pending_enemy : json_boolean;
  planner : json_unknown option;
  setup : save2_setup_member1 option;
  progress : save2_progress_member1 option;
  undo : json_unknown option;
}

and literal_3 = [ `V3 ]
and save3_checkpoint = { wire : wire; seed : json_number; state : save3_checkpoint_state }

and save3_checkpoint_state = {
  wire : wire;
  round : json_number;
  turn : save35_setup_first;
  winner : fragments_native_state38_winner;
  armies : fragments_native_state38_armies_member1 option;
  condition_memory : fragments_native_state38_log option;
  map_id : json_string;
  map_revision : json_number;
  cols : json_number;
  rows : json_number;
  next_id : json_number;
  log : fragments_native_state38_log;
  last_combat : fragments_native_state38_last_combat option;
  units : save2_checkpoint_state_units;
  tiles : fragments_tiles11;
  campaign : save2_checkpoint_state_campaign_member1 option;
  gold : javascript1_state_gold;
  factions : fragments_factions7 option;
  ground_items : fragments_native_state38_ground_items option;
  combat_rng : json_number option;
}

and fragments_tiles11 = fragments_tiles11_item array

and fragments_tiles11_item = {
  wire : wire;
  owner : fragments_tiles11_item_owner;
  id : json_string;
  location : json_string option;
  col : json_number;
  row : json_number;
  terrain : json_string;
  connections : fragments_tiles11_item_connections_member1 option;
  dead_ends : fragments_tiles11_item_connections_member1 option;
  walls : fragments_tiles11_item_connections_member1 option;
  floor : json_string option;
  fog : fragments_rule11 option;
  light : fragments_light23 option;
  wall_corners : fragments_tiles11_item_wall_corners_member1 option;
}

and fragments_light23_when_ = [ `Undefined | `Object2 of fragments_rule11 ]

and fragments_rule11 = {
  wire : wire;
  condition : fragments_rule11_condition;
  remember : fragments_rule11_remember option;
}

and fragments_rule11_condition =
  [ `Owner of fragments_rule11_condition_member0
  | `Visible_units of fragments_rule11_condition_member1
  | `Choice of fragments_rule11_condition_member2
  | `Flag of fragments_rule11_condition_member3
  | `Event of fragments_rule11_condition_member4
  | `Round of fragments_rule11_condition_member5
  | `Proximity of fragments_rule11_condition_member6
  | `Has_units of fragments_rule11_condition_member7
  | `No_units of fragments_rule11_condition_member8
  | `Object10 of fragments_rule11_condition_member9 ]

and fragments_rule11_condition_member9 = {
  wire : wire;
  kind : fragments_rule11_condition_member9_kind;
  conditions : fragments_rule11_condition_member9_conditions;
}

and fragments_rule11_condition_member9_conditions = fragments_rule11_condition array
and fragments_tiles11_item_light = [ `Undefined | `Object2 of fragments_light23 ]

and fragments_light23 = {
  wire : wire;
  radius : json_number;
  color : fragments_light23_color_member1 option;
  fixture : fragments_rule11_remember option;
  dx : json_number option;
  dy : json_number option;
  power : json_number;
  when_ : fragments_rule11 option;
}

and save3_checkpoint_state_factions = [ `Undefined | `Array of fragments_factions7 ]
and fragments_factions7 = fragments_factions7_item array

and fragments_factions7_item = {
  wire : wire;
  id : json_string;
  name : json_string option;
  phase : save35_setup_first;
  controller : fragments_factions7_item_controller;
  palette : json_string;
  recruits : fragments_factions7_item_recruits_member1 option;
  relations : fragments_factions7_item_relations_member1 option;
  transfer : fragments_factions7_item_transfer_member1 option;
}

and fragments_factions7_item_transfer =
  [ `Undefined | `Object2 of fragments_factions7_item_transfer_member1 ]

and fragments_factions7_item_transfer_member1 = {
  wire : wire;
  to_ : json_string;
  when_ : fragments_rule11;
}

and save4 = {
  wire : wire;
  version : literal_4;
  checkpoint : save3_checkpoint;
  pending_enemy : json_boolean;
  planner : json_unknown option;
  setup : save2_setup_member1 option;
  progress : save2_progress_member1 option;
  undo : json_unknown option;
}

and literal_4 = [ `V4 ]

and save5 = {
  wire : wire;
  version : literal_5;
  checkpoint : save3_checkpoint;
  pending_enemy : json_boolean;
  setup : save2_setup_member1 option;
  progress : save2_progress_member1 option;
  undo : json_unknown option;
}

and literal_5 = [ `V5 ]

and save6 = {
  wire : wire;
  version : literal_6;
  checkpoint : save3_checkpoint;
  pending_enemy : json_boolean;
  setup : save2_setup_member1 option;
  progress : save2_progress_member1 option;
  undo : json_unknown option;
}

and literal_6 = [ `V6 ]

and save7 = {
  wire : wire;
  version : literal_7;
  checkpoint : save7_checkpoint;
  pending_enemy : json_boolean;
  setup : save2_setup_member1 option;
  progress : save2_progress_member1 option;
  undo : json_unknown option;
}

and literal_7 = [ `V7 ]
and save7_checkpoint = { wire : wire; seed : json_number; state : save7_checkpoint_state }

and save7_checkpoint_state = {
  wire : wire;
  round : json_number;
  turn : save35_setup_first;
  winner : fragments_native_state38_winner;
  armies : fragments_native_state38_armies_member1 option;
  condition_memory : fragments_native_state38_log option;
  map_id : json_string;
  map_revision : json_number;
  cols : json_number;
  rows : json_number;
  next_id : json_number;
  log : fragments_native_state38_log;
  last_combat : fragments_native_state38_last_combat option;
  units : save2_checkpoint_state_units;
  tiles : fragments_tiles11;
  campaign : save2_checkpoint_state_campaign_member1 option;
  gold : javascript1_state_gold;
  factions : fragments_factions7;
  ground_items : fragments_native_state38_ground_items option;
  combat_rng : json_number option;
}

and save8 = {
  wire : wire;
  version : literal_8;
  checkpoint : save8_checkpoint;
  pending_enemy : json_boolean;
  setup : save2_setup_member1 option;
  progress : save2_progress_member1 option;
  undo : json_unknown option;
}

and literal_8 = [ `V8 ]
and save8_checkpoint = { wire : wire; seed : json_number; state : save8_checkpoint_state }

and save8_checkpoint_state = {
  wire : wire;
  round : json_number;
  turn : save35_setup_first;
  winner : fragments_native_state38_winner;
  armies : fragments_native_state38_armies_member1 option;
  condition_memory : fragments_native_state38_log option;
  map_id : json_string;
  map_revision : json_number;
  cols : json_number;
  rows : json_number;
  next_id : json_number;
  log : fragments_native_state38_log;
  last_combat : fragments_native_state38_last_combat option;
  units : save2_checkpoint_state_units;
  tiles : fragments_tiles11;
  campaign : save2_checkpoint_state_campaign_member1 option;
  gold : javascript1_state_gold;
  factions : fragments_factions7;
  ground_items : fragments_native_state38_ground_items;
  combat_rng : json_number option;
}

and save9 = {
  wire : wire;
  version : literal_9;
  checkpoint : save8_checkpoint;
  pending_enemy : json_boolean;
  setup : save2_setup_member1 option;
  progress : save2_progress_member1 option;
  undo : json_unknown option;
}

and literal_9 = [ `V9 ]

and save10 = {
  wire : wire;
  version : literal_10;
  checkpoint : save8_checkpoint;
  pending_enemy : json_boolean;
  setup : save2_setup_member1 option;
  progress : save2_progress_member1 option;
  undo : json_unknown option;
}

and literal_10 = [ `V10 ]

and save11 = {
  wire : wire;
  version : literal_11;
  checkpoint : save8_checkpoint;
  pending_enemy : json_boolean;
  setup : save2_setup_member1 option;
  progress : save2_progress_member1 option;
  undo : json_unknown option;
}

and literal_11 = [ `V11 ]

and save12 = {
  wire : wire;
  version : literal_12;
  checkpoint : save8_checkpoint;
  pending_enemy : json_boolean;
  setup : save2_setup_member1 option;
  progress : save2_progress_member1 option;
  undo : json_unknown option;
}

and literal_12 = [ `V12 ]

and save13 = {
  wire : wire;
  version : literal_13;
  checkpoint : save8_checkpoint;
  pending_enemy : json_boolean;
  setup : save2_setup_member1 option;
  progress : save2_progress_member1 option;
  undo : json_unknown option;
}

and literal_13 = [ `V13 ]

and save14 = {
  wire : wire;
  version : literal_14;
  checkpoint : save8_checkpoint;
  pending_enemy : json_boolean;
  setup : save2_setup_member1 option;
  progress : save2_progress_member1 option;
  undo : json_unknown option;
}

and literal_14 = [ `V14 ]

and save15 = {
  wire : wire;
  version : literal_15;
  checkpoint : save8_checkpoint;
  pending_enemy : json_boolean;
  setup : save2_setup_member1 option;
  progress : save2_progress_member1 option;
  undo : json_unknown option;
}

and literal_15 = [ `V15 ]

and save16 = {
  wire : wire;
  version : literal_16;
  checkpoint : save8_checkpoint;
  pending_enemy : json_boolean;
  setup : save2_setup_member1 option;
  progress : save2_progress_member1 option;
  undo : json_unknown option;
}

and literal_16 = [ `V16 ]

and save17 = {
  wire : wire;
  version : literal_17;
  checkpoint : save17_checkpoint;
  pending_enemy : json_boolean;
  setup : save2_setup_member1 option;
  progress : save2_progress_member1 option;
  undo : json_unknown option;
}

and literal_17 = [ `V17 ]
and save17_checkpoint = { wire : wire; seed : json_number; state : save17_checkpoint_state }

and save17_checkpoint_state = {
  wire : wire;
  round : json_number;
  turn : save35_setup_first;
  winner : fragments_native_state38_winner;
  armies : fragments_native_state38_armies_member1 option;
  condition_memory : fragments_native_state38_log option;
  map_id : json_string;
  map_revision : json_number;
  cols : json_number;
  rows : json_number;
  next_id : json_number;
  log : fragments_native_state38_log;
  last_combat : fragments_native_state38_last_combat option;
  units : save2_checkpoint_state_units;
  tiles : save17_checkpoint_state_tiles;
  campaign : save2_checkpoint_state_campaign_member1 option;
  gold : javascript1_state_gold;
  factions : fragments_factions7;
  ground_items : fragments_native_state38_ground_items;
  combat_rng : json_number option;
}

and save17_checkpoint_state_tiles = save17_checkpoint_state_tiles_item array

and save17_checkpoint_state_tiles_item = {
  wire : wire;
  owner : fragments_tiles11_item_owner;
  id : json_string;
  location : json_string option;
  col : json_number;
  row : json_number;
  terrain : json_string;
  connections : fragments_tiles11_item_connections_member1 option;
  dead_ends : fragments_tiles11_item_connections_member1 option;
  walls : fragments_tiles11_item_connections_member1 option;
  floor : json_string option;
  fog : fragments_rule11 option;
  light : fragments_light23 option;
  gates : fragments_native_state38_log option;
}

and save18 = {
  wire : wire;
  version : literal_18;
  checkpoint : save18_checkpoint;
  pending_enemy : json_boolean;
  setup : save2_setup_member1 option;
  progress : save2_progress_member1 option;
  undo : json_unknown option;
}

and literal_18 = [ `V18 ]
and save18_checkpoint = { wire : wire; seed : json_number; state : save18_checkpoint_state }

and save18_checkpoint_state = {
  wire : wire;
  round : json_number;
  turn : save35_setup_first;
  winner : fragments_native_state38_winner;
  armies : fragments_native_state38_armies_member1 option;
  condition_memory : fragments_native_state38_log option;
  map_id : json_string;
  map_revision : json_number;
  cols : json_number;
  rows : json_number;
  next_id : json_number;
  log : fragments_native_state38_log;
  last_combat : fragments_native_state38_last_combat option;
  units : save2_checkpoint_state_units;
  tiles : save18_checkpoint_state_tiles;
  campaign : save2_checkpoint_state_campaign_member1 option;
  gold : javascript1_state_gold;
  factions : fragments_factions7;
  ground_items : fragments_native_state38_ground_items;
  combat_rng : json_number option;
}

and save18_checkpoint_state_tiles = save18_checkpoint_state_tiles_item array

and save18_checkpoint_state_tiles_item = {
  wire : wire;
  owner : fragments_tiles11_item_owner;
  id : json_string;
  location : json_string option;
  col : json_number;
  row : json_number;
  terrain : json_string;
  connections : fragments_tiles11_item_connections_member1 option;
  dead_ends : fragments_tiles11_item_connections_member1 option;
  walls : fragments_tiles11_item_connections_member1 option;
  floor : json_string option;
  fog : fragments_rule11 option;
  light : fragments_light23 option;
}

and save19 = {
  wire : wire;
  version : literal_19;
  checkpoint : save18_checkpoint;
  pending_enemy : json_boolean;
  setup : save2_setup_member1 option;
  progress : save2_progress_member1 option;
  undo : json_unknown option;
}

and literal_19 = [ `V19 ]

and save20 = {
  wire : wire;
  version : literal_20;
  checkpoint : save18_checkpoint;
  pending_enemy : json_boolean;
  setup : save2_setup_member1 option;
  progress : save2_progress_member1 option;
  undo : json_unknown option;
}

and literal_20 = [ `V20 ]

and save21 = {
  wire : wire;
  version : literal_21;
  checkpoint : save21_checkpoint;
  pending_enemy : json_boolean;
  setup : save2_setup_member1 option;
  progress : save2_progress_member1 option;
  undo : json_unknown option;
}

and literal_21 = [ `V21 ]
and save21_checkpoint = { wire : wire; seed : json_number; state : save21_checkpoint_state }

and save21_checkpoint_state = {
  wire : wire;
  round : json_number;
  turn : save35_setup_first;
  winner : fragments_native_state38_winner;
  armies : fragments_native_state38_armies_member1 option;
  condition_memory : fragments_native_state38_log option;
  map_id : json_string;
  map_revision : json_number;
  cols : json_number;
  rows : json_number;
  next_id : json_number;
  log : fragments_native_state38_log;
  last_combat : fragments_native_state38_last_combat option;
  units : save2_checkpoint_state_units;
  tiles : save18_checkpoint_state_tiles;
  campaign : save2_checkpoint_state_campaign_member1 option;
  gold : fragments_native_state38_gold;
  factions : fragments_factions7;
  ground_items : fragments_native_state38_ground_items;
  combat_rng : json_number option;
}

and fragments_native_state38_gold = json_number String_map.t

and save22 = {
  wire : wire;
  version : literal_22;
  checkpoint : save21_checkpoint;
  pending_enemy : json_boolean;
  setup : save2_setup_member1 option;
  progress : save2_progress_member1 option;
  undo : json_unknown option;
}

and literal_22 = [ `V22 ]

and save23 = {
  wire : wire;
  version : literal_23;
  checkpoint : save21_checkpoint;
  pending_enemy : json_boolean;
  setup : save2_setup_member1 option;
  progress : save2_progress_member1 option;
  undo : json_unknown option;
}

and literal_23 = [ `V23 ]

and save24 = {
  wire : wire;
  version : literal_24;
  checkpoint : save21_checkpoint;
  pending_enemy : json_boolean;
  setup : save2_setup_member1 option;
  progress : save2_progress_member1 option;
  undo : json_unknown option;
}

and literal_24 = [ `V24 ]

and save25 = {
  wire : wire;
  version : literal_25;
  checkpoint : save21_checkpoint;
  pending_enemy : json_boolean;
  setup : save2_setup_member1 option;
  progress : save2_progress_member1 option;
  undo : json_unknown option;
}

and literal_25 = [ `V25 ]

and save26 = {
  wire : wire;
  version : literal_26;
  checkpoint : save26_checkpoint;
  pending_enemy : json_boolean;
  setup : save2_setup_member1 option;
  progress : save2_progress_member1 option;
  undo : json_unknown option;
}

and literal_26 = [ `V26 ]
and save26_checkpoint = { wire : wire; seed : json_number; state : save26_checkpoint_state }

and save26_checkpoint_state = {
  wire : wire;
  round : json_number;
  turn : save35_setup_first;
  winner : fragments_native_state38_winner;
  armies : fragments_native_state38_armies_member1 option;
  condition_memory : fragments_native_state38_log option;
  map_id : json_string;
  map_revision : json_number;
  cols : json_number;
  rows : json_number;
  next_id : json_number;
  log : fragments_native_state38_log;
  last_combat : fragments_native_state38_last_combat option;
  units : save26_checkpoint_state_units;
  tiles : save26_checkpoint_state_tiles;
  campaign : save26_checkpoint_state_campaign_member1 option;
  gold : fragments_native_state38_gold;
  factions : save26_checkpoint_state_factions;
  ground_items : fragments_native_state38_ground_items;
  combat_rng : json_number option;
}

and save26_checkpoint_state_units = save26_checkpoint_state_units_item array

and save26_checkpoint_state_units_item = {
  wire : wire;
  tile : json_string;
  id : json_string;
  side : save35_setup_first;
  role : fragments_native_state38_units_item_role option;
  faction : json_string option;
  items : fragments_native_state38_units_item_items_member1 option;
  ai_group : json_string option;
  type_ : json_string;
  hp : json_number;
  moved : json_boolean;
  attacked : json_boolean;
  retreating : fragments_rule11_remember option;
  rebuilding : fragments_rule11_remember option;
  character : json_string option;
  experience : json_number option;
}

and save26_checkpoint_state_tiles = save26_checkpoint_state_tiles_item array

and save26_checkpoint_state_tiles_item = {
  wire : wire;
  owner : fragments_tiles11_item_owner;
  id : json_string;
  location : json_string option;
  col : json_number;
  row : json_number;
  terrain : json_string;
  connections : fragments_tiles11_item_connections_member1 option;
  dead_ends : fragments_tiles11_item_connections_member1 option;
  walls : fragments_tiles11_item_connections_member1 option;
  floor : json_string option;
  fog : fragments_light28_member1_when__member1 option;
  light : fragments_light28_member1 option;
}

and fragments_light28_member1_when_ =
  [ `Undefined | `Object2 of fragments_light28_member1_when__member1 ]

and fragments_light28_member1_when__member1 = {
  wire : wire;
  condition : fragments_light28_member1_when__member1_condition;
  remember : fragments_rule11_remember option;
}

and fragments_light28_member1_when__member1_condition =
  [ `Owner of fragments_rule11_condition_member0
  | `Proximity of fragments_light28_member1_when__member1_condition_member1
  | `Visible_units of fragments_rule11_condition_member1
  | `Has_units of fragments_light28_member1_when__member1_condition_member3
  | `No_units of fragments_light28_member1_when__member1_condition_member4
  | `Choice of fragments_rule11_condition_member2
  | `Flag of fragments_rule11_condition_member3
  | `Event of fragments_rule11_condition_member4
  | `Round of fragments_rule11_condition_member5
  | `Object10 of fragments_light28_member1_when__member1_condition_member9 ]

and fragments_light28_member1_when__member1_condition_member1 = {
  wire : wire;
  kind : literal_proximity;
  unit_types : fragments_native_state38_log option;
  tile : json_string option;
  radius : json_number;
  side : fragments_rule11_condition_member1_side;
  unit_type : json_string option;
  character : json_string option;
}

and fragments_light28_member1_when__member1_condition_member3 = {
  wire : wire;
  kind : literal_has_units;
  ai_group : json_string option;
  side : fragments_rule11_condition_member1_side;
  unit_type : json_string option;
  character : json_string option;
}

and fragments_light28_member1_when__member1_condition_member4 = {
  wire : wire;
  kind : literal_no_units;
  side : fragments_rule11_condition_member1_side;
  unit_type : json_string option;
  character : json_string option;
}

and fragments_light28_member1_when__member1_condition_member9 = {
  wire : wire;
  kind : fragments_rule11_condition_member9_kind;
  conditions : fragments_light28_member1_when__member1_condition_member9_conditions;
}

and fragments_light28_member1_when__member1_condition_member9_conditions =
  fragments_light28_member1_when__member1_condition array

and save26_checkpoint_state_tiles_item_light = [ `Undefined | `Object2 of fragments_light28_member1 ]

and fragments_light28_member1 = {
  wire : wire;
  radius : json_number;
  color : fragments_light23_color_member1 option;
  fixture : fragments_rule11_remember option;
  dx : json_number option;
  dy : json_number option;
  power : json_number;
  when_ : fragments_light28_member1_when__member1 option;
}

and save26_checkpoint_state_campaign =
  [ `Undefined | `Object2 of save26_checkpoint_state_campaign_member1 ]

and save26_checkpoint_state_campaign_member1 = {
  wire : wire;
  mission : json_number;
  id : json_string;
  vigil : fragments_native_state38_campaign_member1_vigil_member1 option;
  refuge : fragments_native_state38_campaign_member1_refuge_member1 option;
  completed : json_boolean;
  marks : fragments_native_state38_log;
  holds : fragments_native_state38_campaign_member1_holds_member1 option;
  recall : fragments_native_state38_log;
  fallen_companions : fragments_native_state38_log option;
  carry_gold : json_number;
  dialogue : fragments_native_state38_campaign_member1_dialogue;
  dialogue_index : json_number;
  outcome : fragments_tiles11_item_owner;
  veterans : save26_checkpoint_state_campaign_member1_veterans_member1 option;
  story_veterans : save26_checkpoint_state_campaign_member1_veterans_member1 option;
  evacuated : save26_checkpoint_state_campaign_member1_evacuated_member1 option;
  choices : fragments_native_state38_log option;
}

and save26_checkpoint_state_campaign_member1_veterans =
  [ `Undefined | `Array of save26_checkpoint_state_campaign_member1_veterans_member1 ]

and save26_checkpoint_state_campaign_member1_veterans_member1 =
  save26_checkpoint_state_campaign_member1_veterans_member1_item array

and save26_checkpoint_state_campaign_member1_veterans_member1_item = {
  wire : wire;
  role : fragments_native_state38_units_item_role option;
  experience : json_number;
  items : fragments_native_state38_units_item_items_member1 option;
  type_ : json_string;
  character : json_string option;
}

and save26_checkpoint_state_campaign_member1_evacuated =
  [ `Undefined | `Array of save26_checkpoint_state_campaign_member1_evacuated_member1 ]

and save26_checkpoint_state_campaign_member1_evacuated_member1 =
  save26_checkpoint_state_campaign_member1_evacuated_member1_item array

and save26_checkpoint_state_campaign_member1_evacuated_member1_item = {
  wire : wire;
  event : json_string;
  unit_ : save26_checkpoint_state_units_item;
}

and save26_checkpoint_state_factions = save26_checkpoint_state_factions_item array

and save26_checkpoint_state_factions_item = {
  wire : wire;
  id : json_string;
  name : json_string option;
  phase : save35_setup_first;
  controller : fragments_factions7_item_controller;
  palette : json_string;
  recruits : fragments_factions7_item_recruits_member1 option;
  relations : fragments_factions7_item_relations_member1 option;
  transfer : save26_checkpoint_state_factions_item_transfer_member1 option;
}

and save26_checkpoint_state_factions_item_transfer =
  [ `Undefined | `Object2 of save26_checkpoint_state_factions_item_transfer_member1 ]

and save26_checkpoint_state_factions_item_transfer_member1 = {
  wire : wire;
  to_ : json_string;
  when_ : fragments_light28_member1_when__member1;
}

and save27 = {
  wire : wire;
  version : literal_27;
  checkpoint : save26_checkpoint;
  pending_enemy : json_boolean;
  setup : save2_setup_member1 option;
  progress : save2_progress_member1 option;
  undo : json_unknown option;
}

and literal_27 = [ `V27 ]

and save28 = {
  wire : wire;
  version : literal_28;
  checkpoint : save28_checkpoint;
  pending_enemy : json_boolean;
  setup : save2_setup_member1 option;
  progress : save2_progress_member1 option;
  undo : json_unknown option;
}

and literal_28 = [ `V28 ]
and save28_checkpoint = { wire : wire; seed : json_number; state : save28_checkpoint_state }

and save28_checkpoint_state = {
  wire : wire;
  round : json_number;
  turn : save35_setup_first;
  winner : fragments_native_state38_winner;
  armies : fragments_native_state38_armies_member1 option;
  condition_memory : fragments_native_state38_log option;
  map_id : json_string;
  map_revision : json_number;
  cols : json_number;
  rows : json_number;
  next_id : json_number;
  log : fragments_native_state38_log;
  last_combat : fragments_native_state38_last_combat option;
  units : save26_checkpoint_state_units;
  tiles : save26_checkpoint_state_tiles;
  campaign : save28_checkpoint_state_campaign_member1 option;
  gold : fragments_native_state38_gold;
  factions : save26_checkpoint_state_factions;
  ground_items : fragments_native_state38_ground_items;
  combat_rng : json_number option;
}

and save28_checkpoint_state_campaign =
  [ `Undefined | `Object2 of save28_checkpoint_state_campaign_member1 ]

and save28_checkpoint_state_campaign_member1 = {
  wire : wire;
  mission : json_number;
  id : json_string;
  vigil : fragments_native_state38_campaign_member1_vigil_member1 option;
  refuge : fragments_native_state38_campaign_member1_refuge_member1 option;
  completed : json_boolean;
  marks : fragments_native_state38_log;
  holds : fragments_native_state38_campaign_member1_holds_member1 option;
  recall : fragments_native_state38_log;
  fallen_companions : fragments_native_state38_log option;
  carry_gold : json_number;
  dialogue : fragments_native_state38_campaign_member1_dialogue;
  dialogue_index : json_number;
  outcome : fragments_tiles11_item_owner;
  veterans : save26_checkpoint_state_campaign_member1_veterans_member1 option;
  story_veterans : save26_checkpoint_state_campaign_member1_veterans_member1 option;
  evacuated : save26_checkpoint_state_campaign_member1_evacuated_member1 option;
  scripts : save28_checkpoint_state_campaign_member1_scripts_member1 option;
  choices : fragments_native_state38_log option;
}

and save28_checkpoint_state_campaign_member1_scripts =
  [ `Undefined | `Object2 of save28_checkpoint_state_campaign_member1_scripts_member1 ]

and save28_checkpoint_state_campaign_member1_scripts_member1 = {
  wire : wire;
  recruitment : fragments_native_state38_campaign_member1_scripts_member1_recruitment;
  failed : fragments_native_state38_log option;
  active : fragments_native_state38_log;
  holds : fragments_native_state38_campaign_member1_scripts_member1_holds;
  next_mark : json_number;
  pending : fragments_native_state38_campaign_member1_scripts_member1_pending;
  marks : save28_checkpoint_state_campaign_member1_scripts_member1_marks;
  completed : fragments_native_state38_log option;
  revealed : fragments_native_state38_log option;
}

and fragments_native_state38_campaign_member1_scripts_member1_recruitment =
  fragments_factions7_item_recruits_member1 String_map.t

and fragments_native_state38_campaign_member1_scripts_member1_holds =
  fragments_native_state38_campaign_member1_scripts_member1_holds_item array

and fragments_native_state38_campaign_member1_scripts_member1_holds_item = {
  wire : wire;
  event : json_string;
  predicate : json_number;
  turns : json_number;
  last_round : json_number;
}

and fragments_native_state38_campaign_member1_scripts_member1_pending =
  fragments_native_state38_campaign_member1_scripts_member1_pending_item array

and fragments_native_state38_campaign_member1_scripts_member1_pending_item = {
  wire : wire;
  area : fragments_native_state38_log;
  faction : json_string;
  ai_group : fragments_delta19_item_owner option;
  units : fragments_native_state38_campaign_member1_scripts_member1_pending_item_units;
}

and fragments_delta19_item_owner = [ `Undefined | `Null | `String of json_string ]

and fragments_native_state38_campaign_member1_scripts_member1_pending_item_units =
  fragments_native_state38_campaign_member1_scripts_member1_pending_item_units_item array

and fragments_native_state38_campaign_member1_scripts_member1_pending_item_units_item = {
  wire : wire;
  type_ : json_string;
  character : fragments_delta19_item_owner option;
  role :
    fragments_native_state38_campaign_member1_scripts_member1_pending_item_units_item_role option;
  items : fragments_native_state38_log option;
}

and fragments_native_state38_campaign_member1_scripts_member1_pending_item_units_item_role =
  [ `Undefined | `Null | `Leader | `Courier | `Captive ]

and save28_checkpoint_state_campaign_member1_scripts_member1_marks =
  save28_checkpoint_state_campaign_member1_scripts_member1_marks_item array

and save28_checkpoint_state_campaign_member1_scripts_member1_marks_item = {
  wire : wire;
  id : json_number;
  type_ : json_string;
  tile : json_string;
  light : fragments_light28_member1 option;
  light_remembered : fragments_rule11_remember option;
}

and save29 = {
  wire : wire;
  version : literal_29;
  checkpoint : save29_checkpoint;
  pending_enemy : json_boolean;
  setup : save2_setup_member1 option;
  progress : save2_progress_member1 option;
  undo : json_unknown option;
}

and literal_29 = [ `V29 ]
and save29_checkpoint = { wire : wire; seed : json_number; state : save29_checkpoint_state }

and save29_checkpoint_state = {
  wire : wire;
  round : json_number;
  turn : save35_setup_first;
  winner : fragments_native_state38_winner;
  armies : fragments_native_state38_armies_member1 option;
  condition_memory : fragments_native_state38_log option;
  map_id : json_string;
  map_revision : json_number;
  cols : json_number;
  rows : json_number;
  next_id : json_number;
  log : fragments_native_state38_log;
  last_combat : fragments_native_state38_last_combat option;
  units : save26_checkpoint_state_units;
  tiles : save26_checkpoint_state_tiles;
  campaign : save29_checkpoint_state_campaign_member1 option;
  gold : fragments_native_state38_gold;
  factions : save26_checkpoint_state_factions;
  ground_items : fragments_native_state38_ground_items;
  combat_rng : json_number option;
}

and save29_checkpoint_state_campaign =
  [ `Undefined | `Object2 of save29_checkpoint_state_campaign_member1 ]

and save29_checkpoint_state_campaign_member1 = {
  wire : wire;
  mission : json_number;
  id : json_string;
  vigil : fragments_native_state38_campaign_member1_vigil_member1 option;
  refuge : fragments_native_state38_campaign_member1_refuge_member1 option;
  completed : json_boolean;
  marks : fragments_native_state38_log;
  holds : fragments_native_state38_campaign_member1_holds_member1 option;
  recall : fragments_native_state38_log;
  fallen_companions : fragments_native_state38_log option;
  carry_gold : json_number;
  dialogue : fragments_native_state38_campaign_member1_dialogue;
  dialogue_index : json_number;
  outcome : fragments_tiles11_item_owner;
  veterans : save26_checkpoint_state_campaign_member1_veterans_member1 option;
  story_veterans : save26_checkpoint_state_campaign_member1_veterans_member1 option;
  evacuated : save26_checkpoint_state_campaign_member1_evacuated_member1 option;
  scripts : save28_checkpoint_state_campaign_member1_scripts_member1 option;
  flags : fragments_native_state38_campaign_member1_flags;
}

and fragments_native_state38_campaign_member1_flags =
  fragments_native_state38_campaign_member1_flags_value String_map.t

and fragments_native_state38_campaign_member1_flags_value =
  fragments_native_state38_campaign_member1_flags_value_item array

and fragments_native_state38_campaign_member1_flags_value_item = {
  wire : wire;
  state : json_boolean;
  stage : json_number;
  turn : json_number;
}

and save30 = {
  wire : wire;
  version : literal_30;
  checkpoint : save29_checkpoint;
  pending_enemy : json_boolean;
  setup : save35_setup option;
  progress : save2_progress_member1 option;
  undo : json_unknown option;
}

and literal_30 = [ `V30 ]
and save30_setup = [ `Undefined | `Object2 of save35_setup ]
and save35_setup = { wire : wire; first : save35_setup_first; enemy : save35_setup_enemy }

and save35_setup_enemy = {
  wire : wire;
  policy : save35_setup_enemy_policy;
  blocks : save35_setup_enemy_blocks_member1 option;
  recruitment_limit : json_number option;
  diplomacy : save35_setup_enemy_diplomacy;
}

and save35_setup_enemy_diplomacy = [ `Diplomatic | `Unrestricted ]

and save31 = {
  wire : wire;
  version : literal_31;
  checkpoint : save29_checkpoint;
  pending_enemy : json_boolean;
  setup : save35_setup option;
  progress : save2_progress_member1 option;
  undo : json_unknown option;
}

and literal_31 = [ `V31 ]

and save32 = {
  wire : wire;
  version : literal_32;
  checkpoint : save29_checkpoint;
  pending_enemy : json_boolean;
  setup : save35_setup option;
  progress : save2_progress_member1 option;
  undo : json_unknown option;
}

and literal_32 = [ `V32 ]

and save33 = {
  wire : wire;
  version : literal_33;
  checkpoint : save29_checkpoint;
  pending_enemy : json_boolean;
  setup : save35_setup option;
  progress : save2_progress_member1 option;
  undo : json_unknown option;
}

and literal_33 = [ `V33 ]

and save34 = {
  wire : wire;
  version : literal_34;
  checkpoint : save34_checkpoint;
  pending_enemy : json_boolean;
  setup : save35_setup option;
  progress : save2_progress_member1 option;
  undo : save34_undo_member1 option;
}

and literal_34 = [ `V34 ]
and save34_checkpoint = { wire : wire; seed : json_number; state : save34_checkpoint_state }

and save34_checkpoint_state = {
  wire : wire;
  round : json_number;
  turn : save35_setup_first;
  winner : fragments_native_state38_winner;
  armies : fragments_native_state38_armies_member1 option;
  condition_memory : fragments_native_state38_log option;
  map_id : json_string;
  map_revision : json_number;
  cols : json_number;
  rows : json_number;
  next_id : json_number;
  log : fragments_native_state38_log;
  last_combat : fragments_native_state38_last_combat option;
  units : save26_checkpoint_state_units;
  tiles : save26_checkpoint_state_tiles;
  campaign : save34_checkpoint_state_campaign_member1 option;
  gold : fragments_native_state38_gold;
  factions : save26_checkpoint_state_factions;
  ground_items : fragments_native_state38_ground_items;
  combat_rng : json_number option;
}

and save34_checkpoint_state_campaign =
  [ `Undefined | `Object2 of save34_checkpoint_state_campaign_member1 ]

and save34_checkpoint_state_campaign_member1 = {
  wire : wire;
  mission : json_number;
  id : json_string;
  vigil : fragments_native_state38_campaign_member1_vigil_member1 option;
  refuge : fragments_native_state38_campaign_member1_refuge_member1 option;
  completed : json_boolean;
  marks : fragments_native_state38_log;
  holds : fragments_native_state38_campaign_member1_holds_member1 option;
  recall : fragments_native_state38_log;
  fallen_companions : fragments_native_state38_log option;
  carry_gold : json_number;
  dialogue : fragments_native_state38_campaign_member1_dialogue;
  dialogue_index : json_number;
  outcome : fragments_tiles11_item_owner;
  veterans : save26_checkpoint_state_campaign_member1_veterans_member1 option;
  story_veterans : save26_checkpoint_state_campaign_member1_veterans_member1 option;
  evacuated : save26_checkpoint_state_campaign_member1_evacuated_member1 option;
  scripts : save34_checkpoint_state_campaign_member1_scripts_member1 option;
  flags : fragments_native_state38_campaign_member1_flags;
}

and save34_checkpoint_state_campaign_member1_scripts =
  [ `Undefined | `Object2 of save34_checkpoint_state_campaign_member1_scripts_member1 ]

and save34_checkpoint_state_campaign_member1_scripts_member1 = {
  wire : wire;
  recruitment : fragments_native_state38_campaign_member1_scripts_member1_recruitment;
  failed : fragments_native_state38_log option;
  active : fragments_native_state38_log;
  holds : fragments_native_state38_campaign_member1_scripts_member1_holds;
  next_mark : json_number;
  pending : fragments_native_state38_campaign_member1_scripts_member1_pending;
  marks : save28_checkpoint_state_campaign_member1_scripts_member1_marks;
  completed : fragments_native_state38_log;
  revealed : fragments_native_state38_log;
}

and save34_undo = [ `Undefined | `Array of save34_undo_member1 ]
and save34_undo_member1 = save34_undo_member1_item array

and save34_undo_member1_item = {
  wire : wire;
  round : json_number;
  unit_id : json_string;
  changes : save34_undo_member1_item_changes;
}

and save34_undo_member1_item_changes = save34_undo_member1_item_changes_item array

and save34_undo_member1_item_changes_item =
  [ `Set of save34_undo_member1_item_changes_item_member0
  | `Remove of save34_undo_member1_item_changes_item_member1 ]

and save34_undo_member1_item_changes_item_member0 = {
  wire : wire;
  op : literal_set;
  path : fragments_native_state38_log;
  value : json_unknown;
}

and literal_set = [ `Set ]

and save34_undo_member1_item_changes_item_member1 = {
  wire : wire;
  op : literal_remove;
  path : fragments_native_state38_log;
}

and literal_remove = [ `Remove ]

and save35 = {
  wire : wire;
  version : literal_35;
  checkpoint : save35_checkpoint;
  pending_enemy : json_boolean;
  setup : save35_setup;
  progress : save35_progress;
  undo : save34_undo_member1 option;
}

and literal_35 = [ `V35 ]
and save35_checkpoint = { wire : wire; seed : json_number; state : save35_checkpoint_state }

and save35_checkpoint_state = {
  wire : wire;
  round : json_number;
  turn : save35_setup_first;
  winner : fragments_native_state38_winner;
  armies : fragments_native_state38_armies_member1 option;
  condition_memory : fragments_native_state38_log option;
  map_id : json_string;
  map_revision : json_number;
  cols : json_number;
  rows : json_number;
  next_id : json_number;
  log : fragments_native_state38_log;
  last_combat : fragments_native_state38_last_combat option;
  units : save26_checkpoint_state_units;
  tiles : save26_checkpoint_state_tiles;
  campaign : save34_checkpoint_state_campaign_member1 option;
  gold : fragments_native_state38_gold;
  factions : save26_checkpoint_state_factions;
  ground_items : fragments_native_state38_ground_items;
  combat_rng : json_number;
}

and save35_progress = { wire : wire; entry : save35_progress_entry }

and save35_progress_entry = {
  wire : wire;
  id : json_string;
  saved_at : json_number;
  run : json_string;
  mission : json_string;
  round : json_number;
  side : json_string;
  mission_start : json_boolean;
  run_id : json_string;
  branch_id : json_string;
  checkpoint_id : json_string;
  ordinal : json_number;
  kind : save35_progress_entry_kind;
  parent_id : json_string option;
}

and save35_progress_entry_kind = [ `Turn | `Manual | `Recovery ]
and literal_manual = [ `Manual ]

and save36 = {
  wire : wire;
  version : literal_36;
  checkpoint : save35_checkpoint;
  pending_enemy : json_boolean;
  setup : save35_setup;
  progress : save35_progress;
  undo : save34_undo_member1 option;
}

and literal_36 = [ `V36 ]

and save37 = {
  wire : wire;
  version : literal_37;
  checkpoint : save35_checkpoint;
  pending_enemy : json_boolean;
  setup : save35_setup;
  progress : save37_progress;
  undo : save34_undo_member1 option;
}

and literal_37 = [ `V37 ]
and save37_progress = { wire : wire; entry : save37_progress_entry }

and save37_progress_entry =
  [ `Flash of save37_progress_entry_member0 | `Turn of history4_entries_item ]

and save37_progress_entry_member0 = {
  wire : wire;
  kind : literal_flash;
  id : json_string;
  saved_at : json_number;
  run : json_string;
  mission : json_string;
  round : json_number;
  side : json_string;
  mission_start : json_boolean;
  run_id : json_string;
  branch_id : json_string;
  checkpoint_id : json_string;
  ordinal : json_number;
}

and literal_flash = [ `Flash ]

and history4_entries_item = {
  wire : wire;
  kind : literal_turn;
  manual : json_boolean;
  parent_id : json_string option;
  id : json_string;
  saved_at : json_number;
  run : json_string;
  mission : json_string;
  round : json_number;
  side : json_string;
  mission_start : json_boolean;
  run_id : json_string;
  branch_id : json_string;
  checkpoint_id : json_string;
  ordinal : json_number;
}

and save38 = {
  wire : wire;
  version : literal_38;
  checkpoint : save35_checkpoint;
  pending_enemy : json_boolean;
  setup : save35_setup;
  progress : save37_progress;
  undo : save34_undo_member1 option;
}

and literal_38 = [ `V38 ]
and save39 = [ `Current of save39_member0 | `Turn of save39_member1 ]

and save39_member0 = {
  wire : wire;
  pending_enemy : json_boolean;
  undo : save34_undo_member1;
  progress : save39_member0_progress;
  version : literal_39;
  checkpoint : save39_member0_checkpoint;
  setup : save39_member0_setup;
}

and save39_member0_progress = { wire : wire; entry : save37_progress_entry_member0 }
and literal_39 = [ `V39 ]

and save39_member0_checkpoint = {
  wire : wire;
  seed : json_number;
  state : save39_member0_checkpoint_state;
}

and save39_member0_checkpoint_state = {
  wire : wire;
  factions : save39_member0_checkpoint_state_factions;
  ground_items : save39_member0_checkpoint_state_ground_items;
  armies : fragments_native_state38_armies_member1 option;
  condition_memory : fragments_native_state38_log option;
  map_id : json_string;
  map_revision : json_number;
  cols : json_number;
  rows : json_number;
  round : json_number;
  turn : save35_setup_first;
  gold : fragments_native_state38_gold;
  winner : fragments_native_state38_winner;
  next_id : json_number;
  combat_rng : json_number;
  log : fragments_native_state38_log;
  tiles : save39_member0_checkpoint_state_tiles;
  units : save39_member0_checkpoint_state_units;
  campaign : save39_member0_checkpoint_state_campaign_member1 option;
  last_combat : fragments_native_state38_last_combat option;
}

and save39_member0_checkpoint_state_factions = save39_member0_checkpoint_state_factions_item array

and save39_member0_checkpoint_state_factions_item = {
  wire : wire;
  id : json_string;
  phase : save35_setup_first;
  controller : fragments_factions7_item_controller;
  palette : json_string;
  name : json_string option;
  recruits : fragments_factions7_item_recruits_member1 option;
  relations : fragments_factions7_item_relations_member1 option;
  transfer : save39_member0_checkpoint_state_factions_item_transfer_member1 option;
}

and save39_member0_checkpoint_state_factions_item_transfer =
  [ `Undefined | `Object2 of save39_member0_checkpoint_state_factions_item_transfer_member1 ]

and save39_member0_checkpoint_state_factions_item_transfer_member1 = {
  wire : wire;
  to_ : json_string;
  when_ : save39_member0_checkpoint_state_tiles_item_fog_member1;
}

and save39_member0_checkpoint_state_tiles_item_fog_member1 = {
  wire : wire;
  condition : save39_member0_checkpoint_state_tiles_item_fog_member1_condition;
  remember : fragments_rule11_remember option;
}

and save39_member0_checkpoint_state_tiles_item_fog_member1_condition =
  [ `Owner of save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0
  | `Proximity of save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member1
  | `Visible_units of fragments_rule11_condition_member1
  | `Has_units of fragments_light28_member1_when__member1_condition_member3
  | `No_units of fragments_light28_member1_when__member1_condition_member4
  | `Choice of fragments_rule11_condition_member2
  | `Flag of fragments_rule11_condition_member3
  | `Event of fragments_rule11_condition_member4
  | `Round of fragments_rule11_condition_member5
  | `All of save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member9
  | `Any of save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member10 ]

and save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0 = {
  wire : wire;
  kind : literal_owner;
  tile : save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile option;
  side : json_string;
}

and save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile =
  [ `Undefined
  | `Location of
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member1
  | `Object3 of
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member2
  | `Object4 of
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member3 ]

and save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member1 = {
  wire : wire;
  location : json_string;
}

and save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member2 = {
  wire : wire;
  hex : save39_member0_checkpoint_state_ground_items_item_tile;
}

and save39_member0_checkpoint_state_ground_items_item_tile = {
  wire : wire;
  col : json_number;
  row : json_number;
}

and save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member3 = {
  wire : wire;
  binding :
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member3_binding;
}

and save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member3_binding =
  [ `Unit | `Mark | `Death ]

and literal_unit = [ `Unit ]
and literal_mark = [ `Mark ]
and literal_death = [ `Death ]

and save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member1 = {
  wire : wire;
  kind : literal_proximity;
  unit_types : fragments_native_state38_log option;
  tile : save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile option;
  radius : json_number;
  side : fragments_rule11_condition_member1_side;
  unit_type : json_string option;
  character : json_string option;
}

and save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member9 = {
  wire : wire;
  kind : literal_all;
  conditions : save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member9_conditions;
}

and save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member9_conditions =
  save39_member0_checkpoint_state_tiles_item_fog_member1_condition array

and save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member10 = {
  wire : wire;
  kind : literal_any;
  conditions : save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member9_conditions;
}

and save39_member0_checkpoint_state_ground_items =
  save39_member0_checkpoint_state_ground_items_item array

and save39_member0_checkpoint_state_ground_items_item = {
  wire : wire;
  tile : save39_member0_checkpoint_state_ground_items_item_tile;
  item : fragments_native_state38_ground_items_item_item;
}

and save39_member0_checkpoint_state_tiles = save39_member0_checkpoint_state_tiles_item array

and save39_member0_checkpoint_state_tiles_item = {
  wire : wire;
  id : json_string;
  location : json_string option;
  col : json_number;
  row : json_number;
  terrain : json_string;
  owner : fragments_tiles11_item_owner;
  connections : fragments_tiles11_item_connections_member1 option;
  dead_ends : fragments_tiles11_item_connections_member1 option;
  walls : fragments_tiles11_item_connections_member1 option;
  floor : json_string option;
  fog : save39_member0_checkpoint_state_tiles_item_fog_member1 option;
  light : save39_member0_checkpoint_state_tiles_item_light_member1 option;
}

and save39_member0_checkpoint_state_tiles_item_fog =
  [ `Undefined | `Object2 of save39_member0_checkpoint_state_tiles_item_fog_member1 ]

and save39_member0_checkpoint_state_tiles_item_light =
  [ `Undefined | `Object2 of save39_member0_checkpoint_state_tiles_item_light_member1 ]

and save39_member0_checkpoint_state_tiles_item_light_member1 = {
  wire : wire;
  color : fragments_light23_color_member1 option;
  fixture : fragments_rule11_remember option;
  dx : json_number option;
  dy : json_number option;
  radius : json_number;
  power : json_number;
  when_ : save39_member0_checkpoint_state_tiles_item_fog_member1 option;
}

and save39_member0_checkpoint_state_units = save39_member0_checkpoint_state_units_item array

and save39_member0_checkpoint_state_units_item = {
  wire : wire;
  faction : json_string option;
  items : fragments_native_state38_units_item_items_member1 option;
  ai_group : json_string option;
  id : json_string;
  type_ : json_string;
  side : save35_setup_first;
  tile : save39_member0_checkpoint_state_ground_items_item_tile;
  hp : json_number;
  moved : json_boolean;
  attacked : json_boolean;
  experience : json_number option;
  character : json_string option;
  role : fragments_native_state38_units_item_role option;
  retreating : fragments_rule11_remember option;
  rebuilding : fragments_rule11_remember option;
}

and save39_member0_checkpoint_state_campaign =
  [ `Undefined | `Object2 of save39_member0_checkpoint_state_campaign_member1 ]

and save39_member0_checkpoint_state_campaign_member1 = {
  wire : wire;
  scripts : save39_member0_checkpoint_state_campaign_member1_scripts_member1 option;
  evacuated : save39_member0_checkpoint_state_campaign_member1_evacuated_member1 option;
  id : json_string;
  mission : json_number;
  recall : fragments_native_state38_log;
  veterans : fragments_native_state38_campaign_member1_veterans_member1 option;
  story_veterans : fragments_native_state38_campaign_member1_veterans_member1 option;
  fallen_companions : fragments_native_state38_log option;
  carry_gold : json_number;
  marks : fragments_native_state38_log;
  dialogue : fragments_native_state38_campaign_member1_dialogue;
  dialogue_index : json_number;
  outcome : fragments_tiles11_item_owner;
  completed : json_boolean;
  refuge : fragments_native_state38_campaign_member1_refuge_member1 option;
  vigil : fragments_native_state38_campaign_member1_vigil_member1 option;
  holds : fragments_native_state38_campaign_member1_holds_member1 option;
  flags : fragments_native_state38_campaign_member1_flags;
}

and save39_member0_checkpoint_state_campaign_member1_scripts =
  [ `Undefined | `Object2 of save39_member0_checkpoint_state_campaign_member1_scripts_member1 ]

and save39_member0_checkpoint_state_campaign_member1_scripts_member1 = {
  wire : wire;
  failed : fragments_native_state38_log option;
  completed : fragments_native_state38_log;
  revealed : fragments_native_state38_log;
  recruitment : fragments_native_state38_campaign_member1_scripts_member1_recruitment;
  active : fragments_native_state38_log;
  holds : fragments_native_state38_campaign_member1_scripts_member1_holds;
  marks : save39_member0_checkpoint_state_campaign_member1_scripts_member1_marks;
  pending : save39_member0_checkpoint_state_campaign_member1_scripts_member1_pending;
  next_mark : json_number;
}

and save39_member0_checkpoint_state_campaign_member1_scripts_member1_marks =
  save39_member0_checkpoint_state_campaign_member1_scripts_member1_marks_item array

and save39_member0_checkpoint_state_campaign_member1_scripts_member1_marks_item = {
  wire : wire;
  light_remembered : fragments_rule11_remember option;
  id : json_number;
  type_ : json_string;
  tile : save39_member0_checkpoint_state_ground_items_item_tile;
  light : save39_member0_checkpoint_state_tiles_item_light_member1 option;
}

and save39_member0_checkpoint_state_campaign_member1_scripts_member1_pending =
  save39_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item array

and save39_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item = {
  wire : wire;
  area : save39_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item_area;
  faction : json_string;
  ai_group : fragments_delta19_item_owner option;
  units : fragments_native_state38_campaign_member1_scripts_member1_pending_item_units;
}

and save39_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item_area =
  save39_member0_checkpoint_state_ground_items_item_tile array

and save39_member0_checkpoint_state_campaign_member1_evacuated =
  [ `Undefined | `Array of save39_member0_checkpoint_state_campaign_member1_evacuated_member1 ]

and save39_member0_checkpoint_state_campaign_member1_evacuated_member1 =
  save39_member0_checkpoint_state_campaign_member1_evacuated_member1_item array

and save39_member0_checkpoint_state_campaign_member1_evacuated_member1_item = {
  wire : wire;
  event : json_string;
  unit_ : save39_member0_checkpoint_state_units_item;
}

and fragments_native_state38_campaign_member1_veterans =
  [ `Undefined | `Array of fragments_native_state38_campaign_member1_veterans_member1 ]

and fragments_native_state38_campaign_member1_veterans_member1 =
  fragments_native_state38_campaign_member1_veterans_member1_item array

and fragments_native_state38_campaign_member1_veterans_member1_item = {
  wire : wire;
  items : fragments_native_state38_units_item_items_member1 option;
  type_ : json_string;
  experience : json_number;
  character : json_string option;
  role : fragments_native_state38_units_item_role option;
}

and save39_member0_setup = {
  wire : wire;
  first : save35_setup_first;
  enemy : save39_member0_setup_enemy;
}

and save39_member0_setup_enemy = {
  wire : wire;
  blocks : save35_setup_enemy_blocks_member1 option;
  policy : save35_setup_enemy_policy;
  diplomacy : save35_setup_enemy_diplomacy;
  recruitment_limit : json_number option;
}

and save39_member1 = {
  wire : wire;
  pending_enemy : literal_false;
  undo : json_undefined option;
  progress : save39_member1_progress;
  version : literal_39;
  checkpoint : save39_member0_checkpoint;
  setup : save39_member0_setup;
}

and save39_member1_progress = { wire : wire; entry : history4_entries_item }
and history1 = { wire : wire; version : literal_1; entries : history1_entries }
and literal_1 = [ `V1 ]
and history1_entries = history1_entries_item array
and history2 = { wire : wire; version : literal_2; entries : history2_entries }
and history2_entries = history2_entries_item array

and history2_entries_item = {
  wire : wire;
  id : json_string;
  saved_at : json_number;
  run : json_string;
  mission : json_string;
  round : json_number;
  side : json_string;
  mission_start : json_boolean;
  run_id : json_string;
  branch_id : json_string;
  checkpoint_id : json_string;
  ordinal : json_number;
  kind : literal_turn;
  parent_id : json_string option;
}

and history3 = { wire : wire; version : literal_3; entries : history3_entries }
and history3_entries = history3_entries_item array

and history3_entries_item = {
  wire : wire;
  id : json_string;
  saved_at : json_number;
  run : json_string;
  mission : json_string;
  round : json_number;
  side : json_string;
  mission_start : json_boolean;
  run_id : json_string;
  branch_id : json_string;
  checkpoint_id : json_string;
  ordinal : json_number;
  kind : history3_entries_item_kind;
  parent_id : json_string option;
}

and history3_entries_item_kind = [ `Turn | `Manual ]
and history4 = { wire : wire; version : literal_4; entries : history4_entries }
and history4_entries = history4_entries_item array

and fragments = {
  wire : wire;
  native_state38 : fragments_native_state38;
  factions7 : fragments_factions7;
  factions11 : fragments_factions7;
  factions21 : fragments_factions7;
  factions24 : fragments_factions7;
  tiles11 : fragments_tiles11;
  rule11 : fragments_rule11;
  delta19 : fragments_delta19;
  delta20 : fragments_delta19;
  delta21 : fragments_delta19;
  light23 : fragments_light23;
  light28 : fragments_light28;
}

and fragments_native_state38 = {
  wire : wire;
  factions : fragments_native_state38_factions;
  ground_items : fragments_native_state38_ground_items;
  armies : fragments_native_state38_armies_member1 option;
  condition_memory : fragments_native_state38_log option;
  map_id : json_string;
  map_revision : json_number;
  cols : json_number;
  rows : json_number;
  round : json_number;
  turn : save35_setup_first;
  gold : fragments_native_state38_gold;
  winner : fragments_native_state38_winner;
  next_id : json_number;
  combat_rng : json_number;
  log : fragments_native_state38_log;
  tiles : fragments_native_state38_tiles;
  units : fragments_native_state38_units;
  campaign : fragments_native_state38_campaign_member1 option;
  last_combat : fragments_native_state38_last_combat option;
}

and fragments_native_state38_factions = fragments_native_state38_factions_item array

and fragments_native_state38_factions_item = {
  wire : wire;
  id : json_string;
  phase : save35_setup_first;
  controller : fragments_factions7_item_controller;
  palette : json_string;
  name : json_string option;
  recruits : fragments_factions7_item_recruits_member1 option;
  relations : fragments_factions7_item_relations_member1 option;
  transfer : fragments_native_state38_factions_item_transfer_member1 option;
}

and fragments_native_state38_factions_item_transfer =
  [ `Undefined | `Object2 of fragments_native_state38_factions_item_transfer_member1 ]

and fragments_native_state38_factions_item_transfer_member1 = {
  wire : wire;
  to_ : json_string;
  when_ : fragments_native_state38_tiles_item_fog_member1;
}

and fragments_native_state38_tiles_item_fog_member1 = {
  wire : wire;
  condition : fragments_native_state38_tiles_item_fog_member1_condition;
  remember : fragments_rule11_remember option;
}

and fragments_native_state38_tiles_item_fog_member1_condition =
  [ `Owner of fragments_rule11_condition_member0
  | `Proximity of fragments_light28_member1_when__member1_condition_member1
  | `Visible_units of fragments_rule11_condition_member1
  | `Has_units of fragments_light28_member1_when__member1_condition_member3
  | `No_units of fragments_light28_member1_when__member1_condition_member4
  | `Choice of fragments_rule11_condition_member2
  | `Flag of fragments_rule11_condition_member3
  | `Event of fragments_rule11_condition_member4
  | `Round of fragments_rule11_condition_member5
  | `All of fragments_native_state38_tiles_item_fog_member1_condition_member9
  | `Any of fragments_native_state38_tiles_item_fog_member1_condition_member10 ]

and fragments_native_state38_tiles_item_fog_member1_condition_member9 = {
  wire : wire;
  kind : literal_all;
  conditions : fragments_native_state38_tiles_item_fog_member1_condition_member9_conditions;
}

and fragments_native_state38_tiles_item_fog_member1_condition_member9_conditions =
  fragments_native_state38_tiles_item_fog_member1_condition array

and fragments_native_state38_tiles_item_fog_member1_condition_member10 = {
  wire : wire;
  kind : literal_any;
  conditions : fragments_native_state38_tiles_item_fog_member1_condition_member9_conditions;
}

and fragments_native_state38_tiles = fragments_native_state38_tiles_item array

and fragments_native_state38_tiles_item = {
  wire : wire;
  id : json_string;
  location : json_string option;
  col : json_number;
  row : json_number;
  terrain : json_string;
  owner : fragments_tiles11_item_owner;
  connections : fragments_tiles11_item_connections_member1 option;
  dead_ends : fragments_tiles11_item_connections_member1 option;
  walls : fragments_tiles11_item_connections_member1 option;
  floor : json_string option;
  fog : fragments_native_state38_tiles_item_fog_member1 option;
  light : fragments_native_state38_tiles_item_light_member1 option;
}

and fragments_native_state38_tiles_item_fog =
  [ `Undefined | `Object2 of fragments_native_state38_tiles_item_fog_member1 ]

and fragments_native_state38_tiles_item_light =
  [ `Undefined | `Object2 of fragments_native_state38_tiles_item_light_member1 ]

and fragments_native_state38_tiles_item_light_member1 = {
  wire : wire;
  color : fragments_light23_color_member1 option;
  fixture : fragments_rule11_remember option;
  dx : json_number option;
  dy : json_number option;
  radius : json_number;
  power : json_number;
  when_ : fragments_native_state38_tiles_item_fog_member1 option;
}

and fragments_native_state38_units = fragments_native_state38_units_item array

and fragments_native_state38_units_item = {
  wire : wire;
  faction : json_string option;
  items : fragments_native_state38_units_item_items_member1 option;
  ai_group : json_string option;
  id : json_string;
  type_ : json_string;
  side : save35_setup_first;
  tile : json_string;
  hp : json_number;
  moved : json_boolean;
  attacked : json_boolean;
  experience : json_number option;
  character : json_string option;
  role : fragments_native_state38_units_item_role option;
  retreating : fragments_rule11_remember option;
  rebuilding : fragments_rule11_remember option;
}

and fragments_native_state38_campaign =
  [ `Undefined | `Object2 of fragments_native_state38_campaign_member1 ]

and fragments_native_state38_campaign_member1 = {
  wire : wire;
  scripts : fragments_native_state38_campaign_member1_scripts_member1 option;
  evacuated : fragments_native_state38_campaign_member1_evacuated_member1 option;
  id : json_string;
  mission : json_number;
  recall : fragments_native_state38_log;
  veterans : fragments_native_state38_campaign_member1_veterans_member1 option;
  story_veterans : fragments_native_state38_campaign_member1_veterans_member1 option;
  fallen_companions : fragments_native_state38_log option;
  carry_gold : json_number;
  marks : fragments_native_state38_log;
  dialogue : fragments_native_state38_campaign_member1_dialogue;
  dialogue_index : json_number;
  outcome : fragments_tiles11_item_owner;
  completed : json_boolean;
  refuge : fragments_native_state38_campaign_member1_refuge_member1 option;
  vigil : fragments_native_state38_campaign_member1_vigil_member1 option;
  holds : fragments_native_state38_campaign_member1_holds_member1 option;
  flags : fragments_native_state38_campaign_member1_flags;
}

and fragments_native_state38_campaign_member1_scripts =
  [ `Undefined | `Object2 of fragments_native_state38_campaign_member1_scripts_member1 ]

and fragments_native_state38_campaign_member1_scripts_member1 = {
  wire : wire;
  failed : fragments_native_state38_log option;
  completed : fragments_native_state38_log;
  revealed : fragments_native_state38_log;
  recruitment : fragments_native_state38_campaign_member1_scripts_member1_recruitment;
  active : fragments_native_state38_log;
  holds : fragments_native_state38_campaign_member1_scripts_member1_holds;
  marks : fragments_native_state38_campaign_member1_scripts_member1_marks;
  pending : fragments_native_state38_campaign_member1_scripts_member1_pending;
  next_mark : json_number;
}

and fragments_native_state38_campaign_member1_scripts_member1_marks =
  fragments_native_state38_campaign_member1_scripts_member1_marks_item array

and fragments_native_state38_campaign_member1_scripts_member1_marks_item = {
  wire : wire;
  light_remembered : fragments_rule11_remember option;
  id : json_number;
  type_ : json_string;
  tile : json_string;
  light : fragments_native_state38_tiles_item_light_member1 option;
}

and fragments_native_state38_campaign_member1_evacuated =
  [ `Undefined | `Array of fragments_native_state38_campaign_member1_evacuated_member1 ]

and fragments_native_state38_campaign_member1_evacuated_member1 =
  fragments_native_state38_campaign_member1_evacuated_member1_item array

and fragments_native_state38_campaign_member1_evacuated_member1_item = {
  wire : wire;
  event : json_string;
  unit_ : fragments_native_state38_units_item;
}

and fragments_delta19 = fragments_delta19_item array

and fragments_delta19_item = {
  wire : wire;
  owner : fragments_delta19_item_owner option;
  id : json_string;
  location : json_string option;
  col : json_number option;
  row : json_number option;
  terrain : json_string option;
  connections : fragments_tiles11_item_connections_member1 option;
  dead_ends : fragments_tiles11_item_connections_member1 option;
  walls : fragments_tiles11_item_connections_member1 option;
  floor : json_string option;
  fog : fragments_rule11 option;
  light : fragments_light23 option;
}

and fragments_light28 = [ `Null | `Object2 of fragments_light28_member1 ]

and native1 = {
  wire : wire;
  engine : literal_rust_wasm;
  version : literal_1 option;
  pending_enemy : json_boolean;
  saved : native1_saved;
}

and literal_rust_wasm = [ `Rust_wasm ]
and native1_version = [ `Undefined | `V1 ]

and native1_saved = {
  wire : wire;
  seed : json_number;
  state : save2_checkpoint_state;
  version : literal_1;
}

and javascript1 = {
  wire : wire;
  version : literal_1;
  pending_enemy : json_boolean;
  campaign_deployment_format : literal_named_waves_v1 option;
  state : javascript1_state;
}

and javascript1_campaign_deployment_format = [ `Undefined | `Named_waves_v1 ]
and literal_named_waves_v1 = [ `Named_waves_v1 ]

and javascript1_state = {
  wire : wire;
  map_id : json_string option;
  map_revision : javascript1_state_map_revision option;
  round : json_number;
  turn : save35_setup_first;
  winner : fragments_native_state38_winner;
  gold : javascript1_state_gold;
  next_id : json_number;
  log : fragments_native_state38_log;
  units : javascript1_state_units;
  villages : javascript1_state_villages;
  sites : javascript1_state_villages option;
  campaign : javascript1_state_campaign_member1 option;
  last_combat : fragments_native_state38_last_combat option;
}

and javascript1_state_map_revision = [ `Undefined | `Null | `Number of json_number ]
and javascript1_state_units = javascript1_state_units_item array

and javascript1_state_units_item = {
  wire : wire;
  tile : json_string;
  id : json_string;
  side : save35_setup_first;
  name : json_string option;
  role : fragments_native_state38_units_item_role option;
  character : json_string option;
  faction : json_string option;
  items : javascript1_state_units_item_items_member1 option;
  ai_group : json_string option;
  type_ : javascript1_state_units_item_type_;
  hp : json_number;
  moved : json_boolean;
  attacked : json_boolean;
  retreating : fragments_rule11_remember option;
  rebuilding : fragments_rule11_remember option;
  experience : json_number option;
}

and javascript1_state_units_item_items =
  [ `Undefined | `Array of javascript1_state_units_item_items_member1 ]

and javascript1_state_units_item_items_member1 =
  javascript1_state_units_item_items_member1_item array

and javascript1_state_units_item_items_member1_item = {
  wire : wire;
  id : json_string;
  armor : fragments_native_state38_ground_items_item_item_armor_member1 option;
  name : json_string;
  description : json_string;
  healing : fragments_native_state38_ground_items_item_item_healing_member1 option;
  attacks : javascript1_state_units_item_items_member1_item_attacks;
}

and javascript1_state_units_item_items_member1_item_attacks =
  javascript1_state_units_item_items_member1_item_attacks_item array

and javascript1_state_units_item_items_member1_item_attacks_item = {
  wire : wire;
  spread : json_number option;
  drain : json_number option;
  count : json_number;
  id : json_string;
  name : json_string;
  damage : json_number;
  damage_type : fragments_native_state38_ground_items_item_item_attacks_item_damage_type;
  category : fragments_native_state38_ground_items_item_item_attacks_item_category;
  range : json_number;
  sound : json_string;
}

and javascript1_state_units_item_type_ =
  [ `Ranger
  | `Healer
  | `Briarweaver
  | `Deatharcher
  | `Direwolf
  | `Greatboar
  | `Wraith
  | `Spectre
  | `Lancer
  | `Mage
  | `Marksman
  | `Merfolk_maceguard_l2
  | `Revenant
  | `Skeleton_impact_caster_l2
  | `Soldier
  | `Tidehunter
  | `Tidesage
  | `Tridentguard
  | `Warden
  | `Champion
  | `Hammerman
  | `Warhammer
  | `Sentinel
  | `Halberdier
  | `Halberd_master
  | `Shadowblade
  | `Slinger
  | `Fireslinger
  | `Sharpshooter
  | `Crossbowman
  | `Arbalester
  | `Archmage
  | `High_healer
  | `Knight
  | `Horse_bowman
  | `Horse_marksman
  | `Skeleton_dreadguard
  | `Skeleton_flanker_l2
  | `Skeleton_flanker_l3
  | `Skeleton_impact_caster_l3
  | `Skeleton_scythe_caster_l2
  | `Skeleton_scythe_caster_l3
  | `Skeleton_rider_l2
  | `Skeleton_rider_l3
  | `Skeleton_bow_master
  | `Skeleton_axeman_l2
  | `Skeleton_axeman_l3
  | `Woodland_alpha_wolf
  | `Woodland_ancient_boar
  | `Woodland_thorn_oracle
  | `Woodland_bramble_warden
  | `Woodland_thornbinder
  | `Fox_spearguard_l2
  | `Fox_spearguard_l3
  | `Treant_l2
  | `Treant_l3
  | `Merfolk_harpoon_master
  | `Merfolk_netter_l2
  | `Merfolk_netter_l3
  | `Merfolk_reef_champion
  | `Merfolk_trident_l2
  | `Merfolk_trident_l3
  | `Merfolk_tide_oracle
  | `Merfolk_tidecrusher_l2
  | `Merfolk_tidecrusher_l3
  | `Merfolk_maceguard_l3
  | `Dragon
  | `Skeletal_hound
  | `Captives
  | `Zombie_walker
  | `Zombie_decay_caster
  | `Bear
  | `Giant_bat
  | `Giant_spider
  | `Commander
  | `Grovewarden
  | `Lich
  | `Marshal
  | `Necromancer
  | `Tidekeeper
  | `Tideseer
  | `Wildkeeper
  | `Grand_marshal
  | `Lich_commander_l3
  | `Woodland_grove_oracle
  | `Merfolk_tide_sovereign
  | `Adept
  | `Archer
  | `Boar
  | `Bonearcher
  | `Ghost
  | `Harpooner
  | `Merfolk_maceguard
  | `Reefguard
  | `Rider
  | `Scout
  | `Skeleton_impact_caster
  | `Skeleton
  | `Spearman
  | `Swordsman
  | `Thorncaller
  | `Wavemender
  | `Wolf
  | `Horse_archer
  | `Skeleton_rider
  | `Skeleton_axeman
  | `Fox_spearguard
  | `Treant ]

and literal_ranger = [ `Ranger ]
and literal_healer = [ `Healer ]
and literal_briarweaver = [ `Briarweaver ]
and literal_deatharcher = [ `Deatharcher ]
and literal_direwolf = [ `Direwolf ]
and literal_greatboar = [ `Greatboar ]
and literal_wraith = [ `Wraith ]
and literal_spectre = [ `Spectre ]
and literal_lancer = [ `Lancer ]
and literal_mage = [ `Mage ]
and literal_marksman = [ `Marksman ]
and literal_merfolk_maceguard_l2 = [ `Merfolk_maceguard_l2 ]
and literal_revenant = [ `Revenant ]
and literal_skeleton_impact_caster_l2 = [ `Skeleton_impact_caster_l2 ]
and literal_soldier = [ `Soldier ]
and literal_tidehunter = [ `Tidehunter ]
and literal_tidesage = [ `Tidesage ]
and literal_tridentguard = [ `Tridentguard ]
and literal_warden = [ `Warden ]
and literal_champion = [ `Champion ]
and literal_hammerman = [ `Hammerman ]
and literal_warhammer = [ `Warhammer ]
and literal_sentinel = [ `Sentinel ]
and literal_halberdier = [ `Halberdier ]
and literal_halberd_master = [ `Halberd_master ]
and literal_shadowblade = [ `Shadowblade ]
and literal_slinger = [ `Slinger ]
and literal_fireslinger = [ `Fireslinger ]
and literal_sharpshooter = [ `Sharpshooter ]
and literal_crossbowman = [ `Crossbowman ]
and literal_arbalester = [ `Arbalester ]
and literal_archmage = [ `Archmage ]
and literal_high_healer = [ `High_healer ]
and literal_knight = [ `Knight ]
and literal_horse_bowman = [ `Horse_bowman ]
and literal_horse_marksman = [ `Horse_marksman ]
and literal_skeleton_dreadguard = [ `Skeleton_dreadguard ]
and literal_skeleton_flanker_l2 = [ `Skeleton_flanker_l2 ]
and literal_skeleton_flanker_l3 = [ `Skeleton_flanker_l3 ]
and literal_skeleton_impact_caster_l3 = [ `Skeleton_impact_caster_l3 ]
and literal_skeleton_scythe_caster_l2 = [ `Skeleton_scythe_caster_l2 ]
and literal_skeleton_scythe_caster_l3 = [ `Skeleton_scythe_caster_l3 ]
and literal_skeleton_rider_l2 = [ `Skeleton_rider_l2 ]
and literal_skeleton_rider_l3 = [ `Skeleton_rider_l3 ]
and literal_skeleton_bow_master = [ `Skeleton_bow_master ]
and literal_skeleton_axeman_l2 = [ `Skeleton_axeman_l2 ]
and literal_skeleton_axeman_l3 = [ `Skeleton_axeman_l3 ]
and literal_woodland_alpha_wolf = [ `Woodland_alpha_wolf ]
and literal_woodland_ancient_boar = [ `Woodland_ancient_boar ]
and literal_woodland_thorn_oracle = [ `Woodland_thorn_oracle ]
and literal_woodland_bramble_warden = [ `Woodland_bramble_warden ]
and literal_woodland_thornbinder = [ `Woodland_thornbinder ]
and literal_fox_spearguard_l2 = [ `Fox_spearguard_l2 ]
and literal_fox_spearguard_l3 = [ `Fox_spearguard_l3 ]
and literal_treant_l2 = [ `Treant_l2 ]
and literal_treant_l3 = [ `Treant_l3 ]
and literal_merfolk_harpoon_master = [ `Merfolk_harpoon_master ]
and literal_merfolk_netter_l2 = [ `Merfolk_netter_l2 ]
and literal_merfolk_netter_l3 = [ `Merfolk_netter_l3 ]
and literal_merfolk_reef_champion = [ `Merfolk_reef_champion ]
and literal_merfolk_trident_l2 = [ `Merfolk_trident_l2 ]
and literal_merfolk_trident_l3 = [ `Merfolk_trident_l3 ]
and literal_merfolk_tide_oracle = [ `Merfolk_tide_oracle ]
and literal_merfolk_tidecrusher_l2 = [ `Merfolk_tidecrusher_l2 ]
and literal_merfolk_tidecrusher_l3 = [ `Merfolk_tidecrusher_l3 ]
and literal_merfolk_maceguard_l3 = [ `Merfolk_maceguard_l3 ]
and literal_dragon = [ `Dragon ]
and literal_skeletal_hound = [ `Skeletal_hound ]
and literal_captives = [ `Captives ]
and literal_zombie_walker = [ `Zombie_walker ]
and literal_zombie_decay_caster = [ `Zombie_decay_caster ]
and literal_bear = [ `Bear ]
and literal_giant_bat = [ `Giant_bat ]
and literal_giant_spider = [ `Giant_spider ]
and literal_commander = [ `Commander ]
and literal_grovewarden = [ `Grovewarden ]
and literal_lich = [ `Lich ]
and literal_marshal = [ `Marshal ]
and literal_necromancer = [ `Necromancer ]
and literal_tidekeeper = [ `Tidekeeper ]
and literal_tideseer = [ `Tideseer ]
and literal_wildkeeper = [ `Wildkeeper ]
and literal_grand_marshal = [ `Grand_marshal ]
and literal_lich_commander_l3 = [ `Lich_commander_l3 ]
and literal_woodland_grove_oracle = [ `Woodland_grove_oracle ]
and literal_merfolk_tide_sovereign = [ `Merfolk_tide_sovereign ]
and literal_adept = [ `Adept ]
and literal_archer = [ `Archer ]
and literal_boar = [ `Boar ]
and literal_bonearcher = [ `Bonearcher ]
and literal_ghost = [ `Ghost ]
and literal_harpooner = [ `Harpooner ]
and literal_merfolk_maceguard = [ `Merfolk_maceguard ]
and literal_reefguard = [ `Reefguard ]
and literal_rider = [ `Rider ]
and literal_scout = [ `Scout ]
and literal_skeleton_impact_caster = [ `Skeleton_impact_caster ]
and literal_skeleton = [ `Skeleton ]
and literal_spearman = [ `Spearman ]
and literal_swordsman = [ `Swordsman ]
and literal_thorncaller = [ `Thorncaller ]
and literal_wavemender = [ `Wavemender ]
and literal_wolf = [ `Wolf ]
and literal_horse_archer = [ `Horse_archer ]
and literal_skeleton_rider = [ `Skeleton_rider ]
and literal_skeleton_axeman = [ `Skeleton_axeman ]
and literal_fox_spearguard = [ `Fox_spearguard ]
and literal_treant = [ `Treant ]
and javascript1_state_villages = javascript1_state_villages_item array

and javascript1_state_villages_item = {
  wire : wire;
  id : json_string;
  owner : fragments_tiles11_item_owner;
}

and javascript1_state_sites = [ `Undefined | `Array of javascript1_state_villages ]
and javascript1_state_campaign = [ `Undefined | `Object2 of javascript1_state_campaign_member1 ]

and javascript1_state_campaign_member1 = {
  wire : wire;
  id : javascript1_state_campaign_member1_id;
  mission : json_number;
  recall : javascript1_state_campaign_member1_recall;
  veterans : javascript1_state_campaign_member1_veterans_member1 option;
  story_veterans : javascript1_state_campaign_member1_veterans_member1 option;
  fallen_companions : fragments_native_state38_log option;
  carry_gold : json_number;
  marks : fragments_native_state38_log;
  dialogue : javascript1_state_campaign_member1_dialogue;
  dialogue_index : json_number;
  outcome : fragments_tiles11_item_owner;
  completed : json_boolean;
  refuge : fragments_native_state38_campaign_member1_refuge_member1 option;
  vigil : fragments_native_state38_campaign_member1_vigil_member1 option;
  holds : fragments_native_state38_campaign_member1_holds_member1 option;
  choices : fragments_native_state38_log option;
}

and javascript1_state_campaign_member1_id =
  [ `Lantern_road | `Green_accord | `Ashen_oath | `Borrowed_tide | `Mirror_night | `Bell_night ]

and literal_lantern_road = [ `Lantern_road ]
and literal_green_accord = [ `Green_accord ]
and literal_ashen_oath = [ `Ashen_oath ]
and literal_borrowed_tide = [ `Borrowed_tide ]
and literal_mirror_night = [ `Mirror_night ]
and literal_bell_night = [ `Bell_night ]
and javascript1_state_campaign_member1_recall = javascript1_state_units_item_type_ array

and javascript1_state_campaign_member1_veterans =
  [ `Undefined | `Array of javascript1_state_campaign_member1_veterans_member1 ]

and javascript1_state_campaign_member1_veterans_member1 =
  javascript1_state_campaign_member1_veterans_member1_item array

and javascript1_state_campaign_member1_veterans_member1_item = {
  wire : wire;
  type_ : javascript1_state_units_item_type_;
  experience : json_number;
  name : json_string option;
  role : fragments_native_state38_units_item_role option;
}

and javascript1_state_campaign_member1_dialogue =
  javascript1_state_campaign_member1_dialogue_item array

and javascript1_state_campaign_member1_dialogue_item = {
  wire : wire;
  sound : json_string option;
  speaker : json_string;
  character : json_string option;
  portrait : javascript1_state_campaign_member1_dialogue_item_portrait;
  text : json_string;
}

and javascript1_state_campaign_member1_dialogue_item_portrait =
  [ `Ranger
  | `Healer
  | `Briarweaver
  | `Deatharcher
  | `Direwolf
  | `Greatboar
  | `Wraith
  | `Spectre
  | `Lancer
  | `Mage
  | `Marksman
  | `Merfolk_maceguard_l2
  | `Revenant
  | `Skeleton_impact_caster_l2
  | `Soldier
  | `Tidehunter
  | `Tidesage
  | `Tridentguard
  | `Warden
  | `Champion
  | `Hammerman
  | `Warhammer
  | `Sentinel
  | `Halberdier
  | `Halberd_master
  | `Shadowblade
  | `Slinger
  | `Fireslinger
  | `Sharpshooter
  | `Crossbowman
  | `Arbalester
  | `Archmage
  | `High_healer
  | `Knight
  | `Horse_bowman
  | `Horse_marksman
  | `Skeleton_dreadguard
  | `Skeleton_flanker_l2
  | `Skeleton_flanker_l3
  | `Skeleton_impact_caster_l3
  | `Skeleton_scythe_caster_l2
  | `Skeleton_scythe_caster_l3
  | `Skeleton_rider_l2
  | `Skeleton_rider_l3
  | `Skeleton_bow_master
  | `Skeleton_axeman_l2
  | `Skeleton_axeman_l3
  | `Woodland_alpha_wolf
  | `Woodland_ancient_boar
  | `Woodland_thorn_oracle
  | `Woodland_bramble_warden
  | `Woodland_thornbinder
  | `Fox_spearguard_l2
  | `Fox_spearguard_l3
  | `Treant_l2
  | `Treant_l3
  | `Merfolk_harpoon_master
  | `Merfolk_netter_l2
  | `Merfolk_netter_l3
  | `Merfolk_reef_champion
  | `Merfolk_trident_l2
  | `Merfolk_trident_l3
  | `Merfolk_tide_oracle
  | `Merfolk_tidecrusher_l2
  | `Merfolk_tidecrusher_l3
  | `Merfolk_maceguard_l3
  | `Dragon
  | `Skeletal_hound
  | `Captives
  | `Zombie_walker
  | `Zombie_decay_caster
  | `Bear
  | `Giant_bat
  | `Giant_spider
  | `Commander
  | `Grovewarden
  | `Lich
  | `Marshal
  | `Necromancer
  | `Tidekeeper
  | `Tideseer
  | `Wildkeeper
  | `Grand_marshal
  | `Lich_commander_l3
  | `Woodland_grove_oracle
  | `Merfolk_tide_sovereign
  | `Adept
  | `Archer
  | `Boar
  | `Bonearcher
  | `Ghost
  | `Harpooner
  | `Merfolk_maceguard
  | `Reefguard
  | `Rider
  | `Scout
  | `Skeleton_impact_caster
  | `Skeleton
  | `Spearman
  | `Swordsman
  | `Thorncaller
  | `Wavemender
  | `Wolf
  | `Horse_archer
  | `Skeleton_rider
  | `Skeleton_axeman
  | `Fox_spearguard
  | `Treant
  | `Worker ]

and literal_worker = [ `Worker ]
and save40 = [ `Current of save40_member0 | `Turn of save40_member1 ]

and save40_member0 = {
  wire : wire;
  pending_enemy : json_boolean;
  undo : save34_undo_member1;
  progress : save39_member0_progress;
  version : literal_40;
  checkpoint : save40_member0_checkpoint;
  setup : save39_member0_setup;
}

and literal_40 = [ `V40 ]

and save40_member0_checkpoint = {
  wire : wire;
  seed : json_number;
  state : save40_member0_checkpoint_state;
}

and save40_member0_checkpoint_state = {
  wire : wire;
  factions : save40_member0_checkpoint_state_factions;
  ground_items : save39_member0_checkpoint_state_ground_items;
  armies : fragments_native_state38_armies_member1 option;
  condition_memory : fragments_native_state38_log option;
  map_id : json_string;
  map_revision : json_number;
  cols : json_number;
  rows : json_number;
  round : json_number;
  turn : save35_setup_first;
  gold : fragments_native_state38_gold;
  winner : fragments_native_state38_winner;
  next_id : json_number;
  combat_rng : json_number;
  log : fragments_native_state38_log;
  tiles : save40_member0_checkpoint_state_tiles;
  units : save39_member0_checkpoint_state_units;
  campaign : save40_member0_checkpoint_state_campaign_member1 option;
  last_combat : fragments_native_state38_last_combat option;
}

and save40_member0_checkpoint_state_factions = save40_member0_checkpoint_state_factions_item array

and save40_member0_checkpoint_state_factions_item = {
  wire : wire;
  id : json_string;
  phase : save35_setup_first;
  controller : fragments_factions7_item_controller;
  palette : json_string;
  name : json_string option;
  recruits : fragments_factions7_item_recruits_member1 option;
  relations : fragments_factions7_item_relations_member1 option;
  transfer : save40_member0_checkpoint_state_factions_item_transfer_member1 option;
  vision : save40_member0_checkpoint_state_factions_item_vision_member1 option;
}

and save40_member0_checkpoint_state_factions_item_transfer =
  [ `Undefined | `Object2 of save40_member0_checkpoint_state_factions_item_transfer_member1 ]

and save40_member0_checkpoint_state_factions_item_transfer_member1 = {
  wire : wire;
  to_ : json_string;
  when_ : save40_member0_checkpoint_state_tiles_item_fog_member1;
}

and save40_member0_checkpoint_state_tiles_item_fog_member1 = {
  wire : wire;
  condition : save40_member0_checkpoint_state_tiles_item_fog_member1_condition;
  remember : fragments_rule11_remember option;
}

and save40_member0_checkpoint_state_tiles_item_fog_member1_condition =
  [ `Owner of save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0
  | `Proximity of save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member1
  | `Visible_units of save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member2
  | `Has_units of save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member3
  | `No_units of save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member4
  | `Choice of fragments_rule11_condition_member2
  | `Flag of fragments_rule11_condition_member3
  | `Event of fragments_rule11_condition_member4
  | `Round of fragments_rule11_condition_member5
  | `All of save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member9
  | `Any of save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member10 ]

and save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member1 = {
  wire : wire;
  kind : literal_proximity;
  unit_types : fragments_native_state38_log option;
  tile : save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile option;
  radius : json_number;
  side : save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member1_side;
  unit_type : json_string option;
  character : json_string option;
}

and save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member1_side =
  [ `Player | `Enemy | `Neutral | `Any | `Observer ]

and literal_observer = [ `Observer ]

and save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member2 = {
  wire : wire;
  kind : literal_visible_units;
  side : save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member1_side;
  unit_types : fragments_native_state38_log option;
}

and save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member3 = {
  wire : wire;
  kind : literal_has_units;
  ai_group : json_string option;
  side : save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member1_side;
  unit_type : json_string option;
  character : json_string option;
}

and save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member4 = {
  wire : wire;
  kind : literal_no_units;
  side : save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member1_side;
  unit_type : json_string option;
  character : json_string option;
}

and save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member9 = {
  wire : wire;
  kind : literal_all;
  conditions : save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member9_conditions;
}

and save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member9_conditions =
  save40_member0_checkpoint_state_tiles_item_fog_member1_condition array

and save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member10 = {
  wire : wire;
  kind : literal_any;
  conditions : save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member9_conditions;
}

and save40_member0_checkpoint_state_tiles = save40_member0_checkpoint_state_tiles_item array

and save40_member0_checkpoint_state_tiles_item = {
  wire : wire;
  id : json_string;
  location : json_string option;
  col : json_number;
  row : json_number;
  terrain : json_string;
  owner : fragments_tiles11_item_owner;
  connections : fragments_tiles11_item_connections_member1 option;
  dead_ends : fragments_tiles11_item_connections_member1 option;
  walls : fragments_tiles11_item_connections_member1 option;
  floor : json_string option;
  fog : save40_member0_checkpoint_state_tiles_item_fog_member1 option;
  light : save40_member0_checkpoint_state_tiles_item_light_member1 option;
}

and save40_member0_checkpoint_state_tiles_item_fog =
  [ `Undefined | `Object2 of save40_member0_checkpoint_state_tiles_item_fog_member1 ]

and save40_member0_checkpoint_state_tiles_item_light =
  [ `Undefined | `Object2 of save40_member0_checkpoint_state_tiles_item_light_member1 ]

and save40_member0_checkpoint_state_tiles_item_light_member1 = {
  wire : wire;
  color : fragments_light23_color_member1 option;
  fixture : fragments_rule11_remember option;
  dx : json_number option;
  dy : json_number option;
  radius : json_number;
  power : json_number;
  when_ : save40_member0_checkpoint_state_tiles_item_fog_member1 option;
}

and save40_member0_checkpoint_state_campaign =
  [ `Undefined | `Object2 of save40_member0_checkpoint_state_campaign_member1 ]

and save40_member0_checkpoint_state_campaign_member1 = {
  wire : wire;
  scripts : save40_member0_checkpoint_state_campaign_member1_scripts_member1 option;
  evacuated : save39_member0_checkpoint_state_campaign_member1_evacuated_member1 option;
  id : json_string;
  mission : json_number;
  recall : fragments_native_state38_log;
  veterans : fragments_native_state38_campaign_member1_veterans_member1 option;
  story_veterans : fragments_native_state38_campaign_member1_veterans_member1 option;
  fallen_companions : fragments_native_state38_log option;
  carry_gold : json_number;
  marks : fragments_native_state38_log;
  dialogue : fragments_native_state38_campaign_member1_dialogue;
  dialogue_index : json_number;
  outcome : fragments_tiles11_item_owner;
  completed : json_boolean;
  refuge : fragments_native_state38_campaign_member1_refuge_member1 option;
  vigil : fragments_native_state38_campaign_member1_vigil_member1 option;
  holds : fragments_native_state38_campaign_member1_holds_member1 option;
  flags : fragments_native_state38_campaign_member1_flags;
}

and save40_member0_checkpoint_state_campaign_member1_scripts =
  [ `Undefined | `Object2 of save40_member0_checkpoint_state_campaign_member1_scripts_member1 ]

and save40_member0_checkpoint_state_campaign_member1_scripts_member1 = {
  wire : wire;
  failed : fragments_native_state38_log option;
  completed : fragments_native_state38_log;
  revealed : fragments_native_state38_log;
  recruitment : fragments_native_state38_campaign_member1_scripts_member1_recruitment;
  active : fragments_native_state38_log;
  holds : fragments_native_state38_campaign_member1_scripts_member1_holds;
  marks : save40_member0_checkpoint_state_campaign_member1_scripts_member1_marks;
  pending : save39_member0_checkpoint_state_campaign_member1_scripts_member1_pending;
  next_mark : json_number;
}

and save40_member0_checkpoint_state_campaign_member1_scripts_member1_marks =
  save40_member0_checkpoint_state_campaign_member1_scripts_member1_marks_item array

and save40_member0_checkpoint_state_campaign_member1_scripts_member1_marks_item = {
  wire : wire;
  light_remembered : fragments_rule11_remember option;
  id : json_number;
  type_ : json_string;
  tile : save39_member0_checkpoint_state_ground_items_item_tile;
  light : save40_member0_checkpoint_state_tiles_item_light_member1 option;
}

and save40_member1 = {
  wire : wire;
  pending_enemy : literal_false;
  undo : json_undefined option;
  progress : save39_member1_progress;
  version : literal_40;
  checkpoint : save40_member0_checkpoint;
  setup : save39_member0_setup;
}

and save40_member0_checkpoint_state_factions_item_vision_member1 = {
  wire : wire;
  explored : save39_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item_area;
}

and save40_member0_checkpoint_state_factions_item_vision =
  [ `Undefined | `Object2 of save40_member0_checkpoint_state_factions_item_vision_member1 ]

and save41_member0_checkpoint = {
  wire : wire;
  seed : json_number;
  state : save41_member0_checkpoint_state;
}

and save41_member0_checkpoint_state = {
  wire : wire;
  vision_groups : save41_member0_checkpoint_state_vision_groups;
  factions : save41_member0_checkpoint_state_factions;
  ground_items : save41_member0_checkpoint_state_ground_items;
  armies : fragments_native_state38_armies_member1 option;
  condition_memory : fragments_native_state38_log option;
  map_id : json_string;
  map_revision : json_number;
  cols : json_number;
  rows : json_number;
  round : json_number;
  turn : save41_member0_setup_first;
  gold : fragments_native_state38_gold;
  winner : save41_member0_checkpoint_state_winner;
  next_id : json_number;
  combat_rng : json_number;
  log : fragments_native_state38_log;
  tiles : save41_member0_checkpoint_state_tiles;
  units : save41_member0_checkpoint_state_units;
  campaign : save41_member0_checkpoint_state_campaign_member1 option;
  last_combat : fragments_native_state38_last_combat option;
}

and save41_member0_checkpoint_state_vision_groups_item = {
  wire : wire;
  id : json_string;
  sight : save41_member0_checkpoint_state_vision_groups_item_sight;
  explored : save39_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item_area;
}

and save41_member0_checkpoint_state_vision_groups_item_sight = {
  wire : wire;
  range : save41_member0_checkpoint_state_vision_groups_item_sight_range;
  penetrates : save41_member0_checkpoint_state_vision_groups_item_sight_penetrates;
}

and save41_member0_checkpoint_state_vision_groups_item_sight_range =
  [ `Member1 of save41_member0_checkpoint_state_vision_groups_item_sight_range_member0
  | `Number of json_number ]

and save41_member0_checkpoint_state_vision_groups_item_sight_range_member0 =
  [ `Inherited | `Unlimited ]

and literal_inherited = [ `Inherited ]
and literal_unlimited = [ `Unlimited ]

and save41_member0_checkpoint_state_vision_groups_item_sight_penetrates_item =
  [ `Fog | `Darkness | `Forest_camouflage ]

and literal_fog = [ `Fog ]
and literal_darkness = [ `Darkness ]
and literal_forest_camouflage = [ `Forest_camouflage ]

and save41_member0_checkpoint_state_vision_groups_item_sight_penetrates =
  save41_member0_checkpoint_state_vision_groups_item_sight_penetrates_item array

and save41_member0_checkpoint_state_vision_groups =
  save41_member0_checkpoint_state_vision_groups_item array

and save41_member0_checkpoint_state_factions_item = {
  wire : wire;
  vision_group : json_string;
  id : json_string;
  phase : save41_member0_setup_first;
  controller : fragments_factions7_item_controller;
  palette : json_string;
  name : json_string option;
  recruits : fragments_factions7_item_recruits_member1 option;
  relations : save41_member0_checkpoint_state_factions_item_relations_member1 option;
  transfer : save41_member0_checkpoint_state_factions_item_transfer_member1 option;
}

and save41_member0_setup_first = [ `Player | `Ai ]

and save41_member0_checkpoint_state_factions_item_relations_member1_value =
  [ `Allied | `Neutral | `Hostile ]

and save41_member0_checkpoint_state_factions_item_relations_member1 =
  save41_member0_checkpoint_state_factions_item_relations_member1_value String_map.t

and save41_member0_checkpoint_state_factions_item_relations =
  [ `Undefined | `Object2 of save41_member0_checkpoint_state_factions_item_relations_member1 ]

and save41_member0_checkpoint_state_factions_item_transfer_member1 = {
  wire : wire;
  to_ : json_string;
  when_ : save41_member0_checkpoint_state_factions_item_transfer_member1_when_;
}

and save41_member0_checkpoint_state_factions_item_transfer_member1_when_ = {
  wire : wire;
  condition : save40_member0_checkpoint_state_tiles_item_fog_member1_condition;
  remember : json_boolean;
}

and shape_649 =
  [ `Location of
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member1
  | `Object2 of
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member2
  | `Object3 of
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member3 ]

and save41_member0_checkpoint_state_factions_item_transfer =
  [ `Undefined | `Object2 of save41_member0_checkpoint_state_factions_item_transfer_member1 ]

and save41_member0_checkpoint_state_factions = save41_member0_checkpoint_state_factions_item array

and save41_member0_checkpoint_state_ground_items_item = {
  wire : wire;
  tile : save39_member0_checkpoint_state_ground_items_item_tile;
  item : save41_member0_checkpoint_state_ground_items_item_item;
}

and save41_member0_checkpoint_state_ground_items_item_item = {
  wire : wire;
  perception : save41_member0_checkpoint_state_ground_items_item_item_perception_member1 option;
  armor : fragments_native_state38_ground_items_item_item_armor_member1 option;
  id : json_string;
  name : json_string;
  description : json_string;
  attacks : save41_member0_checkpoint_state_ground_items_item_item_attacks;
  healing : fragments_native_state38_ground_items_item_item_healing_member1 option;
}

and save41_member0_checkpoint_state_ground_items_item_item_perception_member1 = {
  wire : wire;
  range : save41_member0_checkpoint_state_vision_groups_item_sight_range;
  penetrates : save41_member0_checkpoint_state_vision_groups_item_sight_penetrates;
  concealment : save41_member0_checkpoint_state_units_item_perception_effects_item_concealment;
  light : save41_member0_checkpoint_state_tiles_item_light_member1_emission;
}

and save41_member0_checkpoint_state_units_item_perception_effects_item_concealment_item = {
  wire : wire;
  kind : save41_member0_checkpoint_state_vision_groups_item_sight_penetrates_item;
  reveal_within : json_number;
  terrains :
    save41_member0_checkpoint_state_units_item_perception_effects_item_concealment_item_terrains;
}

and save41_member0_checkpoint_state_tiles_item_perception_concealment_item = {
  wire : wire;
  kind : save41_member0_checkpoint_state_vision_groups_item_sight_penetrates_item;
  reveal_within : json_number;
}

and save41_member0_checkpoint_state_units_item_perception_effects_item_concealment_item_terrains_member1_item =
  [ `Bridge
  | `Burned_village
  | `Cave
  | `Church
  | `Ferry
  | `Ford
  | `Forest
  | `Fort
  | `Fortification
  | `Graveyard
  | `Grove
  | `Hill
  | `House
  | `Keep
  | `Landing
  | `Marsh
  | `Monastery
  | `Mountain
  | `Plains
  | `Ritual_spent
  | `Ritual_star
  | `Road
  | `Ruins
  | `Signpost
  | `Village
  | `Water ]

and literal_bridge = [ `Bridge ]
and literal_burned_village = [ `Burned_village ]
and literal_cave = [ `Cave ]
and literal_church = [ `Church ]
and literal_ferry = [ `Ferry ]
and literal_ford = [ `Ford ]
and literal_forest = [ `Forest ]
and literal_fort = [ `Fort ]
and literal_fortification = [ `Fortification ]
and literal_graveyard = [ `Graveyard ]
and literal_grove = [ `Grove ]
and literal_hill = [ `Hill ]
and literal_house = [ `House ]
and literal_keep = [ `Keep ]
and literal_landing = [ `Landing ]
and literal_marsh = [ `Marsh ]
and literal_monastery = [ `Monastery ]
and literal_mountain = [ `Mountain ]
and literal_plains = [ `Plains ]
and literal_ritual_spent = [ `Ritual_spent ]
and literal_ritual_star = [ `Ritual_star ]
and literal_road = [ `Road ]
and literal_ruins = [ `Ruins ]
and literal_signpost = [ `Signpost ]
and literal_village = [ `Village ]
and literal_water = [ `Water ]

and save41_member0_checkpoint_state_units_item_perception_effects_item_concealment_item_terrains_member1 =
  save41_member0_checkpoint_state_units_item_perception_effects_item_concealment_item_terrains_member1_item
  array

and save41_member0_checkpoint_state_units_item_perception_effects_item_concealment_item_terrains =
  [ `Null
  | `Array of
    save41_member0_checkpoint_state_units_item_perception_effects_item_concealment_item_terrains_member1
  ]

and save41_member0_checkpoint_state_units_item_perception_effects_item_concealment =
  save41_member0_checkpoint_state_units_item_perception_effects_item_concealment_item array

and save41_member0_checkpoint_state_tiles_item_light_member1_emission_member1 = {
  wire : wire;
  radius : json_number;
}

and save41_member0_checkpoint_state_tiles_item_light_member1_emission =
  [ `Null | `Object2 of save41_member0_checkpoint_state_tiles_item_light_member1_emission_member1 ]

and save41_member0_checkpoint_state_ground_items_item_item_perception =
  [ `Undefined
  | `Object2 of save41_member0_checkpoint_state_ground_items_item_item_perception_member1 ]

and save41_member0_checkpoint_state_ground_items_item_item_attacks_item = {
  wire : wire;
  id : json_string;
  name : json_string;
  damage : json_number;
  spread : json_number;
  count : json_number;
  damage_type : fragments_native_state38_ground_items_item_item_attacks_item_damage_type;
  category : fragments_native_state38_ground_items_item_item_attacks_item_category;
  range : json_number;
  sound : json_string;
  drain : json_number;
}

and save41_member0_checkpoint_state_ground_items_item_item_attacks =
  save41_member0_checkpoint_state_ground_items_item_item_attacks_item array

and save41_member0_checkpoint_state_ground_items =
  save41_member0_checkpoint_state_ground_items_item array

and shape_693 = [ `Player_victory | `Player_defeat ]
and literal_player_victory = [ `Player_victory ]
and literal_player_defeat = [ `Player_defeat ]
and save41_member0_checkpoint_state_winner = [ `Null | `Player_victory | `Player_defeat ]

and save41_member0_checkpoint_state_tiles_item = {
  wire : wire;
  id : json_string;
  location : json_string option;
  col : json_number;
  row : json_number;
  terrain : json_string;
  owner : fragments_tiles11_item_owner;
  connections : fragments_tiles11_item_connections_member1 option;
  dead_ends : fragments_tiles11_item_connections_member1 option;
  walls : fragments_tiles11_item_connections_member1 option;
  floor : json_string option;
  perception : save41_member0_checkpoint_state_tiles_item_perception;
  light : save41_member0_checkpoint_state_tiles_item_light_member1 option;
}

and save41_member0_checkpoint_state_tiles_item_perception = {
  wire : wire;
  sight : save41_member0_checkpoint_state_tiles_item_perception_sight;
  remember : json_boolean;
  concealment : save41_member0_checkpoint_state_tiles_item_perception_concealment;
}

and save41_member0_checkpoint_state_tiles_item_perception_sight_member1 = {
  wire : wire;
  group : json_string;
  profile : save41_member0_checkpoint_state_vision_groups_item_sight;
}

and save41_member0_checkpoint_state_tiles_item_perception_sight =
  [ `Null | `Object2 of save41_member0_checkpoint_state_tiles_item_perception_sight_member1 ]

and save41_member0_checkpoint_state_tiles_item_perception_concealment =
  save41_member0_checkpoint_state_tiles_item_perception_concealment_item array

and save41_member0_checkpoint_state_tiles_item_light_member1 = {
  wire : wire;
  emission : save41_member0_checkpoint_state_tiles_item_light_member1_emission;
  color : fragments_light23_color_member1 option;
  fixture : json_boolean option;
  dx : json_number option;
  dy : json_number option;
  radius : json_number;
  power : json_number;
  when_ : save41_member0_checkpoint_state_factions_item_transfer_member1_when_ option;
}

and save41_member0_checkpoint_state_units_item_retreating = [ `Undefined | `Boolean of json_boolean ]

and save41_member0_checkpoint_state_tiles_item_light_member1_when_ =
  [ `Undefined | `Object2 of save41_member0_checkpoint_state_factions_item_transfer_member1_when_ ]

and save41_member0_checkpoint_state_tiles_item_light =
  [ `Undefined | `Object2 of save41_member0_checkpoint_state_tiles_item_light_member1 ]

and save41_member0_checkpoint_state_tiles = save41_member0_checkpoint_state_tiles_item array

and save41_member0_checkpoint_state_units_item = {
  wire : wire;
  revealed_to : fragments_native_state38_log;
  perception_effects : save41_member0_checkpoint_state_units_item_perception_effects;
  faction : json_string;
  items : save41_member0_checkpoint_state_units_item_items_member1 option;
  ai_group : json_string option;
  id : json_string;
  type_ : json_string;
  side : save41_member0_setup_first;
  tile : save39_member0_checkpoint_state_ground_items_item_tile;
  hp : json_number;
  moved : json_boolean;
  attack_spent : json_boolean;
  experience : json_number option;
  character : json_string option;
  role : fragments_native_state38_units_item_role option;
  retreating : json_boolean option;
  rebuilding : json_boolean option;
}

and save41_member0_checkpoint_state_units_item_perception_effects_item = {
  wire : wire;
  id : json_string;
  penetrates : save41_member0_checkpoint_state_vision_groups_item_sight_penetrates;
  concealment : save41_member0_checkpoint_state_units_item_perception_effects_item_concealment;
  light : save41_member0_checkpoint_state_tiles_item_light_member1_emission;
  expires_at : save41_member0_checkpoint_state_units_item_perception_effects_item_expires_at;
}

and save41_member0_checkpoint_state_units_item_perception_effects_item_expires_at_member1 = {
  wire : wire;
  round : json_number;
  phase : save41_member0_setup_first;
}

and save41_member0_checkpoint_state_units_item_perception_effects_item_expires_at =
  [ `Null
  | `Object2 of
    save41_member0_checkpoint_state_units_item_perception_effects_item_expires_at_member1 ]

and save41_member0_checkpoint_state_units_item_perception_effects =
  save41_member0_checkpoint_state_units_item_perception_effects_item array

and save41_member0_checkpoint_state_units_item_items_member1 =
  save41_member0_checkpoint_state_ground_items_item_item array

and save41_member0_checkpoint_state_units_item_items =
  [ `Undefined | `Array of save41_member0_checkpoint_state_units_item_items_member1 ]

and shape_714 = [ `Leader | `Courier | `Captive ]
and save41_member0_checkpoint_state_units = save41_member0_checkpoint_state_units_item array

and save41_member0_checkpoint_state_campaign_member1 = {
  wire : wire;
  scripts : save41_member0_checkpoint_state_campaign_member1_scripts_member1 option;
  evacuated : save41_member0_checkpoint_state_campaign_member1_evacuated_member1 option;
  id : json_string;
  mission : json_number;
  recall : fragments_native_state38_log;
  veterans : save41_member0_checkpoint_state_campaign_member1_veterans_member1 option;
  story_veterans : save41_member0_checkpoint_state_campaign_member1_veterans_member1 option;
  fallen_companions : fragments_native_state38_log option;
  carry_gold : json_number;
  marks : fragments_native_state38_log;
  dialogue : save41_member0_checkpoint_state_campaign_member1_dialogue;
  dialogue_index : json_number;
  outcome : fragments_tiles11_item_owner;
  completed : json_boolean;
  refuge : fragments_native_state38_campaign_member1_refuge_member1 option;
  vigil : fragments_native_state38_campaign_member1_vigil_member1 option;
  holds : fragments_native_state38_campaign_member1_holds_member1 option;
  flags : fragments_native_state38_campaign_member1_flags;
}

and save41_member0_checkpoint_state_campaign_member1_scripts_member1 = {
  wire : wire;
  failed : fragments_native_state38_log;
  completed : fragments_native_state38_log;
  revealed : fragments_native_state38_log;
  recruitment : fragments_native_state38_campaign_member1_scripts_member1_recruitment;
  active : fragments_native_state38_log;
  holds : fragments_native_state38_campaign_member1_scripts_member1_holds;
  marks : save41_member0_checkpoint_state_campaign_member1_scripts_member1_marks;
  pending : save41_member0_checkpoint_state_campaign_member1_scripts_member1_pending;
  next_mark : json_number;
}

and save41_member0_checkpoint_state_campaign_member1_scripts_member1_marks_item = {
  wire : wire;
  sight : save41_member0_checkpoint_state_tiles_item_perception_sight;
  light_remembered : json_boolean;
  id : json_number;
  type_ : json_string;
  tile : save39_member0_checkpoint_state_ground_items_item_tile;
  light : save41_member0_checkpoint_state_tiles_item_light_member1 option;
}

and save41_member0_checkpoint_state_campaign_member1_scripts_member1_marks =
  save41_member0_checkpoint_state_campaign_member1_scripts_member1_marks_item array

and save41_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item = {
  wire : wire;
  area : save39_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item_area;
  faction : json_string;
  ai_group : fragments_tiles11_item_owner;
  units : save41_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item_units;
}

and save41_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item_units_item = {
  wire : wire;
  type_ : json_string;
  character : fragments_tiles11_item_owner;
  role :
    save41_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item_units_item_role;
  items : fragments_native_state38_log;
}

and save41_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item_units_item_role =
  [ `Null | `Leader | `Courier | `Captive ]

and save41_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item_units =
  save41_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item_units_item array

and save41_member0_checkpoint_state_campaign_member1_scripts_member1_pending =
  save41_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item array

and save41_member0_checkpoint_state_campaign_member1_scripts =
  [ `Undefined | `Object2 of save41_member0_checkpoint_state_campaign_member1_scripts_member1 ]

and save41_member0_checkpoint_state_campaign_member1_evacuated_member1_item = {
  wire : wire;
  event : json_string;
  unit_ : save41_member0_checkpoint_state_units_item;
}

and save41_member0_checkpoint_state_campaign_member1_evacuated_member1 =
  save41_member0_checkpoint_state_campaign_member1_evacuated_member1_item array

and save41_member0_checkpoint_state_campaign_member1_evacuated =
  [ `Undefined | `Array of save41_member0_checkpoint_state_campaign_member1_evacuated_member1 ]

and save41_member0_checkpoint_state_campaign_member1_veterans_member1_item = {
  wire : wire;
  items : save41_member0_checkpoint_state_units_item_items_member1 option;
  type_ : json_string;
  experience : json_number;
  character : json_string option;
  role : fragments_native_state38_units_item_role option;
}

and save41_member0_checkpoint_state_campaign_member1_veterans_member1 =
  save41_member0_checkpoint_state_campaign_member1_veterans_member1_item array

and save41_member0_checkpoint_state_campaign_member1_veterans =
  [ `Undefined | `Array of save41_member0_checkpoint_state_campaign_member1_veterans_member1 ]

and save41_member0_checkpoint_state_campaign_member1_dialogue_item = {
  wire : wire;
  sound : json_string option;
  speaker : json_string;
  character : json_string option;
  portrait : json_string;
  text : json_string;
}

and save41_member0_checkpoint_state_campaign_member1_dialogue =
  save41_member0_checkpoint_state_campaign_member1_dialogue_item array

and save41_member0_checkpoint_state_campaign =
  [ `Undefined | `Object2 of save41_member0_checkpoint_state_campaign_member1 ]

and shape_735 = [ `Null | `Object2 of fragments_native_state38_last_combat_member2 ]
and save41 = [ `Current of save41_member0 | `Turn of save41_member1 ]

and save41_member0 = {
  wire : wire;
  pending_enemy : json_boolean;
  undo : save34_undo_member1;
  progress : save39_member0_progress;
  version : literal_41;
  checkpoint : save41_member0_checkpoint;
  setup : save41_member0_setup;
}

and literal_41 = [ `V41 ]

and save41_member0_setup = {
  wire : wire;
  first : save41_member0_setup_first;
  enemy : save39_member0_setup_enemy;
}

and save41_member1 = {
  wire : wire;
  pending_enemy : literal_false;
  undo : json_undefined option;
  progress : save39_member1_progress;
  version : literal_41;
  checkpoint : save41_member0_checkpoint;
  setup : save41_member0_setup;
}

and save42_member0_setup_player = [ `Null | `Classic | `Situational | `Predictor ]

and save42_member0_setup = {
  wire : wire;
  first : save41_member0_setup_first;
  enemy : save39_member0_setup_enemy;
  player : save42_member0_setup_player;
}

and literal_42 = [ `V42 ]

and save42_member0 = {
  wire : wire;
  pending_enemy : json_boolean;
  undo : save34_undo_member1;
  progress : save39_member0_progress;
  version : literal_42;
  checkpoint : save41_member0_checkpoint;
  setup : save42_member0_setup;
}

and save42_member1 = {
  wire : wire;
  pending_enemy : literal_false;
  undo : json_undefined option;
  progress : save39_member1_progress;
  version : literal_42;
  checkpoint : save41_member0_checkpoint;
  setup : save42_member0_setup;
}

and save42 = [ `Current of save42_member0 | `Turn of save42_member1 ]
and save43 = [ `Current of save43_member0 | `Turn of save43_member1 ]

and save43_member0 = {
  wire : wire;
  pending_enemy : json_boolean;
  undo : save34_undo_member1;
  progress : save39_member0_progress;
  version : literal_43;
  checkpoint : save43_member0_checkpoint;
  setup : save42_member0_setup;
}

and literal_43 = [ `V43 ]

and save43_member0_checkpoint = {
  wire : wire;
  seed : json_number;
  state : save43_member0_checkpoint_state;
}

and save43_member0_checkpoint_state = {
  wire : wire;
  vision_groups : save41_member0_checkpoint_state_vision_groups;
  factions : save41_member0_checkpoint_state_factions;
  ground_items : save43_member0_checkpoint_state_ground_items;
  armies : fragments_native_state38_armies_member1 option;
  condition_memory : fragments_native_state38_log option;
  map_id : json_string;
  map_revision : json_number;
  cols : json_number;
  rows : json_number;
  round : json_number;
  turn : save41_member0_setup_first;
  gold : fragments_native_state38_gold;
  winner : save41_member0_checkpoint_state_winner;
  next_id : json_number;
  combat_rng : json_number;
  log : fragments_native_state38_log;
  tiles : save41_member0_checkpoint_state_tiles;
  units : save43_member0_checkpoint_state_units;
  campaign : save43_member0_checkpoint_state_campaign_member1 option;
  last_combat : fragments_native_state38_last_combat option;
}

and save43_member0_checkpoint_state_ground_items =
  save43_member0_checkpoint_state_ground_items_item array

and save43_member0_checkpoint_state_ground_items_item = {
  wire : wire;
  tile : save39_member0_checkpoint_state_ground_items_item_tile;
  item : save43_member0_checkpoint_state_ground_items_item_item;
}

and save43_member0_checkpoint_state_ground_items_item_item = {
  wire : wire;
  perception : save43_member0_checkpoint_state_ground_items_item_item_perception_member1 option;
  armor : fragments_native_state38_ground_items_item_item_armor_member1 option;
  id : json_string;
  name : json_string;
  description : json_string;
  attacks : save41_member0_checkpoint_state_ground_items_item_item_attacks;
  healing : fragments_native_state38_ground_items_item_item_healing_member1 option;
}

and save43_member0_checkpoint_state_ground_items_item_item_perception =
  [ `Undefined
  | `Object2 of save43_member0_checkpoint_state_ground_items_item_item_perception_member1 ]

and save43_member0_checkpoint_state_ground_items_item_item_perception_member1 = {
  wire : wire;
  range : save41_member0_checkpoint_state_vision_groups_item_sight_range;
  penetrates : save41_member0_checkpoint_state_vision_groups_item_sight_penetrates;
  concealment : save43_member0_checkpoint_state_units_item_perception_effects_item_concealment;
  light : save41_member0_checkpoint_state_tiles_item_light_member1_emission;
}

and save43_member0_checkpoint_state_units_item_perception_effects_item_concealment =
  save43_member0_checkpoint_state_units_item_perception_effects_item_concealment_item array

and save43_member0_checkpoint_state_units_item_perception_effects_item_concealment_item = {
  wire : wire;
  kind : save41_member0_checkpoint_state_vision_groups_item_sight_penetrates_item;
  reveal_within : json_number;
  terrains :
    save43_member0_checkpoint_state_units_item_perception_effects_item_concealment_item_terrains;
}

and save43_member0_checkpoint_state_units_item_perception_effects_item_concealment_item_terrains =
  [ `Null
  | `Array of
    save43_member0_checkpoint_state_units_item_perception_effects_item_concealment_item_terrains_member1
  ]

and save43_member0_checkpoint_state_units_item_perception_effects_item_concealment_item_terrains_member1 =
  save43_member0_checkpoint_state_units_item_perception_effects_item_concealment_item_terrains_member1_item
  array

and save43_member0_checkpoint_state_units_item_perception_effects_item_concealment_item_terrains_member1_item =
  [ `Bridge
  | `Burned_village
  | `Cave
  | `Church
  | `Ferry
  | `Ford
  | `Forest
  | `Fort
  | `Fortification
  | `Graveyard
  | `Grove
  | `Hill
  | `House
  | `Keep
  | `Landing
  | `Marsh
  | `Monastery
  | `Mountain
  | `Plains
  | `Ritual_spent
  | `Ritual_star
  | `Road
  | `Ruins
  | `Signpost
  | `Village
  | `Water
  | `Cave_wall ]

and literal_cave_wall = [ `Cave_wall ]
and save43_member0_checkpoint_state_units = save43_member0_checkpoint_state_units_item array

and save43_member0_checkpoint_state_units_item = {
  wire : wire;
  revealed_to : fragments_native_state38_log;
  perception_effects : save43_member0_checkpoint_state_units_item_perception_effects;
  faction : json_string;
  items : save43_member0_checkpoint_state_units_item_items_member1 option;
  ai_group : json_string option;
  id : json_string;
  type_ : json_string;
  side : save41_member0_setup_first;
  tile : save39_member0_checkpoint_state_ground_items_item_tile;
  hp : json_number;
  moved : json_boolean;
  attack_spent : json_boolean;
  experience : json_number option;
  character : json_string option;
  role : fragments_native_state38_units_item_role option;
  retreating : json_boolean option;
  rebuilding : json_boolean option;
}

and save43_member0_checkpoint_state_units_item_perception_effects =
  save43_member0_checkpoint_state_units_item_perception_effects_item array

and save43_member0_checkpoint_state_units_item_perception_effects_item = {
  wire : wire;
  id : json_string;
  penetrates : save41_member0_checkpoint_state_vision_groups_item_sight_penetrates;
  concealment : save43_member0_checkpoint_state_units_item_perception_effects_item_concealment;
  light : save41_member0_checkpoint_state_tiles_item_light_member1_emission;
  expires_at : save41_member0_checkpoint_state_units_item_perception_effects_item_expires_at;
}

and save43_member0_checkpoint_state_units_item_items =
  [ `Undefined | `Array of save43_member0_checkpoint_state_units_item_items_member1 ]

and save43_member0_checkpoint_state_units_item_items_member1 =
  save43_member0_checkpoint_state_ground_items_item_item array

and save43_member0_checkpoint_state_campaign =
  [ `Undefined | `Object2 of save43_member0_checkpoint_state_campaign_member1 ]

and save43_member0_checkpoint_state_campaign_member1 = {
  wire : wire;
  scripts : save41_member0_checkpoint_state_campaign_member1_scripts_member1 option;
  evacuated : save43_member0_checkpoint_state_campaign_member1_evacuated_member1 option;
  id : json_string;
  mission : json_number;
  recall : fragments_native_state38_log;
  veterans : save43_member0_checkpoint_state_campaign_member1_veterans_member1 option;
  story_veterans : save43_member0_checkpoint_state_campaign_member1_veterans_member1 option;
  fallen_companions : fragments_native_state38_log option;
  carry_gold : json_number;
  marks : fragments_native_state38_log;
  dialogue : save41_member0_checkpoint_state_campaign_member1_dialogue;
  dialogue_index : json_number;
  outcome : fragments_tiles11_item_owner;
  completed : json_boolean;
  refuge : fragments_native_state38_campaign_member1_refuge_member1 option;
  vigil : fragments_native_state38_campaign_member1_vigil_member1 option;
  holds : fragments_native_state38_campaign_member1_holds_member1 option;
  flags : fragments_native_state38_campaign_member1_flags;
}

and save43_member0_checkpoint_state_campaign_member1_evacuated =
  [ `Undefined | `Array of save43_member0_checkpoint_state_campaign_member1_evacuated_member1 ]

and save43_member0_checkpoint_state_campaign_member1_evacuated_member1 =
  save43_member0_checkpoint_state_campaign_member1_evacuated_member1_item array

and save43_member0_checkpoint_state_campaign_member1_evacuated_member1_item = {
  wire : wire;
  event : json_string;
  unit_ : save43_member0_checkpoint_state_units_item;
}

and save43_member0_checkpoint_state_campaign_member1_veterans =
  [ `Undefined | `Array of save43_member0_checkpoint_state_campaign_member1_veterans_member1 ]

and save43_member0_checkpoint_state_campaign_member1_veterans_member1 =
  save43_member0_checkpoint_state_campaign_member1_veterans_member1_item array

and save43_member0_checkpoint_state_campaign_member1_veterans_member1_item = {
  wire : wire;
  items : save43_member0_checkpoint_state_units_item_items_member1 option;
  type_ : json_string;
  experience : json_number;
  character : json_string option;
  role : fragments_native_state38_units_item_role option;
}

and save43_member1 = {
  wire : wire;
  pending_enemy : literal_false;
  undo : json_undefined option;
  progress : save39_member1_progress;
  version : literal_43;
  checkpoint : save43_member0_checkpoint;
  setup : save42_member0_setup;
}

and save44_member0_checkpoint = {
  wire : wire;
  seed : json_number;
  state : save44_member0_checkpoint_state;
}

and save44_member0_checkpoint_state = {
  wire : wire;
  vision_groups : save41_member0_checkpoint_state_vision_groups;
  factions : save41_member0_checkpoint_state_factions;
  ground_items : save44_member0_checkpoint_state_ground_items;
  armies : fragments_native_state38_armies_member1 option;
  condition_memory : fragments_native_state38_log option;
  map_id : json_string;
  map_revision : json_number;
  cols : json_number;
  rows : json_number;
  round : json_number;
  turn : save41_member0_setup_first;
  gold : fragments_native_state38_gold;
  winner : save41_member0_checkpoint_state_winner;
  next_id : json_number;
  combat_rng : json_number;
  log : fragments_native_state38_log;
  tiles : save44_member0_checkpoint_state_tiles;
  units : save44_member0_checkpoint_state_units;
  campaign : save44_member0_checkpoint_state_campaign_member1 option;
  last_combat : fragments_native_state38_last_combat option;
}

and save44_member0_checkpoint_state_ground_items_item = {
  wire : wire;
  tile : save39_member0_checkpoint_state_ground_items_item_tile;
  item : save44_member0_checkpoint_state_ground_items_item_item;
}

and save44_member0_checkpoint_state_ground_items_item_item = {
  wire : wire;
  perception : save44_member0_checkpoint_state_ground_items_item_item_perception_member1 option;
  armor : fragments_native_state38_ground_items_item_item_armor_member1 option;
  id : json_string;
  name : json_string;
  description : json_string;
  attacks : save41_member0_checkpoint_state_ground_items_item_item_attacks;
  healing : fragments_native_state38_ground_items_item_item_healing_member1 option;
}

and save44_member0_checkpoint_state_ground_items_item_item_perception_member1 = {
  wire : wire;
  range : save41_member0_checkpoint_state_vision_groups_item_sight_range;
  penetrates : save41_member0_checkpoint_state_vision_groups_item_sight_penetrates;
  concealment : save44_member0_checkpoint_state_tiles_item_perception_concealment;
  light : save41_member0_checkpoint_state_tiles_item_light_member1_emission;
}

and save44_member0_checkpoint_state_tiles_item_perception_concealment_item = {
  wire : wire;
  kind : save41_member0_checkpoint_state_vision_groups_item_sight_penetrates_item;
  reveal_within : json_number;
  requires : save44_member0_checkpoint_state_tiles_item_perception_concealment_item_requires;
}

and save44_member0_checkpoint_state_tiles_item_perception_concealment_item_requires = {
  wire : wire;
  terrains :
    save44_member0_checkpoint_state_tiles_item_perception_concealment_item_requires_terrains;
  illumination :
    save44_member0_checkpoint_state_tiles_item_perception_concealment_item_requires_illumination;
}

and save44_member0_checkpoint_state_tiles_item_perception_concealment_item_requires_terrains_member1_item =
  [ `Bridge
  | `Burned_village
  | `Cave
  | `Cave_wall
  | `Church
  | `Ferry
  | `Ford
  | `Forest
  | `Fort
  | `Fortification
  | `Graveyard
  | `Grove
  | `Hill
  | `House
  | `Keep
  | `Landing
  | `Marsh
  | `Monastery
  | `Mountain
  | `Plains
  | `Ritual_spent
  | `Ritual_star
  | `Road
  | `Ruins
  | `Signpost
  | `Village
  | `Water ]

and save44_member0_checkpoint_state_tiles_item_perception_concealment_item_requires_terrains_member1 =
  save44_member0_checkpoint_state_tiles_item_perception_concealment_item_requires_terrains_member1_item
  array

and save44_member0_checkpoint_state_tiles_item_perception_concealment_item_requires_terrains =
  [ `Null
  | `Array of
    save44_member0_checkpoint_state_tiles_item_perception_concealment_item_requires_terrains_member1
  ]

and shape_788 = [ `Lit | `Unlit ]
and literal_lit = [ `Lit ]
and literal_unlit = [ `Unlit ]

and save44_member0_checkpoint_state_tiles_item_perception_concealment_item_requires_illumination =
  [ `Null | `Lit | `Unlit ]

and save44_member0_checkpoint_state_tiles_item_perception_concealment =
  save44_member0_checkpoint_state_tiles_item_perception_concealment_item array

and save44_member0_checkpoint_state_ground_items_item_item_perception =
  [ `Undefined
  | `Object2 of save44_member0_checkpoint_state_ground_items_item_item_perception_member1 ]

and save44_member0_checkpoint_state_ground_items =
  save44_member0_checkpoint_state_ground_items_item array

and save44_member0_checkpoint_state_tiles_item = {
  wire : wire;
  id : json_string;
  location : json_string option;
  col : json_number;
  row : json_number;
  terrain : json_string;
  owner : fragments_tiles11_item_owner;
  connections : fragments_tiles11_item_connections_member1 option;
  dead_ends : fragments_tiles11_item_connections_member1 option;
  walls : fragments_tiles11_item_connections_member1 option;
  floor : json_string option;
  perception : save44_member0_checkpoint_state_tiles_item_perception;
  light : save41_member0_checkpoint_state_tiles_item_light_member1 option;
}

and save44_member0_checkpoint_state_tiles_item_perception = {
  wire : wire;
  sight : save41_member0_checkpoint_state_tiles_item_perception_sight;
  remember : json_boolean;
  concealment : save44_member0_checkpoint_state_tiles_item_perception_concealment;
}

and save44_member0_checkpoint_state_tiles = save44_member0_checkpoint_state_tiles_item array

and save44_member0_checkpoint_state_units_item = {
  wire : wire;
  revealed_to : fragments_native_state38_log;
  perception_effects : save44_member0_checkpoint_state_units_item_perception_effects;
  faction : json_string;
  items : save44_member0_checkpoint_state_units_item_items_member1 option;
  ai_group : json_string option;
  id : json_string;
  type_ : json_string;
  side : save41_member0_setup_first;
  tile : save39_member0_checkpoint_state_ground_items_item_tile;
  hp : json_number;
  moved : json_boolean;
  attack_spent : json_boolean;
  experience : json_number option;
  character : json_string option;
  role : fragments_native_state38_units_item_role option;
  retreating : json_boolean option;
  rebuilding : json_boolean option;
}

and save44_member0_checkpoint_state_units_item_perception_effects_item = {
  wire : wire;
  id : json_string;
  penetrates : save41_member0_checkpoint_state_vision_groups_item_sight_penetrates;
  concealment : save44_member0_checkpoint_state_tiles_item_perception_concealment;
  light : save41_member0_checkpoint_state_tiles_item_light_member1_emission;
  expires_at : save41_member0_checkpoint_state_units_item_perception_effects_item_expires_at;
}

and save44_member0_checkpoint_state_units_item_perception_effects =
  save44_member0_checkpoint_state_units_item_perception_effects_item array

and save44_member0_checkpoint_state_units_item_items_member1 =
  save44_member0_checkpoint_state_ground_items_item_item array

and save44_member0_checkpoint_state_units_item_items =
  [ `Undefined | `Array of save44_member0_checkpoint_state_units_item_items_member1 ]

and save44_member0_checkpoint_state_units = save44_member0_checkpoint_state_units_item array

and save44_member0_checkpoint_state_campaign_member1 = {
  wire : wire;
  scripts : save41_member0_checkpoint_state_campaign_member1_scripts_member1 option;
  evacuated : save44_member0_checkpoint_state_campaign_member1_evacuated_member1 option;
  id : json_string;
  mission : json_number;
  recall : fragments_native_state38_log;
  veterans : save44_member0_checkpoint_state_campaign_member1_veterans_member1 option;
  story_veterans : save44_member0_checkpoint_state_campaign_member1_veterans_member1 option;
  fallen_companions : fragments_native_state38_log option;
  carry_gold : json_number;
  marks : fragments_native_state38_log;
  dialogue : save41_member0_checkpoint_state_campaign_member1_dialogue;
  dialogue_index : json_number;
  outcome : fragments_tiles11_item_owner;
  completed : json_boolean;
  refuge : fragments_native_state38_campaign_member1_refuge_member1 option;
  vigil : fragments_native_state38_campaign_member1_vigil_member1 option;
  holds : fragments_native_state38_campaign_member1_holds_member1 option;
  flags : fragments_native_state38_campaign_member1_flags;
}

and save44_member0_checkpoint_state_campaign_member1_evacuated_member1_item = {
  wire : wire;
  event : json_string;
  unit_ : save44_member0_checkpoint_state_units_item;
}

and save44_member0_checkpoint_state_campaign_member1_evacuated_member1 =
  save44_member0_checkpoint_state_campaign_member1_evacuated_member1_item array

and save44_member0_checkpoint_state_campaign_member1_evacuated =
  [ `Undefined | `Array of save44_member0_checkpoint_state_campaign_member1_evacuated_member1 ]

and save44_member0_checkpoint_state_campaign_member1_veterans_member1_item = {
  wire : wire;
  items : save44_member0_checkpoint_state_units_item_items_member1 option;
  type_ : json_string;
  experience : json_number;
  character : json_string option;
  role : fragments_native_state38_units_item_role option;
}

and save44_member0_checkpoint_state_campaign_member1_veterans_member1 =
  save44_member0_checkpoint_state_campaign_member1_veterans_member1_item array

and save44_member0_checkpoint_state_campaign_member1_veterans =
  [ `Undefined | `Array of save44_member0_checkpoint_state_campaign_member1_veterans_member1 ]

and save44_member0_checkpoint_state_campaign =
  [ `Undefined | `Object2 of save44_member0_checkpoint_state_campaign_member1 ]

and save44 = [ `Current of save44_member0 | `Turn of save44_member1 ]

and save44_member0 = {
  wire : wire;
  pending_enemy : json_boolean;
  undo : save34_undo_member1;
  progress : save39_member0_progress;
  version : literal_44;
  checkpoint : save44_member0_checkpoint;
  setup : save42_member0_setup;
}

and literal_44 = [ `V44 ]

and save44_member1 = {
  wire : wire;
  pending_enemy : literal_false;
  undo : json_undefined option;
  progress : save39_member1_progress;
  version : literal_44;
  checkpoint : save44_member0_checkpoint;
  setup : save42_member0_setup;
}

module V2 = struct
  type nonrec t = save2
  type nonrec version = literal_2
  type nonrec checkpoint = save2_checkpoint
  type nonrec pending_enemy = json_boolean
  type nonrec planner = json_unknown
  type nonrec setup = save2_setup_member1
  type nonrec progress = save2_progress_member1
  type nonrec undo = json_unknown
  type nonrec seed = json_number
  type nonrec state = save2_checkpoint_state
  type nonrec first = save35_setup_first
  type nonrec enemy = save2_setup_member1_enemy
  type nonrec entry = history1_entries_item
  type nonrec round = json_number
  type nonrec side = save35_setup_first
  type nonrec winner = fragments_native_state38_winner
  type nonrec armies = fragments_native_state38_armies_member1
  type nonrec condition_memory = json_string
  type nonrec map_id = json_string
  type nonrec map_revision = json_number
  type nonrec cols = json_number
  type nonrec rows = json_number
  type nonrec next_id = json_number
  type nonrec log = json_string
  type nonrec last_combat = fragments_native_state38_last_combat_member2
  type nonrec unit_ = save2_checkpoint_state_units_item
  type nonrec tile = save2_checkpoint_state_tiles_item
  type nonrec campaign = save2_checkpoint_state_campaign_member1
  type nonrec gold = javascript1_state_gold
  type nonrec faction = save2_checkpoint_state_factions_member1_item
  type nonrec ground_items = fragments_native_state38_ground_items_item
  type nonrec combat_rng = json_number
  type nonrec policy = save35_setup_enemy_policy
  type nonrec blocks = save35_setup_enemy_blocks_member1
  type nonrec recruitment_limit = json_number
  type nonrec diplomacy = save2_setup_member1_enemy_diplomacy
  type nonrec id = json_string
  type nonrec saved_at = json_number
  type nonrec run = json_string
  type nonrec mission = json_string
  type nonrec mission_start = json_boolean
  type nonrec run_id = json_string
  type nonrec branch_id = json_string
  type nonrec checkpoint_id = json_string
  type nonrec ordinal = json_number
  type nonrec kind = history1_entries_item_kind
  type nonrec parent_id = json_string
  type nonrec attacker_id = json_string
  type nonrec attack_id = json_string
  type nonrec sound = json_string
  type nonrec role = fragments_native_state38_units_item_role
  type nonrec items = fragments_native_state38_ground_items_item_item
  type nonrec ai_group = json_string
  type nonrec type_ = json_string
  type nonrec hp = json_number
  type nonrec moved = json_boolean
  type nonrec attacked = json_boolean
  type nonrec retreating = fragments_rule11_remember
  type nonrec rebuilding = fragments_rule11_remember
  type nonrec name = json_string
  type nonrec character = json_string
  type nonrec experience = json_number
  type nonrec owner = json_string
  type nonrec location = json_string
  type nonrec col = json_number
  type nonrec row = json_number
  type nonrec terrain = json_string
  type nonrec connection = fragments_tiles11_item_connections_member1_item
  type nonrec dead_ends = fragments_tiles11_item_connections_member1_item
  type nonrec walls = fragments_tiles11_item_connections_member1_item
  type nonrec floor = json_string
  type nonrec fog = save2_checkpoint_state_tiles_item_fog_member1
  type nonrec light = save2_checkpoint_state_tiles_item_light_member1
  type nonrec wall_corners = json_number
  type nonrec vigil = fragments_native_state38_campaign_member1_vigil_member1
  type nonrec refuge = fragments_native_state38_campaign_member1_refuge_member1
  type nonrec completed = json_boolean
  type nonrec mark = json_string
  type nonrec hold = fragments_native_state38_campaign_member1_holds_member1
  type nonrec recall = json_string
  type nonrec fallen_companions = json_string
  type nonrec carry_gold = json_number
  type nonrec dialogue = fragments_native_state38_campaign_member1_dialogue_item
  type nonrec dialogue_index = json_number
  type nonrec outcome = json_string
  type nonrec veteran = save2_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec story_veteran = save2_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec evacuated = save2_checkpoint_state_campaign_member1_evacuated_member1_item
  type nonrec choices = json_string
  type nonrec player = json_number
  type nonrec phase = save35_setup_first
  type nonrec controller = fragments_factions7_item_controller
  type nonrec palette = json_string
  type nonrec recruits = fragments_factions7_item_recruits_member1_item
  type nonrec relations = fragments_factions7_item_relations_member1
  type nonrec transfer = save2_checkpoint_state_factions_member1_item_transfer_member1
  type nonrec item = fragments_native_state38_ground_items_item_item
  type nonrec recruiter = save35_setup_enemy_blocks_member1_recruiter
  type nonrec armor = fragments_native_state38_ground_items_item_item_armor_member1
  type nonrec description = json_string
  type nonrec attacks = fragments_native_state38_ground_items_item_item_attacks_item
  type nonrec healing = fragments_native_state38_ground_items_item_item_healing_member1
  type nonrec condition = save2_checkpoint_state_tiles_item_fog_member1_condition
  type nonrec remember = fragments_rule11_remember
  type nonrec radius = json_number
  type nonrec color = fragments_light23_color_member1
  type nonrec fixture = fragments_rule11_remember
  type nonrec dx = json_number
  type nonrec dy = json_number
  type nonrec power = json_number
  type nonrec rule = save2_checkpoint_state_tiles_item_fog_member1
  type nonrec held_turns = json_number
  type nonrec last_counted_round = json_number
  type nonrec escalated = json_boolean
  type nonrec speaker = json_string
  type nonrec portrait = json_string
  type nonrec text = json_string
  type nonrec event = json_string
  type nonrec controller_case = fragments_factions7_item_controller_member0
  type nonrec cost = json_number
  type nonrec to_ = json_string
  type nonrec blade = json_number
  type nonrec pierce = json_number
  type nonrec impact = json_number
  type nonrec damage = json_number
  type nonrec spread = json_number
  type nonrec count = json_number
  type nonrec damage_type = fragments_native_state38_ground_items_item_item_attacks_item_damage_type
  type nonrec category = fragments_native_state38_ground_items_item_item_attacks_item_category
  type nonrec range = json_number
  type nonrec drain = json_number
  type nonrec amount = json_number
  type nonrec condition_case = fragments_rule11_condition_member0
  type nonrec unit_types = json_string
  type nonrec choice = json_string
  type nonrec flag = json_string
  type nonrec conditions = save2_checkpoint_state_tiles_item_fog_member1_condition
  type nonrec unit_type = json_string
end

module V3 = struct
  type nonrec t = save3
  type nonrec version = literal_3
  type nonrec checkpoint = save3_checkpoint
  type nonrec pending_enemy = json_boolean
  type nonrec planner = json_unknown
  type nonrec setup = save2_setup_member1
  type nonrec progress = save2_progress_member1
  type nonrec undo = json_unknown
  type nonrec seed = json_number
  type nonrec state = save3_checkpoint_state
  type nonrec first = save35_setup_first
  type nonrec enemy = save2_setup_member1_enemy
  type nonrec entry = history1_entries_item
  type nonrec round = json_number
  type nonrec side = save35_setup_first
  type nonrec winner = fragments_native_state38_winner
  type nonrec armies = fragments_native_state38_armies_member1
  type nonrec condition_memory = json_string
  type nonrec map_id = json_string
  type nonrec map_revision = json_number
  type nonrec cols = json_number
  type nonrec rows = json_number
  type nonrec next_id = json_number
  type nonrec log = json_string
  type nonrec last_combat = fragments_native_state38_last_combat_member2
  type nonrec unit_ = save2_checkpoint_state_units_item
  type nonrec tile = fragments_tiles11_item
  type nonrec campaign = save2_checkpoint_state_campaign_member1
  type nonrec gold = javascript1_state_gold
  type nonrec faction = fragments_factions7_item
  type nonrec ground_items = fragments_native_state38_ground_items_item
  type nonrec combat_rng = json_number
  type nonrec policy = save35_setup_enemy_policy
  type nonrec blocks = save35_setup_enemy_blocks_member1
  type nonrec recruitment_limit = json_number
  type nonrec diplomacy = save2_setup_member1_enemy_diplomacy
  type nonrec id = json_string
  type nonrec saved_at = json_number
  type nonrec run = json_string
  type nonrec mission = json_string
  type nonrec mission_start = json_boolean
  type nonrec run_id = json_string
  type nonrec branch_id = json_string
  type nonrec checkpoint_id = json_string
  type nonrec ordinal = json_number
  type nonrec kind = history1_entries_item_kind
  type nonrec parent_id = json_string
  type nonrec attacker_id = json_string
  type nonrec attack_id = json_string
  type nonrec sound = json_string
  type nonrec role = fragments_native_state38_units_item_role
  type nonrec items = fragments_native_state38_ground_items_item_item
  type nonrec ai_group = json_string
  type nonrec type_ = json_string
  type nonrec hp = json_number
  type nonrec moved = json_boolean
  type nonrec attacked = json_boolean
  type nonrec retreating = fragments_rule11_remember
  type nonrec rebuilding = fragments_rule11_remember
  type nonrec name = json_string
  type nonrec character = json_string
  type nonrec experience = json_number
  type nonrec owner = json_string
  type nonrec location = json_string
  type nonrec col = json_number
  type nonrec row = json_number
  type nonrec terrain = json_string
  type nonrec connection = fragments_tiles11_item_connections_member1_item
  type nonrec dead_ends = fragments_tiles11_item_connections_member1_item
  type nonrec walls = fragments_tiles11_item_connections_member1_item
  type nonrec floor = json_string
  type nonrec fog = fragments_rule11
  type nonrec light = fragments_light23
  type nonrec wall_corners = json_number
  type nonrec vigil = fragments_native_state38_campaign_member1_vigil_member1
  type nonrec refuge = fragments_native_state38_campaign_member1_refuge_member1
  type nonrec completed = json_boolean
  type nonrec mark = json_string
  type nonrec hold = fragments_native_state38_campaign_member1_holds_member1
  type nonrec recall = json_string
  type nonrec fallen_companions = json_string
  type nonrec carry_gold = json_number
  type nonrec dialogue = fragments_native_state38_campaign_member1_dialogue_item
  type nonrec dialogue_index = json_number
  type nonrec outcome = json_string
  type nonrec veteran = save2_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec story_veteran = save2_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec evacuated = save2_checkpoint_state_campaign_member1_evacuated_member1_item
  type nonrec choices = json_string
  type nonrec player = json_number
  type nonrec phase = save35_setup_first
  type nonrec controller = fragments_factions7_item_controller
  type nonrec palette = json_string
  type nonrec recruits = fragments_factions7_item_recruits_member1_item
  type nonrec relations = fragments_factions7_item_relations_member1
  type nonrec transfer = fragments_factions7_item_transfer_member1
  type nonrec item = fragments_native_state38_ground_items_item_item
  type nonrec recruiter = save35_setup_enemy_blocks_member1_recruiter
  type nonrec armor = fragments_native_state38_ground_items_item_item_armor_member1
  type nonrec description = json_string
  type nonrec attacks = fragments_native_state38_ground_items_item_item_attacks_item
  type nonrec healing = fragments_native_state38_ground_items_item_item_healing_member1
  type nonrec condition = fragments_rule11_condition
  type nonrec remember = fragments_rule11_remember
  type nonrec radius = json_number
  type nonrec color = fragments_light23_color_member1
  type nonrec fixture = fragments_rule11_remember
  type nonrec dx = json_number
  type nonrec dy = json_number
  type nonrec power = json_number
  type nonrec rule = fragments_rule11
  type nonrec held_turns = json_number
  type nonrec last_counted_round = json_number
  type nonrec escalated = json_boolean
  type nonrec speaker = json_string
  type nonrec portrait = json_string
  type nonrec text = json_string
  type nonrec event = json_string
  type nonrec controller_case = fragments_factions7_item_controller_member0
  type nonrec cost = json_number
  type nonrec to_ = json_string
  type nonrec blade = json_number
  type nonrec pierce = json_number
  type nonrec impact = json_number
  type nonrec damage = json_number
  type nonrec spread = json_number
  type nonrec count = json_number
  type nonrec damage_type = fragments_native_state38_ground_items_item_item_attacks_item_damage_type
  type nonrec category = fragments_native_state38_ground_items_item_item_attacks_item_category
  type nonrec range = json_number
  type nonrec drain = json_number
  type nonrec amount = json_number
  type nonrec condition_case = fragments_rule11_condition_member0
  type nonrec unit_types = json_string
  type nonrec choice = json_string
  type nonrec flag = json_string
  type nonrec unit_type = json_string
  type nonrec conditions = fragments_rule11_condition
end

module V4 = struct
  type nonrec t = save4
  type nonrec version = literal_4
  type nonrec checkpoint = save3_checkpoint
  type nonrec pending_enemy = json_boolean
  type nonrec planner = json_unknown
  type nonrec setup = save2_setup_member1
  type nonrec progress = save2_progress_member1
  type nonrec undo = json_unknown
  type nonrec seed = json_number
  type nonrec state = save3_checkpoint_state
  type nonrec first = save35_setup_first
  type nonrec enemy = save2_setup_member1_enemy
  type nonrec entry = history1_entries_item
  type nonrec round = json_number
  type nonrec side = save35_setup_first
  type nonrec winner = fragments_native_state38_winner
  type nonrec armies = fragments_native_state38_armies_member1
  type nonrec condition_memory = json_string
  type nonrec map_id = json_string
  type nonrec map_revision = json_number
  type nonrec cols = json_number
  type nonrec rows = json_number
  type nonrec next_id = json_number
  type nonrec log = json_string
  type nonrec last_combat = fragments_native_state38_last_combat_member2
  type nonrec unit_ = save2_checkpoint_state_units_item
  type nonrec tile = fragments_tiles11_item
  type nonrec campaign = save2_checkpoint_state_campaign_member1
  type nonrec gold = javascript1_state_gold
  type nonrec faction = fragments_factions7_item
  type nonrec ground_items = fragments_native_state38_ground_items_item
  type nonrec combat_rng = json_number
  type nonrec policy = save35_setup_enemy_policy
  type nonrec blocks = save35_setup_enemy_blocks_member1
  type nonrec recruitment_limit = json_number
  type nonrec diplomacy = save2_setup_member1_enemy_diplomacy
  type nonrec id = json_string
  type nonrec saved_at = json_number
  type nonrec run = json_string
  type nonrec mission = json_string
  type nonrec mission_start = json_boolean
  type nonrec run_id = json_string
  type nonrec branch_id = json_string
  type nonrec checkpoint_id = json_string
  type nonrec ordinal = json_number
  type nonrec kind = history1_entries_item_kind
  type nonrec parent_id = json_string
  type nonrec attacker_id = json_string
  type nonrec attack_id = json_string
  type nonrec sound = json_string
  type nonrec role = fragments_native_state38_units_item_role
  type nonrec items = fragments_native_state38_ground_items_item_item
  type nonrec ai_group = json_string
  type nonrec type_ = json_string
  type nonrec hp = json_number
  type nonrec moved = json_boolean
  type nonrec attacked = json_boolean
  type nonrec retreating = fragments_rule11_remember
  type nonrec rebuilding = fragments_rule11_remember
  type nonrec name = json_string
  type nonrec character = json_string
  type nonrec experience = json_number
  type nonrec owner = json_string
  type nonrec location = json_string
  type nonrec col = json_number
  type nonrec row = json_number
  type nonrec terrain = json_string
  type nonrec connection = fragments_tiles11_item_connections_member1_item
  type nonrec dead_ends = fragments_tiles11_item_connections_member1_item
  type nonrec walls = fragments_tiles11_item_connections_member1_item
  type nonrec floor = json_string
  type nonrec fog = fragments_rule11
  type nonrec light = fragments_light23
  type nonrec wall_corners = json_number
  type nonrec vigil = fragments_native_state38_campaign_member1_vigil_member1
  type nonrec refuge = fragments_native_state38_campaign_member1_refuge_member1
  type nonrec completed = json_boolean
  type nonrec mark = json_string
  type nonrec hold = fragments_native_state38_campaign_member1_holds_member1
  type nonrec recall = json_string
  type nonrec fallen_companions = json_string
  type nonrec carry_gold = json_number
  type nonrec dialogue = fragments_native_state38_campaign_member1_dialogue_item
  type nonrec dialogue_index = json_number
  type nonrec outcome = json_string
  type nonrec veteran = save2_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec story_veteran = save2_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec evacuated = save2_checkpoint_state_campaign_member1_evacuated_member1_item
  type nonrec choices = json_string
  type nonrec player = json_number
  type nonrec phase = save35_setup_first
  type nonrec controller = fragments_factions7_item_controller
  type nonrec palette = json_string
  type nonrec recruits = fragments_factions7_item_recruits_member1_item
  type nonrec relations = fragments_factions7_item_relations_member1
  type nonrec transfer = fragments_factions7_item_transfer_member1
  type nonrec item = fragments_native_state38_ground_items_item_item
  type nonrec recruiter = save35_setup_enemy_blocks_member1_recruiter
  type nonrec armor = fragments_native_state38_ground_items_item_item_armor_member1
  type nonrec description = json_string
  type nonrec attacks = fragments_native_state38_ground_items_item_item_attacks_item
  type nonrec healing = fragments_native_state38_ground_items_item_item_healing_member1
  type nonrec condition = fragments_rule11_condition
  type nonrec remember = fragments_rule11_remember
  type nonrec radius = json_number
  type nonrec color = fragments_light23_color_member1
  type nonrec fixture = fragments_rule11_remember
  type nonrec dx = json_number
  type nonrec dy = json_number
  type nonrec power = json_number
  type nonrec rule = fragments_rule11
  type nonrec held_turns = json_number
  type nonrec last_counted_round = json_number
  type nonrec escalated = json_boolean
  type nonrec speaker = json_string
  type nonrec portrait = json_string
  type nonrec text = json_string
  type nonrec event = json_string
  type nonrec controller_case = fragments_factions7_item_controller_member0
  type nonrec cost = json_number
  type nonrec to_ = json_string
  type nonrec blade = json_number
  type nonrec pierce = json_number
  type nonrec impact = json_number
  type nonrec damage = json_number
  type nonrec spread = json_number
  type nonrec count = json_number
  type nonrec damage_type = fragments_native_state38_ground_items_item_item_attacks_item_damage_type
  type nonrec category = fragments_native_state38_ground_items_item_item_attacks_item_category
  type nonrec range = json_number
  type nonrec drain = json_number
  type nonrec amount = json_number
  type nonrec condition_case = fragments_rule11_condition_member0
  type nonrec unit_types = json_string
  type nonrec choice = json_string
  type nonrec flag = json_string
  type nonrec unit_type = json_string
  type nonrec conditions = fragments_rule11_condition
end

module V5 = struct
  type nonrec t = save5
  type nonrec version = literal_5
  type nonrec checkpoint = save3_checkpoint
  type nonrec pending_enemy = json_boolean
  type nonrec setup = save2_setup_member1
  type nonrec progress = save2_progress_member1
  type nonrec undo = json_unknown
  type nonrec seed = json_number
  type nonrec state = save3_checkpoint_state
  type nonrec first = save35_setup_first
  type nonrec enemy = save2_setup_member1_enemy
  type nonrec entry = history1_entries_item
  type nonrec round = json_number
  type nonrec side = save35_setup_first
  type nonrec winner = fragments_native_state38_winner
  type nonrec armies = fragments_native_state38_armies_member1
  type nonrec condition_memory = json_string
  type nonrec map_id = json_string
  type nonrec map_revision = json_number
  type nonrec cols = json_number
  type nonrec rows = json_number
  type nonrec next_id = json_number
  type nonrec log = json_string
  type nonrec last_combat = fragments_native_state38_last_combat_member2
  type nonrec unit_ = save2_checkpoint_state_units_item
  type nonrec tile = fragments_tiles11_item
  type nonrec campaign = save2_checkpoint_state_campaign_member1
  type nonrec gold = javascript1_state_gold
  type nonrec faction = fragments_factions7_item
  type nonrec ground_items = fragments_native_state38_ground_items_item
  type nonrec combat_rng = json_number
  type nonrec policy = save35_setup_enemy_policy
  type nonrec blocks = save35_setup_enemy_blocks_member1
  type nonrec recruitment_limit = json_number
  type nonrec diplomacy = save2_setup_member1_enemy_diplomacy
  type nonrec id = json_string
  type nonrec saved_at = json_number
  type nonrec run = json_string
  type nonrec mission = json_string
  type nonrec mission_start = json_boolean
  type nonrec run_id = json_string
  type nonrec branch_id = json_string
  type nonrec checkpoint_id = json_string
  type nonrec ordinal = json_number
  type nonrec kind = history1_entries_item_kind
  type nonrec parent_id = json_string
  type nonrec attacker_id = json_string
  type nonrec attack_id = json_string
  type nonrec sound = json_string
  type nonrec role = fragments_native_state38_units_item_role
  type nonrec items = fragments_native_state38_ground_items_item_item
  type nonrec ai_group = json_string
  type nonrec type_ = json_string
  type nonrec hp = json_number
  type nonrec moved = json_boolean
  type nonrec attacked = json_boolean
  type nonrec retreating = fragments_rule11_remember
  type nonrec rebuilding = fragments_rule11_remember
  type nonrec name = json_string
  type nonrec character = json_string
  type nonrec experience = json_number
  type nonrec owner = json_string
  type nonrec location = json_string
  type nonrec col = json_number
  type nonrec row = json_number
  type nonrec terrain = json_string
  type nonrec connection = fragments_tiles11_item_connections_member1_item
  type nonrec dead_ends = fragments_tiles11_item_connections_member1_item
  type nonrec walls = fragments_tiles11_item_connections_member1_item
  type nonrec floor = json_string
  type nonrec fog = fragments_rule11
  type nonrec light = fragments_light23
  type nonrec wall_corners = json_number
  type nonrec vigil = fragments_native_state38_campaign_member1_vigil_member1
  type nonrec refuge = fragments_native_state38_campaign_member1_refuge_member1
  type nonrec completed = json_boolean
  type nonrec mark = json_string
  type nonrec hold = fragments_native_state38_campaign_member1_holds_member1
  type nonrec recall = json_string
  type nonrec fallen_companions = json_string
  type nonrec carry_gold = json_number
  type nonrec dialogue = fragments_native_state38_campaign_member1_dialogue_item
  type nonrec dialogue_index = json_number
  type nonrec outcome = json_string
  type nonrec veteran = save2_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec story_veteran = save2_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec evacuated = save2_checkpoint_state_campaign_member1_evacuated_member1_item
  type nonrec choices = json_string
  type nonrec player = json_number
  type nonrec phase = save35_setup_first
  type nonrec controller = fragments_factions7_item_controller
  type nonrec palette = json_string
  type nonrec recruits = fragments_factions7_item_recruits_member1_item
  type nonrec relations = fragments_factions7_item_relations_member1
  type nonrec transfer = fragments_factions7_item_transfer_member1
  type nonrec item = fragments_native_state38_ground_items_item_item
  type nonrec recruiter = save35_setup_enemy_blocks_member1_recruiter
  type nonrec planner = save35_setup_enemy_blocks_member1_planner
  type nonrec armor = fragments_native_state38_ground_items_item_item_armor_member1
  type nonrec description = json_string
  type nonrec attacks = fragments_native_state38_ground_items_item_item_attacks_item
  type nonrec healing = fragments_native_state38_ground_items_item_item_healing_member1
  type nonrec condition = fragments_rule11_condition
  type nonrec remember = fragments_rule11_remember
  type nonrec radius = json_number
  type nonrec color = fragments_light23_color_member1
  type nonrec fixture = fragments_rule11_remember
  type nonrec dx = json_number
  type nonrec dy = json_number
  type nonrec power = json_number
  type nonrec rule = fragments_rule11
  type nonrec held_turns = json_number
  type nonrec last_counted_round = json_number
  type nonrec escalated = json_boolean
  type nonrec speaker = json_string
  type nonrec portrait = json_string
  type nonrec text = json_string
  type nonrec event = json_string
  type nonrec controller_case = fragments_factions7_item_controller_member0
  type nonrec cost = json_number
  type nonrec to_ = json_string
  type nonrec blade = json_number
  type nonrec pierce = json_number
  type nonrec impact = json_number
  type nonrec damage = json_number
  type nonrec spread = json_number
  type nonrec count = json_number
  type nonrec damage_type = fragments_native_state38_ground_items_item_item_attacks_item_damage_type
  type nonrec category = fragments_native_state38_ground_items_item_item_attacks_item_category
  type nonrec range = json_number
  type nonrec drain = json_number
  type nonrec amount = json_number
  type nonrec condition_case = fragments_rule11_condition_member0
  type nonrec unit_types = json_string
  type nonrec choice = json_string
  type nonrec flag = json_string
  type nonrec unit_type = json_string
  type nonrec conditions = fragments_rule11_condition
end

module V6 = struct
  type nonrec t = save6
  type nonrec version = literal_6
  type nonrec checkpoint = save3_checkpoint
  type nonrec pending_enemy = json_boolean
  type nonrec setup = save2_setup_member1
  type nonrec progress = save2_progress_member1
  type nonrec undo = json_unknown
  type nonrec seed = json_number
  type nonrec state = save3_checkpoint_state
  type nonrec first = save35_setup_first
  type nonrec enemy = save2_setup_member1_enemy
  type nonrec entry = history1_entries_item
  type nonrec round = json_number
  type nonrec side = save35_setup_first
  type nonrec winner = fragments_native_state38_winner
  type nonrec armies = fragments_native_state38_armies_member1
  type nonrec condition_memory = json_string
  type nonrec map_id = json_string
  type nonrec map_revision = json_number
  type nonrec cols = json_number
  type nonrec rows = json_number
  type nonrec next_id = json_number
  type nonrec log = json_string
  type nonrec last_combat = fragments_native_state38_last_combat_member2
  type nonrec unit_ = save2_checkpoint_state_units_item
  type nonrec tile = fragments_tiles11_item
  type nonrec campaign = save2_checkpoint_state_campaign_member1
  type nonrec gold = javascript1_state_gold
  type nonrec faction = fragments_factions7_item
  type nonrec ground_items = fragments_native_state38_ground_items_item
  type nonrec combat_rng = json_number
  type nonrec policy = save35_setup_enemy_policy
  type nonrec blocks = save35_setup_enemy_blocks_member1
  type nonrec recruitment_limit = json_number
  type nonrec diplomacy = save2_setup_member1_enemy_diplomacy
  type nonrec id = json_string
  type nonrec saved_at = json_number
  type nonrec run = json_string
  type nonrec mission = json_string
  type nonrec mission_start = json_boolean
  type nonrec run_id = json_string
  type nonrec branch_id = json_string
  type nonrec checkpoint_id = json_string
  type nonrec ordinal = json_number
  type nonrec kind = history1_entries_item_kind
  type nonrec parent_id = json_string
  type nonrec attacker_id = json_string
  type nonrec attack_id = json_string
  type nonrec sound = json_string
  type nonrec role = fragments_native_state38_units_item_role
  type nonrec items = fragments_native_state38_ground_items_item_item
  type nonrec ai_group = json_string
  type nonrec type_ = json_string
  type nonrec hp = json_number
  type nonrec moved = json_boolean
  type nonrec attacked = json_boolean
  type nonrec retreating = fragments_rule11_remember
  type nonrec rebuilding = fragments_rule11_remember
  type nonrec name = json_string
  type nonrec character = json_string
  type nonrec experience = json_number
  type nonrec owner = json_string
  type nonrec location = json_string
  type nonrec col = json_number
  type nonrec row = json_number
  type nonrec terrain = json_string
  type nonrec connection = fragments_tiles11_item_connections_member1_item
  type nonrec dead_ends = fragments_tiles11_item_connections_member1_item
  type nonrec walls = fragments_tiles11_item_connections_member1_item
  type nonrec floor = json_string
  type nonrec fog = fragments_rule11
  type nonrec light = fragments_light23
  type nonrec wall_corners = json_number
  type nonrec vigil = fragments_native_state38_campaign_member1_vigil_member1
  type nonrec refuge = fragments_native_state38_campaign_member1_refuge_member1
  type nonrec completed = json_boolean
  type nonrec mark = json_string
  type nonrec hold = fragments_native_state38_campaign_member1_holds_member1
  type nonrec recall = json_string
  type nonrec fallen_companions = json_string
  type nonrec carry_gold = json_number
  type nonrec dialogue = fragments_native_state38_campaign_member1_dialogue_item
  type nonrec dialogue_index = json_number
  type nonrec outcome = json_string
  type nonrec veteran = save2_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec story_veteran = save2_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec evacuated = save2_checkpoint_state_campaign_member1_evacuated_member1_item
  type nonrec choices = json_string
  type nonrec player = json_number
  type nonrec phase = save35_setup_first
  type nonrec controller = fragments_factions7_item_controller
  type nonrec palette = json_string
  type nonrec recruits = fragments_factions7_item_recruits_member1_item
  type nonrec relations = fragments_factions7_item_relations_member1
  type nonrec transfer = fragments_factions7_item_transfer_member1
  type nonrec item = fragments_native_state38_ground_items_item_item
  type nonrec recruiter = save35_setup_enemy_blocks_member1_recruiter
  type nonrec planner = save35_setup_enemy_blocks_member1_planner
  type nonrec armor = fragments_native_state38_ground_items_item_item_armor_member1
  type nonrec description = json_string
  type nonrec attacks = fragments_native_state38_ground_items_item_item_attacks_item
  type nonrec healing = fragments_native_state38_ground_items_item_item_healing_member1
  type nonrec condition = fragments_rule11_condition
  type nonrec remember = fragments_rule11_remember
  type nonrec radius = json_number
  type nonrec color = fragments_light23_color_member1
  type nonrec fixture = fragments_rule11_remember
  type nonrec dx = json_number
  type nonrec dy = json_number
  type nonrec power = json_number
  type nonrec rule = fragments_rule11
  type nonrec held_turns = json_number
  type nonrec last_counted_round = json_number
  type nonrec escalated = json_boolean
  type nonrec speaker = json_string
  type nonrec portrait = json_string
  type nonrec text = json_string
  type nonrec event = json_string
  type nonrec controller_case = fragments_factions7_item_controller_member0
  type nonrec cost = json_number
  type nonrec to_ = json_string
  type nonrec blade = json_number
  type nonrec pierce = json_number
  type nonrec impact = json_number
  type nonrec damage = json_number
  type nonrec spread = json_number
  type nonrec count = json_number
  type nonrec damage_type = fragments_native_state38_ground_items_item_item_attacks_item_damage_type
  type nonrec category = fragments_native_state38_ground_items_item_item_attacks_item_category
  type nonrec range = json_number
  type nonrec drain = json_number
  type nonrec amount = json_number
  type nonrec condition_case = fragments_rule11_condition_member0
  type nonrec unit_types = json_string
  type nonrec choice = json_string
  type nonrec flag = json_string
  type nonrec unit_type = json_string
  type nonrec conditions = fragments_rule11_condition
end

module V7 = struct
  type nonrec t = save7
  type nonrec version = literal_7
  type nonrec checkpoint = save7_checkpoint
  type nonrec pending_enemy = json_boolean
  type nonrec setup = save2_setup_member1
  type nonrec progress = save2_progress_member1
  type nonrec undo = json_unknown
  type nonrec seed = json_number
  type nonrec state = save7_checkpoint_state
  type nonrec first = save35_setup_first
  type nonrec enemy = save2_setup_member1_enemy
  type nonrec entry = history1_entries_item
  type nonrec round = json_number
  type nonrec side = save35_setup_first
  type nonrec winner = fragments_native_state38_winner
  type nonrec armies = fragments_native_state38_armies_member1
  type nonrec condition_memory = json_string
  type nonrec map_id = json_string
  type nonrec map_revision = json_number
  type nonrec cols = json_number
  type nonrec rows = json_number
  type nonrec next_id = json_number
  type nonrec log = json_string
  type nonrec last_combat = fragments_native_state38_last_combat_member2
  type nonrec unit_ = save2_checkpoint_state_units_item
  type nonrec tile = fragments_tiles11_item
  type nonrec campaign = save2_checkpoint_state_campaign_member1
  type nonrec gold = javascript1_state_gold
  type nonrec faction = fragments_factions7_item
  type nonrec ground_items = fragments_native_state38_ground_items_item
  type nonrec combat_rng = json_number
  type nonrec policy = save35_setup_enemy_policy
  type nonrec blocks = save35_setup_enemy_blocks_member1
  type nonrec recruitment_limit = json_number
  type nonrec diplomacy = save2_setup_member1_enemy_diplomacy
  type nonrec id = json_string
  type nonrec saved_at = json_number
  type nonrec run = json_string
  type nonrec mission = json_string
  type nonrec mission_start = json_boolean
  type nonrec run_id = json_string
  type nonrec branch_id = json_string
  type nonrec checkpoint_id = json_string
  type nonrec ordinal = json_number
  type nonrec kind = history1_entries_item_kind
  type nonrec parent_id = json_string
  type nonrec attacker_id = json_string
  type nonrec attack_id = json_string
  type nonrec sound = json_string
  type nonrec role = fragments_native_state38_units_item_role
  type nonrec items = fragments_native_state38_ground_items_item_item
  type nonrec ai_group = json_string
  type nonrec type_ = json_string
  type nonrec hp = json_number
  type nonrec moved = json_boolean
  type nonrec attacked = json_boolean
  type nonrec retreating = fragments_rule11_remember
  type nonrec rebuilding = fragments_rule11_remember
  type nonrec name = json_string
  type nonrec character = json_string
  type nonrec experience = json_number
  type nonrec owner = json_string
  type nonrec location = json_string
  type nonrec col = json_number
  type nonrec row = json_number
  type nonrec terrain = json_string
  type nonrec connection = fragments_tiles11_item_connections_member1_item
  type nonrec dead_ends = fragments_tiles11_item_connections_member1_item
  type nonrec walls = fragments_tiles11_item_connections_member1_item
  type nonrec floor = json_string
  type nonrec fog = fragments_rule11
  type nonrec light = fragments_light23
  type nonrec wall_corners = json_number
  type nonrec vigil = fragments_native_state38_campaign_member1_vigil_member1
  type nonrec refuge = fragments_native_state38_campaign_member1_refuge_member1
  type nonrec completed = json_boolean
  type nonrec mark = json_string
  type nonrec hold = fragments_native_state38_campaign_member1_holds_member1
  type nonrec recall = json_string
  type nonrec fallen_companions = json_string
  type nonrec carry_gold = json_number
  type nonrec dialogue = fragments_native_state38_campaign_member1_dialogue_item
  type nonrec dialogue_index = json_number
  type nonrec outcome = json_string
  type nonrec veteran = save2_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec story_veteran = save2_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec evacuated = save2_checkpoint_state_campaign_member1_evacuated_member1_item
  type nonrec choices = json_string
  type nonrec player = json_number
  type nonrec phase = save35_setup_first
  type nonrec controller = fragments_factions7_item_controller
  type nonrec palette = json_string
  type nonrec recruits = fragments_factions7_item_recruits_member1_item
  type nonrec relations = fragments_factions7_item_relations_member1
  type nonrec transfer = fragments_factions7_item_transfer_member1
  type nonrec item = fragments_native_state38_ground_items_item_item
  type nonrec recruiter = save35_setup_enemy_blocks_member1_recruiter
  type nonrec planner = save35_setup_enemy_blocks_member1_planner
  type nonrec armor = fragments_native_state38_ground_items_item_item_armor_member1
  type nonrec description = json_string
  type nonrec attacks = fragments_native_state38_ground_items_item_item_attacks_item
  type nonrec healing = fragments_native_state38_ground_items_item_item_healing_member1
  type nonrec condition = fragments_rule11_condition
  type nonrec remember = fragments_rule11_remember
  type nonrec radius = json_number
  type nonrec color = fragments_light23_color_member1
  type nonrec fixture = fragments_rule11_remember
  type nonrec dx = json_number
  type nonrec dy = json_number
  type nonrec power = json_number
  type nonrec rule = fragments_rule11
  type nonrec held_turns = json_number
  type nonrec last_counted_round = json_number
  type nonrec escalated = json_boolean
  type nonrec speaker = json_string
  type nonrec portrait = json_string
  type nonrec text = json_string
  type nonrec event = json_string
  type nonrec controller_case = fragments_factions7_item_controller_member0
  type nonrec cost = json_number
  type nonrec to_ = json_string
  type nonrec blade = json_number
  type nonrec pierce = json_number
  type nonrec impact = json_number
  type nonrec damage = json_number
  type nonrec spread = json_number
  type nonrec count = json_number
  type nonrec damage_type = fragments_native_state38_ground_items_item_item_attacks_item_damage_type
  type nonrec category = fragments_native_state38_ground_items_item_item_attacks_item_category
  type nonrec range = json_number
  type nonrec drain = json_number
  type nonrec amount = json_number
  type nonrec condition_case = fragments_rule11_condition_member0
  type nonrec unit_types = json_string
  type nonrec choice = json_string
  type nonrec flag = json_string
  type nonrec unit_type = json_string
  type nonrec conditions = fragments_rule11_condition
end

module V8 = struct
  type nonrec t = save8
  type nonrec version = literal_8
  type nonrec checkpoint = save8_checkpoint
  type nonrec pending_enemy = json_boolean
  type nonrec setup = save2_setup_member1
  type nonrec progress = save2_progress_member1
  type nonrec undo = json_unknown
  type nonrec seed = json_number
  type nonrec state = save8_checkpoint_state
  type nonrec first = save35_setup_first
  type nonrec enemy = save2_setup_member1_enemy
  type nonrec entry = history1_entries_item
  type nonrec round = json_number
  type nonrec side = save35_setup_first
  type nonrec winner = fragments_native_state38_winner
  type nonrec armies = fragments_native_state38_armies_member1
  type nonrec condition_memory = json_string
  type nonrec map_id = json_string
  type nonrec map_revision = json_number
  type nonrec cols = json_number
  type nonrec rows = json_number
  type nonrec next_id = json_number
  type nonrec log = json_string
  type nonrec last_combat = fragments_native_state38_last_combat_member2
  type nonrec unit_ = save2_checkpoint_state_units_item
  type nonrec tile = fragments_tiles11_item
  type nonrec campaign = save2_checkpoint_state_campaign_member1
  type nonrec gold = javascript1_state_gold
  type nonrec faction = fragments_factions7_item
  type nonrec ground_items = fragments_native_state38_ground_items_item
  type nonrec combat_rng = json_number
  type nonrec policy = save35_setup_enemy_policy
  type nonrec blocks = save35_setup_enemy_blocks_member1
  type nonrec recruitment_limit = json_number
  type nonrec diplomacy = save2_setup_member1_enemy_diplomacy
  type nonrec id = json_string
  type nonrec saved_at = json_number
  type nonrec run = json_string
  type nonrec mission = json_string
  type nonrec mission_start = json_boolean
  type nonrec run_id = json_string
  type nonrec branch_id = json_string
  type nonrec checkpoint_id = json_string
  type nonrec ordinal = json_number
  type nonrec kind = history1_entries_item_kind
  type nonrec parent_id = json_string
  type nonrec attacker_id = json_string
  type nonrec attack_id = json_string
  type nonrec sound = json_string
  type nonrec role = fragments_native_state38_units_item_role
  type nonrec items = fragments_native_state38_ground_items_item_item
  type nonrec ai_group = json_string
  type nonrec type_ = json_string
  type nonrec hp = json_number
  type nonrec moved = json_boolean
  type nonrec attacked = json_boolean
  type nonrec retreating = fragments_rule11_remember
  type nonrec rebuilding = fragments_rule11_remember
  type nonrec name = json_string
  type nonrec character = json_string
  type nonrec experience = json_number
  type nonrec owner = json_string
  type nonrec location = json_string
  type nonrec col = json_number
  type nonrec row = json_number
  type nonrec terrain = json_string
  type nonrec connection = fragments_tiles11_item_connections_member1_item
  type nonrec dead_ends = fragments_tiles11_item_connections_member1_item
  type nonrec walls = fragments_tiles11_item_connections_member1_item
  type nonrec floor = json_string
  type nonrec fog = fragments_rule11
  type nonrec light = fragments_light23
  type nonrec wall_corners = json_number
  type nonrec vigil = fragments_native_state38_campaign_member1_vigil_member1
  type nonrec refuge = fragments_native_state38_campaign_member1_refuge_member1
  type nonrec completed = json_boolean
  type nonrec mark = json_string
  type nonrec hold = fragments_native_state38_campaign_member1_holds_member1
  type nonrec recall = json_string
  type nonrec fallen_companions = json_string
  type nonrec carry_gold = json_number
  type nonrec dialogue = fragments_native_state38_campaign_member1_dialogue_item
  type nonrec dialogue_index = json_number
  type nonrec outcome = json_string
  type nonrec veteran = save2_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec story_veteran = save2_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec evacuated = save2_checkpoint_state_campaign_member1_evacuated_member1_item
  type nonrec choices = json_string
  type nonrec player = json_number
  type nonrec phase = save35_setup_first
  type nonrec controller = fragments_factions7_item_controller
  type nonrec palette = json_string
  type nonrec recruits = fragments_factions7_item_recruits_member1_item
  type nonrec relations = fragments_factions7_item_relations_member1
  type nonrec transfer = fragments_factions7_item_transfer_member1
  type nonrec item = fragments_native_state38_ground_items_item_item
  type nonrec recruiter = save35_setup_enemy_blocks_member1_recruiter
  type nonrec planner = save35_setup_enemy_blocks_member1_planner
  type nonrec armor = fragments_native_state38_ground_items_item_item_armor_member1
  type nonrec description = json_string
  type nonrec attacks = fragments_native_state38_ground_items_item_item_attacks_item
  type nonrec healing = fragments_native_state38_ground_items_item_item_healing_member1
  type nonrec condition = fragments_rule11_condition
  type nonrec remember = fragments_rule11_remember
  type nonrec radius = json_number
  type nonrec color = fragments_light23_color_member1
  type nonrec fixture = fragments_rule11_remember
  type nonrec dx = json_number
  type nonrec dy = json_number
  type nonrec power = json_number
  type nonrec rule = fragments_rule11
  type nonrec held_turns = json_number
  type nonrec last_counted_round = json_number
  type nonrec escalated = json_boolean
  type nonrec speaker = json_string
  type nonrec portrait = json_string
  type nonrec text = json_string
  type nonrec event = json_string
  type nonrec controller_case = fragments_factions7_item_controller_member0
  type nonrec cost = json_number
  type nonrec to_ = json_string
  type nonrec blade = json_number
  type nonrec pierce = json_number
  type nonrec impact = json_number
  type nonrec damage = json_number
  type nonrec spread = json_number
  type nonrec count = json_number
  type nonrec damage_type = fragments_native_state38_ground_items_item_item_attacks_item_damage_type
  type nonrec category = fragments_native_state38_ground_items_item_item_attacks_item_category
  type nonrec range = json_number
  type nonrec drain = json_number
  type nonrec amount = json_number
  type nonrec condition_case = fragments_rule11_condition_member0
  type nonrec unit_types = json_string
  type nonrec choice = json_string
  type nonrec flag = json_string
  type nonrec unit_type = json_string
  type nonrec conditions = fragments_rule11_condition
end

module V9 = struct
  type nonrec t = save9
  type nonrec version = literal_9
  type nonrec checkpoint = save8_checkpoint
  type nonrec pending_enemy = json_boolean
  type nonrec setup = save2_setup_member1
  type nonrec progress = save2_progress_member1
  type nonrec undo = json_unknown
  type nonrec seed = json_number
  type nonrec state = save8_checkpoint_state
  type nonrec first = save35_setup_first
  type nonrec enemy = save2_setup_member1_enemy
  type nonrec entry = history1_entries_item
  type nonrec round = json_number
  type nonrec side = save35_setup_first
  type nonrec winner = fragments_native_state38_winner
  type nonrec armies = fragments_native_state38_armies_member1
  type nonrec condition_memory = json_string
  type nonrec map_id = json_string
  type nonrec map_revision = json_number
  type nonrec cols = json_number
  type nonrec rows = json_number
  type nonrec next_id = json_number
  type nonrec log = json_string
  type nonrec last_combat = fragments_native_state38_last_combat_member2
  type nonrec unit_ = save2_checkpoint_state_units_item
  type nonrec tile = fragments_tiles11_item
  type nonrec campaign = save2_checkpoint_state_campaign_member1
  type nonrec gold = javascript1_state_gold
  type nonrec faction = fragments_factions7_item
  type nonrec ground_items = fragments_native_state38_ground_items_item
  type nonrec combat_rng = json_number
  type nonrec policy = save35_setup_enemy_policy
  type nonrec blocks = save35_setup_enemy_blocks_member1
  type nonrec recruitment_limit = json_number
  type nonrec diplomacy = save2_setup_member1_enemy_diplomacy
  type nonrec id = json_string
  type nonrec saved_at = json_number
  type nonrec run = json_string
  type nonrec mission = json_string
  type nonrec mission_start = json_boolean
  type nonrec run_id = json_string
  type nonrec branch_id = json_string
  type nonrec checkpoint_id = json_string
  type nonrec ordinal = json_number
  type nonrec kind = history1_entries_item_kind
  type nonrec parent_id = json_string
  type nonrec attacker_id = json_string
  type nonrec attack_id = json_string
  type nonrec sound = json_string
  type nonrec role = fragments_native_state38_units_item_role
  type nonrec items = fragments_native_state38_ground_items_item_item
  type nonrec ai_group = json_string
  type nonrec type_ = json_string
  type nonrec hp = json_number
  type nonrec moved = json_boolean
  type nonrec attacked = json_boolean
  type nonrec retreating = fragments_rule11_remember
  type nonrec rebuilding = fragments_rule11_remember
  type nonrec name = json_string
  type nonrec character = json_string
  type nonrec experience = json_number
  type nonrec owner = json_string
  type nonrec location = json_string
  type nonrec col = json_number
  type nonrec row = json_number
  type nonrec terrain = json_string
  type nonrec connection = fragments_tiles11_item_connections_member1_item
  type nonrec dead_ends = fragments_tiles11_item_connections_member1_item
  type nonrec walls = fragments_tiles11_item_connections_member1_item
  type nonrec floor = json_string
  type nonrec fog = fragments_rule11
  type nonrec light = fragments_light23
  type nonrec wall_corners = json_number
  type nonrec vigil = fragments_native_state38_campaign_member1_vigil_member1
  type nonrec refuge = fragments_native_state38_campaign_member1_refuge_member1
  type nonrec completed = json_boolean
  type nonrec mark = json_string
  type nonrec hold = fragments_native_state38_campaign_member1_holds_member1
  type nonrec recall = json_string
  type nonrec fallen_companions = json_string
  type nonrec carry_gold = json_number
  type nonrec dialogue = fragments_native_state38_campaign_member1_dialogue_item
  type nonrec dialogue_index = json_number
  type nonrec outcome = json_string
  type nonrec veteran = save2_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec story_veteran = save2_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec evacuated = save2_checkpoint_state_campaign_member1_evacuated_member1_item
  type nonrec choices = json_string
  type nonrec player = json_number
  type nonrec phase = save35_setup_first
  type nonrec controller = fragments_factions7_item_controller
  type nonrec palette = json_string
  type nonrec recruits = fragments_factions7_item_recruits_member1_item
  type nonrec relations = fragments_factions7_item_relations_member1
  type nonrec transfer = fragments_factions7_item_transfer_member1
  type nonrec item = fragments_native_state38_ground_items_item_item
  type nonrec recruiter = save35_setup_enemy_blocks_member1_recruiter
  type nonrec planner = save35_setup_enemy_blocks_member1_planner
  type nonrec armor = fragments_native_state38_ground_items_item_item_armor_member1
  type nonrec description = json_string
  type nonrec attacks = fragments_native_state38_ground_items_item_item_attacks_item
  type nonrec healing = fragments_native_state38_ground_items_item_item_healing_member1
  type nonrec condition = fragments_rule11_condition
  type nonrec remember = fragments_rule11_remember
  type nonrec radius = json_number
  type nonrec color = fragments_light23_color_member1
  type nonrec fixture = fragments_rule11_remember
  type nonrec dx = json_number
  type nonrec dy = json_number
  type nonrec power = json_number
  type nonrec rule = fragments_rule11
  type nonrec held_turns = json_number
  type nonrec last_counted_round = json_number
  type nonrec escalated = json_boolean
  type nonrec speaker = json_string
  type nonrec portrait = json_string
  type nonrec text = json_string
  type nonrec event = json_string
  type nonrec controller_case = fragments_factions7_item_controller_member0
  type nonrec cost = json_number
  type nonrec to_ = json_string
  type nonrec blade = json_number
  type nonrec pierce = json_number
  type nonrec impact = json_number
  type nonrec damage = json_number
  type nonrec spread = json_number
  type nonrec count = json_number
  type nonrec damage_type = fragments_native_state38_ground_items_item_item_attacks_item_damage_type
  type nonrec category = fragments_native_state38_ground_items_item_item_attacks_item_category
  type nonrec range = json_number
  type nonrec drain = json_number
  type nonrec amount = json_number
  type nonrec condition_case = fragments_rule11_condition_member0
  type nonrec unit_types = json_string
  type nonrec choice = json_string
  type nonrec flag = json_string
  type nonrec unit_type = json_string
  type nonrec conditions = fragments_rule11_condition
end

module V10 = struct
  type nonrec t = save10
  type nonrec version = literal_10
  type nonrec checkpoint = save8_checkpoint
  type nonrec pending_enemy = json_boolean
  type nonrec setup = save2_setup_member1
  type nonrec progress = save2_progress_member1
  type nonrec undo = json_unknown
  type nonrec seed = json_number
  type nonrec state = save8_checkpoint_state
  type nonrec first = save35_setup_first
  type nonrec enemy = save2_setup_member1_enemy
  type nonrec entry = history1_entries_item
  type nonrec round = json_number
  type nonrec side = save35_setup_first
  type nonrec winner = fragments_native_state38_winner
  type nonrec armies = fragments_native_state38_armies_member1
  type nonrec condition_memory = json_string
  type nonrec map_id = json_string
  type nonrec map_revision = json_number
  type nonrec cols = json_number
  type nonrec rows = json_number
  type nonrec next_id = json_number
  type nonrec log = json_string
  type nonrec last_combat = fragments_native_state38_last_combat_member2
  type nonrec unit_ = save2_checkpoint_state_units_item
  type nonrec tile = fragments_tiles11_item
  type nonrec campaign = save2_checkpoint_state_campaign_member1
  type nonrec gold = javascript1_state_gold
  type nonrec faction = fragments_factions7_item
  type nonrec ground_items = fragments_native_state38_ground_items_item
  type nonrec combat_rng = json_number
  type nonrec policy = save35_setup_enemy_policy
  type nonrec blocks = save35_setup_enemy_blocks_member1
  type nonrec recruitment_limit = json_number
  type nonrec diplomacy = save2_setup_member1_enemy_diplomacy
  type nonrec id = json_string
  type nonrec saved_at = json_number
  type nonrec run = json_string
  type nonrec mission = json_string
  type nonrec mission_start = json_boolean
  type nonrec run_id = json_string
  type nonrec branch_id = json_string
  type nonrec checkpoint_id = json_string
  type nonrec ordinal = json_number
  type nonrec kind = history1_entries_item_kind
  type nonrec parent_id = json_string
  type nonrec attacker_id = json_string
  type nonrec attack_id = json_string
  type nonrec sound = json_string
  type nonrec role = fragments_native_state38_units_item_role
  type nonrec items = fragments_native_state38_ground_items_item_item
  type nonrec ai_group = json_string
  type nonrec type_ = json_string
  type nonrec hp = json_number
  type nonrec moved = json_boolean
  type nonrec attacked = json_boolean
  type nonrec retreating = fragments_rule11_remember
  type nonrec rebuilding = fragments_rule11_remember
  type nonrec name = json_string
  type nonrec character = json_string
  type nonrec experience = json_number
  type nonrec owner = json_string
  type nonrec location = json_string
  type nonrec col = json_number
  type nonrec row = json_number
  type nonrec terrain = json_string
  type nonrec connection = fragments_tiles11_item_connections_member1_item
  type nonrec dead_ends = fragments_tiles11_item_connections_member1_item
  type nonrec walls = fragments_tiles11_item_connections_member1_item
  type nonrec floor = json_string
  type nonrec fog = fragments_rule11
  type nonrec light = fragments_light23
  type nonrec wall_corners = json_number
  type nonrec vigil = fragments_native_state38_campaign_member1_vigil_member1
  type nonrec refuge = fragments_native_state38_campaign_member1_refuge_member1
  type nonrec completed = json_boolean
  type nonrec mark = json_string
  type nonrec hold = fragments_native_state38_campaign_member1_holds_member1
  type nonrec recall = json_string
  type nonrec fallen_companions = json_string
  type nonrec carry_gold = json_number
  type nonrec dialogue = fragments_native_state38_campaign_member1_dialogue_item
  type nonrec dialogue_index = json_number
  type nonrec outcome = json_string
  type nonrec veteran = save2_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec story_veteran = save2_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec evacuated = save2_checkpoint_state_campaign_member1_evacuated_member1_item
  type nonrec choices = json_string
  type nonrec player = json_number
  type nonrec phase = save35_setup_first
  type nonrec controller = fragments_factions7_item_controller
  type nonrec palette = json_string
  type nonrec recruits = fragments_factions7_item_recruits_member1_item
  type nonrec relations = fragments_factions7_item_relations_member1
  type nonrec transfer = fragments_factions7_item_transfer_member1
  type nonrec item = fragments_native_state38_ground_items_item_item
  type nonrec recruiter = save35_setup_enemy_blocks_member1_recruiter
  type nonrec planner = save35_setup_enemy_blocks_member1_planner
  type nonrec armor = fragments_native_state38_ground_items_item_item_armor_member1
  type nonrec description = json_string
  type nonrec attacks = fragments_native_state38_ground_items_item_item_attacks_item
  type nonrec healing = fragments_native_state38_ground_items_item_item_healing_member1
  type nonrec condition = fragments_rule11_condition
  type nonrec remember = fragments_rule11_remember
  type nonrec radius = json_number
  type nonrec color = fragments_light23_color_member1
  type nonrec fixture = fragments_rule11_remember
  type nonrec dx = json_number
  type nonrec dy = json_number
  type nonrec power = json_number
  type nonrec rule = fragments_rule11
  type nonrec held_turns = json_number
  type nonrec last_counted_round = json_number
  type nonrec escalated = json_boolean
  type nonrec speaker = json_string
  type nonrec portrait = json_string
  type nonrec text = json_string
  type nonrec event = json_string
  type nonrec controller_case = fragments_factions7_item_controller_member0
  type nonrec cost = json_number
  type nonrec to_ = json_string
  type nonrec blade = json_number
  type nonrec pierce = json_number
  type nonrec impact = json_number
  type nonrec damage = json_number
  type nonrec spread = json_number
  type nonrec count = json_number
  type nonrec damage_type = fragments_native_state38_ground_items_item_item_attacks_item_damage_type
  type nonrec category = fragments_native_state38_ground_items_item_item_attacks_item_category
  type nonrec range = json_number
  type nonrec drain = json_number
  type nonrec amount = json_number
  type nonrec condition_case = fragments_rule11_condition_member0
  type nonrec unit_types = json_string
  type nonrec choice = json_string
  type nonrec flag = json_string
  type nonrec unit_type = json_string
  type nonrec conditions = fragments_rule11_condition
end

module V11 = struct
  type nonrec t = save11
  type nonrec version = literal_11
  type nonrec checkpoint = save8_checkpoint
  type nonrec pending_enemy = json_boolean
  type nonrec setup = save2_setup_member1
  type nonrec progress = save2_progress_member1
  type nonrec undo = json_unknown
  type nonrec seed = json_number
  type nonrec state = save8_checkpoint_state
  type nonrec first = save35_setup_first
  type nonrec enemy = save2_setup_member1_enemy
  type nonrec entry = history1_entries_item
  type nonrec round = json_number
  type nonrec side = save35_setup_first
  type nonrec winner = fragments_native_state38_winner
  type nonrec armies = fragments_native_state38_armies_member1
  type nonrec condition_memory = json_string
  type nonrec map_id = json_string
  type nonrec map_revision = json_number
  type nonrec cols = json_number
  type nonrec rows = json_number
  type nonrec next_id = json_number
  type nonrec log = json_string
  type nonrec last_combat = fragments_native_state38_last_combat_member2
  type nonrec unit_ = save2_checkpoint_state_units_item
  type nonrec tile = fragments_tiles11_item
  type nonrec campaign = save2_checkpoint_state_campaign_member1
  type nonrec gold = javascript1_state_gold
  type nonrec faction = fragments_factions7_item
  type nonrec ground_items = fragments_native_state38_ground_items_item
  type nonrec combat_rng = json_number
  type nonrec policy = save35_setup_enemy_policy
  type nonrec blocks = save35_setup_enemy_blocks_member1
  type nonrec recruitment_limit = json_number
  type nonrec diplomacy = save2_setup_member1_enemy_diplomacy
  type nonrec id = json_string
  type nonrec saved_at = json_number
  type nonrec run = json_string
  type nonrec mission = json_string
  type nonrec mission_start = json_boolean
  type nonrec run_id = json_string
  type nonrec branch_id = json_string
  type nonrec checkpoint_id = json_string
  type nonrec ordinal = json_number
  type nonrec kind = history1_entries_item_kind
  type nonrec parent_id = json_string
  type nonrec attacker_id = json_string
  type nonrec attack_id = json_string
  type nonrec sound = json_string
  type nonrec role = fragments_native_state38_units_item_role
  type nonrec items = fragments_native_state38_ground_items_item_item
  type nonrec ai_group = json_string
  type nonrec type_ = json_string
  type nonrec hp = json_number
  type nonrec moved = json_boolean
  type nonrec attacked = json_boolean
  type nonrec retreating = fragments_rule11_remember
  type nonrec rebuilding = fragments_rule11_remember
  type nonrec name = json_string
  type nonrec character = json_string
  type nonrec experience = json_number
  type nonrec owner = json_string
  type nonrec location = json_string
  type nonrec col = json_number
  type nonrec row = json_number
  type nonrec terrain = json_string
  type nonrec connection = fragments_tiles11_item_connections_member1_item
  type nonrec dead_ends = fragments_tiles11_item_connections_member1_item
  type nonrec walls = fragments_tiles11_item_connections_member1_item
  type nonrec floor = json_string
  type nonrec fog = fragments_rule11
  type nonrec light = fragments_light23
  type nonrec wall_corners = json_number
  type nonrec vigil = fragments_native_state38_campaign_member1_vigil_member1
  type nonrec refuge = fragments_native_state38_campaign_member1_refuge_member1
  type nonrec completed = json_boolean
  type nonrec mark = json_string
  type nonrec hold = fragments_native_state38_campaign_member1_holds_member1
  type nonrec recall = json_string
  type nonrec fallen_companions = json_string
  type nonrec carry_gold = json_number
  type nonrec dialogue = fragments_native_state38_campaign_member1_dialogue_item
  type nonrec dialogue_index = json_number
  type nonrec outcome = json_string
  type nonrec veteran = save2_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec story_veteran = save2_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec evacuated = save2_checkpoint_state_campaign_member1_evacuated_member1_item
  type nonrec choices = json_string
  type nonrec player = json_number
  type nonrec phase = save35_setup_first
  type nonrec controller = fragments_factions7_item_controller
  type nonrec palette = json_string
  type nonrec recruits = fragments_factions7_item_recruits_member1_item
  type nonrec relations = fragments_factions7_item_relations_member1
  type nonrec transfer = fragments_factions7_item_transfer_member1
  type nonrec item = fragments_native_state38_ground_items_item_item
  type nonrec recruiter = save35_setup_enemy_blocks_member1_recruiter
  type nonrec planner = save35_setup_enemy_blocks_member1_planner
  type nonrec armor = fragments_native_state38_ground_items_item_item_armor_member1
  type nonrec description = json_string
  type nonrec attacks = fragments_native_state38_ground_items_item_item_attacks_item
  type nonrec healing = fragments_native_state38_ground_items_item_item_healing_member1
  type nonrec condition = fragments_rule11_condition
  type nonrec remember = fragments_rule11_remember
  type nonrec radius = json_number
  type nonrec color = fragments_light23_color_member1
  type nonrec fixture = fragments_rule11_remember
  type nonrec dx = json_number
  type nonrec dy = json_number
  type nonrec power = json_number
  type nonrec rule = fragments_rule11
  type nonrec held_turns = json_number
  type nonrec last_counted_round = json_number
  type nonrec escalated = json_boolean
  type nonrec speaker = json_string
  type nonrec portrait = json_string
  type nonrec text = json_string
  type nonrec event = json_string
  type nonrec controller_case = fragments_factions7_item_controller_member0
  type nonrec cost = json_number
  type nonrec to_ = json_string
  type nonrec blade = json_number
  type nonrec pierce = json_number
  type nonrec impact = json_number
  type nonrec damage = json_number
  type nonrec spread = json_number
  type nonrec count = json_number
  type nonrec damage_type = fragments_native_state38_ground_items_item_item_attacks_item_damage_type
  type nonrec category = fragments_native_state38_ground_items_item_item_attacks_item_category
  type nonrec range = json_number
  type nonrec drain = json_number
  type nonrec amount = json_number
  type nonrec condition_case = fragments_rule11_condition_member0
  type nonrec unit_types = json_string
  type nonrec choice = json_string
  type nonrec flag = json_string
  type nonrec unit_type = json_string
  type nonrec conditions = fragments_rule11_condition
end

module V12 = struct
  type nonrec t = save12
  type nonrec version = literal_12
  type nonrec checkpoint = save8_checkpoint
  type nonrec pending_enemy = json_boolean
  type nonrec setup = save2_setup_member1
  type nonrec progress = save2_progress_member1
  type nonrec undo = json_unknown
  type nonrec seed = json_number
  type nonrec state = save8_checkpoint_state
  type nonrec first = save35_setup_first
  type nonrec enemy = save2_setup_member1_enemy
  type nonrec entry = history1_entries_item
  type nonrec round = json_number
  type nonrec side = save35_setup_first
  type nonrec winner = fragments_native_state38_winner
  type nonrec armies = fragments_native_state38_armies_member1
  type nonrec condition_memory = json_string
  type nonrec map_id = json_string
  type nonrec map_revision = json_number
  type nonrec cols = json_number
  type nonrec rows = json_number
  type nonrec next_id = json_number
  type nonrec log = json_string
  type nonrec last_combat = fragments_native_state38_last_combat_member2
  type nonrec unit_ = save2_checkpoint_state_units_item
  type nonrec tile = fragments_tiles11_item
  type nonrec campaign = save2_checkpoint_state_campaign_member1
  type nonrec gold = javascript1_state_gold
  type nonrec faction = fragments_factions7_item
  type nonrec ground_items = fragments_native_state38_ground_items_item
  type nonrec combat_rng = json_number
  type nonrec policy = save35_setup_enemy_policy
  type nonrec blocks = save35_setup_enemy_blocks_member1
  type nonrec recruitment_limit = json_number
  type nonrec diplomacy = save2_setup_member1_enemy_diplomacy
  type nonrec id = json_string
  type nonrec saved_at = json_number
  type nonrec run = json_string
  type nonrec mission = json_string
  type nonrec mission_start = json_boolean
  type nonrec run_id = json_string
  type nonrec branch_id = json_string
  type nonrec checkpoint_id = json_string
  type nonrec ordinal = json_number
  type nonrec kind = history1_entries_item_kind
  type nonrec parent_id = json_string
  type nonrec attacker_id = json_string
  type nonrec attack_id = json_string
  type nonrec sound = json_string
  type nonrec role = fragments_native_state38_units_item_role
  type nonrec items = fragments_native_state38_ground_items_item_item
  type nonrec ai_group = json_string
  type nonrec type_ = json_string
  type nonrec hp = json_number
  type nonrec moved = json_boolean
  type nonrec attacked = json_boolean
  type nonrec retreating = fragments_rule11_remember
  type nonrec rebuilding = fragments_rule11_remember
  type nonrec name = json_string
  type nonrec character = json_string
  type nonrec experience = json_number
  type nonrec owner = json_string
  type nonrec location = json_string
  type nonrec col = json_number
  type nonrec row = json_number
  type nonrec terrain = json_string
  type nonrec connection = fragments_tiles11_item_connections_member1_item
  type nonrec dead_ends = fragments_tiles11_item_connections_member1_item
  type nonrec walls = fragments_tiles11_item_connections_member1_item
  type nonrec floor = json_string
  type nonrec fog = fragments_rule11
  type nonrec light = fragments_light23
  type nonrec wall_corners = json_number
  type nonrec vigil = fragments_native_state38_campaign_member1_vigil_member1
  type nonrec refuge = fragments_native_state38_campaign_member1_refuge_member1
  type nonrec completed = json_boolean
  type nonrec mark = json_string
  type nonrec hold = fragments_native_state38_campaign_member1_holds_member1
  type nonrec recall = json_string
  type nonrec fallen_companions = json_string
  type nonrec carry_gold = json_number
  type nonrec dialogue = fragments_native_state38_campaign_member1_dialogue_item
  type nonrec dialogue_index = json_number
  type nonrec outcome = json_string
  type nonrec veteran = save2_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec story_veteran = save2_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec evacuated = save2_checkpoint_state_campaign_member1_evacuated_member1_item
  type nonrec choices = json_string
  type nonrec player = json_number
  type nonrec phase = save35_setup_first
  type nonrec controller = fragments_factions7_item_controller
  type nonrec palette = json_string
  type nonrec recruits = fragments_factions7_item_recruits_member1_item
  type nonrec relations = fragments_factions7_item_relations_member1
  type nonrec transfer = fragments_factions7_item_transfer_member1
  type nonrec item = fragments_native_state38_ground_items_item_item
  type nonrec recruiter = save35_setup_enemy_blocks_member1_recruiter
  type nonrec planner = save35_setup_enemy_blocks_member1_planner
  type nonrec armor = fragments_native_state38_ground_items_item_item_armor_member1
  type nonrec description = json_string
  type nonrec attacks = fragments_native_state38_ground_items_item_item_attacks_item
  type nonrec healing = fragments_native_state38_ground_items_item_item_healing_member1
  type nonrec condition = fragments_rule11_condition
  type nonrec remember = fragments_rule11_remember
  type nonrec radius = json_number
  type nonrec color = fragments_light23_color_member1
  type nonrec fixture = fragments_rule11_remember
  type nonrec dx = json_number
  type nonrec dy = json_number
  type nonrec power = json_number
  type nonrec rule = fragments_rule11
  type nonrec held_turns = json_number
  type nonrec last_counted_round = json_number
  type nonrec escalated = json_boolean
  type nonrec speaker = json_string
  type nonrec portrait = json_string
  type nonrec text = json_string
  type nonrec event = json_string
  type nonrec controller_case = fragments_factions7_item_controller_member0
  type nonrec cost = json_number
  type nonrec to_ = json_string
  type nonrec blade = json_number
  type nonrec pierce = json_number
  type nonrec impact = json_number
  type nonrec damage = json_number
  type nonrec spread = json_number
  type nonrec count = json_number
  type nonrec damage_type = fragments_native_state38_ground_items_item_item_attacks_item_damage_type
  type nonrec category = fragments_native_state38_ground_items_item_item_attacks_item_category
  type nonrec range = json_number
  type nonrec drain = json_number
  type nonrec amount = json_number
  type nonrec condition_case = fragments_rule11_condition_member0
  type nonrec unit_types = json_string
  type nonrec choice = json_string
  type nonrec flag = json_string
  type nonrec unit_type = json_string
  type nonrec conditions = fragments_rule11_condition
end

module V13 = struct
  type nonrec t = save13
  type nonrec version = literal_13
  type nonrec checkpoint = save8_checkpoint
  type nonrec pending_enemy = json_boolean
  type nonrec setup = save2_setup_member1
  type nonrec progress = save2_progress_member1
  type nonrec undo = json_unknown
  type nonrec seed = json_number
  type nonrec state = save8_checkpoint_state
  type nonrec first = save35_setup_first
  type nonrec enemy = save2_setup_member1_enemy
  type nonrec entry = history1_entries_item
  type nonrec round = json_number
  type nonrec side = save35_setup_first
  type nonrec winner = fragments_native_state38_winner
  type nonrec armies = fragments_native_state38_armies_member1
  type nonrec condition_memory = json_string
  type nonrec map_id = json_string
  type nonrec map_revision = json_number
  type nonrec cols = json_number
  type nonrec rows = json_number
  type nonrec next_id = json_number
  type nonrec log = json_string
  type nonrec last_combat = fragments_native_state38_last_combat_member2
  type nonrec unit_ = save2_checkpoint_state_units_item
  type nonrec tile = fragments_tiles11_item
  type nonrec campaign = save2_checkpoint_state_campaign_member1
  type nonrec gold = javascript1_state_gold
  type nonrec faction = fragments_factions7_item
  type nonrec ground_items = fragments_native_state38_ground_items_item
  type nonrec combat_rng = json_number
  type nonrec policy = save35_setup_enemy_policy
  type nonrec blocks = save35_setup_enemy_blocks_member1
  type nonrec recruitment_limit = json_number
  type nonrec diplomacy = save2_setup_member1_enemy_diplomacy
  type nonrec id = json_string
  type nonrec saved_at = json_number
  type nonrec run = json_string
  type nonrec mission = json_string
  type nonrec mission_start = json_boolean
  type nonrec run_id = json_string
  type nonrec branch_id = json_string
  type nonrec checkpoint_id = json_string
  type nonrec ordinal = json_number
  type nonrec kind = history1_entries_item_kind
  type nonrec parent_id = json_string
  type nonrec attacker_id = json_string
  type nonrec attack_id = json_string
  type nonrec sound = json_string
  type nonrec role = fragments_native_state38_units_item_role
  type nonrec items = fragments_native_state38_ground_items_item_item
  type nonrec ai_group = json_string
  type nonrec type_ = json_string
  type nonrec hp = json_number
  type nonrec moved = json_boolean
  type nonrec attacked = json_boolean
  type nonrec retreating = fragments_rule11_remember
  type nonrec rebuilding = fragments_rule11_remember
  type nonrec name = json_string
  type nonrec character = json_string
  type nonrec experience = json_number
  type nonrec owner = json_string
  type nonrec location = json_string
  type nonrec col = json_number
  type nonrec row = json_number
  type nonrec terrain = json_string
  type nonrec connection = fragments_tiles11_item_connections_member1_item
  type nonrec dead_ends = fragments_tiles11_item_connections_member1_item
  type nonrec walls = fragments_tiles11_item_connections_member1_item
  type nonrec floor = json_string
  type nonrec fog = fragments_rule11
  type nonrec light = fragments_light23
  type nonrec wall_corners = json_number
  type nonrec vigil = fragments_native_state38_campaign_member1_vigil_member1
  type nonrec refuge = fragments_native_state38_campaign_member1_refuge_member1
  type nonrec completed = json_boolean
  type nonrec mark = json_string
  type nonrec hold = fragments_native_state38_campaign_member1_holds_member1
  type nonrec recall = json_string
  type nonrec fallen_companions = json_string
  type nonrec carry_gold = json_number
  type nonrec dialogue = fragments_native_state38_campaign_member1_dialogue_item
  type nonrec dialogue_index = json_number
  type nonrec outcome = json_string
  type nonrec veteran = save2_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec story_veteran = save2_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec evacuated = save2_checkpoint_state_campaign_member1_evacuated_member1_item
  type nonrec choices = json_string
  type nonrec player = json_number
  type nonrec phase = save35_setup_first
  type nonrec controller = fragments_factions7_item_controller
  type nonrec palette = json_string
  type nonrec recruits = fragments_factions7_item_recruits_member1_item
  type nonrec relations = fragments_factions7_item_relations_member1
  type nonrec transfer = fragments_factions7_item_transfer_member1
  type nonrec item = fragments_native_state38_ground_items_item_item
  type nonrec recruiter = save35_setup_enemy_blocks_member1_recruiter
  type nonrec planner = save35_setup_enemy_blocks_member1_planner
  type nonrec armor = fragments_native_state38_ground_items_item_item_armor_member1
  type nonrec description = json_string
  type nonrec attacks = fragments_native_state38_ground_items_item_item_attacks_item
  type nonrec healing = fragments_native_state38_ground_items_item_item_healing_member1
  type nonrec condition = fragments_rule11_condition
  type nonrec remember = fragments_rule11_remember
  type nonrec radius = json_number
  type nonrec color = fragments_light23_color_member1
  type nonrec fixture = fragments_rule11_remember
  type nonrec dx = json_number
  type nonrec dy = json_number
  type nonrec power = json_number
  type nonrec rule = fragments_rule11
  type nonrec held_turns = json_number
  type nonrec last_counted_round = json_number
  type nonrec escalated = json_boolean
  type nonrec speaker = json_string
  type nonrec portrait = json_string
  type nonrec text = json_string
  type nonrec event = json_string
  type nonrec controller_case = fragments_factions7_item_controller_member0
  type nonrec cost = json_number
  type nonrec to_ = json_string
  type nonrec blade = json_number
  type nonrec pierce = json_number
  type nonrec impact = json_number
  type nonrec damage = json_number
  type nonrec spread = json_number
  type nonrec count = json_number
  type nonrec damage_type = fragments_native_state38_ground_items_item_item_attacks_item_damage_type
  type nonrec category = fragments_native_state38_ground_items_item_item_attacks_item_category
  type nonrec range = json_number
  type nonrec drain = json_number
  type nonrec amount = json_number
  type nonrec condition_case = fragments_rule11_condition_member0
  type nonrec unit_types = json_string
  type nonrec choice = json_string
  type nonrec flag = json_string
  type nonrec unit_type = json_string
  type nonrec conditions = fragments_rule11_condition
end

module V14 = struct
  type nonrec t = save14
  type nonrec version = literal_14
  type nonrec checkpoint = save8_checkpoint
  type nonrec pending_enemy = json_boolean
  type nonrec setup = save2_setup_member1
  type nonrec progress = save2_progress_member1
  type nonrec undo = json_unknown
  type nonrec seed = json_number
  type nonrec state = save8_checkpoint_state
  type nonrec first = save35_setup_first
  type nonrec enemy = save2_setup_member1_enemy
  type nonrec entry = history1_entries_item
  type nonrec round = json_number
  type nonrec side = save35_setup_first
  type nonrec winner = fragments_native_state38_winner
  type nonrec armies = fragments_native_state38_armies_member1
  type nonrec condition_memory = json_string
  type nonrec map_id = json_string
  type nonrec map_revision = json_number
  type nonrec cols = json_number
  type nonrec rows = json_number
  type nonrec next_id = json_number
  type nonrec log = json_string
  type nonrec last_combat = fragments_native_state38_last_combat_member2
  type nonrec unit_ = save2_checkpoint_state_units_item
  type nonrec tile = fragments_tiles11_item
  type nonrec campaign = save2_checkpoint_state_campaign_member1
  type nonrec gold = javascript1_state_gold
  type nonrec faction = fragments_factions7_item
  type nonrec ground_items = fragments_native_state38_ground_items_item
  type nonrec combat_rng = json_number
  type nonrec policy = save35_setup_enemy_policy
  type nonrec blocks = save35_setup_enemy_blocks_member1
  type nonrec recruitment_limit = json_number
  type nonrec diplomacy = save2_setup_member1_enemy_diplomacy
  type nonrec id = json_string
  type nonrec saved_at = json_number
  type nonrec run = json_string
  type nonrec mission = json_string
  type nonrec mission_start = json_boolean
  type nonrec run_id = json_string
  type nonrec branch_id = json_string
  type nonrec checkpoint_id = json_string
  type nonrec ordinal = json_number
  type nonrec kind = history1_entries_item_kind
  type nonrec parent_id = json_string
  type nonrec attacker_id = json_string
  type nonrec attack_id = json_string
  type nonrec sound = json_string
  type nonrec role = fragments_native_state38_units_item_role
  type nonrec items = fragments_native_state38_ground_items_item_item
  type nonrec ai_group = json_string
  type nonrec type_ = json_string
  type nonrec hp = json_number
  type nonrec moved = json_boolean
  type nonrec attacked = json_boolean
  type nonrec retreating = fragments_rule11_remember
  type nonrec rebuilding = fragments_rule11_remember
  type nonrec name = json_string
  type nonrec character = json_string
  type nonrec experience = json_number
  type nonrec owner = json_string
  type nonrec location = json_string
  type nonrec col = json_number
  type nonrec row = json_number
  type nonrec terrain = json_string
  type nonrec connection = fragments_tiles11_item_connections_member1_item
  type nonrec dead_ends = fragments_tiles11_item_connections_member1_item
  type nonrec walls = fragments_tiles11_item_connections_member1_item
  type nonrec floor = json_string
  type nonrec fog = fragments_rule11
  type nonrec light = fragments_light23
  type nonrec wall_corners = json_number
  type nonrec vigil = fragments_native_state38_campaign_member1_vigil_member1
  type nonrec refuge = fragments_native_state38_campaign_member1_refuge_member1
  type nonrec completed = json_boolean
  type nonrec mark = json_string
  type nonrec hold = fragments_native_state38_campaign_member1_holds_member1
  type nonrec recall = json_string
  type nonrec fallen_companions = json_string
  type nonrec carry_gold = json_number
  type nonrec dialogue = fragments_native_state38_campaign_member1_dialogue_item
  type nonrec dialogue_index = json_number
  type nonrec outcome = json_string
  type nonrec veteran = save2_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec story_veteran = save2_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec evacuated = save2_checkpoint_state_campaign_member1_evacuated_member1_item
  type nonrec choices = json_string
  type nonrec player = json_number
  type nonrec phase = save35_setup_first
  type nonrec controller = fragments_factions7_item_controller
  type nonrec palette = json_string
  type nonrec recruits = fragments_factions7_item_recruits_member1_item
  type nonrec relations = fragments_factions7_item_relations_member1
  type nonrec transfer = fragments_factions7_item_transfer_member1
  type nonrec item = fragments_native_state38_ground_items_item_item
  type nonrec recruiter = save35_setup_enemy_blocks_member1_recruiter
  type nonrec planner = save35_setup_enemy_blocks_member1_planner
  type nonrec armor = fragments_native_state38_ground_items_item_item_armor_member1
  type nonrec description = json_string
  type nonrec attacks = fragments_native_state38_ground_items_item_item_attacks_item
  type nonrec healing = fragments_native_state38_ground_items_item_item_healing_member1
  type nonrec condition = fragments_rule11_condition
  type nonrec remember = fragments_rule11_remember
  type nonrec radius = json_number
  type nonrec color = fragments_light23_color_member1
  type nonrec fixture = fragments_rule11_remember
  type nonrec dx = json_number
  type nonrec dy = json_number
  type nonrec power = json_number
  type nonrec rule = fragments_rule11
  type nonrec held_turns = json_number
  type nonrec last_counted_round = json_number
  type nonrec escalated = json_boolean
  type nonrec speaker = json_string
  type nonrec portrait = json_string
  type nonrec text = json_string
  type nonrec event = json_string
  type nonrec controller_case = fragments_factions7_item_controller_member0
  type nonrec cost = json_number
  type nonrec to_ = json_string
  type nonrec blade = json_number
  type nonrec pierce = json_number
  type nonrec impact = json_number
  type nonrec damage = json_number
  type nonrec spread = json_number
  type nonrec count = json_number
  type nonrec damage_type = fragments_native_state38_ground_items_item_item_attacks_item_damage_type
  type nonrec category = fragments_native_state38_ground_items_item_item_attacks_item_category
  type nonrec range = json_number
  type nonrec drain = json_number
  type nonrec amount = json_number
  type nonrec condition_case = fragments_rule11_condition_member0
  type nonrec unit_types = json_string
  type nonrec choice = json_string
  type nonrec flag = json_string
  type nonrec unit_type = json_string
  type nonrec conditions = fragments_rule11_condition
end

module V15 = struct
  type nonrec t = save15
  type nonrec version = literal_15
  type nonrec checkpoint = save8_checkpoint
  type nonrec pending_enemy = json_boolean
  type nonrec setup = save2_setup_member1
  type nonrec progress = save2_progress_member1
  type nonrec undo = json_unknown
  type nonrec seed = json_number
  type nonrec state = save8_checkpoint_state
  type nonrec first = save35_setup_first
  type nonrec enemy = save2_setup_member1_enemy
  type nonrec entry = history1_entries_item
  type nonrec round = json_number
  type nonrec side = save35_setup_first
  type nonrec winner = fragments_native_state38_winner
  type nonrec armies = fragments_native_state38_armies_member1
  type nonrec condition_memory = json_string
  type nonrec map_id = json_string
  type nonrec map_revision = json_number
  type nonrec cols = json_number
  type nonrec rows = json_number
  type nonrec next_id = json_number
  type nonrec log = json_string
  type nonrec last_combat = fragments_native_state38_last_combat_member2
  type nonrec unit_ = save2_checkpoint_state_units_item
  type nonrec tile = fragments_tiles11_item
  type nonrec campaign = save2_checkpoint_state_campaign_member1
  type nonrec gold = javascript1_state_gold
  type nonrec faction = fragments_factions7_item
  type nonrec ground_items = fragments_native_state38_ground_items_item
  type nonrec combat_rng = json_number
  type nonrec policy = save35_setup_enemy_policy
  type nonrec blocks = save35_setup_enemy_blocks_member1
  type nonrec recruitment_limit = json_number
  type nonrec diplomacy = save2_setup_member1_enemy_diplomacy
  type nonrec id = json_string
  type nonrec saved_at = json_number
  type nonrec run = json_string
  type nonrec mission = json_string
  type nonrec mission_start = json_boolean
  type nonrec run_id = json_string
  type nonrec branch_id = json_string
  type nonrec checkpoint_id = json_string
  type nonrec ordinal = json_number
  type nonrec kind = history1_entries_item_kind
  type nonrec parent_id = json_string
  type nonrec attacker_id = json_string
  type nonrec attack_id = json_string
  type nonrec sound = json_string
  type nonrec role = fragments_native_state38_units_item_role
  type nonrec items = fragments_native_state38_ground_items_item_item
  type nonrec ai_group = json_string
  type nonrec type_ = json_string
  type nonrec hp = json_number
  type nonrec moved = json_boolean
  type nonrec attacked = json_boolean
  type nonrec retreating = fragments_rule11_remember
  type nonrec rebuilding = fragments_rule11_remember
  type nonrec name = json_string
  type nonrec character = json_string
  type nonrec experience = json_number
  type nonrec owner = json_string
  type nonrec location = json_string
  type nonrec col = json_number
  type nonrec row = json_number
  type nonrec terrain = json_string
  type nonrec connection = fragments_tiles11_item_connections_member1_item
  type nonrec dead_ends = fragments_tiles11_item_connections_member1_item
  type nonrec walls = fragments_tiles11_item_connections_member1_item
  type nonrec floor = json_string
  type nonrec fog = fragments_rule11
  type nonrec light = fragments_light23
  type nonrec wall_corners = json_number
  type nonrec vigil = fragments_native_state38_campaign_member1_vigil_member1
  type nonrec refuge = fragments_native_state38_campaign_member1_refuge_member1
  type nonrec completed = json_boolean
  type nonrec mark = json_string
  type nonrec hold = fragments_native_state38_campaign_member1_holds_member1
  type nonrec recall = json_string
  type nonrec fallen_companions = json_string
  type nonrec carry_gold = json_number
  type nonrec dialogue = fragments_native_state38_campaign_member1_dialogue_item
  type nonrec dialogue_index = json_number
  type nonrec outcome = json_string
  type nonrec veteran = save2_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec story_veteran = save2_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec evacuated = save2_checkpoint_state_campaign_member1_evacuated_member1_item
  type nonrec choices = json_string
  type nonrec player = json_number
  type nonrec phase = save35_setup_first
  type nonrec controller = fragments_factions7_item_controller
  type nonrec palette = json_string
  type nonrec recruits = fragments_factions7_item_recruits_member1_item
  type nonrec relations = fragments_factions7_item_relations_member1
  type nonrec transfer = fragments_factions7_item_transfer_member1
  type nonrec item = fragments_native_state38_ground_items_item_item
  type nonrec recruiter = save35_setup_enemy_blocks_member1_recruiter
  type nonrec planner = save35_setup_enemy_blocks_member1_planner
  type nonrec armor = fragments_native_state38_ground_items_item_item_armor_member1
  type nonrec description = json_string
  type nonrec attacks = fragments_native_state38_ground_items_item_item_attacks_item
  type nonrec healing = fragments_native_state38_ground_items_item_item_healing_member1
  type nonrec condition = fragments_rule11_condition
  type nonrec remember = fragments_rule11_remember
  type nonrec radius = json_number
  type nonrec color = fragments_light23_color_member1
  type nonrec fixture = fragments_rule11_remember
  type nonrec dx = json_number
  type nonrec dy = json_number
  type nonrec power = json_number
  type nonrec rule = fragments_rule11
  type nonrec held_turns = json_number
  type nonrec last_counted_round = json_number
  type nonrec escalated = json_boolean
  type nonrec speaker = json_string
  type nonrec portrait = json_string
  type nonrec text = json_string
  type nonrec event = json_string
  type nonrec controller_case = fragments_factions7_item_controller_member0
  type nonrec cost = json_number
  type nonrec to_ = json_string
  type nonrec blade = json_number
  type nonrec pierce = json_number
  type nonrec impact = json_number
  type nonrec damage = json_number
  type nonrec spread = json_number
  type nonrec count = json_number
  type nonrec damage_type = fragments_native_state38_ground_items_item_item_attacks_item_damage_type
  type nonrec category = fragments_native_state38_ground_items_item_item_attacks_item_category
  type nonrec range = json_number
  type nonrec drain = json_number
  type nonrec amount = json_number
  type nonrec condition_case = fragments_rule11_condition_member0
  type nonrec unit_types = json_string
  type nonrec choice = json_string
  type nonrec flag = json_string
  type nonrec unit_type = json_string
  type nonrec conditions = fragments_rule11_condition
end

module V16 = struct
  type nonrec t = save16
  type nonrec version = literal_16
  type nonrec checkpoint = save8_checkpoint
  type nonrec pending_enemy = json_boolean
  type nonrec setup = save2_setup_member1
  type nonrec progress = save2_progress_member1
  type nonrec undo = json_unknown
  type nonrec seed = json_number
  type nonrec state = save8_checkpoint_state
  type nonrec first = save35_setup_first
  type nonrec enemy = save2_setup_member1_enemy
  type nonrec entry = history1_entries_item
  type nonrec round = json_number
  type nonrec side = save35_setup_first
  type nonrec winner = fragments_native_state38_winner
  type nonrec armies = fragments_native_state38_armies_member1
  type nonrec condition_memory = json_string
  type nonrec map_id = json_string
  type nonrec map_revision = json_number
  type nonrec cols = json_number
  type nonrec rows = json_number
  type nonrec next_id = json_number
  type nonrec log = json_string
  type nonrec last_combat = fragments_native_state38_last_combat_member2
  type nonrec unit_ = save2_checkpoint_state_units_item
  type nonrec tile = fragments_tiles11_item
  type nonrec campaign = save2_checkpoint_state_campaign_member1
  type nonrec gold = javascript1_state_gold
  type nonrec faction = fragments_factions7_item
  type nonrec ground_items = fragments_native_state38_ground_items_item
  type nonrec combat_rng = json_number
  type nonrec policy = save35_setup_enemy_policy
  type nonrec blocks = save35_setup_enemy_blocks_member1
  type nonrec recruitment_limit = json_number
  type nonrec diplomacy = save2_setup_member1_enemy_diplomacy
  type nonrec id = json_string
  type nonrec saved_at = json_number
  type nonrec run = json_string
  type nonrec mission = json_string
  type nonrec mission_start = json_boolean
  type nonrec run_id = json_string
  type nonrec branch_id = json_string
  type nonrec checkpoint_id = json_string
  type nonrec ordinal = json_number
  type nonrec kind = history1_entries_item_kind
  type nonrec parent_id = json_string
  type nonrec attacker_id = json_string
  type nonrec attack_id = json_string
  type nonrec sound = json_string
  type nonrec role = fragments_native_state38_units_item_role
  type nonrec items = fragments_native_state38_ground_items_item_item
  type nonrec ai_group = json_string
  type nonrec type_ = json_string
  type nonrec hp = json_number
  type nonrec moved = json_boolean
  type nonrec attacked = json_boolean
  type nonrec retreating = fragments_rule11_remember
  type nonrec rebuilding = fragments_rule11_remember
  type nonrec name = json_string
  type nonrec character = json_string
  type nonrec experience = json_number
  type nonrec owner = json_string
  type nonrec location = json_string
  type nonrec col = json_number
  type nonrec row = json_number
  type nonrec terrain = json_string
  type nonrec connection = fragments_tiles11_item_connections_member1_item
  type nonrec dead_ends = fragments_tiles11_item_connections_member1_item
  type nonrec walls = fragments_tiles11_item_connections_member1_item
  type nonrec floor = json_string
  type nonrec fog = fragments_rule11
  type nonrec light = fragments_light23
  type nonrec wall_corners = json_number
  type nonrec vigil = fragments_native_state38_campaign_member1_vigil_member1
  type nonrec refuge = fragments_native_state38_campaign_member1_refuge_member1
  type nonrec completed = json_boolean
  type nonrec mark = json_string
  type nonrec hold = fragments_native_state38_campaign_member1_holds_member1
  type nonrec recall = json_string
  type nonrec fallen_companions = json_string
  type nonrec carry_gold = json_number
  type nonrec dialogue = fragments_native_state38_campaign_member1_dialogue_item
  type nonrec dialogue_index = json_number
  type nonrec outcome = json_string
  type nonrec veteran = save2_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec story_veteran = save2_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec evacuated = save2_checkpoint_state_campaign_member1_evacuated_member1_item
  type nonrec choices = json_string
  type nonrec player = json_number
  type nonrec phase = save35_setup_first
  type nonrec controller = fragments_factions7_item_controller
  type nonrec palette = json_string
  type nonrec recruits = fragments_factions7_item_recruits_member1_item
  type nonrec relations = fragments_factions7_item_relations_member1
  type nonrec transfer = fragments_factions7_item_transfer_member1
  type nonrec item = fragments_native_state38_ground_items_item_item
  type nonrec recruiter = save35_setup_enemy_blocks_member1_recruiter
  type nonrec planner = save35_setup_enemy_blocks_member1_planner
  type nonrec armor = fragments_native_state38_ground_items_item_item_armor_member1
  type nonrec description = json_string
  type nonrec attacks = fragments_native_state38_ground_items_item_item_attacks_item
  type nonrec healing = fragments_native_state38_ground_items_item_item_healing_member1
  type nonrec condition = fragments_rule11_condition
  type nonrec remember = fragments_rule11_remember
  type nonrec radius = json_number
  type nonrec color = fragments_light23_color_member1
  type nonrec fixture = fragments_rule11_remember
  type nonrec dx = json_number
  type nonrec dy = json_number
  type nonrec power = json_number
  type nonrec rule = fragments_rule11
  type nonrec held_turns = json_number
  type nonrec last_counted_round = json_number
  type nonrec escalated = json_boolean
  type nonrec speaker = json_string
  type nonrec portrait = json_string
  type nonrec text = json_string
  type nonrec event = json_string
  type nonrec controller_case = fragments_factions7_item_controller_member0
  type nonrec cost = json_number
  type nonrec to_ = json_string
  type nonrec blade = json_number
  type nonrec pierce = json_number
  type nonrec impact = json_number
  type nonrec damage = json_number
  type nonrec spread = json_number
  type nonrec count = json_number
  type nonrec damage_type = fragments_native_state38_ground_items_item_item_attacks_item_damage_type
  type nonrec category = fragments_native_state38_ground_items_item_item_attacks_item_category
  type nonrec range = json_number
  type nonrec drain = json_number
  type nonrec amount = json_number
  type nonrec condition_case = fragments_rule11_condition_member0
  type nonrec unit_types = json_string
  type nonrec choice = json_string
  type nonrec flag = json_string
  type nonrec unit_type = json_string
  type nonrec conditions = fragments_rule11_condition
end

module V17 = struct
  type nonrec t = save17
  type nonrec version = literal_17
  type nonrec checkpoint = save17_checkpoint
  type nonrec pending_enemy = json_boolean
  type nonrec setup = save2_setup_member1
  type nonrec progress = save2_progress_member1
  type nonrec undo = json_unknown
  type nonrec seed = json_number
  type nonrec state = save17_checkpoint_state
  type nonrec first = save35_setup_first
  type nonrec enemy = save2_setup_member1_enemy
  type nonrec entry = history1_entries_item
  type nonrec round = json_number
  type nonrec side = save35_setup_first
  type nonrec winner = fragments_native_state38_winner
  type nonrec armies = fragments_native_state38_armies_member1
  type nonrec condition_memory = json_string
  type nonrec map_id = json_string
  type nonrec map_revision = json_number
  type nonrec cols = json_number
  type nonrec rows = json_number
  type nonrec next_id = json_number
  type nonrec log = json_string
  type nonrec last_combat = fragments_native_state38_last_combat_member2
  type nonrec unit_ = save2_checkpoint_state_units_item
  type nonrec tile = save17_checkpoint_state_tiles_item
  type nonrec campaign = save2_checkpoint_state_campaign_member1
  type nonrec gold = javascript1_state_gold
  type nonrec faction = fragments_factions7_item
  type nonrec ground_items = fragments_native_state38_ground_items_item
  type nonrec combat_rng = json_number
  type nonrec policy = save35_setup_enemy_policy
  type nonrec blocks = save35_setup_enemy_blocks_member1
  type nonrec recruitment_limit = json_number
  type nonrec diplomacy = save2_setup_member1_enemy_diplomacy
  type nonrec id = json_string
  type nonrec saved_at = json_number
  type nonrec run = json_string
  type nonrec mission = json_string
  type nonrec mission_start = json_boolean
  type nonrec run_id = json_string
  type nonrec branch_id = json_string
  type nonrec checkpoint_id = json_string
  type nonrec ordinal = json_number
  type nonrec kind = history1_entries_item_kind
  type nonrec parent_id = json_string
  type nonrec attacker_id = json_string
  type nonrec attack_id = json_string
  type nonrec sound = json_string
  type nonrec role = fragments_native_state38_units_item_role
  type nonrec items = fragments_native_state38_ground_items_item_item
  type nonrec ai_group = json_string
  type nonrec type_ = json_string
  type nonrec hp = json_number
  type nonrec moved = json_boolean
  type nonrec attacked = json_boolean
  type nonrec retreating = fragments_rule11_remember
  type nonrec rebuilding = fragments_rule11_remember
  type nonrec name = json_string
  type nonrec character = json_string
  type nonrec experience = json_number
  type nonrec owner = json_string
  type nonrec location = json_string
  type nonrec col = json_number
  type nonrec row = json_number
  type nonrec terrain = json_string
  type nonrec connection = fragments_tiles11_item_connections_member1_item
  type nonrec dead_ends = fragments_tiles11_item_connections_member1_item
  type nonrec walls = fragments_tiles11_item_connections_member1_item
  type nonrec floor = json_string
  type nonrec fog = fragments_rule11
  type nonrec light = fragments_light23
  type nonrec gates = json_string
  type nonrec vigil = fragments_native_state38_campaign_member1_vigil_member1
  type nonrec refuge = fragments_native_state38_campaign_member1_refuge_member1
  type nonrec completed = json_boolean
  type nonrec mark = json_string
  type nonrec hold = fragments_native_state38_campaign_member1_holds_member1
  type nonrec recall = json_string
  type nonrec fallen_companions = json_string
  type nonrec carry_gold = json_number
  type nonrec dialogue = fragments_native_state38_campaign_member1_dialogue_item
  type nonrec dialogue_index = json_number
  type nonrec outcome = json_string
  type nonrec veteran = save2_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec story_veteran = save2_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec evacuated = save2_checkpoint_state_campaign_member1_evacuated_member1_item
  type nonrec choices = json_string
  type nonrec player = json_number
  type nonrec phase = save35_setup_first
  type nonrec controller = fragments_factions7_item_controller
  type nonrec palette = json_string
  type nonrec recruits = fragments_factions7_item_recruits_member1_item
  type nonrec relations = fragments_factions7_item_relations_member1
  type nonrec transfer = fragments_factions7_item_transfer_member1
  type nonrec item = fragments_native_state38_ground_items_item_item
  type nonrec recruiter = save35_setup_enemy_blocks_member1_recruiter
  type nonrec planner = save35_setup_enemy_blocks_member1_planner
  type nonrec armor = fragments_native_state38_ground_items_item_item_armor_member1
  type nonrec description = json_string
  type nonrec attacks = fragments_native_state38_ground_items_item_item_attacks_item
  type nonrec healing = fragments_native_state38_ground_items_item_item_healing_member1
  type nonrec condition = fragments_rule11_condition
  type nonrec remember = fragments_rule11_remember
  type nonrec radius = json_number
  type nonrec color = fragments_light23_color_member1
  type nonrec fixture = fragments_rule11_remember
  type nonrec dx = json_number
  type nonrec dy = json_number
  type nonrec power = json_number
  type nonrec rule = fragments_rule11
  type nonrec held_turns = json_number
  type nonrec last_counted_round = json_number
  type nonrec escalated = json_boolean
  type nonrec speaker = json_string
  type nonrec portrait = json_string
  type nonrec text = json_string
  type nonrec event = json_string
  type nonrec controller_case = fragments_factions7_item_controller_member0
  type nonrec cost = json_number
  type nonrec to_ = json_string
  type nonrec blade = json_number
  type nonrec pierce = json_number
  type nonrec impact = json_number
  type nonrec damage = json_number
  type nonrec spread = json_number
  type nonrec count = json_number
  type nonrec damage_type = fragments_native_state38_ground_items_item_item_attacks_item_damage_type
  type nonrec category = fragments_native_state38_ground_items_item_item_attacks_item_category
  type nonrec range = json_number
  type nonrec drain = json_number
  type nonrec amount = json_number
  type nonrec condition_case = fragments_rule11_condition_member0
  type nonrec unit_types = json_string
  type nonrec choice = json_string
  type nonrec flag = json_string
  type nonrec unit_type = json_string
  type nonrec conditions = fragments_rule11_condition
end

module V18 = struct
  type nonrec t = save18
  type nonrec version = literal_18
  type nonrec checkpoint = save18_checkpoint
  type nonrec pending_enemy = json_boolean
  type nonrec setup = save2_setup_member1
  type nonrec progress = save2_progress_member1
  type nonrec undo = json_unknown
  type nonrec seed = json_number
  type nonrec state = save18_checkpoint_state
  type nonrec first = save35_setup_first
  type nonrec enemy = save2_setup_member1_enemy
  type nonrec entry = history1_entries_item
  type nonrec round = json_number
  type nonrec side = save35_setup_first
  type nonrec winner = fragments_native_state38_winner
  type nonrec armies = fragments_native_state38_armies_member1
  type nonrec condition_memory = json_string
  type nonrec map_id = json_string
  type nonrec map_revision = json_number
  type nonrec cols = json_number
  type nonrec rows = json_number
  type nonrec next_id = json_number
  type nonrec log = json_string
  type nonrec last_combat = fragments_native_state38_last_combat_member2
  type nonrec unit_ = save2_checkpoint_state_units_item
  type nonrec tile = save18_checkpoint_state_tiles_item
  type nonrec campaign = save2_checkpoint_state_campaign_member1
  type nonrec gold = javascript1_state_gold
  type nonrec faction = fragments_factions7_item
  type nonrec ground_items = fragments_native_state38_ground_items_item
  type nonrec combat_rng = json_number
  type nonrec policy = save35_setup_enemy_policy
  type nonrec blocks = save35_setup_enemy_blocks_member1
  type nonrec recruitment_limit = json_number
  type nonrec diplomacy = save2_setup_member1_enemy_diplomacy
  type nonrec id = json_string
  type nonrec saved_at = json_number
  type nonrec run = json_string
  type nonrec mission = json_string
  type nonrec mission_start = json_boolean
  type nonrec run_id = json_string
  type nonrec branch_id = json_string
  type nonrec checkpoint_id = json_string
  type nonrec ordinal = json_number
  type nonrec kind = history1_entries_item_kind
  type nonrec parent_id = json_string
  type nonrec attacker_id = json_string
  type nonrec attack_id = json_string
  type nonrec sound = json_string
  type nonrec role = fragments_native_state38_units_item_role
  type nonrec items = fragments_native_state38_ground_items_item_item
  type nonrec ai_group = json_string
  type nonrec type_ = json_string
  type nonrec hp = json_number
  type nonrec moved = json_boolean
  type nonrec attacked = json_boolean
  type nonrec retreating = fragments_rule11_remember
  type nonrec rebuilding = fragments_rule11_remember
  type nonrec name = json_string
  type nonrec character = json_string
  type nonrec experience = json_number
  type nonrec owner = json_string
  type nonrec location = json_string
  type nonrec col = json_number
  type nonrec row = json_number
  type nonrec terrain = json_string
  type nonrec connection = fragments_tiles11_item_connections_member1_item
  type nonrec dead_ends = fragments_tiles11_item_connections_member1_item
  type nonrec walls = fragments_tiles11_item_connections_member1_item
  type nonrec floor = json_string
  type nonrec fog = fragments_rule11
  type nonrec light = fragments_light23
  type nonrec vigil = fragments_native_state38_campaign_member1_vigil_member1
  type nonrec refuge = fragments_native_state38_campaign_member1_refuge_member1
  type nonrec completed = json_boolean
  type nonrec mark = json_string
  type nonrec hold = fragments_native_state38_campaign_member1_holds_member1
  type nonrec recall = json_string
  type nonrec fallen_companions = json_string
  type nonrec carry_gold = json_number
  type nonrec dialogue = fragments_native_state38_campaign_member1_dialogue_item
  type nonrec dialogue_index = json_number
  type nonrec outcome = json_string
  type nonrec veteran = save2_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec story_veteran = save2_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec evacuated = save2_checkpoint_state_campaign_member1_evacuated_member1_item
  type nonrec choices = json_string
  type nonrec player = json_number
  type nonrec phase = save35_setup_first
  type nonrec controller = fragments_factions7_item_controller
  type nonrec palette = json_string
  type nonrec recruits = fragments_factions7_item_recruits_member1_item
  type nonrec relations = fragments_factions7_item_relations_member1
  type nonrec transfer = fragments_factions7_item_transfer_member1
  type nonrec item = fragments_native_state38_ground_items_item_item
  type nonrec recruiter = save35_setup_enemy_blocks_member1_recruiter
  type nonrec planner = save35_setup_enemy_blocks_member1_planner
  type nonrec armor = fragments_native_state38_ground_items_item_item_armor_member1
  type nonrec description = json_string
  type nonrec attacks = fragments_native_state38_ground_items_item_item_attacks_item
  type nonrec healing = fragments_native_state38_ground_items_item_item_healing_member1
  type nonrec condition = fragments_rule11_condition
  type nonrec remember = fragments_rule11_remember
  type nonrec radius = json_number
  type nonrec color = fragments_light23_color_member1
  type nonrec fixture = fragments_rule11_remember
  type nonrec dx = json_number
  type nonrec dy = json_number
  type nonrec power = json_number
  type nonrec rule = fragments_rule11
  type nonrec held_turns = json_number
  type nonrec last_counted_round = json_number
  type nonrec escalated = json_boolean
  type nonrec speaker = json_string
  type nonrec portrait = json_string
  type nonrec text = json_string
  type nonrec event = json_string
  type nonrec controller_case = fragments_factions7_item_controller_member0
  type nonrec cost = json_number
  type nonrec to_ = json_string
  type nonrec blade = json_number
  type nonrec pierce = json_number
  type nonrec impact = json_number
  type nonrec damage = json_number
  type nonrec spread = json_number
  type nonrec count = json_number
  type nonrec damage_type = fragments_native_state38_ground_items_item_item_attacks_item_damage_type
  type nonrec category = fragments_native_state38_ground_items_item_item_attacks_item_category
  type nonrec range = json_number
  type nonrec drain = json_number
  type nonrec amount = json_number
  type nonrec condition_case = fragments_rule11_condition_member0
  type nonrec unit_types = json_string
  type nonrec choice = json_string
  type nonrec flag = json_string
  type nonrec unit_type = json_string
  type nonrec conditions = fragments_rule11_condition
end

module V19 = struct
  type nonrec t = save19
  type nonrec version = literal_19
  type nonrec checkpoint = save18_checkpoint
  type nonrec pending_enemy = json_boolean
  type nonrec setup = save2_setup_member1
  type nonrec progress = save2_progress_member1
  type nonrec undo = json_unknown
  type nonrec seed = json_number
  type nonrec state = save18_checkpoint_state
  type nonrec first = save35_setup_first
  type nonrec enemy = save2_setup_member1_enemy
  type nonrec entry = history1_entries_item
  type nonrec round = json_number
  type nonrec side = save35_setup_first
  type nonrec winner = fragments_native_state38_winner
  type nonrec armies = fragments_native_state38_armies_member1
  type nonrec condition_memory = json_string
  type nonrec map_id = json_string
  type nonrec map_revision = json_number
  type nonrec cols = json_number
  type nonrec rows = json_number
  type nonrec next_id = json_number
  type nonrec log = json_string
  type nonrec last_combat = fragments_native_state38_last_combat_member2
  type nonrec unit_ = save2_checkpoint_state_units_item
  type nonrec tile = save18_checkpoint_state_tiles_item
  type nonrec campaign = save2_checkpoint_state_campaign_member1
  type nonrec gold = javascript1_state_gold
  type nonrec faction = fragments_factions7_item
  type nonrec ground_items = fragments_native_state38_ground_items_item
  type nonrec combat_rng = json_number
  type nonrec policy = save35_setup_enemy_policy
  type nonrec blocks = save35_setup_enemy_blocks_member1
  type nonrec recruitment_limit = json_number
  type nonrec diplomacy = save2_setup_member1_enemy_diplomacy
  type nonrec id = json_string
  type nonrec saved_at = json_number
  type nonrec run = json_string
  type nonrec mission = json_string
  type nonrec mission_start = json_boolean
  type nonrec run_id = json_string
  type nonrec branch_id = json_string
  type nonrec checkpoint_id = json_string
  type nonrec ordinal = json_number
  type nonrec kind = history1_entries_item_kind
  type nonrec parent_id = json_string
  type nonrec attacker_id = json_string
  type nonrec attack_id = json_string
  type nonrec sound = json_string
  type nonrec role = fragments_native_state38_units_item_role
  type nonrec items = fragments_native_state38_ground_items_item_item
  type nonrec ai_group = json_string
  type nonrec type_ = json_string
  type nonrec hp = json_number
  type nonrec moved = json_boolean
  type nonrec attacked = json_boolean
  type nonrec retreating = fragments_rule11_remember
  type nonrec rebuilding = fragments_rule11_remember
  type nonrec name = json_string
  type nonrec character = json_string
  type nonrec experience = json_number
  type nonrec owner = json_string
  type nonrec location = json_string
  type nonrec col = json_number
  type nonrec row = json_number
  type nonrec terrain = json_string
  type nonrec connection = fragments_tiles11_item_connections_member1_item
  type nonrec dead_ends = fragments_tiles11_item_connections_member1_item
  type nonrec walls = fragments_tiles11_item_connections_member1_item
  type nonrec floor = json_string
  type nonrec fog = fragments_rule11
  type nonrec light = fragments_light23
  type nonrec vigil = fragments_native_state38_campaign_member1_vigil_member1
  type nonrec refuge = fragments_native_state38_campaign_member1_refuge_member1
  type nonrec completed = json_boolean
  type nonrec mark = json_string
  type nonrec hold = fragments_native_state38_campaign_member1_holds_member1
  type nonrec recall = json_string
  type nonrec fallen_companions = json_string
  type nonrec carry_gold = json_number
  type nonrec dialogue = fragments_native_state38_campaign_member1_dialogue_item
  type nonrec dialogue_index = json_number
  type nonrec outcome = json_string
  type nonrec veteran = save2_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec story_veteran = save2_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec evacuated = save2_checkpoint_state_campaign_member1_evacuated_member1_item
  type nonrec choices = json_string
  type nonrec player = json_number
  type nonrec phase = save35_setup_first
  type nonrec controller = fragments_factions7_item_controller
  type nonrec palette = json_string
  type nonrec recruits = fragments_factions7_item_recruits_member1_item
  type nonrec relations = fragments_factions7_item_relations_member1
  type nonrec transfer = fragments_factions7_item_transfer_member1
  type nonrec item = fragments_native_state38_ground_items_item_item
  type nonrec recruiter = save35_setup_enemy_blocks_member1_recruiter
  type nonrec planner = save35_setup_enemy_blocks_member1_planner
  type nonrec armor = fragments_native_state38_ground_items_item_item_armor_member1
  type nonrec description = json_string
  type nonrec attacks = fragments_native_state38_ground_items_item_item_attacks_item
  type nonrec healing = fragments_native_state38_ground_items_item_item_healing_member1
  type nonrec condition = fragments_rule11_condition
  type nonrec remember = fragments_rule11_remember
  type nonrec radius = json_number
  type nonrec color = fragments_light23_color_member1
  type nonrec fixture = fragments_rule11_remember
  type nonrec dx = json_number
  type nonrec dy = json_number
  type nonrec power = json_number
  type nonrec rule = fragments_rule11
  type nonrec held_turns = json_number
  type nonrec last_counted_round = json_number
  type nonrec escalated = json_boolean
  type nonrec speaker = json_string
  type nonrec portrait = json_string
  type nonrec text = json_string
  type nonrec event = json_string
  type nonrec controller_case = fragments_factions7_item_controller_member0
  type nonrec cost = json_number
  type nonrec to_ = json_string
  type nonrec blade = json_number
  type nonrec pierce = json_number
  type nonrec impact = json_number
  type nonrec damage = json_number
  type nonrec spread = json_number
  type nonrec count = json_number
  type nonrec damage_type = fragments_native_state38_ground_items_item_item_attacks_item_damage_type
  type nonrec category = fragments_native_state38_ground_items_item_item_attacks_item_category
  type nonrec range = json_number
  type nonrec drain = json_number
  type nonrec amount = json_number
  type nonrec condition_case = fragments_rule11_condition_member0
  type nonrec unit_types = json_string
  type nonrec choice = json_string
  type nonrec flag = json_string
  type nonrec unit_type = json_string
  type nonrec conditions = fragments_rule11_condition
end

module V20 = struct
  type nonrec t = save20
  type nonrec version = literal_20
  type nonrec checkpoint = save18_checkpoint
  type nonrec pending_enemy = json_boolean
  type nonrec setup = save2_setup_member1
  type nonrec progress = save2_progress_member1
  type nonrec undo = json_unknown
  type nonrec seed = json_number
  type nonrec state = save18_checkpoint_state
  type nonrec first = save35_setup_first
  type nonrec enemy = save2_setup_member1_enemy
  type nonrec entry = history1_entries_item
  type nonrec round = json_number
  type nonrec side = save35_setup_first
  type nonrec winner = fragments_native_state38_winner
  type nonrec armies = fragments_native_state38_armies_member1
  type nonrec condition_memory = json_string
  type nonrec map_id = json_string
  type nonrec map_revision = json_number
  type nonrec cols = json_number
  type nonrec rows = json_number
  type nonrec next_id = json_number
  type nonrec log = json_string
  type nonrec last_combat = fragments_native_state38_last_combat_member2
  type nonrec unit_ = save2_checkpoint_state_units_item
  type nonrec tile = save18_checkpoint_state_tiles_item
  type nonrec campaign = save2_checkpoint_state_campaign_member1
  type nonrec gold = javascript1_state_gold
  type nonrec faction = fragments_factions7_item
  type nonrec ground_items = fragments_native_state38_ground_items_item
  type nonrec combat_rng = json_number
  type nonrec policy = save35_setup_enemy_policy
  type nonrec blocks = save35_setup_enemy_blocks_member1
  type nonrec recruitment_limit = json_number
  type nonrec diplomacy = save2_setup_member1_enemy_diplomacy
  type nonrec id = json_string
  type nonrec saved_at = json_number
  type nonrec run = json_string
  type nonrec mission = json_string
  type nonrec mission_start = json_boolean
  type nonrec run_id = json_string
  type nonrec branch_id = json_string
  type nonrec checkpoint_id = json_string
  type nonrec ordinal = json_number
  type nonrec kind = history1_entries_item_kind
  type nonrec parent_id = json_string
  type nonrec attacker_id = json_string
  type nonrec attack_id = json_string
  type nonrec sound = json_string
  type nonrec role = fragments_native_state38_units_item_role
  type nonrec items = fragments_native_state38_ground_items_item_item
  type nonrec ai_group = json_string
  type nonrec type_ = json_string
  type nonrec hp = json_number
  type nonrec moved = json_boolean
  type nonrec attacked = json_boolean
  type nonrec retreating = fragments_rule11_remember
  type nonrec rebuilding = fragments_rule11_remember
  type nonrec name = json_string
  type nonrec character = json_string
  type nonrec experience = json_number
  type nonrec owner = json_string
  type nonrec location = json_string
  type nonrec col = json_number
  type nonrec row = json_number
  type nonrec terrain = json_string
  type nonrec connection = fragments_tiles11_item_connections_member1_item
  type nonrec dead_ends = fragments_tiles11_item_connections_member1_item
  type nonrec walls = fragments_tiles11_item_connections_member1_item
  type nonrec floor = json_string
  type nonrec fog = fragments_rule11
  type nonrec light = fragments_light23
  type nonrec vigil = fragments_native_state38_campaign_member1_vigil_member1
  type nonrec refuge = fragments_native_state38_campaign_member1_refuge_member1
  type nonrec completed = json_boolean
  type nonrec mark = json_string
  type nonrec hold = fragments_native_state38_campaign_member1_holds_member1
  type nonrec recall = json_string
  type nonrec fallen_companions = json_string
  type nonrec carry_gold = json_number
  type nonrec dialogue = fragments_native_state38_campaign_member1_dialogue_item
  type nonrec dialogue_index = json_number
  type nonrec outcome = json_string
  type nonrec veteran = save2_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec story_veteran = save2_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec evacuated = save2_checkpoint_state_campaign_member1_evacuated_member1_item
  type nonrec choices = json_string
  type nonrec player = json_number
  type nonrec phase = save35_setup_first
  type nonrec controller = fragments_factions7_item_controller
  type nonrec palette = json_string
  type nonrec recruits = fragments_factions7_item_recruits_member1_item
  type nonrec relations = fragments_factions7_item_relations_member1
  type nonrec transfer = fragments_factions7_item_transfer_member1
  type nonrec item = fragments_native_state38_ground_items_item_item
  type nonrec recruiter = save35_setup_enemy_blocks_member1_recruiter
  type nonrec planner = save35_setup_enemy_blocks_member1_planner
  type nonrec armor = fragments_native_state38_ground_items_item_item_armor_member1
  type nonrec description = json_string
  type nonrec attacks = fragments_native_state38_ground_items_item_item_attacks_item
  type nonrec healing = fragments_native_state38_ground_items_item_item_healing_member1
  type nonrec condition = fragments_rule11_condition
  type nonrec remember = fragments_rule11_remember
  type nonrec radius = json_number
  type nonrec color = fragments_light23_color_member1
  type nonrec fixture = fragments_rule11_remember
  type nonrec dx = json_number
  type nonrec dy = json_number
  type nonrec power = json_number
  type nonrec rule = fragments_rule11
  type nonrec held_turns = json_number
  type nonrec last_counted_round = json_number
  type nonrec escalated = json_boolean
  type nonrec speaker = json_string
  type nonrec portrait = json_string
  type nonrec text = json_string
  type nonrec event = json_string
  type nonrec controller_case = fragments_factions7_item_controller_member0
  type nonrec cost = json_number
  type nonrec to_ = json_string
  type nonrec blade = json_number
  type nonrec pierce = json_number
  type nonrec impact = json_number
  type nonrec damage = json_number
  type nonrec spread = json_number
  type nonrec count = json_number
  type nonrec damage_type = fragments_native_state38_ground_items_item_item_attacks_item_damage_type
  type nonrec category = fragments_native_state38_ground_items_item_item_attacks_item_category
  type nonrec range = json_number
  type nonrec drain = json_number
  type nonrec amount = json_number
  type nonrec condition_case = fragments_rule11_condition_member0
  type nonrec unit_types = json_string
  type nonrec choice = json_string
  type nonrec flag = json_string
  type nonrec unit_type = json_string
  type nonrec conditions = fragments_rule11_condition
end

module V21 = struct
  type nonrec t = save21
  type nonrec version = literal_21
  type nonrec checkpoint = save21_checkpoint
  type nonrec pending_enemy = json_boolean
  type nonrec setup = save2_setup_member1
  type nonrec progress = save2_progress_member1
  type nonrec undo = json_unknown
  type nonrec seed = json_number
  type nonrec state = save21_checkpoint_state
  type nonrec first = save35_setup_first
  type nonrec enemy = save2_setup_member1_enemy
  type nonrec entry = history1_entries_item
  type nonrec round = json_number
  type nonrec side = save35_setup_first
  type nonrec winner = fragments_native_state38_winner
  type nonrec armies = fragments_native_state38_armies_member1
  type nonrec condition_memory = json_string
  type nonrec map_id = json_string
  type nonrec map_revision = json_number
  type nonrec cols = json_number
  type nonrec rows = json_number
  type nonrec next_id = json_number
  type nonrec log = json_string
  type nonrec last_combat = fragments_native_state38_last_combat_member2
  type nonrec unit_ = save2_checkpoint_state_units_item
  type nonrec tile = save18_checkpoint_state_tiles_item
  type nonrec campaign = save2_checkpoint_state_campaign_member1
  type nonrec gold = fragments_native_state38_gold
  type nonrec faction = fragments_factions7_item
  type nonrec ground_items = fragments_native_state38_ground_items_item
  type nonrec combat_rng = json_number
  type nonrec policy = save35_setup_enemy_policy
  type nonrec blocks = save35_setup_enemy_blocks_member1
  type nonrec recruitment_limit = json_number
  type nonrec diplomacy = save2_setup_member1_enemy_diplomacy
  type nonrec id = json_string
  type nonrec saved_at = json_number
  type nonrec run = json_string
  type nonrec mission = json_string
  type nonrec mission_start = json_boolean
  type nonrec run_id = json_string
  type nonrec branch_id = json_string
  type nonrec checkpoint_id = json_string
  type nonrec ordinal = json_number
  type nonrec kind = history1_entries_item_kind
  type nonrec parent_id = json_string
  type nonrec attacker_id = json_string
  type nonrec attack_id = json_string
  type nonrec sound = json_string
  type nonrec role = fragments_native_state38_units_item_role
  type nonrec items = fragments_native_state38_ground_items_item_item
  type nonrec ai_group = json_string
  type nonrec type_ = json_string
  type nonrec hp = json_number
  type nonrec moved = json_boolean
  type nonrec attacked = json_boolean
  type nonrec retreating = fragments_rule11_remember
  type nonrec rebuilding = fragments_rule11_remember
  type nonrec name = json_string
  type nonrec character = json_string
  type nonrec experience = json_number
  type nonrec owner = json_string
  type nonrec location = json_string
  type nonrec col = json_number
  type nonrec row = json_number
  type nonrec terrain = json_string
  type nonrec connection = fragments_tiles11_item_connections_member1_item
  type nonrec dead_ends = fragments_tiles11_item_connections_member1_item
  type nonrec walls = fragments_tiles11_item_connections_member1_item
  type nonrec floor = json_string
  type nonrec fog = fragments_rule11
  type nonrec light = fragments_light23
  type nonrec vigil = fragments_native_state38_campaign_member1_vigil_member1
  type nonrec refuge = fragments_native_state38_campaign_member1_refuge_member1
  type nonrec completed = json_boolean
  type nonrec mark = json_string
  type nonrec hold = fragments_native_state38_campaign_member1_holds_member1
  type nonrec recall = json_string
  type nonrec fallen_companions = json_string
  type nonrec carry_gold = json_number
  type nonrec dialogue = fragments_native_state38_campaign_member1_dialogue_item
  type nonrec dialogue_index = json_number
  type nonrec outcome = json_string
  type nonrec veteran = save2_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec story_veteran = save2_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec evacuated = save2_checkpoint_state_campaign_member1_evacuated_member1_item
  type nonrec choices = json_string
  type nonrec phase = save35_setup_first
  type nonrec controller = fragments_factions7_item_controller
  type nonrec palette = json_string
  type nonrec recruits = fragments_factions7_item_recruits_member1_item
  type nonrec relations = fragments_factions7_item_relations_member1
  type nonrec transfer = fragments_factions7_item_transfer_member1
  type nonrec item = fragments_native_state38_ground_items_item_item
  type nonrec recruiter = save35_setup_enemy_blocks_member1_recruiter
  type nonrec planner = save35_setup_enemy_blocks_member1_planner
  type nonrec armor = fragments_native_state38_ground_items_item_item_armor_member1
  type nonrec description = json_string
  type nonrec attacks = fragments_native_state38_ground_items_item_item_attacks_item
  type nonrec healing = fragments_native_state38_ground_items_item_item_healing_member1
  type nonrec condition = fragments_rule11_condition
  type nonrec remember = fragments_rule11_remember
  type nonrec radius = json_number
  type nonrec color = fragments_light23_color_member1
  type nonrec fixture = fragments_rule11_remember
  type nonrec dx = json_number
  type nonrec dy = json_number
  type nonrec power = json_number
  type nonrec rule = fragments_rule11
  type nonrec held_turns = json_number
  type nonrec last_counted_round = json_number
  type nonrec escalated = json_boolean
  type nonrec speaker = json_string
  type nonrec portrait = json_string
  type nonrec text = json_string
  type nonrec event = json_string
  type nonrec controller_case = fragments_factions7_item_controller_member0
  type nonrec cost = json_number
  type nonrec to_ = json_string
  type nonrec blade = json_number
  type nonrec pierce = json_number
  type nonrec impact = json_number
  type nonrec damage = json_number
  type nonrec spread = json_number
  type nonrec count = json_number
  type nonrec damage_type = fragments_native_state38_ground_items_item_item_attacks_item_damage_type
  type nonrec category = fragments_native_state38_ground_items_item_item_attacks_item_category
  type nonrec range = json_number
  type nonrec drain = json_number
  type nonrec amount = json_number
  type nonrec condition_case = fragments_rule11_condition_member0
  type nonrec unit_types = json_string
  type nonrec choice = json_string
  type nonrec flag = json_string
  type nonrec unit_type = json_string
  type nonrec conditions = fragments_rule11_condition
end

module V22 = struct
  type nonrec t = save22
  type nonrec version = literal_22
  type nonrec checkpoint = save21_checkpoint
  type nonrec pending_enemy = json_boolean
  type nonrec setup = save2_setup_member1
  type nonrec progress = save2_progress_member1
  type nonrec undo = json_unknown
  type nonrec seed = json_number
  type nonrec state = save21_checkpoint_state
  type nonrec first = save35_setup_first
  type nonrec enemy = save2_setup_member1_enemy
  type nonrec entry = history1_entries_item
  type nonrec round = json_number
  type nonrec side = save35_setup_first
  type nonrec winner = fragments_native_state38_winner
  type nonrec armies = fragments_native_state38_armies_member1
  type nonrec condition_memory = json_string
  type nonrec map_id = json_string
  type nonrec map_revision = json_number
  type nonrec cols = json_number
  type nonrec rows = json_number
  type nonrec next_id = json_number
  type nonrec log = json_string
  type nonrec last_combat = fragments_native_state38_last_combat_member2
  type nonrec unit_ = save2_checkpoint_state_units_item
  type nonrec tile = save18_checkpoint_state_tiles_item
  type nonrec campaign = save2_checkpoint_state_campaign_member1
  type nonrec gold = fragments_native_state38_gold
  type nonrec faction = fragments_factions7_item
  type nonrec ground_items = fragments_native_state38_ground_items_item
  type nonrec combat_rng = json_number
  type nonrec policy = save35_setup_enemy_policy
  type nonrec blocks = save35_setup_enemy_blocks_member1
  type nonrec recruitment_limit = json_number
  type nonrec diplomacy = save2_setup_member1_enemy_diplomacy
  type nonrec id = json_string
  type nonrec saved_at = json_number
  type nonrec run = json_string
  type nonrec mission = json_string
  type nonrec mission_start = json_boolean
  type nonrec run_id = json_string
  type nonrec branch_id = json_string
  type nonrec checkpoint_id = json_string
  type nonrec ordinal = json_number
  type nonrec kind = history1_entries_item_kind
  type nonrec parent_id = json_string
  type nonrec attacker_id = json_string
  type nonrec attack_id = json_string
  type nonrec sound = json_string
  type nonrec role = fragments_native_state38_units_item_role
  type nonrec items = fragments_native_state38_ground_items_item_item
  type nonrec ai_group = json_string
  type nonrec type_ = json_string
  type nonrec hp = json_number
  type nonrec moved = json_boolean
  type nonrec attacked = json_boolean
  type nonrec retreating = fragments_rule11_remember
  type nonrec rebuilding = fragments_rule11_remember
  type nonrec name = json_string
  type nonrec character = json_string
  type nonrec experience = json_number
  type nonrec owner = json_string
  type nonrec location = json_string
  type nonrec col = json_number
  type nonrec row = json_number
  type nonrec terrain = json_string
  type nonrec connection = fragments_tiles11_item_connections_member1_item
  type nonrec dead_ends = fragments_tiles11_item_connections_member1_item
  type nonrec walls = fragments_tiles11_item_connections_member1_item
  type nonrec floor = json_string
  type nonrec fog = fragments_rule11
  type nonrec light = fragments_light23
  type nonrec vigil = fragments_native_state38_campaign_member1_vigil_member1
  type nonrec refuge = fragments_native_state38_campaign_member1_refuge_member1
  type nonrec completed = json_boolean
  type nonrec mark = json_string
  type nonrec hold = fragments_native_state38_campaign_member1_holds_member1
  type nonrec recall = json_string
  type nonrec fallen_companions = json_string
  type nonrec carry_gold = json_number
  type nonrec dialogue = fragments_native_state38_campaign_member1_dialogue_item
  type nonrec dialogue_index = json_number
  type nonrec outcome = json_string
  type nonrec veteran = save2_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec story_veteran = save2_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec evacuated = save2_checkpoint_state_campaign_member1_evacuated_member1_item
  type nonrec choices = json_string
  type nonrec phase = save35_setup_first
  type nonrec controller = fragments_factions7_item_controller
  type nonrec palette = json_string
  type nonrec recruits = fragments_factions7_item_recruits_member1_item
  type nonrec relations = fragments_factions7_item_relations_member1
  type nonrec transfer = fragments_factions7_item_transfer_member1
  type nonrec item = fragments_native_state38_ground_items_item_item
  type nonrec recruiter = save35_setup_enemy_blocks_member1_recruiter
  type nonrec planner = save35_setup_enemy_blocks_member1_planner
  type nonrec armor = fragments_native_state38_ground_items_item_item_armor_member1
  type nonrec description = json_string
  type nonrec attacks = fragments_native_state38_ground_items_item_item_attacks_item
  type nonrec healing = fragments_native_state38_ground_items_item_item_healing_member1
  type nonrec condition = fragments_rule11_condition
  type nonrec remember = fragments_rule11_remember
  type nonrec radius = json_number
  type nonrec color = fragments_light23_color_member1
  type nonrec fixture = fragments_rule11_remember
  type nonrec dx = json_number
  type nonrec dy = json_number
  type nonrec power = json_number
  type nonrec rule = fragments_rule11
  type nonrec held_turns = json_number
  type nonrec last_counted_round = json_number
  type nonrec escalated = json_boolean
  type nonrec speaker = json_string
  type nonrec portrait = json_string
  type nonrec text = json_string
  type nonrec event = json_string
  type nonrec controller_case = fragments_factions7_item_controller_member0
  type nonrec cost = json_number
  type nonrec to_ = json_string
  type nonrec blade = json_number
  type nonrec pierce = json_number
  type nonrec impact = json_number
  type nonrec damage = json_number
  type nonrec spread = json_number
  type nonrec count = json_number
  type nonrec damage_type = fragments_native_state38_ground_items_item_item_attacks_item_damage_type
  type nonrec category = fragments_native_state38_ground_items_item_item_attacks_item_category
  type nonrec range = json_number
  type nonrec drain = json_number
  type nonrec amount = json_number
  type nonrec condition_case = fragments_rule11_condition_member0
  type nonrec unit_types = json_string
  type nonrec choice = json_string
  type nonrec flag = json_string
  type nonrec unit_type = json_string
  type nonrec conditions = fragments_rule11_condition
end

module V23 = struct
  type nonrec t = save23
  type nonrec version = literal_23
  type nonrec checkpoint = save21_checkpoint
  type nonrec pending_enemy = json_boolean
  type nonrec setup = save2_setup_member1
  type nonrec progress = save2_progress_member1
  type nonrec undo = json_unknown
  type nonrec seed = json_number
  type nonrec state = save21_checkpoint_state
  type nonrec first = save35_setup_first
  type nonrec enemy = save2_setup_member1_enemy
  type nonrec entry = history1_entries_item
  type nonrec round = json_number
  type nonrec side = save35_setup_first
  type nonrec winner = fragments_native_state38_winner
  type nonrec armies = fragments_native_state38_armies_member1
  type nonrec condition_memory = json_string
  type nonrec map_id = json_string
  type nonrec map_revision = json_number
  type nonrec cols = json_number
  type nonrec rows = json_number
  type nonrec next_id = json_number
  type nonrec log = json_string
  type nonrec last_combat = fragments_native_state38_last_combat_member2
  type nonrec unit_ = save2_checkpoint_state_units_item
  type nonrec tile = save18_checkpoint_state_tiles_item
  type nonrec campaign = save2_checkpoint_state_campaign_member1
  type nonrec gold = fragments_native_state38_gold
  type nonrec faction = fragments_factions7_item
  type nonrec ground_items = fragments_native_state38_ground_items_item
  type nonrec combat_rng = json_number
  type nonrec policy = save35_setup_enemy_policy
  type nonrec blocks = save35_setup_enemy_blocks_member1
  type nonrec recruitment_limit = json_number
  type nonrec diplomacy = save2_setup_member1_enemy_diplomacy
  type nonrec id = json_string
  type nonrec saved_at = json_number
  type nonrec run = json_string
  type nonrec mission = json_string
  type nonrec mission_start = json_boolean
  type nonrec run_id = json_string
  type nonrec branch_id = json_string
  type nonrec checkpoint_id = json_string
  type nonrec ordinal = json_number
  type nonrec kind = history1_entries_item_kind
  type nonrec parent_id = json_string
  type nonrec attacker_id = json_string
  type nonrec attack_id = json_string
  type nonrec sound = json_string
  type nonrec role = fragments_native_state38_units_item_role
  type nonrec items = fragments_native_state38_ground_items_item_item
  type nonrec ai_group = json_string
  type nonrec type_ = json_string
  type nonrec hp = json_number
  type nonrec moved = json_boolean
  type nonrec attacked = json_boolean
  type nonrec retreating = fragments_rule11_remember
  type nonrec rebuilding = fragments_rule11_remember
  type nonrec name = json_string
  type nonrec character = json_string
  type nonrec experience = json_number
  type nonrec owner = json_string
  type nonrec location = json_string
  type nonrec col = json_number
  type nonrec row = json_number
  type nonrec terrain = json_string
  type nonrec connection = fragments_tiles11_item_connections_member1_item
  type nonrec dead_ends = fragments_tiles11_item_connections_member1_item
  type nonrec walls = fragments_tiles11_item_connections_member1_item
  type nonrec floor = json_string
  type nonrec fog = fragments_rule11
  type nonrec light = fragments_light23
  type nonrec vigil = fragments_native_state38_campaign_member1_vigil_member1
  type nonrec refuge = fragments_native_state38_campaign_member1_refuge_member1
  type nonrec completed = json_boolean
  type nonrec mark = json_string
  type nonrec hold = fragments_native_state38_campaign_member1_holds_member1
  type nonrec recall = json_string
  type nonrec fallen_companions = json_string
  type nonrec carry_gold = json_number
  type nonrec dialogue = fragments_native_state38_campaign_member1_dialogue_item
  type nonrec dialogue_index = json_number
  type nonrec outcome = json_string
  type nonrec veteran = save2_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec story_veteran = save2_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec evacuated = save2_checkpoint_state_campaign_member1_evacuated_member1_item
  type nonrec choices = json_string
  type nonrec phase = save35_setup_first
  type nonrec controller = fragments_factions7_item_controller
  type nonrec palette = json_string
  type nonrec recruits = fragments_factions7_item_recruits_member1_item
  type nonrec relations = fragments_factions7_item_relations_member1
  type nonrec transfer = fragments_factions7_item_transfer_member1
  type nonrec item = fragments_native_state38_ground_items_item_item
  type nonrec recruiter = save35_setup_enemy_blocks_member1_recruiter
  type nonrec planner = save35_setup_enemy_blocks_member1_planner
  type nonrec armor = fragments_native_state38_ground_items_item_item_armor_member1
  type nonrec description = json_string
  type nonrec attacks = fragments_native_state38_ground_items_item_item_attacks_item
  type nonrec healing = fragments_native_state38_ground_items_item_item_healing_member1
  type nonrec condition = fragments_rule11_condition
  type nonrec remember = fragments_rule11_remember
  type nonrec radius = json_number
  type nonrec color = fragments_light23_color_member1
  type nonrec fixture = fragments_rule11_remember
  type nonrec dx = json_number
  type nonrec dy = json_number
  type nonrec power = json_number
  type nonrec rule = fragments_rule11
  type nonrec held_turns = json_number
  type nonrec last_counted_round = json_number
  type nonrec escalated = json_boolean
  type nonrec speaker = json_string
  type nonrec portrait = json_string
  type nonrec text = json_string
  type nonrec event = json_string
  type nonrec controller_case = fragments_factions7_item_controller_member0
  type nonrec cost = json_number
  type nonrec to_ = json_string
  type nonrec blade = json_number
  type nonrec pierce = json_number
  type nonrec impact = json_number
  type nonrec damage = json_number
  type nonrec spread = json_number
  type nonrec count = json_number
  type nonrec damage_type = fragments_native_state38_ground_items_item_item_attacks_item_damage_type
  type nonrec category = fragments_native_state38_ground_items_item_item_attacks_item_category
  type nonrec range = json_number
  type nonrec drain = json_number
  type nonrec amount = json_number
  type nonrec condition_case = fragments_rule11_condition_member0
  type nonrec unit_types = json_string
  type nonrec choice = json_string
  type nonrec flag = json_string
  type nonrec unit_type = json_string
  type nonrec conditions = fragments_rule11_condition
end

module V24 = struct
  type nonrec t = save24
  type nonrec version = literal_24
  type nonrec checkpoint = save21_checkpoint
  type nonrec pending_enemy = json_boolean
  type nonrec setup = save2_setup_member1
  type nonrec progress = save2_progress_member1
  type nonrec undo = json_unknown
  type nonrec seed = json_number
  type nonrec state = save21_checkpoint_state
  type nonrec first = save35_setup_first
  type nonrec enemy = save2_setup_member1_enemy
  type nonrec entry = history1_entries_item
  type nonrec round = json_number
  type nonrec side = save35_setup_first
  type nonrec winner = fragments_native_state38_winner
  type nonrec armies = fragments_native_state38_armies_member1
  type nonrec condition_memory = json_string
  type nonrec map_id = json_string
  type nonrec map_revision = json_number
  type nonrec cols = json_number
  type nonrec rows = json_number
  type nonrec next_id = json_number
  type nonrec log = json_string
  type nonrec last_combat = fragments_native_state38_last_combat_member2
  type nonrec unit_ = save2_checkpoint_state_units_item
  type nonrec tile = save18_checkpoint_state_tiles_item
  type nonrec campaign = save2_checkpoint_state_campaign_member1
  type nonrec gold = fragments_native_state38_gold
  type nonrec faction = fragments_factions7_item
  type nonrec ground_items = fragments_native_state38_ground_items_item
  type nonrec combat_rng = json_number
  type nonrec policy = save35_setup_enemy_policy
  type nonrec blocks = save35_setup_enemy_blocks_member1
  type nonrec recruitment_limit = json_number
  type nonrec diplomacy = save2_setup_member1_enemy_diplomacy
  type nonrec id = json_string
  type nonrec saved_at = json_number
  type nonrec run = json_string
  type nonrec mission = json_string
  type nonrec mission_start = json_boolean
  type nonrec run_id = json_string
  type nonrec branch_id = json_string
  type nonrec checkpoint_id = json_string
  type nonrec ordinal = json_number
  type nonrec kind = history1_entries_item_kind
  type nonrec parent_id = json_string
  type nonrec attacker_id = json_string
  type nonrec attack_id = json_string
  type nonrec sound = json_string
  type nonrec role = fragments_native_state38_units_item_role
  type nonrec items = fragments_native_state38_ground_items_item_item
  type nonrec ai_group = json_string
  type nonrec type_ = json_string
  type nonrec hp = json_number
  type nonrec moved = json_boolean
  type nonrec attacked = json_boolean
  type nonrec retreating = fragments_rule11_remember
  type nonrec rebuilding = fragments_rule11_remember
  type nonrec name = json_string
  type nonrec character = json_string
  type nonrec experience = json_number
  type nonrec owner = json_string
  type nonrec location = json_string
  type nonrec col = json_number
  type nonrec row = json_number
  type nonrec terrain = json_string
  type nonrec connection = fragments_tiles11_item_connections_member1_item
  type nonrec dead_ends = fragments_tiles11_item_connections_member1_item
  type nonrec walls = fragments_tiles11_item_connections_member1_item
  type nonrec floor = json_string
  type nonrec fog = fragments_rule11
  type nonrec light = fragments_light23
  type nonrec vigil = fragments_native_state38_campaign_member1_vigil_member1
  type nonrec refuge = fragments_native_state38_campaign_member1_refuge_member1
  type nonrec completed = json_boolean
  type nonrec mark = json_string
  type nonrec hold = fragments_native_state38_campaign_member1_holds_member1
  type nonrec recall = json_string
  type nonrec fallen_companions = json_string
  type nonrec carry_gold = json_number
  type nonrec dialogue = fragments_native_state38_campaign_member1_dialogue_item
  type nonrec dialogue_index = json_number
  type nonrec outcome = json_string
  type nonrec veteran = save2_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec story_veteran = save2_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec evacuated = save2_checkpoint_state_campaign_member1_evacuated_member1_item
  type nonrec choices = json_string
  type nonrec phase = save35_setup_first
  type nonrec controller = fragments_factions7_item_controller
  type nonrec palette = json_string
  type nonrec recruits = fragments_factions7_item_recruits_member1_item
  type nonrec relations = fragments_factions7_item_relations_member1
  type nonrec transfer = fragments_factions7_item_transfer_member1
  type nonrec item = fragments_native_state38_ground_items_item_item
  type nonrec recruiter = save35_setup_enemy_blocks_member1_recruiter
  type nonrec planner = save35_setup_enemy_blocks_member1_planner
  type nonrec armor = fragments_native_state38_ground_items_item_item_armor_member1
  type nonrec description = json_string
  type nonrec attacks = fragments_native_state38_ground_items_item_item_attacks_item
  type nonrec healing = fragments_native_state38_ground_items_item_item_healing_member1
  type nonrec condition = fragments_rule11_condition
  type nonrec remember = fragments_rule11_remember
  type nonrec radius = json_number
  type nonrec color = fragments_light23_color_member1
  type nonrec fixture = fragments_rule11_remember
  type nonrec dx = json_number
  type nonrec dy = json_number
  type nonrec power = json_number
  type nonrec rule = fragments_rule11
  type nonrec held_turns = json_number
  type nonrec last_counted_round = json_number
  type nonrec escalated = json_boolean
  type nonrec speaker = json_string
  type nonrec portrait = json_string
  type nonrec text = json_string
  type nonrec event = json_string
  type nonrec controller_case = fragments_factions7_item_controller_member0
  type nonrec cost = json_number
  type nonrec to_ = json_string
  type nonrec blade = json_number
  type nonrec pierce = json_number
  type nonrec impact = json_number
  type nonrec damage = json_number
  type nonrec spread = json_number
  type nonrec count = json_number
  type nonrec damage_type = fragments_native_state38_ground_items_item_item_attacks_item_damage_type
  type nonrec category = fragments_native_state38_ground_items_item_item_attacks_item_category
  type nonrec range = json_number
  type nonrec drain = json_number
  type nonrec amount = json_number
  type nonrec condition_case = fragments_rule11_condition_member0
  type nonrec unit_types = json_string
  type nonrec choice = json_string
  type nonrec flag = json_string
  type nonrec unit_type = json_string
  type nonrec conditions = fragments_rule11_condition
end

module V25 = struct
  type nonrec t = save25
  type nonrec version = literal_25
  type nonrec checkpoint = save21_checkpoint
  type nonrec pending_enemy = json_boolean
  type nonrec setup = save2_setup_member1
  type nonrec progress = save2_progress_member1
  type nonrec undo = json_unknown
  type nonrec seed = json_number
  type nonrec state = save21_checkpoint_state
  type nonrec first = save35_setup_first
  type nonrec enemy = save2_setup_member1_enemy
  type nonrec entry = history1_entries_item
  type nonrec round = json_number
  type nonrec side = save35_setup_first
  type nonrec winner = fragments_native_state38_winner
  type nonrec armies = fragments_native_state38_armies_member1
  type nonrec condition_memory = json_string
  type nonrec map_id = json_string
  type nonrec map_revision = json_number
  type nonrec cols = json_number
  type nonrec rows = json_number
  type nonrec next_id = json_number
  type nonrec log = json_string
  type nonrec last_combat = fragments_native_state38_last_combat_member2
  type nonrec unit_ = save2_checkpoint_state_units_item
  type nonrec tile = save18_checkpoint_state_tiles_item
  type nonrec campaign = save2_checkpoint_state_campaign_member1
  type nonrec gold = fragments_native_state38_gold
  type nonrec faction = fragments_factions7_item
  type nonrec ground_items = fragments_native_state38_ground_items_item
  type nonrec combat_rng = json_number
  type nonrec policy = save35_setup_enemy_policy
  type nonrec blocks = save35_setup_enemy_blocks_member1
  type nonrec recruitment_limit = json_number
  type nonrec diplomacy = save2_setup_member1_enemy_diplomacy
  type nonrec id = json_string
  type nonrec saved_at = json_number
  type nonrec run = json_string
  type nonrec mission = json_string
  type nonrec mission_start = json_boolean
  type nonrec run_id = json_string
  type nonrec branch_id = json_string
  type nonrec checkpoint_id = json_string
  type nonrec ordinal = json_number
  type nonrec kind = history1_entries_item_kind
  type nonrec parent_id = json_string
  type nonrec attacker_id = json_string
  type nonrec attack_id = json_string
  type nonrec sound = json_string
  type nonrec role = fragments_native_state38_units_item_role
  type nonrec items = fragments_native_state38_ground_items_item_item
  type nonrec ai_group = json_string
  type nonrec type_ = json_string
  type nonrec hp = json_number
  type nonrec moved = json_boolean
  type nonrec attacked = json_boolean
  type nonrec retreating = fragments_rule11_remember
  type nonrec rebuilding = fragments_rule11_remember
  type nonrec name = json_string
  type nonrec character = json_string
  type nonrec experience = json_number
  type nonrec owner = json_string
  type nonrec location = json_string
  type nonrec col = json_number
  type nonrec row = json_number
  type nonrec terrain = json_string
  type nonrec connection = fragments_tiles11_item_connections_member1_item
  type nonrec dead_ends = fragments_tiles11_item_connections_member1_item
  type nonrec walls = fragments_tiles11_item_connections_member1_item
  type nonrec floor = json_string
  type nonrec fog = fragments_rule11
  type nonrec light = fragments_light23
  type nonrec vigil = fragments_native_state38_campaign_member1_vigil_member1
  type nonrec refuge = fragments_native_state38_campaign_member1_refuge_member1
  type nonrec completed = json_boolean
  type nonrec mark = json_string
  type nonrec hold = fragments_native_state38_campaign_member1_holds_member1
  type nonrec recall = json_string
  type nonrec fallen_companions = json_string
  type nonrec carry_gold = json_number
  type nonrec dialogue = fragments_native_state38_campaign_member1_dialogue_item
  type nonrec dialogue_index = json_number
  type nonrec outcome = json_string
  type nonrec veteran = save2_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec story_veteran = save2_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec evacuated = save2_checkpoint_state_campaign_member1_evacuated_member1_item
  type nonrec choices = json_string
  type nonrec phase = save35_setup_first
  type nonrec controller = fragments_factions7_item_controller
  type nonrec palette = json_string
  type nonrec recruits = fragments_factions7_item_recruits_member1_item
  type nonrec relations = fragments_factions7_item_relations_member1
  type nonrec transfer = fragments_factions7_item_transfer_member1
  type nonrec item = fragments_native_state38_ground_items_item_item
  type nonrec recruiter = save35_setup_enemy_blocks_member1_recruiter
  type nonrec planner = save35_setup_enemy_blocks_member1_planner
  type nonrec armor = fragments_native_state38_ground_items_item_item_armor_member1
  type nonrec description = json_string
  type nonrec attacks = fragments_native_state38_ground_items_item_item_attacks_item
  type nonrec healing = fragments_native_state38_ground_items_item_item_healing_member1
  type nonrec condition = fragments_rule11_condition
  type nonrec remember = fragments_rule11_remember
  type nonrec radius = json_number
  type nonrec color = fragments_light23_color_member1
  type nonrec fixture = fragments_rule11_remember
  type nonrec dx = json_number
  type nonrec dy = json_number
  type nonrec power = json_number
  type nonrec rule = fragments_rule11
  type nonrec held_turns = json_number
  type nonrec last_counted_round = json_number
  type nonrec escalated = json_boolean
  type nonrec speaker = json_string
  type nonrec portrait = json_string
  type nonrec text = json_string
  type nonrec event = json_string
  type nonrec controller_case = fragments_factions7_item_controller_member0
  type nonrec cost = json_number
  type nonrec to_ = json_string
  type nonrec blade = json_number
  type nonrec pierce = json_number
  type nonrec impact = json_number
  type nonrec damage = json_number
  type nonrec spread = json_number
  type nonrec count = json_number
  type nonrec damage_type = fragments_native_state38_ground_items_item_item_attacks_item_damage_type
  type nonrec category = fragments_native_state38_ground_items_item_item_attacks_item_category
  type nonrec range = json_number
  type nonrec drain = json_number
  type nonrec amount = json_number
  type nonrec condition_case = fragments_rule11_condition_member0
  type nonrec unit_types = json_string
  type nonrec choice = json_string
  type nonrec flag = json_string
  type nonrec unit_type = json_string
  type nonrec conditions = fragments_rule11_condition
end

module V26 = struct
  type nonrec t = save26
  type nonrec version = literal_26
  type nonrec checkpoint = save26_checkpoint
  type nonrec pending_enemy = json_boolean
  type nonrec setup = save2_setup_member1
  type nonrec progress = save2_progress_member1
  type nonrec undo = json_unknown
  type nonrec seed = json_number
  type nonrec state = save26_checkpoint_state
  type nonrec first = save35_setup_first
  type nonrec enemy = save2_setup_member1_enemy
  type nonrec entry = history1_entries_item
  type nonrec round = json_number
  type nonrec side = save35_setup_first
  type nonrec winner = fragments_native_state38_winner
  type nonrec armies = fragments_native_state38_armies_member1
  type nonrec condition_memory = json_string
  type nonrec map_id = json_string
  type nonrec map_revision = json_number
  type nonrec cols = json_number
  type nonrec rows = json_number
  type nonrec next_id = json_number
  type nonrec log = json_string
  type nonrec last_combat = fragments_native_state38_last_combat_member2
  type nonrec unit_ = save26_checkpoint_state_units_item
  type nonrec tile = save26_checkpoint_state_tiles_item
  type nonrec campaign = save26_checkpoint_state_campaign_member1
  type nonrec gold = fragments_native_state38_gold
  type nonrec faction = save26_checkpoint_state_factions_item
  type nonrec ground_items = fragments_native_state38_ground_items_item
  type nonrec combat_rng = json_number
  type nonrec policy = save35_setup_enemy_policy
  type nonrec blocks = save35_setup_enemy_blocks_member1
  type nonrec recruitment_limit = json_number
  type nonrec diplomacy = save2_setup_member1_enemy_diplomacy
  type nonrec id = json_string
  type nonrec saved_at = json_number
  type nonrec run = json_string
  type nonrec mission = json_string
  type nonrec mission_start = json_boolean
  type nonrec run_id = json_string
  type nonrec branch_id = json_string
  type nonrec checkpoint_id = json_string
  type nonrec ordinal = json_number
  type nonrec kind = history1_entries_item_kind
  type nonrec parent_id = json_string
  type nonrec attacker_id = json_string
  type nonrec attack_id = json_string
  type nonrec sound = json_string
  type nonrec role = fragments_native_state38_units_item_role
  type nonrec items = fragments_native_state38_ground_items_item_item
  type nonrec ai_group = json_string
  type nonrec type_ = json_string
  type nonrec hp = json_number
  type nonrec moved = json_boolean
  type nonrec attacked = json_boolean
  type nonrec retreating = fragments_rule11_remember
  type nonrec rebuilding = fragments_rule11_remember
  type nonrec character = json_string
  type nonrec experience = json_number
  type nonrec owner = json_string
  type nonrec location = json_string
  type nonrec col = json_number
  type nonrec row = json_number
  type nonrec terrain = json_string
  type nonrec connection = fragments_tiles11_item_connections_member1_item
  type nonrec dead_ends = fragments_tiles11_item_connections_member1_item
  type nonrec walls = fragments_tiles11_item_connections_member1_item
  type nonrec floor = json_string
  type nonrec fog = fragments_light28_member1_when__member1
  type nonrec light = fragments_light28_member1
  type nonrec vigil = fragments_native_state38_campaign_member1_vigil_member1
  type nonrec refuge = fragments_native_state38_campaign_member1_refuge_member1
  type nonrec completed = json_boolean
  type nonrec mark = json_string
  type nonrec hold = fragments_native_state38_campaign_member1_holds_member1
  type nonrec recall = json_string
  type nonrec fallen_companions = json_string
  type nonrec carry_gold = json_number
  type nonrec dialogue = fragments_native_state38_campaign_member1_dialogue_item
  type nonrec dialogue_index = json_number
  type nonrec outcome = json_string
  type nonrec veteran = save26_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec story_veteran = save26_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec evacuated = save26_checkpoint_state_campaign_member1_evacuated_member1_item
  type nonrec choices = json_string
  type nonrec name = json_string
  type nonrec phase = save35_setup_first
  type nonrec controller = fragments_factions7_item_controller
  type nonrec palette = json_string
  type nonrec recruits = fragments_factions7_item_recruits_member1_item
  type nonrec relations = fragments_factions7_item_relations_member1
  type nonrec transfer = save26_checkpoint_state_factions_item_transfer_member1
  type nonrec item = fragments_native_state38_ground_items_item_item
  type nonrec recruiter = save35_setup_enemy_blocks_member1_recruiter
  type nonrec planner = save35_setup_enemy_blocks_member1_planner
  type nonrec armor = fragments_native_state38_ground_items_item_item_armor_member1
  type nonrec description = json_string
  type nonrec attacks = fragments_native_state38_ground_items_item_item_attacks_item
  type nonrec healing = fragments_native_state38_ground_items_item_item_healing_member1
  type nonrec condition = fragments_light28_member1_when__member1_condition
  type nonrec remember = fragments_rule11_remember
  type nonrec radius = json_number
  type nonrec color = fragments_light23_color_member1
  type nonrec fixture = fragments_rule11_remember
  type nonrec dx = json_number
  type nonrec dy = json_number
  type nonrec power = json_number
  type nonrec rule = fragments_light28_member1_when__member1
  type nonrec held_turns = json_number
  type nonrec last_counted_round = json_number
  type nonrec escalated = json_boolean
  type nonrec speaker = json_string
  type nonrec portrait = json_string
  type nonrec text = json_string
  type nonrec event = json_string
  type nonrec controller_case = fragments_factions7_item_controller_member0
  type nonrec cost = json_number
  type nonrec to_ = json_string
  type nonrec blade = json_number
  type nonrec pierce = json_number
  type nonrec impact = json_number
  type nonrec damage = json_number
  type nonrec spread = json_number
  type nonrec count = json_number
  type nonrec damage_type = fragments_native_state38_ground_items_item_item_attacks_item_damage_type
  type nonrec category = fragments_native_state38_ground_items_item_item_attacks_item_category
  type nonrec range = json_number
  type nonrec drain = json_number
  type nonrec amount = json_number
  type nonrec condition_case = fragments_rule11_condition_member0
  type nonrec unit_types = json_string
  type nonrec unit_type = json_string
  type nonrec choice = json_string
  type nonrec flag = json_string
  type nonrec conditions = fragments_light28_member1_when__member1_condition
end

module V27 = struct
  type nonrec t = save27
  type nonrec version = literal_27
  type nonrec checkpoint = save26_checkpoint
  type nonrec pending_enemy = json_boolean
  type nonrec setup = save2_setup_member1
  type nonrec progress = save2_progress_member1
  type nonrec undo = json_unknown
  type nonrec seed = json_number
  type nonrec state = save26_checkpoint_state
  type nonrec first = save35_setup_first
  type nonrec enemy = save2_setup_member1_enemy
  type nonrec entry = history1_entries_item
  type nonrec round = json_number
  type nonrec side = save35_setup_first
  type nonrec winner = fragments_native_state38_winner
  type nonrec armies = fragments_native_state38_armies_member1
  type nonrec condition_memory = json_string
  type nonrec map_id = json_string
  type nonrec map_revision = json_number
  type nonrec cols = json_number
  type nonrec rows = json_number
  type nonrec next_id = json_number
  type nonrec log = json_string
  type nonrec last_combat = fragments_native_state38_last_combat_member2
  type nonrec unit_ = save26_checkpoint_state_units_item
  type nonrec tile = save26_checkpoint_state_tiles_item
  type nonrec campaign = save26_checkpoint_state_campaign_member1
  type nonrec gold = fragments_native_state38_gold
  type nonrec faction = save26_checkpoint_state_factions_item
  type nonrec ground_items = fragments_native_state38_ground_items_item
  type nonrec combat_rng = json_number
  type nonrec policy = save35_setup_enemy_policy
  type nonrec blocks = save35_setup_enemy_blocks_member1
  type nonrec recruitment_limit = json_number
  type nonrec diplomacy = save2_setup_member1_enemy_diplomacy
  type nonrec id = json_string
  type nonrec saved_at = json_number
  type nonrec run = json_string
  type nonrec mission = json_string
  type nonrec mission_start = json_boolean
  type nonrec run_id = json_string
  type nonrec branch_id = json_string
  type nonrec checkpoint_id = json_string
  type nonrec ordinal = json_number
  type nonrec kind = history1_entries_item_kind
  type nonrec parent_id = json_string
  type nonrec attacker_id = json_string
  type nonrec attack_id = json_string
  type nonrec sound = json_string
  type nonrec role = fragments_native_state38_units_item_role
  type nonrec items = fragments_native_state38_ground_items_item_item
  type nonrec ai_group = json_string
  type nonrec type_ = json_string
  type nonrec hp = json_number
  type nonrec moved = json_boolean
  type nonrec attacked = json_boolean
  type nonrec retreating = fragments_rule11_remember
  type nonrec rebuilding = fragments_rule11_remember
  type nonrec character = json_string
  type nonrec experience = json_number
  type nonrec owner = json_string
  type nonrec location = json_string
  type nonrec col = json_number
  type nonrec row = json_number
  type nonrec terrain = json_string
  type nonrec connection = fragments_tiles11_item_connections_member1_item
  type nonrec dead_ends = fragments_tiles11_item_connections_member1_item
  type nonrec walls = fragments_tiles11_item_connections_member1_item
  type nonrec floor = json_string
  type nonrec fog = fragments_light28_member1_when__member1
  type nonrec light = fragments_light28_member1
  type nonrec vigil = fragments_native_state38_campaign_member1_vigil_member1
  type nonrec refuge = fragments_native_state38_campaign_member1_refuge_member1
  type nonrec completed = json_boolean
  type nonrec mark = json_string
  type nonrec hold = fragments_native_state38_campaign_member1_holds_member1
  type nonrec recall = json_string
  type nonrec fallen_companions = json_string
  type nonrec carry_gold = json_number
  type nonrec dialogue = fragments_native_state38_campaign_member1_dialogue_item
  type nonrec dialogue_index = json_number
  type nonrec outcome = json_string
  type nonrec veteran = save26_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec story_veteran = save26_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec evacuated = save26_checkpoint_state_campaign_member1_evacuated_member1_item
  type nonrec choices = json_string
  type nonrec name = json_string
  type nonrec phase = save35_setup_first
  type nonrec controller = fragments_factions7_item_controller
  type nonrec palette = json_string
  type nonrec recruits = fragments_factions7_item_recruits_member1_item
  type nonrec relations = fragments_factions7_item_relations_member1
  type nonrec transfer = save26_checkpoint_state_factions_item_transfer_member1
  type nonrec item = fragments_native_state38_ground_items_item_item
  type nonrec recruiter = save35_setup_enemy_blocks_member1_recruiter
  type nonrec planner = save35_setup_enemy_blocks_member1_planner
  type nonrec armor = fragments_native_state38_ground_items_item_item_armor_member1
  type nonrec description = json_string
  type nonrec attacks = fragments_native_state38_ground_items_item_item_attacks_item
  type nonrec healing = fragments_native_state38_ground_items_item_item_healing_member1
  type nonrec condition = fragments_light28_member1_when__member1_condition
  type nonrec remember = fragments_rule11_remember
  type nonrec radius = json_number
  type nonrec color = fragments_light23_color_member1
  type nonrec fixture = fragments_rule11_remember
  type nonrec dx = json_number
  type nonrec dy = json_number
  type nonrec power = json_number
  type nonrec rule = fragments_light28_member1_when__member1
  type nonrec held_turns = json_number
  type nonrec last_counted_round = json_number
  type nonrec escalated = json_boolean
  type nonrec speaker = json_string
  type nonrec portrait = json_string
  type nonrec text = json_string
  type nonrec event = json_string
  type nonrec controller_case = fragments_factions7_item_controller_member0
  type nonrec cost = json_number
  type nonrec to_ = json_string
  type nonrec blade = json_number
  type nonrec pierce = json_number
  type nonrec impact = json_number
  type nonrec damage = json_number
  type nonrec spread = json_number
  type nonrec count = json_number
  type nonrec damage_type = fragments_native_state38_ground_items_item_item_attacks_item_damage_type
  type nonrec category = fragments_native_state38_ground_items_item_item_attacks_item_category
  type nonrec range = json_number
  type nonrec drain = json_number
  type nonrec amount = json_number
  type nonrec condition_case = fragments_rule11_condition_member0
  type nonrec unit_types = json_string
  type nonrec unit_type = json_string
  type nonrec choice = json_string
  type nonrec flag = json_string
  type nonrec conditions = fragments_light28_member1_when__member1_condition
end

module V28 = struct
  type nonrec t = save28
  type nonrec version = literal_28
  type nonrec checkpoint = save28_checkpoint
  type nonrec pending_enemy = json_boolean
  type nonrec setup = save2_setup_member1
  type nonrec progress = save2_progress_member1
  type nonrec undo = json_unknown
  type nonrec seed = json_number
  type nonrec state = save28_checkpoint_state
  type nonrec first = save35_setup_first
  type nonrec enemy = save2_setup_member1_enemy
  type nonrec entry = history1_entries_item
  type nonrec round = json_number
  type nonrec side = save35_setup_first
  type nonrec winner = fragments_native_state38_winner
  type nonrec armies = fragments_native_state38_armies_member1
  type nonrec condition_memory = json_string
  type nonrec map_id = json_string
  type nonrec map_revision = json_number
  type nonrec cols = json_number
  type nonrec rows = json_number
  type nonrec next_id = json_number
  type nonrec log = json_string
  type nonrec last_combat = fragments_native_state38_last_combat_member2
  type nonrec unit_ = save26_checkpoint_state_units_item
  type nonrec tile = save26_checkpoint_state_tiles_item
  type nonrec campaign = save28_checkpoint_state_campaign_member1
  type nonrec gold = fragments_native_state38_gold
  type nonrec faction = save26_checkpoint_state_factions_item
  type nonrec ground_items = fragments_native_state38_ground_items_item
  type nonrec combat_rng = json_number
  type nonrec policy = save35_setup_enemy_policy
  type nonrec blocks = save35_setup_enemy_blocks_member1
  type nonrec recruitment_limit = json_number
  type nonrec diplomacy = save2_setup_member1_enemy_diplomacy
  type nonrec id = json_string
  type nonrec saved_at = json_number
  type nonrec run = json_string
  type nonrec mission = json_string
  type nonrec mission_start = json_boolean
  type nonrec run_id = json_string
  type nonrec branch_id = json_string
  type nonrec checkpoint_id = json_string
  type nonrec ordinal = json_number
  type nonrec kind = history1_entries_item_kind
  type nonrec parent_id = json_string
  type nonrec attacker_id = json_string
  type nonrec attack_id = json_string
  type nonrec sound = json_string
  type nonrec role = fragments_native_state38_units_item_role
  type nonrec items = fragments_native_state38_ground_items_item_item
  type nonrec ai_group = json_string
  type nonrec type_ = json_string
  type nonrec hp = json_number
  type nonrec moved = json_boolean
  type nonrec attacked = json_boolean
  type nonrec retreating = fragments_rule11_remember
  type nonrec rebuilding = fragments_rule11_remember
  type nonrec character = json_string
  type nonrec experience = json_number
  type nonrec owner = json_string
  type nonrec location = json_string
  type nonrec col = json_number
  type nonrec row = json_number
  type nonrec terrain = json_string
  type nonrec connection = fragments_tiles11_item_connections_member1_item
  type nonrec dead_ends = fragments_tiles11_item_connections_member1_item
  type nonrec walls = fragments_tiles11_item_connections_member1_item
  type nonrec floor = json_string
  type nonrec fog = fragments_light28_member1_when__member1
  type nonrec light = fragments_light28_member1
  type nonrec vigil = fragments_native_state38_campaign_member1_vigil_member1
  type nonrec refuge = fragments_native_state38_campaign_member1_refuge_member1
  type nonrec completed = json_boolean
  type nonrec mark = json_string
  type nonrec hold = fragments_native_state38_campaign_member1_holds_member1
  type nonrec recall = json_string
  type nonrec fallen_companions = json_string
  type nonrec carry_gold = json_number
  type nonrec dialogue = fragments_native_state38_campaign_member1_dialogue_item
  type nonrec dialogue_index = json_number
  type nonrec outcome = json_string
  type nonrec veteran = save26_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec story_veteran = save26_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec evacuated = save26_checkpoint_state_campaign_member1_evacuated_member1_item
  type nonrec scripts = save28_checkpoint_state_campaign_member1_scripts_member1
  type nonrec choices = json_string
  type nonrec name = json_string
  type nonrec phase = save35_setup_first
  type nonrec controller = fragments_factions7_item_controller
  type nonrec palette = json_string
  type nonrec recruits = fragments_factions7_item_recruits_member1_item
  type nonrec relations = fragments_factions7_item_relations_member1
  type nonrec transfer = save26_checkpoint_state_factions_item_transfer_member1
  type nonrec item = fragments_native_state38_ground_items_item_item
  type nonrec recruiter = save35_setup_enemy_blocks_member1_recruiter
  type nonrec planner = save35_setup_enemy_blocks_member1_planner
  type nonrec armor = fragments_native_state38_ground_items_item_item_armor_member1
  type nonrec description = json_string
  type nonrec attacks = fragments_native_state38_ground_items_item_item_attacks_item
  type nonrec healing = fragments_native_state38_ground_items_item_item_healing_member1
  type nonrec condition = fragments_light28_member1_when__member1_condition
  type nonrec remember = fragments_rule11_remember
  type nonrec radius = json_number
  type nonrec color = fragments_light23_color_member1
  type nonrec fixture = fragments_rule11_remember
  type nonrec dx = json_number
  type nonrec dy = json_number
  type nonrec power = json_number
  type nonrec rule = fragments_light28_member1_when__member1
  type nonrec held_turns = json_number
  type nonrec last_counted_round = json_number
  type nonrec escalated = json_boolean
  type nonrec speaker = json_string
  type nonrec portrait = json_string
  type nonrec text = json_string
  type nonrec event = json_string
  type nonrec recruitment = fragments_native_state38_campaign_member1_scripts_member1_recruitment
  type nonrec failed = json_string
  type nonrec active = json_string
  type nonrec next_mark = json_number
  type nonrec pending = fragments_native_state38_campaign_member1_scripts_member1_pending_item
  type nonrec revealed = json_string
  type nonrec controller_case = fragments_factions7_item_controller_member0
  type nonrec cost = json_number
  type nonrec to_ = json_string
  type nonrec blade = json_number
  type nonrec pierce = json_number
  type nonrec impact = json_number
  type nonrec damage = json_number
  type nonrec spread = json_number
  type nonrec count = json_number
  type nonrec damage_type = fragments_native_state38_ground_items_item_item_attacks_item_damage_type
  type nonrec category = fragments_native_state38_ground_items_item_item_attacks_item_category
  type nonrec range = json_number
  type nonrec drain = json_number
  type nonrec amount = json_number
  type nonrec condition_case = fragments_rule11_condition_member0
  type nonrec predicate = json_number
  type nonrec turns = json_number
  type nonrec last_round = json_number
  type nonrec area = json_string
  type nonrec light_remembered = fragments_rule11_remember
  type nonrec unit_types = json_string
  type nonrec unit_type = json_string
  type nonrec choice = json_string
  type nonrec flag = json_string
  type nonrec conditions = fragments_light28_member1_when__member1_condition
end

module V29 = struct
  type nonrec t = save29
  type nonrec version = literal_29
  type nonrec checkpoint = save29_checkpoint
  type nonrec pending_enemy = json_boolean
  type nonrec setup = save2_setup_member1
  type nonrec progress = save2_progress_member1
  type nonrec undo = json_unknown
  type nonrec seed = json_number
  type nonrec state = save29_checkpoint_state
  type nonrec first = save35_setup_first
  type nonrec enemy = save2_setup_member1_enemy
  type nonrec entry = history1_entries_item
  type nonrec round = json_number
  type nonrec side = save35_setup_first
  type nonrec winner = fragments_native_state38_winner
  type nonrec armies = fragments_native_state38_armies_member1
  type nonrec condition_memory = json_string
  type nonrec map_id = json_string
  type nonrec map_revision = json_number
  type nonrec cols = json_number
  type nonrec rows = json_number
  type nonrec next_id = json_number
  type nonrec log = json_string
  type nonrec last_combat = fragments_native_state38_last_combat_member2
  type nonrec unit_ = save26_checkpoint_state_units_item
  type nonrec tile = save26_checkpoint_state_tiles_item
  type nonrec campaign = save29_checkpoint_state_campaign_member1
  type nonrec gold = fragments_native_state38_gold
  type nonrec faction = save26_checkpoint_state_factions_item
  type nonrec ground_items = fragments_native_state38_ground_items_item
  type nonrec combat_rng = json_number
  type nonrec policy = save35_setup_enemy_policy
  type nonrec blocks = save35_setup_enemy_blocks_member1
  type nonrec recruitment_limit = json_number
  type nonrec diplomacy = save2_setup_member1_enemy_diplomacy
  type nonrec id = json_string
  type nonrec saved_at = json_number
  type nonrec run = json_string
  type nonrec mission = json_string
  type nonrec mission_start = json_boolean
  type nonrec run_id = json_string
  type nonrec branch_id = json_string
  type nonrec checkpoint_id = json_string
  type nonrec ordinal = json_number
  type nonrec kind = history1_entries_item_kind
  type nonrec parent_id = json_string
  type nonrec attacker_id = json_string
  type nonrec attack_id = json_string
  type nonrec sound = json_string
  type nonrec role = fragments_native_state38_units_item_role
  type nonrec items = fragments_native_state38_ground_items_item_item
  type nonrec ai_group = json_string
  type nonrec type_ = json_string
  type nonrec hp = json_number
  type nonrec moved = json_boolean
  type nonrec attacked = json_boolean
  type nonrec retreating = fragments_rule11_remember
  type nonrec rebuilding = fragments_rule11_remember
  type nonrec character = json_string
  type nonrec experience = json_number
  type nonrec owner = json_string
  type nonrec location = json_string
  type nonrec col = json_number
  type nonrec row = json_number
  type nonrec terrain = json_string
  type nonrec connection = fragments_tiles11_item_connections_member1_item
  type nonrec dead_ends = fragments_tiles11_item_connections_member1_item
  type nonrec walls = fragments_tiles11_item_connections_member1_item
  type nonrec floor = json_string
  type nonrec fog = fragments_light28_member1_when__member1
  type nonrec light = fragments_light28_member1
  type nonrec vigil = fragments_native_state38_campaign_member1_vigil_member1
  type nonrec refuge = fragments_native_state38_campaign_member1_refuge_member1
  type nonrec completed = json_boolean
  type nonrec mark = json_string
  type nonrec hold = fragments_native_state38_campaign_member1_holds_member1
  type nonrec recall = json_string
  type nonrec fallen_companions = json_string
  type nonrec carry_gold = json_number
  type nonrec dialogue = fragments_native_state38_campaign_member1_dialogue_item
  type nonrec dialogue_index = json_number
  type nonrec outcome = json_string
  type nonrec veteran = save26_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec story_veteran = save26_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec evacuated = save26_checkpoint_state_campaign_member1_evacuated_member1_item
  type nonrec scripts = save28_checkpoint_state_campaign_member1_scripts_member1
  type nonrec flags = fragments_native_state38_campaign_member1_flags
  type nonrec name = json_string
  type nonrec phase = save35_setup_first
  type nonrec controller = fragments_factions7_item_controller
  type nonrec palette = json_string
  type nonrec recruits = fragments_factions7_item_recruits_member1_item
  type nonrec relations = fragments_factions7_item_relations_member1
  type nonrec transfer = save26_checkpoint_state_factions_item_transfer_member1
  type nonrec item = fragments_native_state38_ground_items_item_item
  type nonrec recruiter = save35_setup_enemy_blocks_member1_recruiter
  type nonrec planner = save35_setup_enemy_blocks_member1_planner
  type nonrec armor = fragments_native_state38_ground_items_item_item_armor_member1
  type nonrec description = json_string
  type nonrec attacks = fragments_native_state38_ground_items_item_item_attacks_item
  type nonrec healing = fragments_native_state38_ground_items_item_item_healing_member1
  type nonrec condition = fragments_light28_member1_when__member1_condition
  type nonrec remember = fragments_rule11_remember
  type nonrec radius = json_number
  type nonrec color = fragments_light23_color_member1
  type nonrec fixture = fragments_rule11_remember
  type nonrec dx = json_number
  type nonrec dy = json_number
  type nonrec power = json_number
  type nonrec rule = fragments_light28_member1_when__member1
  type nonrec held_turns = json_number
  type nonrec last_counted_round = json_number
  type nonrec escalated = json_boolean
  type nonrec speaker = json_string
  type nonrec portrait = json_string
  type nonrec text = json_string
  type nonrec event = json_string
  type nonrec recruitment = fragments_native_state38_campaign_member1_scripts_member1_recruitment
  type nonrec failed = json_string
  type nonrec active = json_string
  type nonrec next_mark = json_number
  type nonrec pending = fragments_native_state38_campaign_member1_scripts_member1_pending_item
  type nonrec revealed = json_string
  type nonrec controller_case = fragments_factions7_item_controller_member0
  type nonrec cost = json_number
  type nonrec to_ = json_string
  type nonrec blade = json_number
  type nonrec pierce = json_number
  type nonrec impact = json_number
  type nonrec damage = json_number
  type nonrec spread = json_number
  type nonrec count = json_number
  type nonrec damage_type = fragments_native_state38_ground_items_item_item_attacks_item_damage_type
  type nonrec category = fragments_native_state38_ground_items_item_item_attacks_item_category
  type nonrec range = json_number
  type nonrec drain = json_number
  type nonrec amount = json_number
  type nonrec condition_case = fragments_rule11_condition_member0
  type nonrec predicate = json_number
  type nonrec turns = json_number
  type nonrec last_round = json_number
  type nonrec area = json_string
  type nonrec light_remembered = fragments_rule11_remember
  type nonrec unit_types = json_string
  type nonrec unit_type = json_string
  type nonrec choice = json_string
  type nonrec flag = json_string
  type nonrec conditions = fragments_light28_member1_when__member1_condition
end

module V30 = struct
  type nonrec t = save30
  type nonrec version = literal_30
  type nonrec checkpoint = save29_checkpoint
  type nonrec pending_enemy = json_boolean
  type nonrec setup = save35_setup
  type nonrec progress = save2_progress_member1
  type nonrec undo = json_unknown
  type nonrec seed = json_number
  type nonrec state = save29_checkpoint_state
  type nonrec first = save35_setup_first
  type nonrec enemy = save35_setup_enemy
  type nonrec entry = history1_entries_item
  type nonrec round = json_number
  type nonrec side = save35_setup_first
  type nonrec winner = fragments_native_state38_winner
  type nonrec armies = fragments_native_state38_armies_member1
  type nonrec condition_memory = json_string
  type nonrec map_id = json_string
  type nonrec map_revision = json_number
  type nonrec cols = json_number
  type nonrec rows = json_number
  type nonrec next_id = json_number
  type nonrec log = json_string
  type nonrec last_combat = fragments_native_state38_last_combat_member2
  type nonrec unit_ = save26_checkpoint_state_units_item
  type nonrec tile = save26_checkpoint_state_tiles_item
  type nonrec campaign = save29_checkpoint_state_campaign_member1
  type nonrec gold = fragments_native_state38_gold
  type nonrec faction = save26_checkpoint_state_factions_item
  type nonrec ground_items = fragments_native_state38_ground_items_item
  type nonrec combat_rng = json_number
  type nonrec policy = save35_setup_enemy_policy
  type nonrec blocks = save35_setup_enemy_blocks_member1
  type nonrec recruitment_limit = json_number
  type nonrec diplomacy = save35_setup_enemy_diplomacy
  type nonrec id = json_string
  type nonrec saved_at = json_number
  type nonrec run = json_string
  type nonrec mission = json_string
  type nonrec mission_start = json_boolean
  type nonrec run_id = json_string
  type nonrec branch_id = json_string
  type nonrec checkpoint_id = json_string
  type nonrec ordinal = json_number
  type nonrec kind = history1_entries_item_kind
  type nonrec parent_id = json_string
  type nonrec attacker_id = json_string
  type nonrec attack_id = json_string
  type nonrec sound = json_string
  type nonrec role = fragments_native_state38_units_item_role
  type nonrec items = fragments_native_state38_ground_items_item_item
  type nonrec ai_group = json_string
  type nonrec type_ = json_string
  type nonrec hp = json_number
  type nonrec moved = json_boolean
  type nonrec attacked = json_boolean
  type nonrec retreating = fragments_rule11_remember
  type nonrec rebuilding = fragments_rule11_remember
  type nonrec character = json_string
  type nonrec experience = json_number
  type nonrec owner = json_string
  type nonrec location = json_string
  type nonrec col = json_number
  type nonrec row = json_number
  type nonrec terrain = json_string
  type nonrec connection = fragments_tiles11_item_connections_member1_item
  type nonrec dead_ends = fragments_tiles11_item_connections_member1_item
  type nonrec walls = fragments_tiles11_item_connections_member1_item
  type nonrec floor = json_string
  type nonrec fog = fragments_light28_member1_when__member1
  type nonrec light = fragments_light28_member1
  type nonrec vigil = fragments_native_state38_campaign_member1_vigil_member1
  type nonrec refuge = fragments_native_state38_campaign_member1_refuge_member1
  type nonrec completed = json_boolean
  type nonrec mark = json_string
  type nonrec hold = fragments_native_state38_campaign_member1_holds_member1
  type nonrec recall = json_string
  type nonrec fallen_companions = json_string
  type nonrec carry_gold = json_number
  type nonrec dialogue = fragments_native_state38_campaign_member1_dialogue_item
  type nonrec dialogue_index = json_number
  type nonrec outcome = json_string
  type nonrec veteran = save26_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec story_veteran = save26_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec evacuated = save26_checkpoint_state_campaign_member1_evacuated_member1_item
  type nonrec scripts = save28_checkpoint_state_campaign_member1_scripts_member1
  type nonrec flags = fragments_native_state38_campaign_member1_flags
  type nonrec name = json_string
  type nonrec phase = save35_setup_first
  type nonrec controller = fragments_factions7_item_controller
  type nonrec palette = json_string
  type nonrec recruits = fragments_factions7_item_recruits_member1_item
  type nonrec relations = fragments_factions7_item_relations_member1
  type nonrec transfer = save26_checkpoint_state_factions_item_transfer_member1
  type nonrec item = fragments_native_state38_ground_items_item_item
  type nonrec recruiter = save35_setup_enemy_blocks_member1_recruiter
  type nonrec planner = save35_setup_enemy_blocks_member1_planner
  type nonrec armor = fragments_native_state38_ground_items_item_item_armor_member1
  type nonrec description = json_string
  type nonrec attacks = fragments_native_state38_ground_items_item_item_attacks_item
  type nonrec healing = fragments_native_state38_ground_items_item_item_healing_member1
  type nonrec condition = fragments_light28_member1_when__member1_condition
  type nonrec remember = fragments_rule11_remember
  type nonrec radius = json_number
  type nonrec color = fragments_light23_color_member1
  type nonrec fixture = fragments_rule11_remember
  type nonrec dx = json_number
  type nonrec dy = json_number
  type nonrec power = json_number
  type nonrec rule = fragments_light28_member1_when__member1
  type nonrec held_turns = json_number
  type nonrec last_counted_round = json_number
  type nonrec escalated = json_boolean
  type nonrec speaker = json_string
  type nonrec portrait = json_string
  type nonrec text = json_string
  type nonrec event = json_string
  type nonrec recruitment = fragments_native_state38_campaign_member1_scripts_member1_recruitment
  type nonrec failed = json_string
  type nonrec active = json_string
  type nonrec next_mark = json_number
  type nonrec pending = fragments_native_state38_campaign_member1_scripts_member1_pending_item
  type nonrec revealed = json_string
  type nonrec controller_case = fragments_factions7_item_controller_member0
  type nonrec cost = json_number
  type nonrec to_ = json_string
  type nonrec blade = json_number
  type nonrec pierce = json_number
  type nonrec impact = json_number
  type nonrec damage = json_number
  type nonrec spread = json_number
  type nonrec count = json_number
  type nonrec damage_type = fragments_native_state38_ground_items_item_item_attacks_item_damage_type
  type nonrec category = fragments_native_state38_ground_items_item_item_attacks_item_category
  type nonrec range = json_number
  type nonrec drain = json_number
  type nonrec amount = json_number
  type nonrec condition_case = fragments_rule11_condition_member0
  type nonrec predicate = json_number
  type nonrec turns = json_number
  type nonrec last_round = json_number
  type nonrec area = json_string
  type nonrec light_remembered = fragments_rule11_remember
  type nonrec unit_types = json_string
  type nonrec unit_type = json_string
  type nonrec choice = json_string
  type nonrec flag = json_string
  type nonrec conditions = fragments_light28_member1_when__member1_condition
end

module V31 = struct
  type nonrec t = save31
  type nonrec version = literal_31
  type nonrec checkpoint = save29_checkpoint
  type nonrec pending_enemy = json_boolean
  type nonrec setup = save35_setup
  type nonrec progress = save2_progress_member1
  type nonrec undo = json_unknown
  type nonrec seed = json_number
  type nonrec state = save29_checkpoint_state
  type nonrec first = save35_setup_first
  type nonrec enemy = save35_setup_enemy
  type nonrec entry = history1_entries_item
  type nonrec round = json_number
  type nonrec side = save35_setup_first
  type nonrec winner = fragments_native_state38_winner
  type nonrec armies = fragments_native_state38_armies_member1
  type nonrec condition_memory = json_string
  type nonrec map_id = json_string
  type nonrec map_revision = json_number
  type nonrec cols = json_number
  type nonrec rows = json_number
  type nonrec next_id = json_number
  type nonrec log = json_string
  type nonrec last_combat = fragments_native_state38_last_combat_member2
  type nonrec unit_ = save26_checkpoint_state_units_item
  type nonrec tile = save26_checkpoint_state_tiles_item
  type nonrec campaign = save29_checkpoint_state_campaign_member1
  type nonrec gold = fragments_native_state38_gold
  type nonrec faction = save26_checkpoint_state_factions_item
  type nonrec ground_items = fragments_native_state38_ground_items_item
  type nonrec combat_rng = json_number
  type nonrec policy = save35_setup_enemy_policy
  type nonrec blocks = save35_setup_enemy_blocks_member1
  type nonrec recruitment_limit = json_number
  type nonrec diplomacy = save35_setup_enemy_diplomacy
  type nonrec id = json_string
  type nonrec saved_at = json_number
  type nonrec run = json_string
  type nonrec mission = json_string
  type nonrec mission_start = json_boolean
  type nonrec run_id = json_string
  type nonrec branch_id = json_string
  type nonrec checkpoint_id = json_string
  type nonrec ordinal = json_number
  type nonrec kind = history1_entries_item_kind
  type nonrec parent_id = json_string
  type nonrec attacker_id = json_string
  type nonrec attack_id = json_string
  type nonrec sound = json_string
  type nonrec role = fragments_native_state38_units_item_role
  type nonrec items = fragments_native_state38_ground_items_item_item
  type nonrec ai_group = json_string
  type nonrec type_ = json_string
  type nonrec hp = json_number
  type nonrec moved = json_boolean
  type nonrec attacked = json_boolean
  type nonrec retreating = fragments_rule11_remember
  type nonrec rebuilding = fragments_rule11_remember
  type nonrec character = json_string
  type nonrec experience = json_number
  type nonrec owner = json_string
  type nonrec location = json_string
  type nonrec col = json_number
  type nonrec row = json_number
  type nonrec terrain = json_string
  type nonrec connection = fragments_tiles11_item_connections_member1_item
  type nonrec dead_ends = fragments_tiles11_item_connections_member1_item
  type nonrec walls = fragments_tiles11_item_connections_member1_item
  type nonrec floor = json_string
  type nonrec fog = fragments_light28_member1_when__member1
  type nonrec light = fragments_light28_member1
  type nonrec vigil = fragments_native_state38_campaign_member1_vigil_member1
  type nonrec refuge = fragments_native_state38_campaign_member1_refuge_member1
  type nonrec completed = json_boolean
  type nonrec mark = json_string
  type nonrec hold = fragments_native_state38_campaign_member1_holds_member1
  type nonrec recall = json_string
  type nonrec fallen_companions = json_string
  type nonrec carry_gold = json_number
  type nonrec dialogue = fragments_native_state38_campaign_member1_dialogue_item
  type nonrec dialogue_index = json_number
  type nonrec outcome = json_string
  type nonrec veteran = save26_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec story_veteran = save26_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec evacuated = save26_checkpoint_state_campaign_member1_evacuated_member1_item
  type nonrec scripts = save28_checkpoint_state_campaign_member1_scripts_member1
  type nonrec flags = fragments_native_state38_campaign_member1_flags
  type nonrec name = json_string
  type nonrec phase = save35_setup_first
  type nonrec controller = fragments_factions7_item_controller
  type nonrec palette = json_string
  type nonrec recruits = fragments_factions7_item_recruits_member1_item
  type nonrec relations = fragments_factions7_item_relations_member1
  type nonrec transfer = save26_checkpoint_state_factions_item_transfer_member1
  type nonrec item = fragments_native_state38_ground_items_item_item
  type nonrec recruiter = save35_setup_enemy_blocks_member1_recruiter
  type nonrec planner = save35_setup_enemy_blocks_member1_planner
  type nonrec armor = fragments_native_state38_ground_items_item_item_armor_member1
  type nonrec description = json_string
  type nonrec attacks = fragments_native_state38_ground_items_item_item_attacks_item
  type nonrec healing = fragments_native_state38_ground_items_item_item_healing_member1
  type nonrec condition = fragments_light28_member1_when__member1_condition
  type nonrec remember = fragments_rule11_remember
  type nonrec radius = json_number
  type nonrec color = fragments_light23_color_member1
  type nonrec fixture = fragments_rule11_remember
  type nonrec dx = json_number
  type nonrec dy = json_number
  type nonrec power = json_number
  type nonrec rule = fragments_light28_member1_when__member1
  type nonrec held_turns = json_number
  type nonrec last_counted_round = json_number
  type nonrec escalated = json_boolean
  type nonrec speaker = json_string
  type nonrec portrait = json_string
  type nonrec text = json_string
  type nonrec event = json_string
  type nonrec recruitment = fragments_native_state38_campaign_member1_scripts_member1_recruitment
  type nonrec failed = json_string
  type nonrec active = json_string
  type nonrec next_mark = json_number
  type nonrec pending = fragments_native_state38_campaign_member1_scripts_member1_pending_item
  type nonrec revealed = json_string
  type nonrec controller_case = fragments_factions7_item_controller_member0
  type nonrec cost = json_number
  type nonrec to_ = json_string
  type nonrec blade = json_number
  type nonrec pierce = json_number
  type nonrec impact = json_number
  type nonrec damage = json_number
  type nonrec spread = json_number
  type nonrec count = json_number
  type nonrec damage_type = fragments_native_state38_ground_items_item_item_attacks_item_damage_type
  type nonrec category = fragments_native_state38_ground_items_item_item_attacks_item_category
  type nonrec range = json_number
  type nonrec drain = json_number
  type nonrec amount = json_number
  type nonrec condition_case = fragments_rule11_condition_member0
  type nonrec predicate = json_number
  type nonrec turns = json_number
  type nonrec last_round = json_number
  type nonrec area = json_string
  type nonrec light_remembered = fragments_rule11_remember
  type nonrec unit_types = json_string
  type nonrec unit_type = json_string
  type nonrec choice = json_string
  type nonrec flag = json_string
  type nonrec conditions = fragments_light28_member1_when__member1_condition
end

module V32 = struct
  type nonrec t = save32
  type nonrec version = literal_32
  type nonrec checkpoint = save29_checkpoint
  type nonrec pending_enemy = json_boolean
  type nonrec setup = save35_setup
  type nonrec progress = save2_progress_member1
  type nonrec undo = json_unknown
  type nonrec seed = json_number
  type nonrec state = save29_checkpoint_state
  type nonrec first = save35_setup_first
  type nonrec enemy = save35_setup_enemy
  type nonrec entry = history1_entries_item
  type nonrec round = json_number
  type nonrec side = save35_setup_first
  type nonrec winner = fragments_native_state38_winner
  type nonrec armies = fragments_native_state38_armies_member1
  type nonrec condition_memory = json_string
  type nonrec map_id = json_string
  type nonrec map_revision = json_number
  type nonrec cols = json_number
  type nonrec rows = json_number
  type nonrec next_id = json_number
  type nonrec log = json_string
  type nonrec last_combat = fragments_native_state38_last_combat_member2
  type nonrec unit_ = save26_checkpoint_state_units_item
  type nonrec tile = save26_checkpoint_state_tiles_item
  type nonrec campaign = save29_checkpoint_state_campaign_member1
  type nonrec gold = fragments_native_state38_gold
  type nonrec faction = save26_checkpoint_state_factions_item
  type nonrec ground_items = fragments_native_state38_ground_items_item
  type nonrec combat_rng = json_number
  type nonrec policy = save35_setup_enemy_policy
  type nonrec blocks = save35_setup_enemy_blocks_member1
  type nonrec recruitment_limit = json_number
  type nonrec diplomacy = save35_setup_enemy_diplomacy
  type nonrec id = json_string
  type nonrec saved_at = json_number
  type nonrec run = json_string
  type nonrec mission = json_string
  type nonrec mission_start = json_boolean
  type nonrec run_id = json_string
  type nonrec branch_id = json_string
  type nonrec checkpoint_id = json_string
  type nonrec ordinal = json_number
  type nonrec kind = history1_entries_item_kind
  type nonrec parent_id = json_string
  type nonrec attacker_id = json_string
  type nonrec attack_id = json_string
  type nonrec sound = json_string
  type nonrec role = fragments_native_state38_units_item_role
  type nonrec items = fragments_native_state38_ground_items_item_item
  type nonrec ai_group = json_string
  type nonrec type_ = json_string
  type nonrec hp = json_number
  type nonrec moved = json_boolean
  type nonrec attacked = json_boolean
  type nonrec retreating = fragments_rule11_remember
  type nonrec rebuilding = fragments_rule11_remember
  type nonrec character = json_string
  type nonrec experience = json_number
  type nonrec owner = json_string
  type nonrec location = json_string
  type nonrec col = json_number
  type nonrec row = json_number
  type nonrec terrain = json_string
  type nonrec connection = fragments_tiles11_item_connections_member1_item
  type nonrec dead_ends = fragments_tiles11_item_connections_member1_item
  type nonrec walls = fragments_tiles11_item_connections_member1_item
  type nonrec floor = json_string
  type nonrec fog = fragments_light28_member1_when__member1
  type nonrec light = fragments_light28_member1
  type nonrec vigil = fragments_native_state38_campaign_member1_vigil_member1
  type nonrec refuge = fragments_native_state38_campaign_member1_refuge_member1
  type nonrec completed = json_boolean
  type nonrec mark = json_string
  type nonrec hold = fragments_native_state38_campaign_member1_holds_member1
  type nonrec recall = json_string
  type nonrec fallen_companions = json_string
  type nonrec carry_gold = json_number
  type nonrec dialogue = fragments_native_state38_campaign_member1_dialogue_item
  type nonrec dialogue_index = json_number
  type nonrec outcome = json_string
  type nonrec veteran = save26_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec story_veteran = save26_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec evacuated = save26_checkpoint_state_campaign_member1_evacuated_member1_item
  type nonrec scripts = save28_checkpoint_state_campaign_member1_scripts_member1
  type nonrec flags = fragments_native_state38_campaign_member1_flags
  type nonrec name = json_string
  type nonrec phase = save35_setup_first
  type nonrec controller = fragments_factions7_item_controller
  type nonrec palette = json_string
  type nonrec recruits = fragments_factions7_item_recruits_member1_item
  type nonrec relations = fragments_factions7_item_relations_member1
  type nonrec transfer = save26_checkpoint_state_factions_item_transfer_member1
  type nonrec item = fragments_native_state38_ground_items_item_item
  type nonrec recruiter = save35_setup_enemy_blocks_member1_recruiter
  type nonrec planner = save35_setup_enemy_blocks_member1_planner
  type nonrec armor = fragments_native_state38_ground_items_item_item_armor_member1
  type nonrec description = json_string
  type nonrec attacks = fragments_native_state38_ground_items_item_item_attacks_item
  type nonrec healing = fragments_native_state38_ground_items_item_item_healing_member1
  type nonrec condition = fragments_light28_member1_when__member1_condition
  type nonrec remember = fragments_rule11_remember
  type nonrec radius = json_number
  type nonrec color = fragments_light23_color_member1
  type nonrec fixture = fragments_rule11_remember
  type nonrec dx = json_number
  type nonrec dy = json_number
  type nonrec power = json_number
  type nonrec rule = fragments_light28_member1_when__member1
  type nonrec held_turns = json_number
  type nonrec last_counted_round = json_number
  type nonrec escalated = json_boolean
  type nonrec speaker = json_string
  type nonrec portrait = json_string
  type nonrec text = json_string
  type nonrec event = json_string
  type nonrec recruitment = fragments_native_state38_campaign_member1_scripts_member1_recruitment
  type nonrec failed = json_string
  type nonrec active = json_string
  type nonrec next_mark = json_number
  type nonrec pending = fragments_native_state38_campaign_member1_scripts_member1_pending_item
  type nonrec revealed = json_string
  type nonrec controller_case = fragments_factions7_item_controller_member0
  type nonrec cost = json_number
  type nonrec to_ = json_string
  type nonrec blade = json_number
  type nonrec pierce = json_number
  type nonrec impact = json_number
  type nonrec damage = json_number
  type nonrec spread = json_number
  type nonrec count = json_number
  type nonrec damage_type = fragments_native_state38_ground_items_item_item_attacks_item_damage_type
  type nonrec category = fragments_native_state38_ground_items_item_item_attacks_item_category
  type nonrec range = json_number
  type nonrec drain = json_number
  type nonrec amount = json_number
  type nonrec condition_case = fragments_rule11_condition_member0
  type nonrec predicate = json_number
  type nonrec turns = json_number
  type nonrec last_round = json_number
  type nonrec area = json_string
  type nonrec light_remembered = fragments_rule11_remember
  type nonrec unit_types = json_string
  type nonrec unit_type = json_string
  type nonrec choice = json_string
  type nonrec flag = json_string
  type nonrec conditions = fragments_light28_member1_when__member1_condition
end

module V33 = struct
  type nonrec t = save33
  type nonrec version = literal_33
  type nonrec checkpoint = save29_checkpoint
  type nonrec pending_enemy = json_boolean
  type nonrec setup = save35_setup
  type nonrec progress = save2_progress_member1
  type nonrec undo = json_unknown
  type nonrec seed = json_number
  type nonrec state = save29_checkpoint_state
  type nonrec first = save35_setup_first
  type nonrec enemy = save35_setup_enemy
  type nonrec entry = history1_entries_item
  type nonrec round = json_number
  type nonrec side = save35_setup_first
  type nonrec winner = fragments_native_state38_winner
  type nonrec armies = fragments_native_state38_armies_member1
  type nonrec condition_memory = json_string
  type nonrec map_id = json_string
  type nonrec map_revision = json_number
  type nonrec cols = json_number
  type nonrec rows = json_number
  type nonrec next_id = json_number
  type nonrec log = json_string
  type nonrec last_combat = fragments_native_state38_last_combat_member2
  type nonrec unit_ = save26_checkpoint_state_units_item
  type nonrec tile = save26_checkpoint_state_tiles_item
  type nonrec campaign = save29_checkpoint_state_campaign_member1
  type nonrec gold = fragments_native_state38_gold
  type nonrec faction = save26_checkpoint_state_factions_item
  type nonrec ground_items = fragments_native_state38_ground_items_item
  type nonrec combat_rng = json_number
  type nonrec policy = save35_setup_enemy_policy
  type nonrec blocks = save35_setup_enemy_blocks_member1
  type nonrec recruitment_limit = json_number
  type nonrec diplomacy = save35_setup_enemy_diplomacy
  type nonrec id = json_string
  type nonrec saved_at = json_number
  type nonrec run = json_string
  type nonrec mission = json_string
  type nonrec mission_start = json_boolean
  type nonrec run_id = json_string
  type nonrec branch_id = json_string
  type nonrec checkpoint_id = json_string
  type nonrec ordinal = json_number
  type nonrec kind = history1_entries_item_kind
  type nonrec parent_id = json_string
  type nonrec attacker_id = json_string
  type nonrec attack_id = json_string
  type nonrec sound = json_string
  type nonrec role = fragments_native_state38_units_item_role
  type nonrec items = fragments_native_state38_ground_items_item_item
  type nonrec ai_group = json_string
  type nonrec type_ = json_string
  type nonrec hp = json_number
  type nonrec moved = json_boolean
  type nonrec attacked = json_boolean
  type nonrec retreating = fragments_rule11_remember
  type nonrec rebuilding = fragments_rule11_remember
  type nonrec character = json_string
  type nonrec experience = json_number
  type nonrec owner = json_string
  type nonrec location = json_string
  type nonrec col = json_number
  type nonrec row = json_number
  type nonrec terrain = json_string
  type nonrec connection = fragments_tiles11_item_connections_member1_item
  type nonrec dead_ends = fragments_tiles11_item_connections_member1_item
  type nonrec walls = fragments_tiles11_item_connections_member1_item
  type nonrec floor = json_string
  type nonrec fog = fragments_light28_member1_when__member1
  type nonrec light = fragments_light28_member1
  type nonrec vigil = fragments_native_state38_campaign_member1_vigil_member1
  type nonrec refuge = fragments_native_state38_campaign_member1_refuge_member1
  type nonrec completed = json_boolean
  type nonrec mark = json_string
  type nonrec hold = fragments_native_state38_campaign_member1_holds_member1
  type nonrec recall = json_string
  type nonrec fallen_companions = json_string
  type nonrec carry_gold = json_number
  type nonrec dialogue = fragments_native_state38_campaign_member1_dialogue_item
  type nonrec dialogue_index = json_number
  type nonrec outcome = json_string
  type nonrec veteran = save26_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec story_veteran = save26_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec evacuated = save26_checkpoint_state_campaign_member1_evacuated_member1_item
  type nonrec scripts = save28_checkpoint_state_campaign_member1_scripts_member1
  type nonrec flags = fragments_native_state38_campaign_member1_flags
  type nonrec name = json_string
  type nonrec phase = save35_setup_first
  type nonrec controller = fragments_factions7_item_controller
  type nonrec palette = json_string
  type nonrec recruits = fragments_factions7_item_recruits_member1_item
  type nonrec relations = fragments_factions7_item_relations_member1
  type nonrec transfer = save26_checkpoint_state_factions_item_transfer_member1
  type nonrec item = fragments_native_state38_ground_items_item_item
  type nonrec recruiter = save35_setup_enemy_blocks_member1_recruiter
  type nonrec planner = save35_setup_enemy_blocks_member1_planner
  type nonrec armor = fragments_native_state38_ground_items_item_item_armor_member1
  type nonrec description = json_string
  type nonrec attacks = fragments_native_state38_ground_items_item_item_attacks_item
  type nonrec healing = fragments_native_state38_ground_items_item_item_healing_member1
  type nonrec condition = fragments_light28_member1_when__member1_condition
  type nonrec remember = fragments_rule11_remember
  type nonrec radius = json_number
  type nonrec color = fragments_light23_color_member1
  type nonrec fixture = fragments_rule11_remember
  type nonrec dx = json_number
  type nonrec dy = json_number
  type nonrec power = json_number
  type nonrec rule = fragments_light28_member1_when__member1
  type nonrec held_turns = json_number
  type nonrec last_counted_round = json_number
  type nonrec escalated = json_boolean
  type nonrec speaker = json_string
  type nonrec portrait = json_string
  type nonrec text = json_string
  type nonrec event = json_string
  type nonrec recruitment = fragments_native_state38_campaign_member1_scripts_member1_recruitment
  type nonrec failed = json_string
  type nonrec active = json_string
  type nonrec next_mark = json_number
  type nonrec pending = fragments_native_state38_campaign_member1_scripts_member1_pending_item
  type nonrec revealed = json_string
  type nonrec controller_case = fragments_factions7_item_controller_member0
  type nonrec cost = json_number
  type nonrec to_ = json_string
  type nonrec blade = json_number
  type nonrec pierce = json_number
  type nonrec impact = json_number
  type nonrec damage = json_number
  type nonrec spread = json_number
  type nonrec count = json_number
  type nonrec damage_type = fragments_native_state38_ground_items_item_item_attacks_item_damage_type
  type nonrec category = fragments_native_state38_ground_items_item_item_attacks_item_category
  type nonrec range = json_number
  type nonrec drain = json_number
  type nonrec amount = json_number
  type nonrec condition_case = fragments_rule11_condition_member0
  type nonrec predicate = json_number
  type nonrec turns = json_number
  type nonrec last_round = json_number
  type nonrec area = json_string
  type nonrec light_remembered = fragments_rule11_remember
  type nonrec unit_types = json_string
  type nonrec unit_type = json_string
  type nonrec choice = json_string
  type nonrec flag = json_string
  type nonrec conditions = fragments_light28_member1_when__member1_condition
end

module V34 = struct
  type nonrec t = save34
  type nonrec version = literal_34
  type nonrec checkpoint = save34_checkpoint
  type nonrec pending_enemy = json_boolean
  type nonrec setup = save35_setup
  type nonrec progress = save2_progress_member1
  type nonrec undo = save34_undo_member1_item
  type nonrec seed = json_number
  type nonrec state = save34_checkpoint_state
  type nonrec first = save35_setup_first
  type nonrec enemy = save35_setup_enemy
  type nonrec entry = history1_entries_item
  type nonrec round = json_number
  type nonrec unit_id = json_string
  type nonrec changes = save34_undo_member1_item_changes_item
  type nonrec side = save35_setup_first
  type nonrec winner = fragments_native_state38_winner
  type nonrec armies = fragments_native_state38_armies_member1
  type nonrec condition_memory = json_string
  type nonrec map_id = json_string
  type nonrec map_revision = json_number
  type nonrec cols = json_number
  type nonrec rows = json_number
  type nonrec next_id = json_number
  type nonrec log = json_string
  type nonrec last_combat = fragments_native_state38_last_combat_member2
  type nonrec unit_ = save26_checkpoint_state_units_item
  type nonrec tile = save26_checkpoint_state_tiles_item
  type nonrec campaign = save34_checkpoint_state_campaign_member1
  type nonrec gold = fragments_native_state38_gold
  type nonrec faction = save26_checkpoint_state_factions_item
  type nonrec ground_items = fragments_native_state38_ground_items_item
  type nonrec combat_rng = json_number
  type nonrec policy = save35_setup_enemy_policy
  type nonrec blocks = save35_setup_enemy_blocks_member1
  type nonrec recruitment_limit = json_number
  type nonrec diplomacy = save35_setup_enemy_diplomacy
  type nonrec id = json_string
  type nonrec saved_at = json_number
  type nonrec run = json_string
  type nonrec mission = json_string
  type nonrec mission_start = json_boolean
  type nonrec run_id = json_string
  type nonrec branch_id = json_string
  type nonrec checkpoint_id = json_string
  type nonrec ordinal = json_number
  type nonrec kind = history1_entries_item_kind
  type nonrec parent_id = json_string
  type nonrec changes_case = save34_undo_member1_item_changes_item_member0
  type nonrec attacker_id = json_string
  type nonrec attack_id = json_string
  type nonrec sound = json_string
  type nonrec role = fragments_native_state38_units_item_role
  type nonrec items = fragments_native_state38_ground_items_item_item
  type nonrec ai_group = json_string
  type nonrec type_ = json_string
  type nonrec hp = json_number
  type nonrec moved = json_boolean
  type nonrec attacked = json_boolean
  type nonrec retreating = fragments_rule11_remember
  type nonrec rebuilding = fragments_rule11_remember
  type nonrec character = json_string
  type nonrec experience = json_number
  type nonrec owner = json_string
  type nonrec location = json_string
  type nonrec col = json_number
  type nonrec row = json_number
  type nonrec terrain = json_string
  type nonrec connection = fragments_tiles11_item_connections_member1_item
  type nonrec dead_ends = fragments_tiles11_item_connections_member1_item
  type nonrec walls = fragments_tiles11_item_connections_member1_item
  type nonrec floor = json_string
  type nonrec fog = fragments_light28_member1_when__member1
  type nonrec light = fragments_light28_member1
  type nonrec vigil = fragments_native_state38_campaign_member1_vigil_member1
  type nonrec refuge = fragments_native_state38_campaign_member1_refuge_member1
  type nonrec completed = json_boolean
  type nonrec mark = json_string
  type nonrec hold = fragments_native_state38_campaign_member1_holds_member1
  type nonrec recall = json_string
  type nonrec fallen_companions = json_string
  type nonrec carry_gold = json_number
  type nonrec dialogue = fragments_native_state38_campaign_member1_dialogue_item
  type nonrec dialogue_index = json_number
  type nonrec outcome = json_string
  type nonrec veteran = save26_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec story_veteran = save26_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec evacuated = save26_checkpoint_state_campaign_member1_evacuated_member1_item
  type nonrec scripts = save34_checkpoint_state_campaign_member1_scripts_member1
  type nonrec flags = fragments_native_state38_campaign_member1_flags
  type nonrec name = json_string
  type nonrec phase = save35_setup_first
  type nonrec controller = fragments_factions7_item_controller
  type nonrec palette = json_string
  type nonrec recruits = fragments_factions7_item_recruits_member1_item
  type nonrec relations = fragments_factions7_item_relations_member1
  type nonrec transfer = save26_checkpoint_state_factions_item_transfer_member1
  type nonrec item = fragments_native_state38_ground_items_item_item
  type nonrec recruiter = save35_setup_enemy_blocks_member1_recruiter
  type nonrec planner = save35_setup_enemy_blocks_member1_planner
  type nonrec op = literal_set
  type nonrec path = json_string
  type nonrec value = json_unknown
  type nonrec armor = fragments_native_state38_ground_items_item_item_armor_member1
  type nonrec description = json_string
  type nonrec attacks = fragments_native_state38_ground_items_item_item_attacks_item
  type nonrec healing = fragments_native_state38_ground_items_item_item_healing_member1
  type nonrec condition = fragments_light28_member1_when__member1_condition
  type nonrec remember = fragments_rule11_remember
  type nonrec radius = json_number
  type nonrec color = fragments_light23_color_member1
  type nonrec fixture = fragments_rule11_remember
  type nonrec dx = json_number
  type nonrec dy = json_number
  type nonrec power = json_number
  type nonrec rule = fragments_light28_member1_when__member1
  type nonrec held_turns = json_number
  type nonrec last_counted_round = json_number
  type nonrec escalated = json_boolean
  type nonrec speaker = json_string
  type nonrec portrait = json_string
  type nonrec text = json_string
  type nonrec event = json_string
  type nonrec recruitment = fragments_native_state38_campaign_member1_scripts_member1_recruitment
  type nonrec failed = json_string
  type nonrec active = json_string
  type nonrec next_mark = json_number
  type nonrec pending = fragments_native_state38_campaign_member1_scripts_member1_pending_item
  type nonrec revealed = json_string
  type nonrec controller_case = fragments_factions7_item_controller_member0
  type nonrec cost = json_number
  type nonrec to_ = json_string
  type nonrec blade = json_number
  type nonrec pierce = json_number
  type nonrec impact = json_number
  type nonrec damage = json_number
  type nonrec spread = json_number
  type nonrec count = json_number
  type nonrec damage_type = fragments_native_state38_ground_items_item_item_attacks_item_damage_type
  type nonrec category = fragments_native_state38_ground_items_item_item_attacks_item_category
  type nonrec range = json_number
  type nonrec drain = json_number
  type nonrec amount = json_number
  type nonrec condition_case = fragments_rule11_condition_member0
  type nonrec predicate = json_number
  type nonrec turns = json_number
  type nonrec last_round = json_number
  type nonrec area = json_string
  type nonrec light_remembered = fragments_rule11_remember
  type nonrec unit_types = json_string
  type nonrec unit_type = json_string
  type nonrec choice = json_string
  type nonrec flag = json_string
  type nonrec conditions = fragments_light28_member1_when__member1_condition
end

module V35 = struct
  type nonrec t = save35
  type nonrec version = literal_35
  type nonrec checkpoint = save35_checkpoint
  type nonrec pending_enemy = json_boolean
  type nonrec setup = save35_setup
  type nonrec progress = save35_progress
  type nonrec undo = save34_undo_member1_item
  type nonrec seed = json_number
  type nonrec state = save35_checkpoint_state
  type nonrec first = save35_setup_first
  type nonrec enemy = save35_setup_enemy
  type nonrec entry = save35_progress_entry
  type nonrec round = json_number
  type nonrec unit_id = json_string
  type nonrec changes = save34_undo_member1_item_changes_item
  type nonrec side = save35_setup_first
  type nonrec winner = fragments_native_state38_winner
  type nonrec armies = fragments_native_state38_armies_member1
  type nonrec condition_memory = json_string
  type nonrec map_id = json_string
  type nonrec map_revision = json_number
  type nonrec cols = json_number
  type nonrec rows = json_number
  type nonrec next_id = json_number
  type nonrec log = json_string
  type nonrec last_combat = fragments_native_state38_last_combat_member2
  type nonrec unit_ = save26_checkpoint_state_units_item
  type nonrec tile = save26_checkpoint_state_tiles_item
  type nonrec campaign = save34_checkpoint_state_campaign_member1
  type nonrec gold = fragments_native_state38_gold
  type nonrec faction = save26_checkpoint_state_factions_item
  type nonrec ground_items = fragments_native_state38_ground_items_item
  type nonrec combat_rng = json_number
  type nonrec policy = save35_setup_enemy_policy
  type nonrec blocks = save35_setup_enemy_blocks_member1
  type nonrec recruitment_limit = json_number
  type nonrec diplomacy = save35_setup_enemy_diplomacy
  type nonrec id = json_string
  type nonrec saved_at = json_number
  type nonrec run = json_string
  type nonrec mission = json_string
  type nonrec mission_start = json_boolean
  type nonrec run_id = json_string
  type nonrec branch_id = json_string
  type nonrec checkpoint_id = json_string
  type nonrec ordinal = json_number
  type nonrec kind = save35_progress_entry_kind
  type nonrec parent_id = json_string
  type nonrec changes_case = save34_undo_member1_item_changes_item_member0
  type nonrec attacker_id = json_string
  type nonrec attack_id = json_string
  type nonrec sound = json_string
  type nonrec role = fragments_native_state38_units_item_role
  type nonrec items = fragments_native_state38_ground_items_item_item
  type nonrec ai_group = json_string
  type nonrec type_ = json_string
  type nonrec hp = json_number
  type nonrec moved = json_boolean
  type nonrec attacked = json_boolean
  type nonrec retreating = fragments_rule11_remember
  type nonrec rebuilding = fragments_rule11_remember
  type nonrec character = json_string
  type nonrec experience = json_number
  type nonrec owner = json_string
  type nonrec location = json_string
  type nonrec col = json_number
  type nonrec row = json_number
  type nonrec terrain = json_string
  type nonrec connection = fragments_tiles11_item_connections_member1_item
  type nonrec dead_ends = fragments_tiles11_item_connections_member1_item
  type nonrec walls = fragments_tiles11_item_connections_member1_item
  type nonrec floor = json_string
  type nonrec fog = fragments_light28_member1_when__member1
  type nonrec light = fragments_light28_member1
  type nonrec vigil = fragments_native_state38_campaign_member1_vigil_member1
  type nonrec refuge = fragments_native_state38_campaign_member1_refuge_member1
  type nonrec completed = json_boolean
  type nonrec mark = json_string
  type nonrec hold = fragments_native_state38_campaign_member1_holds_member1
  type nonrec recall = json_string
  type nonrec fallen_companions = json_string
  type nonrec carry_gold = json_number
  type nonrec dialogue = fragments_native_state38_campaign_member1_dialogue_item
  type nonrec dialogue_index = json_number
  type nonrec outcome = json_string
  type nonrec veteran = save26_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec story_veteran = save26_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec evacuated = save26_checkpoint_state_campaign_member1_evacuated_member1_item
  type nonrec scripts = save34_checkpoint_state_campaign_member1_scripts_member1
  type nonrec flags = fragments_native_state38_campaign_member1_flags
  type nonrec name = json_string
  type nonrec phase = save35_setup_first
  type nonrec controller = fragments_factions7_item_controller
  type nonrec palette = json_string
  type nonrec recruits = fragments_factions7_item_recruits_member1_item
  type nonrec relations = fragments_factions7_item_relations_member1
  type nonrec transfer = save26_checkpoint_state_factions_item_transfer_member1
  type nonrec item = fragments_native_state38_ground_items_item_item
  type nonrec recruiter = save35_setup_enemy_blocks_member1_recruiter
  type nonrec planner = save35_setup_enemy_blocks_member1_planner
  type nonrec op = literal_set
  type nonrec path = json_string
  type nonrec value = json_unknown
  type nonrec armor = fragments_native_state38_ground_items_item_item_armor_member1
  type nonrec description = json_string
  type nonrec attacks = fragments_native_state38_ground_items_item_item_attacks_item
  type nonrec healing = fragments_native_state38_ground_items_item_item_healing_member1
  type nonrec condition = fragments_light28_member1_when__member1_condition
  type nonrec remember = fragments_rule11_remember
  type nonrec radius = json_number
  type nonrec color = fragments_light23_color_member1
  type nonrec fixture = fragments_rule11_remember
  type nonrec dx = json_number
  type nonrec dy = json_number
  type nonrec power = json_number
  type nonrec rule = fragments_light28_member1_when__member1
  type nonrec held_turns = json_number
  type nonrec last_counted_round = json_number
  type nonrec escalated = json_boolean
  type nonrec speaker = json_string
  type nonrec portrait = json_string
  type nonrec text = json_string
  type nonrec event = json_string
  type nonrec recruitment = fragments_native_state38_campaign_member1_scripts_member1_recruitment
  type nonrec failed = json_string
  type nonrec active = json_string
  type nonrec next_mark = json_number
  type nonrec pending = fragments_native_state38_campaign_member1_scripts_member1_pending_item
  type nonrec revealed = json_string
  type nonrec controller_case = fragments_factions7_item_controller_member0
  type nonrec cost = json_number
  type nonrec to_ = json_string
  type nonrec blade = json_number
  type nonrec pierce = json_number
  type nonrec impact = json_number
  type nonrec damage = json_number
  type nonrec spread = json_number
  type nonrec count = json_number
  type nonrec damage_type = fragments_native_state38_ground_items_item_item_attacks_item_damage_type
  type nonrec category = fragments_native_state38_ground_items_item_item_attacks_item_category
  type nonrec range = json_number
  type nonrec drain = json_number
  type nonrec amount = json_number
  type nonrec condition_case = fragments_rule11_condition_member0
  type nonrec predicate = json_number
  type nonrec turns = json_number
  type nonrec last_round = json_number
  type nonrec area = json_string
  type nonrec light_remembered = fragments_rule11_remember
  type nonrec unit_types = json_string
  type nonrec unit_type = json_string
  type nonrec choice = json_string
  type nonrec flag = json_string
  type nonrec conditions = fragments_light28_member1_when__member1_condition
end

module V36 = struct
  type nonrec t = save36
  type nonrec version = literal_36
  type nonrec checkpoint = save35_checkpoint
  type nonrec pending_enemy = json_boolean
  type nonrec setup = save35_setup
  type nonrec progress = save35_progress
  type nonrec undo = save34_undo_member1_item
  type nonrec seed = json_number
  type nonrec state = save35_checkpoint_state
  type nonrec first = save35_setup_first
  type nonrec enemy = save35_setup_enemy
  type nonrec entry = save35_progress_entry
  type nonrec round = json_number
  type nonrec unit_id = json_string
  type nonrec changes = save34_undo_member1_item_changes_item
  type nonrec side = save35_setup_first
  type nonrec winner = fragments_native_state38_winner
  type nonrec armies = fragments_native_state38_armies_member1
  type nonrec condition_memory = json_string
  type nonrec map_id = json_string
  type nonrec map_revision = json_number
  type nonrec cols = json_number
  type nonrec rows = json_number
  type nonrec next_id = json_number
  type nonrec log = json_string
  type nonrec last_combat = fragments_native_state38_last_combat_member2
  type nonrec unit_ = save26_checkpoint_state_units_item
  type nonrec tile = save26_checkpoint_state_tiles_item
  type nonrec campaign = save34_checkpoint_state_campaign_member1
  type nonrec gold = fragments_native_state38_gold
  type nonrec faction = save26_checkpoint_state_factions_item
  type nonrec ground_items = fragments_native_state38_ground_items_item
  type nonrec combat_rng = json_number
  type nonrec policy = save35_setup_enemy_policy
  type nonrec blocks = save35_setup_enemy_blocks_member1
  type nonrec recruitment_limit = json_number
  type nonrec diplomacy = save35_setup_enemy_diplomacy
  type nonrec id = json_string
  type nonrec saved_at = json_number
  type nonrec run = json_string
  type nonrec mission = json_string
  type nonrec mission_start = json_boolean
  type nonrec run_id = json_string
  type nonrec branch_id = json_string
  type nonrec checkpoint_id = json_string
  type nonrec ordinal = json_number
  type nonrec kind = save35_progress_entry_kind
  type nonrec parent_id = json_string
  type nonrec changes_case = save34_undo_member1_item_changes_item_member0
  type nonrec attacker_id = json_string
  type nonrec attack_id = json_string
  type nonrec sound = json_string
  type nonrec role = fragments_native_state38_units_item_role
  type nonrec items = fragments_native_state38_ground_items_item_item
  type nonrec ai_group = json_string
  type nonrec type_ = json_string
  type nonrec hp = json_number
  type nonrec moved = json_boolean
  type nonrec attacked = json_boolean
  type nonrec retreating = fragments_rule11_remember
  type nonrec rebuilding = fragments_rule11_remember
  type nonrec character = json_string
  type nonrec experience = json_number
  type nonrec owner = json_string
  type nonrec location = json_string
  type nonrec col = json_number
  type nonrec row = json_number
  type nonrec terrain = json_string
  type nonrec connection = fragments_tiles11_item_connections_member1_item
  type nonrec dead_ends = fragments_tiles11_item_connections_member1_item
  type nonrec walls = fragments_tiles11_item_connections_member1_item
  type nonrec floor = json_string
  type nonrec fog = fragments_light28_member1_when__member1
  type nonrec light = fragments_light28_member1
  type nonrec vigil = fragments_native_state38_campaign_member1_vigil_member1
  type nonrec refuge = fragments_native_state38_campaign_member1_refuge_member1
  type nonrec completed = json_boolean
  type nonrec mark = json_string
  type nonrec hold = fragments_native_state38_campaign_member1_holds_member1
  type nonrec recall = json_string
  type nonrec fallen_companions = json_string
  type nonrec carry_gold = json_number
  type nonrec dialogue = fragments_native_state38_campaign_member1_dialogue_item
  type nonrec dialogue_index = json_number
  type nonrec outcome = json_string
  type nonrec veteran = save26_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec story_veteran = save26_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec evacuated = save26_checkpoint_state_campaign_member1_evacuated_member1_item
  type nonrec scripts = save34_checkpoint_state_campaign_member1_scripts_member1
  type nonrec flags = fragments_native_state38_campaign_member1_flags
  type nonrec name = json_string
  type nonrec phase = save35_setup_first
  type nonrec controller = fragments_factions7_item_controller
  type nonrec palette = json_string
  type nonrec recruits = fragments_factions7_item_recruits_member1_item
  type nonrec relations = fragments_factions7_item_relations_member1
  type nonrec transfer = save26_checkpoint_state_factions_item_transfer_member1
  type nonrec item = fragments_native_state38_ground_items_item_item
  type nonrec recruiter = save35_setup_enemy_blocks_member1_recruiter
  type nonrec planner = save35_setup_enemy_blocks_member1_planner
  type nonrec op = literal_set
  type nonrec path = json_string
  type nonrec value = json_unknown
  type nonrec armor = fragments_native_state38_ground_items_item_item_armor_member1
  type nonrec description = json_string
  type nonrec attacks = fragments_native_state38_ground_items_item_item_attacks_item
  type nonrec healing = fragments_native_state38_ground_items_item_item_healing_member1
  type nonrec condition = fragments_light28_member1_when__member1_condition
  type nonrec remember = fragments_rule11_remember
  type nonrec radius = json_number
  type nonrec color = fragments_light23_color_member1
  type nonrec fixture = fragments_rule11_remember
  type nonrec dx = json_number
  type nonrec dy = json_number
  type nonrec power = json_number
  type nonrec rule = fragments_light28_member1_when__member1
  type nonrec held_turns = json_number
  type nonrec last_counted_round = json_number
  type nonrec escalated = json_boolean
  type nonrec speaker = json_string
  type nonrec portrait = json_string
  type nonrec text = json_string
  type nonrec event = json_string
  type nonrec recruitment = fragments_native_state38_campaign_member1_scripts_member1_recruitment
  type nonrec failed = json_string
  type nonrec active = json_string
  type nonrec next_mark = json_number
  type nonrec pending = fragments_native_state38_campaign_member1_scripts_member1_pending_item
  type nonrec revealed = json_string
  type nonrec controller_case = fragments_factions7_item_controller_member0
  type nonrec cost = json_number
  type nonrec to_ = json_string
  type nonrec blade = json_number
  type nonrec pierce = json_number
  type nonrec impact = json_number
  type nonrec damage = json_number
  type nonrec spread = json_number
  type nonrec count = json_number
  type nonrec damage_type = fragments_native_state38_ground_items_item_item_attacks_item_damage_type
  type nonrec category = fragments_native_state38_ground_items_item_item_attacks_item_category
  type nonrec range = json_number
  type nonrec drain = json_number
  type nonrec amount = json_number
  type nonrec condition_case = fragments_rule11_condition_member0
  type nonrec predicate = json_number
  type nonrec turns = json_number
  type nonrec last_round = json_number
  type nonrec area = json_string
  type nonrec light_remembered = fragments_rule11_remember
  type nonrec unit_types = json_string
  type nonrec unit_type = json_string
  type nonrec choice = json_string
  type nonrec flag = json_string
  type nonrec conditions = fragments_light28_member1_when__member1_condition
end

module V37 = struct
  type nonrec t = save37
  type nonrec version = literal_37
  type nonrec checkpoint = save35_checkpoint
  type nonrec pending_enemy = json_boolean
  type nonrec setup = save35_setup
  type nonrec progress = save37_progress
  type nonrec undo = save34_undo_member1_item
  type nonrec seed = json_number
  type nonrec state = save35_checkpoint_state
  type nonrec first = save35_setup_first
  type nonrec enemy = save35_setup_enemy
  type nonrec entry = save37_progress_entry
  type nonrec round = json_number
  type nonrec unit_id = json_string
  type nonrec changes = save34_undo_member1_item_changes_item
  type nonrec side = save35_setup_first
  type nonrec winner = fragments_native_state38_winner
  type nonrec armies = fragments_native_state38_armies_member1
  type nonrec condition_memory = json_string
  type nonrec map_id = json_string
  type nonrec map_revision = json_number
  type nonrec cols = json_number
  type nonrec rows = json_number
  type nonrec next_id = json_number
  type nonrec log = json_string
  type nonrec last_combat = fragments_native_state38_last_combat_member2
  type nonrec unit_ = save26_checkpoint_state_units_item
  type nonrec tile = save26_checkpoint_state_tiles_item
  type nonrec campaign = save34_checkpoint_state_campaign_member1
  type nonrec gold = fragments_native_state38_gold
  type nonrec faction = save26_checkpoint_state_factions_item
  type nonrec ground_items = fragments_native_state38_ground_items_item
  type nonrec combat_rng = json_number
  type nonrec policy = save35_setup_enemy_policy
  type nonrec blocks = save35_setup_enemy_blocks_member1
  type nonrec recruitment_limit = json_number
  type nonrec diplomacy = save35_setup_enemy_diplomacy
  type nonrec entry_case = save37_progress_entry_member0
  type nonrec changes_case = save34_undo_member1_item_changes_item_member0
  type nonrec attacker_id = json_string
  type nonrec attack_id = json_string
  type nonrec sound = json_string
  type nonrec id = json_string
  type nonrec role = fragments_native_state38_units_item_role
  type nonrec items = fragments_native_state38_ground_items_item_item
  type nonrec ai_group = json_string
  type nonrec type_ = json_string
  type nonrec hp = json_number
  type nonrec moved = json_boolean
  type nonrec attacked = json_boolean
  type nonrec retreating = fragments_rule11_remember
  type nonrec rebuilding = fragments_rule11_remember
  type nonrec character = json_string
  type nonrec experience = json_number
  type nonrec owner = json_string
  type nonrec location = json_string
  type nonrec col = json_number
  type nonrec row = json_number
  type nonrec terrain = json_string
  type nonrec connection = fragments_tiles11_item_connections_member1_item
  type nonrec dead_ends = fragments_tiles11_item_connections_member1_item
  type nonrec walls = fragments_tiles11_item_connections_member1_item
  type nonrec floor = json_string
  type nonrec fog = fragments_light28_member1_when__member1
  type nonrec light = fragments_light28_member1
  type nonrec mission = json_number
  type nonrec vigil = fragments_native_state38_campaign_member1_vigil_member1
  type nonrec refuge = fragments_native_state38_campaign_member1_refuge_member1
  type nonrec completed = json_boolean
  type nonrec mark = json_string
  type nonrec hold = fragments_native_state38_campaign_member1_holds_member1
  type nonrec recall = json_string
  type nonrec fallen_companions = json_string
  type nonrec carry_gold = json_number
  type nonrec dialogue = fragments_native_state38_campaign_member1_dialogue_item
  type nonrec dialogue_index = json_number
  type nonrec outcome = json_string
  type nonrec veteran = save26_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec story_veteran = save26_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec evacuated = save26_checkpoint_state_campaign_member1_evacuated_member1_item
  type nonrec scripts = save34_checkpoint_state_campaign_member1_scripts_member1
  type nonrec flags = fragments_native_state38_campaign_member1_flags
  type nonrec name = json_string
  type nonrec phase = save35_setup_first
  type nonrec controller = fragments_factions7_item_controller
  type nonrec palette = json_string
  type nonrec recruits = fragments_factions7_item_recruits_member1_item
  type nonrec relations = fragments_factions7_item_relations_member1
  type nonrec transfer = save26_checkpoint_state_factions_item_transfer_member1
  type nonrec item = fragments_native_state38_ground_items_item_item
  type nonrec recruiter = save35_setup_enemy_blocks_member1_recruiter
  type nonrec planner = save35_setup_enemy_blocks_member1_planner
  type nonrec kind = literal_flash
  type nonrec saved_at = json_number
  type nonrec run = json_string
  type nonrec mission_start = json_boolean
  type nonrec run_id = json_string
  type nonrec branch_id = json_string
  type nonrec checkpoint_id = json_string
  type nonrec ordinal = json_number
  type nonrec manual = json_boolean
  type nonrec parent_id = json_string
  type nonrec op = literal_set
  type nonrec path = json_string
  type nonrec value = json_unknown
  type nonrec armor = fragments_native_state38_ground_items_item_item_armor_member1
  type nonrec description = json_string
  type nonrec attacks = fragments_native_state38_ground_items_item_item_attacks_item
  type nonrec healing = fragments_native_state38_ground_items_item_item_healing_member1
  type nonrec condition = fragments_light28_member1_when__member1_condition
  type nonrec remember = fragments_rule11_remember
  type nonrec radius = json_number
  type nonrec color = fragments_light23_color_member1
  type nonrec fixture = fragments_rule11_remember
  type nonrec dx = json_number
  type nonrec dy = json_number
  type nonrec power = json_number
  type nonrec rule = fragments_light28_member1_when__member1
  type nonrec held_turns = json_number
  type nonrec last_counted_round = json_number
  type nonrec escalated = json_boolean
  type nonrec speaker = json_string
  type nonrec portrait = json_string
  type nonrec text = json_string
  type nonrec event = json_string
  type nonrec recruitment = fragments_native_state38_campaign_member1_scripts_member1_recruitment
  type nonrec failed = json_string
  type nonrec active = json_string
  type nonrec next_mark = json_number
  type nonrec pending = fragments_native_state38_campaign_member1_scripts_member1_pending_item
  type nonrec revealed = json_string
  type nonrec controller_case = fragments_factions7_item_controller_member0
  type nonrec cost = json_number
  type nonrec to_ = json_string
  type nonrec blade = json_number
  type nonrec pierce = json_number
  type nonrec impact = json_number
  type nonrec damage = json_number
  type nonrec spread = json_number
  type nonrec count = json_number
  type nonrec damage_type = fragments_native_state38_ground_items_item_item_attacks_item_damage_type
  type nonrec category = fragments_native_state38_ground_items_item_item_attacks_item_category
  type nonrec range = json_number
  type nonrec drain = json_number
  type nonrec amount = json_number
  type nonrec condition_case = fragments_rule11_condition_member0
  type nonrec predicate = json_number
  type nonrec turns = json_number
  type nonrec last_round = json_number
  type nonrec area = json_string
  type nonrec light_remembered = fragments_rule11_remember
  type nonrec unit_types = json_string
  type nonrec unit_type = json_string
  type nonrec choice = json_string
  type nonrec flag = json_string
  type nonrec conditions = fragments_light28_member1_when__member1_condition
end

module V38 = struct
  type nonrec t = save38
  type nonrec version = literal_38
  type nonrec checkpoint = save35_checkpoint
  type nonrec pending_enemy = json_boolean
  type nonrec setup = save35_setup
  type nonrec progress = save37_progress
  type nonrec undo = save34_undo_member1_item
  type nonrec seed = json_number
  type nonrec state = save35_checkpoint_state
  type nonrec first = save35_setup_first
  type nonrec enemy = save35_setup_enemy
  type nonrec entry = save37_progress_entry
  type nonrec round = json_number
  type nonrec unit_id = json_string
  type nonrec changes = save34_undo_member1_item_changes_item
  type nonrec side = save35_setup_first
  type nonrec winner = fragments_native_state38_winner
  type nonrec armies = fragments_native_state38_armies_member1
  type nonrec condition_memory = json_string
  type nonrec map_id = json_string
  type nonrec map_revision = json_number
  type nonrec cols = json_number
  type nonrec rows = json_number
  type nonrec next_id = json_number
  type nonrec log = json_string
  type nonrec last_combat = fragments_native_state38_last_combat_member2
  type nonrec unit_ = save26_checkpoint_state_units_item
  type nonrec tile = save26_checkpoint_state_tiles_item
  type nonrec campaign = save34_checkpoint_state_campaign_member1
  type nonrec gold = fragments_native_state38_gold
  type nonrec faction = save26_checkpoint_state_factions_item
  type nonrec ground_items = fragments_native_state38_ground_items_item
  type nonrec combat_rng = json_number
  type nonrec policy = save35_setup_enemy_policy
  type nonrec blocks = save35_setup_enemy_blocks_member1
  type nonrec recruitment_limit = json_number
  type nonrec diplomacy = save35_setup_enemy_diplomacy
  type nonrec entry_case = save37_progress_entry_member0
  type nonrec changes_case = save34_undo_member1_item_changes_item_member0
  type nonrec attacker_id = json_string
  type nonrec attack_id = json_string
  type nonrec sound = json_string
  type nonrec id = json_string
  type nonrec role = fragments_native_state38_units_item_role
  type nonrec items = fragments_native_state38_ground_items_item_item
  type nonrec ai_group = json_string
  type nonrec type_ = json_string
  type nonrec hp = json_number
  type nonrec moved = json_boolean
  type nonrec attacked = json_boolean
  type nonrec retreating = fragments_rule11_remember
  type nonrec rebuilding = fragments_rule11_remember
  type nonrec character = json_string
  type nonrec experience = json_number
  type nonrec owner = json_string
  type nonrec location = json_string
  type nonrec col = json_number
  type nonrec row = json_number
  type nonrec terrain = json_string
  type nonrec connection = fragments_tiles11_item_connections_member1_item
  type nonrec dead_ends = fragments_tiles11_item_connections_member1_item
  type nonrec walls = fragments_tiles11_item_connections_member1_item
  type nonrec floor = json_string
  type nonrec fog = fragments_light28_member1_when__member1
  type nonrec light = fragments_light28_member1
  type nonrec mission = json_number
  type nonrec vigil = fragments_native_state38_campaign_member1_vigil_member1
  type nonrec refuge = fragments_native_state38_campaign_member1_refuge_member1
  type nonrec completed = json_boolean
  type nonrec mark = json_string
  type nonrec hold = fragments_native_state38_campaign_member1_holds_member1
  type nonrec recall = json_string
  type nonrec fallen_companions = json_string
  type nonrec carry_gold = json_number
  type nonrec dialogue = fragments_native_state38_campaign_member1_dialogue_item
  type nonrec dialogue_index = json_number
  type nonrec outcome = json_string
  type nonrec veteran = save26_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec story_veteran = save26_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec evacuated = save26_checkpoint_state_campaign_member1_evacuated_member1_item
  type nonrec scripts = save34_checkpoint_state_campaign_member1_scripts_member1
  type nonrec flags = fragments_native_state38_campaign_member1_flags
  type nonrec name = json_string
  type nonrec phase = save35_setup_first
  type nonrec controller = fragments_factions7_item_controller
  type nonrec palette = json_string
  type nonrec recruits = fragments_factions7_item_recruits_member1_item
  type nonrec relations = fragments_factions7_item_relations_member1
  type nonrec transfer = save26_checkpoint_state_factions_item_transfer_member1
  type nonrec item = fragments_native_state38_ground_items_item_item
  type nonrec recruiter = save35_setup_enemy_blocks_member1_recruiter
  type nonrec planner = save35_setup_enemy_blocks_member1_planner
  type nonrec kind = literal_flash
  type nonrec saved_at = json_number
  type nonrec run = json_string
  type nonrec mission_start = json_boolean
  type nonrec run_id = json_string
  type nonrec branch_id = json_string
  type nonrec checkpoint_id = json_string
  type nonrec ordinal = json_number
  type nonrec manual = json_boolean
  type nonrec parent_id = json_string
  type nonrec op = literal_set
  type nonrec path = json_string
  type nonrec value = json_unknown
  type nonrec armor = fragments_native_state38_ground_items_item_item_armor_member1
  type nonrec description = json_string
  type nonrec attacks = fragments_native_state38_ground_items_item_item_attacks_item
  type nonrec healing = fragments_native_state38_ground_items_item_item_healing_member1
  type nonrec condition = fragments_light28_member1_when__member1_condition
  type nonrec remember = fragments_rule11_remember
  type nonrec radius = json_number
  type nonrec color = fragments_light23_color_member1
  type nonrec fixture = fragments_rule11_remember
  type nonrec dx = json_number
  type nonrec dy = json_number
  type nonrec power = json_number
  type nonrec rule = fragments_light28_member1_when__member1
  type nonrec held_turns = json_number
  type nonrec last_counted_round = json_number
  type nonrec escalated = json_boolean
  type nonrec speaker = json_string
  type nonrec portrait = json_string
  type nonrec text = json_string
  type nonrec event = json_string
  type nonrec recruitment = fragments_native_state38_campaign_member1_scripts_member1_recruitment
  type nonrec failed = json_string
  type nonrec active = json_string
  type nonrec next_mark = json_number
  type nonrec pending = fragments_native_state38_campaign_member1_scripts_member1_pending_item
  type nonrec revealed = json_string
  type nonrec controller_case = fragments_factions7_item_controller_member0
  type nonrec cost = json_number
  type nonrec to_ = json_string
  type nonrec blade = json_number
  type nonrec pierce = json_number
  type nonrec impact = json_number
  type nonrec damage = json_number
  type nonrec spread = json_number
  type nonrec count = json_number
  type nonrec damage_type = fragments_native_state38_ground_items_item_item_attacks_item_damage_type
  type nonrec category = fragments_native_state38_ground_items_item_item_attacks_item_category
  type nonrec range = json_number
  type nonrec drain = json_number
  type nonrec amount = json_number
  type nonrec condition_case = fragments_rule11_condition_member0
  type nonrec predicate = json_number
  type nonrec turns = json_number
  type nonrec last_round = json_number
  type nonrec area = json_string
  type nonrec light_remembered = fragments_rule11_remember
  type nonrec unit_types = json_string
  type nonrec unit_type = json_string
  type nonrec choice = json_string
  type nonrec flag = json_string
  type nonrec conditions = fragments_light28_member1_when__member1_condition
end

module V39 = struct
  type nonrec t = save39
  type nonrec t_case = save39_member0
  type nonrec pending_enemy = json_boolean
  type nonrec undo = save34_undo_member1_item
  type nonrec progress = save39_member0_progress
  type nonrec version = literal_39
  type nonrec checkpoint = save39_member0_checkpoint
  type nonrec setup = save39_member0_setup
  type nonrec round = json_number
  type nonrec unit_id = json_string
  type nonrec changes = save34_undo_member1_item_changes_item
  type nonrec entry = save37_progress_entry_member0
  type nonrec seed = json_number
  type nonrec state = save39_member0_checkpoint_state
  type nonrec first = save35_setup_first
  type nonrec enemy = save39_member0_setup_enemy
  type nonrec changes_case = save34_undo_member1_item_changes_item_member0
  type nonrec kind = literal_flash
  type nonrec id = json_string
  type nonrec saved_at = json_number
  type nonrec run = json_string
  type nonrec mission = json_string
  type nonrec side = json_string
  type nonrec mission_start = json_boolean
  type nonrec run_id = json_string
  type nonrec branch_id = json_string
  type nonrec checkpoint_id = json_string
  type nonrec ordinal = json_number
  type nonrec faction = save39_member0_checkpoint_state_factions_item
  type nonrec ground_items = save39_member0_checkpoint_state_ground_items_item
  type nonrec armies = fragments_native_state38_armies_member1
  type nonrec condition_memory = json_string
  type nonrec map_id = json_string
  type nonrec map_revision = json_number
  type nonrec cols = json_number
  type nonrec rows = json_number
  type nonrec gold = fragments_native_state38_gold
  type nonrec winner = fragments_native_state38_winner
  type nonrec next_id = json_number
  type nonrec combat_rng = json_number
  type nonrec log = json_string
  type nonrec tile = save39_member0_checkpoint_state_tiles_item
  type nonrec unit_ = save39_member0_checkpoint_state_units_item
  type nonrec campaign = save39_member0_checkpoint_state_campaign_member1
  type nonrec last_combat = fragments_native_state38_last_combat_member2
  type nonrec blocks = save35_setup_enemy_blocks_member1
  type nonrec policy = save35_setup_enemy_policy
  type nonrec diplomacy = save35_setup_enemy_diplomacy
  type nonrec recruitment_limit = json_number
  type nonrec manual = json_boolean
  type nonrec parent_id = json_string
  type nonrec op = literal_set
  type nonrec path = json_string
  type nonrec value = json_unknown
  type nonrec phase = save35_setup_first
  type nonrec controller = fragments_factions7_item_controller
  type nonrec palette = json_string
  type nonrec name = json_string
  type nonrec recruits = fragments_factions7_item_recruits_member1_item
  type nonrec relations = fragments_factions7_item_relations_member1
  type nonrec transfer = save39_member0_checkpoint_state_factions_item_transfer_member1
  type nonrec item = fragments_native_state38_ground_items_item_item
  type nonrec location = json_string
  type nonrec col = json_number
  type nonrec row = json_number
  type nonrec terrain = json_string
  type nonrec owner = json_string
  type nonrec connection = fragments_tiles11_item_connections_member1_item
  type nonrec dead_ends = fragments_tiles11_item_connections_member1_item
  type nonrec walls = fragments_tiles11_item_connections_member1_item
  type nonrec floor = json_string
  type nonrec fog = save39_member0_checkpoint_state_tiles_item_fog_member1
  type nonrec light = save39_member0_checkpoint_state_tiles_item_light_member1
  type nonrec items = fragments_native_state38_ground_items_item_item
  type nonrec ai_group = json_string
  type nonrec type_ = json_string
  type nonrec hp = json_number
  type nonrec moved = json_boolean
  type nonrec attacked = json_boolean
  type nonrec experience = json_number
  type nonrec character = json_string
  type nonrec role = fragments_native_state38_units_item_role
  type nonrec retreating = fragments_rule11_remember
  type nonrec rebuilding = fragments_rule11_remember
  type nonrec scripts = save39_member0_checkpoint_state_campaign_member1_scripts_member1
  type nonrec evacuated = save39_member0_checkpoint_state_campaign_member1_evacuated_member1_item
  type nonrec recall = json_string
  type nonrec veteran = fragments_native_state38_campaign_member1_veterans_member1_item
  type nonrec story_veteran = fragments_native_state38_campaign_member1_veterans_member1_item
  type nonrec fallen_companions = json_string
  type nonrec carry_gold = json_number
  type nonrec mark = json_string
  type nonrec dialogue = fragments_native_state38_campaign_member1_dialogue_item
  type nonrec dialogue_index = json_number
  type nonrec outcome = json_string
  type nonrec completed = json_boolean
  type nonrec refuge = fragments_native_state38_campaign_member1_refuge_member1
  type nonrec vigil = fragments_native_state38_campaign_member1_vigil_member1
  type nonrec hold = fragments_native_state38_campaign_member1_holds_member1
  type nonrec flags = fragments_native_state38_campaign_member1_flags
  type nonrec attacker_id = json_string
  type nonrec attack_id = json_string
  type nonrec sound = json_string
  type nonrec recruiter = save35_setup_enemy_blocks_member1_recruiter
  type nonrec planner = save35_setup_enemy_blocks_member1_planner
  type nonrec controller_case = fragments_factions7_item_controller_member0
  type nonrec cost = json_number
  type nonrec to_ = json_string
  type nonrec rule = save39_member0_checkpoint_state_tiles_item_fog_member1
  type nonrec armor = fragments_native_state38_ground_items_item_item_armor_member1
  type nonrec description = json_string
  type nonrec attacks = fragments_native_state38_ground_items_item_item_attacks_item
  type nonrec healing = fragments_native_state38_ground_items_item_item_healing_member1
  type nonrec condition = save39_member0_checkpoint_state_tiles_item_fog_member1_condition
  type nonrec remember = fragments_rule11_remember
  type nonrec color = fragments_light23_color_member1
  type nonrec fixture = fragments_rule11_remember
  type nonrec dx = json_number
  type nonrec dy = json_number
  type nonrec radius = json_number
  type nonrec power = json_number
  type nonrec failed = json_string
  type nonrec revealed = json_string
  type nonrec recruitment = fragments_native_state38_campaign_member1_scripts_member1_recruitment
  type nonrec active = json_string

  type nonrec pending =
    save39_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item

  type nonrec next_mark = json_number
  type nonrec event = json_string
  type nonrec speaker = json_string
  type nonrec portrait = json_string
  type nonrec text = json_string
  type nonrec held_turns = json_number
  type nonrec last_counted_round = json_number
  type nonrec escalated = json_boolean
  type nonrec blade = json_number
  type nonrec pierce = json_number
  type nonrec impact = json_number
  type nonrec damage = json_number
  type nonrec spread = json_number
  type nonrec count = json_number
  type nonrec damage_type = fragments_native_state38_ground_items_item_item_attacks_item_damage_type
  type nonrec category = fragments_native_state38_ground_items_item_item_attacks_item_category
  type nonrec range = json_number
  type nonrec drain = json_number
  type nonrec amount = json_number

  type nonrec condition_case =
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0

  type nonrec predicate = json_number
  type nonrec turns = json_number
  type nonrec last_round = json_number
  type nonrec light_remembered = fragments_rule11_remember
  type nonrec area = save39_member0_checkpoint_state_ground_items_item_tile
  type nonrec unit_types = json_string
  type nonrec unit_type = json_string
  type nonrec choice = json_string
  type nonrec flag = json_string
  type nonrec conditions = save39_member0_checkpoint_state_tiles_item_fog_member1_condition

  type nonrec tile_case =
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member1

  type nonrec hex = save39_member0_checkpoint_state_ground_items_item_tile

  type nonrec binding =
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member3_binding

  type nonrec current = save39_member0
  type nonrec current_pending_enemy = json_boolean
  type nonrec current_undo = save34_undo_member1_item
  type nonrec current_progress = save39_member0_progress
  type nonrec current_version = literal_39
  type nonrec current_checkpoint = save39_member0_checkpoint
  type nonrec current_setup = save39_member0_setup
  type nonrec current_round = json_number
  type nonrec current_unit_id = json_string
  type nonrec current_changes = save34_undo_member1_item_changes_item
  type nonrec current_entry = save37_progress_entry_member0
  type nonrec current_seed = json_number
  type nonrec current_state = save39_member0_checkpoint_state
  type nonrec current_first = save35_setup_first
  type nonrec current_enemy = save39_member0_setup_enemy
  type nonrec current_changes_case = save34_undo_member1_item_changes_item_member0
  type nonrec current_kind = literal_flash
  type nonrec current_id = json_string
  type nonrec current_saved_at = json_number
  type nonrec current_run = json_string
  type nonrec current_mission = json_string
  type nonrec current_side = json_string
  type nonrec current_mission_start = json_boolean
  type nonrec current_run_id = json_string
  type nonrec current_branch_id = json_string
  type nonrec current_checkpoint_id = json_string
  type nonrec current_ordinal = json_number
  type nonrec current_faction = save39_member0_checkpoint_state_factions_item
  type nonrec current_ground_items = save39_member0_checkpoint_state_ground_items_item
  type nonrec current_armies = fragments_native_state38_armies_member1
  type nonrec current_condition_memory = json_string
  type nonrec current_map_id = json_string
  type nonrec current_map_revision = json_number
  type nonrec current_cols = json_number
  type nonrec current_rows = json_number
  type nonrec current_gold = fragments_native_state38_gold
  type nonrec current_winner = fragments_native_state38_winner
  type nonrec current_next_id = json_number
  type nonrec current_combat_rng = json_number
  type nonrec current_log = json_string
  type nonrec current_tile = save39_member0_checkpoint_state_tiles_item
  type nonrec current_unit_ = save39_member0_checkpoint_state_units_item
  type nonrec current_campaign = save39_member0_checkpoint_state_campaign_member1
  type nonrec current_last_combat = fragments_native_state38_last_combat_member2
  type nonrec current_blocks = save35_setup_enemy_blocks_member1
  type nonrec current_policy = save35_setup_enemy_policy
  type nonrec current_diplomacy = save35_setup_enemy_diplomacy
  type nonrec current_recruitment_limit = json_number
  type nonrec current_op = literal_set
  type nonrec current_path = json_string
  type nonrec current_value = json_unknown
  type nonrec current_phase = save35_setup_first
  type nonrec current_controller = fragments_factions7_item_controller
  type nonrec current_palette = json_string
  type nonrec current_name = json_string
  type nonrec current_recruits = fragments_factions7_item_recruits_member1_item
  type nonrec current_relations = fragments_factions7_item_relations_member1
  type nonrec current_transfer = save39_member0_checkpoint_state_factions_item_transfer_member1
  type nonrec current_item = fragments_native_state38_ground_items_item_item
  type nonrec current_location = json_string
  type nonrec current_col = json_number
  type nonrec current_row = json_number
  type nonrec current_terrain = json_string
  type nonrec current_owner = json_string
  type nonrec current_connection = fragments_tiles11_item_connections_member1_item
  type nonrec current_dead_ends = fragments_tiles11_item_connections_member1_item
  type nonrec current_walls = fragments_tiles11_item_connections_member1_item
  type nonrec current_floor = json_string
  type nonrec current_fog = save39_member0_checkpoint_state_tiles_item_fog_member1
  type nonrec current_light = save39_member0_checkpoint_state_tiles_item_light_member1
  type nonrec current_items = fragments_native_state38_ground_items_item_item
  type nonrec current_ai_group = json_string
  type nonrec current_type_ = json_string
  type nonrec current_hp = json_number
  type nonrec current_moved = json_boolean
  type nonrec current_attacked = json_boolean
  type nonrec current_experience = json_number
  type nonrec current_character = json_string
  type nonrec current_role = fragments_native_state38_units_item_role
  type nonrec current_retreating = fragments_rule11_remember
  type nonrec current_rebuilding = fragments_rule11_remember
  type nonrec current_scripts = save39_member0_checkpoint_state_campaign_member1_scripts_member1

  type nonrec current_evacuated =
    save39_member0_checkpoint_state_campaign_member1_evacuated_member1_item

  type nonrec current_recall = json_string
  type nonrec current_veteran = fragments_native_state38_campaign_member1_veterans_member1_item

  type nonrec current_story_veteran =
    fragments_native_state38_campaign_member1_veterans_member1_item

  type nonrec current_fallen_companions = json_string
  type nonrec current_carry_gold = json_number
  type nonrec current_mark = json_string
  type nonrec current_dialogue = fragments_native_state38_campaign_member1_dialogue_item
  type nonrec current_dialogue_index = json_number
  type nonrec current_outcome = json_string
  type nonrec current_completed = json_boolean
  type nonrec current_refuge = fragments_native_state38_campaign_member1_refuge_member1
  type nonrec current_vigil = fragments_native_state38_campaign_member1_vigil_member1
  type nonrec current_hold = fragments_native_state38_campaign_member1_holds_member1
  type nonrec current_flags = fragments_native_state38_campaign_member1_flags
  type nonrec current_attacker_id = json_string
  type nonrec current_attack_id = json_string
  type nonrec current_sound = json_string
  type nonrec current_recruiter = save35_setup_enemy_blocks_member1_recruiter
  type nonrec current_planner = save35_setup_enemy_blocks_member1_planner
  type nonrec current_controller_case = fragments_factions7_item_controller_member0
  type nonrec current_cost = json_number
  type nonrec current_to_ = json_string
  type nonrec current_rule = save39_member0_checkpoint_state_tiles_item_fog_member1
  type nonrec current_armor = fragments_native_state38_ground_items_item_item_armor_member1
  type nonrec current_description = json_string
  type nonrec current_attacks = fragments_native_state38_ground_items_item_item_attacks_item
  type nonrec current_healing = fragments_native_state38_ground_items_item_item_healing_member1
  type nonrec current_condition = save39_member0_checkpoint_state_tiles_item_fog_member1_condition
  type nonrec current_remember = fragments_rule11_remember
  type nonrec current_color = fragments_light23_color_member1
  type nonrec current_fixture = fragments_rule11_remember
  type nonrec current_dx = json_number
  type nonrec current_dy = json_number
  type nonrec current_radius = json_number
  type nonrec current_power = json_number
  type nonrec current_failed = json_string
  type nonrec current_revealed = json_string

  type nonrec current_recruitment =
    fragments_native_state38_campaign_member1_scripts_member1_recruitment

  type nonrec current_active = json_string

  type nonrec current_pending =
    save39_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item

  type nonrec current_next_mark = json_number
  type nonrec current_event = json_string
  type nonrec current_speaker = json_string
  type nonrec current_portrait = json_string
  type nonrec current_text = json_string
  type nonrec current_held_turns = json_number
  type nonrec current_last_counted_round = json_number
  type nonrec current_escalated = json_boolean
  type nonrec current_blade = json_number
  type nonrec current_pierce = json_number
  type nonrec current_impact = json_number
  type nonrec current_damage = json_number
  type nonrec current_spread = json_number
  type nonrec current_count = json_number

  type nonrec current_damage_type =
    fragments_native_state38_ground_items_item_item_attacks_item_damage_type

  type nonrec current_category =
    fragments_native_state38_ground_items_item_item_attacks_item_category

  type nonrec current_range = json_number
  type nonrec current_drain = json_number
  type nonrec current_amount = json_number

  type nonrec current_condition_case =
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0

  type nonrec current_predicate = json_number
  type nonrec current_turns = json_number
  type nonrec current_last_round = json_number
  type nonrec current_light_remembered = fragments_rule11_remember
  type nonrec current_area = save39_member0_checkpoint_state_ground_items_item_tile
  type nonrec current_unit_types = json_string
  type nonrec current_unit_type = json_string
  type nonrec current_choice = json_string
  type nonrec current_flag = json_string
  type nonrec current_conditions = save39_member0_checkpoint_state_tiles_item_fog_member1_condition

  type nonrec current_tile_case =
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member1

  type nonrec current_hex = save39_member0_checkpoint_state_ground_items_item_tile

  type nonrec current_binding =
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member3_binding

  type nonrec turn = save39_member1
  type nonrec turn_pending_enemy = literal_false
  type nonrec turn_undo = json_undefined
  type nonrec turn_progress = save39_member1_progress
  type nonrec turn_version = literal_39
  type nonrec turn_checkpoint = save39_member0_checkpoint
  type nonrec turn_setup = save39_member0_setup
  type nonrec turn_entry = history4_entries_item
  type nonrec turn_seed = json_number
  type nonrec turn_state = save39_member0_checkpoint_state
  type nonrec turn_first = save35_setup_first
  type nonrec turn_enemy = save39_member0_setup_enemy
  type nonrec turn_kind = literal_turn
  type nonrec turn_manual = json_boolean
  type nonrec turn_parent_id = json_string
  type nonrec turn_id = json_string
  type nonrec turn_saved_at = json_number
  type nonrec turn_run = json_string
  type nonrec turn_mission = json_string
  type nonrec turn_round = json_number
  type nonrec turn_side = json_string
  type nonrec turn_mission_start = json_boolean
  type nonrec turn_run_id = json_string
  type nonrec turn_branch_id = json_string
  type nonrec turn_checkpoint_id = json_string
  type nonrec turn_ordinal = json_number
  type nonrec turn_faction = save39_member0_checkpoint_state_factions_item
  type nonrec turn_ground_items = save39_member0_checkpoint_state_ground_items_item
  type nonrec turn_armies = fragments_native_state38_armies_member1
  type nonrec turn_condition_memory = json_string
  type nonrec turn_map_id = json_string
  type nonrec turn_map_revision = json_number
  type nonrec turn_cols = json_number
  type nonrec turn_rows = json_number
  type nonrec turn_gold = fragments_native_state38_gold
  type nonrec turn_winner = fragments_native_state38_winner
  type nonrec turn_next_id = json_number
  type nonrec turn_combat_rng = json_number
  type nonrec turn_log = json_string
  type nonrec turn_tile = save39_member0_checkpoint_state_tiles_item
  type nonrec turn_unit_ = save39_member0_checkpoint_state_units_item
  type nonrec turn_campaign = save39_member0_checkpoint_state_campaign_member1
  type nonrec turn_last_combat = fragments_native_state38_last_combat_member2
  type nonrec turn_blocks = save35_setup_enemy_blocks_member1
  type nonrec turn_policy = save35_setup_enemy_policy
  type nonrec turn_diplomacy = save35_setup_enemy_diplomacy
  type nonrec turn_recruitment_limit = json_number
  type nonrec turn_phase = save35_setup_first
  type nonrec turn_controller = fragments_factions7_item_controller
  type nonrec turn_palette = json_string
  type nonrec turn_name = json_string
  type nonrec turn_recruits = fragments_factions7_item_recruits_member1_item
  type nonrec turn_relations = fragments_factions7_item_relations_member1
  type nonrec turn_transfer = save39_member0_checkpoint_state_factions_item_transfer_member1
  type nonrec turn_item = fragments_native_state38_ground_items_item_item
  type nonrec turn_location = json_string
  type nonrec turn_col = json_number
  type nonrec turn_row = json_number
  type nonrec turn_terrain = json_string
  type nonrec turn_owner = json_string
  type nonrec turn_connection = fragments_tiles11_item_connections_member1_item
  type nonrec turn_dead_ends = fragments_tiles11_item_connections_member1_item
  type nonrec turn_walls = fragments_tiles11_item_connections_member1_item
  type nonrec turn_floor = json_string
  type nonrec turn_fog = save39_member0_checkpoint_state_tiles_item_fog_member1
  type nonrec turn_light = save39_member0_checkpoint_state_tiles_item_light_member1
  type nonrec turn_items = fragments_native_state38_ground_items_item_item
  type nonrec turn_ai_group = json_string
  type nonrec turn_type_ = json_string
  type nonrec turn_hp = json_number
  type nonrec turn_moved = json_boolean
  type nonrec turn_attacked = json_boolean
  type nonrec turn_experience = json_number
  type nonrec turn_character = json_string
  type nonrec turn_role = fragments_native_state38_units_item_role
  type nonrec turn_retreating = fragments_rule11_remember
  type nonrec turn_rebuilding = fragments_rule11_remember
  type nonrec turn_scripts = save39_member0_checkpoint_state_campaign_member1_scripts_member1

  type nonrec turn_evacuated =
    save39_member0_checkpoint_state_campaign_member1_evacuated_member1_item

  type nonrec turn_recall = json_string
  type nonrec turn_veteran = fragments_native_state38_campaign_member1_veterans_member1_item
  type nonrec turn_story_veteran = fragments_native_state38_campaign_member1_veterans_member1_item
  type nonrec turn_fallen_companions = json_string
  type nonrec turn_carry_gold = json_number
  type nonrec turn_mark = json_string
  type nonrec turn_dialogue = fragments_native_state38_campaign_member1_dialogue_item
  type nonrec turn_dialogue_index = json_number
  type nonrec turn_outcome = json_string
  type nonrec turn_completed = json_boolean
  type nonrec turn_refuge = fragments_native_state38_campaign_member1_refuge_member1
  type nonrec turn_vigil = fragments_native_state38_campaign_member1_vigil_member1
  type nonrec turn_hold = fragments_native_state38_campaign_member1_holds_member1
  type nonrec turn_flags = fragments_native_state38_campaign_member1_flags
  type nonrec turn_attacker_id = json_string
  type nonrec turn_attack_id = json_string
  type nonrec turn_sound = json_string
  type nonrec turn_recruiter = save35_setup_enemy_blocks_member1_recruiter
  type nonrec turn_planner = save35_setup_enemy_blocks_member1_planner
  type nonrec turn_controller_case = fragments_factions7_item_controller_member0
  type nonrec turn_cost = json_number
  type nonrec turn_to_ = json_string
  type nonrec turn_rule = save39_member0_checkpoint_state_tiles_item_fog_member1
  type nonrec turn_armor = fragments_native_state38_ground_items_item_item_armor_member1
  type nonrec turn_description = json_string
  type nonrec turn_attacks = fragments_native_state38_ground_items_item_item_attacks_item
  type nonrec turn_healing = fragments_native_state38_ground_items_item_item_healing_member1
  type nonrec turn_condition = save39_member0_checkpoint_state_tiles_item_fog_member1_condition
  type nonrec turn_remember = fragments_rule11_remember
  type nonrec turn_color = fragments_light23_color_member1
  type nonrec turn_fixture = fragments_rule11_remember
  type nonrec turn_dx = json_number
  type nonrec turn_dy = json_number
  type nonrec turn_radius = json_number
  type nonrec turn_power = json_number
  type nonrec turn_failed = json_string
  type nonrec turn_revealed = json_string

  type nonrec turn_recruitment =
    fragments_native_state38_campaign_member1_scripts_member1_recruitment

  type nonrec turn_active = json_string

  type nonrec turn_pending =
    save39_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item

  type nonrec turn_next_mark = json_number
  type nonrec turn_event = json_string
  type nonrec turn_speaker = json_string
  type nonrec turn_portrait = json_string
  type nonrec turn_text = json_string
  type nonrec turn_held_turns = json_number
  type nonrec turn_last_counted_round = json_number
  type nonrec turn_escalated = json_boolean
  type nonrec turn_blade = json_number
  type nonrec turn_pierce = json_number
  type nonrec turn_impact = json_number
  type nonrec turn_damage = json_number
  type nonrec turn_spread = json_number
  type nonrec turn_count = json_number

  type nonrec turn_damage_type =
    fragments_native_state38_ground_items_item_item_attacks_item_damage_type

  type nonrec turn_category = fragments_native_state38_ground_items_item_item_attacks_item_category
  type nonrec turn_range = json_number
  type nonrec turn_drain = json_number
  type nonrec turn_amount = json_number

  type nonrec turn_condition_case =
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0

  type nonrec turn_predicate = json_number
  type nonrec turn_turns = json_number
  type nonrec turn_last_round = json_number
  type nonrec turn_light_remembered = fragments_rule11_remember
  type nonrec turn_area = save39_member0_checkpoint_state_ground_items_item_tile
  type nonrec turn_unit_types = json_string
  type nonrec turn_unit_type = json_string
  type nonrec turn_choice = json_string
  type nonrec turn_flag = json_string
  type nonrec turn_conditions = save39_member0_checkpoint_state_tiles_item_fog_member1_condition

  type nonrec turn_tile_case =
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member1

  type nonrec turn_hex = save39_member0_checkpoint_state_ground_items_item_tile

  type nonrec turn_binding =
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member3_binding
end

module History1 = struct
  type nonrec t = history1
  type nonrec version = literal_1
  type nonrec entries = history1_entries_item
  type nonrec id = json_string
  type nonrec saved_at = json_number
  type nonrec run = json_string
  type nonrec mission = json_string
  type nonrec round = json_number
  type nonrec side = json_string
  type nonrec mission_start = json_boolean
  type nonrec run_id = json_string
  type nonrec branch_id = json_string
  type nonrec checkpoint_id = json_string
  type nonrec ordinal = json_number
  type nonrec kind = history1_entries_item_kind
  type nonrec parent_id = json_string
end

module History2 = struct
  type nonrec t = history2
  type nonrec version = literal_2
  type nonrec entries = history2_entries_item
  type nonrec id = json_string
  type nonrec saved_at = json_number
  type nonrec run = json_string
  type nonrec mission = json_string
  type nonrec round = json_number
  type nonrec side = json_string
  type nonrec mission_start = json_boolean
  type nonrec run_id = json_string
  type nonrec branch_id = json_string
  type nonrec checkpoint_id = json_string
  type nonrec ordinal = json_number
  type nonrec kind = literal_turn
  type nonrec parent_id = json_string
end

module History3 = struct
  type nonrec t = history3
  type nonrec version = literal_3
  type nonrec entries = history3_entries_item
  type nonrec id = json_string
  type nonrec saved_at = json_number
  type nonrec run = json_string
  type nonrec mission = json_string
  type nonrec round = json_number
  type nonrec side = json_string
  type nonrec mission_start = json_boolean
  type nonrec run_id = json_string
  type nonrec branch_id = json_string
  type nonrec checkpoint_id = json_string
  type nonrec ordinal = json_number
  type nonrec kind = history3_entries_item_kind
  type nonrec parent_id = json_string
end

module History4 = struct
  type nonrec t = history4
  type nonrec version = literal_4
  type nonrec entries = history4_entries_item
  type nonrec kind = literal_turn
  type nonrec manual = json_boolean
  type nonrec parent_id = json_string
  type nonrec id = json_string
  type nonrec saved_at = json_number
  type nonrec run = json_string
  type nonrec mission = json_string
  type nonrec round = json_number
  type nonrec side = json_string
  type nonrec mission_start = json_boolean
  type nonrec run_id = json_string
  type nonrec branch_id = json_string
  type nonrec checkpoint_id = json_string
  type nonrec ordinal = json_number
  type nonrec entry = history4_entries_item
end

module Fragments = struct
  type nonrec t = fragments
  type nonrec native_state38 = fragments_native_state38
  type nonrec factions7 = fragments_factions7
  type nonrec factions11 = fragments_factions7
  type nonrec factions21 = fragments_factions7
  type nonrec factions24 = fragments_factions7
  type nonrec tiles11 = fragments_tiles11
  type nonrec rule11 = fragments_rule11
  type nonrec delta19 = fragments_delta19
  type nonrec delta20 = fragments_delta19
  type nonrec delta21 = fragments_delta19
  type nonrec light23 = fragments_light23
  type nonrec light28 = fragments_light28
  type nonrec faction = fragments_native_state38_factions_item
  type nonrec ground_items = fragments_native_state38_ground_items_item
  type nonrec armies = fragments_native_state38_armies_member1
  type nonrec condition_memory = json_string
  type nonrec map_id = json_string
  type nonrec map_revision = json_number
  type nonrec cols = json_number
  type nonrec rows = json_number
  type nonrec round = json_number
  type nonrec side = save35_setup_first
  type nonrec gold = fragments_native_state38_gold
  type nonrec winner = fragments_native_state38_winner
  type nonrec next_id = json_number
  type nonrec combat_rng = json_number
  type nonrec log = json_string
  type nonrec tile = fragments_native_state38_tiles_item
  type nonrec unit_ = fragments_native_state38_units_item
  type nonrec campaign = fragments_native_state38_campaign_member1
  type nonrec last_combat = fragments_native_state38_last_combat_member2
  type nonrec id = json_string
  type nonrec name = json_string
  type nonrec phase = save35_setup_first
  type nonrec controller = fragments_factions7_item_controller
  type nonrec palette = json_string
  type nonrec recruits = fragments_factions7_item_recruits_member1_item
  type nonrec relations = fragments_factions7_item_relations_member1
  type nonrec transfer = fragments_factions7_item_transfer_member1
  type nonrec owner = json_string
  type nonrec location = json_string
  type nonrec col = json_number
  type nonrec row = json_number
  type nonrec terrain = json_string
  type nonrec connection = fragments_tiles11_item_connections_member1_item
  type nonrec dead_ends = fragments_tiles11_item_connections_member1_item
  type nonrec walls = fragments_tiles11_item_connections_member1_item
  type nonrec floor = json_string
  type nonrec fog = fragments_rule11
  type nonrec light = fragments_light23
  type nonrec wall_corners = json_number
  type nonrec condition = fragments_rule11_condition
  type nonrec remember = fragments_rule11_remember
  type nonrec radius = json_number
  type nonrec color = fragments_light23_color_member1
  type nonrec fixture = fragments_rule11_remember
  type nonrec dx = json_number
  type nonrec dy = json_number
  type nonrec power = json_number
  type nonrec rule = fragments_rule11
  type nonrec item = fragments_native_state38_ground_items_item_item
  type nonrec items = fragments_native_state38_ground_items_item_item
  type nonrec ai_group = json_string
  type nonrec type_ = json_string
  type nonrec hp = json_number
  type nonrec moved = json_boolean
  type nonrec attacked = json_boolean
  type nonrec experience = json_number
  type nonrec character = json_string
  type nonrec role = fragments_native_state38_units_item_role
  type nonrec retreating = fragments_rule11_remember
  type nonrec rebuilding = fragments_rule11_remember
  type nonrec scripts = fragments_native_state38_campaign_member1_scripts_member1
  type nonrec evacuated = fragments_native_state38_campaign_member1_evacuated_member1_item
  type nonrec mission = json_number
  type nonrec recall = json_string
  type nonrec veteran = fragments_native_state38_campaign_member1_veterans_member1_item
  type nonrec story_veteran = fragments_native_state38_campaign_member1_veterans_member1_item
  type nonrec fallen_companions = json_string
  type nonrec carry_gold = json_number
  type nonrec mark = json_string
  type nonrec dialogue = fragments_native_state38_campaign_member1_dialogue_item
  type nonrec dialogue_index = json_number
  type nonrec outcome = json_string
  type nonrec completed = json_boolean
  type nonrec refuge = fragments_native_state38_campaign_member1_refuge_member1
  type nonrec vigil = fragments_native_state38_campaign_member1_vigil_member1
  type nonrec hold = fragments_native_state38_campaign_member1_holds_member1
  type nonrec flags = fragments_native_state38_campaign_member1_flags
  type nonrec attacker_id = json_string
  type nonrec attack_id = json_string
  type nonrec sound = json_string
  type nonrec controller_case = fragments_factions7_item_controller_member0
  type nonrec cost = json_number
  type nonrec to_ = json_string
  type nonrec condition_case = fragments_rule11_condition_member0
  type nonrec armor = fragments_native_state38_ground_items_item_item_armor_member1
  type nonrec description = json_string
  type nonrec attacks = fragments_native_state38_ground_items_item_item_attacks_item
  type nonrec healing = fragments_native_state38_ground_items_item_item_healing_member1
  type nonrec failed = json_string
  type nonrec revealed = json_string
  type nonrec recruitment = fragments_native_state38_campaign_member1_scripts_member1_recruitment
  type nonrec active = json_string
  type nonrec pending = fragments_native_state38_campaign_member1_scripts_member1_pending_item
  type nonrec next_mark = json_number
  type nonrec event = json_string
  type nonrec speaker = json_string
  type nonrec portrait = json_string
  type nonrec text = json_string
  type nonrec held_turns = json_number
  type nonrec last_counted_round = json_number
  type nonrec escalated = json_boolean
  type nonrec kind = literal_human
  type nonrec policy = fragments_factions7_item_controller_member2_policy
  type nonrec unit_types = json_string
  type nonrec choice = json_string
  type nonrec flag = json_string
  type nonrec unit_type = json_string
  type nonrec conditions = fragments_rule11_condition
  type nonrec blade = json_number
  type nonrec pierce = json_number
  type nonrec impact = json_number
  type nonrec damage = json_number
  type nonrec spread = json_number
  type nonrec count = json_number
  type nonrec damage_type = fragments_native_state38_ground_items_item_item_attacks_item_damage_type
  type nonrec category = fragments_native_state38_ground_items_item_item_attacks_item_category
  type nonrec range = json_number
  type nonrec drain = json_number
  type nonrec amount = json_number
  type nonrec predicate = json_number
  type nonrec turns = json_number
  type nonrec last_round = json_number
  type nonrec light_remembered = fragments_rule11_remember
  type nonrec area = json_string
  type nonrec factions7_item = fragments_factions7_item
  type nonrec factions11_item = fragments_factions7_item
  type nonrec factions21_item = fragments_factions7_item
  type nonrec factions24_item = fragments_factions7_item
  type nonrec tiles11_item = fragments_tiles11_item
  type nonrec delta19_item = fragments_delta19_item
  type nonrec delta20_item = fragments_delta19_item
  type nonrec delta21_item = fragments_delta19_item
end

module Native1 = struct
  type nonrec t = native1
  type nonrec engine = literal_rust_wasm
  type nonrec version = literal_1
  type nonrec pending_enemy = json_boolean
  type nonrec saved = native1_saved
  type nonrec seed = json_number
  type nonrec state = save2_checkpoint_state
  type nonrec round = json_number
  type nonrec side = save35_setup_first
  type nonrec winner = fragments_native_state38_winner
  type nonrec armies = fragments_native_state38_armies_member1
  type nonrec condition_memory = json_string
  type nonrec map_id = json_string
  type nonrec map_revision = json_number
  type nonrec cols = json_number
  type nonrec rows = json_number
  type nonrec next_id = json_number
  type nonrec log = json_string
  type nonrec last_combat = fragments_native_state38_last_combat_member2
  type nonrec unit_ = save2_checkpoint_state_units_item
  type nonrec tile = save2_checkpoint_state_tiles_item
  type nonrec campaign = save2_checkpoint_state_campaign_member1
  type nonrec gold = javascript1_state_gold
  type nonrec faction = save2_checkpoint_state_factions_member1_item
  type nonrec ground_items = fragments_native_state38_ground_items_item
  type nonrec combat_rng = json_number
  type nonrec attacker_id = json_string
  type nonrec attack_id = json_string
  type nonrec sound = json_string
  type nonrec id = json_string
  type nonrec role = fragments_native_state38_units_item_role
  type nonrec items = fragments_native_state38_ground_items_item_item
  type nonrec ai_group = json_string
  type nonrec type_ = json_string
  type nonrec hp = json_number
  type nonrec moved = json_boolean
  type nonrec attacked = json_boolean
  type nonrec retreating = fragments_rule11_remember
  type nonrec rebuilding = fragments_rule11_remember
  type nonrec name = json_string
  type nonrec character = json_string
  type nonrec experience = json_number
  type nonrec owner = json_string
  type nonrec location = json_string
  type nonrec col = json_number
  type nonrec row = json_number
  type nonrec terrain = json_string
  type nonrec connection = fragments_tiles11_item_connections_member1_item
  type nonrec dead_ends = fragments_tiles11_item_connections_member1_item
  type nonrec walls = fragments_tiles11_item_connections_member1_item
  type nonrec floor = json_string
  type nonrec fog = save2_checkpoint_state_tiles_item_fog_member1
  type nonrec light = save2_checkpoint_state_tiles_item_light_member1
  type nonrec wall_corners = json_number
  type nonrec mission = json_number
  type nonrec vigil = fragments_native_state38_campaign_member1_vigil_member1
  type nonrec refuge = fragments_native_state38_campaign_member1_refuge_member1
  type nonrec completed = json_boolean
  type nonrec mark = json_string
  type nonrec hold = fragments_native_state38_campaign_member1_holds_member1
  type nonrec recall = json_string
  type nonrec fallen_companions = json_string
  type nonrec carry_gold = json_number
  type nonrec dialogue = fragments_native_state38_campaign_member1_dialogue_item
  type nonrec dialogue_index = json_number
  type nonrec outcome = json_string
  type nonrec veteran = save2_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec story_veteran = save2_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec evacuated = save2_checkpoint_state_campaign_member1_evacuated_member1_item
  type nonrec choices = json_string
  type nonrec player = json_number
  type nonrec enemy = json_number
  type nonrec phase = save35_setup_first
  type nonrec controller = fragments_factions7_item_controller
  type nonrec palette = json_string
  type nonrec recruits = fragments_factions7_item_recruits_member1_item
  type nonrec relations = fragments_factions7_item_relations_member1
  type nonrec transfer = save2_checkpoint_state_factions_member1_item_transfer_member1
  type nonrec item = fragments_native_state38_ground_items_item_item
  type nonrec armor = fragments_native_state38_ground_items_item_item_armor_member1
  type nonrec description = json_string
  type nonrec attacks = fragments_native_state38_ground_items_item_item_attacks_item
  type nonrec healing = fragments_native_state38_ground_items_item_item_healing_member1
  type nonrec condition = save2_checkpoint_state_tiles_item_fog_member1_condition
  type nonrec remember = fragments_rule11_remember
  type nonrec radius = json_number
  type nonrec color = fragments_light23_color_member1
  type nonrec fixture = fragments_rule11_remember
  type nonrec dx = json_number
  type nonrec dy = json_number
  type nonrec power = json_number
  type nonrec rule = save2_checkpoint_state_tiles_item_fog_member1
  type nonrec held_turns = json_number
  type nonrec last_counted_round = json_number
  type nonrec escalated = json_boolean
  type nonrec speaker = json_string
  type nonrec portrait = json_string
  type nonrec text = json_string
  type nonrec event = json_string
  type nonrec controller_case = fragments_factions7_item_controller_member0
  type nonrec cost = json_number
  type nonrec to_ = json_string
  type nonrec blade = json_number
  type nonrec pierce = json_number
  type nonrec impact = json_number
  type nonrec damage = json_number
  type nonrec spread = json_number
  type nonrec count = json_number
  type nonrec damage_type = fragments_native_state38_ground_items_item_item_attacks_item_damage_type
  type nonrec category = fragments_native_state38_ground_items_item_item_attacks_item_category
  type nonrec range = json_number
  type nonrec drain = json_number
  type nonrec amount = json_number
  type nonrec condition_case = fragments_rule11_condition_member0
  type nonrec kind = literal_human
  type nonrec policy = fragments_factions7_item_controller_member2_policy
  type nonrec unit_types = json_string
  type nonrec choice = json_string
  type nonrec flag = json_string
  type nonrec conditions = save2_checkpoint_state_tiles_item_fog_member1_condition
  type nonrec unit_type = json_string
end

module Javascript1 = struct
  type nonrec t = javascript1
  type nonrec version = literal_1
  type nonrec pending_enemy = json_boolean
  type nonrec campaign_deployment_format = literal_named_waves_v1
  type nonrec state = javascript1_state
  type nonrec map_id = json_string
  type nonrec map_revision = json_number
  type nonrec round = json_number
  type nonrec side = save35_setup_first
  type nonrec winner = fragments_native_state38_winner
  type nonrec gold = javascript1_state_gold
  type nonrec next_id = json_number
  type nonrec log = json_string
  type nonrec unit_ = javascript1_state_units_item
  type nonrec villages = javascript1_state_villages_item
  type nonrec sites = javascript1_state_villages_item
  type nonrec campaign = javascript1_state_campaign_member1
  type nonrec last_combat = fragments_native_state38_last_combat_member2
  type nonrec player = json_number
  type nonrec enemy = json_number
  type nonrec tile = json_string
  type nonrec id = json_string
  type nonrec name = json_string
  type nonrec role = fragments_native_state38_units_item_role
  type nonrec character = json_string
  type nonrec faction = json_string
  type nonrec items = javascript1_state_units_item_items_member1_item
  type nonrec ai_group = json_string
  type nonrec type_ = javascript1_state_units_item_type_
  type nonrec hp = json_number
  type nonrec moved = json_boolean
  type nonrec attacked = json_boolean
  type nonrec retreating = fragments_rule11_remember
  type nonrec rebuilding = fragments_rule11_remember
  type nonrec experience = json_number
  type nonrec owner = json_string
  type nonrec mission = json_number
  type nonrec recall = javascript1_state_units_item_type_
  type nonrec veteran = javascript1_state_campaign_member1_veterans_member1_item
  type nonrec story_veteran = javascript1_state_campaign_member1_veterans_member1_item
  type nonrec fallen_companions = json_string
  type nonrec carry_gold = json_number
  type nonrec mark = json_string
  type nonrec dialogue = javascript1_state_campaign_member1_dialogue_item
  type nonrec dialogue_index = json_number
  type nonrec outcome = json_string
  type nonrec completed = json_boolean
  type nonrec refuge = fragments_native_state38_campaign_member1_refuge_member1
  type nonrec vigil = fragments_native_state38_campaign_member1_vigil_member1
  type nonrec hold = fragments_native_state38_campaign_member1_holds_member1
  type nonrec choices = json_string
  type nonrec attacker_id = json_string
  type nonrec attack_id = json_string
  type nonrec sound = json_string
  type nonrec armor = fragments_native_state38_ground_items_item_item_armor_member1
  type nonrec description = json_string
  type nonrec healing = fragments_native_state38_ground_items_item_item_healing_member1
  type nonrec attacks = javascript1_state_units_item_items_member1_item_attacks_item
  type nonrec speaker = json_string
  type nonrec portrait = javascript1_state_campaign_member1_dialogue_item_portrait
  type nonrec text = json_string
  type nonrec held_turns = json_number
  type nonrec last_counted_round = json_number
  type nonrec escalated = json_boolean
  type nonrec blade = json_number
  type nonrec pierce = json_number
  type nonrec impact = json_number
  type nonrec amount = json_number
  type nonrec radius = json_number
  type nonrec spread = json_number
  type nonrec drain = json_number
  type nonrec count = json_number
  type nonrec damage = json_number
  type nonrec damage_type = fragments_native_state38_ground_items_item_item_attacks_item_damage_type
  type nonrec category = fragments_native_state38_ground_items_item_item_attacks_item_category
  type nonrec range = json_number
end

module V40 = struct
  type nonrec t = save40
  type nonrec t_case = save40_member0
  type nonrec pending_enemy = json_boolean
  type nonrec undo = save34_undo_member1_item
  type nonrec progress = save39_member0_progress
  type nonrec version = literal_40
  type nonrec checkpoint = save40_member0_checkpoint
  type nonrec setup = save39_member0_setup
  type nonrec round = json_number
  type nonrec unit_id = json_string
  type nonrec changes = save34_undo_member1_item_changes_item
  type nonrec entry = save37_progress_entry_member0
  type nonrec seed = json_number
  type nonrec state = save40_member0_checkpoint_state
  type nonrec first = save35_setup_first
  type nonrec enemy = save39_member0_setup_enemy
  type nonrec changes_case = save34_undo_member1_item_changes_item_member0
  type nonrec kind = literal_flash
  type nonrec id = json_string
  type nonrec saved_at = json_number
  type nonrec run = json_string
  type nonrec mission = json_string
  type nonrec side = json_string
  type nonrec mission_start = json_boolean
  type nonrec run_id = json_string
  type nonrec branch_id = json_string
  type nonrec checkpoint_id = json_string
  type nonrec ordinal = json_number
  type nonrec faction = save40_member0_checkpoint_state_factions_item
  type nonrec ground_items = save39_member0_checkpoint_state_ground_items_item
  type nonrec armies = fragments_native_state38_armies_member1
  type nonrec condition_memory = json_string
  type nonrec map_id = json_string
  type nonrec map_revision = json_number
  type nonrec cols = json_number
  type nonrec rows = json_number
  type nonrec gold = fragments_native_state38_gold
  type nonrec winner = fragments_native_state38_winner
  type nonrec next_id = json_number
  type nonrec combat_rng = json_number
  type nonrec log = json_string
  type nonrec tile = save40_member0_checkpoint_state_tiles_item
  type nonrec unit_ = save39_member0_checkpoint_state_units_item
  type nonrec campaign = save40_member0_checkpoint_state_campaign_member1
  type nonrec last_combat = fragments_native_state38_last_combat_member2
  type nonrec blocks = save35_setup_enemy_blocks_member1
  type nonrec policy = save35_setup_enemy_policy
  type nonrec diplomacy = save35_setup_enemy_diplomacy
  type nonrec recruitment_limit = json_number
  type nonrec manual = json_boolean
  type nonrec parent_id = json_string
  type nonrec op = literal_set
  type nonrec path = json_string
  type nonrec value = json_unknown
  type nonrec phase = save35_setup_first
  type nonrec controller = fragments_factions7_item_controller
  type nonrec palette = json_string
  type nonrec name = json_string
  type nonrec recruits = fragments_factions7_item_recruits_member1_item
  type nonrec relations = fragments_factions7_item_relations_member1
  type nonrec transfer = save40_member0_checkpoint_state_factions_item_transfer_member1
  type nonrec vision = save40_member0_checkpoint_state_factions_item_vision_member1
  type nonrec item = fragments_native_state38_ground_items_item_item
  type nonrec location = json_string
  type nonrec col = json_number
  type nonrec row = json_number
  type nonrec terrain = json_string
  type nonrec owner = json_string
  type nonrec connection = fragments_tiles11_item_connections_member1_item
  type nonrec dead_ends = fragments_tiles11_item_connections_member1_item
  type nonrec walls = fragments_tiles11_item_connections_member1_item
  type nonrec floor = json_string
  type nonrec fog = save40_member0_checkpoint_state_tiles_item_fog_member1
  type nonrec light = save40_member0_checkpoint_state_tiles_item_light_member1
  type nonrec items = fragments_native_state38_ground_items_item_item
  type nonrec ai_group = json_string
  type nonrec type_ = json_string
  type nonrec hp = json_number
  type nonrec moved = json_boolean
  type nonrec attacked = json_boolean
  type nonrec experience = json_number
  type nonrec character = json_string
  type nonrec role = fragments_native_state38_units_item_role
  type nonrec retreating = fragments_rule11_remember
  type nonrec rebuilding = fragments_rule11_remember
  type nonrec scripts = save40_member0_checkpoint_state_campaign_member1_scripts_member1
  type nonrec evacuated = save39_member0_checkpoint_state_campaign_member1_evacuated_member1_item
  type nonrec recall = json_string
  type nonrec veteran = fragments_native_state38_campaign_member1_veterans_member1_item
  type nonrec story_veteran = fragments_native_state38_campaign_member1_veterans_member1_item
  type nonrec fallen_companions = json_string
  type nonrec carry_gold = json_number
  type nonrec mark = json_string
  type nonrec dialogue = fragments_native_state38_campaign_member1_dialogue_item
  type nonrec dialogue_index = json_number
  type nonrec outcome = json_string
  type nonrec completed = json_boolean
  type nonrec refuge = fragments_native_state38_campaign_member1_refuge_member1
  type nonrec vigil = fragments_native_state38_campaign_member1_vigil_member1
  type nonrec hold = fragments_native_state38_campaign_member1_holds_member1
  type nonrec flags = fragments_native_state38_campaign_member1_flags
  type nonrec attacker_id = json_string
  type nonrec attack_id = json_string
  type nonrec sound = json_string
  type nonrec recruiter = save35_setup_enemy_blocks_member1_recruiter
  type nonrec planner = save35_setup_enemy_blocks_member1_planner
  type nonrec controller_case = fragments_factions7_item_controller_member0
  type nonrec cost = json_number
  type nonrec to_ = json_string
  type nonrec rule = save40_member0_checkpoint_state_tiles_item_fog_member1
  type nonrec explored = save39_member0_checkpoint_state_ground_items_item_tile
  type nonrec armor = fragments_native_state38_ground_items_item_item_armor_member1
  type nonrec description = json_string
  type nonrec attacks = fragments_native_state38_ground_items_item_item_attacks_item
  type nonrec healing = fragments_native_state38_ground_items_item_item_healing_member1
  type nonrec condition = save40_member0_checkpoint_state_tiles_item_fog_member1_condition
  type nonrec remember = fragments_rule11_remember
  type nonrec color = fragments_light23_color_member1
  type nonrec fixture = fragments_rule11_remember
  type nonrec dx = json_number
  type nonrec dy = json_number
  type nonrec radius = json_number
  type nonrec power = json_number
  type nonrec failed = json_string
  type nonrec revealed = json_string
  type nonrec recruitment = fragments_native_state38_campaign_member1_scripts_member1_recruitment
  type nonrec active = json_string

  type nonrec pending =
    save39_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item

  type nonrec next_mark = json_number
  type nonrec event = json_string
  type nonrec speaker = json_string
  type nonrec portrait = json_string
  type nonrec text = json_string
  type nonrec held_turns = json_number
  type nonrec last_counted_round = json_number
  type nonrec escalated = json_boolean
  type nonrec blade = json_number
  type nonrec pierce = json_number
  type nonrec impact = json_number
  type nonrec damage = json_number
  type nonrec spread = json_number
  type nonrec count = json_number
  type nonrec damage_type = fragments_native_state38_ground_items_item_item_attacks_item_damage_type
  type nonrec category = fragments_native_state38_ground_items_item_item_attacks_item_category
  type nonrec range = json_number
  type nonrec drain = json_number
  type nonrec amount = json_number

  type nonrec condition_case =
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0

  type nonrec predicate = json_number
  type nonrec turns = json_number
  type nonrec last_round = json_number
  type nonrec light_remembered = fragments_rule11_remember
  type nonrec area = save39_member0_checkpoint_state_ground_items_item_tile
  type nonrec unit_types = json_string
  type nonrec unit_type = json_string
  type nonrec choice = json_string
  type nonrec flag = json_string
  type nonrec conditions = save40_member0_checkpoint_state_tiles_item_fog_member1_condition

  type nonrec tile_case =
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member1

  type nonrec hex = save39_member0_checkpoint_state_ground_items_item_tile

  type nonrec binding =
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member3_binding

  type nonrec current = save40_member0
  type nonrec current_pending_enemy = json_boolean
  type nonrec current_undo = save34_undo_member1_item
  type nonrec current_progress = save39_member0_progress
  type nonrec current_version = literal_40
  type nonrec current_checkpoint = save40_member0_checkpoint
  type nonrec current_setup = save39_member0_setup
  type nonrec current_round = json_number
  type nonrec current_unit_id = json_string
  type nonrec current_changes = save34_undo_member1_item_changes_item
  type nonrec current_entry = save37_progress_entry_member0
  type nonrec current_seed = json_number
  type nonrec current_state = save40_member0_checkpoint_state
  type nonrec current_first = save35_setup_first
  type nonrec current_enemy = save39_member0_setup_enemy
  type nonrec current_changes_case = save34_undo_member1_item_changes_item_member0
  type nonrec current_kind = literal_flash
  type nonrec current_id = json_string
  type nonrec current_saved_at = json_number
  type nonrec current_run = json_string
  type nonrec current_mission = json_string
  type nonrec current_side = json_string
  type nonrec current_mission_start = json_boolean
  type nonrec current_run_id = json_string
  type nonrec current_branch_id = json_string
  type nonrec current_checkpoint_id = json_string
  type nonrec current_ordinal = json_number
  type nonrec current_faction = save40_member0_checkpoint_state_factions_item
  type nonrec current_ground_items = save39_member0_checkpoint_state_ground_items_item
  type nonrec current_armies = fragments_native_state38_armies_member1
  type nonrec current_condition_memory = json_string
  type nonrec current_map_id = json_string
  type nonrec current_map_revision = json_number
  type nonrec current_cols = json_number
  type nonrec current_rows = json_number
  type nonrec current_gold = fragments_native_state38_gold
  type nonrec current_winner = fragments_native_state38_winner
  type nonrec current_next_id = json_number
  type nonrec current_combat_rng = json_number
  type nonrec current_log = json_string
  type nonrec current_tile = save40_member0_checkpoint_state_tiles_item
  type nonrec current_unit_ = save39_member0_checkpoint_state_units_item
  type nonrec current_campaign = save40_member0_checkpoint_state_campaign_member1
  type nonrec current_last_combat = fragments_native_state38_last_combat_member2
  type nonrec current_blocks = save35_setup_enemy_blocks_member1
  type nonrec current_policy = save35_setup_enemy_policy
  type nonrec current_diplomacy = save35_setup_enemy_diplomacy
  type nonrec current_recruitment_limit = json_number
  type nonrec current_op = literal_set
  type nonrec current_path = json_string
  type nonrec current_value = json_unknown
  type nonrec current_phase = save35_setup_first
  type nonrec current_controller = fragments_factions7_item_controller
  type nonrec current_palette = json_string
  type nonrec current_name = json_string
  type nonrec current_recruits = fragments_factions7_item_recruits_member1_item
  type nonrec current_relations = fragments_factions7_item_relations_member1
  type nonrec current_transfer = save40_member0_checkpoint_state_factions_item_transfer_member1
  type nonrec current_vision = save40_member0_checkpoint_state_factions_item_vision_member1
  type nonrec current_item = fragments_native_state38_ground_items_item_item
  type nonrec current_location = json_string
  type nonrec current_col = json_number
  type nonrec current_row = json_number
  type nonrec current_terrain = json_string
  type nonrec current_owner = json_string
  type nonrec current_connection = fragments_tiles11_item_connections_member1_item
  type nonrec current_dead_ends = fragments_tiles11_item_connections_member1_item
  type nonrec current_walls = fragments_tiles11_item_connections_member1_item
  type nonrec current_floor = json_string
  type nonrec current_fog = save40_member0_checkpoint_state_tiles_item_fog_member1
  type nonrec current_light = save40_member0_checkpoint_state_tiles_item_light_member1
  type nonrec current_items = fragments_native_state38_ground_items_item_item
  type nonrec current_ai_group = json_string
  type nonrec current_type_ = json_string
  type nonrec current_hp = json_number
  type nonrec current_moved = json_boolean
  type nonrec current_attacked = json_boolean
  type nonrec current_experience = json_number
  type nonrec current_character = json_string
  type nonrec current_role = fragments_native_state38_units_item_role
  type nonrec current_retreating = fragments_rule11_remember
  type nonrec current_rebuilding = fragments_rule11_remember
  type nonrec current_scripts = save40_member0_checkpoint_state_campaign_member1_scripts_member1

  type nonrec current_evacuated =
    save39_member0_checkpoint_state_campaign_member1_evacuated_member1_item

  type nonrec current_recall = json_string
  type nonrec current_veteran = fragments_native_state38_campaign_member1_veterans_member1_item

  type nonrec current_story_veteran =
    fragments_native_state38_campaign_member1_veterans_member1_item

  type nonrec current_fallen_companions = json_string
  type nonrec current_carry_gold = json_number
  type nonrec current_mark = json_string
  type nonrec current_dialogue = fragments_native_state38_campaign_member1_dialogue_item
  type nonrec current_dialogue_index = json_number
  type nonrec current_outcome = json_string
  type nonrec current_completed = json_boolean
  type nonrec current_refuge = fragments_native_state38_campaign_member1_refuge_member1
  type nonrec current_vigil = fragments_native_state38_campaign_member1_vigil_member1
  type nonrec current_hold = fragments_native_state38_campaign_member1_holds_member1
  type nonrec current_flags = fragments_native_state38_campaign_member1_flags
  type nonrec current_attacker_id = json_string
  type nonrec current_attack_id = json_string
  type nonrec current_sound = json_string
  type nonrec current_recruiter = save35_setup_enemy_blocks_member1_recruiter
  type nonrec current_planner = save35_setup_enemy_blocks_member1_planner
  type nonrec current_controller_case = fragments_factions7_item_controller_member0
  type nonrec current_cost = json_number
  type nonrec current_to_ = json_string
  type nonrec current_rule = save40_member0_checkpoint_state_tiles_item_fog_member1
  type nonrec current_explored = save39_member0_checkpoint_state_ground_items_item_tile
  type nonrec current_armor = fragments_native_state38_ground_items_item_item_armor_member1
  type nonrec current_description = json_string
  type nonrec current_attacks = fragments_native_state38_ground_items_item_item_attacks_item
  type nonrec current_healing = fragments_native_state38_ground_items_item_item_healing_member1
  type nonrec current_condition = save40_member0_checkpoint_state_tiles_item_fog_member1_condition
  type nonrec current_remember = fragments_rule11_remember
  type nonrec current_color = fragments_light23_color_member1
  type nonrec current_fixture = fragments_rule11_remember
  type nonrec current_dx = json_number
  type nonrec current_dy = json_number
  type nonrec current_radius = json_number
  type nonrec current_power = json_number
  type nonrec current_failed = json_string
  type nonrec current_revealed = json_string

  type nonrec current_recruitment =
    fragments_native_state38_campaign_member1_scripts_member1_recruitment

  type nonrec current_active = json_string

  type nonrec current_pending =
    save39_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item

  type nonrec current_next_mark = json_number
  type nonrec current_event = json_string
  type nonrec current_speaker = json_string
  type nonrec current_portrait = json_string
  type nonrec current_text = json_string
  type nonrec current_held_turns = json_number
  type nonrec current_last_counted_round = json_number
  type nonrec current_escalated = json_boolean
  type nonrec current_blade = json_number
  type nonrec current_pierce = json_number
  type nonrec current_impact = json_number
  type nonrec current_damage = json_number
  type nonrec current_spread = json_number
  type nonrec current_count = json_number

  type nonrec current_damage_type =
    fragments_native_state38_ground_items_item_item_attacks_item_damage_type

  type nonrec current_category =
    fragments_native_state38_ground_items_item_item_attacks_item_category

  type nonrec current_range = json_number
  type nonrec current_drain = json_number
  type nonrec current_amount = json_number

  type nonrec current_condition_case =
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0

  type nonrec current_predicate = json_number
  type nonrec current_turns = json_number
  type nonrec current_last_round = json_number
  type nonrec current_light_remembered = fragments_rule11_remember
  type nonrec current_area = save39_member0_checkpoint_state_ground_items_item_tile
  type nonrec current_unit_types = json_string
  type nonrec current_unit_type = json_string
  type nonrec current_choice = json_string
  type nonrec current_flag = json_string
  type nonrec current_conditions = save40_member0_checkpoint_state_tiles_item_fog_member1_condition

  type nonrec current_tile_case =
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member1

  type nonrec current_hex = save39_member0_checkpoint_state_ground_items_item_tile

  type nonrec current_binding =
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member3_binding

  type nonrec turn = save40_member1
  type nonrec turn_pending_enemy = literal_false
  type nonrec turn_undo = json_undefined
  type nonrec turn_progress = save39_member1_progress
  type nonrec turn_version = literal_40
  type nonrec turn_checkpoint = save40_member0_checkpoint
  type nonrec turn_setup = save39_member0_setup
  type nonrec turn_entry = history4_entries_item
  type nonrec turn_seed = json_number
  type nonrec turn_state = save40_member0_checkpoint_state
  type nonrec turn_first = save35_setup_first
  type nonrec turn_enemy = save39_member0_setup_enemy
  type nonrec turn_kind = literal_turn
  type nonrec turn_manual = json_boolean
  type nonrec turn_parent_id = json_string
  type nonrec turn_id = json_string
  type nonrec turn_saved_at = json_number
  type nonrec turn_run = json_string
  type nonrec turn_mission = json_string
  type nonrec turn_round = json_number
  type nonrec turn_side = json_string
  type nonrec turn_mission_start = json_boolean
  type nonrec turn_run_id = json_string
  type nonrec turn_branch_id = json_string
  type nonrec turn_checkpoint_id = json_string
  type nonrec turn_ordinal = json_number
  type nonrec turn_faction = save40_member0_checkpoint_state_factions_item
  type nonrec turn_ground_items = save39_member0_checkpoint_state_ground_items_item
  type nonrec turn_armies = fragments_native_state38_armies_member1
  type nonrec turn_condition_memory = json_string
  type nonrec turn_map_id = json_string
  type nonrec turn_map_revision = json_number
  type nonrec turn_cols = json_number
  type nonrec turn_rows = json_number
  type nonrec turn_gold = fragments_native_state38_gold
  type nonrec turn_winner = fragments_native_state38_winner
  type nonrec turn_next_id = json_number
  type nonrec turn_combat_rng = json_number
  type nonrec turn_log = json_string
  type nonrec turn_tile = save40_member0_checkpoint_state_tiles_item
  type nonrec turn_unit_ = save39_member0_checkpoint_state_units_item
  type nonrec turn_campaign = save40_member0_checkpoint_state_campaign_member1
  type nonrec turn_last_combat = fragments_native_state38_last_combat_member2
  type nonrec turn_blocks = save35_setup_enemy_blocks_member1
  type nonrec turn_policy = save35_setup_enemy_policy
  type nonrec turn_diplomacy = save35_setup_enemy_diplomacy
  type nonrec turn_recruitment_limit = json_number
  type nonrec turn_phase = save35_setup_first
  type nonrec turn_controller = fragments_factions7_item_controller
  type nonrec turn_palette = json_string
  type nonrec turn_name = json_string
  type nonrec turn_recruits = fragments_factions7_item_recruits_member1_item
  type nonrec turn_relations = fragments_factions7_item_relations_member1
  type nonrec turn_transfer = save40_member0_checkpoint_state_factions_item_transfer_member1
  type nonrec turn_vision = save40_member0_checkpoint_state_factions_item_vision_member1
  type nonrec turn_item = fragments_native_state38_ground_items_item_item
  type nonrec turn_location = json_string
  type nonrec turn_col = json_number
  type nonrec turn_row = json_number
  type nonrec turn_terrain = json_string
  type nonrec turn_owner = json_string
  type nonrec turn_connection = fragments_tiles11_item_connections_member1_item
  type nonrec turn_dead_ends = fragments_tiles11_item_connections_member1_item
  type nonrec turn_walls = fragments_tiles11_item_connections_member1_item
  type nonrec turn_floor = json_string
  type nonrec turn_fog = save40_member0_checkpoint_state_tiles_item_fog_member1
  type nonrec turn_light = save40_member0_checkpoint_state_tiles_item_light_member1
  type nonrec turn_items = fragments_native_state38_ground_items_item_item
  type nonrec turn_ai_group = json_string
  type nonrec turn_type_ = json_string
  type nonrec turn_hp = json_number
  type nonrec turn_moved = json_boolean
  type nonrec turn_attacked = json_boolean
  type nonrec turn_experience = json_number
  type nonrec turn_character = json_string
  type nonrec turn_role = fragments_native_state38_units_item_role
  type nonrec turn_retreating = fragments_rule11_remember
  type nonrec turn_rebuilding = fragments_rule11_remember
  type nonrec turn_scripts = save40_member0_checkpoint_state_campaign_member1_scripts_member1

  type nonrec turn_evacuated =
    save39_member0_checkpoint_state_campaign_member1_evacuated_member1_item

  type nonrec turn_recall = json_string
  type nonrec turn_veteran = fragments_native_state38_campaign_member1_veterans_member1_item
  type nonrec turn_story_veteran = fragments_native_state38_campaign_member1_veterans_member1_item
  type nonrec turn_fallen_companions = json_string
  type nonrec turn_carry_gold = json_number
  type nonrec turn_mark = json_string
  type nonrec turn_dialogue = fragments_native_state38_campaign_member1_dialogue_item
  type nonrec turn_dialogue_index = json_number
  type nonrec turn_outcome = json_string
  type nonrec turn_completed = json_boolean
  type nonrec turn_refuge = fragments_native_state38_campaign_member1_refuge_member1
  type nonrec turn_vigil = fragments_native_state38_campaign_member1_vigil_member1
  type nonrec turn_hold = fragments_native_state38_campaign_member1_holds_member1
  type nonrec turn_flags = fragments_native_state38_campaign_member1_flags
  type nonrec turn_attacker_id = json_string
  type nonrec turn_attack_id = json_string
  type nonrec turn_sound = json_string
  type nonrec turn_recruiter = save35_setup_enemy_blocks_member1_recruiter
  type nonrec turn_planner = save35_setup_enemy_blocks_member1_planner
  type nonrec turn_controller_case = fragments_factions7_item_controller_member0
  type nonrec turn_cost = json_number
  type nonrec turn_to_ = json_string
  type nonrec turn_rule = save40_member0_checkpoint_state_tiles_item_fog_member1
  type nonrec turn_explored = save39_member0_checkpoint_state_ground_items_item_tile
  type nonrec turn_armor = fragments_native_state38_ground_items_item_item_armor_member1
  type nonrec turn_description = json_string
  type nonrec turn_attacks = fragments_native_state38_ground_items_item_item_attacks_item
  type nonrec turn_healing = fragments_native_state38_ground_items_item_item_healing_member1
  type nonrec turn_condition = save40_member0_checkpoint_state_tiles_item_fog_member1_condition
  type nonrec turn_remember = fragments_rule11_remember
  type nonrec turn_color = fragments_light23_color_member1
  type nonrec turn_fixture = fragments_rule11_remember
  type nonrec turn_dx = json_number
  type nonrec turn_dy = json_number
  type nonrec turn_radius = json_number
  type nonrec turn_power = json_number
  type nonrec turn_failed = json_string
  type nonrec turn_revealed = json_string

  type nonrec turn_recruitment =
    fragments_native_state38_campaign_member1_scripts_member1_recruitment

  type nonrec turn_active = json_string

  type nonrec turn_pending =
    save39_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item

  type nonrec turn_next_mark = json_number
  type nonrec turn_event = json_string
  type nonrec turn_speaker = json_string
  type nonrec turn_portrait = json_string
  type nonrec turn_text = json_string
  type nonrec turn_held_turns = json_number
  type nonrec turn_last_counted_round = json_number
  type nonrec turn_escalated = json_boolean
  type nonrec turn_blade = json_number
  type nonrec turn_pierce = json_number
  type nonrec turn_impact = json_number
  type nonrec turn_damage = json_number
  type nonrec turn_spread = json_number
  type nonrec turn_count = json_number

  type nonrec turn_damage_type =
    fragments_native_state38_ground_items_item_item_attacks_item_damage_type

  type nonrec turn_category = fragments_native_state38_ground_items_item_item_attacks_item_category
  type nonrec turn_range = json_number
  type nonrec turn_drain = json_number
  type nonrec turn_amount = json_number

  type nonrec turn_condition_case =
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0

  type nonrec turn_predicate = json_number
  type nonrec turn_turns = json_number
  type nonrec turn_last_round = json_number
  type nonrec turn_light_remembered = fragments_rule11_remember
  type nonrec turn_area = save39_member0_checkpoint_state_ground_items_item_tile
  type nonrec turn_unit_types = json_string
  type nonrec turn_unit_type = json_string
  type nonrec turn_choice = json_string
  type nonrec turn_flag = json_string
  type nonrec turn_conditions = save40_member0_checkpoint_state_tiles_item_fog_member1_condition

  type nonrec turn_tile_case =
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member1

  type nonrec turn_hex = save39_member0_checkpoint_state_ground_items_item_tile

  type nonrec turn_binding =
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member3_binding
end

module V41 = struct
  type nonrec t = save41
  type nonrec t_case = save41_member0
  type nonrec pending_enemy = json_boolean
  type nonrec undo = save34_undo_member1_item
  type nonrec progress = save39_member0_progress
  type nonrec version = literal_41
  type nonrec checkpoint = save41_member0_checkpoint
  type nonrec setup = save41_member0_setup
  type nonrec round = json_number
  type nonrec unit_id = json_string
  type nonrec changes = save34_undo_member1_item_changes_item
  type nonrec entry = save37_progress_entry_member0
  type nonrec seed = json_number
  type nonrec state = save41_member0_checkpoint_state
  type nonrec first = save41_member0_setup_first
  type nonrec enemy = save39_member0_setup_enemy
  type nonrec changes_case = save34_undo_member1_item_changes_item_member0
  type nonrec kind = literal_flash
  type nonrec id = json_string
  type nonrec saved_at = json_number
  type nonrec run = json_string
  type nonrec mission = json_string
  type nonrec side = json_string
  type nonrec mission_start = json_boolean
  type nonrec run_id = json_string
  type nonrec branch_id = json_string
  type nonrec checkpoint_id = json_string
  type nonrec ordinal = json_number
  type nonrec vision_groups = save41_member0_checkpoint_state_vision_groups_item
  type nonrec faction = save41_member0_checkpoint_state_factions_item
  type nonrec ground_items = save41_member0_checkpoint_state_ground_items_item
  type nonrec armies = fragments_native_state38_armies_member1
  type nonrec condition_memory = json_string
  type nonrec map_id = json_string
  type nonrec map_revision = json_number
  type nonrec cols = json_number
  type nonrec rows = json_number
  type nonrec gold = fragments_native_state38_gold
  type nonrec winner = save41_member0_checkpoint_state_winner
  type nonrec next_id = json_number
  type nonrec combat_rng = json_number
  type nonrec log = json_string
  type nonrec tile = save41_member0_checkpoint_state_tiles_item
  type nonrec unit_ = save41_member0_checkpoint_state_units_item
  type nonrec campaign = save41_member0_checkpoint_state_campaign_member1
  type nonrec last_combat = fragments_native_state38_last_combat_member2
  type nonrec blocks = save35_setup_enemy_blocks_member1
  type nonrec policy = save35_setup_enemy_policy
  type nonrec diplomacy = save35_setup_enemy_diplomacy
  type nonrec recruitment_limit = json_number
  type nonrec manual = json_boolean
  type nonrec parent_id = json_string
  type nonrec op = literal_set
  type nonrec path = json_string
  type nonrec value = json_unknown
  type nonrec sight = save41_member0_checkpoint_state_vision_groups_item_sight
  type nonrec explored = save39_member0_checkpoint_state_ground_items_item_tile
  type nonrec vision_group = json_string
  type nonrec phase = save41_member0_setup_first
  type nonrec controller = fragments_factions7_item_controller
  type nonrec palette = json_string
  type nonrec name = json_string
  type nonrec recruits = fragments_factions7_item_recruits_member1_item
  type nonrec relations = save41_member0_checkpoint_state_factions_item_relations_member1
  type nonrec transfer = save41_member0_checkpoint_state_factions_item_transfer_member1
  type nonrec item = save41_member0_checkpoint_state_ground_items_item_item
  type nonrec location = json_string
  type nonrec col = json_number
  type nonrec row = json_number
  type nonrec terrain = json_string
  type nonrec owner = json_string
  type nonrec connection = fragments_tiles11_item_connections_member1_item
  type nonrec dead_ends = fragments_tiles11_item_connections_member1_item
  type nonrec walls = fragments_tiles11_item_connections_member1_item
  type nonrec floor = json_string
  type nonrec perception = save41_member0_checkpoint_state_tiles_item_perception
  type nonrec light = save41_member0_checkpoint_state_tiles_item_light_member1
  type nonrec revealed_to = json_string

  type nonrec perception_effects =
    save41_member0_checkpoint_state_units_item_perception_effects_item

  type nonrec items = save41_member0_checkpoint_state_ground_items_item_item
  type nonrec ai_group = json_string
  type nonrec type_ = json_string
  type nonrec hp = json_number
  type nonrec moved = json_boolean
  type nonrec attack_spent = json_boolean
  type nonrec experience = json_number
  type nonrec character = json_string
  type nonrec role = fragments_native_state38_units_item_role
  type nonrec retreating = json_boolean
  type nonrec rebuilding = json_boolean
  type nonrec scripts = save41_member0_checkpoint_state_campaign_member1_scripts_member1
  type nonrec evacuated = save41_member0_checkpoint_state_campaign_member1_evacuated_member1_item
  type nonrec recall = json_string
  type nonrec veteran = save41_member0_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec story_veteran = save41_member0_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec fallen_companions = json_string
  type nonrec carry_gold = json_number
  type nonrec mark = json_string
  type nonrec dialogue = save41_member0_checkpoint_state_campaign_member1_dialogue_item
  type nonrec dialogue_index = json_number
  type nonrec outcome = json_string
  type nonrec completed = json_boolean
  type nonrec refuge = fragments_native_state38_campaign_member1_refuge_member1
  type nonrec vigil = fragments_native_state38_campaign_member1_vigil_member1
  type nonrec hold = fragments_native_state38_campaign_member1_holds_member1
  type nonrec flags = fragments_native_state38_campaign_member1_flags
  type nonrec attacker_id = json_string
  type nonrec attack_id = json_string
  type nonrec sound = json_string
  type nonrec recruiter = save35_setup_enemy_blocks_member1_recruiter
  type nonrec planner = save35_setup_enemy_blocks_member1_planner
  type nonrec range = save41_member0_checkpoint_state_vision_groups_item_sight_range
  type nonrec penetrates = save41_member0_checkpoint_state_vision_groups_item_sight_penetrates_item
  type nonrec controller_case = fragments_factions7_item_controller_member0
  type nonrec cost = json_number
  type nonrec to_ = json_string
  type nonrec rule = save41_member0_checkpoint_state_factions_item_transfer_member1_when_
  type nonrec armor = fragments_native_state38_ground_items_item_item_armor_member1
  type nonrec description = json_string
  type nonrec attacks = save41_member0_checkpoint_state_ground_items_item_item_attacks_item
  type nonrec healing = fragments_native_state38_ground_items_item_item_healing_member1
  type nonrec remember = json_boolean
  type nonrec concealment = save41_member0_checkpoint_state_tiles_item_perception_concealment_item
  type nonrec emission = save41_member0_checkpoint_state_tiles_item_light_member1_emission_member1
  type nonrec color = fragments_light23_color_member1
  type nonrec fixture = json_boolean
  type nonrec dx = json_number
  type nonrec dy = json_number
  type nonrec radius = json_number
  type nonrec power = json_number

  type nonrec expires_at =
    save41_member0_checkpoint_state_units_item_perception_effects_item_expires_at_member1

  type nonrec failed = json_string
  type nonrec revealed = json_string
  type nonrec recruitment = fragments_native_state38_campaign_member1_scripts_member1_recruitment
  type nonrec active = json_string

  type nonrec pending =
    save41_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item

  type nonrec next_mark = json_number
  type nonrec event = json_string
  type nonrec speaker = json_string
  type nonrec portrait = json_string
  type nonrec text = json_string
  type nonrec held_turns = json_number
  type nonrec last_counted_round = json_number
  type nonrec escalated = json_boolean
  type nonrec condition = save40_member0_checkpoint_state_tiles_item_fog_member1_condition
  type nonrec blade = json_number
  type nonrec pierce = json_number
  type nonrec impact = json_number
  type nonrec damage = json_number
  type nonrec spread = json_number
  type nonrec count = json_number
  type nonrec damage_type = fragments_native_state38_ground_items_item_item_attacks_item_damage_type
  type nonrec category = fragments_native_state38_ground_items_item_item_attacks_item_category
  type nonrec drain = json_number
  type nonrec amount = json_number
  type nonrec group = json_string
  type nonrec profile = save41_member0_checkpoint_state_vision_groups_item_sight
  type nonrec reveal_within = json_number

  type nonrec terrains =
    save41_member0_checkpoint_state_units_item_perception_effects_item_concealment_item_terrains_member1_item

  type nonrec predicate = json_number
  type nonrec turns = json_number
  type nonrec last_round = json_number
  type nonrec light_remembered = json_boolean
  type nonrec area = save39_member0_checkpoint_state_ground_items_item_tile

  type nonrec condition_case =
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0

  type nonrec unit_types = json_string
  type nonrec unit_type = json_string
  type nonrec choice = json_string
  type nonrec flag = json_string
  type nonrec conditions = save40_member0_checkpoint_state_tiles_item_fog_member1_condition

  type nonrec tile_case =
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member1

  type nonrec hex = save39_member0_checkpoint_state_ground_items_item_tile

  type nonrec binding =
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member3_binding

  type nonrec current = save41_member0
  type nonrec current_pending_enemy = json_boolean
  type nonrec current_undo = save34_undo_member1_item
  type nonrec current_progress = save39_member0_progress
  type nonrec current_version = literal_41
  type nonrec current_checkpoint = save41_member0_checkpoint
  type nonrec current_setup = save41_member0_setup
  type nonrec current_round = json_number
  type nonrec current_unit_id = json_string
  type nonrec current_changes = save34_undo_member1_item_changes_item
  type nonrec current_entry = save37_progress_entry_member0
  type nonrec current_seed = json_number
  type nonrec current_state = save41_member0_checkpoint_state
  type nonrec current_first = save41_member0_setup_first
  type nonrec current_enemy = save39_member0_setup_enemy
  type nonrec current_changes_case = save34_undo_member1_item_changes_item_member0
  type nonrec current_kind = literal_flash
  type nonrec current_id = json_string
  type nonrec current_saved_at = json_number
  type nonrec current_run = json_string
  type nonrec current_mission = json_string
  type nonrec current_side = json_string
  type nonrec current_mission_start = json_boolean
  type nonrec current_run_id = json_string
  type nonrec current_branch_id = json_string
  type nonrec current_checkpoint_id = json_string
  type nonrec current_ordinal = json_number
  type nonrec current_vision_groups = save41_member0_checkpoint_state_vision_groups_item
  type nonrec current_faction = save41_member0_checkpoint_state_factions_item
  type nonrec current_ground_items = save41_member0_checkpoint_state_ground_items_item
  type nonrec current_armies = fragments_native_state38_armies_member1
  type nonrec current_condition_memory = json_string
  type nonrec current_map_id = json_string
  type nonrec current_map_revision = json_number
  type nonrec current_cols = json_number
  type nonrec current_rows = json_number
  type nonrec current_gold = fragments_native_state38_gold
  type nonrec current_winner = save41_member0_checkpoint_state_winner
  type nonrec current_next_id = json_number
  type nonrec current_combat_rng = json_number
  type nonrec current_log = json_string
  type nonrec current_tile = save41_member0_checkpoint_state_tiles_item
  type nonrec current_unit_ = save41_member0_checkpoint_state_units_item
  type nonrec current_campaign = save41_member0_checkpoint_state_campaign_member1
  type nonrec current_last_combat = fragments_native_state38_last_combat_member2
  type nonrec current_blocks = save35_setup_enemy_blocks_member1
  type nonrec current_policy = save35_setup_enemy_policy
  type nonrec current_diplomacy = save35_setup_enemy_diplomacy
  type nonrec current_recruitment_limit = json_number
  type nonrec current_op = literal_set
  type nonrec current_path = json_string
  type nonrec current_value = json_unknown
  type nonrec current_sight = save41_member0_checkpoint_state_vision_groups_item_sight
  type nonrec current_explored = save39_member0_checkpoint_state_ground_items_item_tile
  type nonrec current_vision_group = json_string
  type nonrec current_phase = save41_member0_setup_first
  type nonrec current_controller = fragments_factions7_item_controller
  type nonrec current_palette = json_string
  type nonrec current_name = json_string
  type nonrec current_recruits = fragments_factions7_item_recruits_member1_item
  type nonrec current_relations = save41_member0_checkpoint_state_factions_item_relations_member1
  type nonrec current_transfer = save41_member0_checkpoint_state_factions_item_transfer_member1
  type nonrec current_item = save41_member0_checkpoint_state_ground_items_item_item
  type nonrec current_location = json_string
  type nonrec current_col = json_number
  type nonrec current_row = json_number
  type nonrec current_terrain = json_string
  type nonrec current_owner = json_string
  type nonrec current_connection = fragments_tiles11_item_connections_member1_item
  type nonrec current_dead_ends = fragments_tiles11_item_connections_member1_item
  type nonrec current_walls = fragments_tiles11_item_connections_member1_item
  type nonrec current_floor = json_string
  type nonrec current_perception = save41_member0_checkpoint_state_tiles_item_perception
  type nonrec current_light = save41_member0_checkpoint_state_tiles_item_light_member1
  type nonrec current_revealed_to = json_string

  type nonrec current_perception_effects =
    save41_member0_checkpoint_state_units_item_perception_effects_item

  type nonrec current_items = save41_member0_checkpoint_state_ground_items_item_item
  type nonrec current_ai_group = json_string
  type nonrec current_type_ = json_string
  type nonrec current_hp = json_number
  type nonrec current_moved = json_boolean
  type nonrec current_attack_spent = json_boolean
  type nonrec current_experience = json_number
  type nonrec current_character = json_string
  type nonrec current_role = fragments_native_state38_units_item_role
  type nonrec current_retreating = json_boolean
  type nonrec current_rebuilding = json_boolean
  type nonrec current_scripts = save41_member0_checkpoint_state_campaign_member1_scripts_member1

  type nonrec current_evacuated =
    save41_member0_checkpoint_state_campaign_member1_evacuated_member1_item

  type nonrec current_recall = json_string

  type nonrec current_veteran =
    save41_member0_checkpoint_state_campaign_member1_veterans_member1_item

  type nonrec current_story_veteran =
    save41_member0_checkpoint_state_campaign_member1_veterans_member1_item

  type nonrec current_fallen_companions = json_string
  type nonrec current_carry_gold = json_number
  type nonrec current_mark = json_string
  type nonrec current_dialogue = save41_member0_checkpoint_state_campaign_member1_dialogue_item
  type nonrec current_dialogue_index = json_number
  type nonrec current_outcome = json_string
  type nonrec current_completed = json_boolean
  type nonrec current_refuge = fragments_native_state38_campaign_member1_refuge_member1
  type nonrec current_vigil = fragments_native_state38_campaign_member1_vigil_member1
  type nonrec current_hold = fragments_native_state38_campaign_member1_holds_member1
  type nonrec current_flags = fragments_native_state38_campaign_member1_flags
  type nonrec current_attacker_id = json_string
  type nonrec current_attack_id = json_string
  type nonrec current_sound = json_string
  type nonrec current_recruiter = save35_setup_enemy_blocks_member1_recruiter
  type nonrec current_planner = save35_setup_enemy_blocks_member1_planner
  type nonrec current_range = save41_member0_checkpoint_state_vision_groups_item_sight_range

  type nonrec current_penetrates =
    save41_member0_checkpoint_state_vision_groups_item_sight_penetrates_item

  type nonrec current_controller_case = fragments_factions7_item_controller_member0
  type nonrec current_cost = json_number
  type nonrec current_to_ = json_string
  type nonrec current_rule = save41_member0_checkpoint_state_factions_item_transfer_member1_when_
  type nonrec current_armor = fragments_native_state38_ground_items_item_item_armor_member1
  type nonrec current_description = json_string
  type nonrec current_attacks = save41_member0_checkpoint_state_ground_items_item_item_attacks_item
  type nonrec current_healing = fragments_native_state38_ground_items_item_item_healing_member1
  type nonrec current_remember = json_boolean

  type nonrec current_concealment =
    save41_member0_checkpoint_state_tiles_item_perception_concealment_item

  type nonrec current_emission =
    save41_member0_checkpoint_state_tiles_item_light_member1_emission_member1

  type nonrec current_color = fragments_light23_color_member1
  type nonrec current_fixture = json_boolean
  type nonrec current_dx = json_number
  type nonrec current_dy = json_number
  type nonrec current_radius = json_number
  type nonrec current_power = json_number

  type nonrec current_expires_at =
    save41_member0_checkpoint_state_units_item_perception_effects_item_expires_at_member1

  type nonrec current_failed = json_string
  type nonrec current_revealed = json_string

  type nonrec current_recruitment =
    fragments_native_state38_campaign_member1_scripts_member1_recruitment

  type nonrec current_active = json_string

  type nonrec current_pending =
    save41_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item

  type nonrec current_next_mark = json_number
  type nonrec current_event = json_string
  type nonrec current_speaker = json_string
  type nonrec current_portrait = json_string
  type nonrec current_text = json_string
  type nonrec current_held_turns = json_number
  type nonrec current_last_counted_round = json_number
  type nonrec current_escalated = json_boolean
  type nonrec current_condition = save40_member0_checkpoint_state_tiles_item_fog_member1_condition
  type nonrec current_blade = json_number
  type nonrec current_pierce = json_number
  type nonrec current_impact = json_number
  type nonrec current_damage = json_number
  type nonrec current_spread = json_number
  type nonrec current_count = json_number

  type nonrec current_damage_type =
    fragments_native_state38_ground_items_item_item_attacks_item_damage_type

  type nonrec current_category =
    fragments_native_state38_ground_items_item_item_attacks_item_category

  type nonrec current_drain = json_number
  type nonrec current_amount = json_number
  type nonrec current_group = json_string
  type nonrec current_profile = save41_member0_checkpoint_state_vision_groups_item_sight
  type nonrec current_reveal_within = json_number

  type nonrec current_terrains =
    save41_member0_checkpoint_state_units_item_perception_effects_item_concealment_item_terrains_member1_item

  type nonrec current_predicate = json_number
  type nonrec current_turns = json_number
  type nonrec current_last_round = json_number
  type nonrec current_light_remembered = json_boolean
  type nonrec current_area = save39_member0_checkpoint_state_ground_items_item_tile

  type nonrec current_condition_case =
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0

  type nonrec current_unit_types = json_string
  type nonrec current_unit_type = json_string
  type nonrec current_choice = json_string
  type nonrec current_flag = json_string
  type nonrec current_conditions = save40_member0_checkpoint_state_tiles_item_fog_member1_condition

  type nonrec current_tile_case =
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member1

  type nonrec current_hex = save39_member0_checkpoint_state_ground_items_item_tile

  type nonrec current_binding =
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member3_binding

  type nonrec turn = save41_member1
  type nonrec turn_pending_enemy = literal_false
  type nonrec turn_undo = json_undefined
  type nonrec turn_progress = save39_member1_progress
  type nonrec turn_version = literal_41
  type nonrec turn_checkpoint = save41_member0_checkpoint
  type nonrec turn_setup = save41_member0_setup
  type nonrec turn_entry = history4_entries_item
  type nonrec turn_seed = json_number
  type nonrec turn_state = save41_member0_checkpoint_state
  type nonrec turn_first = save41_member0_setup_first
  type nonrec turn_enemy = save39_member0_setup_enemy
  type nonrec turn_kind = literal_turn
  type nonrec turn_manual = json_boolean
  type nonrec turn_parent_id = json_string
  type nonrec turn_id = json_string
  type nonrec turn_saved_at = json_number
  type nonrec turn_run = json_string
  type nonrec turn_mission = json_string
  type nonrec turn_round = json_number
  type nonrec turn_side = json_string
  type nonrec turn_mission_start = json_boolean
  type nonrec turn_run_id = json_string
  type nonrec turn_branch_id = json_string
  type nonrec turn_checkpoint_id = json_string
  type nonrec turn_ordinal = json_number
  type nonrec turn_vision_groups = save41_member0_checkpoint_state_vision_groups_item
  type nonrec turn_faction = save41_member0_checkpoint_state_factions_item
  type nonrec turn_ground_items = save41_member0_checkpoint_state_ground_items_item
  type nonrec turn_armies = fragments_native_state38_armies_member1
  type nonrec turn_condition_memory = json_string
  type nonrec turn_map_id = json_string
  type nonrec turn_map_revision = json_number
  type nonrec turn_cols = json_number
  type nonrec turn_rows = json_number
  type nonrec turn_gold = fragments_native_state38_gold
  type nonrec turn_winner = save41_member0_checkpoint_state_winner
  type nonrec turn_next_id = json_number
  type nonrec turn_combat_rng = json_number
  type nonrec turn_log = json_string
  type nonrec turn_tile = save41_member0_checkpoint_state_tiles_item
  type nonrec turn_unit_ = save41_member0_checkpoint_state_units_item
  type nonrec turn_campaign = save41_member0_checkpoint_state_campaign_member1
  type nonrec turn_last_combat = fragments_native_state38_last_combat_member2
  type nonrec turn_blocks = save35_setup_enemy_blocks_member1
  type nonrec turn_policy = save35_setup_enemy_policy
  type nonrec turn_diplomacy = save35_setup_enemy_diplomacy
  type nonrec turn_recruitment_limit = json_number
  type nonrec turn_sight = save41_member0_checkpoint_state_vision_groups_item_sight
  type nonrec turn_explored = save39_member0_checkpoint_state_ground_items_item_tile
  type nonrec turn_vision_group = json_string
  type nonrec turn_phase = save41_member0_setup_first
  type nonrec turn_controller = fragments_factions7_item_controller
  type nonrec turn_palette = json_string
  type nonrec turn_name = json_string
  type nonrec turn_recruits = fragments_factions7_item_recruits_member1_item
  type nonrec turn_relations = save41_member0_checkpoint_state_factions_item_relations_member1
  type nonrec turn_transfer = save41_member0_checkpoint_state_factions_item_transfer_member1
  type nonrec turn_item = save41_member0_checkpoint_state_ground_items_item_item
  type nonrec turn_location = json_string
  type nonrec turn_col = json_number
  type nonrec turn_row = json_number
  type nonrec turn_terrain = json_string
  type nonrec turn_owner = json_string
  type nonrec turn_connection = fragments_tiles11_item_connections_member1_item
  type nonrec turn_dead_ends = fragments_tiles11_item_connections_member1_item
  type nonrec turn_walls = fragments_tiles11_item_connections_member1_item
  type nonrec turn_floor = json_string
  type nonrec turn_perception = save41_member0_checkpoint_state_tiles_item_perception
  type nonrec turn_light = save41_member0_checkpoint_state_tiles_item_light_member1
  type nonrec turn_revealed_to = json_string

  type nonrec turn_perception_effects =
    save41_member0_checkpoint_state_units_item_perception_effects_item

  type nonrec turn_items = save41_member0_checkpoint_state_ground_items_item_item
  type nonrec turn_ai_group = json_string
  type nonrec turn_type_ = json_string
  type nonrec turn_hp = json_number
  type nonrec turn_moved = json_boolean
  type nonrec turn_attack_spent = json_boolean
  type nonrec turn_experience = json_number
  type nonrec turn_character = json_string
  type nonrec turn_role = fragments_native_state38_units_item_role
  type nonrec turn_retreating = json_boolean
  type nonrec turn_rebuilding = json_boolean
  type nonrec turn_scripts = save41_member0_checkpoint_state_campaign_member1_scripts_member1

  type nonrec turn_evacuated =
    save41_member0_checkpoint_state_campaign_member1_evacuated_member1_item

  type nonrec turn_recall = json_string
  type nonrec turn_veteran = save41_member0_checkpoint_state_campaign_member1_veterans_member1_item

  type nonrec turn_story_veteran =
    save41_member0_checkpoint_state_campaign_member1_veterans_member1_item

  type nonrec turn_fallen_companions = json_string
  type nonrec turn_carry_gold = json_number
  type nonrec turn_mark = json_string
  type nonrec turn_dialogue = save41_member0_checkpoint_state_campaign_member1_dialogue_item
  type nonrec turn_dialogue_index = json_number
  type nonrec turn_outcome = json_string
  type nonrec turn_completed = json_boolean
  type nonrec turn_refuge = fragments_native_state38_campaign_member1_refuge_member1
  type nonrec turn_vigil = fragments_native_state38_campaign_member1_vigil_member1
  type nonrec turn_hold = fragments_native_state38_campaign_member1_holds_member1
  type nonrec turn_flags = fragments_native_state38_campaign_member1_flags
  type nonrec turn_attacker_id = json_string
  type nonrec turn_attack_id = json_string
  type nonrec turn_sound = json_string
  type nonrec turn_recruiter = save35_setup_enemy_blocks_member1_recruiter
  type nonrec turn_planner = save35_setup_enemy_blocks_member1_planner
  type nonrec turn_range = save41_member0_checkpoint_state_vision_groups_item_sight_range

  type nonrec turn_penetrates =
    save41_member0_checkpoint_state_vision_groups_item_sight_penetrates_item

  type nonrec turn_controller_case = fragments_factions7_item_controller_member0
  type nonrec turn_cost = json_number
  type nonrec turn_to_ = json_string
  type nonrec turn_rule = save41_member0_checkpoint_state_factions_item_transfer_member1_when_
  type nonrec turn_armor = fragments_native_state38_ground_items_item_item_armor_member1
  type nonrec turn_description = json_string
  type nonrec turn_attacks = save41_member0_checkpoint_state_ground_items_item_item_attacks_item
  type nonrec turn_healing = fragments_native_state38_ground_items_item_item_healing_member1
  type nonrec turn_remember = json_boolean

  type nonrec turn_concealment =
    save41_member0_checkpoint_state_tiles_item_perception_concealment_item

  type nonrec turn_emission =
    save41_member0_checkpoint_state_tiles_item_light_member1_emission_member1

  type nonrec turn_color = fragments_light23_color_member1
  type nonrec turn_fixture = json_boolean
  type nonrec turn_dx = json_number
  type nonrec turn_dy = json_number
  type nonrec turn_radius = json_number
  type nonrec turn_power = json_number

  type nonrec turn_expires_at =
    save41_member0_checkpoint_state_units_item_perception_effects_item_expires_at_member1

  type nonrec turn_failed = json_string
  type nonrec turn_revealed = json_string

  type nonrec turn_recruitment =
    fragments_native_state38_campaign_member1_scripts_member1_recruitment

  type nonrec turn_active = json_string

  type nonrec turn_pending =
    save41_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item

  type nonrec turn_next_mark = json_number
  type nonrec turn_event = json_string
  type nonrec turn_speaker = json_string
  type nonrec turn_portrait = json_string
  type nonrec turn_text = json_string
  type nonrec turn_held_turns = json_number
  type nonrec turn_last_counted_round = json_number
  type nonrec turn_escalated = json_boolean
  type nonrec turn_condition = save40_member0_checkpoint_state_tiles_item_fog_member1_condition
  type nonrec turn_blade = json_number
  type nonrec turn_pierce = json_number
  type nonrec turn_impact = json_number
  type nonrec turn_damage = json_number
  type nonrec turn_spread = json_number
  type nonrec turn_count = json_number

  type nonrec turn_damage_type =
    fragments_native_state38_ground_items_item_item_attacks_item_damage_type

  type nonrec turn_category = fragments_native_state38_ground_items_item_item_attacks_item_category
  type nonrec turn_drain = json_number
  type nonrec turn_amount = json_number
  type nonrec turn_group = json_string
  type nonrec turn_profile = save41_member0_checkpoint_state_vision_groups_item_sight
  type nonrec turn_reveal_within = json_number

  type nonrec turn_terrains =
    save41_member0_checkpoint_state_units_item_perception_effects_item_concealment_item_terrains_member1_item

  type nonrec turn_predicate = json_number
  type nonrec turn_turns = json_number
  type nonrec turn_last_round = json_number
  type nonrec turn_light_remembered = json_boolean
  type nonrec turn_area = save39_member0_checkpoint_state_ground_items_item_tile

  type nonrec turn_condition_case =
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0

  type nonrec turn_unit_types = json_string
  type nonrec turn_unit_type = json_string
  type nonrec turn_choice = json_string
  type nonrec turn_flag = json_string
  type nonrec turn_conditions = save40_member0_checkpoint_state_tiles_item_fog_member1_condition

  type nonrec turn_tile_case =
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member1

  type nonrec turn_hex = save39_member0_checkpoint_state_ground_items_item_tile

  type nonrec turn_binding =
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member3_binding
end

module V42 = struct
  type nonrec t = save42
  type nonrec t_case = save42_member0
  type nonrec pending_enemy = json_boolean
  type nonrec undo = save34_undo_member1_item
  type nonrec progress = save39_member0_progress
  type nonrec version = literal_42
  type nonrec checkpoint = save41_member0_checkpoint
  type nonrec setup = save42_member0_setup
  type nonrec round = json_number
  type nonrec unit_id = json_string
  type nonrec changes = save34_undo_member1_item_changes_item
  type nonrec entry = save37_progress_entry_member0
  type nonrec seed = json_number
  type nonrec state = save41_member0_checkpoint_state
  type nonrec first = save41_member0_setup_first
  type nonrec enemy = save39_member0_setup_enemy
  type nonrec player = save42_member0_setup_player
  type nonrec changes_case = save34_undo_member1_item_changes_item_member0
  type nonrec kind = literal_flash
  type nonrec id = json_string
  type nonrec saved_at = json_number
  type nonrec run = json_string
  type nonrec mission = json_string
  type nonrec side = json_string
  type nonrec mission_start = json_boolean
  type nonrec run_id = json_string
  type nonrec branch_id = json_string
  type nonrec checkpoint_id = json_string
  type nonrec ordinal = json_number
  type nonrec vision_groups = save41_member0_checkpoint_state_vision_groups_item
  type nonrec faction = save41_member0_checkpoint_state_factions_item
  type nonrec ground_items = save41_member0_checkpoint_state_ground_items_item
  type nonrec armies = fragments_native_state38_armies_member1
  type nonrec condition_memory = json_string
  type nonrec map_id = json_string
  type nonrec map_revision = json_number
  type nonrec cols = json_number
  type nonrec rows = json_number
  type nonrec gold = fragments_native_state38_gold
  type nonrec winner = save41_member0_checkpoint_state_winner
  type nonrec next_id = json_number
  type nonrec combat_rng = json_number
  type nonrec log = json_string
  type nonrec tile = save41_member0_checkpoint_state_tiles_item
  type nonrec unit_ = save41_member0_checkpoint_state_units_item
  type nonrec campaign = save41_member0_checkpoint_state_campaign_member1
  type nonrec last_combat = fragments_native_state38_last_combat_member2
  type nonrec blocks = save35_setup_enemy_blocks_member1
  type nonrec policy = save35_setup_enemy_policy
  type nonrec diplomacy = save35_setup_enemy_diplomacy
  type nonrec recruitment_limit = json_number
  type nonrec manual = json_boolean
  type nonrec parent_id = json_string
  type nonrec op = literal_set
  type nonrec path = json_string
  type nonrec value = json_unknown
  type nonrec sight = save41_member0_checkpoint_state_vision_groups_item_sight
  type nonrec explored = save39_member0_checkpoint_state_ground_items_item_tile
  type nonrec vision_group = json_string
  type nonrec phase = save41_member0_setup_first
  type nonrec controller = fragments_factions7_item_controller
  type nonrec palette = json_string
  type nonrec name = json_string
  type nonrec recruits = fragments_factions7_item_recruits_member1_item
  type nonrec relations = save41_member0_checkpoint_state_factions_item_relations_member1
  type nonrec transfer = save41_member0_checkpoint_state_factions_item_transfer_member1
  type nonrec item = save41_member0_checkpoint_state_ground_items_item_item
  type nonrec location = json_string
  type nonrec col = json_number
  type nonrec row = json_number
  type nonrec terrain = json_string
  type nonrec owner = json_string
  type nonrec connection = fragments_tiles11_item_connections_member1_item
  type nonrec dead_ends = fragments_tiles11_item_connections_member1_item
  type nonrec walls = fragments_tiles11_item_connections_member1_item
  type nonrec floor = json_string
  type nonrec perception = save41_member0_checkpoint_state_tiles_item_perception
  type nonrec light = save41_member0_checkpoint_state_tiles_item_light_member1
  type nonrec revealed_to = json_string

  type nonrec perception_effects =
    save41_member0_checkpoint_state_units_item_perception_effects_item

  type nonrec items = save41_member0_checkpoint_state_ground_items_item_item
  type nonrec ai_group = json_string
  type nonrec type_ = json_string
  type nonrec hp = json_number
  type nonrec moved = json_boolean
  type nonrec attack_spent = json_boolean
  type nonrec experience = json_number
  type nonrec character = json_string
  type nonrec role = fragments_native_state38_units_item_role
  type nonrec retreating = json_boolean
  type nonrec rebuilding = json_boolean
  type nonrec scripts = save41_member0_checkpoint_state_campaign_member1_scripts_member1
  type nonrec evacuated = save41_member0_checkpoint_state_campaign_member1_evacuated_member1_item
  type nonrec recall = json_string
  type nonrec veteran = save41_member0_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec story_veteran = save41_member0_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec fallen_companions = json_string
  type nonrec carry_gold = json_number
  type nonrec mark = json_string
  type nonrec dialogue = save41_member0_checkpoint_state_campaign_member1_dialogue_item
  type nonrec dialogue_index = json_number
  type nonrec outcome = json_string
  type nonrec completed = json_boolean
  type nonrec refuge = fragments_native_state38_campaign_member1_refuge_member1
  type nonrec vigil = fragments_native_state38_campaign_member1_vigil_member1
  type nonrec hold = fragments_native_state38_campaign_member1_holds_member1
  type nonrec flags = fragments_native_state38_campaign_member1_flags
  type nonrec attacker_id = json_string
  type nonrec attack_id = json_string
  type nonrec sound = json_string
  type nonrec recruiter = save35_setup_enemy_blocks_member1_recruiter
  type nonrec planner = save35_setup_enemy_blocks_member1_planner
  type nonrec range = save41_member0_checkpoint_state_vision_groups_item_sight_range
  type nonrec penetrates = save41_member0_checkpoint_state_vision_groups_item_sight_penetrates_item
  type nonrec controller_case = fragments_factions7_item_controller_member0
  type nonrec cost = json_number
  type nonrec to_ = json_string
  type nonrec rule = save41_member0_checkpoint_state_factions_item_transfer_member1_when_
  type nonrec armor = fragments_native_state38_ground_items_item_item_armor_member1
  type nonrec description = json_string
  type nonrec attacks = save41_member0_checkpoint_state_ground_items_item_item_attacks_item
  type nonrec healing = fragments_native_state38_ground_items_item_item_healing_member1
  type nonrec remember = json_boolean
  type nonrec concealment = save41_member0_checkpoint_state_tiles_item_perception_concealment_item
  type nonrec emission = save41_member0_checkpoint_state_tiles_item_light_member1_emission_member1
  type nonrec color = fragments_light23_color_member1
  type nonrec fixture = json_boolean
  type nonrec dx = json_number
  type nonrec dy = json_number
  type nonrec radius = json_number
  type nonrec power = json_number

  type nonrec expires_at =
    save41_member0_checkpoint_state_units_item_perception_effects_item_expires_at_member1

  type nonrec failed = json_string
  type nonrec revealed = json_string
  type nonrec recruitment = fragments_native_state38_campaign_member1_scripts_member1_recruitment
  type nonrec active = json_string

  type nonrec pending =
    save41_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item

  type nonrec next_mark = json_number
  type nonrec event = json_string
  type nonrec speaker = json_string
  type nonrec portrait = json_string
  type nonrec text = json_string
  type nonrec held_turns = json_number
  type nonrec last_counted_round = json_number
  type nonrec escalated = json_boolean
  type nonrec condition = save40_member0_checkpoint_state_tiles_item_fog_member1_condition
  type nonrec blade = json_number
  type nonrec pierce = json_number
  type nonrec impact = json_number
  type nonrec damage = json_number
  type nonrec spread = json_number
  type nonrec count = json_number
  type nonrec damage_type = fragments_native_state38_ground_items_item_item_attacks_item_damage_type
  type nonrec category = fragments_native_state38_ground_items_item_item_attacks_item_category
  type nonrec drain = json_number
  type nonrec amount = json_number
  type nonrec group = json_string
  type nonrec profile = save41_member0_checkpoint_state_vision_groups_item_sight
  type nonrec reveal_within = json_number

  type nonrec terrains =
    save41_member0_checkpoint_state_units_item_perception_effects_item_concealment_item_terrains_member1_item

  type nonrec predicate = json_number
  type nonrec turns = json_number
  type nonrec last_round = json_number
  type nonrec light_remembered = json_boolean
  type nonrec area = save39_member0_checkpoint_state_ground_items_item_tile

  type nonrec condition_case =
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0

  type nonrec unit_types = json_string
  type nonrec unit_type = json_string
  type nonrec choice = json_string
  type nonrec flag = json_string
  type nonrec conditions = save40_member0_checkpoint_state_tiles_item_fog_member1_condition

  type nonrec tile_case =
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member1

  type nonrec hex = save39_member0_checkpoint_state_ground_items_item_tile

  type nonrec binding =
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member3_binding

  type nonrec current = save42_member0
  type nonrec current_pending_enemy = json_boolean
  type nonrec current_undo = save34_undo_member1_item
  type nonrec current_progress = save39_member0_progress
  type nonrec current_version = literal_42
  type nonrec current_checkpoint = save41_member0_checkpoint
  type nonrec current_setup = save42_member0_setup
  type nonrec current_round = json_number
  type nonrec current_unit_id = json_string
  type nonrec current_changes = save34_undo_member1_item_changes_item
  type nonrec current_entry = save37_progress_entry_member0
  type nonrec current_seed = json_number
  type nonrec current_state = save41_member0_checkpoint_state
  type nonrec current_first = save41_member0_setup_first
  type nonrec current_enemy = save39_member0_setup_enemy
  type nonrec current_player = save42_member0_setup_player
  type nonrec current_changes_case = save34_undo_member1_item_changes_item_member0
  type nonrec current_kind = literal_flash
  type nonrec current_id = json_string
  type nonrec current_saved_at = json_number
  type nonrec current_run = json_string
  type nonrec current_mission = json_string
  type nonrec current_side = json_string
  type nonrec current_mission_start = json_boolean
  type nonrec current_run_id = json_string
  type nonrec current_branch_id = json_string
  type nonrec current_checkpoint_id = json_string
  type nonrec current_ordinal = json_number
  type nonrec current_vision_groups = save41_member0_checkpoint_state_vision_groups_item
  type nonrec current_faction = save41_member0_checkpoint_state_factions_item
  type nonrec current_ground_items = save41_member0_checkpoint_state_ground_items_item
  type nonrec current_armies = fragments_native_state38_armies_member1
  type nonrec current_condition_memory = json_string
  type nonrec current_map_id = json_string
  type nonrec current_map_revision = json_number
  type nonrec current_cols = json_number
  type nonrec current_rows = json_number
  type nonrec current_gold = fragments_native_state38_gold
  type nonrec current_winner = save41_member0_checkpoint_state_winner
  type nonrec current_next_id = json_number
  type nonrec current_combat_rng = json_number
  type nonrec current_log = json_string
  type nonrec current_tile = save41_member0_checkpoint_state_tiles_item
  type nonrec current_unit_ = save41_member0_checkpoint_state_units_item
  type nonrec current_campaign = save41_member0_checkpoint_state_campaign_member1
  type nonrec current_last_combat = fragments_native_state38_last_combat_member2
  type nonrec current_blocks = save35_setup_enemy_blocks_member1
  type nonrec current_policy = save35_setup_enemy_policy
  type nonrec current_diplomacy = save35_setup_enemy_diplomacy
  type nonrec current_recruitment_limit = json_number
  type nonrec current_op = literal_set
  type nonrec current_path = json_string
  type nonrec current_value = json_unknown
  type nonrec current_sight = save41_member0_checkpoint_state_vision_groups_item_sight
  type nonrec current_explored = save39_member0_checkpoint_state_ground_items_item_tile
  type nonrec current_vision_group = json_string
  type nonrec current_phase = save41_member0_setup_first
  type nonrec current_controller = fragments_factions7_item_controller
  type nonrec current_palette = json_string
  type nonrec current_name = json_string
  type nonrec current_recruits = fragments_factions7_item_recruits_member1_item
  type nonrec current_relations = save41_member0_checkpoint_state_factions_item_relations_member1
  type nonrec current_transfer = save41_member0_checkpoint_state_factions_item_transfer_member1
  type nonrec current_item = save41_member0_checkpoint_state_ground_items_item_item
  type nonrec current_location = json_string
  type nonrec current_col = json_number
  type nonrec current_row = json_number
  type nonrec current_terrain = json_string
  type nonrec current_owner = json_string
  type nonrec current_connection = fragments_tiles11_item_connections_member1_item
  type nonrec current_dead_ends = fragments_tiles11_item_connections_member1_item
  type nonrec current_walls = fragments_tiles11_item_connections_member1_item
  type nonrec current_floor = json_string
  type nonrec current_perception = save41_member0_checkpoint_state_tiles_item_perception
  type nonrec current_light = save41_member0_checkpoint_state_tiles_item_light_member1
  type nonrec current_revealed_to = json_string

  type nonrec current_perception_effects =
    save41_member0_checkpoint_state_units_item_perception_effects_item

  type nonrec current_items = save41_member0_checkpoint_state_ground_items_item_item
  type nonrec current_ai_group = json_string
  type nonrec current_type_ = json_string
  type nonrec current_hp = json_number
  type nonrec current_moved = json_boolean
  type nonrec current_attack_spent = json_boolean
  type nonrec current_experience = json_number
  type nonrec current_character = json_string
  type nonrec current_role = fragments_native_state38_units_item_role
  type nonrec current_retreating = json_boolean
  type nonrec current_rebuilding = json_boolean
  type nonrec current_scripts = save41_member0_checkpoint_state_campaign_member1_scripts_member1

  type nonrec current_evacuated =
    save41_member0_checkpoint_state_campaign_member1_evacuated_member1_item

  type nonrec current_recall = json_string

  type nonrec current_veteran =
    save41_member0_checkpoint_state_campaign_member1_veterans_member1_item

  type nonrec current_story_veteran =
    save41_member0_checkpoint_state_campaign_member1_veterans_member1_item

  type nonrec current_fallen_companions = json_string
  type nonrec current_carry_gold = json_number
  type nonrec current_mark = json_string
  type nonrec current_dialogue = save41_member0_checkpoint_state_campaign_member1_dialogue_item
  type nonrec current_dialogue_index = json_number
  type nonrec current_outcome = json_string
  type nonrec current_completed = json_boolean
  type nonrec current_refuge = fragments_native_state38_campaign_member1_refuge_member1
  type nonrec current_vigil = fragments_native_state38_campaign_member1_vigil_member1
  type nonrec current_hold = fragments_native_state38_campaign_member1_holds_member1
  type nonrec current_flags = fragments_native_state38_campaign_member1_flags
  type nonrec current_attacker_id = json_string
  type nonrec current_attack_id = json_string
  type nonrec current_sound = json_string
  type nonrec current_recruiter = save35_setup_enemy_blocks_member1_recruiter
  type nonrec current_planner = save35_setup_enemy_blocks_member1_planner
  type nonrec current_range = save41_member0_checkpoint_state_vision_groups_item_sight_range

  type nonrec current_penetrates =
    save41_member0_checkpoint_state_vision_groups_item_sight_penetrates_item

  type nonrec current_controller_case = fragments_factions7_item_controller_member0
  type nonrec current_cost = json_number
  type nonrec current_to_ = json_string
  type nonrec current_rule = save41_member0_checkpoint_state_factions_item_transfer_member1_when_
  type nonrec current_armor = fragments_native_state38_ground_items_item_item_armor_member1
  type nonrec current_description = json_string
  type nonrec current_attacks = save41_member0_checkpoint_state_ground_items_item_item_attacks_item
  type nonrec current_healing = fragments_native_state38_ground_items_item_item_healing_member1
  type nonrec current_remember = json_boolean

  type nonrec current_concealment =
    save41_member0_checkpoint_state_tiles_item_perception_concealment_item

  type nonrec current_emission =
    save41_member0_checkpoint_state_tiles_item_light_member1_emission_member1

  type nonrec current_color = fragments_light23_color_member1
  type nonrec current_fixture = json_boolean
  type nonrec current_dx = json_number
  type nonrec current_dy = json_number
  type nonrec current_radius = json_number
  type nonrec current_power = json_number

  type nonrec current_expires_at =
    save41_member0_checkpoint_state_units_item_perception_effects_item_expires_at_member1

  type nonrec current_failed = json_string
  type nonrec current_revealed = json_string

  type nonrec current_recruitment =
    fragments_native_state38_campaign_member1_scripts_member1_recruitment

  type nonrec current_active = json_string

  type nonrec current_pending =
    save41_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item

  type nonrec current_next_mark = json_number
  type nonrec current_event = json_string
  type nonrec current_speaker = json_string
  type nonrec current_portrait = json_string
  type nonrec current_text = json_string
  type nonrec current_held_turns = json_number
  type nonrec current_last_counted_round = json_number
  type nonrec current_escalated = json_boolean
  type nonrec current_condition = save40_member0_checkpoint_state_tiles_item_fog_member1_condition
  type nonrec current_blade = json_number
  type nonrec current_pierce = json_number
  type nonrec current_impact = json_number
  type nonrec current_damage = json_number
  type nonrec current_spread = json_number
  type nonrec current_count = json_number

  type nonrec current_damage_type =
    fragments_native_state38_ground_items_item_item_attacks_item_damage_type

  type nonrec current_category =
    fragments_native_state38_ground_items_item_item_attacks_item_category

  type nonrec current_drain = json_number
  type nonrec current_amount = json_number
  type nonrec current_group = json_string
  type nonrec current_profile = save41_member0_checkpoint_state_vision_groups_item_sight
  type nonrec current_reveal_within = json_number

  type nonrec current_terrains =
    save41_member0_checkpoint_state_units_item_perception_effects_item_concealment_item_terrains_member1_item

  type nonrec current_predicate = json_number
  type nonrec current_turns = json_number
  type nonrec current_last_round = json_number
  type nonrec current_light_remembered = json_boolean
  type nonrec current_area = save39_member0_checkpoint_state_ground_items_item_tile

  type nonrec current_condition_case =
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0

  type nonrec current_unit_types = json_string
  type nonrec current_unit_type = json_string
  type nonrec current_choice = json_string
  type nonrec current_flag = json_string
  type nonrec current_conditions = save40_member0_checkpoint_state_tiles_item_fog_member1_condition

  type nonrec current_tile_case =
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member1

  type nonrec current_hex = save39_member0_checkpoint_state_ground_items_item_tile

  type nonrec current_binding =
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member3_binding

  type nonrec turn = save42_member1
  type nonrec turn_pending_enemy = literal_false
  type nonrec turn_undo = json_undefined
  type nonrec turn_progress = save39_member1_progress
  type nonrec turn_version = literal_42
  type nonrec turn_checkpoint = save41_member0_checkpoint
  type nonrec turn_setup = save42_member0_setup
  type nonrec turn_entry = history4_entries_item
  type nonrec turn_seed = json_number
  type nonrec turn_state = save41_member0_checkpoint_state
  type nonrec turn_first = save41_member0_setup_first
  type nonrec turn_enemy = save39_member0_setup_enemy
  type nonrec turn_player = save42_member0_setup_player
  type nonrec turn_kind = literal_turn
  type nonrec turn_manual = json_boolean
  type nonrec turn_parent_id = json_string
  type nonrec turn_id = json_string
  type nonrec turn_saved_at = json_number
  type nonrec turn_run = json_string
  type nonrec turn_mission = json_string
  type nonrec turn_round = json_number
  type nonrec turn_side = json_string
  type nonrec turn_mission_start = json_boolean
  type nonrec turn_run_id = json_string
  type nonrec turn_branch_id = json_string
  type nonrec turn_checkpoint_id = json_string
  type nonrec turn_ordinal = json_number
  type nonrec turn_vision_groups = save41_member0_checkpoint_state_vision_groups_item
  type nonrec turn_faction = save41_member0_checkpoint_state_factions_item
  type nonrec turn_ground_items = save41_member0_checkpoint_state_ground_items_item
  type nonrec turn_armies = fragments_native_state38_armies_member1
  type nonrec turn_condition_memory = json_string
  type nonrec turn_map_id = json_string
  type nonrec turn_map_revision = json_number
  type nonrec turn_cols = json_number
  type nonrec turn_rows = json_number
  type nonrec turn_gold = fragments_native_state38_gold
  type nonrec turn_winner = save41_member0_checkpoint_state_winner
  type nonrec turn_next_id = json_number
  type nonrec turn_combat_rng = json_number
  type nonrec turn_log = json_string
  type nonrec turn_tile = save41_member0_checkpoint_state_tiles_item
  type nonrec turn_unit_ = save41_member0_checkpoint_state_units_item
  type nonrec turn_campaign = save41_member0_checkpoint_state_campaign_member1
  type nonrec turn_last_combat = fragments_native_state38_last_combat_member2
  type nonrec turn_blocks = save35_setup_enemy_blocks_member1
  type nonrec turn_policy = save35_setup_enemy_policy
  type nonrec turn_diplomacy = save35_setup_enemy_diplomacy
  type nonrec turn_recruitment_limit = json_number
  type nonrec turn_sight = save41_member0_checkpoint_state_vision_groups_item_sight
  type nonrec turn_explored = save39_member0_checkpoint_state_ground_items_item_tile
  type nonrec turn_vision_group = json_string
  type nonrec turn_phase = save41_member0_setup_first
  type nonrec turn_controller = fragments_factions7_item_controller
  type nonrec turn_palette = json_string
  type nonrec turn_name = json_string
  type nonrec turn_recruits = fragments_factions7_item_recruits_member1_item
  type nonrec turn_relations = save41_member0_checkpoint_state_factions_item_relations_member1
  type nonrec turn_transfer = save41_member0_checkpoint_state_factions_item_transfer_member1
  type nonrec turn_item = save41_member0_checkpoint_state_ground_items_item_item
  type nonrec turn_location = json_string
  type nonrec turn_col = json_number
  type nonrec turn_row = json_number
  type nonrec turn_terrain = json_string
  type nonrec turn_owner = json_string
  type nonrec turn_connection = fragments_tiles11_item_connections_member1_item
  type nonrec turn_dead_ends = fragments_tiles11_item_connections_member1_item
  type nonrec turn_walls = fragments_tiles11_item_connections_member1_item
  type nonrec turn_floor = json_string
  type nonrec turn_perception = save41_member0_checkpoint_state_tiles_item_perception
  type nonrec turn_light = save41_member0_checkpoint_state_tiles_item_light_member1
  type nonrec turn_revealed_to = json_string

  type nonrec turn_perception_effects =
    save41_member0_checkpoint_state_units_item_perception_effects_item

  type nonrec turn_items = save41_member0_checkpoint_state_ground_items_item_item
  type nonrec turn_ai_group = json_string
  type nonrec turn_type_ = json_string
  type nonrec turn_hp = json_number
  type nonrec turn_moved = json_boolean
  type nonrec turn_attack_spent = json_boolean
  type nonrec turn_experience = json_number
  type nonrec turn_character = json_string
  type nonrec turn_role = fragments_native_state38_units_item_role
  type nonrec turn_retreating = json_boolean
  type nonrec turn_rebuilding = json_boolean
  type nonrec turn_scripts = save41_member0_checkpoint_state_campaign_member1_scripts_member1

  type nonrec turn_evacuated =
    save41_member0_checkpoint_state_campaign_member1_evacuated_member1_item

  type nonrec turn_recall = json_string
  type nonrec turn_veteran = save41_member0_checkpoint_state_campaign_member1_veterans_member1_item

  type nonrec turn_story_veteran =
    save41_member0_checkpoint_state_campaign_member1_veterans_member1_item

  type nonrec turn_fallen_companions = json_string
  type nonrec turn_carry_gold = json_number
  type nonrec turn_mark = json_string
  type nonrec turn_dialogue = save41_member0_checkpoint_state_campaign_member1_dialogue_item
  type nonrec turn_dialogue_index = json_number
  type nonrec turn_outcome = json_string
  type nonrec turn_completed = json_boolean
  type nonrec turn_refuge = fragments_native_state38_campaign_member1_refuge_member1
  type nonrec turn_vigil = fragments_native_state38_campaign_member1_vigil_member1
  type nonrec turn_hold = fragments_native_state38_campaign_member1_holds_member1
  type nonrec turn_flags = fragments_native_state38_campaign_member1_flags
  type nonrec turn_attacker_id = json_string
  type nonrec turn_attack_id = json_string
  type nonrec turn_sound = json_string
  type nonrec turn_recruiter = save35_setup_enemy_blocks_member1_recruiter
  type nonrec turn_planner = save35_setup_enemy_blocks_member1_planner
  type nonrec turn_range = save41_member0_checkpoint_state_vision_groups_item_sight_range

  type nonrec turn_penetrates =
    save41_member0_checkpoint_state_vision_groups_item_sight_penetrates_item

  type nonrec turn_controller_case = fragments_factions7_item_controller_member0
  type nonrec turn_cost = json_number
  type nonrec turn_to_ = json_string
  type nonrec turn_rule = save41_member0_checkpoint_state_factions_item_transfer_member1_when_
  type nonrec turn_armor = fragments_native_state38_ground_items_item_item_armor_member1
  type nonrec turn_description = json_string
  type nonrec turn_attacks = save41_member0_checkpoint_state_ground_items_item_item_attacks_item
  type nonrec turn_healing = fragments_native_state38_ground_items_item_item_healing_member1
  type nonrec turn_remember = json_boolean

  type nonrec turn_concealment =
    save41_member0_checkpoint_state_tiles_item_perception_concealment_item

  type nonrec turn_emission =
    save41_member0_checkpoint_state_tiles_item_light_member1_emission_member1

  type nonrec turn_color = fragments_light23_color_member1
  type nonrec turn_fixture = json_boolean
  type nonrec turn_dx = json_number
  type nonrec turn_dy = json_number
  type nonrec turn_radius = json_number
  type nonrec turn_power = json_number

  type nonrec turn_expires_at =
    save41_member0_checkpoint_state_units_item_perception_effects_item_expires_at_member1

  type nonrec turn_failed = json_string
  type nonrec turn_revealed = json_string

  type nonrec turn_recruitment =
    fragments_native_state38_campaign_member1_scripts_member1_recruitment

  type nonrec turn_active = json_string

  type nonrec turn_pending =
    save41_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item

  type nonrec turn_next_mark = json_number
  type nonrec turn_event = json_string
  type nonrec turn_speaker = json_string
  type nonrec turn_portrait = json_string
  type nonrec turn_text = json_string
  type nonrec turn_held_turns = json_number
  type nonrec turn_last_counted_round = json_number
  type nonrec turn_escalated = json_boolean
  type nonrec turn_condition = save40_member0_checkpoint_state_tiles_item_fog_member1_condition
  type nonrec turn_blade = json_number
  type nonrec turn_pierce = json_number
  type nonrec turn_impact = json_number
  type nonrec turn_damage = json_number
  type nonrec turn_spread = json_number
  type nonrec turn_count = json_number

  type nonrec turn_damage_type =
    fragments_native_state38_ground_items_item_item_attacks_item_damage_type

  type nonrec turn_category = fragments_native_state38_ground_items_item_item_attacks_item_category
  type nonrec turn_drain = json_number
  type nonrec turn_amount = json_number
  type nonrec turn_group = json_string
  type nonrec turn_profile = save41_member0_checkpoint_state_vision_groups_item_sight
  type nonrec turn_reveal_within = json_number

  type nonrec turn_terrains =
    save41_member0_checkpoint_state_units_item_perception_effects_item_concealment_item_terrains_member1_item

  type nonrec turn_predicate = json_number
  type nonrec turn_turns = json_number
  type nonrec turn_last_round = json_number
  type nonrec turn_light_remembered = json_boolean
  type nonrec turn_area = save39_member0_checkpoint_state_ground_items_item_tile

  type nonrec turn_condition_case =
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0

  type nonrec turn_unit_types = json_string
  type nonrec turn_unit_type = json_string
  type nonrec turn_choice = json_string
  type nonrec turn_flag = json_string
  type nonrec turn_conditions = save40_member0_checkpoint_state_tiles_item_fog_member1_condition

  type nonrec turn_tile_case =
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member1

  type nonrec turn_hex = save39_member0_checkpoint_state_ground_items_item_tile

  type nonrec turn_binding =
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member3_binding
end

module V43 = struct
  type nonrec t = save43
  type nonrec t_case = save43_member0
  type nonrec pending_enemy = json_boolean
  type nonrec undo = save34_undo_member1_item
  type nonrec progress = save39_member0_progress
  type nonrec version = literal_43
  type nonrec checkpoint = save43_member0_checkpoint
  type nonrec setup = save42_member0_setup
  type nonrec round = json_number
  type nonrec unit_id = json_string
  type nonrec changes = save34_undo_member1_item_changes_item
  type nonrec entry = save37_progress_entry_member0
  type nonrec seed = json_number
  type nonrec state = save43_member0_checkpoint_state
  type nonrec first = save41_member0_setup_first
  type nonrec enemy = save39_member0_setup_enemy
  type nonrec player = save42_member0_setup_player
  type nonrec changes_case = save34_undo_member1_item_changes_item_member0
  type nonrec kind = literal_flash
  type nonrec id = json_string
  type nonrec saved_at = json_number
  type nonrec run = json_string
  type nonrec mission = json_string
  type nonrec side = json_string
  type nonrec mission_start = json_boolean
  type nonrec run_id = json_string
  type nonrec branch_id = json_string
  type nonrec checkpoint_id = json_string
  type nonrec ordinal = json_number
  type nonrec vision_groups = save41_member0_checkpoint_state_vision_groups_item
  type nonrec faction = save41_member0_checkpoint_state_factions_item
  type nonrec ground_items = save43_member0_checkpoint_state_ground_items_item
  type nonrec armies = fragments_native_state38_armies_member1
  type nonrec condition_memory = json_string
  type nonrec map_id = json_string
  type nonrec map_revision = json_number
  type nonrec cols = json_number
  type nonrec rows = json_number
  type nonrec gold = fragments_native_state38_gold
  type nonrec winner = save41_member0_checkpoint_state_winner
  type nonrec next_id = json_number
  type nonrec combat_rng = json_number
  type nonrec log = json_string
  type nonrec tile = save41_member0_checkpoint_state_tiles_item
  type nonrec unit_ = save43_member0_checkpoint_state_units_item
  type nonrec campaign = save43_member0_checkpoint_state_campaign_member1
  type nonrec last_combat = fragments_native_state38_last_combat_member2
  type nonrec blocks = save35_setup_enemy_blocks_member1
  type nonrec policy = save35_setup_enemy_policy
  type nonrec diplomacy = save35_setup_enemy_diplomacy
  type nonrec recruitment_limit = json_number
  type nonrec manual = json_boolean
  type nonrec parent_id = json_string
  type nonrec op = literal_set
  type nonrec path = json_string
  type nonrec value = json_unknown
  type nonrec sight = save41_member0_checkpoint_state_vision_groups_item_sight
  type nonrec explored = save39_member0_checkpoint_state_ground_items_item_tile
  type nonrec vision_group = json_string
  type nonrec phase = save41_member0_setup_first
  type nonrec controller = fragments_factions7_item_controller
  type nonrec palette = json_string
  type nonrec name = json_string
  type nonrec recruits = fragments_factions7_item_recruits_member1_item
  type nonrec relations = save41_member0_checkpoint_state_factions_item_relations_member1
  type nonrec transfer = save41_member0_checkpoint_state_factions_item_transfer_member1
  type nonrec item = save43_member0_checkpoint_state_ground_items_item_item
  type nonrec location = json_string
  type nonrec col = json_number
  type nonrec row = json_number
  type nonrec terrain = json_string
  type nonrec owner = json_string
  type nonrec connection = fragments_tiles11_item_connections_member1_item
  type nonrec dead_ends = fragments_tiles11_item_connections_member1_item
  type nonrec walls = fragments_tiles11_item_connections_member1_item
  type nonrec floor = json_string
  type nonrec perception = save41_member0_checkpoint_state_tiles_item_perception
  type nonrec light = save41_member0_checkpoint_state_tiles_item_light_member1
  type nonrec revealed_to = json_string

  type nonrec perception_effects =
    save43_member0_checkpoint_state_units_item_perception_effects_item

  type nonrec items = save43_member0_checkpoint_state_ground_items_item_item
  type nonrec ai_group = json_string
  type nonrec type_ = json_string
  type nonrec hp = json_number
  type nonrec moved = json_boolean
  type nonrec attack_spent = json_boolean
  type nonrec experience = json_number
  type nonrec character = json_string
  type nonrec role = fragments_native_state38_units_item_role
  type nonrec retreating = json_boolean
  type nonrec rebuilding = json_boolean
  type nonrec scripts = save41_member0_checkpoint_state_campaign_member1_scripts_member1
  type nonrec evacuated = save43_member0_checkpoint_state_campaign_member1_evacuated_member1_item
  type nonrec recall = json_string
  type nonrec veteran = save43_member0_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec story_veteran = save43_member0_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec fallen_companions = json_string
  type nonrec carry_gold = json_number
  type nonrec mark = json_string
  type nonrec dialogue = save41_member0_checkpoint_state_campaign_member1_dialogue_item
  type nonrec dialogue_index = json_number
  type nonrec outcome = json_string
  type nonrec completed = json_boolean
  type nonrec refuge = fragments_native_state38_campaign_member1_refuge_member1
  type nonrec vigil = fragments_native_state38_campaign_member1_vigil_member1
  type nonrec hold = fragments_native_state38_campaign_member1_holds_member1
  type nonrec flags = fragments_native_state38_campaign_member1_flags
  type nonrec attacker_id = json_string
  type nonrec attack_id = json_string
  type nonrec sound = json_string
  type nonrec recruiter = save35_setup_enemy_blocks_member1_recruiter
  type nonrec planner = save35_setup_enemy_blocks_member1_planner
  type nonrec range = save41_member0_checkpoint_state_vision_groups_item_sight_range
  type nonrec penetrates = save41_member0_checkpoint_state_vision_groups_item_sight_penetrates_item
  type nonrec controller_case = fragments_factions7_item_controller_member0
  type nonrec cost = json_number
  type nonrec to_ = json_string
  type nonrec rule = save41_member0_checkpoint_state_factions_item_transfer_member1_when_
  type nonrec armor = fragments_native_state38_ground_items_item_item_armor_member1
  type nonrec description = json_string
  type nonrec attacks = save41_member0_checkpoint_state_ground_items_item_item_attacks_item
  type nonrec healing = fragments_native_state38_ground_items_item_item_healing_member1
  type nonrec remember = json_boolean
  type nonrec concealment = save41_member0_checkpoint_state_tiles_item_perception_concealment_item
  type nonrec emission = save41_member0_checkpoint_state_tiles_item_light_member1_emission_member1
  type nonrec color = fragments_light23_color_member1
  type nonrec fixture = json_boolean
  type nonrec dx = json_number
  type nonrec dy = json_number
  type nonrec radius = json_number
  type nonrec power = json_number

  type nonrec expires_at =
    save41_member0_checkpoint_state_units_item_perception_effects_item_expires_at_member1

  type nonrec failed = json_string
  type nonrec revealed = json_string
  type nonrec recruitment = fragments_native_state38_campaign_member1_scripts_member1_recruitment
  type nonrec active = json_string

  type nonrec pending =
    save41_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item

  type nonrec next_mark = json_number
  type nonrec event = json_string
  type nonrec speaker = json_string
  type nonrec portrait = json_string
  type nonrec text = json_string
  type nonrec held_turns = json_number
  type nonrec last_counted_round = json_number
  type nonrec escalated = json_boolean
  type nonrec condition = save40_member0_checkpoint_state_tiles_item_fog_member1_condition
  type nonrec blade = json_number
  type nonrec pierce = json_number
  type nonrec impact = json_number
  type nonrec damage = json_number
  type nonrec spread = json_number
  type nonrec count = json_number
  type nonrec damage_type = fragments_native_state38_ground_items_item_item_attacks_item_damage_type
  type nonrec category = fragments_native_state38_ground_items_item_item_attacks_item_category
  type nonrec drain = json_number
  type nonrec amount = json_number
  type nonrec group = json_string
  type nonrec profile = save41_member0_checkpoint_state_vision_groups_item_sight
  type nonrec reveal_within = json_number

  type nonrec terrains =
    save43_member0_checkpoint_state_units_item_perception_effects_item_concealment_item_terrains_member1_item

  type nonrec predicate = json_number
  type nonrec turns = json_number
  type nonrec last_round = json_number
  type nonrec light_remembered = json_boolean
  type nonrec area = save39_member0_checkpoint_state_ground_items_item_tile

  type nonrec condition_case =
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0

  type nonrec unit_types = json_string
  type nonrec unit_type = json_string
  type nonrec choice = json_string
  type nonrec flag = json_string
  type nonrec conditions = save40_member0_checkpoint_state_tiles_item_fog_member1_condition

  type nonrec tile_case =
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member1

  type nonrec hex = save39_member0_checkpoint_state_ground_items_item_tile

  type nonrec binding =
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member3_binding

  type nonrec current = save43_member0
  type nonrec current_pending_enemy = json_boolean
  type nonrec current_undo = save34_undo_member1_item
  type nonrec current_progress = save39_member0_progress
  type nonrec current_version = literal_43
  type nonrec current_checkpoint = save43_member0_checkpoint
  type nonrec current_setup = save42_member0_setup
  type nonrec current_round = json_number
  type nonrec current_unit_id = json_string
  type nonrec current_changes = save34_undo_member1_item_changes_item
  type nonrec current_entry = save37_progress_entry_member0
  type nonrec current_seed = json_number
  type nonrec current_state = save43_member0_checkpoint_state
  type nonrec current_first = save41_member0_setup_first
  type nonrec current_enemy = save39_member0_setup_enemy
  type nonrec current_player = save42_member0_setup_player
  type nonrec current_changes_case = save34_undo_member1_item_changes_item_member0
  type nonrec current_kind = literal_flash
  type nonrec current_id = json_string
  type nonrec current_saved_at = json_number
  type nonrec current_run = json_string
  type nonrec current_mission = json_string
  type nonrec current_side = json_string
  type nonrec current_mission_start = json_boolean
  type nonrec current_run_id = json_string
  type nonrec current_branch_id = json_string
  type nonrec current_checkpoint_id = json_string
  type nonrec current_ordinal = json_number
  type nonrec current_vision_groups = save41_member0_checkpoint_state_vision_groups_item
  type nonrec current_faction = save41_member0_checkpoint_state_factions_item
  type nonrec current_ground_items = save43_member0_checkpoint_state_ground_items_item
  type nonrec current_armies = fragments_native_state38_armies_member1
  type nonrec current_condition_memory = json_string
  type nonrec current_map_id = json_string
  type nonrec current_map_revision = json_number
  type nonrec current_cols = json_number
  type nonrec current_rows = json_number
  type nonrec current_gold = fragments_native_state38_gold
  type nonrec current_winner = save41_member0_checkpoint_state_winner
  type nonrec current_next_id = json_number
  type nonrec current_combat_rng = json_number
  type nonrec current_log = json_string
  type nonrec current_tile = save41_member0_checkpoint_state_tiles_item
  type nonrec current_unit_ = save43_member0_checkpoint_state_units_item
  type nonrec current_campaign = save43_member0_checkpoint_state_campaign_member1
  type nonrec current_last_combat = fragments_native_state38_last_combat_member2
  type nonrec current_blocks = save35_setup_enemy_blocks_member1
  type nonrec current_policy = save35_setup_enemy_policy
  type nonrec current_diplomacy = save35_setup_enemy_diplomacy
  type nonrec current_recruitment_limit = json_number
  type nonrec current_op = literal_set
  type nonrec current_path = json_string
  type nonrec current_value = json_unknown
  type nonrec current_sight = save41_member0_checkpoint_state_vision_groups_item_sight
  type nonrec current_explored = save39_member0_checkpoint_state_ground_items_item_tile
  type nonrec current_vision_group = json_string
  type nonrec current_phase = save41_member0_setup_first
  type nonrec current_controller = fragments_factions7_item_controller
  type nonrec current_palette = json_string
  type nonrec current_name = json_string
  type nonrec current_recruits = fragments_factions7_item_recruits_member1_item
  type nonrec current_relations = save41_member0_checkpoint_state_factions_item_relations_member1
  type nonrec current_transfer = save41_member0_checkpoint_state_factions_item_transfer_member1
  type nonrec current_item = save43_member0_checkpoint_state_ground_items_item_item
  type nonrec current_location = json_string
  type nonrec current_col = json_number
  type nonrec current_row = json_number
  type nonrec current_terrain = json_string
  type nonrec current_owner = json_string
  type nonrec current_connection = fragments_tiles11_item_connections_member1_item
  type nonrec current_dead_ends = fragments_tiles11_item_connections_member1_item
  type nonrec current_walls = fragments_tiles11_item_connections_member1_item
  type nonrec current_floor = json_string
  type nonrec current_perception = save41_member0_checkpoint_state_tiles_item_perception
  type nonrec current_light = save41_member0_checkpoint_state_tiles_item_light_member1
  type nonrec current_revealed_to = json_string

  type nonrec current_perception_effects =
    save43_member0_checkpoint_state_units_item_perception_effects_item

  type nonrec current_items = save43_member0_checkpoint_state_ground_items_item_item
  type nonrec current_ai_group = json_string
  type nonrec current_type_ = json_string
  type nonrec current_hp = json_number
  type nonrec current_moved = json_boolean
  type nonrec current_attack_spent = json_boolean
  type nonrec current_experience = json_number
  type nonrec current_character = json_string
  type nonrec current_role = fragments_native_state38_units_item_role
  type nonrec current_retreating = json_boolean
  type nonrec current_rebuilding = json_boolean
  type nonrec current_scripts = save41_member0_checkpoint_state_campaign_member1_scripts_member1

  type nonrec current_evacuated =
    save43_member0_checkpoint_state_campaign_member1_evacuated_member1_item

  type nonrec current_recall = json_string

  type nonrec current_veteran =
    save43_member0_checkpoint_state_campaign_member1_veterans_member1_item

  type nonrec current_story_veteran =
    save43_member0_checkpoint_state_campaign_member1_veterans_member1_item

  type nonrec current_fallen_companions = json_string
  type nonrec current_carry_gold = json_number
  type nonrec current_mark = json_string
  type nonrec current_dialogue = save41_member0_checkpoint_state_campaign_member1_dialogue_item
  type nonrec current_dialogue_index = json_number
  type nonrec current_outcome = json_string
  type nonrec current_completed = json_boolean
  type nonrec current_refuge = fragments_native_state38_campaign_member1_refuge_member1
  type nonrec current_vigil = fragments_native_state38_campaign_member1_vigil_member1
  type nonrec current_hold = fragments_native_state38_campaign_member1_holds_member1
  type nonrec current_flags = fragments_native_state38_campaign_member1_flags
  type nonrec current_attacker_id = json_string
  type nonrec current_attack_id = json_string
  type nonrec current_sound = json_string
  type nonrec current_recruiter = save35_setup_enemy_blocks_member1_recruiter
  type nonrec current_planner = save35_setup_enemy_blocks_member1_planner
  type nonrec current_range = save41_member0_checkpoint_state_vision_groups_item_sight_range

  type nonrec current_penetrates =
    save41_member0_checkpoint_state_vision_groups_item_sight_penetrates_item

  type nonrec current_controller_case = fragments_factions7_item_controller_member0
  type nonrec current_cost = json_number
  type nonrec current_to_ = json_string
  type nonrec current_rule = save41_member0_checkpoint_state_factions_item_transfer_member1_when_
  type nonrec current_armor = fragments_native_state38_ground_items_item_item_armor_member1
  type nonrec current_description = json_string
  type nonrec current_attacks = save41_member0_checkpoint_state_ground_items_item_item_attacks_item
  type nonrec current_healing = fragments_native_state38_ground_items_item_item_healing_member1
  type nonrec current_remember = json_boolean

  type nonrec current_concealment =
    save41_member0_checkpoint_state_tiles_item_perception_concealment_item

  type nonrec current_emission =
    save41_member0_checkpoint_state_tiles_item_light_member1_emission_member1

  type nonrec current_color = fragments_light23_color_member1
  type nonrec current_fixture = json_boolean
  type nonrec current_dx = json_number
  type nonrec current_dy = json_number
  type nonrec current_radius = json_number
  type nonrec current_power = json_number

  type nonrec current_expires_at =
    save41_member0_checkpoint_state_units_item_perception_effects_item_expires_at_member1

  type nonrec current_failed = json_string
  type nonrec current_revealed = json_string

  type nonrec current_recruitment =
    fragments_native_state38_campaign_member1_scripts_member1_recruitment

  type nonrec current_active = json_string

  type nonrec current_pending =
    save41_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item

  type nonrec current_next_mark = json_number
  type nonrec current_event = json_string
  type nonrec current_speaker = json_string
  type nonrec current_portrait = json_string
  type nonrec current_text = json_string
  type nonrec current_held_turns = json_number
  type nonrec current_last_counted_round = json_number
  type nonrec current_escalated = json_boolean
  type nonrec current_condition = save40_member0_checkpoint_state_tiles_item_fog_member1_condition
  type nonrec current_blade = json_number
  type nonrec current_pierce = json_number
  type nonrec current_impact = json_number
  type nonrec current_damage = json_number
  type nonrec current_spread = json_number
  type nonrec current_count = json_number

  type nonrec current_damage_type =
    fragments_native_state38_ground_items_item_item_attacks_item_damage_type

  type nonrec current_category =
    fragments_native_state38_ground_items_item_item_attacks_item_category

  type nonrec current_drain = json_number
  type nonrec current_amount = json_number
  type nonrec current_group = json_string
  type nonrec current_profile = save41_member0_checkpoint_state_vision_groups_item_sight
  type nonrec current_reveal_within = json_number

  type nonrec current_terrains =
    save43_member0_checkpoint_state_units_item_perception_effects_item_concealment_item_terrains_member1_item

  type nonrec current_predicate = json_number
  type nonrec current_turns = json_number
  type nonrec current_last_round = json_number
  type nonrec current_light_remembered = json_boolean
  type nonrec current_area = save39_member0_checkpoint_state_ground_items_item_tile

  type nonrec current_condition_case =
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0

  type nonrec current_unit_types = json_string
  type nonrec current_unit_type = json_string
  type nonrec current_choice = json_string
  type nonrec current_flag = json_string
  type nonrec current_conditions = save40_member0_checkpoint_state_tiles_item_fog_member1_condition

  type nonrec current_tile_case =
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member1

  type nonrec current_hex = save39_member0_checkpoint_state_ground_items_item_tile

  type nonrec current_binding =
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member3_binding

  type nonrec turn = save43_member1
  type nonrec turn_pending_enemy = literal_false
  type nonrec turn_undo = json_undefined
  type nonrec turn_progress = save39_member1_progress
  type nonrec turn_version = literal_43
  type nonrec turn_checkpoint = save43_member0_checkpoint
  type nonrec turn_setup = save42_member0_setup
  type nonrec turn_entry = history4_entries_item
  type nonrec turn_seed = json_number
  type nonrec turn_state = save43_member0_checkpoint_state
  type nonrec turn_first = save41_member0_setup_first
  type nonrec turn_enemy = save39_member0_setup_enemy
  type nonrec turn_player = save42_member0_setup_player
  type nonrec turn_kind = literal_turn
  type nonrec turn_manual = json_boolean
  type nonrec turn_parent_id = json_string
  type nonrec turn_id = json_string
  type nonrec turn_saved_at = json_number
  type nonrec turn_run = json_string
  type nonrec turn_mission = json_string
  type nonrec turn_round = json_number
  type nonrec turn_side = json_string
  type nonrec turn_mission_start = json_boolean
  type nonrec turn_run_id = json_string
  type nonrec turn_branch_id = json_string
  type nonrec turn_checkpoint_id = json_string
  type nonrec turn_ordinal = json_number
  type nonrec turn_vision_groups = save41_member0_checkpoint_state_vision_groups_item
  type nonrec turn_faction = save41_member0_checkpoint_state_factions_item
  type nonrec turn_ground_items = save43_member0_checkpoint_state_ground_items_item
  type nonrec turn_armies = fragments_native_state38_armies_member1
  type nonrec turn_condition_memory = json_string
  type nonrec turn_map_id = json_string
  type nonrec turn_map_revision = json_number
  type nonrec turn_cols = json_number
  type nonrec turn_rows = json_number
  type nonrec turn_gold = fragments_native_state38_gold
  type nonrec turn_winner = save41_member0_checkpoint_state_winner
  type nonrec turn_next_id = json_number
  type nonrec turn_combat_rng = json_number
  type nonrec turn_log = json_string
  type nonrec turn_tile = save41_member0_checkpoint_state_tiles_item
  type nonrec turn_unit_ = save43_member0_checkpoint_state_units_item
  type nonrec turn_campaign = save43_member0_checkpoint_state_campaign_member1
  type nonrec turn_last_combat = fragments_native_state38_last_combat_member2
  type nonrec turn_blocks = save35_setup_enemy_blocks_member1
  type nonrec turn_policy = save35_setup_enemy_policy
  type nonrec turn_diplomacy = save35_setup_enemy_diplomacy
  type nonrec turn_recruitment_limit = json_number
  type nonrec turn_sight = save41_member0_checkpoint_state_vision_groups_item_sight
  type nonrec turn_explored = save39_member0_checkpoint_state_ground_items_item_tile
  type nonrec turn_vision_group = json_string
  type nonrec turn_phase = save41_member0_setup_first
  type nonrec turn_controller = fragments_factions7_item_controller
  type nonrec turn_palette = json_string
  type nonrec turn_name = json_string
  type nonrec turn_recruits = fragments_factions7_item_recruits_member1_item
  type nonrec turn_relations = save41_member0_checkpoint_state_factions_item_relations_member1
  type nonrec turn_transfer = save41_member0_checkpoint_state_factions_item_transfer_member1
  type nonrec turn_item = save43_member0_checkpoint_state_ground_items_item_item
  type nonrec turn_location = json_string
  type nonrec turn_col = json_number
  type nonrec turn_row = json_number
  type nonrec turn_terrain = json_string
  type nonrec turn_owner = json_string
  type nonrec turn_connection = fragments_tiles11_item_connections_member1_item
  type nonrec turn_dead_ends = fragments_tiles11_item_connections_member1_item
  type nonrec turn_walls = fragments_tiles11_item_connections_member1_item
  type nonrec turn_floor = json_string
  type nonrec turn_perception = save41_member0_checkpoint_state_tiles_item_perception
  type nonrec turn_light = save41_member0_checkpoint_state_tiles_item_light_member1
  type nonrec turn_revealed_to = json_string

  type nonrec turn_perception_effects =
    save43_member0_checkpoint_state_units_item_perception_effects_item

  type nonrec turn_items = save43_member0_checkpoint_state_ground_items_item_item
  type nonrec turn_ai_group = json_string
  type nonrec turn_type_ = json_string
  type nonrec turn_hp = json_number
  type nonrec turn_moved = json_boolean
  type nonrec turn_attack_spent = json_boolean
  type nonrec turn_experience = json_number
  type nonrec turn_character = json_string
  type nonrec turn_role = fragments_native_state38_units_item_role
  type nonrec turn_retreating = json_boolean
  type nonrec turn_rebuilding = json_boolean
  type nonrec turn_scripts = save41_member0_checkpoint_state_campaign_member1_scripts_member1

  type nonrec turn_evacuated =
    save43_member0_checkpoint_state_campaign_member1_evacuated_member1_item

  type nonrec turn_recall = json_string
  type nonrec turn_veteran = save43_member0_checkpoint_state_campaign_member1_veterans_member1_item

  type nonrec turn_story_veteran =
    save43_member0_checkpoint_state_campaign_member1_veterans_member1_item

  type nonrec turn_fallen_companions = json_string
  type nonrec turn_carry_gold = json_number
  type nonrec turn_mark = json_string
  type nonrec turn_dialogue = save41_member0_checkpoint_state_campaign_member1_dialogue_item
  type nonrec turn_dialogue_index = json_number
  type nonrec turn_outcome = json_string
  type nonrec turn_completed = json_boolean
  type nonrec turn_refuge = fragments_native_state38_campaign_member1_refuge_member1
  type nonrec turn_vigil = fragments_native_state38_campaign_member1_vigil_member1
  type nonrec turn_hold = fragments_native_state38_campaign_member1_holds_member1
  type nonrec turn_flags = fragments_native_state38_campaign_member1_flags
  type nonrec turn_attacker_id = json_string
  type nonrec turn_attack_id = json_string
  type nonrec turn_sound = json_string
  type nonrec turn_recruiter = save35_setup_enemy_blocks_member1_recruiter
  type nonrec turn_planner = save35_setup_enemy_blocks_member1_planner
  type nonrec turn_range = save41_member0_checkpoint_state_vision_groups_item_sight_range

  type nonrec turn_penetrates =
    save41_member0_checkpoint_state_vision_groups_item_sight_penetrates_item

  type nonrec turn_controller_case = fragments_factions7_item_controller_member0
  type nonrec turn_cost = json_number
  type nonrec turn_to_ = json_string
  type nonrec turn_rule = save41_member0_checkpoint_state_factions_item_transfer_member1_when_
  type nonrec turn_armor = fragments_native_state38_ground_items_item_item_armor_member1
  type nonrec turn_description = json_string
  type nonrec turn_attacks = save41_member0_checkpoint_state_ground_items_item_item_attacks_item
  type nonrec turn_healing = fragments_native_state38_ground_items_item_item_healing_member1
  type nonrec turn_remember = json_boolean

  type nonrec turn_concealment =
    save41_member0_checkpoint_state_tiles_item_perception_concealment_item

  type nonrec turn_emission =
    save41_member0_checkpoint_state_tiles_item_light_member1_emission_member1

  type nonrec turn_color = fragments_light23_color_member1
  type nonrec turn_fixture = json_boolean
  type nonrec turn_dx = json_number
  type nonrec turn_dy = json_number
  type nonrec turn_radius = json_number
  type nonrec turn_power = json_number

  type nonrec turn_expires_at =
    save41_member0_checkpoint_state_units_item_perception_effects_item_expires_at_member1

  type nonrec turn_failed = json_string
  type nonrec turn_revealed = json_string

  type nonrec turn_recruitment =
    fragments_native_state38_campaign_member1_scripts_member1_recruitment

  type nonrec turn_active = json_string

  type nonrec turn_pending =
    save41_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item

  type nonrec turn_next_mark = json_number
  type nonrec turn_event = json_string
  type nonrec turn_speaker = json_string
  type nonrec turn_portrait = json_string
  type nonrec turn_text = json_string
  type nonrec turn_held_turns = json_number
  type nonrec turn_last_counted_round = json_number
  type nonrec turn_escalated = json_boolean
  type nonrec turn_condition = save40_member0_checkpoint_state_tiles_item_fog_member1_condition
  type nonrec turn_blade = json_number
  type nonrec turn_pierce = json_number
  type nonrec turn_impact = json_number
  type nonrec turn_damage = json_number
  type nonrec turn_spread = json_number
  type nonrec turn_count = json_number

  type nonrec turn_damage_type =
    fragments_native_state38_ground_items_item_item_attacks_item_damage_type

  type nonrec turn_category = fragments_native_state38_ground_items_item_item_attacks_item_category
  type nonrec turn_drain = json_number
  type nonrec turn_amount = json_number
  type nonrec turn_group = json_string
  type nonrec turn_profile = save41_member0_checkpoint_state_vision_groups_item_sight
  type nonrec turn_reveal_within = json_number

  type nonrec turn_terrains =
    save43_member0_checkpoint_state_units_item_perception_effects_item_concealment_item_terrains_member1_item

  type nonrec turn_predicate = json_number
  type nonrec turn_turns = json_number
  type nonrec turn_last_round = json_number
  type nonrec turn_light_remembered = json_boolean
  type nonrec turn_area = save39_member0_checkpoint_state_ground_items_item_tile

  type nonrec turn_condition_case =
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0

  type nonrec turn_unit_types = json_string
  type nonrec turn_unit_type = json_string
  type nonrec turn_choice = json_string
  type nonrec turn_flag = json_string
  type nonrec turn_conditions = save40_member0_checkpoint_state_tiles_item_fog_member1_condition

  type nonrec turn_tile_case =
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member1

  type nonrec turn_hex = save39_member0_checkpoint_state_ground_items_item_tile

  type nonrec turn_binding =
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member3_binding
end

module V44 = struct
  type nonrec t = save44
  type nonrec t_case = save44_member0
  type nonrec pending_enemy = json_boolean
  type nonrec undo = save34_undo_member1_item
  type nonrec progress = save39_member0_progress
  type nonrec version = literal_44
  type nonrec checkpoint = save44_member0_checkpoint
  type nonrec setup = save42_member0_setup
  type nonrec round = json_number
  type nonrec unit_id = json_string
  type nonrec changes = save34_undo_member1_item_changes_item
  type nonrec entry = save37_progress_entry_member0
  type nonrec seed = json_number
  type nonrec state = save44_member0_checkpoint_state
  type nonrec first = save41_member0_setup_first
  type nonrec enemy = save39_member0_setup_enemy
  type nonrec player = save42_member0_setup_player
  type nonrec changes_case = save34_undo_member1_item_changes_item_member0
  type nonrec kind = literal_flash
  type nonrec id = json_string
  type nonrec saved_at = json_number
  type nonrec run = json_string
  type nonrec mission = json_string
  type nonrec side = json_string
  type nonrec mission_start = json_boolean
  type nonrec run_id = json_string
  type nonrec branch_id = json_string
  type nonrec checkpoint_id = json_string
  type nonrec ordinal = json_number
  type nonrec vision_groups = save41_member0_checkpoint_state_vision_groups_item
  type nonrec faction = save41_member0_checkpoint_state_factions_item
  type nonrec ground_items = save44_member0_checkpoint_state_ground_items_item
  type nonrec armies = fragments_native_state38_armies_member1
  type nonrec condition_memory = json_string
  type nonrec map_id = json_string
  type nonrec map_revision = json_number
  type nonrec cols = json_number
  type nonrec rows = json_number
  type nonrec gold = fragments_native_state38_gold
  type nonrec winner = save41_member0_checkpoint_state_winner
  type nonrec next_id = json_number
  type nonrec combat_rng = json_number
  type nonrec log = json_string
  type nonrec tile = save44_member0_checkpoint_state_tiles_item
  type nonrec unit_ = save44_member0_checkpoint_state_units_item
  type nonrec campaign = save44_member0_checkpoint_state_campaign_member1
  type nonrec last_combat = fragments_native_state38_last_combat_member2
  type nonrec blocks = save35_setup_enemy_blocks_member1
  type nonrec policy = save35_setup_enemy_policy
  type nonrec diplomacy = save35_setup_enemy_diplomacy
  type nonrec recruitment_limit = json_number
  type nonrec manual = json_boolean
  type nonrec parent_id = json_string
  type nonrec op = literal_set
  type nonrec path = json_string
  type nonrec value = json_unknown
  type nonrec sight = save41_member0_checkpoint_state_vision_groups_item_sight
  type nonrec explored = save39_member0_checkpoint_state_ground_items_item_tile
  type nonrec vision_group = json_string
  type nonrec phase = save41_member0_setup_first
  type nonrec controller = fragments_factions7_item_controller
  type nonrec palette = json_string
  type nonrec name = json_string
  type nonrec recruits = fragments_factions7_item_recruits_member1_item
  type nonrec relations = save41_member0_checkpoint_state_factions_item_relations_member1
  type nonrec transfer = save41_member0_checkpoint_state_factions_item_transfer_member1
  type nonrec item = save44_member0_checkpoint_state_ground_items_item_item
  type nonrec location = json_string
  type nonrec col = json_number
  type nonrec row = json_number
  type nonrec terrain = json_string
  type nonrec owner = json_string
  type nonrec connection = fragments_tiles11_item_connections_member1_item
  type nonrec dead_ends = fragments_tiles11_item_connections_member1_item
  type nonrec walls = fragments_tiles11_item_connections_member1_item
  type nonrec floor = json_string
  type nonrec perception = save44_member0_checkpoint_state_tiles_item_perception
  type nonrec light = save41_member0_checkpoint_state_tiles_item_light_member1
  type nonrec revealed_to = json_string

  type nonrec perception_effects =
    save44_member0_checkpoint_state_units_item_perception_effects_item

  type nonrec items = save44_member0_checkpoint_state_ground_items_item_item
  type nonrec ai_group = json_string
  type nonrec type_ = json_string
  type nonrec hp = json_number
  type nonrec moved = json_boolean
  type nonrec attack_spent = json_boolean
  type nonrec experience = json_number
  type nonrec character = json_string
  type nonrec role = fragments_native_state38_units_item_role
  type nonrec retreating = json_boolean
  type nonrec rebuilding = json_boolean
  type nonrec scripts = save41_member0_checkpoint_state_campaign_member1_scripts_member1
  type nonrec evacuated = save44_member0_checkpoint_state_campaign_member1_evacuated_member1_item
  type nonrec recall = json_string
  type nonrec veteran = save44_member0_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec story_veteran = save44_member0_checkpoint_state_campaign_member1_veterans_member1_item
  type nonrec fallen_companions = json_string
  type nonrec carry_gold = json_number
  type nonrec mark = json_string
  type nonrec dialogue = save41_member0_checkpoint_state_campaign_member1_dialogue_item
  type nonrec dialogue_index = json_number
  type nonrec outcome = json_string
  type nonrec completed = json_boolean
  type nonrec refuge = fragments_native_state38_campaign_member1_refuge_member1
  type nonrec vigil = fragments_native_state38_campaign_member1_vigil_member1
  type nonrec hold = fragments_native_state38_campaign_member1_holds_member1
  type nonrec flags = fragments_native_state38_campaign_member1_flags
  type nonrec attacker_id = json_string
  type nonrec attack_id = json_string
  type nonrec sound = json_string
  type nonrec recruiter = save35_setup_enemy_blocks_member1_recruiter
  type nonrec planner = save35_setup_enemy_blocks_member1_planner
  type nonrec range = save41_member0_checkpoint_state_vision_groups_item_sight_range
  type nonrec penetrates = save41_member0_checkpoint_state_vision_groups_item_sight_penetrates_item
  type nonrec controller_case = fragments_factions7_item_controller_member0
  type nonrec cost = json_number
  type nonrec to_ = json_string
  type nonrec rule = save41_member0_checkpoint_state_factions_item_transfer_member1_when_
  type nonrec armor = fragments_native_state38_ground_items_item_item_armor_member1
  type nonrec description = json_string
  type nonrec attacks = save41_member0_checkpoint_state_ground_items_item_item_attacks_item
  type nonrec healing = fragments_native_state38_ground_items_item_item_healing_member1
  type nonrec remember = json_boolean
  type nonrec concealment = save44_member0_checkpoint_state_tiles_item_perception_concealment_item
  type nonrec emission = save41_member0_checkpoint_state_tiles_item_light_member1_emission_member1
  type nonrec color = fragments_light23_color_member1
  type nonrec fixture = json_boolean
  type nonrec dx = json_number
  type nonrec dy = json_number
  type nonrec radius = json_number
  type nonrec power = json_number

  type nonrec expires_at =
    save41_member0_checkpoint_state_units_item_perception_effects_item_expires_at_member1

  type nonrec failed = json_string
  type nonrec revealed = json_string
  type nonrec recruitment = fragments_native_state38_campaign_member1_scripts_member1_recruitment
  type nonrec active = json_string

  type nonrec pending =
    save41_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item

  type nonrec next_mark = json_number
  type nonrec event = json_string
  type nonrec speaker = json_string
  type nonrec portrait = json_string
  type nonrec text = json_string
  type nonrec held_turns = json_number
  type nonrec last_counted_round = json_number
  type nonrec escalated = json_boolean
  type nonrec condition = save40_member0_checkpoint_state_tiles_item_fog_member1_condition
  type nonrec blade = json_number
  type nonrec pierce = json_number
  type nonrec impact = json_number
  type nonrec damage = json_number
  type nonrec spread = json_number
  type nonrec count = json_number
  type nonrec damage_type = fragments_native_state38_ground_items_item_item_attacks_item_damage_type
  type nonrec category = fragments_native_state38_ground_items_item_item_attacks_item_category
  type nonrec drain = json_number
  type nonrec amount = json_number
  type nonrec group = json_string
  type nonrec profile = save41_member0_checkpoint_state_vision_groups_item_sight
  type nonrec reveal_within = json_number

  type nonrec requires =
    save44_member0_checkpoint_state_tiles_item_perception_concealment_item_requires

  type nonrec predicate = json_number
  type nonrec turns = json_number
  type nonrec last_round = json_number
  type nonrec light_remembered = json_boolean
  type nonrec area = save39_member0_checkpoint_state_ground_items_item_tile

  type nonrec condition_case =
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0

  type nonrec terrains =
    save44_member0_checkpoint_state_tiles_item_perception_concealment_item_requires_terrains_member1_item

  type nonrec illumination =
    save44_member0_checkpoint_state_tiles_item_perception_concealment_item_requires_illumination

  type nonrec unit_types = json_string
  type nonrec unit_type = json_string
  type nonrec choice = json_string
  type nonrec flag = json_string
  type nonrec conditions = save40_member0_checkpoint_state_tiles_item_fog_member1_condition

  type nonrec tile_case =
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member1

  type nonrec hex = save39_member0_checkpoint_state_ground_items_item_tile

  type nonrec binding =
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member3_binding

  type nonrec current = save44_member0
  type nonrec current_pending_enemy = json_boolean
  type nonrec current_undo = save34_undo_member1_item
  type nonrec current_progress = save39_member0_progress
  type nonrec current_version = literal_44
  type nonrec current_checkpoint = save44_member0_checkpoint
  type nonrec current_setup = save42_member0_setup
  type nonrec current_round = json_number
  type nonrec current_unit_id = json_string
  type nonrec current_changes = save34_undo_member1_item_changes_item
  type nonrec current_entry = save37_progress_entry_member0
  type nonrec current_seed = json_number
  type nonrec current_state = save44_member0_checkpoint_state
  type nonrec current_first = save41_member0_setup_first
  type nonrec current_enemy = save39_member0_setup_enemy
  type nonrec current_player = save42_member0_setup_player
  type nonrec current_changes_case = save34_undo_member1_item_changes_item_member0
  type nonrec current_kind = literal_flash
  type nonrec current_id = json_string
  type nonrec current_saved_at = json_number
  type nonrec current_run = json_string
  type nonrec current_mission = json_string
  type nonrec current_side = json_string
  type nonrec current_mission_start = json_boolean
  type nonrec current_run_id = json_string
  type nonrec current_branch_id = json_string
  type nonrec current_checkpoint_id = json_string
  type nonrec current_ordinal = json_number
  type nonrec current_vision_groups = save41_member0_checkpoint_state_vision_groups_item
  type nonrec current_faction = save41_member0_checkpoint_state_factions_item
  type nonrec current_ground_items = save44_member0_checkpoint_state_ground_items_item
  type nonrec current_armies = fragments_native_state38_armies_member1
  type nonrec current_condition_memory = json_string
  type nonrec current_map_id = json_string
  type nonrec current_map_revision = json_number
  type nonrec current_cols = json_number
  type nonrec current_rows = json_number
  type nonrec current_gold = fragments_native_state38_gold
  type nonrec current_winner = save41_member0_checkpoint_state_winner
  type nonrec current_next_id = json_number
  type nonrec current_combat_rng = json_number
  type nonrec current_log = json_string
  type nonrec current_tile = save44_member0_checkpoint_state_tiles_item
  type nonrec current_unit_ = save44_member0_checkpoint_state_units_item
  type nonrec current_campaign = save44_member0_checkpoint_state_campaign_member1
  type nonrec current_last_combat = fragments_native_state38_last_combat_member2
  type nonrec current_blocks = save35_setup_enemy_blocks_member1
  type nonrec current_policy = save35_setup_enemy_policy
  type nonrec current_diplomacy = save35_setup_enemy_diplomacy
  type nonrec current_recruitment_limit = json_number
  type nonrec current_op = literal_set
  type nonrec current_path = json_string
  type nonrec current_value = json_unknown
  type nonrec current_sight = save41_member0_checkpoint_state_vision_groups_item_sight
  type nonrec current_explored = save39_member0_checkpoint_state_ground_items_item_tile
  type nonrec current_vision_group = json_string
  type nonrec current_phase = save41_member0_setup_first
  type nonrec current_controller = fragments_factions7_item_controller
  type nonrec current_palette = json_string
  type nonrec current_name = json_string
  type nonrec current_recruits = fragments_factions7_item_recruits_member1_item
  type nonrec current_relations = save41_member0_checkpoint_state_factions_item_relations_member1
  type nonrec current_transfer = save41_member0_checkpoint_state_factions_item_transfer_member1
  type nonrec current_item = save44_member0_checkpoint_state_ground_items_item_item
  type nonrec current_location = json_string
  type nonrec current_col = json_number
  type nonrec current_row = json_number
  type nonrec current_terrain = json_string
  type nonrec current_owner = json_string
  type nonrec current_connection = fragments_tiles11_item_connections_member1_item
  type nonrec current_dead_ends = fragments_tiles11_item_connections_member1_item
  type nonrec current_walls = fragments_tiles11_item_connections_member1_item
  type nonrec current_floor = json_string
  type nonrec current_perception = save44_member0_checkpoint_state_tiles_item_perception
  type nonrec current_light = save41_member0_checkpoint_state_tiles_item_light_member1
  type nonrec current_revealed_to = json_string

  type nonrec current_perception_effects =
    save44_member0_checkpoint_state_units_item_perception_effects_item

  type nonrec current_items = save44_member0_checkpoint_state_ground_items_item_item
  type nonrec current_ai_group = json_string
  type nonrec current_type_ = json_string
  type nonrec current_hp = json_number
  type nonrec current_moved = json_boolean
  type nonrec current_attack_spent = json_boolean
  type nonrec current_experience = json_number
  type nonrec current_character = json_string
  type nonrec current_role = fragments_native_state38_units_item_role
  type nonrec current_retreating = json_boolean
  type nonrec current_rebuilding = json_boolean
  type nonrec current_scripts = save41_member0_checkpoint_state_campaign_member1_scripts_member1

  type nonrec current_evacuated =
    save44_member0_checkpoint_state_campaign_member1_evacuated_member1_item

  type nonrec current_recall = json_string

  type nonrec current_veteran =
    save44_member0_checkpoint_state_campaign_member1_veterans_member1_item

  type nonrec current_story_veteran =
    save44_member0_checkpoint_state_campaign_member1_veterans_member1_item

  type nonrec current_fallen_companions = json_string
  type nonrec current_carry_gold = json_number
  type nonrec current_mark = json_string
  type nonrec current_dialogue = save41_member0_checkpoint_state_campaign_member1_dialogue_item
  type nonrec current_dialogue_index = json_number
  type nonrec current_outcome = json_string
  type nonrec current_completed = json_boolean
  type nonrec current_refuge = fragments_native_state38_campaign_member1_refuge_member1
  type nonrec current_vigil = fragments_native_state38_campaign_member1_vigil_member1
  type nonrec current_hold = fragments_native_state38_campaign_member1_holds_member1
  type nonrec current_flags = fragments_native_state38_campaign_member1_flags
  type nonrec current_attacker_id = json_string
  type nonrec current_attack_id = json_string
  type nonrec current_sound = json_string
  type nonrec current_recruiter = save35_setup_enemy_blocks_member1_recruiter
  type nonrec current_planner = save35_setup_enemy_blocks_member1_planner
  type nonrec current_range = save41_member0_checkpoint_state_vision_groups_item_sight_range

  type nonrec current_penetrates =
    save41_member0_checkpoint_state_vision_groups_item_sight_penetrates_item

  type nonrec current_controller_case = fragments_factions7_item_controller_member0
  type nonrec current_cost = json_number
  type nonrec current_to_ = json_string
  type nonrec current_rule = save41_member0_checkpoint_state_factions_item_transfer_member1_when_
  type nonrec current_armor = fragments_native_state38_ground_items_item_item_armor_member1
  type nonrec current_description = json_string
  type nonrec current_attacks = save41_member0_checkpoint_state_ground_items_item_item_attacks_item
  type nonrec current_healing = fragments_native_state38_ground_items_item_item_healing_member1
  type nonrec current_remember = json_boolean

  type nonrec current_concealment =
    save44_member0_checkpoint_state_tiles_item_perception_concealment_item

  type nonrec current_emission =
    save41_member0_checkpoint_state_tiles_item_light_member1_emission_member1

  type nonrec current_color = fragments_light23_color_member1
  type nonrec current_fixture = json_boolean
  type nonrec current_dx = json_number
  type nonrec current_dy = json_number
  type nonrec current_radius = json_number
  type nonrec current_power = json_number

  type nonrec current_expires_at =
    save41_member0_checkpoint_state_units_item_perception_effects_item_expires_at_member1

  type nonrec current_failed = json_string
  type nonrec current_revealed = json_string

  type nonrec current_recruitment =
    fragments_native_state38_campaign_member1_scripts_member1_recruitment

  type nonrec current_active = json_string

  type nonrec current_pending =
    save41_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item

  type nonrec current_next_mark = json_number
  type nonrec current_event = json_string
  type nonrec current_speaker = json_string
  type nonrec current_portrait = json_string
  type nonrec current_text = json_string
  type nonrec current_held_turns = json_number
  type nonrec current_last_counted_round = json_number
  type nonrec current_escalated = json_boolean
  type nonrec current_condition = save40_member0_checkpoint_state_tiles_item_fog_member1_condition
  type nonrec current_blade = json_number
  type nonrec current_pierce = json_number
  type nonrec current_impact = json_number
  type nonrec current_damage = json_number
  type nonrec current_spread = json_number
  type nonrec current_count = json_number

  type nonrec current_damage_type =
    fragments_native_state38_ground_items_item_item_attacks_item_damage_type

  type nonrec current_category =
    fragments_native_state38_ground_items_item_item_attacks_item_category

  type nonrec current_drain = json_number
  type nonrec current_amount = json_number
  type nonrec current_group = json_string
  type nonrec current_profile = save41_member0_checkpoint_state_vision_groups_item_sight
  type nonrec current_reveal_within = json_number

  type nonrec current_requires =
    save44_member0_checkpoint_state_tiles_item_perception_concealment_item_requires

  type nonrec current_predicate = json_number
  type nonrec current_turns = json_number
  type nonrec current_last_round = json_number
  type nonrec current_light_remembered = json_boolean
  type nonrec current_area = save39_member0_checkpoint_state_ground_items_item_tile

  type nonrec current_condition_case =
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0

  type nonrec current_terrains =
    save44_member0_checkpoint_state_tiles_item_perception_concealment_item_requires_terrains_member1_item

  type nonrec current_illumination =
    save44_member0_checkpoint_state_tiles_item_perception_concealment_item_requires_illumination

  type nonrec current_unit_types = json_string
  type nonrec current_unit_type = json_string
  type nonrec current_choice = json_string
  type nonrec current_flag = json_string
  type nonrec current_conditions = save40_member0_checkpoint_state_tiles_item_fog_member1_condition

  type nonrec current_tile_case =
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member1

  type nonrec current_hex = save39_member0_checkpoint_state_ground_items_item_tile

  type nonrec current_binding =
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member3_binding

  type nonrec turn = save44_member1
  type nonrec turn_pending_enemy = literal_false
  type nonrec turn_undo = json_undefined
  type nonrec turn_progress = save39_member1_progress
  type nonrec turn_version = literal_44
  type nonrec turn_checkpoint = save44_member0_checkpoint
  type nonrec turn_setup = save42_member0_setup
  type nonrec turn_entry = history4_entries_item
  type nonrec turn_seed = json_number
  type nonrec turn_state = save44_member0_checkpoint_state
  type nonrec turn_first = save41_member0_setup_first
  type nonrec turn_enemy = save39_member0_setup_enemy
  type nonrec turn_player = save42_member0_setup_player
  type nonrec turn_kind = literal_turn
  type nonrec turn_manual = json_boolean
  type nonrec turn_parent_id = json_string
  type nonrec turn_id = json_string
  type nonrec turn_saved_at = json_number
  type nonrec turn_run = json_string
  type nonrec turn_mission = json_string
  type nonrec turn_round = json_number
  type nonrec turn_side = json_string
  type nonrec turn_mission_start = json_boolean
  type nonrec turn_run_id = json_string
  type nonrec turn_branch_id = json_string
  type nonrec turn_checkpoint_id = json_string
  type nonrec turn_ordinal = json_number
  type nonrec turn_vision_groups = save41_member0_checkpoint_state_vision_groups_item
  type nonrec turn_faction = save41_member0_checkpoint_state_factions_item
  type nonrec turn_ground_items = save44_member0_checkpoint_state_ground_items_item
  type nonrec turn_armies = fragments_native_state38_armies_member1
  type nonrec turn_condition_memory = json_string
  type nonrec turn_map_id = json_string
  type nonrec turn_map_revision = json_number
  type nonrec turn_cols = json_number
  type nonrec turn_rows = json_number
  type nonrec turn_gold = fragments_native_state38_gold
  type nonrec turn_winner = save41_member0_checkpoint_state_winner
  type nonrec turn_next_id = json_number
  type nonrec turn_combat_rng = json_number
  type nonrec turn_log = json_string
  type nonrec turn_tile = save44_member0_checkpoint_state_tiles_item
  type nonrec turn_unit_ = save44_member0_checkpoint_state_units_item
  type nonrec turn_campaign = save44_member0_checkpoint_state_campaign_member1
  type nonrec turn_last_combat = fragments_native_state38_last_combat_member2
  type nonrec turn_blocks = save35_setup_enemy_blocks_member1
  type nonrec turn_policy = save35_setup_enemy_policy
  type nonrec turn_diplomacy = save35_setup_enemy_diplomacy
  type nonrec turn_recruitment_limit = json_number
  type nonrec turn_sight = save41_member0_checkpoint_state_vision_groups_item_sight
  type nonrec turn_explored = save39_member0_checkpoint_state_ground_items_item_tile
  type nonrec turn_vision_group = json_string
  type nonrec turn_phase = save41_member0_setup_first
  type nonrec turn_controller = fragments_factions7_item_controller
  type nonrec turn_palette = json_string
  type nonrec turn_name = json_string
  type nonrec turn_recruits = fragments_factions7_item_recruits_member1_item
  type nonrec turn_relations = save41_member0_checkpoint_state_factions_item_relations_member1
  type nonrec turn_transfer = save41_member0_checkpoint_state_factions_item_transfer_member1
  type nonrec turn_item = save44_member0_checkpoint_state_ground_items_item_item
  type nonrec turn_location = json_string
  type nonrec turn_col = json_number
  type nonrec turn_row = json_number
  type nonrec turn_terrain = json_string
  type nonrec turn_owner = json_string
  type nonrec turn_connection = fragments_tiles11_item_connections_member1_item
  type nonrec turn_dead_ends = fragments_tiles11_item_connections_member1_item
  type nonrec turn_walls = fragments_tiles11_item_connections_member1_item
  type nonrec turn_floor = json_string
  type nonrec turn_perception = save44_member0_checkpoint_state_tiles_item_perception
  type nonrec turn_light = save41_member0_checkpoint_state_tiles_item_light_member1
  type nonrec turn_revealed_to = json_string

  type nonrec turn_perception_effects =
    save44_member0_checkpoint_state_units_item_perception_effects_item

  type nonrec turn_items = save44_member0_checkpoint_state_ground_items_item_item
  type nonrec turn_ai_group = json_string
  type nonrec turn_type_ = json_string
  type nonrec turn_hp = json_number
  type nonrec turn_moved = json_boolean
  type nonrec turn_attack_spent = json_boolean
  type nonrec turn_experience = json_number
  type nonrec turn_character = json_string
  type nonrec turn_role = fragments_native_state38_units_item_role
  type nonrec turn_retreating = json_boolean
  type nonrec turn_rebuilding = json_boolean
  type nonrec turn_scripts = save41_member0_checkpoint_state_campaign_member1_scripts_member1

  type nonrec turn_evacuated =
    save44_member0_checkpoint_state_campaign_member1_evacuated_member1_item

  type nonrec turn_recall = json_string
  type nonrec turn_veteran = save44_member0_checkpoint_state_campaign_member1_veterans_member1_item

  type nonrec turn_story_veteran =
    save44_member0_checkpoint_state_campaign_member1_veterans_member1_item

  type nonrec turn_fallen_companions = json_string
  type nonrec turn_carry_gold = json_number
  type nonrec turn_mark = json_string
  type nonrec turn_dialogue = save41_member0_checkpoint_state_campaign_member1_dialogue_item
  type nonrec turn_dialogue_index = json_number
  type nonrec turn_outcome = json_string
  type nonrec turn_completed = json_boolean
  type nonrec turn_refuge = fragments_native_state38_campaign_member1_refuge_member1
  type nonrec turn_vigil = fragments_native_state38_campaign_member1_vigil_member1
  type nonrec turn_hold = fragments_native_state38_campaign_member1_holds_member1
  type nonrec turn_flags = fragments_native_state38_campaign_member1_flags
  type nonrec turn_attacker_id = json_string
  type nonrec turn_attack_id = json_string
  type nonrec turn_sound = json_string
  type nonrec turn_recruiter = save35_setup_enemy_blocks_member1_recruiter
  type nonrec turn_planner = save35_setup_enemy_blocks_member1_planner
  type nonrec turn_range = save41_member0_checkpoint_state_vision_groups_item_sight_range

  type nonrec turn_penetrates =
    save41_member0_checkpoint_state_vision_groups_item_sight_penetrates_item

  type nonrec turn_controller_case = fragments_factions7_item_controller_member0
  type nonrec turn_cost = json_number
  type nonrec turn_to_ = json_string
  type nonrec turn_rule = save41_member0_checkpoint_state_factions_item_transfer_member1_when_
  type nonrec turn_armor = fragments_native_state38_ground_items_item_item_armor_member1
  type nonrec turn_description = json_string
  type nonrec turn_attacks = save41_member0_checkpoint_state_ground_items_item_item_attacks_item
  type nonrec turn_healing = fragments_native_state38_ground_items_item_item_healing_member1
  type nonrec turn_remember = json_boolean

  type nonrec turn_concealment =
    save44_member0_checkpoint_state_tiles_item_perception_concealment_item

  type nonrec turn_emission =
    save41_member0_checkpoint_state_tiles_item_light_member1_emission_member1

  type nonrec turn_color = fragments_light23_color_member1
  type nonrec turn_fixture = json_boolean
  type nonrec turn_dx = json_number
  type nonrec turn_dy = json_number
  type nonrec turn_radius = json_number
  type nonrec turn_power = json_number

  type nonrec turn_expires_at =
    save41_member0_checkpoint_state_units_item_perception_effects_item_expires_at_member1

  type nonrec turn_failed = json_string
  type nonrec turn_revealed = json_string

  type nonrec turn_recruitment =
    fragments_native_state38_campaign_member1_scripts_member1_recruitment

  type nonrec turn_active = json_string

  type nonrec turn_pending =
    save41_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item

  type nonrec turn_next_mark = json_number
  type nonrec turn_event = json_string
  type nonrec turn_speaker = json_string
  type nonrec turn_portrait = json_string
  type nonrec turn_text = json_string
  type nonrec turn_held_turns = json_number
  type nonrec turn_last_counted_round = json_number
  type nonrec turn_escalated = json_boolean
  type nonrec turn_condition = save40_member0_checkpoint_state_tiles_item_fog_member1_condition
  type nonrec turn_blade = json_number
  type nonrec turn_pierce = json_number
  type nonrec turn_impact = json_number
  type nonrec turn_damage = json_number
  type nonrec turn_spread = json_number
  type nonrec turn_count = json_number

  type nonrec turn_damage_type =
    fragments_native_state38_ground_items_item_item_attacks_item_damage_type

  type nonrec turn_category = fragments_native_state38_ground_items_item_item_attacks_item_category
  type nonrec turn_drain = json_number
  type nonrec turn_amount = json_number
  type nonrec turn_group = json_string
  type nonrec turn_profile = save41_member0_checkpoint_state_vision_groups_item_sight
  type nonrec turn_reveal_within = json_number

  type nonrec turn_requires =
    save44_member0_checkpoint_state_tiles_item_perception_concealment_item_requires

  type nonrec turn_predicate = json_number
  type nonrec turn_turns = json_number
  type nonrec turn_last_round = json_number
  type nonrec turn_light_remembered = json_boolean
  type nonrec turn_area = save39_member0_checkpoint_state_ground_items_item_tile

  type nonrec turn_condition_case =
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0

  type nonrec turn_terrains =
    save44_member0_checkpoint_state_tiles_item_perception_concealment_item_requires_terrains_member1_item

  type nonrec turn_illumination =
    save44_member0_checkpoint_state_tiles_item_perception_concealment_item_requires_illumination

  type nonrec turn_unit_types = json_string
  type nonrec turn_unit_type = json_string
  type nonrec turn_choice = json_string
  type nonrec turn_flag = json_string
  type nonrec turn_conditions = save40_member0_checkpoint_state_tiles_item_fog_member1_condition

  type nonrec turn_tile_case =
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member1

  type nonrec turn_hex = save39_member0_checkpoint_state_ground_items_item_tile

  type nonrec turn_binding =
    save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member3_binding
end
