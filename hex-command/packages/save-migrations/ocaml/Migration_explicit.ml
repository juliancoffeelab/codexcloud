(* Frozen v49 -> v50. Complete resolved fields; preserve state and undo.
   Mechanical inheritance is resolved here only from frozen v49 facts. *)
open Historical_types

[@@@warning "+8+9"]
[@@@warnerror "+8+9"]

let rec convert_3449_3758 ambient (value : save49) : save50 =
  let _ = ambient in
  match value with
  | `Current v -> `Current (convert_3445_3759 ambient v)
  | `Turn v -> `Turn (convert_3448_3817 ambient v)

and convert_3445_3759 ambient (value : save49_member0) : save50_member0 =
  let _ = ambient in
  {
    wire = value.wire;
    pending_enemy = value.pending_enemy;
    undo = value.undo;
    progress = value.progress;
    version = convert_3441_3780 ambient value.version;
    checkpoint = convert_2985_3781 ambient value.checkpoint;
    setup = value.setup;
  }

and convert_3448_3817 ambient (value : save49_member1) : save50_member1 =
  let _ = ambient in
  {
    wire = value.wire;
    pending_enemy = value.pending_enemy;
    undo = value.undo;
    progress = value.progress;
    version = convert_3441_3780 ambient value.version;
    checkpoint = convert_2985_3781 ambient value.checkpoint;
    setup = value.setup;
  }

and convert_3441_3780 ambient (value : literal_49) : literal_50 =
  let _ = ambient in
  let _ = value in
  `V50

and convert_2985_3781 ambient (value : save48_member0_checkpoint) : save50_member0_checkpoint =
  let _ = ambient in
  { wire = value.wire; seed = value.seed; state = convert_2987_3456 ambient value.state }

and convert_2987_3456 ambient (value : save48_member0_checkpoint_state) :
    save50_member0_checkpoint_state =
  let _ = ambient in
  let ambient = Migration_explicit_lighting.ambient value.map_id value.map_revision in
  let converted =
    {
      wire = value.wire;
      vision_groups = value.vision_groups;
      factions = convert_3006_3565 ambient value.factions;
      ground_items = convert_3117_3636 ambient value.ground_items;
      armies = (match value.armies with None -> `Null | Some v -> convert_3194_3638 ambient v);
      condition_memory = (match value.condition_memory with None -> [||] | Some v -> v);
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
      tiles = convert_3214_3679 ambient value.tiles;
      units = convert_3263_3701 ambient value.units;
      campaign = (match value.campaign with None -> `Null | Some v -> convert_3297_3754 ambient v);
      last_combat =
        (match value.last_combat with None -> `Null | Some v -> convert_3394_3757 ambient v);
    }
  in
  let factions =
    Array.map
      (fun (faction : save50_member0_checkpoint_state_factions_item) ->
        let relations =
          Array.fold_left
            (fun relations (other : save50_member0_checkpoint_state_factions_item) ->
              let relation =
                if faction.id = other.id then `Allied
                else
                  match String_map.find_opt other.id faction.relations with
                  | Some value -> value
                  | None -> (
                      match String_map.find_opt faction.id other.relations with
                      | Some value -> value
                      | None -> `Neutral)
              in
              String_map.add other.id relation relations)
            String_map.empty converted.factions
        in
        { faction with relations })
      converted.factions
  in
  { converted with factions }

and convert_3006_3565 ambient (value : save47_member0_checkpoint_state_factions) :
    save50_member0_checkpoint_state_factions =
  let _ = ambient in
  Array.map (fun v -> convert_3007_3474 ambient v) value

and convert_3117_3636 ambient (value : save48_member0_checkpoint_state_ground_items) :
    save50_member0_checkpoint_state_ground_items =
  let _ = ambient in
  Array.map (fun v -> convert_3118_3566 ambient v) value

and convert_3194_3638 ambient (value : fragments_native_state38_armies_member1) :
    save50_member0_checkpoint_state_armies =
  let _ = ambient in
  `Tuple value

and convert_3214_3679 ambient (value : save48_member0_checkpoint_state_tiles) :
    save50_member0_checkpoint_state_tiles =
  let _ = ambient in
  Array.map (fun v -> convert_3215_3648 ambient v) value

and convert_3263_3701 ambient (value : save48_member0_checkpoint_state_units) :
    save50_member0_checkpoint_state_units =
  let _ = ambient in
  Array.map (fun v -> convert_3264_3680 ambient v) value

and convert_3297_3754 ambient (value : save48_member0_checkpoint_state_campaign_member1) :
    save50_member0_checkpoint_state_campaign =
  let _ = ambient in
  `Object2 (convert_3297_3702 ambient value)

