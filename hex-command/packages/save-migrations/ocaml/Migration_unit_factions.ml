(* Frozen v45 -> v46: faction membership is authoritative; remove redundant unit phases.
   Every undo state is converted and re-diffed through the same typed boundary. *)
open Historical_types

let validate_phases (state : V45.state) =
  let unit_ (unit : V45.unit_) =
    let faction = Array.find_opt (fun (f : V45.faction) -> f.id = unit.faction) state.factions in
    Migration_json.require
      (Option.exists (fun (f : V45.faction) -> f.phase = unit.side) faction)
      "Saved unit phase differs from its faction"
  in
  Array.iter unit_ state.units;
  Option.iter
    (fun (c : V45.campaign) ->
      Option.iter
        (Array.iter
           (fun (e : save44_member0_checkpoint_state_campaign_member1_evacuated_member1_item) ->
             unit_ e.unit_))
        c.evacuated)
    state.campaign

let rec convert_2927_2928 (value : save45) : save46 =
  match value with
  | `Current v -> `Current (convert_2924_2929 v)
  | `Turn v -> `Turn (convert_2926_2940 v)

and convert_2924_2929 (value : save45_member0) : save46_member0 =
  {
    wire = value.wire;
    pending_enemy = value.pending_enemy;
    undo = value.undo;
    progress = value.progress;
    version = convert_2923_2930 value.version;
    checkpoint = convert_2445_2931 value.checkpoint;
    setup = value.setup;
  }

and convert_2926_2940 (value : save45_member1) : save46_member1 =
  {
    wire = value.wire;
    pending_enemy = value.pending_enemy;
    undo = value.undo;
    progress = value.progress;
    version = convert_2925_2941 value.version;
    checkpoint = convert_2445_2931 value.checkpoint;
    setup = value.setup;
  }

and convert_2923_2930 (value : literal_45) : literal_46 =
  let _ = value in
  `V46

and convert_2445_2931 (value : save44_member0_checkpoint) : save46_member0_checkpoint =
  { wire = value.wire; seed = value.seed; state = convert_2447_2932 value.state }

and convert_2925_2941 (value : literal_45) : literal_46 =
  let _ = value in
  `V46

and convert_2447_2932 (value : save44_member0_checkpoint_state) : save46_member0_checkpoint_state =
  validate_phases value;
  {
    wire = value.wire;
    vision_groups = value.vision_groups;
    factions = value.factions;
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
    tiles = value.tiles;
    units = convert_2759_2933 value.units;
    campaign = Option.map (fun v -> convert_2760_2936 v) value.campaign;
    last_combat = value.last_combat;
  }

and convert_2759_2933 (value : save44_member0_checkpoint_state_units) :
    save46_member0_checkpoint_state_units =
  Array.map (fun v -> convert_2724_2934 v) value

and convert_2760_2936 (value : save44_member0_checkpoint_state_campaign_member1) :
    save46_member0_checkpoint_state_campaign_member1 =
  {
    wire = value.wire;
    scripts = value.scripts;
    evacuated = Option.map (fun v -> convert_2803_2938 v) value.evacuated;
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

and convert_2724_2934 (value : save44_member0_checkpoint_state_units_item) :
    save46_member0_checkpoint_state_units_item =
  {
    wire = value.wire;
    revealed_to = value.revealed_to;
    perception_effects = value.perception_effects;
    faction = value.faction;
    items = value.items;
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

and convert_2803_2938 (value : save44_member0_checkpoint_state_campaign_member1_evacuated_member1) :
    save46_member0_checkpoint_state_campaign_member1_evacuated_member1 =
  Array.map (fun v -> convert_2801_2939 v) value

and convert_2801_2939
    (value : save44_member0_checkpoint_state_campaign_member1_evacuated_member1_item) :
    save46_member0_checkpoint_state_campaign_member1_evacuated_member1_item =
  { wire = value.wire; event = value.event; unit_ = convert_2724_2934 value.unit_ }

let v45 (old : V45.t) : V46.t =
  let convert checkpoint undo =
    Migration_late.convert_state_and_undo
      ~encode_old:Historical_codec.encode_save44_member0_checkpoint_state
      ~decode_old:Historical_codec.read_save44_member0_checkpoint_state
      ~encode_new:Historical_codec.encode_save46_member0_checkpoint_state convert_2447_2932
      checkpoint undo
  in
  match (old, convert_2927_2928 old) with
  | `Current old, `Current value ->
      let state, undo = convert old.checkpoint.state (Some old.undo) in
      `Current { value with checkpoint = { value.checkpoint with state }; undo = Option.get undo }
  | `Turn old, `Turn value ->
      let state, _ = convert old.checkpoint.state None in
      `Turn { value with checkpoint = { value.checkpoint with state } }
  | _ -> assert false
