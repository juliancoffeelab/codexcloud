(* Independent current and frozen contracts. Exhaustive record patterns reject
   added optional fields as well as removed/changed fields. The native module
   equality covers the complete checkpoint graph; the functions below check
   browser-owned wire fields in both directions without JSON casts. *)
[@@@warning "+8+9"]
[@@@warnerror "+8+9"]

module Native_checked : module type of Native_v47 = Domain
module Native_bridge = Migration_native_contract.Make (Domain)
module H = Historical_types
module V = Current_save
module A = Save_format

let side_from : H.V47.current_first -> Domain.Turn_phase.t = function
  | `Player -> Player
  | `Ai -> Ai

let side_to : Domain.Turn_phase.t -> H.V47.current_first = function Player -> `Player | Ai -> `Ai

let policy_from : V.policy -> A.policy = function
  | `Classic -> `Classic
  | `Situational -> `Situational
  | `Predictor -> `Predictor

let policy_to : A.policy -> V.policy = function
  | `Classic -> `Classic
  | `Situational -> `Situational
  | `Predictor -> `Predictor

let diplomacy_from : H.save35_setup_enemy_diplomacy -> Domain.Diplomacy.t = function
  | `Diplomatic -> Diplomatic
  | `Unrestricted -> Unrestricted

let diplomacy_to : Domain.Diplomacy.t -> H.save35_setup_enemy_diplomacy = function
  | Diplomatic -> `Diplomatic
  | Unrestricted -> `Unrestricted

let planner_from : H.save35_setup_enemy_blocks_member1_planner -> Domain.Planner_kind.t = function
  | `Classic -> Classic
  | `Full -> Full
  | `Queued -> Queued

let planner_to : Domain.Planner_kind.t -> H.save35_setup_enemy_blocks_member1_planner = function
  | Classic -> `Classic
  | Full -> `Full
  | Queued -> `Queued

let recruiter_from : H.save35_setup_enemy_blocks_member1_recruiter -> Domain.Recruiter_kind.t =
  function
  | `Classic -> Classic
  | `Rotation -> Rotation
  | `Frontline -> Frontline
  | `Balanced -> Balanced
  | `Situational -> Situational
  | `Predictor -> Predictor
  | `Aggressive_predictor -> Aggressive_predictor

let recruiter_to : Domain.Recruiter_kind.t -> H.save35_setup_enemy_blocks_member1_recruiter =
  function
  | Classic -> `Classic
  | Rotation -> `Rotation
  | Frontline -> `Frontline
  | Balanced -> `Balanced
  | Situational -> `Situational
  | Predictor -> `Predictor
  | Aggressive_predictor -> `Aggressive_predictor

let blocks_from ({ V.wire = _; planner; recruiter; campaign } : V.current_blocks) : A.blocks =
  { planner = planner_from planner; recruiter = recruiter_from recruiter; campaign }

let blocks_to ({ A.planner; recruiter; campaign } : A.blocks) : V.current_blocks =
  {
    wire = H.empty_wire;
    planner = planner_to planner;
    recruiter = recruiter_to recruiter;
    campaign;
  }

let enemy_from ({ V.wire = _; policy; diplomacy; blocks } : V.current_enemy) : A.enemy =
  {
    policy = policy_from policy;
    diplomacy = diplomacy_from diplomacy;
    blocks = Option.map blocks_from blocks;
  }

let enemy_to ({ A.policy; diplomacy; blocks } : A.enemy) : V.current_enemy =
  {
    wire = H.empty_wire;
    policy = policy_to policy;
    diplomacy = diplomacy_to diplomacy;
    blocks = Option.map blocks_to blocks;
  }

let setup_from ({ V.wire = _; first; enemy; player } : V.current_setup) : A.setup =
  { first = side_from first; enemy = enemy_from enemy; player = Option.map policy_from player }

let setup_to ({ A.first; enemy; player } : A.setup) : V.current_setup =
  {
    wire = H.empty_wire;
    first = side_to first;
    enemy = enemy_to enemy;
    player = Option.map policy_to player;
  }

let current_entry_from
    ({
       H.wire = _;
       kind;
       id;
       saved_at;
       run;
       mission;
       round;
       side;
       mission_start;
       run_id;
       branch_id;
       checkpoint_id;
       ordinal;
     } :
      V.current_entry) : A.current_entry =
  {
    kind;
    id;
    saved_at;
    run;
    mission;
    round;
    side;
    mission_start;
    run_id;
    branch_id;
    checkpoint_id;
    ordinal;
  }

let current_entry_to
    ({
       A.kind;
       id;
       saved_at;
       run;
       mission;
       round;
       side;
       mission_start;
       run_id;
       branch_id;
       checkpoint_id;
       ordinal;
     } :
      A.current_entry) : V.current_entry =
  {
    wire = H.empty_wire;
    kind;
    id;
    saved_at;
    run;
    mission;
    round;
    side;
    mission_start;
    run_id;
    branch_id;
    checkpoint_id;
    ordinal;
  }