and convert_3394_3757 ambient (value : fragments_native_state38_last_combat) : shape_735 =
  let _ = ambient in
  match value with `Undefined -> `Null | `Null -> `Null | `Object3 v -> `Object2 v

and convert_3007_3474 ambient (value : save47_member0_checkpoint_state_factions_item) :
    save50_member0_checkpoint_state_factions_item =
  let _ = ambient in
  {
    wire = value.wire;
    vision_group = value.vision_group;
    id = value.id;
    phase = value.phase;
    controller = convert_3013_3479 ambient value.controller;
    palette = value.palette;
    name = (match value.name with None -> `Null | Some v -> convert_3030_3496 ambient v);
    recruits = (match value.recruits with None -> `Null | Some v -> convert_3032_3500 ambient v);
    relations = (match value.relations with None -> String_map.empty | Some v -> v);
    transfer = (match value.transfer with None -> `Null | Some v -> convert_3044_3564 ambient v);
  }

and convert_3118_3566 ambient (value : save48_member0_checkpoint_state_ground_items_item) :
    save50_member0_checkpoint_state_ground_items_item =
  let _ = ambient in
  { wire = value.wire; tile = value.tile; item = convert_3119_3567 ambient value.item }

and convert_3215_3648 ambient (value : save48_member0_checkpoint_state_tiles_item) :
    save50_member0_checkpoint_state_tiles_item =
  let _ = ambient in
  let ambient _ _ = ambient value.col value.row in
  {
    wire = value.wire;
    id = value.id;
    location = (match value.location with None -> `Null | Some v -> convert_3218_3649 ambient v);
    col = value.col;
    row = value.row;
    terrain = value.terrain;
    owner = value.owner;
    connections =
      (match value.connections with None -> `Null | Some v -> convert_3225_3660 ambient v);
    dead_ends = (match value.dead_ends with None -> `Null | Some v -> convert_3234_3662 ambient v);
    walls = (match value.walls with None -> `Null | Some v -> convert_3236_3664 ambient v);
    floor = (match value.floor with None -> `Null | Some v -> convert_3238_3665 ambient v);
    perception = convert_3239_3666 ambient value.perception;
    light = (match value.light with None -> `Null | Some v -> convert_3247_3678 ambient v);
  }

and convert_3264_3680 ambient (value : save48_member0_checkpoint_state_units_item) :
    save50_member0_checkpoint_state_units_item =
  let _ = ambient in
  {
    wire = value.wire;
    revealed_to = value.revealed_to;
    perception_effects = convert_3267_3688 ambient value.perception_effects;
    faction = value.faction;
    items = (match value.items with None -> [||] | Some v -> convert_3278_3689 ambient v);
    ai_group = (match value.ai_group with None -> `Null | Some v -> convert_3280_3690 ambient v);
    id = value.id;
    type_ = value.type_;
    tile = value.tile;
    hp = value.hp;
    moved = value.moved;
    attack_spent = value.attack_spent;
    experience =
      (match value.experience with None -> `Null | Some v -> convert_3287_3691 ambient v);
    character = (match value.character with None -> `Null | Some v -> convert_3116_3692 ambient v);
    role = (match value.role with None -> `Null | Some v -> convert_3288_3698 ambient v);
    retreating =
      (match value.retreating with None -> `Null | Some v -> convert_3293_3699 ambient v);
    rebuilding =
      (match value.rebuilding with None -> `Null | Some v -> convert_3295_3700 ambient v);
  }

and convert_3297_3702 ambient (value : save48_member0_checkpoint_state_campaign_member1) :
    save50_member0_checkpoint_state_campaign_member1 =
  let _ = ambient in
  {
    wire = value.wire;
    scripts = (match value.scripts with None -> `Null | Some v -> convert_3299_3725 ambient v);
    evacuated = (match value.evacuated with None -> [||] | Some v -> convert_3339_3727 ambient v);
    id = value.id;
    mission = value.mission;
    recall = value.recall;
    veterans =
      (match match value.veterans with None -> `Null | Some v -> convert_3347_3734 ambient v with
      | `Null ->
          `Array
            (Array.map
               (fun unit_type ->
                 ({
                    wire = empty_wire;
                    items = [||];
                    type_ = unit_type;
                    experience = 0.;
                    character = `Null;
                    role = `Null;
                  }
                   : save50_member0_checkpoint_state_campaign_member1_veterans_member1_item))
               value.recall)
      | `Array values -> `Array values);
    story_veterans =
      (match
         match value.story_veterans with None -> `Null | Some v -> convert_3357_3736 ambient v
       with
      | `Null -> `Array [||]
      | `Array values -> `Array values);
    fallen_companions =
      (match
         match value.fallen_companions with None -> `Null | Some v -> convert_3359_3738 ambient v
       with
      | `Null -> `Array [||]
      | `Array values -> `Array values);
    carry_gold = value.carry_gold;
    marks = value.marks;
    dialogue = convert_3364_3743 ambient value.dialogue;
    dialogue_index = value.dialogue_index;
    outcome = value.outcome;
    completed = value.completed;
    refuge = (match value.refuge with None -> `Null | Some v -> convert_3378_3746 ambient v);
    vigil = (match value.vigil with None -> `Null | Some v -> convert_3383_3748 ambient v);
    holds = (match value.holds with None -> `Null | Some v -> convert_3387_3750 ambient v);
    flags = value.flags;
  }

