(* Generated from frozen src/formats/readers.json. Unchecked decoder helpers are private. *)
open Historical_types

val wire_to_json : known:string list -> wire -> (string * Js.Json.t) list -> Js.Json.t
val read_save2 : Js.Json.t -> save2
val encode_save2 : save2 -> Js.Json.t
val read_literal_2 : Js.Json.t -> literal_2
val encode_literal_2 : literal_2 -> Js.Json.t
val read_save2_checkpoint : Js.Json.t -> save2_checkpoint
val encode_save2_checkpoint : save2_checkpoint -> Js.Json.t
val read_json_number : Js.Json.t -> json_number
val encode_json_number : json_number -> Js.Json.t
val read_save2_checkpoint_state : Js.Json.t -> save2_checkpoint_state
val encode_save2_checkpoint_state : save2_checkpoint_state -> Js.Json.t
val read_save35_setup_first : Js.Json.t -> save35_setup_first
val encode_save35_setup_first : save35_setup_first -> Js.Json.t
val read_literal_player : Js.Json.t -> literal_player
val encode_literal_player : literal_player -> Js.Json.t
val read_literal_enemy : Js.Json.t -> literal_enemy
val encode_literal_enemy : literal_enemy -> Js.Json.t
val read_fragments_native_state38_winner : Js.Json.t -> fragments_native_state38_winner
val encode_fragments_native_state38_winner : fragments_native_state38_winner -> Js.Json.t
val read_json_null : Js.Json.t -> json_null
val encode_json_null : json_null -> Js.Json.t
val read_fragments_native_state38_armies : Js.Json.t -> fragments_native_state38_armies
val encode_fragments_native_state38_armies : fragments_native_state38_armies -> Js.Json.t
val read_json_undefined : Js.Json.t -> json_undefined
val encode_json_undefined : json_undefined -> Js.Json.t

val read_fragments_native_state38_armies_member1 :
  Js.Json.t -> fragments_native_state38_armies_member1

val encode_fragments_native_state38_armies_member1 :
  fragments_native_state38_armies_member1 -> Js.Json.t

val read_json_string : Js.Json.t -> json_string
val encode_json_string : json_string -> Js.Json.t

val read_fragments_native_state38_condition_memory :
  Js.Json.t -> fragments_native_state38_condition_memory

val encode_fragments_native_state38_condition_memory :
  fragments_native_state38_condition_memory -> Js.Json.t

val read_fragments_native_state38_log : Js.Json.t -> fragments_native_state38_log
val encode_fragments_native_state38_log : fragments_native_state38_log -> Js.Json.t
val read_fragments_native_state38_last_combat : Js.Json.t -> fragments_native_state38_last_combat
val encode_fragments_native_state38_last_combat : fragments_native_state38_last_combat -> Js.Json.t

val read_fragments_native_state38_last_combat_member2 :
  Js.Json.t -> fragments_native_state38_last_combat_member2

val encode_fragments_native_state38_last_combat_member2 :
  fragments_native_state38_last_combat_member2 -> Js.Json.t

val read_save2_checkpoint_state_units : Js.Json.t -> save2_checkpoint_state_units
val encode_save2_checkpoint_state_units : save2_checkpoint_state_units -> Js.Json.t
val read_save2_checkpoint_state_units_item : Js.Json.t -> save2_checkpoint_state_units_item
val encode_save2_checkpoint_state_units_item : save2_checkpoint_state_units_item -> Js.Json.t

val read_fragments_native_state38_units_item_role :
  Js.Json.t -> fragments_native_state38_units_item_role

val encode_fragments_native_state38_units_item_role :
  fragments_native_state38_units_item_role -> Js.Json.t

val read_literal_leader : Js.Json.t -> literal_leader
val encode_literal_leader : literal_leader -> Js.Json.t
val read_literal_courier : Js.Json.t -> literal_courier
val encode_literal_courier : literal_courier -> Js.Json.t
val read_literal_captive : Js.Json.t -> literal_captive
val encode_literal_captive : literal_captive -> Js.Json.t
val read_javascript1_state_map_id : Js.Json.t -> javascript1_state_map_id
val encode_javascript1_state_map_id : javascript1_state_map_id -> Js.Json.t

val read_fragments_native_state38_units_item_items :
  Js.Json.t -> fragments_native_state38_units_item_items

val encode_fragments_native_state38_units_item_items :
  fragments_native_state38_units_item_items -> Js.Json.t

val read_fragments_native_state38_units_item_items_member1 :
  Js.Json.t -> fragments_native_state38_units_item_items_member1

val encode_fragments_native_state38_units_item_items_member1 :
  fragments_native_state38_units_item_items_member1 -> Js.Json.t

val read_fragments_native_state38_ground_items_item_item :
  Js.Json.t -> fragments_native_state38_ground_items_item_item

val encode_fragments_native_state38_ground_items_item_item :
  fragments_native_state38_ground_items_item_item -> Js.Json.t

val read_fragments_native_state38_ground_items_item_item_armor :
  Js.Json.t -> fragments_native_state38_ground_items_item_item_armor

val encode_fragments_native_state38_ground_items_item_item_armor :
  fragments_native_state38_ground_items_item_item_armor -> Js.Json.t

val read_fragments_native_state38_ground_items_item_item_armor_member1 :
  Js.Json.t -> fragments_native_state38_ground_items_item_item_armor_member1

val encode_fragments_native_state38_ground_items_item_item_armor_member1 :
  fragments_native_state38_ground_items_item_item_armor_member1 -> Js.Json.t

val read_fragments_native_state38_ground_items_item_item_attacks :
  Js.Json.t -> fragments_native_state38_ground_items_item_item_attacks

val encode_fragments_native_state38_ground_items_item_item_attacks :
  fragments_native_state38_ground_items_item_item_attacks -> Js.Json.t

val read_fragments_native_state38_ground_items_item_item_attacks_item :
  Js.Json.t -> fragments_native_state38_ground_items_item_item_attacks_item

val encode_fragments_native_state38_ground_items_item_item_attacks_item :
  fragments_native_state38_ground_items_item_item_attacks_item -> Js.Json.t

val read_fragments_light23_dx : Js.Json.t -> fragments_light23_dx
val encode_fragments_light23_dx : fragments_light23_dx -> Js.Json.t

val read_fragments_native_state38_ground_items_item_item_attacks_item_damage_type :
  Js.Json.t -> fragments_native_state38_ground_items_item_item_attacks_item_damage_type

val encode_fragments_native_state38_ground_items_item_item_attacks_item_damage_type :
  fragments_native_state38_ground_items_item_item_attacks_item_damage_type -> Js.Json.t

val read_literal_blade : Js.Json.t -> literal_blade
val encode_literal_blade : literal_blade -> Js.Json.t
val read_literal_pierce : Js.Json.t -> literal_pierce
val encode_literal_pierce : literal_pierce -> Js.Json.t
val read_literal_impact : Js.Json.t -> literal_impact
val encode_literal_impact : literal_impact -> Js.Json.t

val read_fragments_native_state38_ground_items_item_item_attacks_item_category :
  Js.Json.t -> fragments_native_state38_ground_items_item_item_attacks_item_category

val encode_fragments_native_state38_ground_items_item_item_attacks_item_category :
  fragments_native_state38_ground_items_item_item_attacks_item_category -> Js.Json.t

val read_literal_melee : Js.Json.t -> literal_melee
val encode_literal_melee : literal_melee -> Js.Json.t
val read_literal_ranged : Js.Json.t -> literal_ranged
val encode_literal_ranged : literal_ranged -> Js.Json.t

val read_fragments_native_state38_ground_items_item_item_healing :
  Js.Json.t -> fragments_native_state38_ground_items_item_item_healing

val encode_fragments_native_state38_ground_items_item_item_healing :
  fragments_native_state38_ground_items_item_item_healing -> Js.Json.t

val read_fragments_native_state38_ground_items_item_item_healing_member1 :
  Js.Json.t -> fragments_native_state38_ground_items_item_item_healing_member1

val encode_fragments_native_state38_ground_items_item_item_healing_member1 :
  fragments_native_state38_ground_items_item_item_healing_member1 -> Js.Json.t

val read_json_boolean : Js.Json.t -> json_boolean
val encode_json_boolean : json_boolean -> Js.Json.t
val read_fragments_rule11_remember : Js.Json.t -> fragments_rule11_remember
val encode_fragments_rule11_remember : fragments_rule11_remember -> Js.Json.t
val read_literal_false : Js.Json.t -> literal_false
val encode_literal_false : literal_false -> Js.Json.t
val read_literal_true : Js.Json.t -> literal_true
val encode_literal_true : literal_true -> Js.Json.t
val read_save2_checkpoint_state_tiles : Js.Json.t -> save2_checkpoint_state_tiles
val encode_save2_checkpoint_state_tiles : save2_checkpoint_state_tiles -> Js.Json.t
val read_save2_checkpoint_state_tiles_item : Js.Json.t -> save2_checkpoint_state_tiles_item
val encode_save2_checkpoint_state_tiles_item : save2_checkpoint_state_tiles_item -> Js.Json.t
val read_fragments_tiles11_item_owner : Js.Json.t -> fragments_tiles11_item_owner
val encode_fragments_tiles11_item_owner : fragments_tiles11_item_owner -> Js.Json.t
val read_fragments_tiles11_item_connections : Js.Json.t -> fragments_tiles11_item_connections
val encode_fragments_tiles11_item_connections : fragments_tiles11_item_connections -> Js.Json.t

val read_fragments_tiles11_item_connections_member1 :
  Js.Json.t -> fragments_tiles11_item_connections_member1

val encode_fragments_tiles11_item_connections_member1 :
  fragments_tiles11_item_connections_member1 -> Js.Json.t

val read_fragments_tiles11_item_connections_member1_item :
  Js.Json.t -> fragments_tiles11_item_connections_member1_item

val encode_fragments_tiles11_item_connections_member1_item :
  fragments_tiles11_item_connections_member1_item -> Js.Json.t

val read_literal_e : Js.Json.t -> literal_e
val encode_literal_e : literal_e -> Js.Json.t
val read_literal_se : Js.Json.t -> literal_se
val encode_literal_se : literal_se -> Js.Json.t
val read_literal_sw : Js.Json.t -> literal_sw
val encode_literal_sw : literal_sw -> Js.Json.t
val read_literal_w : Js.Json.t -> literal_w
val encode_literal_w : literal_w -> Js.Json.t
val read_literal_nw : Js.Json.t -> literal_nw
val encode_literal_nw : literal_nw -> Js.Json.t
val read_literal_ne : Js.Json.t -> literal_ne
val encode_literal_ne : literal_ne -> Js.Json.t
val read_save2_checkpoint_state_tiles_item_fog : Js.Json.t -> save2_checkpoint_state_tiles_item_fog

val encode_save2_checkpoint_state_tiles_item_fog :
  save2_checkpoint_state_tiles_item_fog -> Js.Json.t

val read_save2_checkpoint_state_tiles_item_fog_member1 :
  Js.Json.t -> save2_checkpoint_state_tiles_item_fog_member1

val encode_save2_checkpoint_state_tiles_item_fog_member1 :
  save2_checkpoint_state_tiles_item_fog_member1 -> Js.Json.t

val read_save2_checkpoint_state_tiles_item_fog_member1_condition :
  Js.Json.t -> save2_checkpoint_state_tiles_item_fog_member1_condition

val encode_save2_checkpoint_state_tiles_item_fog_member1_condition :
  save2_checkpoint_state_tiles_item_fog_member1_condition -> Js.Json.t

val read_fragments_rule11_condition_member0 : Js.Json.t -> fragments_rule11_condition_member0
val encode_fragments_rule11_condition_member0 : fragments_rule11_condition_member0 -> Js.Json.t
val read_literal_owner : Js.Json.t -> literal_owner
val encode_literal_owner : literal_owner -> Js.Json.t
val read_fragments_rule11_condition_member1 : Js.Json.t -> fragments_rule11_condition_member1
val encode_fragments_rule11_condition_member1 : fragments_rule11_condition_member1 -> Js.Json.t
val read_literal_visible_units : Js.Json.t -> literal_visible_units
val encode_literal_visible_units : literal_visible_units -> Js.Json.t

val read_fragments_rule11_condition_member1_side :
  Js.Json.t -> fragments_rule11_condition_member1_side

val encode_fragments_rule11_condition_member1_side :
  fragments_rule11_condition_member1_side -> Js.Json.t

val read_literal_neutral : Js.Json.t -> literal_neutral
val encode_literal_neutral : literal_neutral -> Js.Json.t
val read_literal_any : Js.Json.t -> literal_any
val encode_literal_any : literal_any -> Js.Json.t
val read_fragments_rule11_condition_member2 : Js.Json.t -> fragments_rule11_condition_member2
val encode_fragments_rule11_condition_member2 : fragments_rule11_condition_member2 -> Js.Json.t
val read_literal_choice : Js.Json.t -> literal_choice
val encode_literal_choice : literal_choice -> Js.Json.t
val read_fragments_rule11_condition_member3 : Js.Json.t -> fragments_rule11_condition_member3
val encode_fragments_rule11_condition_member3 : fragments_rule11_condition_member3 -> Js.Json.t
val read_literal_flag : Js.Json.t -> literal_flag
val encode_literal_flag : literal_flag -> Js.Json.t
val read_fragments_rule11_condition_member4 : Js.Json.t -> fragments_rule11_condition_member4
val encode_fragments_rule11_condition_member4 : fragments_rule11_condition_member4 -> Js.Json.t
val read_literal_event : Js.Json.t -> literal_event
val encode_literal_event : literal_event -> Js.Json.t
val read_fragments_rule11_condition_member5 : Js.Json.t -> fragments_rule11_condition_member5
val encode_fragments_rule11_condition_member5 : fragments_rule11_condition_member5 -> Js.Json.t
val read_literal_round : Js.Json.t -> literal_round
val encode_literal_round : literal_round -> Js.Json.t

val read_save2_checkpoint_state_tiles_item_fog_member1_condition_member6 :
  Js.Json.t -> save2_checkpoint_state_tiles_item_fog_member1_condition_member6

val encode_save2_checkpoint_state_tiles_item_fog_member1_condition_member6 :
  save2_checkpoint_state_tiles_item_fog_member1_condition_member6 -> Js.Json.t

val read_fragments_rule11_condition_member9_kind :
  Js.Json.t -> fragments_rule11_condition_member9_kind

val encode_fragments_rule11_condition_member9_kind :
  fragments_rule11_condition_member9_kind -> Js.Json.t

val read_literal_all : Js.Json.t -> literal_all
val encode_literal_all : literal_all -> Js.Json.t

val read_save2_checkpoint_state_tiles_item_fog_member1_condition_member6_conditions :
  Js.Json.t -> save2_checkpoint_state_tiles_item_fog_member1_condition_member6_conditions

val encode_save2_checkpoint_state_tiles_item_fog_member1_condition_member6_conditions :
  save2_checkpoint_state_tiles_item_fog_member1_condition_member6_conditions -> Js.Json.t

val read_fragments_rule11_condition_member6 : Js.Json.t -> fragments_rule11_condition_member6
val encode_fragments_rule11_condition_member6 : fragments_rule11_condition_member6 -> Js.Json.t
val read_literal_proximity : Js.Json.t -> literal_proximity
val encode_literal_proximity : literal_proximity -> Js.Json.t
val read_fragments_rule11_condition_member7 : Js.Json.t -> fragments_rule11_condition_member7
val encode_fragments_rule11_condition_member7 : fragments_rule11_condition_member7 -> Js.Json.t
val read_literal_has_units : Js.Json.t -> literal_has_units
val encode_literal_has_units : literal_has_units -> Js.Json.t
val read_fragments_rule11_condition_member8 : Js.Json.t -> fragments_rule11_condition_member8
val encode_fragments_rule11_condition_member8 : fragments_rule11_condition_member8 -> Js.Json.t
val read_literal_no_units : Js.Json.t -> literal_no_units
val encode_literal_no_units : literal_no_units -> Js.Json.t

val read_save2_checkpoint_state_tiles_item_light :
  Js.Json.t -> save2_checkpoint_state_tiles_item_light

val encode_save2_checkpoint_state_tiles_item_light :
  save2_checkpoint_state_tiles_item_light -> Js.Json.t

val read_save2_checkpoint_state_tiles_item_light_member1 :
  Js.Json.t -> save2_checkpoint_state_tiles_item_light_member1

val encode_save2_checkpoint_state_tiles_item_light_member1 :
  save2_checkpoint_state_tiles_item_light_member1 -> Js.Json.t

val read_fragments_light23_color : Js.Json.t -> fragments_light23_color
val encode_fragments_light23_color : fragments_light23_color -> Js.Json.t
val read_fragments_light23_color_member1 : Js.Json.t -> fragments_light23_color_member1
val encode_fragments_light23_color_member1 : fragments_light23_color_member1 -> Js.Json.t
val read_fragments_tiles11_item_wall_corners : Js.Json.t -> fragments_tiles11_item_wall_corners
val encode_fragments_tiles11_item_wall_corners : fragments_tiles11_item_wall_corners -> Js.Json.t

val read_fragments_tiles11_item_wall_corners_member1 :
  Js.Json.t -> fragments_tiles11_item_wall_corners_member1

val encode_fragments_tiles11_item_wall_corners_member1 :
  fragments_tiles11_item_wall_corners_member1 -> Js.Json.t

val read_save2_checkpoint_state_campaign : Js.Json.t -> save2_checkpoint_state_campaign
val encode_save2_checkpoint_state_campaign : save2_checkpoint_state_campaign -> Js.Json.t

val read_save2_checkpoint_state_campaign_member1 :
  Js.Json.t -> save2_checkpoint_state_campaign_member1

val encode_save2_checkpoint_state_campaign_member1 :
  save2_checkpoint_state_campaign_member1 -> Js.Json.t

val read_fragments_native_state38_campaign_member1_vigil :
  Js.Json.t -> fragments_native_state38_campaign_member1_vigil

val encode_fragments_native_state38_campaign_member1_vigil :
  fragments_native_state38_campaign_member1_vigil -> Js.Json.t

val read_fragments_native_state38_campaign_member1_vigil_member1 :
  Js.Json.t -> fragments_native_state38_campaign_member1_vigil_member1

val encode_fragments_native_state38_campaign_member1_vigil_member1 :
  fragments_native_state38_campaign_member1_vigil_member1 -> Js.Json.t

val read_fragments_native_state38_campaign_member1_refuge :
  Js.Json.t -> fragments_native_state38_campaign_member1_refuge

val encode_fragments_native_state38_campaign_member1_refuge :
  fragments_native_state38_campaign_member1_refuge -> Js.Json.t

val read_fragments_native_state38_campaign_member1_refuge_member1 :
  Js.Json.t -> fragments_native_state38_campaign_member1_refuge_member1

val encode_fragments_native_state38_campaign_member1_refuge_member1 :
  fragments_native_state38_campaign_member1_refuge_member1 -> Js.Json.t

val read_fragments_native_state38_campaign_member1_holds :
  Js.Json.t -> fragments_native_state38_campaign_member1_holds

val encode_fragments_native_state38_campaign_member1_holds :
  fragments_native_state38_campaign_member1_holds -> Js.Json.t

val read_fragments_native_state38_campaign_member1_holds_member1 :
  Js.Json.t -> fragments_native_state38_campaign_member1_holds_member1

val encode_fragments_native_state38_campaign_member1_holds_member1 :
  fragments_native_state38_campaign_member1_holds_member1 -> Js.Json.t

val read_fragments_native_state38_campaign_member1_dialogue :
  Js.Json.t -> fragments_native_state38_campaign_member1_dialogue

val encode_fragments_native_state38_campaign_member1_dialogue :
  fragments_native_state38_campaign_member1_dialogue -> Js.Json.t

val read_fragments_native_state38_campaign_member1_dialogue_item :
  Js.Json.t -> fragments_native_state38_campaign_member1_dialogue_item

val encode_fragments_native_state38_campaign_member1_dialogue_item :
  fragments_native_state38_campaign_member1_dialogue_item -> Js.Json.t

val read_save2_checkpoint_state_campaign_member1_veterans :
  Js.Json.t -> save2_checkpoint_state_campaign_member1_veterans

val encode_save2_checkpoint_state_campaign_member1_veterans :
  save2_checkpoint_state_campaign_member1_veterans -> Js.Json.t

val read_save2_checkpoint_state_campaign_member1_veterans_member1 :
  Js.Json.t -> save2_checkpoint_state_campaign_member1_veterans_member1

val encode_save2_checkpoint_state_campaign_member1_veterans_member1 :
  save2_checkpoint_state_campaign_member1_veterans_member1 -> Js.Json.t

val read_save2_checkpoint_state_campaign_member1_veterans_member1_item :
  Js.Json.t -> save2_checkpoint_state_campaign_member1_veterans_member1_item

val encode_save2_checkpoint_state_campaign_member1_veterans_member1_item :
  save2_checkpoint_state_campaign_member1_veterans_member1_item -> Js.Json.t

val read_save2_checkpoint_state_campaign_member1_evacuated :
  Js.Json.t -> save2_checkpoint_state_campaign_member1_evacuated

val encode_save2_checkpoint_state_campaign_member1_evacuated :
  save2_checkpoint_state_campaign_member1_evacuated -> Js.Json.t

val read_save2_checkpoint_state_campaign_member1_evacuated_member1 :
  Js.Json.t -> save2_checkpoint_state_campaign_member1_evacuated_member1

val encode_save2_checkpoint_state_campaign_member1_evacuated_member1 :
  save2_checkpoint_state_campaign_member1_evacuated_member1 -> Js.Json.t

val read_save2_checkpoint_state_campaign_member1_evacuated_member1_item :
  Js.Json.t -> save2_checkpoint_state_campaign_member1_evacuated_member1_item

val encode_save2_checkpoint_state_campaign_member1_evacuated_member1_item :
  save2_checkpoint_state_campaign_member1_evacuated_member1_item -> Js.Json.t

val read_javascript1_state_gold : Js.Json.t -> javascript1_state_gold
val encode_javascript1_state_gold : javascript1_state_gold -> Js.Json.t
val read_save2_checkpoint_state_factions : Js.Json.t -> save2_checkpoint_state_factions
val encode_save2_checkpoint_state_factions : save2_checkpoint_state_factions -> Js.Json.t

val read_save2_checkpoint_state_factions_member1 :
  Js.Json.t -> save2_checkpoint_state_factions_member1

val encode_save2_checkpoint_state_factions_member1 :
  save2_checkpoint_state_factions_member1 -> Js.Json.t

val read_save2_checkpoint_state_factions_member1_item :
  Js.Json.t -> save2_checkpoint_state_factions_member1_item

val encode_save2_checkpoint_state_factions_member1_item :
  save2_checkpoint_state_factions_member1_item -> Js.Json.t

val read_fragments_factions7_item_controller : Js.Json.t -> fragments_factions7_item_controller
val encode_fragments_factions7_item_controller : fragments_factions7_item_controller -> Js.Json.t

val read_fragments_factions7_item_controller_member0 :
  Js.Json.t -> fragments_factions7_item_controller_member0

val encode_fragments_factions7_item_controller_member0 :
  fragments_factions7_item_controller_member0 -> Js.Json.t

val read_literal_human : Js.Json.t -> literal_human
val encode_literal_human : literal_human -> Js.Json.t

val read_fragments_factions7_item_controller_member1 :
  Js.Json.t -> fragments_factions7_item_controller_member1

val encode_fragments_factions7_item_controller_member1 :
  fragments_factions7_item_controller_member1 -> Js.Json.t

val read_literal_passive : Js.Json.t -> literal_passive
val encode_literal_passive : literal_passive -> Js.Json.t

val read_fragments_factions7_item_controller_member2 :
  Js.Json.t -> fragments_factions7_item_controller_member2

val encode_fragments_factions7_item_controller_member2 :
  fragments_factions7_item_controller_member2 -> Js.Json.t

val read_literal_ai : Js.Json.t -> literal_ai
val encode_literal_ai : literal_ai -> Js.Json.t

val read_fragments_factions7_item_controller_member2_policy :
  Js.Json.t -> fragments_factions7_item_controller_member2_policy

val encode_fragments_factions7_item_controller_member2_policy :
  fragments_factions7_item_controller_member2_policy -> Js.Json.t