let turn_entry_from
    ({
       H.wire = _;
       kind;
       manual;
       parent_id;
       id;
       saved_at;
       run;
       mission;
       round;
       side;
       mission_start;
       run_id;
       branch_id;
       checkpoint_id;
       ordinal;
     } :
      V.turn_entry) : A.turn_entry =
  {
    kind;
    manual;
    parent_id;
    id;
    saved_at;
    run;
    mission;
    round;
    side;
    mission_start;
    run_id;
    branch_id;
    checkpoint_id;
    ordinal;
  }

let turn_entry_to
    ({
       A.kind;
       manual;
       parent_id;
       id;
       saved_at;
       run;
       mission;
       round;
       side;
       mission_start;
       run_id;
       branch_id;
       checkpoint_id;
       ordinal;
     } :
      A.turn_entry) : V.turn_entry =
  {
    wire = H.empty_wire;
    kind;
    manual;
    parent_id;
    id;
    saved_at;
    run;
    mission;
    round;
    side;
    mission_start;
    run_id;
    branch_id;
    checkpoint_id;
    ordinal;
  }

let change_from : V.current_changes -> A.undo_change = function
  | `Set { H.wire = _; op; path; value } -> Set { op; path; value }
  | `Remove { H.wire = _; op; path } -> Remove { op; path }

let change_to : A.undo_change -> V.current_changes = function
  | Set { op; path; value } -> `Set { wire = H.empty_wire; op; path; value }
  | Remove { op; path } -> `Remove { wire = H.empty_wire; op; path }

let undo_from ({ H.wire = _; round; unit_id; changes } : V.current_undo) : A.undo_step =
  { round; unit_id; changes = Array.map change_from changes }

let undo_to ({ A.round; unit_id; changes } : A.undo_step) : V.current_undo =
  { wire = H.empty_wire; round; unit_id; changes = Array.map change_to changes }

let current_progress_from ({ H.wire = _; entry } : V.current_progress) : A.current_progress =
  { entry = current_entry_from entry }

let current_progress_to ({ A.entry } : A.current_progress) : V.current_progress =
  { wire = H.empty_wire; entry = current_entry_to entry }

let turn_progress_from ({ H.wire = _; entry } : V.turn_progress) : A.turn_progress =
  { entry = turn_entry_from entry }

let turn_progress_to ({ A.entry } : A.turn_progress) : V.turn_progress =
  { wire = H.empty_wire; entry = turn_entry_to entry }

(* Every checkpoint crosses the exhaustive typed bridge, which also checks the
   generated historical graph against the independently frozen native graph. *)
let current_from
    ({ V.wire = _; pending_enemy; undo; progress; version; checkpoint = saved; setup } : V.current)
    : Domain.saved_game A.current_save =
  {
    pending_enemy;
    undo = Array.map undo_from undo;
    progress = current_progress_from progress;
    version;
    checkpoint = Native_bridge.to_native saved;
    setup = setup_from setup;
  }

let current_to
    ({ A.pending_enemy; undo; progress; version; checkpoint = saved; setup } :
      Domain.saved_game A.current_save) : V.current =
  {
    wire = H.empty_wire;
    pending_enemy;
    undo = Array.map undo_to undo;
    progress = current_progress_to progress;
    version;
    checkpoint = Native_bridge.of_native saved;
    setup = setup_to setup;
  }

let turn_from
    ({ V.wire = _; pending_enemy; undo; progress; version; checkpoint = saved; setup } : V.turn) :
    Domain.saved_game A.turn_save =
  {
    pending_enemy;
    undo;
    progress = turn_progress_from progress;
    version;
    checkpoint = Native_bridge.to_native saved;
    setup = setup_from setup;
  }

let turn_to
    ({ A.pending_enemy; undo; progress; version; checkpoint = saved; setup } :
      Domain.saved_game A.turn_save) : V.turn =
  {
    wire = H.empty_wire;
    pending_enemy;
    undo;
    progress = turn_progress_to progress;
    version;
    checkpoint = Native_bridge.of_native saved;
    setup = setup_to setup;
  }

let save_from : V.t -> A.t = function
  | `Current value -> Current (current_from value)
  | `Turn value -> Turn (turn_from value)

let save_to : A.t -> V.t = function
  | Current value -> `Current (current_to value)
  | Turn value -> `Turn (turn_to value)

let history_from ({ H.wire = _; version; entries } : H.History4.t) : A.history =
  { version; entries = Array.map turn_entry_from entries }

let history_to ({ A.version; entries } : A.history) : H.History4.t =
  { wire = H.empty_wire; version; entries = Array.map turn_entry_to entries }

(* Check the actual migration API as well as the current model, so changing its
   return type cannot leave this contract pointed at an obsolete declaration. *)
let checked_migrate input : A.t * bool =
  let save, changed = Save_migrations.migrate_typed input in
  (save_from save, changed)
