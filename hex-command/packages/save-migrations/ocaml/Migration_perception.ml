(* Frozen v40 -> v41: bounded perception, explicit identity and scheduling.
   Conversion uses only frozen typed histories. No current rules or engine calls. *)
open Historical_types

[@@@warning "+8+9-27-39"]
[@@@warnerror "+8+9"]

let remembered = function Some `True -> true | Some (`False | `Undefined) | None -> false

let primary_factions (old : V40.state) =
  let factions = ref (Array.to_list old.factions) in
  List.iter
    (fun id ->
      if not (List.exists (fun (f : V40.faction) -> f.id = id) !factions) then
        factions :=
          !factions
          @ [
              {
                wire = empty_wire;
                id;
                phase = (if id = "player" then `Player else `Enemy);
                controller =
                  (if id = "player" then `Human { wire = empty_wire; kind = `Human }
                   else `Ai { wire = empty_wire; kind = `Ai; policy = None });
                palette = id;
                name = None;
                recruits = None;
                relations = None;
                transfer = None;
                vision = None;
              };
            ])
    [ "player"; "enemy" ];
  { old with factions = Array.of_list !factions }

let groups factions : save41_member0_checkpoint_state_vision_groups =
  Array.map
    (fun (f : V40.faction) ->
      ({
         wire = empty_wire;
         id = f.id;
         sight =
           {
             wire = empty_wire;
             range = `Member1 `Unlimited;
             penetrates = (if Option.is_some f.vision then [||] else [| `Fog |]);
           };
         explored = (match f.vision with None -> [||] | Some v -> v.explored);
       }
        : save41_member0_checkpoint_state_vision_groups_item))
    factions

let fog_radius (condition : V40.condition) =
  let proximity (value : V40.condition) =
    match value with
    | `Proximity p
      when p.side = `Observer && p.tile = None && p.unit_type = None && p.character = None
           && Option.fold ~none:true ~some:(fun v -> Array.length v = 0) p.unit_types ->
        Some p.radius
    | _ -> None
  in
  match proximity condition with
  | Some radius -> radius
  | None -> (
      match condition with
      | `Any a when Array.length a.conditions = 2 -> (
          let flag =
            Array.exists (function `Flag f -> f.flag = "fog-cleared" | _ -> false) a.conditions
          in
          let radius = Array.to_list a.conditions |> List.find_map proximity in
          match (flag, radius) with
          | true, Some radius -> radius
          | _ -> invalid_arg "Unsupported historical fog expression")
      | _ -> invalid_arg "Unsupported historical fog expression")

let tile_perception cleared (tile : V40.tile) :
    save41_member0_checkpoint_state_tiles_item_perception =
  match tile.fog with
  | None -> { wire = empty_wire; sight = `Null; remember = false; concealment = [||] }
  | Some fog ->
      let radius = fog_radius fog.condition in
      let conditional = match fog.condition with `Any _ -> true | _ -> false in
      {
        wire = empty_wire;
        sight = `Null;
        remember = remembered fog.remember;
        concealment =
          (if conditional && cleared then [||]
           else [| { wire = empty_wire; kind = `Fog; reveal_within = radius } |]);
      }

