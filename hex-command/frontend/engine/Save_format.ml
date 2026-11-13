(* Current application wire types. These are maintained independently of the frozen
   migrations. JSON appears only at the persistence boundary and in opaque undo values. *)
type version = [ `V46 ]
type history_version = [ `V4 ]

(* Browser preferences deliberately describe saved UI choices, not every engine
   simulation mode. Setup.ai_options is the explicit adapter to Rust capabilities;
   a constructible saved preference must always survive this module's JSON reader. *)
type policy = [ `Classic | `Situational | `Predictor ]

type blocks = {
  planner : Domain.Planner_kind.t;
  recruiter : Domain.Recruiter_kind.t;
  campaign : [ `Mission ];
}

type enemy = { policy : policy; diplomacy : Domain.Diplomacy.t; blocks : blocks option }
type setup = { first : Domain.Turn_phase.t; enemy : enemy; player : policy option }

type current_entry = {
  kind : [ `Flash ];
  id : string;
  saved_at : float;
  run : string;
  mission : string;
  round : float;
  side : string;
  mission_start : bool;
  run_id : string;
  branch_id : string;
  checkpoint_id : string;
  ordinal : float;
}

type turn_entry = {
  kind : [ `Turn ];
  manual : bool;
  parent_id : string option;
  id : string;
  saved_at : float;
  run : string;
  mission : string;
  round : float;
  side : string;
  mission_start : bool;
  run_id : string;
  branch_id : string;
  checkpoint_id : string;
  ordinal : float;
}

type undo_change =
  | Set of { op : [ `Set ]; path : string array; value : Js.Json.t }
  | Remove of { op : [ `Remove ]; path : string array }

type undo_step = { round : float; unit_id : string; changes : undo_change array }
type current_progress = { entry : current_entry }
type turn_progress = { entry : turn_entry }

type 'checkpoint current_save = {
  pending_enemy : bool;
  undo : undo_step array;
  progress : current_progress;
  version : version;
  checkpoint : 'checkpoint;
  setup : setup;
}

type 'checkpoint turn_save = {
  pending_enemy : [ `False ];
  undo : [ `Undefined ] option;
  progress : turn_progress;
  version : version;
  checkpoint : 'checkpoint;
  setup : setup;
}

type 'checkpoint save = Current of 'checkpoint current_save | Turn of 'checkpoint turn_save
type t = Domain.saved_game save
type history = { version : history_version; entries : turn_entry array }

let version_tag : version = `V46
let history_version_tag : history_version = `V4
let version = match version_tag with `V46 -> 46
let history_version = match history_version_tag with `V4 -> 4
let invalid () = invalid_arg "Invalid current save envelope"
let object_ value = match Js.Json.decodeObject value with Some x -> x | None -> invalid ()
let field value key = match Js.Dict.get (object_ value) key with Some x -> x | None -> invalid ()
let missing value = Js.Types.test value Js.Types.Undefined

let get value key =
  Option.bind (Js.Dict.get (object_ value) key) (fun x -> if missing x then None else Some x)

let text value = match Js.Json.decodeString value with Some x -> Utf8.of_js x | None -> invalid ()

let number value =
  match Js.Json.decodeNumber value with Some x when Float.is_finite x -> x | _ -> invalid ()

let boolean value = match Js.Json.decodeBoolean value with Some x -> x | None -> invalid ()
let array value = match Js.Json.decodeArray value with Some x -> x | None -> invalid ()

external safe_integer : float -> bool = "isSafeInteger" [@@mel.scope "Number"]

let integer value =
  let x = number value in
  if safe_integer x then x else invalid ()

let keys value allowed =
  if Array.exists (fun k -> not (List.mem k allowed)) (Js.Dict.keys (object_ value)) then invalid ()

let parse f value = match f (text value) with Some x -> x | None -> invalid ()
let optional f value key = Option.map f (get value key)
let json_string x = Js.Json.string (Utf8.to_js x)
let json_object xs = Js.Json.object_ (Js.Dict.fromArray (Array.of_list xs))

