(* Frozen v50 -> v51. Sources become typed effects; expiry is retired.
   New source contributions are explicit; Darkvision uses its v51 additive
   adjustment. Saved tile illumination remains explicit, and map time comes
   from the frozen v50 catalog. *)
open Historical_types

[@@@warning "+8+9"]
[@@@warnerror "+8+9"]

let neutral_sight : save51_member0_checkpoint_state_map_modifiers_sight =
  ({
     wire = empty_wire;
     range = 0.;
     unlimited = false;
     sight_modifiers =
       ({ wire = empty_wire; lit = 0.; unlit = 0.; darkness = 0. }
         : save51_member0_checkpoint_state_map_modifiers_sight_sight_modifiers);
     penetrates = [||];
   }
    : save51_member0_checkpoint_state_map_modifiers_sight)

let scoped radius include_self targets group combine effect_ :
    save51_member0_checkpoint_state_tiles_item_effects_item =
  {
    wire = empty_wire;
    scope = { wire = empty_wire; radius; include_self; targets };
    stacking = { wire = empty_wire; group; combine };
    effect_;
  }

let rec convert_3758_4159 (value : save50) : save51 =
  match value with
  | `Current v -> `Current (convert_3759_4160 v)
  | `Turn v -> `Turn (convert_3817_4218 v)

and convert_3759_4160 (value : save50_member0) : save51_member0 =
  {
    wire = value.wire;
    pending_enemy = value.pending_enemy;
    undo = value.undo;
    progress = value.progress;
    version = convert_3780_4181 value.version;
    checkpoint = convert_3781_4182 value.checkpoint;
    setup = value.setup;
  }

and convert_3817_4218 (value : save50_member1) : save51_member1 =
  {
    wire = value.wire;
    pending_enemy = value.pending_enemy;
    undo = value.undo;
    progress = value.progress;
    version = convert_3780_4181 value.version;
    checkpoint = convert_3781_4182 value.checkpoint;
    setup = value.setup;
  }