let rec copy_0 (_cleared : bool) (value : save40_member0_checkpoint_state) :
    save41_member0_checkpoint_state =
  let ({
         wire;
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
        : save40_member0_checkpoint_state) =
    value
  in
  {
    wire;
    vision_groups = groups value.factions;
    factions = copy_1 _cleared factions;
    ground_items = copy_2 _cleared ground_items;
    armies;
    condition_memory;
    map_id;
    map_revision;
    cols;
    rows;
    round;
    turn = copy_3 _cleared turn;
    gold;
    winner = copy_4 _cleared winner;
    next_id;
    combat_rng;
    log;
    tiles = copy_5 _cleared tiles;
    units = copy_6 _cleared units;
    campaign = Option.map (fun v -> copy_7 _cleared v) campaign;
    last_combat;
  }

and copy_1 (_cleared : bool) (value : save40_member0_checkpoint_state_factions) :
    save41_member0_checkpoint_state_factions =
  Array.map (fun v -> copy_8 _cleared v) value

and copy_2 (_cleared : bool) (value : save39_member0_checkpoint_state_ground_items) :
    save41_member0_checkpoint_state_ground_items =
  Array.map (fun v -> copy_9 _cleared v) value

and copy_3 (_cleared : bool) (value : save35_setup_first) : save41_member0_setup_first =
  match value with `Player -> `Player | `Enemy -> `Ai

and copy_4 (_cleared : bool) (value : fragments_native_state38_winner) :
    save41_member0_checkpoint_state_winner =
  match value with `Null -> `Null | `Player -> `Player_victory | `Enemy -> `Player_defeat

and copy_5 (_cleared : bool) (value : save40_member0_checkpoint_state_tiles) :
    save41_member0_checkpoint_state_tiles =
  Array.map (fun v -> copy_10 _cleared v) value

and copy_6 (_cleared : bool) (value : save39_member0_checkpoint_state_units) :
    save41_member0_checkpoint_state_units =
  Array.map (fun v -> copy_11 _cleared v) value

and copy_7 (_cleared : bool) (value : save40_member0_checkpoint_state_campaign_member1) :
    save41_member0_checkpoint_state_campaign_member1 =
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
        : save40_member0_checkpoint_state_campaign_member1) =
    value
  in
  {
    wire;
    scripts = Option.map (fun v -> copy_12 _cleared v) scripts;
    evacuated = Option.map (fun v -> copy_13 _cleared v) evacuated;
    id;
    mission;
    recall;
    veterans = Option.map (fun v -> copy_14 _cleared v) veterans;
    story_veterans = Option.map (fun v -> copy_14 _cleared v) story_veterans;
    fallen_companions;
    carry_gold;
    marks;
    dialogue = copy_15 _cleared dialogue;
    dialogue_index;
    outcome;
    completed;
    refuge;
    vigil;
    holds;
    flags;
  }

and copy_8 (_cleared : bool) (value : save40_member0_checkpoint_state_factions_item) :
    save41_member0_checkpoint_state_factions_item =
  let ({ wire; id; phase; controller; palette; name; recruits; relations; transfer; vision }
        : save40_member0_checkpoint_state_factions_item) =
    value
  in
  {
    wire = remove_fields wire [ "vision" ];
    vision_group = id;
    id;
    phase = copy_3 _cleared phase;
    controller;
    palette;
    name;
    recruits;
    relations = Option.map (fun v -> copy_16 _cleared v) relations;
    transfer = Option.map (fun v -> copy_17 _cleared v) transfer;
  }

and copy_9 (_cleared : bool) (value : save39_member0_checkpoint_state_ground_items_item) :
    save41_member0_checkpoint_state_ground_items_item =
  let ({ wire; tile; item } : save39_member0_checkpoint_state_ground_items_item) = value in
  { wire; tile; item = copy_18 _cleared item }

and copy_10 (_cleared : bool) (value : save40_member0_checkpoint_state_tiles_item) :
    save41_member0_checkpoint_state_tiles_item =
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
         fog;
         light;
       }
        : save40_member0_checkpoint_state_tiles_item) =
    value
  in
  {
    wire = remove_fields wire [ "fog" ];
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
    perception = tile_perception _cleared value;
    light = Option.map (fun v -> copy_19 _cleared v) light;
  }

and copy_11 (_cleared : bool) (value : save39_member0_checkpoint_state_units_item) :
    save41_member0_checkpoint_state_units_item =
  let ({
         wire;
         faction;
         items;
         ai_group;
         id;
         type_;
         side;
         tile;
         hp;
         moved;
         attacked;
         experience;
         character;
         role;
         retreating;
         rebuilding;
       }
        : save39_member0_checkpoint_state_units_item) =
    value
  in
  {
    wire = remove_fields wire [ "attacked" ];
    revealed_to = [||];
    perception_effects = [||];
    faction =
      Option.value faction ~default:(match side with `Player -> "player" | `Enemy -> "enemy");
    items = Option.map (fun v -> copy_20 _cleared v) items;
    ai_group;
    id;
    type_;
    side = copy_3 _cleared side;
    tile;
    hp;
    moved;
    attack_spent = attacked;
    experience;
    character;
    role;
    retreating =
      Option.bind retreating (fun value ->
          match value with
          | `Undefined -> None
          | `False -> Some (copy_21 _cleared `False)
          | `True -> Some (copy_22 _cleared `True));
    rebuilding =
      Option.bind rebuilding (fun value ->
          match value with
          | `Undefined -> None
          | `False -> Some (copy_21 _cleared `False)
          | `True -> Some (copy_22 _cleared `True));
  }

