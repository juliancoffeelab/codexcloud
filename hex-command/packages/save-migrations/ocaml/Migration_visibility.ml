(* Frozen v39 -> v40 conversion. Every structural change is typed; unchanged facts retain their values. *)

open Historical_types

[@@@warning "+8+9"]
[@@@warnerror "+8+9"]

let rec copy_state (value : save39_member0_checkpoint_state) : save40_member0_checkpoint_state =
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
        : save39_member0_checkpoint_state) =
    value
  in
  ({
     wire;
     factions = copy_factions factions;
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
     tiles = copy_tiles tiles;
     units;
     campaign = Option.map (fun v -> copy_campaign v) campaign;
     last_combat;
   }
    : save40_member0_checkpoint_state)

and copy_factions (value : save39_member0_checkpoint_state_factions) :
    save40_member0_checkpoint_state_factions =
  Array.map (fun v -> copy_faction v) value

and copy_tiles (value : save39_member0_checkpoint_state_tiles) :
    save40_member0_checkpoint_state_tiles =
  Array.map (fun v -> copy_tile v) value

and copy_campaign (value : save39_member0_checkpoint_state_campaign_member1) :
    save40_member0_checkpoint_state_campaign_member1 =
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
        : save39_member0_checkpoint_state_campaign_member1) =
    value
  in
  ({
     wire;
     scripts = Option.map (fun v -> copy_scripts v) scripts;
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
    : save40_member0_checkpoint_state_campaign_member1)

and copy_faction (value : save39_member0_checkpoint_state_factions_item) :
    save40_member0_checkpoint_state_factions_item =
  let ({ wire; id; phase; controller; palette; name; recruits; relations; transfer }
        : save39_member0_checkpoint_state_factions_item) =
    value
  in
  ({
     wire;
     id;
     phase;
     controller;
     palette;
     name;
     recruits;
     relations;
     transfer = Option.map (fun v -> copy_transfer v) transfer;
     vision = None;
   }
    : save40_member0_checkpoint_state_factions_item)

and copy_tile (value : save39_member0_checkpoint_state_tiles_item) :
    save40_member0_checkpoint_state_tiles_item =
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
        : save39_member0_checkpoint_state_tiles_item) =
    value
  in
  ({
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
     fog = Option.map (fun v -> copy_rule v) fog;
     light = Option.map (fun v -> copy_light v) light;
   }
    : save40_member0_checkpoint_state_tiles_item)

and copy_scripts (value : save39_member0_checkpoint_state_campaign_member1_scripts_member1) :
    save40_member0_checkpoint_state_campaign_member1_scripts_member1 =
  let ({ wire; failed; completed; revealed; recruitment; active; holds; marks; pending; next_mark }
        : save39_member0_checkpoint_state_campaign_member1_scripts_member1) =
    value
  in
  ({
     wire;
     failed;
     completed;
     revealed;
     recruitment;
     active;
     holds;
     marks = copy_marks marks;
     pending;
     next_mark;
   }
    : save40_member0_checkpoint_state_campaign_member1_scripts_member1)

and copy_transfer (value : save39_member0_checkpoint_state_factions_item_transfer_member1) :
    save40_member0_checkpoint_state_factions_item_transfer_member1 =
  let ({ wire; to_; when_ } : save39_member0_checkpoint_state_factions_item_transfer_member1) =
    value
  in
  ({ wire; to_; when_ = copy_rule when_ }
    : save40_member0_checkpoint_state_factions_item_transfer_member1)

and copy_rule (value : save39_member0_checkpoint_state_tiles_item_fog_member1) :
    save40_member0_checkpoint_state_tiles_item_fog_member1 =
  let ({ wire; condition; remember } : save39_member0_checkpoint_state_tiles_item_fog_member1) =
    value
  in
  ({ wire; condition = copy_condition condition; remember }
    : save40_member0_checkpoint_state_tiles_item_fog_member1)

and copy_light (value : save39_member0_checkpoint_state_tiles_item_light_member1) :
    save40_member0_checkpoint_state_tiles_item_light_member1 =
  let ({ wire; color; fixture; dx; dy; radius; power; when_ }
        : save39_member0_checkpoint_state_tiles_item_light_member1) =
    value
  in
  ({ wire; color; fixture; dx; dy; radius; power; when_ = Option.map (fun v -> copy_rule v) when_ }
    : save40_member0_checkpoint_state_tiles_item_light_member1)

