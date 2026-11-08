open Historical_types

let entry_info_order =
  [
    "id";
    "runId";
    "branchId";
    "checkpointId";
    "ordinal";
    "savedAt";
    "run";
    "mission";
    "round";
    "side";
    "missionStart";
  ]

let fresh_wire keys = { empty_wire with order = keys }

let v1_to_v2 (value : History1.t) : History2.t * string array =
  let removed = ref [] in
  let entries =
    Array.to_list value.entries
    |> List.filter_map (fun (entry : history1_entries_item) ->
        if entry.kind <> `Turn then (
          removed := !removed @ [ entry.id ];
          None)
        else
          Some
            ({
               wire =
                 fresh_wire
                   (entry_info_order @ [ "kind" ]
                   @ if Option.is_some entry.parent_id then [ "parentId" ] else []);
               id = entry.id;
               saved_at = entry.saved_at;
               run = entry.run;
               mission = entry.mission;
               round = entry.round;
               side = entry.side;
               mission_start = entry.mission_start;
               run_id = entry.run_id;
               branch_id = entry.branch_id;
               checkpoint_id = entry.checkpoint_id;
               ordinal = entry.ordinal;
               kind = `Turn;
               parent_id = entry.parent_id;
             }
              : history2_entries_item))
    |> Array.of_list
  in
  ( { wire = fresh_wire [ "version"; "entries" ]; version = `V2; entries },
    Array.of_list
      (List.filter
         (fun id ->
           not (Array.exists (fun (entry : history2_entries_item) -> entry.id = id) entries))
         !removed) )

let v2_to_v3 (value : History2.t) : History3.t =
  let entries =
    Array.map
      (fun (entry : history2_entries_item) ->
        ({
           wire = entry.wire;
           id = entry.id;
           saved_at = entry.saved_at;
           run = entry.run;
           mission = entry.mission;
           round = entry.round;
           side = entry.side;
           mission_start = entry.mission_start;
           run_id = entry.run_id;
           branch_id = entry.branch_id;
           checkpoint_id = entry.checkpoint_id;
           ordinal = entry.ordinal;
           kind = `Turn;
           parent_id = entry.parent_id;
         }
          : history3_entries_item))
      value.entries
  in
  { wire = fresh_wire [ "version"; "entries" ]; version = `V3; entries }

let v3_to_v4 (value : History3.t) : History4.t =
  let entries =
    Array.map
      (fun (entry : history3_entries_item) ->
        ({
           wire = merge_wire entry.wire (fresh_wire [ "manual" ]);
           kind = `Turn;
           manual = entry.kind = `Manual;
           parent_id = entry.parent_id;
           id = entry.id;
           saved_at = entry.saved_at;
           run = entry.run;
           mission = entry.mission;
           round = entry.round;
           side = entry.side;
           mission_start = entry.mission_start;
           run_id = entry.run_id;
           branch_id = entry.branch_id;
           checkpoint_id = entry.checkpoint_id;
           ordinal = entry.ordinal;
         }
          : history4_entries_item))
      value.entries
  in
  { wire = fresh_wire [ "version"; "entries" ]; version = `V4; entries }

let migrate_typed input =
  let original = Migration_json.num input "version" in
  let index, removed =
    match original with
    | 1. ->
        let value, removed = v1_to_v2 (Historical_codec.History1.of_json input) in
        (v3_to_v4 (v2_to_v3 value), removed)
    | 2. -> (v3_to_v4 (v2_to_v3 (Historical_codec.History2.of_json input)), [||])
    | 3. -> (v3_to_v4 (Historical_codec.History3.of_json input), [||])
    | 4. -> (Historical_codec.History4.of_json input, [||])
    | _ -> invalid_arg "Invalid saved history index"
  in
  let _ = Migration_envelope.current_history (Historical_codec.History4.to_json index) in
  (index, original <> 4., removed)

let migrate input =
  let index, migrated, removed = migrate_typed input in
  let index = Historical_codec.History4.to_json index in
  Migration_json.obj
    [
      ("index", index);
      ("migrated", Migration_json.b migrated);
      ("removedIds", Js.Json.array (Array.map Migration_json.s removed));
    ]
