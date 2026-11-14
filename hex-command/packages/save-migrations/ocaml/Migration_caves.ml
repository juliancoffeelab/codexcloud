(* Frozen v42 -> v43: admit the new cave-wall terrain role. Existing facts,
   including undo patches, retain their exact wire values. *)

open Historical_types

[@@@warning "+8+9-27-39"]
[@@@warnerror "+8+9"]

let rec convert_744_748 (value : save42_member0) : save43_member0 =
  let ({ wire; pending_enemy; undo; progress; version; checkpoint; setup } : save42_member0) =
    value
  in
  {
    wire;
    pending_enemy;
    undo;
    progress;
    version = convert_743_749 version;
    checkpoint = convert_628_750 checkpoint;
    setup;
  }

and convert_745_777 (value : save42_member1) : save43_member1 =
  let ({ wire; pending_enemy; undo; progress; version; checkpoint; setup } : save42_member1) =
    value
  in
  {
    wire;
    pending_enemy;
    undo;
    progress;
    version = convert_743_749 version;
    checkpoint = convert_628_750 checkpoint;
    setup;
  }

and convert_743_749 (value : literal_42) : literal_43 =
  let `V42 = value in
  `V43

and convert_628_750 (value : save41_member0_checkpoint) : save43_member0_checkpoint =
  let ({ wire; seed; state } : save41_member0_checkpoint) = value in
  { wire; seed; state = convert_629_751 state }