val read_literal_classic : Js.Json.t -> literal_classic
val encode_literal_classic : literal_classic -> Js.Json.t
val read_literal_rush : Js.Json.t -> literal_rush
val encode_literal_rush : literal_rush -> Js.Json.t
val read_literal_balanced : Js.Json.t -> literal_balanced
val encode_literal_balanced : literal_balanced -> Js.Json.t
val read_literal_rotated : Js.Json.t -> literal_rotated
val encode_literal_rotated : literal_rotated -> Js.Json.t
val read_literal_situational : Js.Json.t -> literal_situational
val encode_literal_situational : literal_situational -> Js.Json.t
val read_literal_predictor : Js.Json.t -> literal_predictor
val encode_literal_predictor : literal_predictor -> Js.Json.t
val read_fragments_factions7_item_recruits : Js.Json.t -> fragments_factions7_item_recruits
val encode_fragments_factions7_item_recruits : fragments_factions7_item_recruits -> Js.Json.t

val read_fragments_factions7_item_recruits_member1 :
  Js.Json.t -> fragments_factions7_item_recruits_member1

val encode_fragments_factions7_item_recruits_member1 :
  fragments_factions7_item_recruits_member1 -> Js.Json.t

val read_fragments_factions7_item_recruits_member1_item :
  Js.Json.t -> fragments_factions7_item_recruits_member1_item

val encode_fragments_factions7_item_recruits_member1_item :
  fragments_factions7_item_recruits_member1_item -> Js.Json.t

val read_fragments_factions7_item_relations : Js.Json.t -> fragments_factions7_item_relations
val encode_fragments_factions7_item_relations : fragments_factions7_item_relations -> Js.Json.t

val read_fragments_factions7_item_relations_member1 :
  Js.Json.t -> fragments_factions7_item_relations_member1

val encode_fragments_factions7_item_relations_member1 :
  fragments_factions7_item_relations_member1 -> Js.Json.t

val read_fragments_factions7_item_relations_member1_value :
  Js.Json.t -> fragments_factions7_item_relations_member1_value

val encode_fragments_factions7_item_relations_member1_value :
  fragments_factions7_item_relations_member1_value -> Js.Json.t

val read_literal_allied : Js.Json.t -> literal_allied
val encode_literal_allied : literal_allied -> Js.Json.t
val read_literal_hostile : Js.Json.t -> literal_hostile
val encode_literal_hostile : literal_hostile -> Js.Json.t

val read_save2_checkpoint_state_factions_member1_item_transfer :
  Js.Json.t -> save2_checkpoint_state_factions_member1_item_transfer

val encode_save2_checkpoint_state_factions_member1_item_transfer :
  save2_checkpoint_state_factions_member1_item_transfer -> Js.Json.t

val read_save2_checkpoint_state_factions_member1_item_transfer_member1 :
  Js.Json.t -> save2_checkpoint_state_factions_member1_item_transfer_member1

val encode_save2_checkpoint_state_factions_member1_item_transfer_member1 :
  save2_checkpoint_state_factions_member1_item_transfer_member1 -> Js.Json.t

val read_save2_checkpoint_state_ground_items : Js.Json.t -> save2_checkpoint_state_ground_items
val encode_save2_checkpoint_state_ground_items : save2_checkpoint_state_ground_items -> Js.Json.t
val read_fragments_native_state38_ground_items : Js.Json.t -> fragments_native_state38_ground_items

val encode_fragments_native_state38_ground_items :
  fragments_native_state38_ground_items -> Js.Json.t

val read_fragments_native_state38_ground_items_item :
  Js.Json.t -> fragments_native_state38_ground_items_item

val encode_fragments_native_state38_ground_items_item :
  fragments_native_state38_ground_items_item -> Js.Json.t

val read_json_unknown : Js.Json.t -> json_unknown
val encode_json_unknown : json_unknown -> Js.Json.t
val read_save2_setup : Js.Json.t -> save2_setup
val encode_save2_setup : save2_setup -> Js.Json.t
val read_save2_setup_member1 : Js.Json.t -> save2_setup_member1
val encode_save2_setup_member1 : save2_setup_member1 -> Js.Json.t
val read_save2_setup_member1_enemy : Js.Json.t -> save2_setup_member1_enemy
val encode_save2_setup_member1_enemy : save2_setup_member1_enemy -> Js.Json.t
val read_save35_setup_enemy_policy : Js.Json.t -> save35_setup_enemy_policy
val encode_save35_setup_enemy_policy : save35_setup_enemy_policy -> Js.Json.t
val read_save35_setup_enemy_blocks : Js.Json.t -> save35_setup_enemy_blocks
val encode_save35_setup_enemy_blocks : save35_setup_enemy_blocks -> Js.Json.t
val read_save35_setup_enemy_blocks_member1 : Js.Json.t -> save35_setup_enemy_blocks_member1
val encode_save35_setup_enemy_blocks_member1 : save35_setup_enemy_blocks_member1 -> Js.Json.t

val read_save35_setup_enemy_blocks_member1_recruiter :
  Js.Json.t -> save35_setup_enemy_blocks_member1_recruiter

val encode_save35_setup_enemy_blocks_member1_recruiter :
  save35_setup_enemy_blocks_member1_recruiter -> Js.Json.t

val read_literal_rotation : Js.Json.t -> literal_rotation
val encode_literal_rotation : literal_rotation -> Js.Json.t
val read_literal_frontline : Js.Json.t -> literal_frontline
val encode_literal_frontline : literal_frontline -> Js.Json.t
val read_literal_aggressive_predictor : Js.Json.t -> literal_aggressive_predictor
val encode_literal_aggressive_predictor : literal_aggressive_predictor -> Js.Json.t

val read_save35_setup_enemy_blocks_member1_campaign :
  Js.Json.t -> save35_setup_enemy_blocks_member1_campaign

val encode_save35_setup_enemy_blocks_member1_campaign :
  save35_setup_enemy_blocks_member1_campaign -> Js.Json.t

val read_literal_arena : Js.Json.t -> literal_arena
val encode_literal_arena : literal_arena -> Js.Json.t
val read_literal_mission : Js.Json.t -> literal_mission
val encode_literal_mission : literal_mission -> Js.Json.t

val read_save35_setup_enemy_blocks_member1_planner :
  Js.Json.t -> save35_setup_enemy_blocks_member1_planner

val encode_save35_setup_enemy_blocks_member1_planner :
  save35_setup_enemy_blocks_member1_planner -> Js.Json.t

val read_literal_full : Js.Json.t -> literal_full
val encode_literal_full : literal_full -> Js.Json.t
val read_literal_queued : Js.Json.t -> literal_queued
val encode_literal_queued : literal_queued -> Js.Json.t
val read_save2_setup_member1_enemy_diplomacy : Js.Json.t -> save2_setup_member1_enemy_diplomacy
val encode_save2_setup_member1_enemy_diplomacy : save2_setup_member1_enemy_diplomacy -> Js.Json.t
val read_literal_diplomatic : Js.Json.t -> literal_diplomatic
val encode_literal_diplomatic : literal_diplomatic -> Js.Json.t
val read_literal_unrestricted : Js.Json.t -> literal_unrestricted
val encode_literal_unrestricted : literal_unrestricted -> Js.Json.t
val read_literal_pragmatic : Js.Json.t -> literal_pragmatic
val encode_literal_pragmatic : literal_pragmatic -> Js.Json.t
val read_literal_restrained : Js.Json.t -> literal_restrained
val encode_literal_restrained : literal_restrained -> Js.Json.t
val read_save2_progress : Js.Json.t -> save2_progress
val encode_save2_progress : save2_progress -> Js.Json.t
val read_save2_progress_member1 : Js.Json.t -> save2_progress_member1
val encode_save2_progress_member1 : save2_progress_member1 -> Js.Json.t
val read_history1_entries_item : Js.Json.t -> history1_entries_item
val encode_history1_entries_item : history1_entries_item -> Js.Json.t
val read_history1_entries_item_kind : Js.Json.t -> history1_entries_item_kind
val encode_history1_entries_item_kind : history1_entries_item_kind -> Js.Json.t
val read_literal_turn : Js.Json.t -> literal_turn
val encode_literal_turn : literal_turn -> Js.Json.t
val read_literal_recovery : Js.Json.t -> literal_recovery
val encode_literal_recovery : literal_recovery -> Js.Json.t
val read_literal_resume : Js.Json.t -> literal_resume
val encode_literal_resume : literal_resume -> Js.Json.t
val read_save3 : Js.Json.t -> save3
val encode_save3 : save3 -> Js.Json.t
val read_literal_3 : Js.Json.t -> literal_3
val encode_literal_3 : literal_3 -> Js.Json.t
val read_save3_checkpoint : Js.Json.t -> save3_checkpoint
val encode_save3_checkpoint : save3_checkpoint -> Js.Json.t
val read_save3_checkpoint_state : Js.Json.t -> save3_checkpoint_state
val encode_save3_checkpoint_state : save3_checkpoint_state -> Js.Json.t
val read_fragments_tiles11 : Js.Json.t -> fragments_tiles11
val encode_fragments_tiles11 : fragments_tiles11 -> Js.Json.t
val read_fragments_tiles11_item : Js.Json.t -> fragments_tiles11_item
val encode_fragments_tiles11_item : fragments_tiles11_item -> Js.Json.t
val read_fragments_light23_when_ : Js.Json.t -> fragments_light23_when_
val encode_fragments_light23_when_ : fragments_light23_when_ -> Js.Json.t
val read_fragments_rule11 : Js.Json.t -> fragments_rule11
val encode_fragments_rule11 : fragments_rule11 -> Js.Json.t
val read_fragments_rule11_condition : Js.Json.t -> fragments_rule11_condition
val encode_fragments_rule11_condition : fragments_rule11_condition -> Js.Json.t
val read_fragments_rule11_condition_member9 : Js.Json.t -> fragments_rule11_condition_member9
val encode_fragments_rule11_condition_member9 : fragments_rule11_condition_member9 -> Js.Json.t

val read_fragments_rule11_condition_member9_conditions :
  Js.Json.t -> fragments_rule11_condition_member9_conditions

val encode_fragments_rule11_condition_member9_conditions :
  fragments_rule11_condition_member9_conditions -> Js.Json.t

val read_fragments_tiles11_item_light : Js.Json.t -> fragments_tiles11_item_light
val encode_fragments_tiles11_item_light : fragments_tiles11_item_light -> Js.Json.t
val read_fragments_light23 : Js.Json.t -> fragments_light23
val encode_fragments_light23 : fragments_light23 -> Js.Json.t
val read_save3_checkpoint_state_factions : Js.Json.t -> save3_checkpoint_state_factions
val encode_save3_checkpoint_state_factions : save3_checkpoint_state_factions -> Js.Json.t
val read_fragments_factions7 : Js.Json.t -> fragments_factions7
val encode_fragments_factions7 : fragments_factions7 -> Js.Json.t
val read_fragments_factions7_item : Js.Json.t -> fragments_factions7_item
val encode_fragments_factions7_item : fragments_factions7_item -> Js.Json.t
val read_fragments_factions7_item_transfer : Js.Json.t -> fragments_factions7_item_transfer
val encode_fragments_factions7_item_transfer : fragments_factions7_item_transfer -> Js.Json.t

val read_fragments_factions7_item_transfer_member1 :
  Js.Json.t -> fragments_factions7_item_transfer_member1

val encode_fragments_factions7_item_transfer_member1 :
  fragments_factions7_item_transfer_member1 -> Js.Json.t

val read_save4 : Js.Json.t -> save4
val encode_save4 : save4 -> Js.Json.t
val read_literal_4 : Js.Json.t -> literal_4
val encode_literal_4 : literal_4 -> Js.Json.t
val read_save5 : Js.Json.t -> save5
val encode_save5 : save5 -> Js.Json.t
val read_literal_5 : Js.Json.t -> literal_5
val encode_literal_5 : literal_5 -> Js.Json.t
val read_save6 : Js.Json.t -> save6
val encode_save6 : save6 -> Js.Json.t
val read_literal_6 : Js.Json.t -> literal_6
val encode_literal_6 : literal_6 -> Js.Json.t
val read_save7 : Js.Json.t -> save7
val encode_save7 : save7 -> Js.Json.t
val read_literal_7 : Js.Json.t -> literal_7
val encode_literal_7 : literal_7 -> Js.Json.t
val read_save7_checkpoint : Js.Json.t -> save7_checkpoint
val encode_save7_checkpoint : save7_checkpoint -> Js.Json.t
val read_save7_checkpoint_state : Js.Json.t -> save7_checkpoint_state
val encode_save7_checkpoint_state : save7_checkpoint_state -> Js.Json.t
val read_save8 : Js.Json.t -> save8
val encode_save8 : save8 -> Js.Json.t
val read_literal_8 : Js.Json.t -> literal_8
val encode_literal_8 : literal_8 -> Js.Json.t
val read_save8_checkpoint : Js.Json.t -> save8_checkpoint
val encode_save8_checkpoint : save8_checkpoint -> Js.Json.t
val read_save8_checkpoint_state : Js.Json.t -> save8_checkpoint_state
val encode_save8_checkpoint_state : save8_checkpoint_state -> Js.Json.t
val read_save9 : Js.Json.t -> save9
val encode_save9 : save9 -> Js.Json.t
val read_literal_9 : Js.Json.t -> literal_9
val encode_literal_9 : literal_9 -> Js.Json.t
val read_save10 : Js.Json.t -> save10
val encode_save10 : save10 -> Js.Json.t
val read_literal_10 : Js.Json.t -> literal_10
val encode_literal_10 : literal_10 -> Js.Json.t
val read_save11 : Js.Json.t -> save11
val encode_save11 : save11 -> Js.Json.t
val read_literal_11 : Js.Json.t -> literal_11
val encode_literal_11 : literal_11 -> Js.Json.t
val read_save12 : Js.Json.t -> save12
val encode_save12 : save12 -> Js.Json.t
val read_literal_12 : Js.Json.t -> literal_12
val encode_literal_12 : literal_12 -> Js.Json.t
val read_save13 : Js.Json.t -> save13
val encode_save13 : save13 -> Js.Json.t
val read_literal_13 : Js.Json.t -> literal_13
val encode_literal_13 : literal_13 -> Js.Json.t
val read_save14 : Js.Json.t -> save14
val encode_save14 : save14 -> Js.Json.t
val read_literal_14 : Js.Json.t -> literal_14
val encode_literal_14 : literal_14 -> Js.Json.t
val read_save15 : Js.Json.t -> save15
val encode_save15 : save15 -> Js.Json.t
val read_literal_15 : Js.Json.t -> literal_15
val encode_literal_15 : literal_15 -> Js.Json.t
val read_save16 : Js.Json.t -> save16
val encode_save16 : save16 -> Js.Json.t
val read_literal_16 : Js.Json.t -> literal_16
val encode_literal_16 : literal_16 -> Js.Json.t
val read_save17 : Js.Json.t -> save17
val encode_save17 : save17 -> Js.Json.t
val read_literal_17 : Js.Json.t -> literal_17
val encode_literal_17 : literal_17 -> Js.Json.t
val read_save17_checkpoint : Js.Json.t -> save17_checkpoint
val encode_save17_checkpoint : save17_checkpoint -> Js.Json.t
val read_save17_checkpoint_state : Js.Json.t -> save17_checkpoint_state
val encode_save17_checkpoint_state : save17_checkpoint_state -> Js.Json.t
val read_save17_checkpoint_state_tiles : Js.Json.t -> save17_checkpoint_state_tiles
val encode_save17_checkpoint_state_tiles : save17_checkpoint_state_tiles -> Js.Json.t
val read_save17_checkpoint_state_tiles_item : Js.Json.t -> save17_checkpoint_state_tiles_item
val encode_save17_checkpoint_state_tiles_item : save17_checkpoint_state_tiles_item -> Js.Json.t
val read_save18 : Js.Json.t -> save18
val encode_save18 : save18 -> Js.Json.t
val read_literal_18 : Js.Json.t -> literal_18
val encode_literal_18 : literal_18 -> Js.Json.t
val read_save18_checkpoint : Js.Json.t -> save18_checkpoint
val encode_save18_checkpoint : save18_checkpoint -> Js.Json.t
val read_save18_checkpoint_state : Js.Json.t -> save18_checkpoint_state
val encode_save18_checkpoint_state : save18_checkpoint_state -> Js.Json.t
val read_save18_checkpoint_state_tiles : Js.Json.t -> save18_checkpoint_state_tiles
val encode_save18_checkpoint_state_tiles : save18_checkpoint_state_tiles -> Js.Json.t
val read_save18_checkpoint_state_tiles_item : Js.Json.t -> save18_checkpoint_state_tiles_item
val encode_save18_checkpoint_state_tiles_item : save18_checkpoint_state_tiles_item -> Js.Json.t
val read_save19 : Js.Json.t -> save19
val encode_save19 : save19 -> Js.Json.t
val read_literal_19 : Js.Json.t -> literal_19
val encode_literal_19 : literal_19 -> Js.Json.t
val read_save20 : Js.Json.t -> save20
val encode_save20 : save20 -> Js.Json.t
val read_literal_20 : Js.Json.t -> literal_20
val encode_literal_20 : literal_20 -> Js.Json.t
val read_save21 : Js.Json.t -> save21
val encode_save21 : save21 -> Js.Json.t
val read_literal_21 : Js.Json.t -> literal_21
val encode_literal_21 : literal_21 -> Js.Json.t
val read_save21_checkpoint : Js.Json.t -> save21_checkpoint
val encode_save21_checkpoint : save21_checkpoint -> Js.Json.t
val read_save21_checkpoint_state : Js.Json.t -> save21_checkpoint_state
val encode_save21_checkpoint_state : save21_checkpoint_state -> Js.Json.t
val read_fragments_native_state38_gold : Js.Json.t -> fragments_native_state38_gold
val encode_fragments_native_state38_gold : fragments_native_state38_gold -> Js.Json.t
val read_save22 : Js.Json.t -> save22
val encode_save22 : save22 -> Js.Json.t
val read_literal_22 : Js.Json.t -> literal_22
val encode_literal_22 : literal_22 -> Js.Json.t
val read_save23 : Js.Json.t -> save23
val encode_save23 : save23 -> Js.Json.t
val read_literal_23 : Js.Json.t -> literal_23
val encode_literal_23 : literal_23 -> Js.Json.t
val read_save24 : Js.Json.t -> save24
val encode_save24 : save24 -> Js.Json.t
val read_literal_24 : Js.Json.t -> literal_24
val encode_literal_24 : literal_24 -> Js.Json.t
val read_save25 : Js.Json.t -> save25
val encode_save25 : save25 -> Js.Json.t
val read_literal_25 : Js.Json.t -> literal_25
val encode_literal_25 : literal_25 -> Js.Json.t
val read_save26 : Js.Json.t -> save26
val encode_save26 : save26 -> Js.Json.t
val read_literal_26 : Js.Json.t -> literal_26
val encode_literal_26 : literal_26 -> Js.Json.t
val read_save26_checkpoint : Js.Json.t -> save26_checkpoint
val encode_save26_checkpoint : save26_checkpoint -> Js.Json.t
val read_save26_checkpoint_state : Js.Json.t -> save26_checkpoint_state
val encode_save26_checkpoint_state : save26_checkpoint_state -> Js.Json.t
val read_save26_checkpoint_state_units : Js.Json.t -> save26_checkpoint_state_units
val encode_save26_checkpoint_state_units : save26_checkpoint_state_units -> Js.Json.t
val read_save26_checkpoint_state_units_item : Js.Json.t -> save26_checkpoint_state_units_item
val encode_save26_checkpoint_state_units_item : save26_checkpoint_state_units_item -> Js.Json.t
val read_save26_checkpoint_state_tiles : Js.Json.t -> save26_checkpoint_state_tiles
val encode_save26_checkpoint_state_tiles : save26_checkpoint_state_tiles -> Js.Json.t
val read_save26_checkpoint_state_tiles_item : Js.Json.t -> save26_checkpoint_state_tiles_item
val encode_save26_checkpoint_state_tiles_item : save26_checkpoint_state_tiles_item -> Js.Json.t
val read_fragments_light28_member1_when_ : Js.Json.t -> fragments_light28_member1_when_
val encode_fragments_light28_member1_when_ : fragments_light28_member1_when_ -> Js.Json.t

val read_fragments_light28_member1_when__member1 :
  Js.Json.t -> fragments_light28_member1_when__member1

val encode_fragments_light28_member1_when__member1 :
  fragments_light28_member1_when__member1 -> Js.Json.t

val read_fragments_light28_member1_when__member1_condition :
  Js.Json.t -> fragments_light28_member1_when__member1_condition

val encode_fragments_light28_member1_when__member1_condition :
  fragments_light28_member1_when__member1_condition -> Js.Json.t

val read_fragments_light28_member1_when__member1_condition_member1 :
  Js.Json.t -> fragments_light28_member1_when__member1_condition_member1

val encode_fragments_light28_member1_when__member1_condition_member1 :
  fragments_light28_member1_when__member1_condition_member1 -> Js.Json.t

val read_fragments_light28_member1_when__member1_condition_member3 :
  Js.Json.t -> fragments_light28_member1_when__member1_condition_member3

val encode_fragments_light28_member1_when__member1_condition_member3 :
  fragments_light28_member1_when__member1_condition_member3 -> Js.Json.t

val read_fragments_light28_member1_when__member1_condition_member4 :
  Js.Json.t -> fragments_light28_member1_when__member1_condition_member4

val encode_fragments_light28_member1_when__member1_condition_member4 :
  fragments_light28_member1_when__member1_condition_member4 -> Js.Json.t

val read_fragments_light28_member1_when__member1_condition_member9 :
  Js.Json.t -> fragments_light28_member1_when__member1_condition_member9

val encode_fragments_light28_member1_when__member1_condition_member9 :
  fragments_light28_member1_when__member1_condition_member9 -> Js.Json.t

val read_fragments_light28_member1_when__member1_condition_member9_conditions :
  Js.Json.t -> fragments_light28_member1_when__member1_condition_member9_conditions

val encode_fragments_light28_member1_when__member1_condition_member9_conditions :
  fragments_light28_member1_when__member1_condition_member9_conditions -> Js.Json.t

val read_save26_checkpoint_state_tiles_item_light :
  Js.Json.t -> save26_checkpoint_state_tiles_item_light

val encode_save26_checkpoint_state_tiles_item_light :
  save26_checkpoint_state_tiles_item_light -> Js.Json.t

val read_fragments_light28_member1 : Js.Json.t -> fragments_light28_member1
val encode_fragments_light28_member1 : fragments_light28_member1 -> Js.Json.t
val read_save26_checkpoint_state_campaign : Js.Json.t -> save26_checkpoint_state_campaign
val encode_save26_checkpoint_state_campaign : save26_checkpoint_state_campaign -> Js.Json.t

val read_save26_checkpoint_state_campaign_member1 :
  Js.Json.t -> save26_checkpoint_state_campaign_member1

val encode_save26_checkpoint_state_campaign_member1 :
  save26_checkpoint_state_campaign_member1 -> Js.Json.t

val read_save26_checkpoint_state_campaign_member1_veterans :
  Js.Json.t -> save26_checkpoint_state_campaign_member1_veterans

val encode_save26_checkpoint_state_campaign_member1_veterans :
  save26_checkpoint_state_campaign_member1_veterans -> Js.Json.t

val read_save26_checkpoint_state_campaign_member1_veterans_member1 :
  Js.Json.t -> save26_checkpoint_state_campaign_member1_veterans_member1

val encode_save26_checkpoint_state_campaign_member1_veterans_member1 :
  save26_checkpoint_state_campaign_member1_veterans_member1 -> Js.Json.t

val read_save26_checkpoint_state_campaign_member1_veterans_member1_item :
  Js.Json.t -> save26_checkpoint_state_campaign_member1_veterans_member1_item

val encode_save26_checkpoint_state_campaign_member1_veterans_member1_item :
  save26_checkpoint_state_campaign_member1_veterans_member1_item -> Js.Json.t

val read_save26_checkpoint_state_campaign_member1_evacuated :
  Js.Json.t -> save26_checkpoint_state_campaign_member1_evacuated

val encode_save26_checkpoint_state_campaign_member1_evacuated :
  save26_checkpoint_state_campaign_member1_evacuated -> Js.Json.t

val read_save26_checkpoint_state_campaign_member1_evacuated_member1 :
  Js.Json.t -> save26_checkpoint_state_campaign_member1_evacuated_member1

val encode_save26_checkpoint_state_campaign_member1_evacuated_member1 :
  save26_checkpoint_state_campaign_member1_evacuated_member1 -> Js.Json.t

val read_save26_checkpoint_state_campaign_member1_evacuated_member1_item :
  Js.Json.t -> save26_checkpoint_state_campaign_member1_evacuated_member1_item

val encode_save26_checkpoint_state_campaign_member1_evacuated_member1_item :
  save26_checkpoint_state_campaign_member1_evacuated_member1_item -> Js.Json.t