and copy_12 (_cleared : bool)
    (value : save40_member0_checkpoint_state_campaign_member1_scripts_member1) :
    save41_member0_checkpoint_state_campaign_member1_scripts_member1 =
  let ({ wire; failed; completed; revealed; recruitment; active; holds; marks; pending; next_mark }
        : save40_member0_checkpoint_state_campaign_member1_scripts_member1) =
    value
  in
  {
    wire;
    failed = Option.fold ~none:[||] ~some:(fun v -> v) failed;
    completed;
    revealed;
    recruitment;
    active;
    holds;
    marks = copy_23 _cleared marks;
    pending = copy_24 _cleared pending;
    next_mark;
  }

and copy_13 (_cleared : bool)
    (value : save39_member0_checkpoint_state_campaign_member1_evacuated_member1) :
    save41_member0_checkpoint_state_campaign_member1_evacuated_member1 =
  Array.map (fun v -> copy_25 _cleared v) value

and copy_14 (_cleared : bool) (value : fragments_native_state38_campaign_member1_veterans_member1) :
    save41_member0_checkpoint_state_campaign_member1_veterans_member1 =
  Array.map (fun v -> copy_26 _cleared v) value

and copy_15 (_cleared : bool) (value : fragments_native_state38_campaign_member1_dialogue) :
    save41_member0_checkpoint_state_campaign_member1_dialogue =
  Array.map (fun v -> copy_27 _cleared v) value

and copy_16 (_cleared : bool) (value : fragments_factions7_item_relations_member1) :
    save41_member0_checkpoint_state_factions_item_relations_member1 =
  String_map.map (fun v -> copy_28 _cleared v) value

and copy_17 (_cleared : bool)
    (value : save40_member0_checkpoint_state_factions_item_transfer_member1) :
    save41_member0_checkpoint_state_factions_item_transfer_member1 =
  let ({ wire; to_; when_ } : save40_member0_checkpoint_state_factions_item_transfer_member1) =
    value
  in
  { wire; to_; when_ = copy_29 _cleared when_ }

and copy_18 (_cleared : bool) (value : fragments_native_state38_ground_items_item_item) :
    save41_member0_checkpoint_state_ground_items_item_item =
  let ({ wire; armor; id; name; description; attacks; healing }
        : fragments_native_state38_ground_items_item_item) =
    value
  in
  {
    wire;
    perception = None;
    armor;
    id;
    name;
    description;
    attacks = copy_30 _cleared attacks;
    healing;
  }

and copy_19 (_cleared : bool) (value : save40_member0_checkpoint_state_tiles_item_light_member1) :
    save41_member0_checkpoint_state_tiles_item_light_member1 =
  let ({ wire; color; fixture; dx; dy; radius; power; when_ }
        : save40_member0_checkpoint_state_tiles_item_light_member1) =
    value
  in
  {
    wire;
    emission = `Null;
    color;
    fixture =
      Option.bind fixture (fun value ->
          match value with
          | `Undefined -> None
          | `False -> Some (copy_21 _cleared `False)
          | `True -> Some (copy_22 _cleared `True));
    dx;
    dy;
    radius;
    power;
    when_ = Option.map (fun v -> copy_29 _cleared v) when_;
  }

and copy_20 (_cleared : bool) (value : fragments_native_state38_units_item_items_member1) :
    save41_member0_checkpoint_state_units_item_items_member1 =
  Array.map (fun v -> copy_18 _cleared v) value

and copy_21 (_cleared : bool) (value : literal_false) : json_boolean =
  let _ = value in
  false