and copy_marks (value : save39_member0_checkpoint_state_campaign_member1_scripts_member1_marks) :
    save40_member0_checkpoint_state_campaign_member1_scripts_member1_marks =
  Array.map (fun v -> copy_mark v) value

and copy_condition (value : save39_member0_checkpoint_state_tiles_item_fog_member1_condition) :
    save40_member0_checkpoint_state_tiles_item_fog_member1_condition =
  match value with
  | `Owner v -> `Owner v
  | `Proximity v -> `Proximity (copy_proximity v)
  | `Visible_units v -> `Visible_units (copy_visible_units v)
  | `Has_units v -> `Has_units (copy_has_units v)
  | `No_units v -> `No_units (copy_no_units v)
  | `Choice v -> `Choice v
  | `Flag v -> `Flag v
  | `Event v -> `Event v
  | `Round v -> `Round v
  | `All v -> `All (copy_all v)
  | `Any v -> `Any (copy_any v)

and copy_mark (value : save39_member0_checkpoint_state_campaign_member1_scripts_member1_marks_item)
    : save40_member0_checkpoint_state_campaign_member1_scripts_member1_marks_item =
  let ({ wire; light_remembered; id; type_; tile; light }
        : save39_member0_checkpoint_state_campaign_member1_scripts_member1_marks_item) =
    value
  in
  ({ wire; light_remembered; id; type_; tile; light = Option.map (fun v -> copy_light v) light }
    : save40_member0_checkpoint_state_campaign_member1_scripts_member1_marks_item)

and copy_proximity
    (value : save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member1) :
    save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member1 =
  let ({ wire; kind; unit_types; tile; radius; side; unit_type; character }
        : save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member1) =
    value
  in
  ({ wire; kind; unit_types; tile; radius; side = copy_side side; unit_type; character }
    : save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member1)

and copy_visible_units (value : fragments_rule11_condition_member1) :
    save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member2 =
  let ({ wire; kind; side; unit_types } : fragments_rule11_condition_member1) = value in
  ({ wire; kind; side = copy_side side; unit_types }
    : save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member2)

and copy_has_units (value : fragments_light28_member1_when__member1_condition_member3) :
    save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member3 =
  let ({ wire; kind; ai_group; side; unit_type; character }
        : fragments_light28_member1_when__member1_condition_member3) =
    value
  in
  ({ wire; kind; ai_group; side = copy_side side; unit_type; character }
    : save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member3)

and copy_no_units (value : fragments_light28_member1_when__member1_condition_member4) :
    save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member4 =
  let ({ wire; kind; side; unit_type; character }
        : fragments_light28_member1_when__member1_condition_member4) =
    value
  in
  ({ wire; kind; side = copy_side side; unit_type; character }
    : save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member4)

and copy_all (value : save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member9) :
    save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member9 =
  let ({ wire; kind; conditions }
        : save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member9) =
    value
  in
  ({ wire; kind; conditions = copy_conditions conditions }
    : save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member9)

and copy_any (value : save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member10) :
    save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member10 =
  let ({ wire; kind; conditions }
        : save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member10) =
    value
  in
  ({ wire; kind; conditions = copy_conditions conditions }
    : save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member10)

and copy_side (value : fragments_rule11_condition_member1_side) :
    save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member1_side =
  match value with `Player -> `Player | `Enemy -> `Enemy | `Neutral -> `Neutral | `Any -> `Any

and copy_conditions
    (value : save39_member0_checkpoint_state_tiles_item_fog_member1_condition_member9_conditions) :
    save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member9_conditions =
  Array.map (fun v -> copy_condition v) value

(* Only terrain fog is observer-relative. Event, transfer and light selectors
   retain their previous player/enemy/neutral meanings. *)
let relative_side :
    save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member1_side ->
    save40_member0_checkpoint_state_tiles_item_fog_member1_condition_member1_side = function
  | `Player -> `Observer
  | (`Enemy | `Neutral | `Any | `Observer) as side -> side