val read_save26_checkpoint_state_factions : Js.Json.t -> save26_checkpoint_state_factions
val encode_save26_checkpoint_state_factions : save26_checkpoint_state_factions -> Js.Json.t
val read_save26_checkpoint_state_factions_item : Js.Json.t -> save26_checkpoint_state_factions_item

val encode_save26_checkpoint_state_factions_item :
  save26_checkpoint_state_factions_item -> Js.Json.t

val read_save26_checkpoint_state_factions_item_transfer :
  Js.Json.t -> save26_checkpoint_state_factions_item_transfer

val encode_save26_checkpoint_state_factions_item_transfer :
  save26_checkpoint_state_factions_item_transfer -> Js.Json.t

val read_save26_checkpoint_state_factions_item_transfer_member1 :
  Js.Json.t -> save26_checkpoint_state_factions_item_transfer_member1

val encode_save26_checkpoint_state_factions_item_transfer_member1 :
  save26_checkpoint_state_factions_item_transfer_member1 -> Js.Json.t

val read_save27 : Js.Json.t -> save27
val encode_save27 : save27 -> Js.Json.t
val read_literal_27 : Js.Json.t -> literal_27
val encode_literal_27 : literal_27 -> Js.Json.t
val read_save28 : Js.Json.t -> save28
val encode_save28 : save28 -> Js.Json.t
val read_literal_28 : Js.Json.t -> literal_28
val encode_literal_28 : literal_28 -> Js.Json.t
val read_save28_checkpoint : Js.Json.t -> save28_checkpoint
val encode_save28_checkpoint : save28_checkpoint -> Js.Json.t
val read_save28_checkpoint_state : Js.Json.t -> save28_checkpoint_state
val encode_save28_checkpoint_state : save28_checkpoint_state -> Js.Json.t
val read_save28_checkpoint_state_campaign : Js.Json.t -> save28_checkpoint_state_campaign
val encode_save28_checkpoint_state_campaign : save28_checkpoint_state_campaign -> Js.Json.t

val read_save28_checkpoint_state_campaign_member1 :
  Js.Json.t -> save28_checkpoint_state_campaign_member1

val encode_save28_checkpoint_state_campaign_member1 :
  save28_checkpoint_state_campaign_member1 -> Js.Json.t

val read_save28_checkpoint_state_campaign_member1_scripts :
  Js.Json.t -> save28_checkpoint_state_campaign_member1_scripts

val encode_save28_checkpoint_state_campaign_member1_scripts :
  save28_checkpoint_state_campaign_member1_scripts -> Js.Json.t

val read_save28_checkpoint_state_campaign_member1_scripts_member1 :
  Js.Json.t -> save28_checkpoint_state_campaign_member1_scripts_member1

val encode_save28_checkpoint_state_campaign_member1_scripts_member1 :
  save28_checkpoint_state_campaign_member1_scripts_member1 -> Js.Json.t

val read_fragments_native_state38_campaign_member1_scripts_member1_recruitment :
  Js.Json.t -> fragments_native_state38_campaign_member1_scripts_member1_recruitment

val encode_fragments_native_state38_campaign_member1_scripts_member1_recruitment :
  fragments_native_state38_campaign_member1_scripts_member1_recruitment -> Js.Json.t

val read_fragments_native_state38_campaign_member1_scripts_member1_holds :
  Js.Json.t -> fragments_native_state38_campaign_member1_scripts_member1_holds

val encode_fragments_native_state38_campaign_member1_scripts_member1_holds :
  fragments_native_state38_campaign_member1_scripts_member1_holds -> Js.Json.t

val read_fragments_native_state38_campaign_member1_scripts_member1_holds_item :
  Js.Json.t -> fragments_native_state38_campaign_member1_scripts_member1_holds_item

val encode_fragments_native_state38_campaign_member1_scripts_member1_holds_item :
  fragments_native_state38_campaign_member1_scripts_member1_holds_item -> Js.Json.t

val read_fragments_native_state38_campaign_member1_scripts_member1_pending :
  Js.Json.t -> fragments_native_state38_campaign_member1_scripts_member1_pending

val encode_fragments_native_state38_campaign_member1_scripts_member1_pending :
  fragments_native_state38_campaign_member1_scripts_member1_pending -> Js.Json.t

val read_fragments_native_state38_campaign_member1_scripts_member1_pending_item :
  Js.Json.t -> fragments_native_state38_campaign_member1_scripts_member1_pending_item

val encode_fragments_native_state38_campaign_member1_scripts_member1_pending_item :
  fragments_native_state38_campaign_member1_scripts_member1_pending_item -> Js.Json.t

val read_fragments_delta19_item_owner : Js.Json.t -> fragments_delta19_item_owner
val encode_fragments_delta19_item_owner : fragments_delta19_item_owner -> Js.Json.t

val read_fragments_native_state38_campaign_member1_scripts_member1_pending_item_units :
  Js.Json.t -> fragments_native_state38_campaign_member1_scripts_member1_pending_item_units

val encode_fragments_native_state38_campaign_member1_scripts_member1_pending_item_units :
  fragments_native_state38_campaign_member1_scripts_member1_pending_item_units -> Js.Json.t

val read_fragments_native_state38_campaign_member1_scripts_member1_pending_item_units_item :
  Js.Json.t -> fragments_native_state38_campaign_member1_scripts_member1_pending_item_units_item

val encode_fragments_native_state38_campaign_member1_scripts_member1_pending_item_units_item :
  fragments_native_state38_campaign_member1_scripts_member1_pending_item_units_item -> Js.Json.t

val read_fragments_native_state38_campaign_member1_scripts_member1_pending_item_units_item_role :
  Js.Json.t ->
  fragments_native_state38_campaign_member1_scripts_member1_pending_item_units_item_role

val encode_fragments_native_state38_campaign_member1_scripts_member1_pending_item_units_item_role :
  fragments_native_state38_campaign_member1_scripts_member1_pending_item_units_item_role ->
  Js.Json.t

val read_save28_checkpoint_state_campaign_member1_scripts_member1_marks :
  Js.Json.t -> save28_checkpoint_state_campaign_member1_scripts_member1_marks

val encode_save28_checkpoint_state_campaign_member1_scripts_member1_marks :
  save28_checkpoint_state_campaign_member1_scripts_member1_marks -> Js.Json.t

val read_save28_checkpoint_state_campaign_member1_scripts_member1_marks_item :
  Js.Json.t -> save28_checkpoint_state_campaign_member1_scripts_member1_marks_item

val encode_save28_checkpoint_state_campaign_member1_scripts_member1_marks_item :
  save28_checkpoint_state_campaign_member1_scripts_member1_marks_item -> Js.Json.t

val read_save29 : Js.Json.t -> save29
val encode_save29 : save29 -> Js.Json.t
val read_literal_29 : Js.Json.t -> literal_29
val encode_literal_29 : literal_29 -> Js.Json.t
val read_save29_checkpoint : Js.Json.t -> save29_checkpoint
val encode_save29_checkpoint : save29_checkpoint -> Js.Json.t
val read_save29_checkpoint_state : Js.Json.t -> save29_checkpoint_state
val encode_save29_checkpoint_state : save29_checkpoint_state -> Js.Json.t
val read_save29_checkpoint_state_campaign : Js.Json.t -> save29_checkpoint_state_campaign
val encode_save29_checkpoint_state_campaign : save29_checkpoint_state_campaign -> Js.Json.t

val read_save29_checkpoint_state_campaign_member1 :
  Js.Json.t -> save29_checkpoint_state_campaign_member1

val encode_save29_checkpoint_state_campaign_member1 :
  save29_checkpoint_state_campaign_member1 -> Js.Json.t

val read_fragments_native_state38_campaign_member1_flags :
  Js.Json.t -> fragments_native_state38_campaign_member1_flags

val encode_fragments_native_state38_campaign_member1_flags :
  fragments_native_state38_campaign_member1_flags -> Js.Json.t

val read_fragments_native_state38_campaign_member1_flags_value :
  Js.Json.t -> fragments_native_state38_campaign_member1_flags_value

val encode_fragments_native_state38_campaign_member1_flags_value :
  fragments_native_state38_campaign_member1_flags_value -> Js.Json.t

val read_fragments_native_state38_campaign_member1_flags_value_item :
  Js.Json.t -> fragments_native_state38_campaign_member1_flags_value_item

val encode_fragments_native_state38_campaign_member1_flags_value_item :
  fragments_native_state38_campaign_member1_flags_value_item -> Js.Json.t

val read_save30 : Js.Json.t -> save30
val encode_save30 : save30 -> Js.Json.t
val read_literal_30 : Js.Json.t -> literal_30
val encode_literal_30 : literal_30 -> Js.Json.t
val read_save30_setup : Js.Json.t -> save30_setup
val encode_save30_setup : save30_setup -> Js.Json.t
val read_save35_setup : Js.Json.t -> save35_setup
val encode_save35_setup : save35_setup -> Js.Json.t
val read_save35_setup_enemy : Js.Json.t -> save35_setup_enemy
val encode_save35_setup_enemy : save35_setup_enemy -> Js.Json.t
val read_save35_setup_enemy_diplomacy : Js.Json.t -> save35_setup_enemy_diplomacy
val encode_save35_setup_enemy_diplomacy : save35_setup_enemy_diplomacy -> Js.Json.t
val read_save31 : Js.Json.t -> save31
val encode_save31 : save31 -> Js.Json.t
val read_literal_31 : Js.Json.t -> literal_31
val encode_literal_31 : literal_31 -> Js.Json.t
val read_save32 : Js.Json.t -> save32
val encode_save32 : save32 -> Js.Json.t
val read_literal_32 : Js.Json.t -> literal_32
val encode_literal_32 : literal_32 -> Js.Json.t
val read_save33 : Js.Json.t -> save33
val encode_save33 : save33 -> Js.Json.t
val read_literal_33 : Js.Json.t -> literal_33
val encode_literal_33 : literal_33 -> Js.Json.t
val read_save34 : Js.Json.t -> save34
val encode_save34 : save34 -> Js.Json.t
val read_literal_34 : Js.Json.t -> literal_34
val encode_literal_34 : literal_34 -> Js.Json.t
val read_save34_checkpoint : Js.Json.t -> save34_checkpoint
val encode_save34_checkpoint : save34_checkpoint -> Js.Json.t
val read_save34_checkpoint_state : Js.Json.t -> save34_checkpoint_state
val encode_save34_checkpoint_state : save34_checkpoint_state -> Js.Json.t
val read_save34_checkpoint_state_campaign : Js.Json.t -> save34_checkpoint_state_campaign
val encode_save34_checkpoint_state_campaign : save34_checkpoint_state_campaign -> Js.Json.t

val read_save34_checkpoint_state_campaign_member1 :
  Js.Json.t -> save34_checkpoint_state_campaign_member1

val encode_save34_checkpoint_state_campaign_member1 :
  save34_checkpoint_state_campaign_member1 -> Js.Json.t

val read_save34_checkpoint_state_campaign_member1_scripts :
  Js.Json.t -> save34_checkpoint_state_campaign_member1_scripts

val encode_save34_checkpoint_state_campaign_member1_scripts :
  save34_checkpoint_state_campaign_member1_scripts -> Js.Json.t

val read_save34_checkpoint_state_campaign_member1_scripts_member1 :
  Js.Json.t -> save34_checkpoint_state_campaign_member1_scripts_member1

val encode_save34_checkpoint_state_campaign_member1_scripts_member1 :
  save34_checkpoint_state_campaign_member1_scripts_member1 -> Js.Json.t

val read_save34_undo : Js.Json.t -> save34_undo
val encode_save34_undo : save34_undo -> Js.Json.t
val read_save34_undo_member1 : Js.Json.t -> save34_undo_member1
val encode_save34_undo_member1 : save34_undo_member1 -> Js.Json.t
val read_save34_undo_member1_item : Js.Json.t -> save34_undo_member1_item
val encode_save34_undo_member1_item : save34_undo_member1_item -> Js.Json.t
val read_save34_undo_member1_item_changes : Js.Json.t -> save34_undo_member1_item_changes
val encode_save34_undo_member1_item_changes : save34_undo_member1_item_changes -> Js.Json.t
val read_save34_undo_member1_item_changes_item : Js.Json.t -> save34_undo_member1_item_changes_item

val encode_save34_undo_member1_item_changes_item :
  save34_undo_member1_item_changes_item -> Js.Json.t

val read_save34_undo_member1_item_changes_item_member0 :
  Js.Json.t -> save34_undo_member1_item_changes_item_member0

val encode_save34_undo_member1_item_changes_item_member0 :
  save34_undo_member1_item_changes_item_member0 -> Js.Json.t

val read_literal_set : Js.Json.t -> literal_set
val encode_literal_set : literal_set -> Js.Json.t

val read_save34_undo_member1_item_changes_item_member1 :
  Js.Json.t -> save34_undo_member1_item_changes_item_member1

val encode_save34_undo_member1_item_changes_item_member1 :
  save34_undo_member1_item_changes_item_member1 -> Js.Json.t

val read_literal_remove : Js.Json.t -> literal_remove
val encode_literal_remove : literal_remove -> Js.Json.t
val read_save35 : Js.Json.t -> save35
val encode_save35 : save35 -> Js.Json.t
val read_literal_35 : Js.Json.t -> literal_35
val encode_literal_35 : literal_35 -> Js.Json.t
val read_save35_checkpoint : Js.Json.t -> save35_checkpoint
val encode_save35_checkpoint : save35_checkpoint -> Js.Json.t
val read_save35_checkpoint_state : Js.Json.t -> save35_checkpoint_state
val encode_save35_checkpoint_state : save35_checkpoint_state -> Js.Json.t
val read_save35_progress : Js.Json.t -> save35_progress
val encode_save35_progress : save35_progress -> Js.Json.t
val read_save35_progress_entry : Js.Json.t -> save35_progress_entry
val encode_save35_progress_entry : save35_progress_entry -> Js.Json.t
val read_save35_progress_entry_kind : Js.Json.t -> save35_progress_entry_kind
val encode_save35_progress_entry_kind : save35_progress_entry_kind -> Js.Json.t
val read_literal_manual : Js.Json.t -> literal_manual
val encode_literal_manual : literal_manual -> Js.Json.t
val read_save36 : Js.Json.t -> save36
val encode_save36 : save36 -> Js.Json.t
val read_literal_36 : Js.Json.t -> literal_36
val encode_literal_36 : literal_36 -> Js.Json.t
val read_save37 : Js.Json.t -> save37
val encode_save37 : save37 -> Js.Json.t
val read_literal_37 : Js.Json.t -> literal_37
val encode_literal_37 : literal_37 -> Js.Json.t
val read_save37_progress : Js.Json.t -> save37_progress
val encode_save37_progress : save37_progress -> Js.Json.t
val read_save37_progress_entry : Js.Json.t -> save37_progress_entry
val encode_save37_progress_entry : save37_progress_entry -> Js.Json.t
val read_save37_progress_entry_member0 : Js.Json.t -> save37_progress_entry_member0
val encode_save37_progress_entry_member0 : save37_progress_entry_member0 -> Js.Json.t
val read_literal_flash : Js.Json.t -> literal_flash
val encode_literal_flash : literal_flash -> Js.Json.t
val read_history4_entries_item : Js.Json.t -> history4_entries_item
val encode_history4_entries_item : history4_entries_item -> Js.Json.t
val read_save38 : Js.Json.t -> save38
val encode_save38 : save38 -> Js.Json.t
val read_literal_38 : Js.Json.t -> literal_38
val encode_literal_38 : literal_38 -> Js.Json.t
val read_save39 : Js.Json.t -> save39
val encode_save39 : save39 -> Js.Json.t
val read_save39_member0 : Js.Json.t -> save39_member0
val encode_save39_member0 : save39_member0 -> Js.Json.t
val read_save39_member0_progress : Js.Json.t -> save39_member0_progress
val encode_save39_member0_progress : save39_member0_progress -> Js.Json.t
val read_literal_39 : Js.Json.t -> literal_39
val encode_literal_39 : literal_39 -> Js.Json.t
val read_save39_member0_checkpoint : Js.Json.t -> save39_member0_checkpoint
val encode_save39_member0_checkpoint : save39_member0_checkpoint -> Js.Json.t
val read_save39_member0_checkpoint_state : Js.Json.t -> save39_member0_checkpoint_state
val encode_save39_member0_checkpoint_state : save39_member0_checkpoint_state -> Js.Json.t

val read_save39_member0_checkpoint_state_factions :
  Js.Json.t -> save39_member0_checkpoint_state_factions

val encode_save39_member0_checkpoint_state_factions :
  save39_member0_checkpoint_state_factions -> Js.Json.t

val read_save39_member0_checkpoint_state_factions_item :
  Js.Json.t -> save39_member0_checkpoint_state_factions_item

val encode_save39_member0_checkpoint_state_factions_item :
  save39_member0_checkpoint_state_factions_item -> Js.Json.t

val read_save39_member0_checkpoint_state_factions_item_transfer :
  Js.Json.t -> save39_member0_checkpoint_state_factions_item_transfer

val encode_save39_member0_checkpoint_state_factions_item_transfer :
  save39_member0_checkpoint_state_factions_item_transfer -> Js.Json.t

val read_save39_member0_checkpoint_state_factions_item_transfer_member1 :
  Js.Json.t -> save39_member0_checkpoint_state_factions_item_transfer_member1

val encode_save39_member0_checkpoint_state_factions_item_transfer_member1 :
  save39_member0_checkpoint_state_factions_item_transfer_member1 -> Js.Json.t

val read_save39_member0_checkpoint_state_tiles_item_fog_member1 :
  Js.Json.t -> save39_member0_checkpoint_state_tiles_item_fog_member1

val encode_save39_member0_checkpoint_state_tiles_item_fog_member1 :
  save39_member0_checkpoint_state_tiles_item_fog_member1 -> Js.Json.t

val read_save39_member0_checkpoint_state_tiles_item_fog_member1_condition :
  Js.Json.t -> save39_member0_checkpoint_state_tiles_item_fog_member1_condition

val encode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition :
  save39_member0_checkpoint_state_tiles_item_fog_member1_condition -> Js.Json.t

val read_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0 :
  Js.Json.t -> save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0

val encode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0 :
  save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0 -> Js.Json.t

val read_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile :
  Js.Json.t -> save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile

val encode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile :
  save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile -> Js.Json.t

val read_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member1 :
  Js.Json.t -> save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member1

val encode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member1 :
  save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member1 -> Js.Json.t

val read_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member2 :
  Js.Json.t -> save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member2

val encode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member2 :
  save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member2 -> Js.Json.t

val read_save39_member0_checkpoint_state_ground_items_item_tile :
  Js.Json.t -> save39_member0_checkpoint_state_ground_items_item_tile

val encode_save39_member0_checkpoint_state_ground_items_item_tile :
  save39_member0_checkpoint_state_ground_items_item_tile -> Js.Json.t

val read_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member3 :
  Js.Json.t -> save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member3

val encode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member3 :
  save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member3 -> Js.Json.t

val read_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member3_binding :
  Js.Json.t ->
  save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member3_binding

val encode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member3_binding :
  save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile_member3_binding ->
  Js.Json.t

val read_literal_unit : Js.Json.t -> literal_unit
val encode_literal_unit : literal_unit -> Js.Json.t
val read_literal_mark : Js.Json.t -> literal_mark
val encode_literal_mark : literal_mark -> Js.Json.t
val read_literal_death : Js.Json.t -> literal_death
val encode_literal_death : literal_death -> Js.Json.t

val read_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member1 :
  Js.Json.t -> save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member1

val encode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member1 :
  save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member1 -> Js.Json.t

val read_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member9 :
  Js.Json.t -> save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member9

val encode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member9 :
  save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member9 -> Js.Json.t

val read_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member9_conditions :
  Js.Json.t -> save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member9_conditions

val encode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member9_conditions :
  save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member9_conditions -> Js.Json.t

val read_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member10 :
  Js.Json.t -> save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member10

val encode_save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member10 :
  save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member10 -> Js.Json.t

val read_save39_member0_checkpoint_state_ground_items :
  Js.Json.t -> save39_member0_checkpoint_state_ground_items

val encode_save39_member0_checkpoint_state_ground_items :
  save39_member0_checkpoint_state_ground_items -> Js.Json.t

val read_save39_member0_checkpoint_state_ground_items_item :
  Js.Json.t -> save39_member0_checkpoint_state_ground_items_item

val encode_save39_member0_checkpoint_state_ground_items_item :
  save39_member0_checkpoint_state_ground_items_item -> Js.Json.t

val read_save39_member0_checkpoint_state_tiles : Js.Json.t -> save39_member0_checkpoint_state_tiles

val encode_save39_member0_checkpoint_state_tiles :
  save39_member0_checkpoint_state_tiles -> Js.Json.t

val read_save39_member0_checkpoint_state_tiles_item :
  Js.Json.t -> save39_member0_checkpoint_state_tiles_item

val encode_save39_member0_checkpoint_state_tiles_item :
  save39_member0_checkpoint_state_tiles_item -> Js.Json.t

val read_save39_member0_checkpoint_state_tiles_item_fog :
  Js.Json.t -> save39_member0_checkpoint_state_tiles_item_fog

val encode_save39_member0_checkpoint_state_tiles_item_fog :
  save39_member0_checkpoint_state_tiles_item_fog -> Js.Json.t

val read_save39_member0_checkpoint_state_tiles_item_light :
  Js.Json.t -> save39_member0_checkpoint_state_tiles_item_light

val encode_save39_member0_checkpoint_state_tiles_item_light :
  save39_member0_checkpoint_state_tiles_item_light -> Js.Json.t

val read_save39_member0_checkpoint_state_tiles_item_light_member1 :
  Js.Json.t -> save39_member0_checkpoint_state_tiles_item_light_member1

val encode_save39_member0_checkpoint_state_tiles_item_light_member1 :
  save39_member0_checkpoint_state_tiles_item_light_member1 -> Js.Json.t

val read_save39_member0_checkpoint_state_units : Js.Json.t -> save39_member0_checkpoint_state_units

val encode_save39_member0_checkpoint_state_units :
  save39_member0_checkpoint_state_units -> Js.Json.t

val read_save39_member0_checkpoint_state_units_item :
  Js.Json.t -> save39_member0_checkpoint_state_units_item

val encode_save39_member0_checkpoint_state_units_item :
  save39_member0_checkpoint_state_units_item -> Js.Json.t

val read_save39_member0_checkpoint_state_campaign :
  Js.Json.t -> save39_member0_checkpoint_state_campaign

val encode_save39_member0_checkpoint_state_campaign :
  save39_member0_checkpoint_state_campaign -> Js.Json.t

val read_save39_member0_checkpoint_state_campaign_member1 :
  Js.Json.t -> save39_member0_checkpoint_state_campaign_member1

val encode_save39_member0_checkpoint_state_campaign_member1 :
  save39_member0_checkpoint_state_campaign_member1 -> Js.Json.t

val read_save39_member0_checkpoint_state_campaign_member1_scripts :
  Js.Json.t -> save39_member0_checkpoint_state_campaign_member1_scripts

val encode_save39_member0_checkpoint_state_campaign_member1_scripts :
  save39_member0_checkpoint_state_campaign_member1_scripts -> Js.Json.t

val read_save39_member0_checkpoint_state_campaign_member1_scripts_member1 :
  Js.Json.t -> save39_member0_checkpoint_state_campaign_member1_scripts_member1

val encode_save39_member0_checkpoint_state_campaign_member1_scripts_member1 :
  save39_member0_checkpoint_state_campaign_member1_scripts_member1 -> Js.Json.t

val read_save39_member0_checkpoint_state_campaign_member1_scripts_member1_marks :
  Js.Json.t -> save39_member0_checkpoint_state_campaign_member1_scripts_member1_marks

val encode_save39_member0_checkpoint_state_campaign_member1_scripts_member1_marks :
  save39_member0_checkpoint_state_campaign_member1_scripts_member1_marks -> Js.Json.t

val read_save39_member0_checkpoint_state_campaign_member1_scripts_member1_marks_item :
  Js.Json.t -> save39_member0_checkpoint_state_campaign_member1_scripts_member1_marks_item

val encode_save39_member0_checkpoint_state_campaign_member1_scripts_member1_marks_item :
  save39_member0_checkpoint_state_campaign_member1_scripts_member1_marks_item -> Js.Json.t

val read_save39_member0_checkpoint_state_campaign_member1_scripts_member1_pending :
  Js.Json.t -> save39_member0_checkpoint_state_campaign_member1_scripts_member1_pending

val encode_save39_member0_checkpoint_state_campaign_member1_scripts_member1_pending :
  save39_member0_checkpoint_state_campaign_member1_scripts_member1_pending -> Js.Json.t

val read_save39_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item :
  Js.Json.t -> save39_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item

val encode_save39_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item :
  save39_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item -> Js.Json.t

