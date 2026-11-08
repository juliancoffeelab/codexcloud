(* The historical V39 declaration describes candidate saves using the full Rust
   AI capability type. Valid browser saves are narrower: only the three browser
   presets, mission campaign logic, and no recruitmentLimit field. Keep that
   historical contract frozen; this is the validated current output contract. *)
type policy = [ `Classic | `Situational | `Predictor ]

type current_blocks = {
  wire : Historical_types.wire;
  planner : Historical_types.V39.current_planner;
  recruiter : Historical_types.V39.current_recruiter;
  campaign : [ `Mission ];
}

type current_enemy = {
  wire : Historical_types.wire;
  policy : policy;
  diplomacy : Historical_types.V39.current_diplomacy;
  blocks : current_blocks option;
}

type current_setup = {
  wire : Historical_types.wire;
  first : Historical_types.V39.current_first;
  enemy : current_enemy;
}

type current_entry = Historical_types.V39.current_entry
type turn_entry = Historical_types.V39.turn_entry
type current_changes = Historical_types.V39.current_changes
type current_undo = Historical_types.V39.current_undo
type current_progress = Historical_types.V39.current_progress
type turn_progress = Historical_types.V39.turn_progress
type current_checkpoint = Historical_types.V39.current_checkpoint

type current = {
  wire : Historical_types.wire;
  pending_enemy : bool;
  undo : current_undo array;
  progress : current_progress;
  version : Historical_types.V39.current_version;
  checkpoint : current_checkpoint;
  setup : current_setup;
}

type turn = {
  wire : Historical_types.wire;
  pending_enemy : [ `False ];
  undo : [ `Undefined ] option;
  progress : turn_progress;
  version : Historical_types.V39.current_version;
  checkpoint : current_checkpoint;
  setup : current_setup;
}

type t = [ `Current of current | `Turn of turn ]

[@@@warning "+8+9"]
[@@@warnerror "+8+9"]

module H = Historical_types
module V = Historical_types.V39

let invalid () = invalid_arg "Invalid saved browser preferences"

let narrow_policy : V.current_policy -> policy = function
  | `Classic -> `Classic
  | `Situational -> `Situational
  | `Predictor -> `Predictor
  | `Rush | `Balanced | `Rotated -> invalid ()

let widen_policy : policy -> V.current_policy = function
  | `Classic -> `Classic
  | `Situational -> `Situational
  | `Predictor -> `Predictor

let narrow_blocks ({ H.wire; planner; recruiter; campaign } : V.current_blocks) : current_blocks =
  let campaign = match campaign with `Mission -> `Mission | `Arena -> invalid () in
  { wire; planner; recruiter; campaign }

let widen_blocks ({ wire; planner; recruiter; campaign } : current_blocks) : V.current_blocks =
  { wire; planner; recruiter; campaign = (match campaign with `Mission -> `Mission) }

let narrow_enemy ({ H.wire; policy; diplomacy; blocks; recruitment_limit } : V.current_enemy) :
    current_enemy =
  if recruitment_limit <> None then invalid ();
  { wire; policy = narrow_policy policy; diplomacy; blocks = Option.map narrow_blocks blocks }

let widen_enemy ({ wire; policy; diplomacy; blocks } : current_enemy) : V.current_enemy =
  {
    wire;
    policy = widen_policy policy;
    diplomacy;
    blocks = Option.map widen_blocks blocks;
    recruitment_limit = None;
  }

let narrow_setup ({ H.wire; first; enemy } : V.current_setup) : current_setup =
  { wire; first; enemy = narrow_enemy enemy }

let widen_setup ({ wire; first; enemy } : current_setup) : V.current_setup =
  { wire; first; enemy = widen_enemy enemy }

let narrow : V.t -> t = function
  | `Current { H.wire; pending_enemy; undo; progress; version; checkpoint; setup } ->
      `Current
        { wire; pending_enemy; undo; progress; version; checkpoint; setup = narrow_setup setup }
  | `Turn { H.wire; pending_enemy; undo; progress; version; checkpoint; setup } ->
      `Turn { wire; pending_enemy; undo; progress; version; checkpoint; setup = narrow_setup setup }

let widen : t -> V.t = function
  | `Current { wire; pending_enemy; undo; progress; version; checkpoint; setup } ->
      `Current
        { wire; pending_enemy; undo; progress; version; checkpoint; setup = widen_setup setup }
  | `Turn { wire; pending_enemy; undo; progress; version; checkpoint; setup } ->
      `Turn { wire; pending_enemy; undo; progress; version; checkpoint; setup = widen_setup setup }

let to_json value = Historical_codec.V39.to_json (widen value)