and convert_3013_3479 ambient (value : fragments_factions7_item_controller) :
    save50_member0_checkpoint_state_factions_item_controller =
  let _ = ambient in
  match value with
  | `Human v -> `Human v
  | `Passive v -> `Passive v
  | `Ai v -> `Ai (convert_3018_3494 ambient v)

and convert_3030_3496 ambient (value : json_string) : fragments_tiles11_item_owner =
  let _ = ambient in
  `String value

and convert_3032_3500 ambient (value : fragments_factions7_item_recruits_member1) :
    save50_member0_checkpoint_state_factions_item_recruits =
  let _ = ambient in
  `Array (convert_3032_3499 ambient value)

and convert_3044_3564 ambient
    (value : save47_member0_checkpoint_state_factions_item_transfer_member1) :
    save50_member0_checkpoint_state_factions_item_transfer =
  let _ = ambient in
  `Object2 (convert_3044_3507 ambient value)

and convert_3119_3567 ambient (value : save48_member0_checkpoint_state_ground_items_item_item) :
    save50_member0_checkpoint_state_ground_items_item_item =
  let _ = ambient in
  {
    wire = value.wire;
    perception =
      (match value.perception with None -> `Null | Some v -> convert_3121_3614 ambient v);
    armor = (match value.armor with None -> `Null | Some v -> convert_3165_3616 ambient v);
    id = value.id;
    name = value.name;
    description = value.description;
    attacks = value.attacks;
    healing = (match value.healing with None -> `Null | Some v -> convert_3190_3635 ambient v);
  }

and convert_3218_3649 ambient (value : json_string) : fragments_tiles11_item_owner =
  let _ = ambient in
  `String value

and convert_3225_3660 ambient (value : fragments_tiles11_item_connections_member1) :
    save50_member0_checkpoint_state_tiles_item_connections =
  let _ = ambient in
  `Array value

and convert_3234_3662 ambient (value : fragments_tiles11_item_connections_member1) :
    save50_member0_checkpoint_state_tiles_item_connections =
  let _ = ambient in
  `Array value

and convert_3236_3664 ambient (value : fragments_tiles11_item_connections_member1) :
    save50_member0_checkpoint_state_tiles_item_connections =
  let _ = ambient in
  `Array value

and convert_3238_3665 ambient (value : json_string) : fragments_tiles11_item_owner =
  let _ = ambient in
  `String value

and convert_3239_3666 ambient (value : save48_member0_checkpoint_state_tiles_item_perception) :
    save50_member0_checkpoint_state_tiles_item_perception =
  let _ = ambient in
  {
    wire = value.wire;
    illumination = ambient 0. 0.;
    sight = value.sight;
    remember = value.remember;
    concealment = convert_3244_3669 ambient value.concealment;
  }

and convert_3247_3678 ambient (value : save47_member0_checkpoint_state_tiles_item_light_member1) :
    save50_member0_checkpoint_state_tiles_item_light =
  let _ = ambient in
  `Object2 (convert_3247_3670 ambient value)

and convert_3267_3688 ambient
    (value : save48_member0_checkpoint_state_units_item_perception_effects) :
    save50_member0_checkpoint_state_units_item_perception_effects =
  let _ = ambient in
  Array.map (fun v -> convert_3268_3682 ambient v) value

and convert_3278_3689 ambient (value : save48_member0_checkpoint_state_units_item_items_member1) :
    save50_member0_checkpoint_state_units_item_items =
  let _ = ambient in
  Array.map (fun v -> convert_3119_3567 ambient v) value

and convert_3280_3690 ambient (value : json_string) : fragments_tiles11_item_owner =
  let _ = ambient in
  `String value

and convert_3287_3691 ambient (value : json_number) : history5_entries_item_mission =
  let _ = ambient in
  `Number value

