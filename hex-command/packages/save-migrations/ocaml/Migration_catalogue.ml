(* Pure v51 to v52: retain equipment identities; retire autonomous tile observers. *)

open Historical_types

[@@@warning "+8+9-30-39"]
[@@@warnerror "+8+9"]

let rec convert_4159_4224 (value : save51) : save52 =
  match value with
  | `Current v -> `Current (convert_4160_4225 v)
  | `Turn v -> `Turn (convert_4218_4577 v)

and convert_4160_4225 (value : save51_member0) : save52_member0 =
  let ({ wire; pending_enemy; undo; progress; version; checkpoint; setup } : save51_member0) =
    value
  in
  {
    wire;
    pending_enemy;
    undo;
    progress;
    version = convert_4181_4246 version;
    checkpoint = convert_4182_4247 checkpoint;
    setup;
  }

and convert_4218_4577 (value : save51_member1) : save52_member1 =
  let ({ wire; pending_enemy; undo; progress; version; checkpoint; setup } : save51_member1) =
    value
  in
  {
    wire;
    pending_enemy;
    undo;
    progress;
    version = convert_4181_4246 version;
    checkpoint = convert_4182_4247 checkpoint;
    setup;
  }

and convert_4181_4246 (value : literal_51) : literal_52 =
  let _ = value in
  `V52

and convert_4182_4247 (value : save51_member0_checkpoint) : save52_member0_checkpoint =
  let ({ wire; seed; state } : save51_member0_checkpoint) = value in
  { wire; seed; state = convert_3827_4249 state }

and convert_3827_4249 (value : save51_member0_checkpoint_state) : save52_member0_checkpoint_state =
  let ({
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
       }
        : save51_member0_checkpoint_state) =
    value
  in
  {
    wire;
    map_modifiers;
    lighting;
    vision_groups = convert_3857_4270 vision_groups;
    factions;
    ground_items = convert_4035_4361 ground_items;
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
    tiles = convert_4081_4371 tiles;
    units = convert_4102_4470 units;
    campaign = convert_4155_4489 campaign;
    last_combat;
  }

and convert_3857_4270 (value : save51_member0_checkpoint_state_vision_groups) :
    save52_member0_checkpoint_state_vision_groups =
  Array.map (fun v -> convert_3847_4271 v) value

and convert_4035_4361 (value : save51_member0_checkpoint_state_ground_items) :
    save52_member0_checkpoint_state_ground_items =
  Array.map (fun v -> convert_3950_4362 v) value

and convert_4081_4371 (value : save51_member0_checkpoint_state_tiles) :
    save52_member0_checkpoint_state_tiles =
  Array.map (fun v -> convert_4047_4372 v) value

and convert_4102_4470 (value : save51_member0_checkpoint_state_units) :
    save52_member0_checkpoint_state_units =
  Array.map (fun v -> convert_4082_4471 v) value

and convert_4155_4489 (value : save51_member0_checkpoint_state_campaign) :
    save52_member0_checkpoint_state_campaign =
  match value with `Null -> `Null | `Object2 v -> `Object2 (convert_4103_4490 v)

and convert_3847_4271 (value : save51_member0_checkpoint_state_vision_groups_item) :
    save52_member0_checkpoint_state_vision_groups_item =
  let ({ wire; id; sight; explored } : save51_member0_checkpoint_state_vision_groups_item) =
    value
  in
  { wire; id; sight = convert_3849_4273 sight; explored }

and convert_3950_4362 (value : save51_member0_checkpoint_state_ground_items_item) :
    save52_member0_checkpoint_state_ground_items_item =
  let ({ wire; tile; item } : save51_member0_checkpoint_state_ground_items_item) = value in
  { wire; tile; item = convert_3951_4272 item }

and convert_4047_4372 (value : save51_member0_checkpoint_state_tiles_item) :
    save52_member0_checkpoint_state_tiles_item =
  let ({
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
       }
        : save51_member0_checkpoint_state_tiles_item) =
    value
  in
  {
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
    perception = convert_4066_4457 perception;
    light;
  }

and convert_4082_4471 (value : save51_member0_checkpoint_state_units_item) :
    save52_member0_checkpoint_state_units_item =
  let ({
         wire;
         revealed_to;
         perception_effects;
         faction;
         items;
         ai_group;
         id;
         type_;
         tile;
         hp;
         moved;
         attack_spent;
         experience;
         character;
         role;
         retreating;
         rebuilding;
       }
        : save51_member0_checkpoint_state_units_item) =
    value
  in
  {
    wire;
    revealed_to;
    perception_effects;
    faction;
    items = convert_4090_4479 items;
    ai_group;
    id;
    type_;
    tile;
    hp;
    moved;
    attack_spent;
    experience;
    character;
    role;
    retreating;
    rebuilding;
  }

and convert_4103_4490 (value : save51_member0_checkpoint_state_campaign_member1) :
    save52_member0_checkpoint_state_campaign_member1 =
  let ({
         wire;
         scripts;
         evacuated;
         id;
         mission;
         recall;
         veterans;
         story_veterans;
         fallen_companions;
         carry_gold;
         marks;
         dialogue;
         dialogue_index;
         outcome;
         completed;
         refuge;
         vigil;
         holds;
         flags;
       }
        : save51_member0_checkpoint_state_campaign_member1) =
    value
  in
  {
    wire;
    scripts = convert_4126_4491 scripts;
    evacuated = convert_4128_4513 evacuated;
    id;
    mission;
    recall;
    veterans = convert_4135_4516 veterans;
    story_veterans = convert_4137_4522 story_veterans;
    fallen_companions;
    carry_gold;
    marks;
    dialogue;
    dialogue_index;
    outcome;
    completed;
    refuge;
    vigil;
    holds;
    flags;
  }

and convert_3849_4273 (value : save51_member0_checkpoint_state_vision_groups_item_sight) :
    save51_member0_checkpoint_state_map_modifiers_sight =
  let ({ wire; range; penetrates } : save51_member0_checkpoint_state_vision_groups_item_sight) =
    value
  in
  let range, unlimited = match range with `Number n -> (n, false) | `Unlimited -> (0., true) in
  {
    wire;
    range;
    unlimited;
    sight_modifiers = { wire = empty_wire; lit = 0.; unlit = 0.; darkness = 0. };
    penetrates;
  }

and convert_3951_4272 (value : save51_member0_checkpoint_state_ground_items_item_item) : json_string
    =
  let ({ wire = _; sight = _; effects = _; armor = _; id; name = _; description = _; attacks = _ }
        : save51_member0_checkpoint_state_ground_items_item_item) =
    value
  in
  id

and convert_4066_4457 (value : save51_member0_checkpoint_state_tiles_item_perception) :
    save52_member0_checkpoint_state_tiles_item_perception =
  let ({ wire; illumination; sight = _; remember; concealment }
        : save51_member0_checkpoint_state_tiles_item_perception) =
    value
  in
  { wire; illumination; remember; concealment }

and convert_4090_4479 (value : save51_member0_checkpoint_state_units_item_items) :
    fragments_native_state38_log =
  Array.map (fun v -> convert_3951_4272 v) value

and convert_4126_4491 (value : save51_member0_checkpoint_state_campaign_member1_scripts) :
    save52_member0_checkpoint_state_campaign_member1_scripts =
  match value with `Null -> `Null | `Object2 v -> `Object2 (convert_4104_4492 v)

and convert_4128_4513 (value : save51_member0_checkpoint_state_campaign_member1_evacuated) :
    save52_member0_checkpoint_state_campaign_member1_evacuated =
  Array.map (fun v -> convert_4127_4514 v) value

and convert_4135_4516 (value : save51_member0_checkpoint_state_campaign_member1_veterans) :
    save52_member0_checkpoint_state_campaign_member1_veterans =
  match value with `Null -> `Null | `Array v -> `Array (convert_4134_4517 v)

and convert_4137_4522 (value : save51_member0_checkpoint_state_campaign_member1_veterans) :
    save52_member0_checkpoint_state_campaign_member1_veterans =
  match value with `Null -> `Null | `Array v -> `Array (convert_4136_4523 v)

and convert_4104_4492 (value : save51_member0_checkpoint_state_campaign_member1_scripts_member1) :
    save52_member0_checkpoint_state_campaign_member1_scripts_member1 =
  let ({ wire; failed; completed; revealed; recruitment; active; holds; marks; pending; next_mark }
        : save51_member0_checkpoint_state_campaign_member1_scripts_member1) =
    value
  in
  {
    wire;
    failed;
    completed;
    revealed;
    recruitment;
    active;
    holds;
    marks = convert_4116_4501 marks;
    pending;
    next_mark;
  }

and convert_4127_4514 (value : save51_member0_checkpoint_state_campaign_member1_evacuated_item) :
    save52_member0_checkpoint_state_campaign_member1_evacuated_item =
  let ({ wire; event; unit_ } : save51_member0_checkpoint_state_campaign_member1_evacuated_item) =
    value
  in
  { wire; event; unit_ = convert_4082_4471 unit_ }

and convert_4134_4517 (value : save51_member0_checkpoint_state_campaign_member1_veterans_member1) :
    save52_member0_checkpoint_state_campaign_member1_veterans_member1 =
  Array.map (fun v -> convert_4130_4518 v) value

and convert_4136_4523 (value : save51_member0_checkpoint_state_campaign_member1_veterans_member1) :
    save52_member0_checkpoint_state_campaign_member1_veterans_member1 =
  Array.map (fun v -> convert_4130_4518 v) value

and convert_4116_4501
    (value : save51_member0_checkpoint_state_campaign_member1_scripts_member1_marks) :
    save52_member0_checkpoint_state_campaign_member1_scripts_member1_marks =
  Array.map (fun v -> convert_4113_4502 v) value

and convert_4130_4518
    (value : save51_member0_checkpoint_state_campaign_member1_veterans_member1_item) :
    save52_member0_checkpoint_state_campaign_member1_veterans_member1_item =
  let ({ wire; items; type_; experience; character; role }
        : save51_member0_checkpoint_state_campaign_member1_veterans_member1_item) =
    value
  in
  { wire; items = convert_4131_4519 items; type_; experience; character; role }

and convert_4113_4502
    (value : save51_member0_checkpoint_state_campaign_member1_scripts_member1_marks_item) :
    save52_member0_checkpoint_state_campaign_member1_scripts_member1_marks_item =
  let ({ wire; sight = _; light_remembered; id; type_; tile; light }
        : save51_member0_checkpoint_state_campaign_member1_scripts_member1_marks_item) =
    value
  in
  { wire; light_remembered; id; type_; tile; light }

and convert_4131_4519 (value : save51_member0_checkpoint_state_units_item_items) :
    fragments_native_state38_log =
  Array.map (fun v -> convert_3951_4272 v) value

let v51 (old : V51.t) : V52.t =
  let convert checkpoint undo =
    Migration_late.convert_state_and_undo_with_codecs
      ~encode_change:Historical_codec.V51.current_changes_item_to_json
      ~decode_change:Historical_codec.V52.current_changes_item_of_json
      ~encode_old:Historical_codec.V51.current_state_to_json
      ~decode_old:Historical_codec.V51.current_state_of_json
      ~encode_new:Historical_codec.V52.current_state_to_json convert_3827_4249 checkpoint undo
  in
  match (old, convert_4159_4224 old) with
  | `Current old, `Current value ->
      let state, undo = convert old.checkpoint.state (Some old.undo) in
      `Current { value with checkpoint = { value.checkpoint with state }; undo = Option.get undo }
  | `Turn old, `Turn value ->
      let state, _ = convert old.checkpoint.state None in
      `Turn { value with checkpoint = { value.checkpoint with state } }
  | _ -> assert false