val read_save39_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item_area :
  Js.Json.t -> save39_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item_area

val encode_save39_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item_area :
  save39_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item_area -> Js.Json.t

val read_save39_member0_checkpoint_state_campaign_member1_evacuated :
  Js.Json.t -> save39_member0_checkpoint_state_campaign_member1_evacuated

val encode_save39_member0_checkpoint_state_campaign_member1_evacuated :
  save39_member0_checkpoint_state_campaign_member1_evacuated -> Js.Json.t

val read_save39_member0_checkpoint_state_campaign_member1_evacuated_member1 :
  Js.Json.t -> save39_member0_checkpoint_state_campaign_member1_evacuated_member1

val encode_save39_member0_checkpoint_state_campaign_member1_evacuated_member1 :
  save39_member0_checkpoint_state_campaign_member1_evacuated_member1 -> Js.Json.t

val read_save39_member0_checkpoint_state_campaign_member1_evacuated_member1_item :
  Js.Json.t -> save39_member0_checkpoint_state_campaign_member1_evacuated_member1_item

val encode_save39_member0_checkpoint_state_campaign_member1_evacuated_member1_item :
  save39_member0_checkpoint_state_campaign_member1_evacuated_member1_item -> Js.Json.t

val read_fragments_native_state38_campaign_member1_veterans :
  Js.Json.t -> fragments_native_state38_campaign_member1_veterans

val encode_fragments_native_state38_campaign_member1_veterans :
  fragments_native_state38_campaign_member1_veterans -> Js.Json.t

val read_fragments_native_state38_campaign_member1_veterans_member1 :
  Js.Json.t -> fragments_native_state38_campaign_member1_veterans_member1

val encode_fragments_native_state38_campaign_member1_veterans_member1 :
  fragments_native_state38_campaign_member1_veterans_member1 -> Js.Json.t

val read_fragments_native_state38_campaign_member1_veterans_member1_item :
  Js.Json.t -> fragments_native_state38_campaign_member1_veterans_member1_item

val encode_fragments_native_state38_campaign_member1_veterans_member1_item :
  fragments_native_state38_campaign_member1_veterans_member1_item -> Js.Json.t

val read_save39_member0_setup : Js.Json.t -> save39_member0_setup
val encode_save39_member0_setup : save39_member0_setup -> Js.Json.t
val read_save39_member0_setup_enemy : Js.Json.t -> save39_member0_setup_enemy
val encode_save39_member0_setup_enemy : save39_member0_setup_enemy -> Js.Json.t
val read_save39_member1 : Js.Json.t -> save39_member1
val encode_save39_member1 : save39_member1 -> Js.Json.t
val read_save39_member1_progress : Js.Json.t -> save39_member1_progress
val encode_save39_member1_progress : save39_member1_progress -> Js.Json.t
val read_history1 : Js.Json.t -> history1
val encode_history1 : history1 -> Js.Json.t
val read_literal_1 : Js.Json.t -> literal_1
val encode_literal_1 : literal_1 -> Js.Json.t
val read_history1_entries : Js.Json.t -> history1_entries
val encode_history1_entries : history1_entries -> Js.Json.t
val read_history2 : Js.Json.t -> history2
val encode_history2 : history2 -> Js.Json.t
val read_history2_entries : Js.Json.t -> history2_entries
val encode_history2_entries : history2_entries -> Js.Json.t
val read_history2_entries_item : Js.Json.t -> history2_entries_item
val encode_history2_entries_item : history2_entries_item -> Js.Json.t
val read_history3 : Js.Json.t -> history3
val encode_history3 : history3 -> Js.Json.t
val read_history3_entries : Js.Json.t -> history3_entries
val encode_history3_entries : history3_entries -> Js.Json.t
val read_history3_entries_item : Js.Json.t -> history3_entries_item
val encode_history3_entries_item : history3_entries_item -> Js.Json.t
val read_history3_entries_item_kind : Js.Json.t -> history3_entries_item_kind
val encode_history3_entries_item_kind : history3_entries_item_kind -> Js.Json.t
val read_history4 : Js.Json.t -> history4
val encode_history4 : history4 -> Js.Json.t
val read_history4_entries : Js.Json.t -> history4_entries
val encode_history4_entries : history4_entries -> Js.Json.t
val read_fragments : Js.Json.t -> fragments
val encode_fragments : fragments -> Js.Json.t
val read_fragments_native_state38 : Js.Json.t -> fragments_native_state38
val encode_fragments_native_state38 : fragments_native_state38 -> Js.Json.t
val read_fragments_native_state38_factions : Js.Json.t -> fragments_native_state38_factions
val encode_fragments_native_state38_factions : fragments_native_state38_factions -> Js.Json.t

val read_fragments_native_state38_factions_item :
  Js.Json.t -> fragments_native_state38_factions_item

val encode_fragments_native_state38_factions_item :
  fragments_native_state38_factions_item -> Js.Json.t

val read_fragments_native_state38_factions_item_transfer :
  Js.Json.t -> fragments_native_state38_factions_item_transfer

val encode_fragments_native_state38_factions_item_transfer :
  fragments_native_state38_factions_item_transfer -> Js.Json.t

val read_fragments_native_state38_factions_item_transfer_member1 :
  Js.Json.t -> fragments_native_state38_factions_item_transfer_member1

val encode_fragments_native_state38_factions_item_transfer_member1 :
  fragments_native_state38_factions_item_transfer_member1 -> Js.Json.t

val read_fragments_native_state38_tiles_item_fog_member1 :
  Js.Json.t -> fragments_native_state38_tiles_item_fog_member1

val encode_fragments_native_state38_tiles_item_fog_member1 :
  fragments_native_state38_tiles_item_fog_member1 -> Js.Json.t

val read_fragments_native_state38_tiles_item_fog_member1_condition :
  Js.Json.t -> fragments_native_state38_tiles_item_fog_member1_condition

val encode_fragments_native_state38_tiles_item_fog_member1_condition :
  fragments_native_state38_tiles_item_fog_member1_condition -> Js.Json.t

val read_fragments_native_state38_tiles_item_fog_member1_condition_member9 :
  Js.Json.t -> fragments_native_state38_tiles_item_fog_member1_condition_member9

val encode_fragments_native_state38_tiles_item_fog_member1_condition_member9 :
  fragments_native_state38_tiles_item_fog_member1_condition_member9 -> Js.Json.t

val read_fragments_native_state38_tiles_item_fog_member1_condition_member9_conditions :
  Js.Json.t -> fragments_native_state38_tiles_item_fog_member1_condition_member9_conditions

val encode_fragments_native_state38_tiles_item_fog_member1_condition_member9_conditions :
  fragments_native_state38_tiles_item_fog_member1_condition_member9_conditions -> Js.Json.t

val read_fragments_native_state38_tiles_item_fog_member1_condition_member10 :
  Js.Json.t -> fragments_native_state38_tiles_item_fog_member1_condition_member10

val encode_fragments_native_state38_tiles_item_fog_member1_condition_member10 :
  fragments_native_state38_tiles_item_fog_member1_condition_member10 -> Js.Json.t

val read_fragments_native_state38_tiles : Js.Json.t -> fragments_native_state38_tiles
val encode_fragments_native_state38_tiles : fragments_native_state38_tiles -> Js.Json.t
val read_fragments_native_state38_tiles_item : Js.Json.t -> fragments_native_state38_tiles_item
val encode_fragments_native_state38_tiles_item : fragments_native_state38_tiles_item -> Js.Json.t

val read_fragments_native_state38_tiles_item_fog :
  Js.Json.t -> fragments_native_state38_tiles_item_fog

val encode_fragments_native_state38_tiles_item_fog :
  fragments_native_state38_tiles_item_fog -> Js.Json.t

val read_fragments_native_state38_tiles_item_light :
  Js.Json.t -> fragments_native_state38_tiles_item_light

val encode_fragments_native_state38_tiles_item_light :
  fragments_native_state38_tiles_item_light -> Js.Json.t

val read_fragments_native_state38_tiles_item_light_member1 :
  Js.Json.t -> fragments_native_state38_tiles_item_light_member1

val encode_fragments_native_state38_tiles_item_light_member1 :
  fragments_native_state38_tiles_item_light_member1 -> Js.Json.t

val read_fragments_native_state38_units : Js.Json.t -> fragments_native_state38_units
val encode_fragments_native_state38_units : fragments_native_state38_units -> Js.Json.t
val read_fragments_native_state38_units_item : Js.Json.t -> fragments_native_state38_units_item
val encode_fragments_native_state38_units_item : fragments_native_state38_units_item -> Js.Json.t
val read_fragments_native_state38_campaign : Js.Json.t -> fragments_native_state38_campaign
val encode_fragments_native_state38_campaign : fragments_native_state38_campaign -> Js.Json.t

val read_fragments_native_state38_campaign_member1 :
  Js.Json.t -> fragments_native_state38_campaign_member1

val encode_fragments_native_state38_campaign_member1 :
  fragments_native_state38_campaign_member1 -> Js.Json.t

val read_fragments_native_state38_campaign_member1_scripts :
  Js.Json.t -> fragments_native_state38_campaign_member1_scripts

val encode_fragments_native_state38_campaign_member1_scripts :
  fragments_native_state38_campaign_member1_scripts -> Js.Json.t

val read_fragments_native_state38_campaign_member1_scripts_member1 :
  Js.Json.t -> fragments_native_state38_campaign_member1_scripts_member1

val encode_fragments_native_state38_campaign_member1_scripts_member1 :
  fragments_native_state38_campaign_member1_scripts_member1 -> Js.Json.t

val read_fragments_native_state38_campaign_member1_scripts_member1_marks :
  Js.Json.t -> fragments_native_state38_campaign_member1_scripts_member1_marks

val encode_fragments_native_state38_campaign_member1_scripts_member1_marks :
  fragments_native_state38_campaign_member1_scripts_member1_marks -> Js.Json.t

val read_fragments_native_state38_campaign_member1_scripts_member1_marks_item :
  Js.Json.t -> fragments_native_state38_campaign_member1_scripts_member1_marks_item

val encode_fragments_native_state38_campaign_member1_scripts_member1_marks_item :
  fragments_native_state38_campaign_member1_scripts_member1_marks_item -> Js.Json.t

val read_fragments_native_state38_campaign_member1_evacuated :
  Js.Json.t -> fragments_native_state38_campaign_member1_evacuated

val encode_fragments_native_state38_campaign_member1_evacuated :
  fragments_native_state38_campaign_member1_evacuated -> Js.Json.t

val read_fragments_native_state38_campaign_member1_evacuated_member1 :
  Js.Json.t -> fragments_native_state38_campaign_member1_evacuated_member1

val encode_fragments_native_state38_campaign_member1_evacuated_member1 :
  fragments_native_state38_campaign_member1_evacuated_member1 -> Js.Json.t

val read_fragments_native_state38_campaign_member1_evacuated_member1_item :
  Js.Json.t -> fragments_native_state38_campaign_member1_evacuated_member1_item

val encode_fragments_native_state38_campaign_member1_evacuated_member1_item :
  fragments_native_state38_campaign_member1_evacuated_member1_item -> Js.Json.t

val read_fragments_delta19 : Js.Json.t -> fragments_delta19
val encode_fragments_delta19 : fragments_delta19 -> Js.Json.t
val read_fragments_delta19_item : Js.Json.t -> fragments_delta19_item
val encode_fragments_delta19_item : fragments_delta19_item -> Js.Json.t
val read_fragments_light28 : Js.Json.t -> fragments_light28
val encode_fragments_light28 : fragments_light28 -> Js.Json.t
val read_native1 : Js.Json.t -> native1
val encode_native1 : native1 -> Js.Json.t
val read_literal_rust_wasm : Js.Json.t -> literal_rust_wasm
val encode_literal_rust_wasm : literal_rust_wasm -> Js.Json.t
val read_native1_version : Js.Json.t -> native1_version
val encode_native1_version : native1_version -> Js.Json.t
val read_native1_saved : Js.Json.t -> native1_saved
val encode_native1_saved : native1_saved -> Js.Json.t
val read_javascript1 : Js.Json.t -> javascript1
val encode_javascript1 : javascript1 -> Js.Json.t

val read_javascript1_campaign_deployment_format :
  Js.Json.t -> javascript1_campaign_deployment_format

val encode_javascript1_campaign_deployment_format :
  javascript1_campaign_deployment_format -> Js.Json.t

val read_literal_named_waves_v1 : Js.Json.t -> literal_named_waves_v1
val encode_literal_named_waves_v1 : literal_named_waves_v1 -> Js.Json.t
val read_javascript1_state : Js.Json.t -> javascript1_state
val encode_javascript1_state : javascript1_state -> Js.Json.t
val read_javascript1_state_map_revision : Js.Json.t -> javascript1_state_map_revision
val encode_javascript1_state_map_revision : javascript1_state_map_revision -> Js.Json.t
val read_javascript1_state_units : Js.Json.t -> javascript1_state_units
val encode_javascript1_state_units : javascript1_state_units -> Js.Json.t
val read_javascript1_state_units_item : Js.Json.t -> javascript1_state_units_item
val encode_javascript1_state_units_item : javascript1_state_units_item -> Js.Json.t
val read_javascript1_state_units_item_items : Js.Json.t -> javascript1_state_units_item_items
val encode_javascript1_state_units_item_items : javascript1_state_units_item_items -> Js.Json.t

val read_javascript1_state_units_item_items_member1 :
  Js.Json.t -> javascript1_state_units_item_items_member1

val encode_javascript1_state_units_item_items_member1 :
  javascript1_state_units_item_items_member1 -> Js.Json.t

val read_javascript1_state_units_item_items_member1_item :
  Js.Json.t -> javascript1_state_units_item_items_member1_item

val encode_javascript1_state_units_item_items_member1_item :
  javascript1_state_units_item_items_member1_item -> Js.Json.t

val read_javascript1_state_units_item_items_member1_item_attacks :
  Js.Json.t -> javascript1_state_units_item_items_member1_item_attacks

val encode_javascript1_state_units_item_items_member1_item_attacks :
  javascript1_state_units_item_items_member1_item_attacks -> Js.Json.t

val read_javascript1_state_units_item_items_member1_item_attacks_item :
  Js.Json.t -> javascript1_state_units_item_items_member1_item_attacks_item

val encode_javascript1_state_units_item_items_member1_item_attacks_item :
  javascript1_state_units_item_items_member1_item_attacks_item -> Js.Json.t

val read_javascript1_state_units_item_type_ : Js.Json.t -> javascript1_state_units_item_type_
val encode_javascript1_state_units_item_type_ : javascript1_state_units_item_type_ -> Js.Json.t
val read_literal_ranger : Js.Json.t -> literal_ranger
val encode_literal_ranger : literal_ranger -> Js.Json.t
val read_literal_healer : Js.Json.t -> literal_healer
val encode_literal_healer : literal_healer -> Js.Json.t
val read_literal_briarweaver : Js.Json.t -> literal_briarweaver
val encode_literal_briarweaver : literal_briarweaver -> Js.Json.t
val read_literal_deatharcher : Js.Json.t -> literal_deatharcher
val encode_literal_deatharcher : literal_deatharcher -> Js.Json.t
val read_literal_direwolf : Js.Json.t -> literal_direwolf
val encode_literal_direwolf : literal_direwolf -> Js.Json.t
val read_literal_greatboar : Js.Json.t -> literal_greatboar
val encode_literal_greatboar : literal_greatboar -> Js.Json.t
val read_literal_wraith : Js.Json.t -> literal_wraith
val encode_literal_wraith : literal_wraith -> Js.Json.t
val read_literal_spectre : Js.Json.t -> literal_spectre
val encode_literal_spectre : literal_spectre -> Js.Json.t
val read_literal_lancer : Js.Json.t -> literal_lancer
val encode_literal_lancer : literal_lancer -> Js.Json.t
val read_literal_mage : Js.Json.t -> literal_mage
val encode_literal_mage : literal_mage -> Js.Json.t
val read_literal_marksman : Js.Json.t -> literal_marksman
val encode_literal_marksman : literal_marksman -> Js.Json.t
val read_literal_merfolk_maceguard_l2 : Js.Json.t -> literal_merfolk_maceguard_l2
val encode_literal_merfolk_maceguard_l2 : literal_merfolk_maceguard_l2 -> Js.Json.t
val read_literal_revenant : Js.Json.t -> literal_revenant
val encode_literal_revenant : literal_revenant -> Js.Json.t
val read_literal_skeleton_impact_caster_l2 : Js.Json.t -> literal_skeleton_impact_caster_l2
val encode_literal_skeleton_impact_caster_l2 : literal_skeleton_impact_caster_l2 -> Js.Json.t
val read_literal_soldier : Js.Json.t -> literal_soldier
val encode_literal_soldier : literal_soldier -> Js.Json.t
val read_literal_tidehunter : Js.Json.t -> literal_tidehunter
val encode_literal_tidehunter : literal_tidehunter -> Js.Json.t
val read_literal_tidesage : Js.Json.t -> literal_tidesage
val encode_literal_tidesage : literal_tidesage -> Js.Json.t
val read_literal_tridentguard : Js.Json.t -> literal_tridentguard
val encode_literal_tridentguard : literal_tridentguard -> Js.Json.t
val read_literal_warden : Js.Json.t -> literal_warden
val encode_literal_warden : literal_warden -> Js.Json.t
val read_literal_champion : Js.Json.t -> literal_champion
val encode_literal_champion : literal_champion -> Js.Json.t
val read_literal_hammerman : Js.Json.t -> literal_hammerman
val encode_literal_hammerman : literal_hammerman -> Js.Json.t
val read_literal_warhammer : Js.Json.t -> literal_warhammer
val encode_literal_warhammer : literal_warhammer -> Js.Json.t
val read_literal_sentinel : Js.Json.t -> literal_sentinel
val encode_literal_sentinel : literal_sentinel -> Js.Json.t
val read_literal_halberdier : Js.Json.t -> literal_halberdier
val encode_literal_halberdier : literal_halberdier -> Js.Json.t
val read_literal_halberd_master : Js.Json.t -> literal_halberd_master
val encode_literal_halberd_master : literal_halberd_master -> Js.Json.t
val read_literal_shadowblade : Js.Json.t -> literal_shadowblade
val encode_literal_shadowblade : literal_shadowblade -> Js.Json.t
val read_literal_slinger : Js.Json.t -> literal_slinger
val encode_literal_slinger : literal_slinger -> Js.Json.t
val read_literal_fireslinger : Js.Json.t -> literal_fireslinger
val encode_literal_fireslinger : literal_fireslinger -> Js.Json.t
val read_literal_sharpshooter : Js.Json.t -> literal_sharpshooter
val encode_literal_sharpshooter : literal_sharpshooter -> Js.Json.t
val read_literal_crossbowman : Js.Json.t -> literal_crossbowman
val encode_literal_crossbowman : literal_crossbowman -> Js.Json.t
val read_literal_arbalester : Js.Json.t -> literal_arbalester
val encode_literal_arbalester : literal_arbalester -> Js.Json.t
val read_literal_archmage : Js.Json.t -> literal_archmage
val encode_literal_archmage : literal_archmage -> Js.Json.t
val read_literal_high_healer : Js.Json.t -> literal_high_healer
val encode_literal_high_healer : literal_high_healer -> Js.Json.t
val read_literal_knight : Js.Json.t -> literal_knight
val encode_literal_knight : literal_knight -> Js.Json.t
val read_literal_horse_bowman : Js.Json.t -> literal_horse_bowman
val encode_literal_horse_bowman : literal_horse_bowman -> Js.Json.t
val read_literal_horse_marksman : Js.Json.t -> literal_horse_marksman
val encode_literal_horse_marksman : literal_horse_marksman -> Js.Json.t
val read_literal_skeleton_dreadguard : Js.Json.t -> literal_skeleton_dreadguard
val encode_literal_skeleton_dreadguard : literal_skeleton_dreadguard -> Js.Json.t
val read_literal_skeleton_flanker_l2 : Js.Json.t -> literal_skeleton_flanker_l2
val encode_literal_skeleton_flanker_l2 : literal_skeleton_flanker_l2 -> Js.Json.t
val read_literal_skeleton_flanker_l3 : Js.Json.t -> literal_skeleton_flanker_l3
val encode_literal_skeleton_flanker_l3 : literal_skeleton_flanker_l3 -> Js.Json.t
val read_literal_skeleton_impact_caster_l3 : Js.Json.t -> literal_skeleton_impact_caster_l3
val encode_literal_skeleton_impact_caster_l3 : literal_skeleton_impact_caster_l3 -> Js.Json.t
val read_literal_skeleton_scythe_caster_l2 : Js.Json.t -> literal_skeleton_scythe_caster_l2
val encode_literal_skeleton_scythe_caster_l2 : literal_skeleton_scythe_caster_l2 -> Js.Json.t
val read_literal_skeleton_scythe_caster_l3 : Js.Json.t -> literal_skeleton_scythe_caster_l3
val encode_literal_skeleton_scythe_caster_l3 : literal_skeleton_scythe_caster_l3 -> Js.Json.t
val read_literal_skeleton_rider_l2 : Js.Json.t -> literal_skeleton_rider_l2
val encode_literal_skeleton_rider_l2 : literal_skeleton_rider_l2 -> Js.Json.t
val read_literal_skeleton_rider_l3 : Js.Json.t -> literal_skeleton_rider_l3
val encode_literal_skeleton_rider_l3 : literal_skeleton_rider_l3 -> Js.Json.t
val read_literal_skeleton_bow_master : Js.Json.t -> literal_skeleton_bow_master
val encode_literal_skeleton_bow_master : literal_skeleton_bow_master -> Js.Json.t
val read_literal_skeleton_axeman_l2 : Js.Json.t -> literal_skeleton_axeman_l2
val encode_literal_skeleton_axeman_l2 : literal_skeleton_axeman_l2 -> Js.Json.t
val read_literal_skeleton_axeman_l3 : Js.Json.t -> literal_skeleton_axeman_l3
val encode_literal_skeleton_axeman_l3 : literal_skeleton_axeman_l3 -> Js.Json.t
val read_literal_woodland_alpha_wolf : Js.Json.t -> literal_woodland_alpha_wolf
val encode_literal_woodland_alpha_wolf : literal_woodland_alpha_wolf -> Js.Json.t
val read_literal_woodland_ancient_boar : Js.Json.t -> literal_woodland_ancient_boar
val encode_literal_woodland_ancient_boar : literal_woodland_ancient_boar -> Js.Json.t
val read_literal_woodland_thorn_oracle : Js.Json.t -> literal_woodland_thorn_oracle
val encode_literal_woodland_thorn_oracle : literal_woodland_thorn_oracle -> Js.Json.t
val read_literal_woodland_bramble_warden : Js.Json.t -> literal_woodland_bramble_warden
val encode_literal_woodland_bramble_warden : literal_woodland_bramble_warden -> Js.Json.t
val read_literal_woodland_thornbinder : Js.Json.t -> literal_woodland_thornbinder
val encode_literal_woodland_thornbinder : literal_woodland_thornbinder -> Js.Json.t
val read_literal_fox_spearguard_l2 : Js.Json.t -> literal_fox_spearguard_l2
val encode_literal_fox_spearguard_l2 : literal_fox_spearguard_l2 -> Js.Json.t
val read_literal_fox_spearguard_l3 : Js.Json.t -> literal_fox_spearguard_l3
val encode_literal_fox_spearguard_l3 : literal_fox_spearguard_l3 -> Js.Json.t
val read_literal_treant_l2 : Js.Json.t -> literal_treant_l2
val encode_literal_treant_l2 : literal_treant_l2 -> Js.Json.t
val read_literal_treant_l3 : Js.Json.t -> literal_treant_l3
val encode_literal_treant_l3 : literal_treant_l3 -> Js.Json.t
val read_literal_merfolk_harpoon_master : Js.Json.t -> literal_merfolk_harpoon_master
val encode_literal_merfolk_harpoon_master : literal_merfolk_harpoon_master -> Js.Json.t
val read_literal_merfolk_netter_l2 : Js.Json.t -> literal_merfolk_netter_l2
val encode_literal_merfolk_netter_l2 : literal_merfolk_netter_l2 -> Js.Json.t
val read_literal_merfolk_netter_l3 : Js.Json.t -> literal_merfolk_netter_l3
val encode_literal_merfolk_netter_l3 : literal_merfolk_netter_l3 -> Js.Json.t
val read_literal_merfolk_reef_champion : Js.Json.t -> literal_merfolk_reef_champion
val encode_literal_merfolk_reef_champion : literal_merfolk_reef_champion -> Js.Json.t
val read_literal_merfolk_trident_l2 : Js.Json.t -> literal_merfolk_trident_l2
val encode_literal_merfolk_trident_l2 : literal_merfolk_trident_l2 -> Js.Json.t
val read_literal_merfolk_trident_l3 : Js.Json.t -> literal_merfolk_trident_l3
val encode_literal_merfolk_trident_l3 : literal_merfolk_trident_l3 -> Js.Json.t
val read_literal_merfolk_tide_oracle : Js.Json.t -> literal_merfolk_tide_oracle
val encode_literal_merfolk_tide_oracle : literal_merfolk_tide_oracle -> Js.Json.t
val read_literal_merfolk_tidecrusher_l2 : Js.Json.t -> literal_merfolk_tidecrusher_l2
val encode_literal_merfolk_tidecrusher_l2 : literal_merfolk_tidecrusher_l2 -> Js.Json.t
val read_literal_merfolk_tidecrusher_l3 : Js.Json.t -> literal_merfolk_tidecrusher_l3
val encode_literal_merfolk_tidecrusher_l3 : literal_merfolk_tidecrusher_l3 -> Js.Json.t
val read_literal_merfolk_maceguard_l3 : Js.Json.t -> literal_merfolk_maceguard_l3
val encode_literal_merfolk_maceguard_l3 : literal_merfolk_maceguard_l3 -> Js.Json.t
val read_literal_dragon : Js.Json.t -> literal_dragon
val encode_literal_dragon : literal_dragon -> Js.Json.t
val read_literal_skeletal_hound : Js.Json.t -> literal_skeletal_hound
val encode_literal_skeletal_hound : literal_skeletal_hound -> Js.Json.t
val read_literal_captives : Js.Json.t -> literal_captives
val encode_literal_captives : literal_captives -> Js.Json.t
val read_literal_zombie_walker : Js.Json.t -> literal_zombie_walker
val encode_literal_zombie_walker : literal_zombie_walker -> Js.Json.t
val read_literal_zombie_decay_caster : Js.Json.t -> literal_zombie_decay_caster
val encode_literal_zombie_decay_caster : literal_zombie_decay_caster -> Js.Json.t
val read_literal_bear : Js.Json.t -> literal_bear
val encode_literal_bear : literal_bear -> Js.Json.t
val read_literal_giant_bat : Js.Json.t -> literal_giant_bat
val encode_literal_giant_bat : literal_giant_bat -> Js.Json.t
val read_literal_giant_spider : Js.Json.t -> literal_giant_spider
val encode_literal_giant_spider : literal_giant_spider -> Js.Json.t
val read_literal_commander : Js.Json.t -> literal_commander
val encode_literal_commander : literal_commander -> Js.Json.t
val read_literal_grovewarden : Js.Json.t -> literal_grovewarden
val encode_literal_grovewarden : literal_grovewarden -> Js.Json.t
val read_literal_lich : Js.Json.t -> literal_lich
val encode_literal_lich : literal_lich -> Js.Json.t
val read_literal_marshal : Js.Json.t -> literal_marshal
val encode_literal_marshal : literal_marshal -> Js.Json.t
val read_literal_necromancer : Js.Json.t -> literal_necromancer
val encode_literal_necromancer : literal_necromancer -> Js.Json.t
val read_literal_tidekeeper : Js.Json.t -> literal_tidekeeper
val encode_literal_tidekeeper : literal_tidekeeper -> Js.Json.t
val read_literal_tideseer : Js.Json.t -> literal_tideseer
val encode_literal_tideseer : literal_tideseer -> Js.Json.t
val read_literal_wildkeeper : Js.Json.t -> literal_wildkeeper
val encode_literal_wildkeeper : literal_wildkeeper -> Js.Json.t
val read_literal_grand_marshal : Js.Json.t -> literal_grand_marshal
val encode_literal_grand_marshal : literal_grand_marshal -> Js.Json.t
val read_literal_lich_commander_l3 : Js.Json.t -> literal_lich_commander_l3
val encode_literal_lich_commander_l3 : literal_lich_commander_l3 -> Js.Json.t
val read_literal_woodland_grove_oracle : Js.Json.t -> literal_woodland_grove_oracle
val encode_literal_woodland_grove_oracle : literal_woodland_grove_oracle -> Js.Json.t
val read_literal_merfolk_tide_sovereign : Js.Json.t -> literal_merfolk_tide_sovereign
val encode_literal_merfolk_tide_sovereign : literal_merfolk_tide_sovereign -> Js.Json.t
val read_literal_adept : Js.Json.t -> literal_adept
val encode_literal_adept : literal_adept -> Js.Json.t
val read_literal_archer : Js.Json.t -> literal_archer
val encode_literal_archer : literal_archer -> Js.Json.t
val read_literal_boar : Js.Json.t -> literal_boar
val encode_literal_boar : literal_boar -> Js.Json.t
val read_literal_bonearcher : Js.Json.t -> literal_bonearcher
val encode_literal_bonearcher : literal_bonearcher -> Js.Json.t
val read_literal_ghost : Js.Json.t -> literal_ghost
val encode_literal_ghost : literal_ghost -> Js.Json.t
val read_literal_harpooner : Js.Json.t -> literal_harpooner
val encode_literal_harpooner : literal_harpooner -> Js.Json.t
val read_literal_merfolk_maceguard : Js.Json.t -> literal_merfolk_maceguard
val encode_literal_merfolk_maceguard : literal_merfolk_maceguard -> Js.Json.t
val read_literal_reefguard : Js.Json.t -> literal_reefguard
val encode_literal_reefguard : literal_reefguard -> Js.Json.t
val read_literal_rider : Js.Json.t -> literal_rider
val encode_literal_rider : literal_rider -> Js.Json.t
val read_literal_scout : Js.Json.t -> literal_scout
val encode_literal_scout : literal_scout -> Js.Json.t
val read_literal_skeleton_impact_caster : Js.Json.t -> literal_skeleton_impact_caster
val encode_literal_skeleton_impact_caster : literal_skeleton_impact_caster -> Js.Json.t
val read_literal_skeleton : Js.Json.t -> literal_skeleton
val encode_literal_skeleton : literal_skeleton -> Js.Json.t
val read_literal_spearman : Js.Json.t -> literal_spearman
val encode_literal_spearman : literal_spearman -> Js.Json.t
val read_literal_swordsman : Js.Json.t -> literal_swordsman
val encode_literal_swordsman : literal_swordsman -> Js.Json.t
val read_literal_thorncaller : Js.Json.t -> literal_thorncaller
val encode_literal_thorncaller : literal_thorncaller -> Js.Json.t
val read_literal_wavemender : Js.Json.t -> literal_wavemender
val encode_literal_wavemender : literal_wavemender -> Js.Json.t
val read_literal_wolf : Js.Json.t -> literal_wolf
val encode_literal_wolf : literal_wolf -> Js.Json.t
val read_literal_horse_archer : Js.Json.t -> literal_horse_archer
val encode_literal_horse_archer : literal_horse_archer -> Js.Json.t
val read_literal_skeleton_rider : Js.Json.t -> literal_skeleton_rider
val encode_literal_skeleton_rider : literal_skeleton_rider -> Js.Json.t
val read_literal_skeleton_axeman : Js.Json.t -> literal_skeleton_axeman
val encode_literal_skeleton_axeman : literal_skeleton_axeman -> Js.Json.t
val read_literal_fox_spearguard : Js.Json.t -> literal_fox_spearguard
val encode_literal_fox_spearguard : literal_fox_spearguard -> Js.Json.t
val read_literal_treant : Js.Json.t -> literal_treant
val encode_literal_treant : literal_treant -> Js.Json.t
val read_javascript1_state_villages : Js.Json.t -> javascript1_state_villages
val encode_javascript1_state_villages : javascript1_state_villages -> Js.Json.t
val read_javascript1_state_villages_item : Js.Json.t -> javascript1_state_villages_item
val encode_javascript1_state_villages_item : javascript1_state_villages_item -> Js.Json.t
val read_javascript1_state_sites : Js.Json.t -> javascript1_state_sites
val encode_javascript1_state_sites : javascript1_state_sites -> Js.Json.t
val read_javascript1_state_campaign : Js.Json.t -> javascript1_state_campaign
val encode_javascript1_state_campaign : javascript1_state_campaign -> Js.Json.t
val read_javascript1_state_campaign_member1 : Js.Json.t -> javascript1_state_campaign_member1
val encode_javascript1_state_campaign_member1 : javascript1_state_campaign_member1 -> Js.Json.t
val read_javascript1_state_campaign_member1_id : Js.Json.t -> javascript1_state_campaign_member1_id

