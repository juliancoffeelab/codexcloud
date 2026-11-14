(* Frozen v46 -> v47: make the historical primary observer explicit only on
   observer-dependent world rules. Reconstruct and convert undo through the same boundary. *)
open Historical_types

let rec needs_observer
    (condition : save40_member0_checkpoint_state_tiles_item_fog_member1_condition) =
  match condition with
  | `Visible_units _ -> true
  | `Proximity v -> v.side = `Observer
  | `Has_units v -> v.side = `Observer
  | `No_units v -> v.side = `Observer
  | `All v -> Array.exists needs_observer v.conditions
  | `Any v -> Array.exists needs_observer v.conditions
  | `Owner _ | `Choice _ | `Flag _ | `Event _ | `Round _ -> false

let rec convert_2928_2942 (value : save46) : save47 =
  match value with
  | `Current v -> `Current (convert_2929_2943 v)
  | `Turn v -> `Turn (convert_2940_2964 v)

and convert_2929_2943 (value : save46_member0) : save47_member0 =
  {
    wire = value.wire;
    pending_enemy = value.pending_enemy;
    undo = value.undo;
    progress = value.progress;
    version = convert_2930_2944 value.version;
    checkpoint = convert_2931_2945 value.checkpoint;
    setup = value.setup;
  }

and convert_2940_2964 (value : save46_member1) : save47_member1 =
  {
    wire = value.wire;
    pending_enemy = value.pending_enemy;
    undo = value.undo;
    progress = value.progress;
    version = convert_2941_2965 value.version;
    checkpoint = convert_2931_2945 value.checkpoint;
    setup = value.setup;
  }

and convert_2930_2944 (value : literal_46) : literal_47 =
  let _ = value in
  `V47

and convert_2931_2945 (value : save46_member0_checkpoint) : save47_member0_checkpoint =
  { wire = value.wire; seed = value.seed; state = convert_2932_2946 value.state }

and convert_2941_2965 (value : literal_46) : literal_47 =
  let _ = value in
  `V47

and convert_2932_2946 (value : save46_member0_checkpoint_state) : save47_member0_checkpoint_state =
  {
    wire = value.wire;
    vision_groups = value.vision_groups;
    factions = convert_2575_2947 value.factions;
    ground_items = value.ground_items;
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
    tiles = convert_2723_2952 value.tiles;
    units = value.units;
    campaign = Option.map (fun v -> convert_2936_2958 v) value.campaign;
    last_combat = value.last_combat;
  }

and convert_2575_2947 (value : save41_member0_checkpoint_state_factions) :
    save47_member0_checkpoint_state_factions =
  Array.map (fun v -> convert_2466_2948 v) value

and convert_2723_2952 (value : save44_member0_checkpoint_state_tiles) :
    save47_member0_checkpoint_state_tiles =
  Array.map (fun v -> convert_2676_2953 v) value

and convert_2936_2958 (value : save46_member0_checkpoint_state_campaign_member1) :
    save47_member0_checkpoint_state_campaign_member1 =
  {
    wire = value.wire;
    scripts = Option.map (fun v -> convert_2761_2960 v) value.scripts;
    evacuated = value.evacuated;
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

and convert_2466_2948 (value : save41_member0_checkpoint_state_factions_item) :
    save47_member0_checkpoint_state_factions_item =
  {
    wire = value.wire;
    vision_group = value.vision_group;
    id = value.id;
    phase = value.phase;
    controller = value.controller;
    palette = value.palette;
    name = value.name;
    recruits = value.recruits;
    relations = value.relations;
    transfer = Option.map (fun v -> convert_2502_2950 v) value.transfer;
  }

and convert_2676_2953 (value : save44_member0_checkpoint_state_tiles_item) :
    save47_member0_checkpoint_state_tiles_item =
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
    perception = value.perception;
    light = Option.map (fun v -> convert_2706_2955 v) value.light;
  }

and convert_2761_2960 (value : save41_member0_checkpoint_state_campaign_member1_scripts_member1) :
    save47_member0_checkpoint_state_campaign_member1_scripts_member1 =
  {
    wire = value.wire;
    failed = value.failed;
    completed = value.completed;
    revealed = value.revealed;
    recruitment = value.recruitment;
    active = value.active;
    holds = value.holds;
    marks = convert_2784_2961 value.marks;
    pending = value.pending;
    next_mark = value.next_mark;
  }

and convert_2502_2950 (value : save41_member0_checkpoint_state_factions_item_transfer_member1) :
    save47_member0_checkpoint_state_factions_item_transfer_member1 =
  { wire = value.wire; to_ = value.to_; when_ = convert_2504_2951 value.when_ }

and convert_2706_2955 (value : save41_member0_checkpoint_state_tiles_item_light_member1) :
    save47_member0_checkpoint_state_tiles_item_light_member1 =
  {
    wire = value.wire;
    emission = value.emission;
    color = value.color;
    fixture = value.fixture;
    dx = value.dx;
    dy = value.dy;
    radius = value.radius;
    power = value.power;
    when_ = Option.map (fun v -> convert_2504_2951 v) value.when_;
  }

and convert_2784_2961
    (value : save41_member0_checkpoint_state_campaign_member1_scripts_member1_marks) :
    save47_member0_checkpoint_state_campaign_member1_scripts_member1_marks =
  Array.map (fun v -> convert_2778_2962 v) value

and convert_2504_2951 (value : save41_member0_checkpoint_state_factions_item_transfer_member1_when_)
    : save47_member0_checkpoint_state_factions_item_transfer_member1_when_ =
  {
    wire = value.wire;
    condition = value.condition;
    remember = value.remember;
    observer = (if needs_observer value.condition then Some "player" else None);
  }

and convert_2778_2962
    (value : save41_member0_checkpoint_state_campaign_member1_scripts_member1_marks_item) :
    save47_member0_checkpoint_state_campaign_member1_scripts_member1_marks_item =
  {
    wire = value.wire;
    sight = value.sight;
    light_remembered = value.light_remembered;
    id = value.id;
    type_ = value.type_;
    tile = value.tile;
    light = Option.map (fun v -> convert_2706_2955 v) value.light;
  }

let v46 (old : V46.t) : V47.t =
  let convert checkpoint undo =
    Migration_late.convert_state_and_undo
      ~encode_old:Historical_codec.encode_save46_member0_checkpoint_state
      ~decode_old:Historical_codec.read_save46_member0_checkpoint_state
      ~encode_new:Historical_codec.encode_save47_member0_checkpoint_state convert_2932_2946
      checkpoint undo
  in
  match (old, convert_2928_2942 old) with
  | `Current old, `Current value ->
      let state, undo = convert old.checkpoint.state (Some old.undo) in
      `Current { value with checkpoint = { value.checkpoint with state }; undo = Option.get undo }
  | `Turn old, `Turn value ->
      let state, _ = convert old.checkpoint.state None in
      `Turn { value with checkpoint = { value.checkpoint with state } }
  | _ -> assert false