and convert_3780_4181 (value : literal_50) : literal_51 =
  let _ = value in
  `V51

and convert_3781_4182 (value : save50_member0_checkpoint) : save51_member0_checkpoint =
  { wire = value.wire; seed = value.seed; state = convert_3456_3827 value.state }

and convert_3456_3827 (value : save50_member0_checkpoint_state) : save51_member0_checkpoint_state =
  {
    wire = value.wire;
    map_modifiers =
      ({
         wire = empty_wire;
         sight =
           ({
              wire = empty_wire;
              range = 0.;
              unlimited = false;
              sight_modifiers =
                ({ wire = empty_wire; lit = 0.; unlit = 0.; darkness = 0. }
                  : save51_member0_checkpoint_state_map_modifiers_sight_sight_modifiers);
              penetrates = [||];
            }
             : save51_member0_checkpoint_state_map_modifiers_sight);
         damage_modifiers =
           ({ wire = empty_wire; blade = 0.; pierce = 0.; impact = 0. }
             : fragments_native_state38_ground_items_item_item_armor_member1);
         terrain_modifiers = String_map.empty;
       }
        : save51_member0_checkpoint_state_map_modifiers);
    lighting =
      { wire = empty_wire; time = Migration_stacking_lighting.time value.map_id value.map_revision };
    vision_groups = convert_3473_3857 value.vision_groups;
    factions = value.factions;
    ground_items = convert_3636_4035 value.ground_items;
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
    tiles = convert_3679_4081 value.tiles;
    units = convert_3701_4102 value.units;
    campaign = convert_3754_4155 value.campaign;
    last_combat = value.last_combat;
  }

and convert_3473_3857 (value : save41_member0_checkpoint_state_vision_groups) :
    save51_member0_checkpoint_state_vision_groups =
  Array.map (fun v -> convert_3457_3847 v) value

and convert_3636_4035 (value : save50_member0_checkpoint_state_ground_items) :
    save51_member0_checkpoint_state_ground_items =
  Array.map (fun v -> convert_3566_3950 v) value

and convert_3679_4081 (value : save50_member0_checkpoint_state_tiles) :
    save51_member0_checkpoint_state_tiles =
  Array.map (fun v -> convert_3648_4047 v) value

and convert_3701_4102 (value : save50_member0_checkpoint_state_units) :
    save51_member0_checkpoint_state_units =
  Array.map (fun v -> convert_3680_4082 v) value

and convert_3754_4155 (value : save50_member0_checkpoint_state_campaign) :
    save51_member0_checkpoint_state_campaign =
  match value with `Null -> `Null | `Object2 v -> `Object2 (convert_3702_4103 v)

and convert_3457_3847 (value : save41_member0_checkpoint_state_vision_groups_item) :
    save51_member0_checkpoint_state_vision_groups_item =
  {
    wire = value.wire;
    id = value.id;
    sight = convert_3458_3849 value.sight;
    explored = value.explored;
  }

and convert_3566_3950 (value : save50_member0_checkpoint_state_ground_items_item) :
    save51_member0_checkpoint_state_ground_items_item =
  { wire = value.wire; tile = value.tile; item = convert_3567_3951 value.item }

and convert_3648_4047 (value : save50_member0_checkpoint_state_tiles_item) :
    save51_member0_checkpoint_state_tiles_item =
  {
    wire = value.wire;
    effects = [||];
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
    perception = convert_3666_4066 value.perception;
    light = value.light;
  }

and convert_3680_4082 (value : save50_member0_checkpoint_state_units_item) :
    save51_member0_checkpoint_state_units_item =
  {
    wire = value.wire;
    revealed_to = value.revealed_to;
    perception_effects = convert_3688_4089 value.perception_effects;
    faction = value.faction;
    items = convert_3689_4090 value.items;
    ai_group = value.ai_group;
    id = value.id;
    type_ = value.type_;
    tile = value.tile;
    hp = value.hp;
    moved = value.moved;
    attack_spent = value.attack_spent;
    experience = value.experience;
    character = value.character;
    role = value.role;
    retreating = value.retreating;
    rebuilding = value.rebuilding;
  }

and convert_3702_4103 (value : save50_member0_checkpoint_state_campaign_member1) :
    save51_member0_checkpoint_state_campaign_member1 =
  {
    wire = value.wire;
    scripts = convert_3725_4126 value.scripts;
    evacuated = convert_3727_4128 value.evacuated;
    id = value.id;
    mission = value.mission;
    recall = value.recall;
    veterans = convert_3734_4135 value.veterans;
    story_veterans = convert_3736_4137 value.story_veterans;
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

and convert_3458_3849 (value : save41_member0_checkpoint_state_vision_groups_item_sight) :
    save51_member0_checkpoint_state_vision_groups_item_sight =
  { wire = value.wire; range = convert_3459_3850 value.range; penetrates = value.penetrates }

and convert_3567_3951 (value : save50_member0_checkpoint_state_ground_items_item_item) :
    save51_member0_checkpoint_state_ground_items_item_item =
  let sight =
    match value.perception with
    | `Null -> neutral_sight
    | `Object2 p ->
        let sight =
          match p.range with
          | `Member1 v -> (
              match v with
              | `Inherited -> neutral_sight
              | `Unlimited -> { neutral_sight with unlimited = true })
          | `Number 10. when value.id = "rl-darkvision" ->
              {
                neutral_sight with
                sight_modifiers = { wire = empty_wire; lit = 0.; unlit = 3.; darkness = 3. };
              }
          | `Number _ -> invalid_arg "Unsupported v50 finite item sight profile"
        in
        { sight with penetrates = p.penetrates }
  in
  let healing =
    match value.healing with
    | `Null -> [||]
    | `Object2 h ->
        [|
          scoped h.radius h.include_self h.targets "healing" `Max
            (`Healing
               ({ wire = empty_wire; kind = `Healing; power = h.power }
                 : save51_member0_checkpoint_state_tiles_item_effects_item_effect__member0));
        |]
  in
  let capabilities =
    match value.perception with
    | `Null -> [||]
    | `Object2 p ->
        Array.append
          (Array.map
             (fun rule ->
               scoped 0. true `Allies "concealment" `Union
                 (`Concealment
                    ({ wire = empty_wire; kind = `Concealment; rule }
                      : save51_member0_checkpoint_state_tiles_item_effects_item_effect__member3)))
             p.concealment)
          (match p.light with
          | `Null -> [||]
          | `Object2 light ->
              [|
                scoped 0. true `Allies "light" `Union
                  (`Light
                     ({ wire = empty_wire; kind = `Light; radius = light.radius }
                       : save51_member0_checkpoint_state_tiles_item_effects_item_effect__member4));
              |])
  in
  {
    wire = remove_fields value.wire [ "perception"; "healing" ];
    sight;
    effects = Array.append healing capabilities;
    armor = value.armor;
    id = value.id;
    name = value.name;
    description = value.description;
    attacks = value.attacks;
  }

and convert_3666_4066 (value : save50_member0_checkpoint_state_tiles_item_perception) :
    save51_member0_checkpoint_state_tiles_item_perception =
  {
    wire = value.wire;
    illumination = convert_3604_4067 value.illumination;
    sight = convert_3668_4069 value.sight;
    remember = value.remember;
    concealment = value.concealment;
  }

and convert_3688_4089 (value : save50_member0_checkpoint_state_units_item_perception_effects) :
    save51_member0_checkpoint_state_units_item_perception_effects =
  Array.map (fun v -> convert_3682_4084 v) value

and convert_3689_4090 (value : save50_member0_checkpoint_state_units_item_items) :
    save51_member0_checkpoint_state_units_item_items =
  Array.map (fun v -> convert_3567_3951 v) value

and convert_3725_4126 (value : save50_member0_checkpoint_state_campaign_member1_scripts) :
    save51_member0_checkpoint_state_campaign_member1_scripts =
  match value with `Null -> `Null | `Object2 v -> `Object2 (convert_3703_4104 v)

and convert_3727_4128 (value : save50_member0_checkpoint_state_campaign_member1_evacuated) :
    save51_member0_checkpoint_state_campaign_member1_evacuated =
  Array.map (fun v -> convert_3726_4127 v) value

and convert_3734_4135 (value : save50_member0_checkpoint_state_campaign_member1_veterans) :
    save51_member0_checkpoint_state_campaign_member1_veterans =
  match value with `Null -> `Null | `Array v -> `Array (convert_3733_4134 v)

and convert_3736_4137 (value : save50_member0_checkpoint_state_campaign_member1_veterans) :
    save51_member0_checkpoint_state_campaign_member1_veterans =
  match value with `Null -> `Null | `Array v -> `Array (convert_3735_4136 v)

and convert_3459_3850 (value : save41_member0_checkpoint_state_vision_groups_item_sight_range) :
    save51_member0_checkpoint_state_vision_groups_item_sight_range =
  match value with `Member1 v -> convert_3460_3850 v | `Number v -> `Number v

and convert_3604_4067 (value : save50_member0_checkpoint_state_tiles_item_perception_illumination) :
    save51_member0_checkpoint_state_tiles_item_perception_illumination =
  match value with `Lit -> `Lit | `Unlit -> `Unlit | `Darkness -> `Darkness

and convert_3668_4069 (value : save41_member0_checkpoint_state_tiles_item_perception_sight) :
    save51_member0_checkpoint_state_tiles_item_perception_sight =
  match value with `Null -> `Null | `Object2 v -> `Object2 (convert_3667_4068 v)

and convert_3682_4084 (value : save50_member0_checkpoint_state_units_item_perception_effects_item) :
    save51_member0_checkpoint_state_units_item_perception_effects_item =
  {
    wire = remove_fields value.wire [ "expiresAt" ];
    id = value.id;
    penetrates = value.penetrates;
    concealment = value.concealment;
    light = value.light;
  }

and convert_3703_4104 (value : save50_member0_checkpoint_state_campaign_member1_scripts_member1) :
    save51_member0_checkpoint_state_campaign_member1_scripts_member1 =
  {
    wire = value.wire;
    failed = value.failed;
    completed = value.completed;
    revealed = value.revealed;
    recruitment = value.recruitment;
    active = value.active;
    holds = value.holds;
    marks = convert_3715_4116 value.marks;
    pending = value.pending;
    next_mark = value.next_mark;
  }

and convert_3726_4127 (value : save50_member0_checkpoint_state_campaign_member1_evacuated_item) :
    save51_member0_checkpoint_state_campaign_member1_evacuated_item =
  { wire = value.wire; event = value.event; unit_ = convert_3680_4082 value.unit_ }

and convert_3733_4134 (value : save50_member0_checkpoint_state_campaign_member1_veterans_member1) :
    save51_member0_checkpoint_state_campaign_member1_veterans_member1 =
  Array.map (fun v -> convert_3729_4130 v) value

and convert_3735_4136 (value : save50_member0_checkpoint_state_campaign_member1_veterans_member1) :
    save51_member0_checkpoint_state_campaign_member1_veterans_member1 =
  Array.map (fun v -> convert_3729_4130 v) value

and convert_3460_3850
    (value : save41_member0_checkpoint_state_vision_groups_item_sight_range_member0) :
    save51_member0_checkpoint_state_vision_groups_item_sight_range =
  match value with
  | `Inherited -> invalid_arg "Inherited faction sight is invalid in v50"
  | `Unlimited -> `Unlimited

and convert_3667_4068 (value : save41_member0_checkpoint_state_tiles_item_perception_sight_member1)
    : save51_member0_checkpoint_state_tiles_item_perception_sight_member1 =
  { wire = value.wire; group = value.group; profile = convert_3458_3849 value.profile }

and convert_3715_4116
    (value : save50_member0_checkpoint_state_campaign_member1_scripts_member1_marks) :
    save51_member0_checkpoint_state_campaign_member1_scripts_member1_marks =
  Array.map (fun v -> convert_3712_4113 v) value

and convert_3729_4130
    (value : save50_member0_checkpoint_state_campaign_member1_veterans_member1_item) :
    save51_member0_checkpoint_state_campaign_member1_veterans_member1_item =
  {
    wire = value.wire;
    items = convert_3730_4131 value.items;
    type_ = value.type_;
    experience = value.experience;
    character = value.character;
    role = value.role;
  }

and convert_3712_4113
    (value : save50_member0_checkpoint_state_campaign_member1_scripts_member1_marks_item) :
    save51_member0_checkpoint_state_campaign_member1_scripts_member1_marks_item =
  {
    wire = value.wire;
    sight = convert_3713_4114 value.sight;
    light_remembered = value.light_remembered;
    id = value.id;
    type_ = value.type_;
    tile = value.tile;
    light = value.light;
  }

and convert_3730_4131 (value : save50_member0_checkpoint_state_units_item_items) :
    save51_member0_checkpoint_state_units_item_items =
  Array.map (fun v -> convert_3567_3951 v) value

and convert_3713_4114 (value : save41_member0_checkpoint_state_tiles_item_perception_sight) :
    save51_member0_checkpoint_state_tiles_item_perception_sight =
  match value with `Null -> `Null | `Object2 v -> `Object2 (convert_3667_4068 v)

let v50 (old : V50.t) : V51.t =
  let convert checkpoint undo =
    Migration_late.convert_state_and_undo_with_codecs
      ~encode_change:Historical_codec.V50.current_changes_item_to_json
      ~decode_change:Historical_codec.V51.current_changes_item_of_json
      ~encode_old:Historical_codec.V50.current_state_to_json
      ~decode_old:Historical_codec.V50.current_state_of_json
      ~encode_new:Historical_codec.V51.current_state_to_json convert_3456_3827 checkpoint undo
  in
  match (old, convert_3758_4159 old) with
  | `Current old, `Current value ->
      let state, undo = convert old.checkpoint.state (Some old.undo) in
      `Current { value with checkpoint = { value.checkpoint with state }; undo = Option.get undo }
  | `Turn old, `Turn value ->
      let state, _ = convert old.checkpoint.state None in
      `Turn { value with checkpoint = { value.checkpoint with state } }
  | _ -> assert false