and copy_22 (_cleared : bool) (value : literal_true) : json_boolean =
  let _ = value in
  true

and copy_23 (_cleared : bool)
    (value : save40_member0_checkpoint_state_campaign_member1_scripts_member1_marks) :
    save41_member0_checkpoint_state_campaign_member1_scripts_member1_marks =
  Array.map (fun v -> copy_31 _cleared v) value

and copy_24 (_cleared : bool)
    (value : save39_member0_checkpoint_state_campaign_member1_scripts_member1_pending) :
    save41_member0_checkpoint_state_campaign_member1_scripts_member1_pending =
  Array.map (fun v -> copy_32 _cleared v) value

and copy_25 (_cleared : bool)
    (value : save39_member0_checkpoint_state_campaign_member1_evacuated_member1_item) :
    save41_member0_checkpoint_state_campaign_member1_evacuated_member1_item =
  let ({ wire; event; unit_ }
        : save39_member0_checkpoint_state_campaign_member1_evacuated_member1_item) =
    value
  in
  { wire; event; unit_ = copy_11 _cleared unit_ }

and copy_26 (_cleared : bool)
    (value : fragments_native_state38_campaign_member1_veterans_member1_item) :
    save41_member0_checkpoint_state_campaign_member1_veterans_member1_item =
  let ({ wire; items; type_; experience; character; role }
        : fragments_native_state38_campaign_member1_veterans_member1_item) =
    value
  in
  {
    wire;
    items = Option.map (fun v -> copy_20 _cleared v) items;
    type_;
    experience;
    character;
    role;
  }

and copy_27 (_cleared : bool) (value : fragments_native_state38_campaign_member1_dialogue_item) :
    save41_member0_checkpoint_state_campaign_member1_dialogue_item =
  let ({ wire; sound; speaker; character; portrait; text }
        : fragments_native_state38_campaign_member1_dialogue_item) =
    value
  in
  {
    wire;
    sound;
    speaker = Option.fold ~none:"" ~some:(fun v -> v) speaker;
    character;
    portrait;
    text;
  }

and copy_28 (_cleared : bool) (value : fragments_factions7_item_relations_member1_value) :
    save41_member0_checkpoint_state_factions_item_relations_member1_value =
  match value with `Neutral -> `Neutral | `Allied -> `Allied | `Hostile -> `Hostile

and copy_29 (_cleared : bool) (value : save40_member0_checkpoint_state_tiles_item_fog_member1) :
    save41_member0_checkpoint_state_factions_item_transfer_member1_when_ =
  let ({ wire; condition; remember } : save40_member0_checkpoint_state_tiles_item_fog_member1) =
    value
  in
  {
    wire;
    condition;
    remember = Option.fold ~none:false ~some:(fun v -> copy_33 _cleared v) remember;
  }

and copy_30 (_cleared : bool) (value : fragments_native_state38_ground_items_item_item_attacks) :
    save41_member0_checkpoint_state_ground_items_item_item_attacks =
  Array.map (fun v -> copy_34 _cleared v) value

and copy_31 (_cleared : bool)
    (value : save40_member0_checkpoint_state_campaign_member1_scripts_member1_marks_item) :
    save41_member0_checkpoint_state_campaign_member1_scripts_member1_marks_item =
  let ({ wire; light_remembered; id; type_; tile; light }
        : save40_member0_checkpoint_state_campaign_member1_scripts_member1_marks_item) =
    value
  in
  {
    wire;
    sight = `Null;
    light_remembered = Option.fold ~none:false ~some:(fun v -> copy_33 _cleared v) light_remembered;
    id;
    type_;
    tile;
    light = Option.map (fun v -> copy_19 _cleared v) light;
  }

and copy_32 (_cleared : bool)
    (value : save39_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item) :
    save41_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item =
  let ({ wire; area; faction; ai_group; units }
        : save39_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item) =
    value
  in
  {
    wire;
    area;
    faction;
    ai_group = Option.fold ~none:`Null ~some:(fun v -> copy_35 _cleared v) ai_group;
    units = copy_36 _cleared units;
  }