and convert_3116_3692 ambient (value : json_string) : fragments_tiles11_item_owner =
  let _ = ambient in
  `String value

and convert_3288_3698 ambient (value : fragments_native_state38_units_item_role) :
    save41_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item_units_item_role =
  let _ = ambient in
  match value with
  | `Undefined -> `Null
  | `Leader -> `Leader
  | `Courier -> `Courier
  | `Captive -> `Captive

and convert_3293_3699 ambient (value : json_boolean) :
    save50_member0_checkpoint_state_units_item_retreating =
  let _ = ambient in
  `Boolean value

and convert_3295_3700 ambient (value : json_boolean) :
    save50_member0_checkpoint_state_units_item_retreating =
  let _ = ambient in
  `Boolean value

and convert_3299_3725 ambient
    (value : save47_member0_checkpoint_state_campaign_member1_scripts_member1) :
    save50_member0_checkpoint_state_campaign_member1_scripts =
  let _ = ambient in
  `Object2 (convert_3299_3703 ambient value)

and convert_3339_3727 ambient
    (value : save48_member0_checkpoint_state_campaign_member1_evacuated_member1) :
    save50_member0_checkpoint_state_campaign_member1_evacuated =
  let _ = ambient in
  Array.map (fun v -> convert_3340_3726 ambient v) value

and convert_3347_3734 ambient
    (value : save48_member0_checkpoint_state_campaign_member1_veterans_member1) :
    save50_member0_checkpoint_state_campaign_member1_veterans =
  let _ = ambient in
  `Array (convert_3347_3733 ambient value)

and convert_3357_3736 ambient
    (value : save48_member0_checkpoint_state_campaign_member1_veterans_member1) :
    save50_member0_checkpoint_state_campaign_member1_veterans =
  let _ = ambient in
  `Array (convert_3357_3735 ambient value)

and convert_3359_3738 ambient (value : fragments_native_state38_log) :
    save50_member0_checkpoint_state_campaign_member1_fallen_companions =
  let _ = ambient in
  `Array value

and convert_3364_3743 ambient (value : save41_member0_checkpoint_state_campaign_member1_dialogue) :
    save50_member0_checkpoint_state_campaign_member1_dialogue =
  let _ = ambient in
  Array.map (fun v -> convert_3365_3740 ambient v) value

and convert_3378_3746 ambient (value : fragments_native_state38_campaign_member1_refuge_member1) :
    save50_member0_checkpoint_state_campaign_member1_refuge =
  let _ = ambient in
  `Object2 value

and convert_3383_3748 ambient (value : fragments_native_state38_campaign_member1_vigil_member1) :
    save50_member0_checkpoint_state_campaign_member1_vigil =
  let _ = ambient in
  `Object2 value

and convert_3387_3750 ambient (value : fragments_native_state38_campaign_member1_holds_member1) :
    save50_member0_checkpoint_state_campaign_member1_holds =
  let _ = ambient in
  `Object2 value