val encode_javascript1_state_campaign_member1_id :
  javascript1_state_campaign_member1_id -> Js.Json.t

val read_literal_lantern_road : Js.Json.t -> literal_lantern_road
val encode_literal_lantern_road : literal_lantern_road -> Js.Json.t
val read_literal_green_accord : Js.Json.t -> literal_green_accord
val encode_literal_green_accord : literal_green_accord -> Js.Json.t
val read_literal_ashen_oath : Js.Json.t -> literal_ashen_oath
val encode_literal_ashen_oath : literal_ashen_oath -> Js.Json.t
val read_literal_borrowed_tide : Js.Json.t -> literal_borrowed_tide
val encode_literal_borrowed_tide : literal_borrowed_tide -> Js.Json.t
val read_literal_mirror_night : Js.Json.t -> literal_mirror_night
val encode_literal_mirror_night : literal_mirror_night -> Js.Json.t
val read_literal_bell_night : Js.Json.t -> literal_bell_night
val encode_literal_bell_night : literal_bell_night -> Js.Json.t

val read_javascript1_state_campaign_member1_recall :
  Js.Json.t -> javascript1_state_campaign_member1_recall

val encode_javascript1_state_campaign_member1_recall :
  javascript1_state_campaign_member1_recall -> Js.Json.t

val read_javascript1_state_campaign_member1_veterans :
  Js.Json.t -> javascript1_state_campaign_member1_veterans

val encode_javascript1_state_campaign_member1_veterans :
  javascript1_state_campaign_member1_veterans -> Js.Json.t

val read_javascript1_state_campaign_member1_veterans_member1 :
  Js.Json.t -> javascript1_state_campaign_member1_veterans_member1

val encode_javascript1_state_campaign_member1_veterans_member1 :
  javascript1_state_campaign_member1_veterans_member1 -> Js.Json.t

val read_javascript1_state_campaign_member1_veterans_member1_item :
  Js.Json.t -> javascript1_state_campaign_member1_veterans_member1_item

val encode_javascript1_state_campaign_member1_veterans_member1_item :
  javascript1_state_campaign_member1_veterans_member1_item -> Js.Json.t

val read_javascript1_state_campaign_member1_dialogue :
  Js.Json.t -> javascript1_state_campaign_member1_dialogue

val encode_javascript1_state_campaign_member1_dialogue :
  javascript1_state_campaign_member1_dialogue -> Js.Json.t

val read_javascript1_state_campaign_member1_dialogue_item :
  Js.Json.t -> javascript1_state_campaign_member1_dialogue_item

val encode_javascript1_state_campaign_member1_dialogue_item :
  javascript1_state_campaign_member1_dialogue_item -> Js.Json.t

val read_javascript1_state_campaign_member1_dialogue_item_portrait :
  Js.Json.t -> javascript1_state_campaign_member1_dialogue_item_portrait

val encode_javascript1_state_campaign_member1_dialogue_item_portrait :
  javascript1_state_campaign_member1_dialogue_item_portrait -> Js.Json.t

val read_literal_worker : Js.Json.t -> literal_worker
val encode_literal_worker : literal_worker -> Js.Json.t
val read_save40 : Js.Json.t -> save40
val encode_save40 : save40 -> Js.Json.t
val read_save40_member0 : Js.Json.t -> save40_member0
val encode_save40_member0 : save40_member0 -> Js.Json.t
val read_literal_40 : Js.Json.t -> literal_40
val encode_literal_40 : literal_40 -> Js.Json.t
val read_save40_member0_checkpoint : Js.Json.t -> save40_member0_checkpoint
val encode_save40_member0_checkpoint : save40_member0_checkpoint -> Js.Json.t
val read_save40_member0_checkpoint_state : Js.Json.t -> save40_member0_checkpoint_state
val encode_save40_member0_checkpoint_state : save40_member0_checkpoint_state -> Js.Json.t

val read_save40_member0_checkpoint_state_factions :
  Js.Json.t -> save40_member0_checkpoint_state_factions

val encode_save40_member0_checkpoint_state_factions :
  save40_member0_checkpoint_state_factions -> Js.Json.t

val read_save40_member0_checkpoint_state_factions_item :
  Js.Json.t -> save40_member0_checkpoint_state_factions_item

val encode_save40_member0_checkpoint_state_factions_item :
  save40_member0_checkpoint_state_factions_item -> Js.Json.t

val read_save40_member0_checkpoint_state_factions_item_transfer :
  Js.Json.t -> save40_member0_checkpoint_state_factions_item_transfer

val encode_save40_member0_checkpoint_state_factions_item_transfer :
  save40_member0_checkpoint_state_factions_item_transfer -> Js.Json.t

val read_save40_member0_checkpoint_state_factions_item_transfer_member1 :
  Js.Json.t -> save40_member0_checkpoint_state_factions_item_transfer_member1

val encode_save40_member0_checkpoint_state_factions_item_transfer_member1 :
  save40_member0_checkpoint_state_factions_item_transfer_member1 -> Js.Json.t

val read_save40_member0_checkpoint_state_tiles_item_fog_member1 :
  Js.Json.t -> save40_member0_checkpoint_state_tiles_item_fog_member1

val encode_save40_member0_checkpoint_state_tiles_item_fog_member1 :
  save40_member0_checkpoint_state_tiles_item_fog_member1 -> Js.Json.t

val read_save40_member0_checkpoint_state_tiles_item_fog_member1_condition :
  Js.Json.t -> save40_member0_checkpoint_state_tiles_item_fog_member1_condition

val encode_save40_member0_checkpoint_state_tiles_item_fog_member1_condition :
  save40_member0_checkpoint_state_tiles_item_fog_member1_condition -> Js.Json.t

val read_save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member1 :
  Js.Json.t -> save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member1

val encode_save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member1 :
  save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member1 -> Js.Json.t

val read_save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member1_side :
  Js.Json.t -> save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member1_side

val encode_save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member1_side :
  save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member1_side -> Js.Json.t

val read_literal_observer : Js.Json.t -> literal_observer
val encode_literal_observer : literal_observer -> Js.Json.t

val read_save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member2 :
  Js.Json.t -> save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member2

val encode_save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member2 :
  save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member2 -> Js.Json.t

val read_save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member3 :
  Js.Json.t -> save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member3

val encode_save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member3 :
  save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member3 -> Js.Json.t

val read_save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member4 :
  Js.Json.t -> save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member4

val encode_save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member4 :
  save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member4 -> Js.Json.t

val read_save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member9 :
  Js.Json.t -> save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member9

val encode_save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member9 :
  save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member9 -> Js.Json.t

val read_save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member9_conditions :
  Js.Json.t -> save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member9_conditions

val encode_save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member9_conditions :
  save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member9_conditions -> Js.Json.t

val read_save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member10 :
  Js.Json.t -> save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member10

val encode_save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member10 :
  save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member10 -> Js.Json.t

val read_save40_member0_checkpoint_state_tiles : Js.Json.t -> save40_member0_checkpoint_state_tiles

val encode_save40_member0_checkpoint_state_tiles :
  save40_member0_checkpoint_state_tiles -> Js.Json.t

val read_save40_member0_checkpoint_state_tiles_item :
  Js.Json.t -> save40_member0_checkpoint_state_tiles_item

val encode_save40_member0_checkpoint_state_tiles_item :
  save40_member0_checkpoint_state_tiles_item -> Js.Json.t

val read_save40_member0_checkpoint_state_tiles_item_fog :
  Js.Json.t -> save40_member0_checkpoint_state_tiles_item_fog

val encode_save40_member0_checkpoint_state_tiles_item_fog :
  save40_member0_checkpoint_state_tiles_item_fog -> Js.Json.t

val read_save40_member0_checkpoint_state_tiles_item_light :
  Js.Json.t -> save40_member0_checkpoint_state_tiles_item_light

val encode_save40_member0_checkpoint_state_tiles_item_light :
  save40_member0_checkpoint_state_tiles_item_light -> Js.Json.t

val read_save40_member0_checkpoint_state_tiles_item_light_member1 :
  Js.Json.t -> save40_member0_checkpoint_state_tiles_item_light_member1

val encode_save40_member0_checkpoint_state_tiles_item_light_member1 :
  save40_member0_checkpoint_state_tiles_item_light_member1 -> Js.Json.t

val read_save40_member0_checkpoint_state_campaign :
  Js.Json.t -> save40_member0_checkpoint_state_campaign

val encode_save40_member0_checkpoint_state_campaign :
  save40_member0_checkpoint_state_campaign -> Js.Json.t

val read_save40_member0_checkpoint_state_campaign_member1 :
  Js.Json.t -> save40_member0_checkpoint_state_campaign_member1

val encode_save40_member0_checkpoint_state_campaign_member1 :
  save40_member0_checkpoint_state_campaign_member1 -> Js.Json.t

val read_save40_member0_checkpoint_state_campaign_member1_scripts :
  Js.Json.t -> save40_member0_checkpoint_state_campaign_member1_scripts

val encode_save40_member0_checkpoint_state_campaign_member1_scripts :
  save40_member0_checkpoint_state_campaign_member1_scripts -> Js.Json.t

val read_save40_member0_checkpoint_state_campaign_member1_scripts_member1 :
  Js.Json.t -> save40_member0_checkpoint_state_campaign_member1_scripts_member1

val encode_save40_member0_checkpoint_state_campaign_member1_scripts_member1 :
  save40_member0_checkpoint_state_campaign_member1_scripts_member1 -> Js.Json.t

val read_save40_member0_checkpoint_state_campaign_member1_scripts_member1_marks :
  Js.Json.t -> save40_member0_checkpoint_state_campaign_member1_scripts_member1_marks

val encode_save40_member0_checkpoint_state_campaign_member1_scripts_member1_marks :
  save40_member0_checkpoint_state_campaign_member1_scripts_member1_marks -> Js.Json.t

val read_save40_member0_checkpoint_state_campaign_member1_scripts_member1_marks_item :
  Js.Json.t -> save40_member0_checkpoint_state_campaign_member1_scripts_member1_marks_item

val encode_save40_member0_checkpoint_state_campaign_member1_scripts_member1_marks_item :
  save40_member0_checkpoint_state_campaign_member1_scripts_member1_marks_item -> Js.Json.t

val read_save40_member1 : Js.Json.t -> save40_member1
val encode_save40_member1 : save40_member1 -> Js.Json.t

val read_save40_member0_checkpoint_state_factions_item_vision_member1 :
  Js.Json.t -> save40_member0_checkpoint_state_factions_item_vision_member1

val encode_save40_member0_checkpoint_state_factions_item_vision_member1 :
  save40_member0_checkpoint_state_factions_item_vision_member1 -> Js.Json.t

val read_save40_member0_checkpoint_state_factions_item_vision :
  Js.Json.t -> save40_member0_checkpoint_state_factions_item_vision

val encode_save40_member0_checkpoint_state_factions_item_vision :
  save40_member0_checkpoint_state_factions_item_vision -> Js.Json.t

val read_save41_member0_checkpoint : Js.Json.t -> save41_member0_checkpoint
val encode_save41_member0_checkpoint : save41_member0_checkpoint -> Js.Json.t
val read_save41_member0_checkpoint_state : Js.Json.t -> save41_member0_checkpoint_state
val encode_save41_member0_checkpoint_state : save41_member0_checkpoint_state -> Js.Json.t

val read_save41_member0_checkpoint_state_vision_groups_item :
  Js.Json.t -> save41_member0_checkpoint_state_vision_groups_item

val encode_save41_member0_checkpoint_state_vision_groups_item :
  save41_member0_checkpoint_state_vision_groups_item -> Js.Json.t

val read_save41_member0_checkpoint_state_vision_groups_item_sight :
  Js.Json.t -> save41_member0_checkpoint_state_vision_groups_item_sight

val encode_save41_member0_checkpoint_state_vision_groups_item_sight :
  save41_member0_checkpoint_state_vision_groups_item_sight -> Js.Json.t

val read_save41_member0_checkpoint_state_vision_groups_item_sight_range :
  Js.Json.t -> save41_member0_checkpoint_state_vision_groups_item_sight_range

val encode_save41_member0_checkpoint_state_vision_groups_item_sight_range :
  save41_member0_checkpoint_state_vision_groups_item_sight_range -> Js.Json.t

val read_save41_member0_checkpoint_state_vision_groups_item_sight_range_member0 :
  Js.Json.t -> save41_member0_checkpoint_state_vision_groups_item_sight_range_member0

val encode_save41_member0_checkpoint_state_vision_groups_item_sight_range_member0 :
  save41_member0_checkpoint_state_vision_groups_item_sight_range_member0 -> Js.Json.t

val read_literal_inherited : Js.Json.t -> literal_inherited
val encode_literal_inherited : literal_inherited -> Js.Json.t
val read_literal_unlimited : Js.Json.t -> literal_unlimited
val encode_literal_unlimited : literal_unlimited -> Js.Json.t

val read_save41_member0_checkpoint_state_vision_groups_item_sight_penetrates_item :
  Js.Json.t -> save41_member0_checkpoint_state_vision_groups_item_sight_penetrates_item

val encode_save41_member0_checkpoint_state_vision_groups_item_sight_penetrates_item :
  save41_member0_checkpoint_state_vision_groups_item_sight_penetrates_item -> Js.Json.t

val read_literal_fog : Js.Json.t -> literal_fog
val encode_literal_fog : literal_fog -> Js.Json.t
val read_literal_darkness : Js.Json.t -> literal_darkness
val encode_literal_darkness : literal_darkness -> Js.Json.t
val read_literal_forest_camouflage : Js.Json.t -> literal_forest_camouflage
val encode_literal_forest_camouflage : literal_forest_camouflage -> Js.Json.t

val read_save41_member0_checkpoint_state_vision_groups_item_sight_penetrates :
  Js.Json.t -> save41_member0_checkpoint_state_vision_groups_item_sight_penetrates

val encode_save41_member0_checkpoint_state_vision_groups_item_sight_penetrates :
  save41_member0_checkpoint_state_vision_groups_item_sight_penetrates -> Js.Json.t

val read_save41_member0_checkpoint_state_vision_groups :
  Js.Json.t -> save41_member0_checkpoint_state_vision_groups

val encode_save41_member0_checkpoint_state_vision_groups :
  save41_member0_checkpoint_state_vision_groups -> Js.Json.t

val read_save41_member0_checkpoint_state_factions_item :
  Js.Json.t -> save41_member0_checkpoint_state_factions_item

val encode_save41_member0_checkpoint_state_factions_item :
  save41_member0_checkpoint_state_factions_item -> Js.Json.t

val read_save41_member0_setup_first : Js.Json.t -> save41_member0_setup_first
val encode_save41_member0_setup_first : save41_member0_setup_first -> Js.Json.t

val read_save41_member0_checkpoint_state_factions_item_relations_member1_value :
  Js.Json.t -> save41_member0_checkpoint_state_factions_item_relations_member1_value

val encode_save41_member0_checkpoint_state_factions_item_relations_member1_value :
  save41_member0_checkpoint_state_factions_item_relations_member1_value -> Js.Json.t

val read_save41_member0_checkpoint_state_factions_item_relations_member1 :
  Js.Json.t -> save41_member0_checkpoint_state_factions_item_relations_member1

val encode_save41_member0_checkpoint_state_factions_item_relations_member1 :
  save41_member0_checkpoint_state_factions_item_relations_member1 -> Js.Json.t

val read_save41_member0_checkpoint_state_factions_item_relations :
  Js.Json.t -> save41_member0_checkpoint_state_factions_item_relations

val encode_save41_member0_checkpoint_state_factions_item_relations :
  save41_member0_checkpoint_state_factions_item_relations -> Js.Json.t

val read_save41_member0_checkpoint_state_factions_item_transfer_member1 :
  Js.Json.t -> save41_member0_checkpoint_state_factions_item_transfer_member1

val encode_save41_member0_checkpoint_state_factions_item_transfer_member1 :
  save41_member0_checkpoint_state_factions_item_transfer_member1 -> Js.Json.t

val read_save41_member0_checkpoint_state_factions_item_transfer_member1_when_ :
  Js.Json.t -> save41_member0_checkpoint_state_factions_item_transfer_member1_when_

val encode_save41_member0_checkpoint_state_factions_item_transfer_member1_when_ :
  save41_member0_checkpoint_state_factions_item_transfer_member1_when_ -> Js.Json.t

val read_shape_649 : Js.Json.t -> shape_649
val encode_shape_649 : shape_649 -> Js.Json.t