let setup_of_json value : setup =
  keys value [ "first"; "enemy"; "player" ];
  let first = parse Enum.phase_of_string (field value "first") in
  let source = field value "enemy" in
  keys source [ "policy"; "diplomacy"; "blocks" ];
  let policy =
    match text (field source "policy") with
    | "classic" -> `Classic
    | "situational" -> `Situational
    | "predictor" -> `Predictor
    | _ -> invalid ()
  in
  let diplomacy =
    match text (field source "diplomacy") with
    | "diplomatic" -> Domain.Diplomacy.Diplomatic
    | "unrestricted" -> Domain.Diplomacy.Unrestricted
    | _ -> invalid ()
  in
  let blocks =
    optional
      (fun value ->
        keys value [ "planner"; "recruiter"; "campaign" ];
        if
          Array.length (Js.Dict.keys (object_ value)) <> 3
          || text (field value "campaign") <> "mission"
        then invalid ();
        ({
           planner = parse Enum.planner_kind_of_string (field value "planner");
           recruiter = parse Enum.recruiter_kind_of_string (field value "recruiter");
           campaign = `Mission;
         }
          : blocks))
      source "blocks"
  in
  let player =
    match field value "player" with
    | v when v = Js.Json.null -> None
    | v ->
        Some
          (match text v with
          | "classic" -> `Classic
          | "situational" -> `Situational
          | "predictor" -> `Predictor
          | _ -> invalid ())
  in
  { first; enemy = { policy; diplomacy; blocks }; player }

let setup_to_json ({ first; enemy; player } : setup) =
  let { policy; diplomacy; blocks } = enemy in
  json_object
    [
      ("first", json_string (Enum.string_of_phase first));
      ( "player",
        Option.fold ~none:Js.Json.null
          ~some:(fun p ->
            json_string
              (match p with
              | `Classic -> "classic"
              | `Situational -> "situational"
              | `Predictor -> "predictor"))
          player );
      ( "enemy",
        json_object
          ([
             ( "policy",
               json_string
                 (match policy with
                 | `Classic -> "classic"
                 | `Situational -> "situational"
                 | `Predictor -> "predictor") );
             ( "diplomacy",
               json_string
                 (match diplomacy with
                 | Domain.Diplomacy.Diplomatic -> "diplomatic"
                 | Domain.Diplomacy.Unrestricted -> "unrestricted") );
           ]
          @ Option.fold ~none:[]
              ~some:(fun ({ planner; recruiter; campaign = `Mission } : blocks) ->
                [
                  ( "blocks",
                    json_object
                      [
                        ("planner", json_string (Enum.string_of_planner_kind planner));
                        ("recruiter", json_string (Enum.string_of_recruiter_kind recruiter));
                        ("campaign", json_string "mission");
                      ] );
                ])
              blocks) );
    ]

let entry_keys =
  [
    "id";
    "savedAt";
    "run";
    "mission";
    "round";
    "side";
    "missionStart";
    "runId";
    "branchId";
    "checkpointId";
    "ordinal";
    "kind";
  ]