and convert_3018_3494 ambient (value : fragments_factions7_item_controller_member2) :
    save50_member0_checkpoint_state_factions_item_controller_member2 =
  let _ = ambient in
  {
    wire = value.wire;
    kind = value.kind;
    policy = (match value.policy with None -> `Null | Some v -> convert_3020_3493 ambient v);
  }

and convert_3032_3499 ambient (value : fragments_factions7_item_recruits_member1) :
    save50_member0_checkpoint_state_factions_item_recruits_member1 =
  let _ = ambient in
  Array.map (fun v -> convert_3033_3497 ambient v) value

and convert_3044_3507 ambient
    (value : save47_member0_checkpoint_state_factions_item_transfer_member1) :
    save50_member0_checkpoint_state_factions_item_transfer_member1 =
  let _ = ambient in
  { wire = value.wire; to_ = value.to_; when_ = convert_3046_3508 ambient value.when_ }

and convert_3121_3614 ambient
    (value : save48_member0_checkpoint_state_ground_items_item_item_perception_member1) :
    save50_member0_checkpoint_state_ground_items_item_item_perception =
  let _ = ambient in
  `Object2 (convert_3121_3568 ambient value)

and convert_3165_3616 ambient
    (value : fragments_native_state38_ground_items_item_item_armor_member1) :
    save50_member0_checkpoint_state_ground_items_item_item_armor =
  let _ = ambient in
  `Object2 value

and convert_3190_3635 ambient
    (value : fragments_native_state38_ground_items_item_item_healing_member1) :
    save50_member0_checkpoint_state_ground_items_item_item_healing =
  let _ = ambient in
  `Object2 (convert_3190_3628 ambient value)

and convert_3244_3669 ambient
    (value : save48_member0_checkpoint_state_tiles_item_perception_concealment) :
    save50_member0_checkpoint_state_tiles_item_perception_concealment =
  let _ = ambient in
  Array.map (fun v -> convert_3245_3571 ambient v) value

and convert_3247_3670 ambient (value : save47_member0_checkpoint_state_tiles_item_light_member1) :
    save50_member0_checkpoint_state_tiles_item_light_member1 =
  let _ = ambient in
  {
    wire = value.wire;
    emission = value.emission;
    color = (match value.color with None -> `Null | Some v -> convert_3250_3673 ambient v);
    fixture = (match value.fixture with None -> `Null | Some v -> convert_3255_3674 ambient v);
    dx = (match value.dx with None -> `Null | Some v -> convert_3257_3675 ambient v);
    dy = (match value.dy with None -> `Null | Some v -> convert_3259_3676 ambient v);
    radius = value.radius;
    power = value.power;
    when_ = (match value.when_ with None -> `Null | Some v -> convert_3046_3677 ambient v);
  }

and convert_3268_3682 ambient
    (value : save48_member0_checkpoint_state_units_item_perception_effects_item) :
    save50_member0_checkpoint_state_units_item_perception_effects_item =
  let _ = ambient in
  {
    wire = value.wire;
    id = value.id;
    penetrates = value.penetrates;
    concealment = convert_3271_3684 ambient value.concealment;
    light = value.light;
    expires_at = value.expires_at;
  }

and convert_3299_3703 ambient
    (value : save47_member0_checkpoint_state_campaign_member1_scripts_member1) :
    save50_member0_checkpoint_state_campaign_member1_scripts_member1 =
  let _ = ambient in
  {
    wire = value.wire;
    failed = value.failed;
    completed = value.completed;
    revealed = value.revealed;
    recruitment = convert_3306_3708 ambient value.recruitment;
    active =
      (let active = value.active in
       if Array.exists (( = ) "company-survival") active then active
       else Array.append active [| "company-survival" |]);
    holds = value.holds;
    marks = convert_3316_3715 ambient value.marks;
    pending = value.pending;
    next_mark = value.next_mark;
  }

and convert_3340_3726 ambient
    (value : save48_member0_checkpoint_state_campaign_member1_evacuated_member1_item) :
    save50_member0_checkpoint_state_campaign_member1_evacuated_item =
  let _ = ambient in
  { wire = value.wire; event = value.event; unit_ = convert_3264_3680 ambient value.unit_ }

and convert_3347_3733 ambient
    (value : save48_member0_checkpoint_state_campaign_member1_veterans_member1) :
    save50_member0_checkpoint_state_campaign_member1_veterans_member1 =
  let _ = ambient in
  Array.map (fun v -> convert_3348_3729 ambient v) value

and convert_3357_3735 ambient
    (value : save48_member0_checkpoint_state_campaign_member1_veterans_member1) :
    save50_member0_checkpoint_state_campaign_member1_veterans_member1 =
  let _ = ambient in
  Array.map (fun v -> convert_3348_3729 ambient v) value

and convert_3365_3740 ambient
    (value : save41_member0_checkpoint_state_campaign_member1_dialogue_item) :
    save50_member0_checkpoint_state_campaign_member1_dialogue_item =
  let _ = ambient in
  {
    wire = value.wire;
    sound = (match value.sound with None -> `Null | Some v -> convert_3367_3741 ambient v);
    speaker = value.speaker;
    character = (match value.character with None -> `Null | Some v -> convert_3370_3742 ambient v);
    portrait = value.portrait;
    text = value.text;
  }

and convert_3020_3493 ambient (value : fragments_factions7_item_controller_member2_policy) :
    save50_member0_checkpoint_state_factions_item_controller_member2_policy =
  let _ = ambient in
  match value with
  | `Undefined -> `Null
  | `Classic -> `Classic
  | `Rush -> `Rush
  | `Balanced -> `Balanced
  | `Rotated -> `Rotated
  | `Situational -> `Situational
  | `Predictor -> `Predictor

and convert_3033_3497 ambient (value : fragments_factions7_item_recruits_member1_item) :
    save50_member0_checkpoint_state_factions_item_recruits_member1_item =
  let _ = ambient in
  {
    wire = value.wire;
    unit_ = value.unit_;
    cost = (match value.cost with None -> `Null | Some v -> convert_3036_3498 ambient v);
  }

and convert_3046_3508 ambient
    (value : save47_member0_checkpoint_state_factions_item_transfer_member1_when_) :
    save50_member0_checkpoint_state_factions_item_transfer_member1_when_ =
  let _ = ambient in
  {
    wire = value.wire;
    observer = (match value.observer with None -> `Null | Some v -> convert_3116_3509 ambient v);
    condition = convert_3047_3510 ambient value.condition;
    remember = value.remember;
  }

and convert_3121_3568 ambient
    (value : save48_member0_checkpoint_state_ground_items_item_item_perception_member1) :
    save50_member0_checkpoint_state_ground_items_item_item_perception_member1 =
  let _ = ambient in
  {
    wire = value.wire;
    range = value.range;
    penetrates = value.penetrates;
    concealment = convert_3123_3611 ambient value.concealment;
    light = value.light;
  }

and convert_3190_3628 ambient
    (value : fragments_native_state38_ground_items_item_item_healing_member1) :
    save50_member0_checkpoint_state_ground_items_item_item_healing_member1 =
  let _ = ambient in
  {
    wire = value.wire;
    power = value.amount;
    radius = value.radius;
    include_self = true;
    targets = `Allies;
  }

and convert_3245_3571 ambient
    (value : save48_member0_checkpoint_state_tiles_item_perception_concealment_item) :
    save50_member0_checkpoint_state_tiles_item_perception_concealment_item =
  let _ = ambient in
  {
    wire = value.wire;
    kind = value.kind;
    reveal_within = value.reveal_within;
    requires = convert_3126_3572 ambient value.requires;
  }

and convert_3250_3673 ambient (value : fragments_light23_color_member1) :
    save50_member0_checkpoint_state_tiles_item_light_member1_color =
  let _ = ambient in
  `Tuple value

and convert_3255_3674 ambient (value : json_boolean) :
    save50_member0_checkpoint_state_units_item_retreating =
  let _ = ambient in
  `Boolean value

and convert_3257_3675 ambient (value : json_number) : history5_entries_item_mission =
  let _ = ambient in
  `Number value

and convert_3259_3676 ambient (value : json_number) : history5_entries_item_mission =
  let _ = ambient in
  `Number value

and convert_3046_3677 ambient
    (value : save47_member0_checkpoint_state_factions_item_transfer_member1_when_) :
    save50_member0_checkpoint_state_tiles_item_light_member1_when_ =
  let _ = ambient in
  `Object2 (convert_3046_3508 ambient value)

and convert_3271_3684 ambient
    (value : save48_member0_checkpoint_state_tiles_item_perception_concealment) :
    save50_member0_checkpoint_state_tiles_item_perception_concealment =
  let _ = ambient in
  Array.map (fun v -> convert_3124_3570 ambient v) value

and convert_3306_3708 ambient
    (value : fragments_native_state38_campaign_member1_scripts_member1_recruitment) :
    save50_member0_checkpoint_state_campaign_member1_scripts_member1_recruitment =
  let _ = ambient in
  String_map.map (fun v -> convert_3307_3707 ambient v) value

and convert_3316_3715 ambient
    (value : save47_member0_checkpoint_state_campaign_member1_scripts_member1_marks) :
    save50_member0_checkpoint_state_campaign_member1_scripts_member1_marks =
  let _ = ambient in
  Array.map (fun v -> convert_3317_3712 ambient v) value

and convert_3348_3729 ambient
    (value : save48_member0_checkpoint_state_campaign_member1_veterans_member1_item) :
    save50_member0_checkpoint_state_campaign_member1_veterans_member1_item =
  let _ = ambient in
  {
    wire = value.wire;
    items = (match value.items with None -> [||] | Some v -> convert_3350_3730 ambient v);
    type_ = value.type_;
    experience = value.experience;
    character = (match value.character with None -> `Null | Some v -> convert_3354_3731 ambient v);
    role = (match value.role with None -> `Null | Some v -> convert_3355_3732 ambient v);
  }

and convert_3367_3741 ambient (value : json_string) : fragments_tiles11_item_owner =
  let _ = ambient in
  `String value

and convert_3370_3742 ambient (value : json_string) : fragments_tiles11_item_owner =
  let _ = ambient in
  `String value

and convert_3036_3498 ambient (value : json_number) : history5_entries_item_mission =
  let _ = ambient in
  `Number value

and convert_3116_3509 ambient (value : json_string) : fragments_tiles11_item_owner =
  let _ = ambient in
  `String value

and convert_3047_3510 ambient
    (value : save40_member0_checkpoint_state_tiles_item_fog_member1_condition) :
    save50_member0_checkpoint_state_factions_item_transfer_member1_when__condition =
  let _ = ambient in
  match value with
  | `Owner v -> `Owner (convert_3048_3523 ambient v)
  | `Proximity v -> `Proximity (convert_3060_3536 ambient v)
  | `Visible_units v -> `Visible_units (convert_3077_3539 ambient v)
  | `Has_units v -> `Has_units (convert_3082_3544 ambient v)
  | `No_units v -> `No_units (convert_3090_3548 ambient v)
  | `Choice v -> `Choice v
  | `Flag v -> `Flag v
  | `Event v -> `Event v
  | `Round v -> `Round v
  | `All v -> `All (convert_3108_3559 ambient v)
  | `Any v -> `Any (convert_3111_3562 ambient v)

and convert_3123_3611 ambient
    (value : save48_member0_checkpoint_state_tiles_item_perception_concealment) :
    save50_member0_checkpoint_state_tiles_item_perception_concealment =
  let _ = ambient in
  Array.map (fun v -> convert_3124_3570 ambient v) value

and convert_3126_3572 ambient
    (value : save48_member0_checkpoint_state_tiles_item_perception_concealment_item_requires) :
    save50_member0_checkpoint_state_tiles_item_perception_concealment_item_requires =
  let _ = ambient in
  {
    wire = value.wire;
    terrains = value.terrains;
    illumination = convert_3158_3610 ambient value.illumination;
  }

and convert_3124_3570 ambient
    (value : save48_member0_checkpoint_state_tiles_item_perception_concealment_item) :
    save50_member0_checkpoint_state_tiles_item_perception_concealment_item =
  let _ = ambient in
  {
    wire = value.wire;
    kind = value.kind;
    reveal_within = value.reveal_within;
    requires = convert_3126_3572 ambient value.requires;
  }

and convert_3307_3707 ambient (value : fragments_factions7_item_recruits_member1) :
    save50_member0_checkpoint_state_factions_item_recruits_member1 =
  let _ = ambient in
  Array.map (fun v -> convert_3033_3497 ambient v) value

and convert_3317_3712 ambient
    (value : save47_member0_checkpoint_state_campaign_member1_scripts_member1_marks_item) :
    save50_member0_checkpoint_state_campaign_member1_scripts_member1_marks_item =
  let _ = ambient in
  {
    wire = value.wire;
    sight = value.sight;
    light_remembered = value.light_remembered;
    id = value.id;
    type_ = value.type_;
    tile = value.tile;
    light = (match value.light with None -> `Null | Some v -> convert_3247_3714 ambient v);
  }

and convert_3350_3730 ambient (value : save48_member0_checkpoint_state_units_item_items_member1) :
    save50_member0_checkpoint_state_units_item_items =
  let _ = ambient in
  Array.map (fun v -> convert_3119_3567 ambient v) value

and convert_3354_3731 ambient (value : json_string) : fragments_tiles11_item_owner =
  let _ = ambient in
  `String value

and convert_3355_3732 ambient (value : fragments_native_state38_units_item_role) :
    save41_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item_units_item_role =
  let _ = ambient in
  match value with
  | `Undefined -> `Null
  | `Leader -> `Leader
  | `Courier -> `Courier
  | `Captive -> `Captive

and convert_3048_3523 ambient
    (value : save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0) :
    save50_member0_checkpoint_state_factions_item_transfer_member1_when__condition_member0 =
  let _ = ambient in
  {
    wire = value.wire;
    kind = value.kind;
    tile = (match value.tile with None -> `Null | Some v -> convert_3050_3522 ambient v);
    side = value.side;
  }