val read_save41_member0_checkpoint_state_factions_item_transfer :
  Js.Json.t -> save41_member0_checkpoint_state_factions_item_transfer

val encode_save41_member0_checkpoint_state_factions_item_transfer :
  save41_member0_checkpoint_state_factions_item_transfer -> Js.Json.t

val read_save41_member0_checkpoint_state_factions :
  Js.Json.t -> save41_member0_checkpoint_state_factions

val encode_save41_member0_checkpoint_state_factions :
  save41_member0_checkpoint_state_factions -> Js.Json.t

val read_save41_member0_checkpoint_state_ground_items_item :
  Js.Json.t -> save41_member0_checkpoint_state_ground_items_item

val encode_save41_member0_checkpoint_state_ground_items_item :
  save41_member0_checkpoint_state_ground_items_item -> Js.Json.t

val read_save41_member0_checkpoint_state_ground_items_item_item :
  Js.Json.t -> save41_member0_checkpoint_state_ground_items_item_item

val encode_save41_member0_checkpoint_state_ground_items_item_item :
  save41_member0_checkpoint_state_ground_items_item_item -> Js.Json.t

val read_save41_member0_checkpoint_state_ground_items_item_item_perception_member1 :
  Js.Json.t -> save41_member0_checkpoint_state_ground_items_item_item_perception_member1

val encode_save41_member0_checkpoint_state_ground_items_item_item_perception_member1 :
  save41_member0_checkpoint_state_ground_items_item_item_perception_member1 -> Js.Json.t

val read_save41_member0_checkpoint_state_units_item_perception_effects_item_concealment_item :
  Js.Json.t -> save41_member0_checkpoint_state_units_item_perception_effects_item_concealment_item

val encode_save41_member0_checkpoint_state_units_item_perception_effects_item_concealment_item :
  save41_member0_checkpoint_state_units_item_perception_effects_item_concealment_item -> Js.Json.t

val read_save41_member0_checkpoint_state_tiles_item_perception_concealment_item :
  Js.Json.t -> save41_member0_checkpoint_state_tiles_item_perception_concealment_item

val encode_save41_member0_checkpoint_state_tiles_item_perception_concealment_item :
  save41_member0_checkpoint_state_tiles_item_perception_concealment_item -> Js.Json.t

val read_save41_member0_checkpoint_state_units_item_perception_effects_item_concealment_item_terrains_member1_item :
  Js.Json.t ->
  save41_member0_checkpoint_state_units_item_perception_effects_item_concealment_item_terrains_member1_item

val encode_save41_member0_checkpoint_state_units_item_perception_effects_item_concealment_item_terrains_member1_item :
  save41_member0_checkpoint_state_units_item_perception_effects_item_concealment_item_terrains_member1_item ->
  Js.Json.t

val read_literal_bridge : Js.Json.t -> literal_bridge
val encode_literal_bridge : literal_bridge -> Js.Json.t
val read_literal_burned_village : Js.Json.t -> literal_burned_village
val encode_literal_burned_village : literal_burned_village -> Js.Json.t
val read_literal_cave : Js.Json.t -> literal_cave
val encode_literal_cave : literal_cave -> Js.Json.t
val read_literal_church : Js.Json.t -> literal_church
val encode_literal_church : literal_church -> Js.Json.t
val read_literal_ferry : Js.Json.t -> literal_ferry
val encode_literal_ferry : literal_ferry -> Js.Json.t
val read_literal_ford : Js.Json.t -> literal_ford
val encode_literal_ford : literal_ford -> Js.Json.t
val read_literal_forest : Js.Json.t -> literal_forest
val encode_literal_forest : literal_forest -> Js.Json.t
val read_literal_fort : Js.Json.t -> literal_fort
val encode_literal_fort : literal_fort -> Js.Json.t
val read_literal_fortification : Js.Json.t -> literal_fortification
val encode_literal_fortification : literal_fortification -> Js.Json.t
val read_literal_graveyard : Js.Json.t -> literal_graveyard
val encode_literal_graveyard : literal_graveyard -> Js.Json.t
val read_literal_grove : Js.Json.t -> literal_grove
val encode_literal_grove : literal_grove -> Js.Json.t
val read_literal_hill : Js.Json.t -> literal_hill
val encode_literal_hill : literal_hill -> Js.Json.t
val read_literal_house : Js.Json.t -> literal_house
val encode_literal_house : literal_house -> Js.Json.t
val read_literal_keep : Js.Json.t -> literal_keep
val encode_literal_keep : literal_keep -> Js.Json.t
val read_literal_landing : Js.Json.t -> literal_landing
val encode_literal_landing : literal_landing -> Js.Json.t
val read_literal_marsh : Js.Json.t -> literal_marsh
val encode_literal_marsh : literal_marsh -> Js.Json.t
val read_literal_monastery : Js.Json.t -> literal_monastery
val encode_literal_monastery : literal_monastery -> Js.Json.t
val read_literal_mountain : Js.Json.t -> literal_mountain
val encode_literal_mountain : literal_mountain -> Js.Json.t
val read_literal_plains : Js.Json.t -> literal_plains
val encode_literal_plains : literal_plains -> Js.Json.t
val read_literal_ritual_spent : Js.Json.t -> literal_ritual_spent
val encode_literal_ritual_spent : literal_ritual_spent -> Js.Json.t
val read_literal_ritual_star : Js.Json.t -> literal_ritual_star
val encode_literal_ritual_star : literal_ritual_star -> Js.Json.t
val read_literal_road : Js.Json.t -> literal_road
val encode_literal_road : literal_road -> Js.Json.t
val read_literal_ruins : Js.Json.t -> literal_ruins
val encode_literal_ruins : literal_ruins -> Js.Json.t
val read_literal_signpost : Js.Json.t -> literal_signpost
val encode_literal_signpost : literal_signpost -> Js.Json.t
val read_literal_village : Js.Json.t -> literal_village
val encode_literal_village : literal_village -> Js.Json.t
val read_literal_water : Js.Json.t -> literal_water
val encode_literal_water : literal_water -> Js.Json.t

val read_save41_member0_checkpoint_state_units_item_perception_effects_item_concealment_item_terrains_member1 :
  Js.Json.t ->
  save41_member0_checkpoint_state_units_item_perception_effects_item_concealment_item_terrains_member1

val encode_save41_member0_checkpoint_state_units_item_perception_effects_item_concealment_item_terrains_member1 :
  save41_member0_checkpoint_state_units_item_perception_effects_item_concealment_item_terrains_member1 ->
  Js.Json.t

val read_save41_member0_checkpoint_state_units_item_perception_effects_item_concealment_item_terrains :
  Js.Json.t ->
  save41_member0_checkpoint_state_units_item_perception_effects_item_concealment_item_terrains

val encode_save41_member0_checkpoint_state_units_item_perception_effects_item_concealment_item_terrains :
  save41_member0_checkpoint_state_units_item_perception_effects_item_concealment_item_terrains ->
  Js.Json.t

val read_save41_member0_checkpoint_state_units_item_perception_effects_item_concealment :
  Js.Json.t -> save41_member0_checkpoint_state_units_item_perception_effects_item_concealment

val encode_save41_member0_checkpoint_state_units_item_perception_effects_item_concealment :
  save41_member0_checkpoint_state_units_item_perception_effects_item_concealment -> Js.Json.t

val read_save41_member0_checkpoint_state_tiles_item_light_member1_emission_member1 :
  Js.Json.t -> save41_member0_checkpoint_state_tiles_item_light_member1_emission_member1

val encode_save41_member0_checkpoint_state_tiles_item_light_member1_emission_member1 :
  save41_member0_checkpoint_state_tiles_item_light_member1_emission_member1 -> Js.Json.t

val read_save41_member0_checkpoint_state_tiles_item_light_member1_emission :
  Js.Json.t -> save41_member0_checkpoint_state_tiles_item_light_member1_emission

val encode_save41_member0_checkpoint_state_tiles_item_light_member1_emission :
  save41_member0_checkpoint_state_tiles_item_light_member1_emission -> Js.Json.t

val read_save41_member0_checkpoint_state_ground_items_item_item_perception :
  Js.Json.t -> save41_member0_checkpoint_state_ground_items_item_item_perception

val encode_save41_member0_checkpoint_state_ground_items_item_item_perception :
  save41_member0_checkpoint_state_ground_items_item_item_perception -> Js.Json.t

val read_save41_member0_checkpoint_state_ground_items_item_item_attacks_item :
  Js.Json.t -> save41_member0_checkpoint_state_ground_items_item_item_attacks_item

val encode_save41_member0_checkpoint_state_ground_items_item_item_attacks_item :
  save41_member0_checkpoint_state_ground_items_item_item_attacks_item -> Js.Json.t

val read_save41_member0_checkpoint_state_ground_items_item_item_attacks :
  Js.Json.t -> save41_member0_checkpoint_state_ground_items_item_item_attacks

val encode_save41_member0_checkpoint_state_ground_items_item_item_attacks :
  save41_member0_checkpoint_state_ground_items_item_item_attacks -> Js.Json.t

val read_save41_member0_checkpoint_state_ground_items :
  Js.Json.t -> save41_member0_checkpoint_state_ground_items

val encode_save41_member0_checkpoint_state_ground_items :
  save41_member0_checkpoint_state_ground_items -> Js.Json.t

val read_shape_693 : Js.Json.t -> shape_693
val encode_shape_693 : shape_693 -> Js.Json.t
val read_literal_player_victory : Js.Json.t -> literal_player_victory
val encode_literal_player_victory : literal_player_victory -> Js.Json.t
val read_literal_player_defeat : Js.Json.t -> literal_player_defeat
val encode_literal_player_defeat : literal_player_defeat -> Js.Json.t

val read_save41_member0_checkpoint_state_winner :
  Js.Json.t -> save41_member0_checkpoint_state_winner

val encode_save41_member0_checkpoint_state_winner :
  save41_member0_checkpoint_state_winner -> Js.Json.t

val read_save41_member0_checkpoint_state_tiles_item :
  Js.Json.t -> save41_member0_checkpoint_state_tiles_item

val encode_save41_member0_checkpoint_state_tiles_item :
  save41_member0_checkpoint_state_tiles_item -> Js.Json.t

val read_save41_member0_checkpoint_state_tiles_item_perception :
  Js.Json.t -> save41_member0_checkpoint_state_tiles_item_perception

val encode_save41_member0_checkpoint_state_tiles_item_perception :
  save41_member0_checkpoint_state_tiles_item_perception -> Js.Json.t

val read_save41_member0_checkpoint_state_tiles_item_perception_sight_member1 :
  Js.Json.t -> save41_member0_checkpoint_state_tiles_item_perception_sight_member1

val encode_save41_member0_checkpoint_state_tiles_item_perception_sight_member1 :
  save41_member0_checkpoint_state_tiles_item_perception_sight_member1 -> Js.Json.t

val read_save41_member0_checkpoint_state_tiles_item_perception_sight :
  Js.Json.t -> save41_member0_checkpoint_state_tiles_item_perception_sight

val encode_save41_member0_checkpoint_state_tiles_item_perception_sight :
  save41_member0_checkpoint_state_tiles_item_perception_sight -> Js.Json.t

val read_save41_member0_checkpoint_state_tiles_item_perception_concealment :
  Js.Json.t -> save41_member0_checkpoint_state_tiles_item_perception_concealment

val encode_save41_member0_checkpoint_state_tiles_item_perception_concealment :
  save41_member0_checkpoint_state_tiles_item_perception_concealment -> Js.Json.t

val read_save41_member0_checkpoint_state_tiles_item_light_member1 :
  Js.Json.t -> save41_member0_checkpoint_state_tiles_item_light_member1

val encode_save41_member0_checkpoint_state_tiles_item_light_member1 :
  save41_member0_checkpoint_state_tiles_item_light_member1 -> Js.Json.t

val read_save41_member0_checkpoint_state_units_item_retreating :
  Js.Json.t -> save41_member0_checkpoint_state_units_item_retreating

val encode_save41_member0_checkpoint_state_units_item_retreating :
  save41_member0_checkpoint_state_units_item_retreating -> Js.Json.t

val read_save41_member0_checkpoint_state_tiles_item_light_member1_when_ :
  Js.Json.t -> save41_member0_checkpoint_state_tiles_item_light_member1_when_

val encode_save41_member0_checkpoint_state_tiles_item_light_member1_when_ :
  save41_member0_checkpoint_state_tiles_item_light_member1_when_ -> Js.Json.t

val read_save41_member0_checkpoint_state_tiles_item_light :
  Js.Json.t -> save41_member0_checkpoint_state_tiles_item_light

val encode_save41_member0_checkpoint_state_tiles_item_light :
  save41_member0_checkpoint_state_tiles_item_light -> Js.Json.t

val read_save41_member0_checkpoint_state_tiles : Js.Json.t -> save41_member0_checkpoint_state_tiles

val encode_save41_member0_checkpoint_state_tiles :
  save41_member0_checkpoint_state_tiles -> Js.Json.t

val read_save41_member0_checkpoint_state_units_item :
  Js.Json.t -> save41_member0_checkpoint_state_units_item

val encode_save41_member0_checkpoint_state_units_item :
  save41_member0_checkpoint_state_units_item -> Js.Json.t

val read_save41_member0_checkpoint_state_units_item_perception_effects_item :
  Js.Json.t -> save41_member0_checkpoint_state_units_item_perception_effects_item

val encode_save41_member0_checkpoint_state_units_item_perception_effects_item :
  save41_member0_checkpoint_state_units_item_perception_effects_item -> Js.Json.t

val read_save41_member0_checkpoint_state_units_item_perception_effects_item_expires_at_member1 :
  Js.Json.t -> save41_member0_checkpoint_state_units_item_perception_effects_item_expires_at_member1

val encode_save41_member0_checkpoint_state_units_item_perception_effects_item_expires_at_member1 :
  save41_member0_checkpoint_state_units_item_perception_effects_item_expires_at_member1 -> Js.Json.t

val read_save41_member0_checkpoint_state_units_item_perception_effects_item_expires_at :
  Js.Json.t -> save41_member0_checkpoint_state_units_item_perception_effects_item_expires_at

val encode_save41_member0_checkpoint_state_units_item_perception_effects_item_expires_at :
  save41_member0_checkpoint_state_units_item_perception_effects_item_expires_at -> Js.Json.t

val read_save41_member0_checkpoint_state_units_item_perception_effects :
  Js.Json.t -> save41_member0_checkpoint_state_units_item_perception_effects

val encode_save41_member0_checkpoint_state_units_item_perception_effects :
  save41_member0_checkpoint_state_units_item_perception_effects -> Js.Json.t

val read_save41_member0_checkpoint_state_units_item_items_member1 :
  Js.Json.t -> save41_member0_checkpoint_state_units_item_items_member1

val encode_save41_member0_checkpoint_state_units_item_items_member1 :
  save41_member0_checkpoint_state_units_item_items_member1 -> Js.Json.t

val read_save41_member0_checkpoint_state_units_item_items :
  Js.Json.t -> save41_member0_checkpoint_state_units_item_items

val encode_save41_member0_checkpoint_state_units_item_items :
  save41_member0_checkpoint_state_units_item_items -> Js.Json.t

val read_shape_714 : Js.Json.t -> shape_714
val encode_shape_714 : shape_714 -> Js.Json.t
val read_save41_member0_checkpoint_state_units : Js.Json.t -> save41_member0_checkpoint_state_units

val encode_save41_member0_checkpoint_state_units :
  save41_member0_checkpoint_state_units -> Js.Json.t

val read_save41_member0_checkpoint_state_campaign_member1 :
  Js.Json.t -> save41_member0_checkpoint_state_campaign_member1

val encode_save41_member0_checkpoint_state_campaign_member1 :
  save41_member0_checkpoint_state_campaign_member1 -> Js.Json.t

val read_save41_member0_checkpoint_state_campaign_member1_scripts_member1 :
  Js.Json.t -> save41_member0_checkpoint_state_campaign_member1_scripts_member1

val encode_save41_member0_checkpoint_state_campaign_member1_scripts_member1 :
  save41_member0_checkpoint_state_campaign_member1_scripts_member1 -> Js.Json.t

val read_save41_member0_checkpoint_state_campaign_member1_scripts_member1_marks_item :
  Js.Json.t -> save41_member0_checkpoint_state_campaign_member1_scripts_member1_marks_item

val encode_save41_member0_checkpoint_state_campaign_member1_scripts_member1_marks_item :
  save41_member0_checkpoint_state_campaign_member1_scripts_member1_marks_item -> Js.Json.t

val read_save41_member0_checkpoint_state_campaign_member1_scripts_member1_marks :
  Js.Json.t -> save41_member0_checkpoint_state_campaign_member1_scripts_member1_marks

val encode_save41_member0_checkpoint_state_campaign_member1_scripts_member1_marks :
  save41_member0_checkpoint_state_campaign_member1_scripts_member1_marks -> Js.Json.t

val read_save41_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item :
  Js.Json.t -> save41_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item

val encode_save41_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item :
  save41_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item -> Js.Json.t

val read_save41_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item_units_item :
  Js.Json.t ->
  save41_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item_units_item

val encode_save41_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item_units_item :
  save41_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item_units_item ->
  Js.Json.t

val read_save41_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item_units_item_role :
  Js.Json.t ->
  save41_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item_units_item_role

val encode_save41_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item_units_item_role :
  save41_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item_units_item_role ->
  Js.Json.t

val read_save41_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item_units :
  Js.Json.t -> save41_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item_units

val encode_save41_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item_units :
  save41_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item_units -> Js.Json.t

val read_save41_member0_checkpoint_state_campaign_member1_scripts_member1_pending :
  Js.Json.t -> save41_member0_checkpoint_state_campaign_member1_scripts_member1_pending

val encode_save41_member0_checkpoint_state_campaign_member1_scripts_member1_pending :
  save41_member0_checkpoint_state_campaign_member1_scripts_member1_pending -> Js.Json.t

val read_save41_member0_checkpoint_state_campaign_member1_scripts :
  Js.Json.t -> save41_member0_checkpoint_state_campaign_member1_scripts

val encode_save41_member0_checkpoint_state_campaign_member1_scripts :
  save41_member0_checkpoint_state_campaign_member1_scripts -> Js.Json.t

val read_save41_member0_checkpoint_state_campaign_member1_evacuated_member1_item :
  Js.Json.t -> save41_member0_checkpoint_state_campaign_member1_evacuated_member1_item

val encode_save41_member0_checkpoint_state_campaign_member1_evacuated_member1_item :
  save41_member0_checkpoint_state_campaign_member1_evacuated_member1_item -> Js.Json.t

val read_save41_member0_checkpoint_state_campaign_member1_evacuated_member1 :
  Js.Json.t -> save41_member0_checkpoint_state_campaign_member1_evacuated_member1

val encode_save41_member0_checkpoint_state_campaign_member1_evacuated_member1 :
  save41_member0_checkpoint_state_campaign_member1_evacuated_member1 -> Js.Json.t

val read_save41_member0_checkpoint_state_campaign_member1_evacuated :
  Js.Json.t -> save41_member0_checkpoint_state_campaign_member1_evacuated

val encode_save41_member0_checkpoint_state_campaign_member1_evacuated :
  save41_member0_checkpoint_state_campaign_member1_evacuated -> Js.Json.t

val read_save41_member0_checkpoint_state_campaign_member1_veterans_member1_item :
  Js.Json.t -> save41_member0_checkpoint_state_campaign_member1_veterans_member1_item

val encode_save41_member0_checkpoint_state_campaign_member1_veterans_member1_item :
  save41_member0_checkpoint_state_campaign_member1_veterans_member1_item -> Js.Json.t

val read_save41_member0_checkpoint_state_campaign_member1_veterans_member1 :
  Js.Json.t -> save41_member0_checkpoint_state_campaign_member1_veterans_member1

val encode_save41_member0_checkpoint_state_campaign_member1_veterans_member1 :
  save41_member0_checkpoint_state_campaign_member1_veterans_member1 -> Js.Json.t

val read_save41_member0_checkpoint_state_campaign_member1_veterans :
  Js.Json.t -> save41_member0_checkpoint_state_campaign_member1_veterans

val encode_save41_member0_checkpoint_state_campaign_member1_veterans :
  save41_member0_checkpoint_state_campaign_member1_veterans -> Js.Json.t

val read_save41_member0_checkpoint_state_campaign_member1_dialogue_item :
  Js.Json.t -> save41_member0_checkpoint_state_campaign_member1_dialogue_item

val encode_save41_member0_checkpoint_state_campaign_member1_dialogue_item :
  save41_member0_checkpoint_state_campaign_member1_dialogue_item -> Js.Json.t

val read_save41_member0_checkpoint_state_campaign_member1_dialogue :
  Js.Json.t -> save41_member0_checkpoint_state_campaign_member1_dialogue

val encode_save41_member0_checkpoint_state_campaign_member1_dialogue :
  save41_member0_checkpoint_state_campaign_member1_dialogue -> Js.Json.t

val read_save41_member0_checkpoint_state_campaign :
  Js.Json.t -> save41_member0_checkpoint_state_campaign

val encode_save41_member0_checkpoint_state_campaign :
  save41_member0_checkpoint_state_campaign -> Js.Json.t

val read_shape_735 : Js.Json.t -> shape_735
val encode_shape_735 : shape_735 -> Js.Json.t
val read_save41 : Js.Json.t -> save41
val encode_save41 : save41 -> Js.Json.t
val read_save41_member0 : Js.Json.t -> save41_member0
val encode_save41_member0 : save41_member0 -> Js.Json.t
val read_literal_41 : Js.Json.t -> literal_41
val encode_literal_41 : literal_41 -> Js.Json.t
val read_save41_member0_setup : Js.Json.t -> save41_member0_setup
val encode_save41_member0_setup : save41_member0_setup -> Js.Json.t
val read_save41_member1 : Js.Json.t -> save41_member1
val encode_save41_member1 : save41_member1 -> Js.Json.t
val read_save42_member0_setup_player : Js.Json.t -> save42_member0_setup_player
val encode_save42_member0_setup_player : save42_member0_setup_player -> Js.Json.t
val read_save42_member0_setup : Js.Json.t -> save42_member0_setup
val encode_save42_member0_setup : save42_member0_setup -> Js.Json.t
val read_literal_42 : Js.Json.t -> literal_42
val encode_literal_42 : literal_42 -> Js.Json.t
val read_save42_member0 : Js.Json.t -> save42_member0
val encode_save42_member0 : save42_member0 -> Js.Json.t
val read_save42_member1 : Js.Json.t -> save42_member1
val encode_save42_member1 : save42_member1 -> Js.Json.t
val read_save42 : Js.Json.t -> save42
val encode_save42 : save42 -> Js.Json.t
val read_save43 : Js.Json.t -> save43
val encode_save43 : save43 -> Js.Json.t
val read_save43_member0 : Js.Json.t -> save43_member0
val encode_save43_member0 : save43_member0 -> Js.Json.t
val read_literal_43 : Js.Json.t -> literal_43
val encode_literal_43 : literal_43 -> Js.Json.t
val read_save43_member0_checkpoint : Js.Json.t -> save43_member0_checkpoint
val encode_save43_member0_checkpoint : save43_member0_checkpoint -> Js.Json.t
val read_save43_member0_checkpoint_state : Js.Json.t -> save43_member0_checkpoint_state
val encode_save43_member0_checkpoint_state : save43_member0_checkpoint_state -> Js.Json.t

val read_save43_member0_checkpoint_state_ground_items :
  Js.Json.t -> save43_member0_checkpoint_state_ground_items

val encode_save43_member0_checkpoint_state_ground_items :
  save43_member0_checkpoint_state_ground_items -> Js.Json.t

val read_save43_member0_checkpoint_state_ground_items_item :
  Js.Json.t -> save43_member0_checkpoint_state_ground_items_item

val encode_save43_member0_checkpoint_state_ground_items_item :
  save43_member0_checkpoint_state_ground_items_item -> Js.Json.t

val read_save43_member0_checkpoint_state_ground_items_item_item :
  Js.Json.t -> save43_member0_checkpoint_state_ground_items_item_item

val encode_save43_member0_checkpoint_state_ground_items_item_item :
  save43_member0_checkpoint_state_ground_items_item_item -> Js.Json.t

val read_save43_member0_checkpoint_state_ground_items_item_item_perception :
  Js.Json.t -> save43_member0_checkpoint_state_ground_items_item_item_perception

val encode_save43_member0_checkpoint_state_ground_items_item_item_perception :
  save43_member0_checkpoint_state_ground_items_item_item_perception -> Js.Json.t

val read_save43_member0_checkpoint_state_ground_items_item_item_perception_member1 :
  Js.Json.t -> save43_member0_checkpoint_state_ground_items_item_item_perception_member1

