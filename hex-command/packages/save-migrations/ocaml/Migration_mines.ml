(* Frozen v47 -> v48: rename the mountain mine and repair the ordinary lair opening.
   Preserve resolved RL terrain; new cave geometry applies to new battles only. *)
open Historical_types

(* These scenario-only leader labels were never campaign characters. *)
let arena_maps =
  [
    "rl-meadow";
    "rl-woodland";
    "rl-ridge";
    "rl-wetland";
    "rl-river";
    "rl-islands";
    "rl-cavern";
    "rl-ruins";
    "rl-broadfield";
    "rl-floodplain";
  ]

let rec convert_2942_2966 (value : save47) : save48 =
  match value with
  | `Current v -> `Current (convert_2943_2967 v)
  | `Turn v -> `Turn (convert_2964_3433 v)

and convert_2943_2967 (value : save47_member0) : save48_member0 =
  {
    wire = value.wire;
    pending_enemy = value.pending_enemy;
    undo = value.undo;
    progress = value.progress;
    version = convert_2944_2984 value.version;
    checkpoint = convert_2945_2985 value.checkpoint;
    setup = value.setup;
  }

and convert_2964_3433 (value : save47_member1) : save48_member1 =
  {
    wire = value.wire;
    pending_enemy = value.pending_enemy;
    undo = value.undo;
    progress = value.progress;
    version = convert_2965_3439 value.version;
    checkpoint = convert_2945_2985 value.checkpoint;
    setup = value.setup;
  }

and convert_2944_2984 (value : literal_47) : literal_48 =
  let _ = value in
  `V48

and convert_2945_2985 (value : save47_member0_checkpoint) : save48_member0_checkpoint =
  { wire = value.wire; seed = value.seed; state = convert_2946_2987 value.state }

and convert_2965_3439 (value : literal_47) : literal_48 =
  let _ = value in
  `V48

and convert_2946_2987 (value : save47_member0_checkpoint_state) : save48_member0_checkpoint_state =
  {
    wire = value.wire;
    vision_groups = value.vision_groups;
    factions = value.factions;
    ground_items = convert_2652_3117 value.ground_items;
    armies = value.armies;
    condition_memory = value.condition_memory;
    map_id = value.map_id;
    map_revision =
      (if
         value.map_revision = 1.
         && List.mem value.map_id [ "underground-lair"; "rl-cavern"; "rl-survive-1" ]
       then 2.
       else value.map_revision);
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
      (let tiles = convert_2952_3214 value.tiles in
       if value.map_id = "underground-lair" && value.map_revision = 1. then
         Array.map
           (fun (tile : save48_member0_checkpoint_state_tiles_item) ->
             if tile.col = 16. && tile.row = 23. then
               {
                 tile with
                 terrain = "cave-passage";
                 floor = Some "cave-stone";
                 connections = Some [| `E; `Se; `Sw; `Nw |];
               }
             else tile)
           tiles
       else tiles);
    units =
      (let units = convert_2933_3263 value.units in
       if Option.is_none value.campaign && List.mem value.map_id arena_maps then
         Array.map
           (fun (unit : save48_member0_checkpoint_state_units_item) ->
             if List.mem unit.character [ Some "player-leader"; Some "enemy-leader" ] then
               { unit with character = None }
             else unit)
           units
       else units);
    campaign = Option.map (fun v -> convert_2958_3297 v) value.campaign;
    last_combat = value.last_combat;
  }

and convert_2652_3117 (value : save44_member0_checkpoint_state_ground_items) :
    save48_member0_checkpoint_state_ground_items =
  Array.map (fun v -> convert_2576_3118 v) value

and convert_2952_3214 (value : save47_member0_checkpoint_state_tiles) :
    save48_member0_checkpoint_state_tiles =
  Array.map (fun v -> convert_2953_3215 v) value

and convert_2933_3263 (value : save46_member0_checkpoint_state_units) :
    save48_member0_checkpoint_state_units =
  Array.map (fun v -> convert_2934_3264 v) value