and convert_3060_3536 ambient
    (value : save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member1) :
    save50_member0_checkpoint_state_factions_item_transfer_member1_when__condition_member1 =
  let _ = ambient in
  {
    wire = value.wire;
    kind = value.kind;
    unit_types = (match value.unit_types with None -> [||] | Some v -> v);
    tile = (match value.tile with None -> `Null | Some v -> convert_3065_3526 ambient v);
    radius = value.radius;
    side = value.side;
    unit_type = (match value.unit_type with None -> `Null | Some v -> convert_3074_3534 ambient v);
    character = (match value.character with None -> `Null | Some v -> convert_3076_3535 ambient v);
  }

and convert_3077_3539 ambient
    (value : save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member2) :
    save50_member0_checkpoint_state_factions_item_transfer_member1_when__condition_member2 =
  let _ = ambient in
  {
    wire = value.wire;
    kind = value.kind;
    side = value.side;
    unit_types = (match value.unit_types with None -> [||] | Some v -> v);
  }

and convert_3082_3544 ambient
    (value : save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member3) :
    save50_member0_checkpoint_state_factions_item_transfer_member1_when__condition_member3 =
  let _ = ambient in
  {
    wire = value.wire;
    kind = value.kind;
    ai_group = (match value.ai_group with None -> `Null | Some v -> convert_3085_3541 ambient v);
    side = value.side;
    unit_type = (match value.unit_type with None -> `Null | Some v -> convert_3087_3542 ambient v);
    character = (match value.character with None -> `Null | Some v -> convert_3089_3543 ambient v);
  }