and copy_33 (_cleared : bool) (value : fragments_rule11_remember) : json_boolean =
  match value with `Undefined -> false | `False -> false | `True -> true

and copy_34 (_cleared : bool) (value : fragments_native_state38_ground_items_item_item_attacks_item)
    : save41_member0_checkpoint_state_ground_items_item_item_attacks_item =
  let ({ wire; id; name; damage; spread; count; damage_type; category; range; sound; drain }
        : fragments_native_state38_ground_items_item_item_attacks_item) =
    value
  in
  {
    wire;
    id;
    name;
    damage;
    spread = Option.fold ~none:0. ~some:(fun v -> v) spread;
    count = Option.fold ~none:1. ~some:(fun v -> v) count;
    damage_type;
    category;
    range;
    sound;
    drain = Option.fold ~none:0. ~some:(fun v -> v) drain;
  }

and copy_35 (_cleared : bool) (value : fragments_delta19_item_owner) : fragments_tiles11_item_owner
    =
  match value with `Undefined -> `Null | `Null -> `Null | `String v -> `String v

and copy_36 (_cleared : bool)
    (value : fragments_native_state38_campaign_member1_scripts_member1_pending_item_units) :
    save41_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item_units =
  Array.map (fun v -> copy_37 _cleared v) value

and copy_37 (_cleared : bool)
    (value : fragments_native_state38_campaign_member1_scripts_member1_pending_item_units_item) :
    save41_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item_units_item =
  let ({ wire; type_; character; role; items }
        : fragments_native_state38_campaign_member1_scripts_member1_pending_item_units_item) =
    value
  in
  {
    wire;
    type_;
    character = Option.fold ~none:`Null ~some:(fun v -> copy_35 _cleared v) character;
    role = Option.fold ~none:`Null ~some:(fun v -> copy_38 _cleared v) role;
    items = Option.fold ~none:[||] ~some:(fun v -> v) items;
  }

and copy_38 (_cleared : bool)
    (value : fragments_native_state38_campaign_member1_scripts_member1_pending_item_units_item_role)
    : save41_member0_checkpoint_state_campaign_member1_scripts_member1_pending_item_units_item_role
    =
  match value with
  | `Undefined -> `Null
  | `Null -> `Null
  | `Leader -> `Leader
  | `Courier -> `Courier
  | `Captive -> `Captive

let state (old : V40.state) : V41.state =
  let old = primary_factions old in
  let cleared =
    match old.campaign with
    | None -> false
    | Some c -> (
        match String_map.find_opt "fog-cleared" c.flags with
        | None -> false
        | Some history -> Array.length history > 0 && history.(Array.length history - 1).state)
  in
  copy_0 cleared old

let v40 (old : V40.t) : V41.t =
  let convert checkpoint undo =
    Migration_late.convert_state_and_undo
      ~encode_old:Historical_codec.encode_save40_member0_checkpoint_state
      ~decode_old:Historical_codec.read_save40_member0_checkpoint_state
      ~encode_new:Historical_codec.encode_save41_member0_checkpoint_state state checkpoint undo
  in
  let first = function `Player -> `Player | `Enemy -> `Ai in
  match old with
  | `Current old ->
      let state, undo = convert old.checkpoint.state (Some old.undo) in
      `Current
        {
          wire = old.wire;
          pending_enemy = old.pending_enemy;
          undo = Option.get undo;
          progress = old.progress;
          version = `V41;
          checkpoint = { wire = old.checkpoint.wire; seed = old.checkpoint.seed; state };
          setup = { wire = old.setup.wire; enemy = old.setup.enemy; first = first old.setup.first };
        }
  | `Turn old ->
      let state, _ = convert old.checkpoint.state None in
      `Turn
        {
          wire = old.wire;
          pending_enemy = old.pending_enemy;
          undo = None;
          progress = old.progress;
          version = `V41;
          checkpoint = { wire = old.checkpoint.wire; seed = old.checkpoint.seed; state };
          setup = { wire = old.setup.wire; enemy = old.setup.enemy; first = first old.setup.first };
        }
