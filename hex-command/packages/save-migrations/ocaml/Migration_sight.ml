(* Pure v52 to v53: fold sight.range into sightModifiers and drop sight.range. *)

open Historical_types

[@@@warning "+8+9-30-39"]
[@@@warnerror "+8+9"]

let fold_range range unlimited
    ({ wire; lit; unlit; darkness } :
      save51_member0_checkpoint_state_map_modifiers_sight_sight_modifiers) :
    save51_member0_checkpoint_state_map_modifiers_sight_sight_modifiers =
  if unlimited || range = 0. then { wire; lit; unlit; darkness }
  else { wire; lit = lit +. range; unlit = unlit +. range; darkness = darkness +. range }

let convert_sight
    ({ wire; range; unlimited; sight_modifiers; penetrates } :
      save51_member0_checkpoint_state_map_modifiers_sight) :
    save53_member0_checkpoint_state_map_modifiers_sight =
  { wire; unlimited; sight_modifiers = fold_range range unlimited sight_modifiers; penetrates }

let convert_map_modifiers
    ({ wire; sight; damage_modifiers; terrain_modifiers } :
      save51_member0_checkpoint_state_map_modifiers) : save53_member0_checkpoint_state_map_modifiers
    =
  { wire; sight = convert_sight sight; damage_modifiers; terrain_modifiers }

let convert_effect_value :
    save51_member0_checkpoint_state_tiles_item_effects_item_effect_ ->
    save53_member0_checkpoint_state_tiles_item_effects_item_effect_ = function
  | `Healing v -> `Healing v
  | `Thorns v -> `Thorns v
  | `Sight { wire; kind; range; unlimited; sight_modifiers; penetrates } ->
      `Sight
        {
          wire;
          kind;
          unlimited;
          sight_modifiers = fold_range range unlimited sight_modifiers;
          penetrates;
        }
  | `Concealment v -> `Concealment v
  | `Light v -> `Light v
  | `Stat_modifiers { wire; kind; modifiers } ->
      `Stat_modifiers { wire; kind; modifiers = convert_map_modifiers modifiers }

let convert_scoped_effect
    ({ wire; scope; stacking; effect_ } : save51_member0_checkpoint_state_tiles_item_effects_item) :
    save53_member0_checkpoint_state_tiles_item_effects_item =
  { wire; scope; stacking; effect_ = convert_effect_value effect_ }

let convert_tile
    ({
       wire;
       effects;
       id;
       location;
       col;
       row;
       terrain;
       owner;
       connections;
       dead_ends;
       walls;
       floor;
       perception;
       light;
     } :
      save52_member0_checkpoint_state_tiles_item) : save53_member0_checkpoint_state_tiles_item =
  {
    wire;
    effects = Array.map convert_scoped_effect effects;
    id;
    location;
    col;
    row;
    terrain;
    owner;
    connections;
    dead_ends;
    walls;
    floor;
    perception;
    light;
  }

let convert_vision_group
    ({ wire; id; sight; explored } : save52_member0_checkpoint_state_vision_groups_item) :
    save53_member0_checkpoint_state_vision_groups_item =
  { wire; id; sight = convert_sight sight; explored }

let convert_state
    ({
       wire;
       map_modifiers;
       lighting;
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
       tiles;
       units;
       campaign;
       last_combat;
     } :
      save52_member0_checkpoint_state) : save53_member0_checkpoint_state =
  {
    wire;
    map_modifiers = convert_map_modifiers map_modifiers;
    lighting;
    vision_groups = Array.map convert_vision_group vision_groups;
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
    tiles = Array.map convert_tile tiles;
    units;
    campaign;
    last_combat;
  }

let convert_checkpoint ({ wire; seed; state } : save52_member0_checkpoint) :
    save53_member0_checkpoint =
  { wire; seed; state = convert_state state }

let convert_version (_ : literal_52) : literal_53 = `V53

let convert_save (value : save52) : save53 =
  match value with
  | `Current { wire; pending_enemy; undo; progress; version; checkpoint; setup } ->
      `Current
        {
          wire;
          pending_enemy;
          undo;
          progress;
          version = convert_version version;
          checkpoint = convert_checkpoint checkpoint;
          setup;
        }
  | `Turn { wire; pending_enemy; undo; progress; version; checkpoint; setup } ->
      `Turn
        {
          wire;
          pending_enemy;
          undo;
          progress;
          version = convert_version version;
          checkpoint = convert_checkpoint checkpoint;
          setup;
        }

let v52 (old : V52.t) : V53.t =
  let convert checkpoint undo =
    Migration_late.convert_state_and_undo_with_codecs
      ~encode_change:Historical_codec.V52.current_changes_item_to_json
      ~decode_change:Historical_codec.V53.current_changes_item_of_json
      ~encode_old:Historical_codec.V52.current_state_to_json
      ~decode_old:Historical_codec.V52.current_state_of_json
      ~encode_new:Historical_codec.V53.current_state_to_json convert_state checkpoint undo
  in
  match (old, convert_save old) with
  | `Current old, `Current value ->
      let state, undo = convert old.checkpoint.state (Some old.undo) in
      `Current { value with checkpoint = { value.checkpoint with state }; undo = Option.get undo }
  | `Turn old, `Turn value ->
      let state, _ = convert old.checkpoint.state None in
      `Turn { value with checkpoint = { value.checkpoint with state } }
  | _ -> assert false