and convert_629_751 (value : save41_member0_checkpoint_state) : save43_member0_checkpoint_state =
  let ({
         wire;
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
        : save41_member0_checkpoint_state) =
    value
  in
  {
    wire;
    vision_groups;
    factions;
    ground_items = convert_692_752 ground_items;
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
    units = convert_715_763 units;
    campaign = Option.map (fun campaign -> convert_716_770 campaign) campaign;
    last_combat;
  }

and convert_692_752 (value : save41_member0_checkpoint_state_ground_items) :
    save43_member0_checkpoint_state_ground_items =
  Array.map (fun value -> convert_652_753 value) value

and convert_715_763 (value : save41_member0_checkpoint_state_units) :
    save43_member0_checkpoint_state_units =
  Array.map (fun value -> convert_707_764 value) value

and convert_716_770 (value : save41_member0_checkpoint_state_campaign_member1) :
    save43_member0_checkpoint_state_campaign_member1 =
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
        : save41_member0_checkpoint_state_campaign_member1) =
    value
  in
  {
    wire;
    scripts;
    evacuated = Option.map (fun evacuated -> convert_727_772 evacuated) evacuated;
    id;
    mission;
    recall;
    veterans = Option.map (fun veterans -> convert_730_775 veterans) veterans;
    story_veterans =
      Option.map (fun story_veterans -> convert_730_775 story_veterans) story_veterans;
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

and convert_652_753 (value : save41_member0_checkpoint_state_ground_items_item) :
    save43_member0_checkpoint_state_ground_items_item =
  let ({ wire; tile; item } : save41_member0_checkpoint_state_ground_items_item) = value in
  { wire; tile; item = convert_653_754 item }

and convert_707_764 (value : save41_member0_checkpoint_state_units_item) :
    save43_member0_checkpoint_state_units_item =
  let ({
         wire;
         revealed_to;
         perception_effects;
         faction;
         items;
         ai_group;
         id;
         type_;
         side;
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
        : save41_member0_checkpoint_state_units_item) =
    value
  in
  {
    wire;
    revealed_to;
    perception_effects = convert_711_765 perception_effects;
    faction;
    items = Option.map (fun items -> convert_712_768 items) items;
    ai_group;
    id;
    type_;
    side;
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

and convert_727_772 (value : save41_member0_checkpoint_state_campaign_member1_evacuated_member1) :
    save43_member0_checkpoint_state_campaign_member1_evacuated_member1 =
  Array.map (fun value -> convert_726_773 value) value

and convert_730_775 (value : save41_member0_checkpoint_state_campaign_member1_veterans_member1) :
    save43_member0_checkpoint_state_campaign_member1_veterans_member1 =
  Array.map (fun value -> convert_729_776 value) value

and convert_653_754 (value : save41_member0_checkpoint_state_ground_items_item_item) :
    save43_member0_checkpoint_state_ground_items_item_item =
  let ({ wire; perception; armor; id; name; description; attacks; healing }
        : save41_member0_checkpoint_state_ground_items_item_item) =
    value
  in
  {
    wire;
    perception = Option.map (fun perception -> convert_654_756 perception) perception;
    armor;
    id;
    name;
    description;
    attacks;
    healing;
  }

and convert_711_765 (value : save41_member0_checkpoint_state_units_item_perception_effects) :
    save43_member0_checkpoint_state_units_item_perception_effects =
  Array.map (fun value -> convert_708_766 value) value

and convert_712_768 (value : save41_member0_checkpoint_state_units_item_items_member1) :
    save43_member0_checkpoint_state_units_item_items_member1 =
  Array.map (fun value -> convert_653_754 value) value

and convert_726_773
    (value : save41_member0_checkpoint_state_campaign_member1_evacuated_member1_item) :
    save43_member0_checkpoint_state_campaign_member1_evacuated_member1_item =
  let ({ wire; event; unit_ }
        : save41_member0_checkpoint_state_campaign_member1_evacuated_member1_item) =
    value
  in
  { wire; event; unit_ = convert_707_764 unit_ }

and convert_729_776 (value : save41_member0_checkpoint_state_campaign_member1_veterans_member1_item)
    : save43_member0_checkpoint_state_campaign_member1_veterans_member1_item =
  let ({ wire; items; type_; experience; character; role }
        : save41_member0_checkpoint_state_campaign_member1_veterans_member1_item) =
    value
  in
  {
    wire;
    items = Option.map (fun items -> convert_712_768 items) items;
    type_;
    experience;
    character;
    role;
  }

and convert_654_756
    (value : save41_member0_checkpoint_state_ground_items_item_item_perception_member1) :
    save43_member0_checkpoint_state_ground_items_item_item_perception_member1 =
  let ({ wire; range; penetrates; concealment; light }
        : save41_member0_checkpoint_state_ground_items_item_item_perception_member1) =
    value
  in
  { wire; range; penetrates; concealment = convert_686_757 concealment; light }

and convert_708_766 (value : save41_member0_checkpoint_state_units_item_perception_effects_item) :
    save43_member0_checkpoint_state_units_item_perception_effects_item =
  let ({ wire; id; penetrates; concealment; light; expires_at }
        : save41_member0_checkpoint_state_units_item_perception_effects_item) =
    value
  in
  { wire; id; penetrates; concealment = convert_686_757 concealment; light; expires_at }

and convert_686_757
    (value : save41_member0_checkpoint_state_units_item_perception_effects_item_concealment) :
    save43_member0_checkpoint_state_units_item_perception_effects_item_concealment =
  Array.map (fun value -> convert_655_758 value) value

and convert_655_758
    (value : save41_member0_checkpoint_state_units_item_perception_effects_item_concealment_item) :
    save43_member0_checkpoint_state_units_item_perception_effects_item_concealment_item =
  let ({ wire; kind; reveal_within; terrains }
        : save41_member0_checkpoint_state_units_item_perception_effects_item_concealment_item) =
    value
  in
  { wire; kind; reveal_within; terrains = convert_685_759 terrains }

and convert_685_759
    (value :
      save41_member0_checkpoint_state_units_item_perception_effects_item_concealment_item_terrains)
    : save43_member0_checkpoint_state_units_item_perception_effects_item_concealment_item_terrains =
  match value with `Null -> `Null | `Array value -> `Array (convert_684_760 value)

and convert_684_760
    (value :
      save41_member0_checkpoint_state_units_item_perception_effects_item_concealment_item_terrains_member1)
    :
    save43_member0_checkpoint_state_units_item_perception_effects_item_concealment_item_terrains_member1
    =
  Array.map (fun value -> convert_657_761 value) value

and convert_657_761
    (value :
      save41_member0_checkpoint_state_units_item_perception_effects_item_concealment_item_terrains_member1_item)
    :
    save43_member0_checkpoint_state_units_item_perception_effects_item_concealment_item_terrains_member1_item
    =
  (value
    : save41_member0_checkpoint_state_units_item_perception_effects_item_concealment_item_terrains_member1_item
    :> save43_member0_checkpoint_state_units_item_perception_effects_item_concealment_item_terrains_member1_item)

let v42 : V42.t -> V43.t = function
  | `Current value -> `Current (convert_744_748 value)
  | `Turn value -> `Turn (convert_745_777 value)