val encode_save43_member0_checkpoint_state_ground_items_item_item_perception_member1 :
  save43_member0_checkpoint_state_ground_items_item_item_perception_member1 -> Js.Json.t

val read_save43_member0_checkpoint_state_units_item_perception_effects_item_concealment :
  Js.Json.t -> save43_member0_checkpoint_state_units_item_perception_effects_item_concealment

val encode_save43_member0_checkpoint_state_units_item_perception_effects_item_concealment :
  save43_member0_checkpoint_state_units_item_perception_effects_item_concealment -> Js.Json.t

val read_save43_member0_checkpoint_state_units_item_perception_effects_item_concealment_item :
  Js.Json.t -> save43_member0_checkpoint_state_units_item_perception_effects_item_concealment_item

val encode_save43_member0_checkpoint_state_units_item_perception_effects_item_concealment_item :
  save43_member0_checkpoint_state_units_item_perception_effects_item_concealment_item -> Js.Json.t

val read_save43_member0_checkpoint_state_units_item_perception_effects_item_concealment_item_terrains :
  Js.Json.t ->
  save43_member0_checkpoint_state_units_item_perception_effects_item_concealment_item_terrains

val encode_save43_member0_checkpoint_state_units_item_perception_effects_item_concealment_item_terrains :
  save43_member0_checkpoint_state_units_item_perception_effects_item_concealment_item_terrains ->
  Js.Json.t

val read_save43_member0_checkpoint_state_units_item_perception_effects_item_concealment_item_terrains_member1 :
  Js.Json.t ->
  save43_member0_checkpoint_state_units_item_perception_effects_item_concealment_item_terrains_member1

val encode_save43_member0_checkpoint_state_units_item_perception_effects_item_concealment_item_terrains_member1 :
  save43_member0_checkpoint_state_units_item_perception_effects_item_concealment_item_terrains_member1 ->
  Js.Json.t

val read_save43_member0_checkpoint_state_units_item_perception_effects_item_concealment_item_terrains_member1_item :
  Js.Json.t ->
  save43_member0_checkpoint_state_units_item_perception_effects_item_concealment_item_terrains_member1_item

val encode_save43_member0_checkpoint_state_units_item_perception_effects_item_concealment_item_terrains_member1_item :
  save43_member0_checkpoint_state_units_item_perception_effects_item_concealment_item_terrains_member1_item ->
  Js.Json.t

val read_literal_cave_wall : Js.Json.t -> literal_cave_wall
val encode_literal_cave_wall : literal_cave_wall -> Js.Json.t
val read_save43_member0_checkpoint_state_units : Js.Json.t -> save43_member0_checkpoint_state_units

val encode_save43_member0_checkpoint_state_units :
  save43_member0_checkpoint_state_units -> Js.Json.t

val read_save43_member0_checkpoint_state_units_item :
  Js.Json.t -> save43_member0_checkpoint_state_units_item

val encode_save43_member0_checkpoint_state_units_item :
  save43_member0_checkpoint_state_units_item -> Js.Json.t

val read_save43_member0_checkpoint_state_units_item_perception_effects :
  Js.Json.t -> save43_member0_checkpoint_state_units_item_perception_effects

val encode_save43_member0_checkpoint_state_units_item_perception_effects :
  save43_member0_checkpoint_state_units_item_perception_effects -> Js.Json.t

val read_save43_member0_checkpoint_state_units_item_perception_effects_item :
  Js.Json.t -> save43_member0_checkpoint_state_units_item_perception_effects_item

val encode_save43_member0_checkpoint_state_units_item_perception_effects_item :
  save43_member0_checkpoint_state_units_item_perception_effects_item -> Js.Json.t

val read_save43_member0_checkpoint_state_units_item_items :
  Js.Json.t -> save43_member0_checkpoint_state_units_item_items

val encode_save43_member0_checkpoint_state_units_item_items :
  save43_member0_checkpoint_state_units_item_items -> Js.Json.t

val read_save43_member0_checkpoint_state_units_item_items_member1 :
  Js.Json.t -> save43_member0_checkpoint_state_units_item_items_member1

val encode_save43_member0_checkpoint_state_units_item_items_member1 :
  save43_member0_checkpoint_state_units_item_items_member1 -> Js.Json.t

val read_save43_member0_checkpoint_state_campaign :
  Js.Json.t -> save43_member0_checkpoint_state_campaign

val encode_save43_member0_checkpoint_state_campaign :
  save43_member0_checkpoint_state_campaign -> Js.Json.t

val read_save43_member0_checkpoint_state_campaign_member1 :
  Js.Json.t -> save43_member0_checkpoint_state_campaign_member1

val encode_save43_member0_checkpoint_state_campaign_member1 :
  save43_member0_checkpoint_state_campaign_member1 -> Js.Json.t

val read_save43_member0_checkpoint_state_campaign_member1_evacuated :
  Js.Json.t -> save43_member0_checkpoint_state_campaign_member1_evacuated

val encode_save43_member0_checkpoint_state_campaign_member1_evacuated :
  save43_member0_checkpoint_state_campaign_member1_evacuated -> Js.Json.t

val read_save43_member0_checkpoint_state_campaign_member1_evacuated_member1 :
  Js.Json.t -> save43_member0_checkpoint_state_campaign_member1_evacuated_member1

val encode_save43_member0_checkpoint_state_campaign_member1_evacuated_member1 :
  save43_member0_checkpoint_state_campaign_member1_evacuated_member1 -> Js.Json.t

val read_save43_member0_checkpoint_state_campaign_member1_evacuated_member1_item :
  Js.Json.t -> save43_member0_checkpoint_state_campaign_member1_evacuated_member1_item

val encode_save43_member0_checkpoint_state_campaign_member1_evacuated_member1_item :
  save43_member0_checkpoint_state_campaign_member1_evacuated_member1_item -> Js.Json.t

val read_save43_member0_checkpoint_state_campaign_member1_veterans :
  Js.Json.t -> save43_member0_checkpoint_state_campaign_member1_veterans

val encode_save43_member0_checkpoint_state_campaign_member1_veterans :
  save43_member0_checkpoint_state_campaign_member1_veterans -> Js.Json.t

val read_save43_member0_checkpoint_state_campaign_member1_veterans_member1 :
  Js.Json.t -> save43_member0_checkpoint_state_campaign_member1_veterans_member1

val encode_save43_member0_checkpoint_state_campaign_member1_veterans_member1 :
  save43_member0_checkpoint_state_campaign_member1_veterans_member1 -> Js.Json.t

val read_save43_member0_checkpoint_state_campaign_member1_veterans_member1_item :
  Js.Json.t -> save43_member0_checkpoint_state_campaign_member1_veterans_member1_item

val encode_save43_member0_checkpoint_state_campaign_member1_veterans_member1_item :
  save43_member0_checkpoint_state_campaign_member1_veterans_member1_item -> Js.Json.t

val read_save43_member1 : Js.Json.t -> save43_member1
val encode_save43_member1 : save43_member1 -> Js.Json.t
val read_save44_member0_checkpoint : Js.Json.t -> save44_member0_checkpoint
val encode_save44_member0_checkpoint : save44_member0_checkpoint -> Js.Json.t
val read_save44_member0_checkpoint_state : Js.Json.t -> save44_member0_checkpoint_state
val encode_save44_member0_checkpoint_state : save44_member0_checkpoint_state -> Js.Json.t

val read_save44_member0_checkpoint_state_ground_items_item :
  Js.Json.t -> save44_member0_checkpoint_state_ground_items_item

val encode_save44_member0_checkpoint_state_ground_items_item :
  save44_member0_checkpoint_state_ground_items_item -> Js.Json.t

val read_save44_member0_checkpoint_state_ground_items_item_item :
  Js.Json.t -> save44_member0_checkpoint_state_ground_items_item_item

val encode_save44_member0_checkpoint_state_ground_items_item_item :
  save44_member0_checkpoint_state_ground_items_item_item -> Js.Json.t

val read_save44_member0_checkpoint_state_ground_items_item_item_perception_member1 :
  Js.Json.t -> save44_member0_checkpoint_state_ground_items_item_item_perception_member1

val encode_save44_member0_checkpoint_state_ground_items_item_item_perception_member1 :
  save44_member0_checkpoint_state_ground_items_item_item_perception_member1 -> Js.Json.t

val read_save44_member0_checkpoint_state_tiles_item_perception_concealment_item :
  Js.Json.t -> save44_member0_checkpoint_state_tiles_item_perception_concealment_item

val encode_save44_member0_checkpoint_state_tiles_item_perception_concealment_item :
  save44_member0_checkpoint_state_tiles_item_perception_concealment_item -> Js.Json.t

val read_save44_member0_checkpoint_state_tiles_item_perception_concealment_item_requires :
  Js.Json.t -> save44_member0_checkpoint_state_tiles_item_perception_concealment_item_requires

val encode_save44_member0_checkpoint_state_tiles_item_perception_concealment_item_requires :
  save44_member0_checkpoint_state_tiles_item_perception_concealment_item_requires -> Js.Json.t

val read_save44_member0_checkpoint_state_tiles_item_perception_concealment_item_requires_terrains_member1_item :
  Js.Json.t ->
  save44_member0_checkpoint_state_tiles_item_perception_concealment_item_requires_terrains_member1_item

val encode_save44_member0_checkpoint_state_tiles_item_perception_concealment_item_requires_terrains_member1_item :
  save44_member0_checkpoint_state_tiles_item_perception_concealment_item_requires_terrains_member1_item ->
  Js.Json.t

val read_save44_member0_checkpoint_state_tiles_item_perception_concealment_item_requires_terrains_member1 :
  Js.Json.t ->
  save44_member0_checkpoint_state_tiles_item_perception_concealment_item_requires_terrains_member1

val encode_save44_member0_checkpoint_state_tiles_item_perception_concealment_item_requires_terrains_member1 :
  save44_member0_checkpoint_state_tiles_item_perception_concealment_item_requires_terrains_member1 ->
  Js.Json.t

val read_save44_member0_checkpoint_state_tiles_item_perception_concealment_item_requires_terrains :
  Js.Json.t ->
  save44_member0_checkpoint_state_tiles_item_perception_concealment_item_requires_terrains

val encode_save44_member0_checkpoint_state_tiles_item_perception_concealment_item_requires_terrains :
  save44_member0_checkpoint_state_tiles_item_perception_concealment_item_requires_terrains ->
  Js.Json.t

val read_shape_788 : Js.Json.t -> shape_788
val encode_shape_788 : shape_788 -> Js.Json.t
val read_literal_lit : Js.Json.t -> literal_lit
val encode_literal_lit : literal_lit -> Js.Json.t
val read_literal_unlit : Js.Json.t -> literal_unlit
val encode_literal_unlit : literal_unlit -> Js.Json.t

val read_save44_member0_checkpoint_state_tiles_item_perception_concealment_item_requires_illumination :
  Js.Json.t ->
  save44_member0_checkpoint_state_tiles_item_perception_concealment_item_requires_illumination

val encode_save44_member0_checkpoint_state_tiles_item_perception_concealment_item_requires_illumination :
  save44_member0_checkpoint_state_tiles_item_perception_concealment_item_requires_illumination ->
  Js.Json.t

val read_save44_member0_checkpoint_state_tiles_item_perception_concealment :
  Js.Json.t -> save44_member0_checkpoint_state_tiles_item_perception_concealment

val encode_save44_member0_checkpoint_state_tiles_item_perception_concealment :
  save44_member0_checkpoint_state_tiles_item_perception_concealment -> Js.Json.t

val read_save44_member0_checkpoint_state_ground_items_item_item_perception :
  Js.Json.t -> save44_member0_checkpoint_state_ground_items_item_item_perception

val encode_save44_member0_checkpoint_state_ground_items_item_item_perception :
  save44_member0_checkpoint_state_ground_items_item_item_perception -> Js.Json.t

val read_save44_member0_checkpoint_state_ground_items :
  Js.Json.t -> save44_member0_checkpoint_state_ground_items

val encode_save44_member0_checkpoint_state_ground_items :
  save44_member0_checkpoint_state_ground_items -> Js.Json.t

val read_save44_member0_checkpoint_state_tiles_item :
  Js.Json.t -> save44_member0_checkpoint_state_tiles_item

val encode_save44_member0_checkpoint_state_tiles_item :
  save44_member0_checkpoint_state_tiles_item -> Js.Json.t

val read_save44_member0_checkpoint_state_tiles_item_perception :
  Js.Json.t -> save44_member0_checkpoint_state_tiles_item_perception

val encode_save44_member0_checkpoint_state_tiles_item_perception :
  save44_member0_checkpoint_state_tiles_item_perception -> Js.Json.t

val read_save44_member0_checkpoint_state_tiles : Js.Json.t -> save44_member0_checkpoint_state_tiles

val encode_save44_member0_checkpoint_state_tiles :
  save44_member0_checkpoint_state_tiles -> Js.Json.t

val read_save44_member0_checkpoint_state_units_item :
  Js.Json.t -> save44_member0_checkpoint_state_units_item

val encode_save44_member0_checkpoint_state_units_item :
  save44_member0_checkpoint_state_units_item -> Js.Json.t

val read_save44_member0_checkpoint_state_units_item_perception_effects_item :
  Js.Json.t -> save44_member0_checkpoint_state_units_item_perception_effects_item

val encode_save44_member0_checkpoint_state_units_item_perception_effects_item :
  save44_member0_checkpoint_state_units_item_perception_effects_item -> Js.Json.t

val read_save44_member0_checkpoint_state_units_item_perception_effects :
  Js.Json.t -> save44_member0_checkpoint_state_units_item_perception_effects

val encode_save44_member0_checkpoint_state_units_item_perception_effects :
  save44_member0_checkpoint_state_units_item_perception_effects -> Js.Json.t

val read_save44_member0_checkpoint_state_units_item_items_member1 :
  Js.Json.t -> save44_member0_checkpoint_state_units_item_items_member1

val encode_save44_member0_checkpoint_state_units_item_items_member1 :
  save44_member0_checkpoint_state_units_item_items_member1 -> Js.Json.t

val read_save44_member0_checkpoint_state_units_item_items :
  Js.Json.t -> save44_member0_checkpoint_state_units_item_items

val encode_save44_member0_checkpoint_state_units_item_items :
  save44_member0_checkpoint_state_units_item_items -> Js.Json.t

val read_save44_member0_checkpoint_state_units : Js.Json.t -> save44_member0_checkpoint_state_units

val encode_save44_member0_checkpoint_state_units :
  save44_member0_checkpoint_state_units -> Js.Json.t

val read_save44_member0_checkpoint_state_campaign_member1 :
  Js.Json.t -> save44_member0_checkpoint_state_campaign_member1

val encode_save44_member0_checkpoint_state_campaign_member1 :
  save44_member0_checkpoint_state_campaign_member1 -> Js.Json.t

val read_save44_member0_checkpoint_state_campaign_member1_evacuated_member1_item :
  Js.Json.t -> save44_member0_checkpoint_state_campaign_member1_evacuated_member1_item

val encode_save44_member0_checkpoint_state_campaign_member1_evacuated_member1_item :
  save44_member0_checkpoint_state_campaign_member1_evacuated_member1_item -> Js.Json.t

val read_save44_member0_checkpoint_state_campaign_member1_evacuated_member1 :
  Js.Json.t -> save44_member0_checkpoint_state_campaign_member1_evacuated_member1

val encode_save44_member0_checkpoint_state_campaign_member1_evacuated_member1 :
  save44_member0_checkpoint_state_campaign_member1_evacuated_member1 -> Js.Json.t

val read_save44_member0_checkpoint_state_campaign_member1_evacuated :
  Js.Json.t -> save44_member0_checkpoint_state_campaign_member1_evacuated

val encode_save44_member0_checkpoint_state_campaign_member1_evacuated :
  save44_member0_checkpoint_state_campaign_member1_evacuated -> Js.Json.t

val read_save44_member0_checkpoint_state_campaign_member1_veterans_member1_item :
  Js.Json.t -> save44_member0_checkpoint_state_campaign_member1_veterans_member1_item

val encode_save44_member0_checkpoint_state_campaign_member1_veterans_member1_item :
  save44_member0_checkpoint_state_campaign_member1_veterans_member1_item -> Js.Json.t

val read_save44_member0_checkpoint_state_campaign_member1_veterans_member1 :
  Js.Json.t -> save44_member0_checkpoint_state_campaign_member1_veterans_member1

val encode_save44_member0_checkpoint_state_campaign_member1_veterans_member1 :
  save44_member0_checkpoint_state_campaign_member1_veterans_member1 -> Js.Json.t

val read_save44_member0_checkpoint_state_campaign_member1_veterans :
  Js.Json.t -> save44_member0_checkpoint_state_campaign_member1_veterans

val encode_save44_member0_checkpoint_state_campaign_member1_veterans :
  save44_member0_checkpoint_state_campaign_member1_veterans -> Js.Json.t

val read_save44_member0_checkpoint_state_campaign :
  Js.Json.t -> save44_member0_checkpoint_state_campaign

val encode_save44_member0_checkpoint_state_campaign :
  save44_member0_checkpoint_state_campaign -> Js.Json.t

val read_save44 : Js.Json.t -> save44
val encode_save44 : save44 -> Js.Json.t
val read_save44_member0 : Js.Json.t -> save44_member0
val encode_save44_member0 : save44_member0 -> Js.Json.t
val read_literal_44 : Js.Json.t -> literal_44
val encode_literal_44 : literal_44 -> Js.Json.t
val read_save44_member1 : Js.Json.t -> save44_member1
val encode_save44_member1 : save44_member1 -> Js.Json.t
val read_literal_45 : Js.Json.t -> literal_45
val encode_literal_45 : literal_45 -> Js.Json.t
val read_save45_member0 : Js.Json.t -> save45_member0
val encode_save45_member0 : save45_member0 -> Js.Json.t
val read_save45_member1 : Js.Json.t -> save45_member1
val encode_save45_member1 : save45_member1 -> Js.Json.t
val read_save45 : Js.Json.t -> save45
val encode_save45 : save45 -> Js.Json.t
val read_save46 : Js.Json.t -> save46
val encode_save46 : save46 -> Js.Json.t
val read_save46_member0 : Js.Json.t -> save46_member0
val encode_save46_member0 : save46_member0 -> Js.Json.t
val read_literal_46 : Js.Json.t -> literal_46
val encode_literal_46 : literal_46 -> Js.Json.t
val read_save46_member0_checkpoint : Js.Json.t -> save46_member0_checkpoint
val encode_save46_member0_checkpoint : save46_member0_checkpoint -> Js.Json.t
val read_save46_member0_checkpoint_state : Js.Json.t -> save46_member0_checkpoint_state
val encode_save46_member0_checkpoint_state : save46_member0_checkpoint_state -> Js.Json.t
val read_save46_member0_checkpoint_state_units : Js.Json.t -> save46_member0_checkpoint_state_units

val encode_save46_member0_checkpoint_state_units :
  save46_member0_checkpoint_state_units -> Js.Json.t

val read_save46_member0_checkpoint_state_units_item :
  Js.Json.t -> save46_member0_checkpoint_state_units_item

val encode_save46_member0_checkpoint_state_units_item :
  save46_member0_checkpoint_state_units_item -> Js.Json.t

val read_save46_member0_checkpoint_state_campaign :
  Js.Json.t -> save46_member0_checkpoint_state_campaign

val encode_save46_member0_checkpoint_state_campaign :
  save46_member0_checkpoint_state_campaign -> Js.Json.t

val read_save46_member0_checkpoint_state_campaign_member1 :
  Js.Json.t -> save46_member0_checkpoint_state_campaign_member1

val encode_save46_member0_checkpoint_state_campaign_member1 :
  save46_member0_checkpoint_state_campaign_member1 -> Js.Json.t

val read_save46_member0_checkpoint_state_campaign_member1_evacuated :
  Js.Json.t -> save46_member0_checkpoint_state_campaign_member1_evacuated

val encode_save46_member0_checkpoint_state_campaign_member1_evacuated :
  save46_member0_checkpoint_state_campaign_member1_evacuated -> Js.Json.t

val read_save46_member0_checkpoint_state_campaign_member1_evacuated_member1 :
  Js.Json.t -> save46_member0_checkpoint_state_campaign_member1_evacuated_member1

val encode_save46_member0_checkpoint_state_campaign_member1_evacuated_member1 :
  save46_member0_checkpoint_state_campaign_member1_evacuated_member1 -> Js.Json.t

val read_save46_member0_checkpoint_state_campaign_member1_evacuated_member1_item :
  Js.Json.t -> save46_member0_checkpoint_state_campaign_member1_evacuated_member1_item

val encode_save46_member0_checkpoint_state_campaign_member1_evacuated_member1_item :
  save46_member0_checkpoint_state_campaign_member1_evacuated_member1_item -> Js.Json.t

val read_save46_member1 : Js.Json.t -> save46_member1
val encode_save46_member1 : save46_member1 -> Js.Json.t
val read_save47 : Js.Json.t -> save47
val encode_save47 : save47 -> Js.Json.t
val read_save47_member0 : Js.Json.t -> save47_member0
val encode_save47_member0 : save47_member0 -> Js.Json.t
val read_literal_47 : Js.Json.t -> literal_47
val encode_literal_47 : literal_47 -> Js.Json.t
val read_save47_member0_checkpoint : Js.Json.t -> save47_member0_checkpoint
val encode_save47_member0_checkpoint : save47_member0_checkpoint -> Js.Json.t
val read_save47_member0_checkpoint_state : Js.Json.t -> save47_member0_checkpoint_state
val encode_save47_member0_checkpoint_state : save47_member0_checkpoint_state -> Js.Json.t

val read_save47_member0_checkpoint_state_factions :
  Js.Json.t -> save47_member0_checkpoint_state_factions

val encode_save47_member0_checkpoint_state_factions :
  save47_member0_checkpoint_state_factions -> Js.Json.t

val read_save47_member0_checkpoint_state_factions_item :
  Js.Json.t -> save47_member0_checkpoint_state_factions_item

val encode_save47_member0_checkpoint_state_factions_item :
  save47_member0_checkpoint_state_factions_item -> Js.Json.t

val read_save47_member0_checkpoint_state_factions_item_transfer :
  Js.Json.t -> save47_member0_checkpoint_state_factions_item_transfer

val encode_save47_member0_checkpoint_state_factions_item_transfer :
  save47_member0_checkpoint_state_factions_item_transfer -> Js.Json.t

val read_save47_member0_checkpoint_state_factions_item_transfer_member1 :
  Js.Json.t -> save47_member0_checkpoint_state_factions_item_transfer_member1

val encode_save47_member0_checkpoint_state_factions_item_transfer_member1 :
  save47_member0_checkpoint_state_factions_item_transfer_member1 -> Js.Json.t

val read_save47_member0_checkpoint_state_factions_item_transfer_member1_when_ :
  Js.Json.t -> save47_member0_checkpoint_state_factions_item_transfer_member1_when_

val encode_save47_member0_checkpoint_state_factions_item_transfer_member1_when_ :
  save47_member0_checkpoint_state_factions_item_transfer_member1_when_ -> Js.Json.t

val read_save47_member0_checkpoint_state_tiles : Js.Json.t -> save47_member0_checkpoint_state_tiles

val encode_save47_member0_checkpoint_state_tiles :
  save47_member0_checkpoint_state_tiles -> Js.Json.t

val read_save47_member0_checkpoint_state_tiles_item :
  Js.Json.t -> save47_member0_checkpoint_state_tiles_item

val encode_save47_member0_checkpoint_state_tiles_item :
  save47_member0_checkpoint_state_tiles_item -> Js.Json.t

val read_save47_member0_checkpoint_state_tiles_item_light :
  Js.Json.t -> save47_member0_checkpoint_state_tiles_item_light

val encode_save47_member0_checkpoint_state_tiles_item_light :
  save47_member0_checkpoint_state_tiles_item_light -> Js.Json.t

val read_save47_member0_checkpoint_state_tiles_item_light_member1 :
  Js.Json.t -> save47_member0_checkpoint_state_tiles_item_light_member1

val encode_save47_member0_checkpoint_state_tiles_item_light_member1 :
  save47_member0_checkpoint_state_tiles_item_light_member1 -> Js.Json.t

val read_save47_member0_checkpoint_state_tiles_item_light_member1_when_ :
  Js.Json.t -> save47_member0_checkpoint_state_tiles_item_light_member1_when_

val encode_save47_member0_checkpoint_state_tiles_item_light_member1_when_ :
  save47_member0_checkpoint_state_tiles_item_light_member1_when_ -> Js.Json.t

val read_save47_member0_checkpoint_state_campaign :
  Js.Json.t -> save47_member0_checkpoint_state_campaign