let current_entry_of_json value : current_entry =
  keys value entry_keys;
  if text (field value "kind") <> "flash" then invalid ();
  let id = text (field value "id") and branch_id = text (field value "branchId") in
  let mission_start = boolean (field value "missionStart") in
  if id <> branch_id || mission_start then invalid ();
  {
    kind = `Flash;
    id;
    saved_at = number (field value "savedAt");
    run = text (field value "run");
    mission = text (field value "mission");
    round = number (field value "round");
    side = text (field value "side");
    mission_start;
    run_id = text (field value "runId");
    branch_id;
    checkpoint_id = text (field value "checkpointId");
    ordinal = integer (field value "ordinal");
  }

let turn_entry_of_json value : turn_entry =
  keys value (entry_keys @ [ "manual"; "parentId" ]);
  if text (field value "kind") <> "turn" then invalid ();
  let id = text (field value "id") and checkpoint_id = text (field value "checkpointId") in
  if id <> checkpoint_id then invalid ();
  {
    kind = `Turn;
    manual = boolean (field value "manual");
    parent_id = optional text value "parentId";
    id;
    saved_at = number (field value "savedAt");
    run = text (field value "run");
    mission = text (field value "mission");
    round = number (field value "round");
    side = text (field value "side");
    mission_start = boolean (field value "missionStart");
    run_id = text (field value "runId");
    branch_id = text (field value "branchId");
    checkpoint_id;
    ordinal = integer (field value "ordinal");
  }

let undo_change_of_json value =
  let path = Array.map text (array (field value "path")) in
  if
    Array.length path = 0
    || Array.exists (fun x -> List.mem x [ "__proto__"; "constructor"; "prototype" ]) path
  then invalid ();
  match text (field value "op") with
  | "set" -> Set { op = `Set; path; value = field value "value" }
  | "remove" -> Remove { op = `Remove; path }
  | _ -> invalid ()

let undo_step_of_json value : undo_step =
  {
    round = integer (field value "round");
    unit_id = text (field value "unitId");
    changes = Array.map undo_change_of_json (array (field value "changes"));
  }

let read ~checkpoint value =
  keys value [ "version"; "checkpoint"; "setup"; "progress"; "pendingEnemy"; "undo" ];
  if number (field value "version") <> float_of_int version then invalid ();
  let setup = setup_of_json (field value "setup") in
  let progress = field value "progress" in
  keys progress [ "entry" ];
  let entry = field progress "entry" in
  let pending_enemy = boolean (field value "pendingEnemy") in
  let raw_checkpoint = field value "checkpoint" in
  keys raw_checkpoint [ "seed"; "state" ];
  let state = field raw_checkpoint "state" in
  let _ = object_ state in
  let _ = array (field state "factions") in
  let _ = array (field state "groundItems") in
  if pending_enemy && (text (field state "turn") <> "player" || field state "winner" <> Js.Json.null)
  then invalid ();
  let checkpoint = checkpoint raw_checkpoint in
  match text (field entry "kind") with
  | "flash" ->
      Current
        {
          pending_enemy;
          checkpoint;
          setup;
          version = version_tag;
          progress = { entry = current_entry_of_json entry };
          undo = Array.map undo_step_of_json (array (field value "undo"));
        }
  | "turn" ->
      if pending_enemy || get value "undo" <> None then invalid ();
      Turn
        {
          pending_enemy = `False;
          undo = None;
          checkpoint;
          setup;
          version = version_tag;
          progress = { entry = turn_entry_of_json entry };
        }
  | _ -> invalid ()

let current_json value =
  let _ = read ~checkpoint:(fun x -> x) value in
  value

let of_json value : t =
  read
    ~checkpoint:(fun value ->
      match Saved_game_json.decode value with Ok x -> x | Error error -> invalid_arg error)
    value

let history_of_json value : history =
  keys value [ "version"; "entries" ];
  if number (field value "version") <> float_of_int history_version then invalid ();
  let entries = Array.map turn_entry_of_json (array (field value "entries")) in
  let ids = Hashtbl.create (Array.length entries) in
  Array.iter
    (fun (entry : turn_entry) ->
      if Hashtbl.mem ids entry.id then invalid ();
      Hashtbl.add ids entry.id ())
    entries;
  { version = history_version_tag; entries }

let checkpoint : t -> Domain.saved_game = function
  | Current value -> value.checkpoint
  | Turn value -> value.checkpoint

let pending_enemy : t -> bool = function Current value -> value.pending_enemy | Turn _ -> false
let setup : t -> setup = function Current value -> value.setup | Turn value -> value.setup