and convert_3090_3548 ambient
    (value : save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member4) :
    save50_member0_checkpoint_state_factions_item_transfer_member1_when__condition_member4 =
  let _ = ambient in
  {
    wire = value.wire;
    kind = value.kind;
    side = value.side;
    unit_type = (match value.unit_type with None -> `Null | Some v -> convert_3093_3546 ambient v);
    character = (match value.character with None -> `Null | Some v -> convert_3095_3547 ambient v);
  }

and convert_3108_3559 ambient
    (value : save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member9) :
    save50_member0_checkpoint_state_factions_item_transfer_member1_when__condition_member9 =
  let _ = ambient in
  { wire = value.wire; kind = value.kind; conditions = convert_3110_3558 ambient value.conditions }

and convert_3111_3562 ambient
    (value : save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member10) :
    save50_member0_checkpoint_state_factions_item_transfer_member1_when__condition_member10 =
  let _ = ambient in
  { wire = value.wire; kind = value.kind; conditions = convert_3113_3561 ambient value.conditions }

and convert_3158_3610 ambient
    (value :
      save44_member0_checkpoint_state_tiles_item_perception_concealment_item_requires_illumination)
    : save50_member0_checkpoint_state_tiles_item_perception_concealment_item_requires_illumination =
  let _ = ambient in
  match value with
  | `Null -> `Null
  | `Lit -> `Array [| `Lit |]
  | `Unlit -> `Array [| `Unlit; `Darkness |]

and convert_3247_3714 ambient (value : save47_member0_checkpoint_state_tiles_item_light_member1) :
    save50_member0_checkpoint_state_tiles_item_light =
  let _ = ambient in
  `Object2 (convert_3247_3670 ambient value)

and convert_3050_3522 ambient
    (value : save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile) :
    save50_member0_checkpoint_state_factions_item_transfer_member1_when__condition_member0_tile =
  let _ = ambient in
  match value with
  | `Undefined -> `Null
  | `Location v -> `Location v
  | `Object3 v -> `Object3 v
  | `Object4 v -> `Object4 v

and convert_3065_3526 ambient
    (value : save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member0_tile) :
    save50_member0_checkpoint_state_factions_item_transfer_member1_when__condition_member0_tile =
  let _ = ambient in
  match value with
  | `Undefined -> `Null
  | `Location v -> `Location v
  | `Object3 v -> `Object3 v
  | `Object4 v -> `Object4 v

and convert_3074_3534 ambient (value : json_string) : fragments_tiles11_item_owner =
  let _ = ambient in
  `String value

and convert_3076_3535 ambient (value : json_string) : fragments_tiles11_item_owner =
  let _ = ambient in
  `String value

and convert_3085_3541 ambient (value : json_string) : fragments_tiles11_item_owner =
  let _ = ambient in
  `String value

and convert_3087_3542 ambient (value : json_string) : fragments_tiles11_item_owner =
  let _ = ambient in
  `String value

and convert_3089_3543 ambient (value : json_string) : fragments_tiles11_item_owner =
  let _ = ambient in
  `String value

and convert_3093_3546 ambient (value : json_string) : fragments_tiles11_item_owner =
  let _ = ambient in
  `String value

and convert_3095_3547 ambient (value : json_string) : fragments_tiles11_item_owner =
  let _ = ambient in
  `String value

and convert_3110_3558 ambient
    (value : save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member9_conditions) :
    save50_member0_checkpoint_state_factions_item_transfer_member1_when__condition_member9_conditions
    =
  let _ = ambient in
  Array.map (fun v -> convert_3047_3510 ambient v) value

and convert_3113_3561 ambient
    (value : save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member9_conditions) :
    save50_member0_checkpoint_state_factions_item_transfer_member1_when__condition_member9_conditions
    =
  let _ = ambient in
  Array.map (fun v -> convert_3047_3510 ambient v) value

let v49 (old : V49.t) : V50.t =
  let ambient _ _ = invalid_arg "Lighting conversion requires a saved map" in
  let convert checkpoint undo =
    Migration_late.convert_state_and_undo_with_codecs
      ~encode_change:Historical_codec.V49.current_changes_item_to_json
      ~decode_change:Historical_codec.V50.current_changes_item_of_json
      ~encode_old:Historical_codec.V49.current_state_to_json
      ~decode_old:Historical_codec.V49.current_state_of_json
      ~encode_new:Historical_codec.V50.current_state_to_json (convert_2987_3456 ambient) checkpoint
      undo
  in
  match (old, convert_3449_3758 ambient old) with
  | `Current old, `Current value ->
      let state, undo = convert old.checkpoint.state (Some old.undo) in
      `Current { value with checkpoint = { value.checkpoint with state }; undo = Option.get undo }
  | `Turn old, `Turn value ->
      let state, _ = convert old.checkpoint.state None in
      `Turn { value with checkpoint = { value.checkpoint with state } }
  | _ -> assert false
