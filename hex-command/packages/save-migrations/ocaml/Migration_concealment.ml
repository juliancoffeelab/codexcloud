(* Frozen v43 -> v44: express old concealment applicability as explicit rule data. *)

open Historical_types

[@@@warning "+8+9-27-39"]
[@@@warnerror "+8+9"]

let rec convert_748_813 (value : save43_member0) : save44_member0 =
  let ({ wire; pending_enemy; undo; progress; version; checkpoint; setup } : save43_member0) =
    value
  in
  {
    wire;
    pending_enemy;
    undo;
    progress;
    version = convert_749_814 version;
    checkpoint = convert_750_778 checkpoint;
    setup;
  }

and convert_777_815 (value : save43_member1) : save44_member1 =
  let ({ wire; pending_enemy; undo; progress; version; checkpoint; setup } : save43_member1) =
    value
  in
  {
    wire;
    pending_enemy;
    undo;
    progress;
    version = convert_749_814 version;
    checkpoint = convert_750_778 checkpoint;
    setup;
  }

and convert_749_814 (value : literal_43) : literal_44 =
  let `V43 = value in
  `V44

and convert_750_778 (value : save43_member0_checkpoint) : save44_member0_checkpoint =
  let ({ wire; seed; state } : save43_member0_checkpoint) = value in
  { wire; seed; state = convert_751_779 state }

and convert_751_779 (value : save43_member0_checkpoint_state) : save44_member0_checkpoint_state =
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
        : save43_member0_checkpoint_state) =
    value
  in
  {
    wire;
    vision_groups;
    factions;
    ground_items = convert_752_794 ground_items;
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
    tiles = convert_706_797 tiles;
    units = convert_763_803 units;
    campaign = Option.map (fun campaign -> convert_770_804 campaign) campaign;
    last_combat;
  }

and convert_752_794 (value : save43_member0_checkpoint_state_ground_items) :
    save44_member0_checkpoint_state_ground_items =
  Array.map (fun value -> convert_753_780 value) value

and convert_706_797 (value : save41_member0_checkpoint_state_tiles) :
    save44_member0_checkpoint_state_tiles =
  Array.map (fun value -> convert_697_795 value) value

and convert_763_803 (value : save43_member0_checkpoint_state_units) :
    save44_member0_checkpoint_state_units =
  Array.map (fun value -> convert_764_798 value) value

