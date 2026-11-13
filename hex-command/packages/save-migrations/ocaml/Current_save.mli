(* The historical V44 declaration describes candidate saves using the full Rust
   AI capability type. Valid browser saves are narrower: only the three browser
   presets, mission campaign logic, and no recruitmentLimit field. Keep that
   historical contract frozen; this is the validated current output contract. *)
type policy = [ `Classic | `Situational | `Predictor ]

type current_blocks = {
  wire : Historical_types.wire;
  planner : Historical_types.V44.current_planner;
  recruiter : Historical_types.V44.current_recruiter;
  campaign : [ `Mission ];
}

type current_enemy = {
  wire : Historical_types.wire;
  policy : policy;
  diplomacy : Historical_types.V44.current_diplomacy;
  blocks : current_blocks option;
}

type current_setup = {
  wire : Historical_types.wire;
  first : Historical_types.V44.current_first;
  enemy : current_enemy;
  player : policy option;
}

type current_entry = Historical_types.V44.current_entry
type turn_entry = Historical_types.V44.turn_entry
type current_changes = Historical_types.V44.current_changes
type current_undo = Historical_types.V44.current_undo
type current_progress = Historical_types.V44.current_progress
type turn_progress = Historical_types.V44.turn_progress
type current_checkpoint = Historical_types.V44.current_checkpoint

type current = {
  wire : Historical_types.wire;
  pending_enemy : bool;
  undo : current_undo array;
  progress : current_progress;
  version : Historical_types.V44.current_version;
  checkpoint : current_checkpoint;
  setup : current_setup;
}

type turn = {
  wire : Historical_types.wire;
  pending_enemy : [ `False ];
  undo : [ `Undefined ] option;
  progress : turn_progress;
  version : Historical_types.V44.current_version;
  checkpoint : current_checkpoint;
  setup : current_setup;
}

type t = [ `Current of current | `Turn of turn ]

val narrow : Historical_types.V44.t -> t
(** Narrow an already structurally read candidate to browser-supported preferences. Invalid
    engine-only preferences are rejected; none are silently rewritten. *)

val widen : t -> Historical_types.V44.t
val to_json : t -> Js.Json.t