and convert_2958_3297 (value : save47_member0_checkpoint_state_campaign_member1) :
    save48_member0_checkpoint_state_campaign_member1 =
  {
    wire = value.wire;
    scripts = value.scripts;
    evacuated = Option.map (fun v -> convert_2938_3339 v) value.evacuated;
    id = value.id;
    mission = value.mission;
    recall = value.recall;
    veterans = Option.map (fun v -> convert_2817_3347 v) value.veterans;
    story_veterans = Option.map (fun v -> convert_2819_3357 v) value.story_veterans;
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

and convert_2576_3118 (value : save44_member0_checkpoint_state_ground_items_item) :
    save48_member0_checkpoint_state_ground_items_item =
  { wire = value.wire; tile = value.tile; item = convert_2577_3119 value.item }

and convert_2953_3215 (value : save47_member0_checkpoint_state_tiles_item) :
    save48_member0_checkpoint_state_tiles_item =
  {
    wire = value.wire;
    id = value.id;
    location = value.location;
    col = value.col;
    row = value.row;
    terrain = (if value.terrain = "cave" then "mountain-mine" else value.terrain);
    owner = value.owner;
    connections = value.connections;
    dead_ends = value.dead_ends;
    walls = value.walls;
    floor = value.floor;
    perception = convert_2700_3239 value.perception;
    light = value.light;
  }

and convert_2934_3264 (value : save46_member0_checkpoint_state_units_item) :
    save48_member0_checkpoint_state_units_item =
  {
    wire = value.wire;
    revealed_to = value.revealed_to;
    perception_effects = convert_2735_3267 value.perception_effects;
    faction = value.faction;
    items = Option.map (fun v -> convert_2737_3278 v) value.items;
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

and convert_2938_3339 (value : save46_member0_checkpoint_state_campaign_member1_evacuated_member1) :
    save48_member0_checkpoint_state_campaign_member1_evacuated_member1 =
  Array.map (fun v -> convert_2939_3340 v) value

and convert_2817_3347 (value : save44_member0_checkpoint_state_campaign_member1_veterans_member1) :
    save48_member0_checkpoint_state_campaign_member1_veterans_member1 =
  Array.map (fun v -> convert_2809_3348 v) value

and convert_2819_3357 (value : save44_member0_checkpoint_state_campaign_member1_veterans_member1) :
    save48_member0_checkpoint_state_campaign_member1_veterans_member1 =
  Array.map (fun v -> convert_2809_3348 v) value

and convert_2577_3119 (value : save44_member0_checkpoint_state_ground_items_item_item) :
    save48_member0_checkpoint_state_ground_items_item_item =
  {
    wire = value.wire;
    perception = Option.map (fun v -> convert_2578_3121 v) value.perception;
    armor = value.armor;
    id = value.id;
    name = value.name;
    description = value.description;
    attacks = value.attacks;
    healing = value.healing;
  }

and convert_2700_3239 (value : save44_member0_checkpoint_state_tiles_item_perception) :
    save48_member0_checkpoint_state_tiles_item_perception =
  {
    wire = value.wire;
    sight = value.sight;
    remember = value.remember;
    concealment = convert_2705_3244 value.concealment;
  }

and convert_2735_3267 (value : save44_member0_checkpoint_state_units_item_perception_effects) :
    save48_member0_checkpoint_state_units_item_perception_effects =
  Array.map (fun v -> convert_2727_3268 v) value

and convert_2737_3278 (value : save44_member0_checkpoint_state_units_item_items_member1) :
    save48_member0_checkpoint_state_units_item_items_member1 =
  Array.map (fun v -> convert_2577_3119 v) value

and convert_2939_3340
    (value : save46_member0_checkpoint_state_campaign_member1_evacuated_member1_item) :
    save48_member0_checkpoint_state_campaign_member1_evacuated_member1_item =
  { wire = value.wire; event = value.event; unit_ = convert_2934_3264 value.unit_ }

and convert_2809_3348
    (value : save44_member0_checkpoint_state_campaign_member1_veterans_member1_item) :
    save48_member0_checkpoint_state_campaign_member1_veterans_member1_item =
  {
    wire = value.wire;
    items = Option.map (fun v -> convert_2810_3350 v) value.items;
    type_ = value.type_;
    experience = value.experience;
    character = value.character;
    role = value.role;
  }

and convert_2578_3121
    (value : save44_member0_checkpoint_state_ground_items_item_item_perception_member1) :
    save48_member0_checkpoint_state_ground_items_item_item_perception_member1 =
  {
    wire = value.wire;
    range = value.range;
    penetrates = value.penetrates;
    concealment = convert_2618_3123 value.concealment;
    light = value.light;
  }

and convert_2705_3244 (value : save44_member0_checkpoint_state_tiles_item_perception_concealment) :
    save48_member0_checkpoint_state_tiles_item_perception_concealment =
  Array.map (fun v -> convert_2581_3245 v) value

and convert_2727_3268 (value : save44_member0_checkpoint_state_units_item_perception_effects_item) :
    save48_member0_checkpoint_state_units_item_perception_effects_item =
  {
    wire = value.wire;
    id = value.id;
    penetrates = value.penetrates;
    concealment = convert_2730_3271 value.concealment;
    light = value.light;
    expires_at = value.expires_at;
  }

and convert_2810_3350 (value : save44_member0_checkpoint_state_units_item_items_member1) :
    save48_member0_checkpoint_state_units_item_items_member1 =
  Array.map (fun v -> convert_2577_3119 v) value

and convert_2618_3123 (value : save44_member0_checkpoint_state_tiles_item_perception_concealment) :
    save48_member0_checkpoint_state_tiles_item_perception_concealment =
  Array.map (fun v -> convert_2580_3124 v) value

and convert_2581_3245
    (value : save44_member0_checkpoint_state_tiles_item_perception_concealment_item) :
    save48_member0_checkpoint_state_tiles_item_perception_concealment_item =
  {
    wire = value.wire;
    kind = value.kind;
    reveal_within = value.reveal_within;
    requires = convert_2583_3126 value.requires;
  }

and convert_2730_3271 (value : save44_member0_checkpoint_state_tiles_item_perception_concealment) :
    save48_member0_checkpoint_state_tiles_item_perception_concealment =
  Array.map (fun v -> convert_2580_3124 v) value

and convert_2580_3124
    (value : save44_member0_checkpoint_state_tiles_item_perception_concealment_item) :
    save48_member0_checkpoint_state_tiles_item_perception_concealment_item =
  {
    wire = value.wire;
    kind = value.kind;
    reveal_within = value.reveal_within;
    requires = convert_2583_3126 value.requires;
  }

and convert_2583_3126
    (value : save44_member0_checkpoint_state_tiles_item_perception_concealment_item_requires) :
    save48_member0_checkpoint_state_tiles_item_perception_concealment_item_requires =
  {
    wire = value.wire;
    terrains = convert_2613_3127 value.terrains;
    illumination = value.illumination;
  }

and convert_2613_3127
    (value :
      save44_member0_checkpoint_state_tiles_item_perception_concealment_item_requires_terrains) :
    save48_member0_checkpoint_state_tiles_item_perception_concealment_item_requires_terrains =
  match value with `Null -> `Null | `Array v -> `Array (convert_2612_3129 v)

and convert_2612_3129
    (value :
      save44_member0_checkpoint_state_tiles_item_perception_concealment_item_requires_terrains_member1)
    :
    save48_member0_checkpoint_state_tiles_item_perception_concealment_item_requires_terrains_member1
    =
  Array.map (fun v -> convert_2584_3130 v) value

and convert_2584_3130
    (value :
      save44_member0_checkpoint_state_tiles_item_perception_concealment_item_requires_terrains_member1_item)
    :
    save48_member0_checkpoint_state_tiles_item_perception_concealment_item_requires_terrains_member1_item
    =
  match value with
  | `Bridge -> `Bridge
  | `Burned_village -> `Burned_village
  | `Cave -> `Mountain_mine
  | `Cave_wall -> `Cave_wall
  | `Church -> `Church
  | `Ferry -> `Ferry
  | `Ford -> `Ford
  | `Forest -> `Forest
  | `Fort -> `Fort
  | `Fortification -> `Fortification
  | `Graveyard -> `Graveyard
  | `Grove -> `Grove
  | `Hill -> `Hill
  | `House -> `House
  | `Keep -> `Keep
  | `Landing -> `Landing
  | `Marsh -> `Marsh
  | `Monastery -> `Monastery
  | `Mountain -> `Mountain
  | `Plains -> `Plains
  | `Ritual_spent -> `Ritual_spent
  | `Ritual_star -> `Ritual_star
  | `Road -> `Road
  | `Ruins -> `Ruins
  | `Signpost -> `Signpost
  | `Village -> `Village
  | `Water -> `Water

let v47 (old : V47.t) : V48.t =
  let convert checkpoint undo =
    Migration_late.convert_state_and_undo
      ~encode_old:Historical_codec.encode_save47_member0_checkpoint_state
      ~decode_old:Historical_codec.read_save47_member0_checkpoint_state
      ~encode_new:Historical_codec.encode_save48_member0_checkpoint_state convert_2946_2987
      checkpoint undo
  in
  match (old, convert_2942_2966 old) with
  | `Current old, `Current value ->
      let state, undo = convert old.checkpoint.state (Some old.undo) in
      `Current { value with checkpoint = { value.checkpoint with state }; undo = Option.get undo }
  | `Turn old, `Turn value ->
      let state, _ = convert old.checkpoint.state None in
      `Turn { value with checkpoint = { value.checkpoint with state } }
  | _ -> assert false