and convert_770_804 (value : save43_member0_checkpoint_state_campaign_member1) :
    save44_member0_checkpoint_state_campaign_member1 =
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
        : save43_member0_checkpoint_state_campaign_member1) =
    value
  in
  {
    wire;
    scripts;
    evacuated = Option.map (fun evacuated -> convert_772_806 evacuated) evacuated;
    id;
    mission;
    recall;
    veterans = Option.map (fun veterans -> convert_775_809 veterans) veterans;
    story_veterans =
      Option.map (fun story_veterans -> convert_775_809 story_veterans) story_veterans;
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

and convert_753_780 (value : save43_member0_checkpoint_state_ground_items_item) :
    save44_member0_checkpoint_state_ground_items_item =
  let ({ wire; tile; item } : save43_member0_checkpoint_state_ground_items_item) = value in
  { wire; tile; item = convert_754_781 item }

and convert_697_795 (value : save41_member0_checkpoint_state_tiles_item) :
    save44_member0_checkpoint_state_tiles_item =
  let ({
         wire;
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
        : save41_member0_checkpoint_state_tiles_item) =
    value
  in
  {
    wire;
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
    perception = convert_698_796 perception;
    light;
  }

and convert_764_798 (value : save43_member0_checkpoint_state_units_item) :
    save44_member0_checkpoint_state_units_item =
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
        : save43_member0_checkpoint_state_units_item) =
    value
  in
  {
    wire;
    revealed_to;
    perception_effects = convert_765_800 perception_effects;
    faction;
    items = Option.map (fun items -> convert_768_801 items) items;
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

and convert_772_806 (value : save43_member0_checkpoint_state_campaign_member1_evacuated_member1) :
    save44_member0_checkpoint_state_campaign_member1_evacuated_member1 =
  Array.map (fun value -> convert_773_805 value) value

and convert_775_809 (value : save43_member0_checkpoint_state_campaign_member1_veterans_member1) :
    save44_member0_checkpoint_state_campaign_member1_veterans_member1 =
  Array.map (fun value -> convert_776_808 value) value

and convert_754_781 (value : save43_member0_checkpoint_state_ground_items_item_item) :
    save44_member0_checkpoint_state_ground_items_item_item =
  let ({ wire; perception; armor; id; name; description; attacks; healing }
        : save43_member0_checkpoint_state_ground_items_item_item) =
    value
  in
  {
    wire;
    perception = Option.map (fun perception -> convert_756_782 perception) perception;
    armor;
    id;
    name;
    description;
    attacks;
    healing;
  }

and convert_698_796 (value : save41_member0_checkpoint_state_tiles_item_perception) :
    save44_member0_checkpoint_state_tiles_item_perception =
  let ({ wire; sight; remember; concealment }
        : save41_member0_checkpoint_state_tiles_item_perception) =
    value
  in
  { wire; sight; remember; concealment = convert_701_792 concealment }

and convert_765_800 (value : save43_member0_checkpoint_state_units_item_perception_effects) :
    save44_member0_checkpoint_state_units_item_perception_effects =
  Array.map (fun value -> convert_766_799 value) value

and convert_768_801 (value : save43_member0_checkpoint_state_units_item_items_member1) :
    save44_member0_checkpoint_state_units_item_items_member1 =
  Array.map (fun value -> convert_754_781 value) value

and convert_773_805
    (value : save43_member0_checkpoint_state_campaign_member1_evacuated_member1_item) :
    save44_member0_checkpoint_state_campaign_member1_evacuated_member1_item =
  let ({ wire; event; unit_ }
        : save43_member0_checkpoint_state_campaign_member1_evacuated_member1_item) =
    value
  in
  { wire; event; unit_ = convert_764_798 unit_ }

and convert_776_808 (value : save43_member0_checkpoint_state_campaign_member1_veterans_member1_item)
    : save44_member0_checkpoint_state_campaign_member1_veterans_member1_item =
  let ({ wire; items; type_; experience; character; role }
        : save43_member0_checkpoint_state_campaign_member1_veterans_member1_item) =
    value
  in
  {
    wire;
    items = Option.map (fun items -> convert_768_801 items) items;
    type_;
    experience;
    character;
    role;
  }

and convert_756_782
    (value : save43_member0_checkpoint_state_ground_items_item_item_perception_member1) :
    save44_member0_checkpoint_state_ground_items_item_item_perception_member1 =
  let ({ wire; range; penetrates; concealment; light }
        : save43_member0_checkpoint_state_ground_items_item_item_perception_member1) =
    value
  in
  { wire; range; penetrates; concealment = convert_757_792 concealment; light }

and convert_701_792 (value : save41_member0_checkpoint_state_tiles_item_perception_concealment) :
    save44_member0_checkpoint_state_tiles_item_perception_concealment =
  Array.map (fun value -> convert_656_783 value) value

and convert_766_799 (value : save43_member0_checkpoint_state_units_item_perception_effects_item) :
    save44_member0_checkpoint_state_units_item_perception_effects_item =
  let ({ wire; id; penetrates; concealment; light; expires_at }
        : save43_member0_checkpoint_state_units_item_perception_effects_item) =
    value
  in
  { wire; id; penetrates; concealment = convert_757_792 concealment; light; expires_at }

and convert_757_792
    (value : save43_member0_checkpoint_state_units_item_perception_effects_item_concealment) :
    save44_member0_checkpoint_state_tiles_item_perception_concealment =
  Array.map (fun value -> convert_758_783 value) value

and convert_656_783 (value : save41_member0_checkpoint_state_tiles_item_perception_concealment_item)
    : save44_member0_checkpoint_state_tiles_item_perception_concealment_item =
  let ({ wire; kind; reveal_within }
        : save41_member0_checkpoint_state_tiles_item_perception_concealment_item) =
    value
  in
  {
    wire;
    kind;
    reveal_within;
    requires =
      ({
         wire = empty_wire;
         terrains = `Null;
         illumination = (match kind with `Darkness -> `Unlit | `Fog | `Forest_camouflage -> `Null);
       }
        : save44_member0_checkpoint_state_tiles_item_perception_concealment_item_requires);
  }

and convert_758_783
    (value : save43_member0_checkpoint_state_units_item_perception_effects_item_concealment_item) :
    save44_member0_checkpoint_state_tiles_item_perception_concealment_item =
  let ({ wire; kind; reveal_within; terrains }
        : save43_member0_checkpoint_state_units_item_perception_effects_item_concealment_item) =
    value
  in
  {
    wire;
    kind;
    reveal_within;
    requires =
      ({
         wire = empty_wire;
         terrains;
         illumination = (match kind with `Darkness -> `Unlit | `Fog | `Forest_camouflage -> `Null);
       }
        : save44_member0_checkpoint_state_tiles_item_perception_concealment_item_requires);
  }

let v43 (old : V43.t) : V44.t =
  let convert checkpoint undo =
    Migration_late.convert_state_and_undo
      ~encode_old:Historical_codec.encode_save43_member0_checkpoint_state
      ~decode_old:Historical_codec.read_save43_member0_checkpoint_state
      ~encode_new:Historical_codec.encode_save44_member0_checkpoint_state
      (fun value -> convert_751_779 value)
      checkpoint undo
  in
  match old with
  | `Current old ->
      let state, undo = convert old.checkpoint.state (Some old.undo) in
      let value = convert_748_813 old in
      `Current { value with checkpoint = { value.checkpoint with state }; undo = Option.get undo }
  | `Turn old ->
      let state, _ = convert old.checkpoint.state None in
      let value = convert_777_815 old in
      `Turn { value with checkpoint = { value.checkpoint with state } }