val encode_save47_member0_checkpoint_state_campaign :
  save47_member0_checkpoint_state_campaign -> Js.Json.t

val read_save47_member0_checkpoint_state_campaign_member1 :
  Js.Json.t -> save47_member0_checkpoint_state_campaign_member1

val encode_save47_member0_checkpoint_state_campaign_member1 :
  save47_member0_checkpoint_state_campaign_member1 -> Js.Json.t

val read_save47_member0_checkpoint_state_campaign_member1_scripts :
  Js.Json.t -> save47_member0_checkpoint_state_campaign_member1_scripts

val encode_save47_member0_checkpoint_state_campaign_member1_scripts :
  save47_member0_checkpoint_state_campaign_member1_scripts -> Js.Json.t

val read_save47_member0_checkpoint_state_campaign_member1_scripts_member1 :
  Js.Json.t -> save47_member0_checkpoint_state_campaign_member1_scripts_member1

val encode_save47_member0_checkpoint_state_campaign_member1_scripts_member1 :
  save47_member0_checkpoint_state_campaign_member1_scripts_member1 -> Js.Json.t

val read_save47_member0_checkpoint_state_campaign_member1_scripts_member1_marks :
  Js.Json.t -> save47_member0_checkpoint_state_campaign_member1_scripts_member1_marks

val encode_save47_member0_checkpoint_state_campaign_member1_scripts_member1_marks :
  save47_member0_checkpoint_state_campaign_member1_scripts_member1_marks -> Js.Json.t

val read_save47_member0_checkpoint_state_campaign_member1_scripts_member1_marks_item :
  Js.Json.t -> save47_member0_checkpoint_state_campaign_member1_scripts_member1_marks_item

val encode_save47_member0_checkpoint_state_campaign_member1_scripts_member1_marks_item :
  save47_member0_checkpoint_state_campaign_member1_scripts_member1_marks_item -> Js.Json.t

val read_save47_member1 : Js.Json.t -> save47_member1
val encode_save47_member1 : save47_member1 -> Js.Json.t
val read_save48 : Js.Json.t -> save48
val encode_save48 : save48 -> Js.Json.t
val read_save48_member0 : Js.Json.t -> save48_member0
val encode_save48_member0 : save48_member0 -> Js.Json.t
val read_literal_48 : Js.Json.t -> literal_48
val encode_literal_48 : literal_48 -> Js.Json.t
val read_save48_member0_checkpoint : Js.Json.t -> save48_member0_checkpoint
val encode_save48_member0_checkpoint : save48_member0_checkpoint -> Js.Json.t
val read_save48_member0_checkpoint_state : Js.Json.t -> save48_member0_checkpoint_state
val encode_save48_member0_checkpoint_state : save48_member0_checkpoint_state -> Js.Json.t

val read_save48_member0_checkpoint_state_ground_items :
  Js.Json.t -> save48_member0_checkpoint_state_ground_items

val encode_save48_member0_checkpoint_state_ground_items :
  save48_member0_checkpoint_state_ground_items -> Js.Json.t

val read_save48_member0_checkpoint_state_ground_items_item :
  Js.Json.t -> save48_member0_checkpoint_state_ground_items_item

val encode_save48_member0_checkpoint_state_ground_items_item :
  save48_member0_checkpoint_state_ground_items_item -> Js.Json.t

val read_save48_member0_checkpoint_state_ground_items_item_item :
  Js.Json.t -> save48_member0_checkpoint_state_ground_items_item_item

val encode_save48_member0_checkpoint_state_ground_items_item_item :
  save48_member0_checkpoint_state_ground_items_item_item -> Js.Json.t

val read_save48_member0_checkpoint_state_ground_items_item_item_perception :
  Js.Json.t -> save48_member0_checkpoint_state_ground_items_item_item_perception

val encode_save48_member0_checkpoint_state_ground_items_item_item_perception :
  save48_member0_checkpoint_state_ground_items_item_item_perception -> Js.Json.t

val read_save48_member0_checkpoint_state_ground_items_item_item_perception_member1 :
  Js.Json.t -> save48_member0_checkpoint_state_ground_items_item_item_perception_member1

val encode_save48_member0_checkpoint_state_ground_items_item_item_perception_member1 :
  save48_member0_checkpoint_state_ground_items_item_item_perception_member1 -> Js.Json.t

val read_save48_member0_checkpoint_state_tiles_item_perception_concealment :
  Js.Json.t -> save48_member0_checkpoint_state_tiles_item_perception_concealment

val encode_save48_member0_checkpoint_state_tiles_item_perception_concealment :
  save48_member0_checkpoint_state_tiles_item_perception_concealment -> Js.Json.t

val read_save48_member0_checkpoint_state_tiles_item_perception_concealment_item :
  Js.Json.t -> save48_member0_checkpoint_state_tiles_item_perception_concealment_item

val encode_save48_member0_checkpoint_state_tiles_item_perception_concealment_item :
  save48_member0_checkpoint_state_tiles_item_perception_concealment_item -> Js.Json.t

val read_save48_member0_checkpoint_state_tiles_item_perception_concealment_item_requires :
  Js.Json.t -> save48_member0_checkpoint_state_tiles_item_perception_concealment_item_requires

val encode_save48_member0_checkpoint_state_tiles_item_perception_concealment_item_requires :
  save48_member0_checkpoint_state_tiles_item_perception_concealment_item_requires -> Js.Json.t

val read_save48_member0_checkpoint_state_tiles_item_perception_concealment_item_requires_terrains :
  Js.Json.t ->
  save48_member0_checkpoint_state_tiles_item_perception_concealment_item_requires_terrains

val encode_save48_member0_checkpoint_state_tiles_item_perception_concealment_item_requires_terrains :
  save48_member0_checkpoint_state_tiles_item_perception_concealment_item_requires_terrains ->
  Js.Json.t

val read_save48_member0_checkpoint_state_tiles_item_perception_concealment_item_requires_terrains_member1 :
  Js.Json.t ->
  save48_member0_checkpoint_state_tiles_item_perception_concealment_item_requires_terrains_member1

val encode_save48_member0_checkpoint_state_tiles_item_perception_concealment_item_requires_terrains_member1 :
  save48_member0_checkpoint_state_tiles_item_perception_concealment_item_requires_terrains_member1 ->
  Js.Json.t

val read_save48_member0_checkpoint_state_tiles_item_perception_concealment_item_requires_terrains_member1_item :
  Js.Json.t ->
  save48_member0_checkpoint_state_tiles_item_perception_concealment_item_requires_terrains_member1_item

val encode_save48_member0_checkpoint_state_tiles_item_perception_concealment_item_requires_terrains_member1_item :
  save48_member0_checkpoint_state_tiles_item_perception_concealment_item_requires_terrains_member1_item ->
  Js.Json.t

val read_literal_mountain_mine : Js.Json.t -> literal_mountain_mine
val encode_literal_mountain_mine : literal_mountain_mine -> Js.Json.t
val read_save48_member0_checkpoint_state_tiles : Js.Json.t -> save48_member0_checkpoint_state_tiles

val encode_save48_member0_checkpoint_state_tiles :
  save48_member0_checkpoint_state_tiles -> Js.Json.t

val read_save48_member0_checkpoint_state_tiles_item :
  Js.Json.t -> save48_member0_checkpoint_state_tiles_item

val encode_save48_member0_checkpoint_state_tiles_item :
  save48_member0_checkpoint_state_tiles_item -> Js.Json.t

val read_save48_member0_checkpoint_state_tiles_item_perception :
  Js.Json.t -> save48_member0_checkpoint_state_tiles_item_perception

val encode_save48_member0_checkpoint_state_tiles_item_perception :
  save48_member0_checkpoint_state_tiles_item_perception -> Js.Json.t

val read_save48_member0_checkpoint_state_units : Js.Json.t -> save48_member0_checkpoint_state_units

val encode_save48_member0_checkpoint_state_units :
  save48_member0_checkpoint_state_units -> Js.Json.t

val read_save48_member0_checkpoint_state_units_item :
  Js.Json.t -> save48_member0_checkpoint_state_units_item

val encode_save48_member0_checkpoint_state_units_item :
  save48_member0_checkpoint_state_units_item -> Js.Json.t

val read_save48_member0_checkpoint_state_units_item_perception_effects :
  Js.Json.t -> save48_member0_checkpoint_state_units_item_perception_effects

val encode_save48_member0_checkpoint_state_units_item_perception_effects :
  save48_member0_checkpoint_state_units_item_perception_effects -> Js.Json.t

val read_save48_member0_checkpoint_state_units_item_perception_effects_item :
  Js.Json.t -> save48_member0_checkpoint_state_units_item_perception_effects_item

val encode_save48_member0_checkpoint_state_units_item_perception_effects_item :
  save48_member0_checkpoint_state_units_item_perception_effects_item -> Js.Json.t

val read_save48_member0_checkpoint_state_units_item_items :
  Js.Json.t -> save48_member0_checkpoint_state_units_item_items

val encode_save48_member0_checkpoint_state_units_item_items :
  save48_member0_checkpoint_state_units_item_items -> Js.Json.t

val read_save48_member0_checkpoint_state_units_item_items_member1 :
  Js.Json.t -> save48_member0_checkpoint_state_units_item_items_member1

val encode_save48_member0_checkpoint_state_units_item_items_member1 :
  save48_member0_checkpoint_state_units_item_items_member1 -> Js.Json.t

val read_save48_member0_checkpoint_state_campaign :
  Js.Json.t -> save48_member0_checkpoint_state_campaign

val encode_save48_member0_checkpoint_state_campaign :
  save48_member0_checkpoint_state_campaign -> Js.Json.t

val read_save48_member0_checkpoint_state_campaign_member1 :
  Js.Json.t -> save48_member0_checkpoint_state_campaign_member1

val encode_save48_member0_checkpoint_state_campaign_member1 :
  save48_member0_checkpoint_state_campaign_member1 -> Js.Json.t

val read_save48_member0_checkpoint_state_campaign_member1_evacuated :
  Js.Json.t -> save48_member0_checkpoint_state_campaign_member1_evacuated

val encode_save48_member0_checkpoint_state_campaign_member1_evacuated :
  save48_member0_checkpoint_state_campaign_member1_evacuated -> Js.Json.t

val read_save48_member0_checkpoint_state_campaign_member1_evacuated_member1 :
  Js.Json.t -> save48_member0_checkpoint_state_campaign_member1_evacuated_member1

val encode_save48_member0_checkpoint_state_campaign_member1_evacuated_member1 :
  save48_member0_checkpoint_state_campaign_member1_evacuated_member1 -> Js.Json.t

val read_save48_member0_checkpoint_state_campaign_member1_evacuated_member1_item :
  Js.Json.t -> save48_member0_checkpoint_state_campaign_member1_evacuated_member1_item

val encode_save48_member0_checkpoint_state_campaign_member1_evacuated_member1_item :
  save48_member0_checkpoint_state_campaign_member1_evacuated_member1_item -> Js.Json.t

val read_save48_member0_checkpoint_state_campaign_member1_veterans :
  Js.Json.t -> save48_member0_checkpoint_state_campaign_member1_veterans

val encode_save48_member0_checkpoint_state_campaign_member1_veterans :
  save48_member0_checkpoint_state_campaign_member1_veterans -> Js.Json.t

val read_save48_member0_checkpoint_state_campaign_member1_veterans_member1 :
  Js.Json.t -> save48_member0_checkpoint_state_campaign_member1_veterans_member1

val encode_save48_member0_checkpoint_state_campaign_member1_veterans_member1 :
  save48_member0_checkpoint_state_campaign_member1_veterans_member1 -> Js.Json.t

val read_save48_member0_checkpoint_state_campaign_member1_veterans_member1_item :
  Js.Json.t -> save48_member0_checkpoint_state_campaign_member1_veterans_member1_item

val encode_save48_member0_checkpoint_state_campaign_member1_veterans_member1_item :
  save48_member0_checkpoint_state_campaign_member1_veterans_member1_item -> Js.Json.t

val read_save48_member1 : Js.Json.t -> save48_member1
val encode_save48_member1 : save48_member1 -> Js.Json.t
val read_history5_entries_item_mission : Js.Json.t -> history5_entries_item_mission
val encode_history5_entries_item_mission : history5_entries_item_mission -> Js.Json.t
val read_literal_49 : Js.Json.t -> literal_49
val encode_literal_49 : literal_49 -> Js.Json.t
val read_save49_member0_progress_entry : Js.Json.t -> save49_member0_progress_entry
val encode_save49_member0_progress_entry : save49_member0_progress_entry -> Js.Json.t
val read_save49_member0_progress : Js.Json.t -> save49_member0_progress
val encode_save49_member0_progress : save49_member0_progress -> Js.Json.t
val read_save49_member0 : Js.Json.t -> save49_member0
val encode_save49_member0 : save49_member0 -> Js.Json.t
val read_history5_entries_item : Js.Json.t -> history5_entries_item
val encode_history5_entries_item : history5_entries_item -> Js.Json.t
val read_save49_member1_progress : Js.Json.t -> save49_member1_progress
val encode_save49_member1_progress : save49_member1_progress -> Js.Json.t
val read_save49_member1 : Js.Json.t -> save49_member1
val encode_save49_member1 : save49_member1 -> Js.Json.t
val read_save49 : Js.Json.t -> save49
val encode_save49 : save49 -> Js.Json.t
val read_history5_entries : Js.Json.t -> history5_entries
val encode_history5_entries : history5_entries -> Js.Json.t
val read_history5 : Js.Json.t -> history5
val encode_history5 : history5 -> Js.Json.t

module V2 : sig
  val of_json : Js.Json.t -> Historical_types.V2.t
  val to_json : Historical_types.V2.t -> Js.Json.t
end

module V3 : sig
  val of_json : Js.Json.t -> Historical_types.V3.t
  val to_json : Historical_types.V3.t -> Js.Json.t
end

module V4 : sig
  val of_json : Js.Json.t -> Historical_types.V4.t
  val to_json : Historical_types.V4.t -> Js.Json.t
end

module V5 : sig
  val of_json : Js.Json.t -> Historical_types.V5.t
  val to_json : Historical_types.V5.t -> Js.Json.t
end

module V6 : sig
  val of_json : Js.Json.t -> Historical_types.V6.t
  val to_json : Historical_types.V6.t -> Js.Json.t
end

module V7 : sig
  val of_json : Js.Json.t -> Historical_types.V7.t
  val to_json : Historical_types.V7.t -> Js.Json.t
end

module V8 : sig
  val of_json : Js.Json.t -> Historical_types.V8.t
  val to_json : Historical_types.V8.t -> Js.Json.t
end

module V9 : sig
  val of_json : Js.Json.t -> Historical_types.V9.t
  val to_json : Historical_types.V9.t -> Js.Json.t
end

module V10 : sig
  val of_json : Js.Json.t -> Historical_types.V10.t
  val to_json : Historical_types.V10.t -> Js.Json.t
end

module V11 : sig
  val of_json : Js.Json.t -> Historical_types.V11.t
  val to_json : Historical_types.V11.t -> Js.Json.t
end

module V12 : sig
  val of_json : Js.Json.t -> Historical_types.V12.t
  val to_json : Historical_types.V12.t -> Js.Json.t
end

module V13 : sig
  val of_json : Js.Json.t -> Historical_types.V13.t
  val to_json : Historical_types.V13.t -> Js.Json.t
end

module V14 : sig
  val of_json : Js.Json.t -> Historical_types.V14.t
  val to_json : Historical_types.V14.t -> Js.Json.t
end

module V15 : sig
  val of_json : Js.Json.t -> Historical_types.V15.t
  val to_json : Historical_types.V15.t -> Js.Json.t
end

module V16 : sig
  val of_json : Js.Json.t -> Historical_types.V16.t
  val to_json : Historical_types.V16.t -> Js.Json.t
end

module V17 : sig
  val of_json : Js.Json.t -> Historical_types.V17.t
  val to_json : Historical_types.V17.t -> Js.Json.t
end

module V18 : sig
  val of_json : Js.Json.t -> Historical_types.V18.t
  val to_json : Historical_types.V18.t -> Js.Json.t
end

module V19 : sig
  val of_json : Js.Json.t -> Historical_types.V19.t
  val to_json : Historical_types.V19.t -> Js.Json.t
end

module V20 : sig
  val of_json : Js.Json.t -> Historical_types.V20.t
  val to_json : Historical_types.V20.t -> Js.Json.t
end

module V21 : sig
  val of_json : Js.Json.t -> Historical_types.V21.t
  val to_json : Historical_types.V21.t -> Js.Json.t
end

module V22 : sig
  val of_json : Js.Json.t -> Historical_types.V22.t
  val to_json : Historical_types.V22.t -> Js.Json.t
end

module V23 : sig
  val of_json : Js.Json.t -> Historical_types.V23.t
  val to_json : Historical_types.V23.t -> Js.Json.t
end

module V24 : sig
  val of_json : Js.Json.t -> Historical_types.V24.t
  val to_json : Historical_types.V24.t -> Js.Json.t
end

module V25 : sig
  val of_json : Js.Json.t -> Historical_types.V25.t
  val to_json : Historical_types.V25.t -> Js.Json.t
end

module V26 : sig
  val of_json : Js.Json.t -> Historical_types.V26.t
  val to_json : Historical_types.V26.t -> Js.Json.t
end

module V27 : sig
  val of_json : Js.Json.t -> Historical_types.V27.t
  val to_json : Historical_types.V27.t -> Js.Json.t
end

module V28 : sig
  val of_json : Js.Json.t -> Historical_types.V28.t
  val to_json : Historical_types.V28.t -> Js.Json.t
end

module V29 : sig
  val of_json : Js.Json.t -> Historical_types.V29.t
  val to_json : Historical_types.V29.t -> Js.Json.t
end

module V30 : sig
  val of_json : Js.Json.t -> Historical_types.V30.t
  val to_json : Historical_types.V30.t -> Js.Json.t
end

module V31 : sig
  val of_json : Js.Json.t -> Historical_types.V31.t
  val to_json : Historical_types.V31.t -> Js.Json.t
end

module V32 : sig
  val of_json : Js.Json.t -> Historical_types.V32.t
  val to_json : Historical_types.V32.t -> Js.Json.t
end

module V33 : sig
  val of_json : Js.Json.t -> Historical_types.V33.t
  val to_json : Historical_types.V33.t -> Js.Json.t
end

module V34 : sig
  val of_json : Js.Json.t -> Historical_types.V34.t
  val to_json : Historical_types.V34.t -> Js.Json.t
end

module V35 : sig
  val of_json : Js.Json.t -> Historical_types.V35.t
  val to_json : Historical_types.V35.t -> Js.Json.t
end

module V36 : sig
  val of_json : Js.Json.t -> Historical_types.V36.t
  val to_json : Historical_types.V36.t -> Js.Json.t
end

module V37 : sig
  val of_json : Js.Json.t -> Historical_types.V37.t
  val to_json : Historical_types.V37.t -> Js.Json.t
end

module V38 : sig
  val of_json : Js.Json.t -> Historical_types.V38.t
  val to_json : Historical_types.V38.t -> Js.Json.t
end

module V39 : sig
  val of_json : Js.Json.t -> Historical_types.V39.t
  val to_json : Historical_types.V39.t -> Js.Json.t
end

module History1 : sig
  val of_json : Js.Json.t -> Historical_types.History1.t
  val to_json : Historical_types.History1.t -> Js.Json.t
end

module History2 : sig
  val of_json : Js.Json.t -> Historical_types.History2.t
  val to_json : Historical_types.History2.t -> Js.Json.t
end

module History3 : sig
  val of_json : Js.Json.t -> Historical_types.History3.t
  val to_json : Historical_types.History3.t -> Js.Json.t
end

module History4 : sig
  val of_json : Js.Json.t -> Historical_types.History4.t
  val to_json : Historical_types.History4.t -> Js.Json.t
  val of_legacy : Historical_types.History4.t -> Historical_types.History4.t
end

module Fragments_document : sig
  val of_json : Js.Json.t -> Historical_types.Fragments.t
  val to_json : Historical_types.Fragments.t -> Js.Json.t
end

module Native1 : sig
  val of_json : Js.Json.t -> Historical_types.Native1.t
  val to_json : Historical_types.Native1.t -> Js.Json.t
end

module Javascript1 : sig
  val of_json : Js.Json.t -> Historical_types.Javascript1.t
  val to_json : Historical_types.Javascript1.t -> Js.Json.t
end

module V40 : sig
  val of_json : Js.Json.t -> Historical_types.V40.t
  val to_json : Historical_types.V40.t -> Js.Json.t
end

module V41 : sig
  val of_json : Js.Json.t -> Historical_types.V41.t
  val to_json : Historical_types.V41.t -> Js.Json.t
end

module V42 : sig
  val of_json : Js.Json.t -> Historical_types.V42.t
  val to_json : Historical_types.V42.t -> Js.Json.t
end

module V43 : sig
  val of_json : Js.Json.t -> Historical_types.V43.t
  val to_json : Historical_types.V43.t -> Js.Json.t
end

module V44 : sig
  val of_json : Js.Json.t -> Historical_types.V44.t
  val to_json : Historical_types.V44.t -> Js.Json.t
end

module V45 : sig
  val of_json : Js.Json.t -> Historical_types.V45.t
  val to_json : Historical_types.V45.t -> Js.Json.t
end

module V46 : sig
  val of_json : Js.Json.t -> Historical_types.V46.t
  val to_json : Historical_types.V46.t -> Js.Json.t
end

module V47 : sig
  val of_json : Js.Json.t -> Historical_types.V47.t
  val to_json : Historical_types.V47.t -> Js.Json.t
end

module V48 : sig
  val of_json : Js.Json.t -> Historical_types.V48.t
  val to_json : Historical_types.V48.t -> Js.Json.t
end

module V49 : sig
  val of_json : Js.Json.t -> Historical_types.V49.t
  val to_json : Historical_types.V49.t -> Js.Json.t
  val of_legacy : Historical_types.V49.t -> Historical_types.V49.t
  val current_entry_to_json : Historical_types.V49.current_entry -> Js.Json.t
  val turn_entry_to_json : Historical_types.V49.turn_entry -> Js.Json.t
end

module History5 : sig
  val of_json : Js.Json.t -> Historical_types.History5.t
  val to_json : Historical_types.History5.t -> Js.Json.t
end

module Fragment : sig
  val native_state38 : Js.Json.t -> fragments_native_state38
  val factions7 : Js.Json.t -> fragments_factions7
  val factions11 : Js.Json.t -> fragments_factions7
  val factions21 : Js.Json.t -> fragments_factions7
  val factions24 : Js.Json.t -> fragments_factions7
  val tiles11 : Js.Json.t -> fragments_tiles11
  val rule11 : Js.Json.t -> fragments_rule11
  val delta19 : Js.Json.t -> fragments_delta19
  val delta20 : Js.Json.t -> fragments_delta19
  val delta21 : Js.Json.t -> fragments_delta19
  val light23 : Js.Json.t -> fragments_light23
  val light28 : Js.Json.t -> fragments_light28
end

module Fragments : sig
  val native_state38_of_json : Js.Json.t -> fragments_native_state38
  val native_state38_to_json : fragments_native_state38 -> Js.Json.t
  val factions7_of_json : Js.Json.t -> fragments_factions7
  val factions7_to_json : fragments_factions7 -> Js.Json.t
  val factions11_of_json : Js.Json.t -> fragments_factions7
  val factions11_to_json : fragments_factions7 -> Js.Json.t
  val factions21_of_json : Js.Json.t -> fragments_factions7
  val factions21_to_json : fragments_factions7 -> Js.Json.t
  val factions24_of_json : Js.Json.t -> fragments_factions7
  val factions24_to_json : fragments_factions7 -> Js.Json.t
  val tiles11_of_json : Js.Json.t -> fragments_tiles11
  val tiles11_to_json : fragments_tiles11 -> Js.Json.t
  val rule11_of_json : Js.Json.t -> fragments_rule11
  val rule11_to_json : fragments_rule11 -> Js.Json.t
  val delta19_of_json : Js.Json.t -> fragments_delta19
  val delta19_to_json : fragments_delta19 -> Js.Json.t
  val delta20_of_json : Js.Json.t -> fragments_delta19
  val delta20_to_json : fragments_delta19 -> Js.Json.t
  val delta21_of_json : Js.Json.t -> fragments_delta19
  val delta21_to_json : fragments_delta19 -> Js.Json.t
  val light23_of_json : Js.Json.t -> fragments_light23
  val light23_to_json : fragments_light23 -> Js.Json.t
  val light28_of_json : Js.Json.t -> fragments_light28
  val light28_to_json : fragments_light28 -> Js.Json.t
end