let rec relative_condition : V40.condition -> V40.condition = function
  | `Proximity v -> `Proximity { v with side = relative_side v.side }
  | `Visible_units v -> `Visible_units { v with side = relative_side v.side }
  | `Has_units v -> `Has_units { v with side = relative_side v.side }
  | `No_units v -> `No_units { v with side = relative_side v.side }
  | `All v -> `All { v with conditions = Array.map relative_condition v.conditions }
  | `Any v -> `Any { v with conditions = Array.map relative_condition v.conditions }
  | (`Owner _ | `Choice _ | `Flag _ | `Event _ | `Round _) as condition -> condition

let state (old : V39.state) : V40.state =
  let current = copy_state old in
  (* The old engine ignored unknown/malformed fog keys. Only facts that its
     coordinate lookup actually recognized become explored hexes. *)
  let explored =
    Option.value ~default:[||] old.condition_memory
    |> Array.to_list
    |> List.filter_map (fun key ->
        if not (String.starts_with ~prefix:"fog:" key) then None
        else
          let coords = String.sub key 4 (String.length key - 4) in
          match String.split_on_char ',' coords with
          | [ col; row ] -> (
              match (int_of_string_opt col, int_of_string_opt row) with
              | Some col, Some row ->
                  let col = float_of_int col and row = float_of_int row in
                  if Array.exists (fun (t : V39.tile) -> t.col = col && t.row = row) old.tiles then
                    Some ({ wire = empty_wire; col; row } : V40.hex)
                  else None
              | _ -> None)
          | _ -> None)
    |> List.sort_uniq (fun (a : V40.hex) b -> compare (a.col, a.row) (b.col, b.row))
    |> Array.of_list
  in
  let has_fog = Array.exists (fun (tile : V39.tile) -> Option.is_some tile.fog) old.tiles in
  let factions =
    Array.map
      (fun (f : V40.faction) ->
        let vision =
          if
            has_fog
            && (f.id = "player"
               || match f.controller with `Human _ -> true | `Ai _ | `Passive _ -> false)
          then Some ({ wire = empty_wire; explored } : V40.vision)
          else if
            old.map_id = "mirror-night"
            && match f.controller with `Ai _ -> true | `Human _ | `Passive _ -> false
          then Some ({ wire = empty_wire; explored = [||] } : V40.vision)
          else None
        in
        { f with wire = remove_fields f.wire [ "vision" ]; vision })
      current.factions
  in
  let tiles =
    Array.map
      (fun (tile : V40.tile) ->
        {
          tile with
          fog =
            Option.map
              (fun (rule : V40.fog) -> { rule with condition = relative_condition rule.condition })
              tile.fog;
        })
      current.tiles
  in
  let condition_memory =
    Option.map
      (fun values ->
        Array.to_list values
        |> List.filter (fun key -> not (String.starts_with ~prefix:"fog:" key))
        |> Array.of_list)
      old.condition_memory
  in
  { current with factions; tiles; condition_memory }

let v39 (old : V39.t) : V40.t =
  let convert checkpoint undo =
    let state, undo =
      Migration_late.convert_state_and_undo
        ~encode_old:Historical_codec.encode_save39_member0_checkpoint_state
        ~decode_old:Historical_codec.read_save39_member0_checkpoint_state
        ~encode_new:Historical_codec.encode_save40_member0_checkpoint_state state checkpoint undo
    in
    (state, undo)
  in
  match old with
  | `Current old ->
      let state, undo = convert old.checkpoint.state (Some old.undo) in
      `Current
        {
          wire = old.wire;
          pending_enemy = old.pending_enemy;
          undo = Option.get undo;
          progress = old.progress;
          version = `V40;
          checkpoint = { wire = old.checkpoint.wire; seed = old.checkpoint.seed; state };
          setup = old.setup;
        }
  | `Turn old ->
      let state, _ = convert old.checkpoint.state None in
      `Turn
        {
          wire = old.wire;
          pending_enemy = old.pending_enemy;
          undo = None;
          progress = old.progress;
          version = `V40;
          checkpoint = { wire = old.checkpoint.wire; seed = old.checkpoint.seed; state };
          setup = old.setup;
        }
