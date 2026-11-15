(* V48 -> V49 and history V4 -> V5: persist chapter facts, not formatted labels. *)
open Historical_types
open Migration_json

let entry_wire wire =
  let wire = remove_fields wire [ {js|side|js}; {js|mission|js}; {js|mapId|js} ] in
  { wire with order = wire.order @ [ {js|mission|js}; {js|mapId|js} ] }

let flash_entry (old : V48.current_entry) map_id mission : V49.current_entry =
  {
    wire = entry_wire old.wire;
    kind = old.kind;
    id = old.id;
    saved_at = old.saved_at;
    run = old.run;
    map_id;
    mission;
    round = old.round;
    mission_start = old.mission_start;
    run_id = old.run_id;
    branch_id = old.branch_id;
    checkpoint_id = old.checkpoint_id;
    ordinal = old.ordinal;
  }

let turn_entry (old : V48.turn_entry) map_id mission : V49.turn_entry =
  {
    wire = entry_wire old.wire;
    kind = old.kind;
    manual = old.manual;
    parent_id = old.parent_id;
    id = old.id;
    saved_at = old.saved_at;
    run = old.run;
    map_id;
    mission;
    round = old.round;
    mission_start = old.mission_start;
    run_id = old.run_id;
    branch_id = old.branch_id;
    checkpoint_id = old.checkpoint_id;
    ordinal = old.ordinal;
  }

let chapter (saved : V48.current_checkpoint) =
  match saved.state.campaign with None -> `Null | Some campaign -> `Number (campaign.mission +. 1.)

let v48 (old : V48.t) : V49.t =
  let _ = Migration_envelope_v48.current_save (Historical_codec.V48.to_json old) in
  Historical_codec.V49.of_legacy
    (match old with
    | `Current value ->
        `Current
          {
            wire = value.wire;
            version = `V49;
            checkpoint = value.checkpoint;
            setup = value.setup;
            pending_enemy = value.pending_enemy;
            undo = value.undo;
            progress =
              {
                wire = value.progress.wire;
                entry =
                  flash_entry value.progress.entry value.checkpoint.state.map_id
                    (chapter value.checkpoint);
              };
          }
    | `Turn value ->
        `Turn
          {
            wire = value.wire;
            version = `V49;
            checkpoint = value.checkpoint;
            setup = value.setup;
            pending_enemy = value.pending_enemy;
            undo = value.undo;
            progress =
              {
                wire = value.progress.wire;
                entry =
                  turn_entry value.progress.entry value.checkpoint.state.map_id
                    (chapter value.checkpoint);
              };
          })

(* Old index rows have no checkpoint. Interpret their published label grammar here. *)
let indexed_position label =
  if Js.String.startsWith ~prefix:{js|Mission |js} label then
    match Array.to_list (Js.String.split ~sep:{js| |js} label) with
    | prefix :: number :: separator :: map
      when prefix = {js|Mission|js} && separator = {js|·|js}
           && Js.Re.test ~str:number (Js.Re.fromString {js|^[1-9][0-9]*$|js})
           && map <> [] ->
        let chapter = Js.Float.fromString number in
        let map_id = Js.Array.join ~sep:{js| |js} (Array.of_list map) in
        require
          (safe_integer chapter && map_id <> {js||js})
          {js|Invalid historical mission metadata|js};
        (map_id, `Number chapter)
    | _ -> invalid {js|Invalid historical mission metadata|js}
  else (
    require (label <> {js||js}) {js|Invalid historical map metadata|js};
    (label, `Null))

let history_v4 (old : History4.t) : History5.t =
  let _ = Migration_envelope_v48.current_history (Historical_codec.History4.to_json old) in
  let old = Historical_codec.History4.of_legacy old in
  {
    wire = old.wire;
    version = `V5;
    entries =
      Array.map
        (fun (entry : History4.entry) ->
          let map_id, mission = indexed_position entry.mission in
          turn_entry entry map_id mission)
        old.entries;
  }
