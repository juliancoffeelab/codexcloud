(* Current application wire types. These are maintained independently of the frozen
   migrations. JSON appears only at the persistence boundary and in opaque undo values. *)
type version = [ `V52 ]
type history_version = [ `V5 ]

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
  mission : float option;
  round : float;
  map_id : string;
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
  mission : float option;
  round : float;
  map_id : string;
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

let version_tag : version = `V52
let history_version_tag : history_version = `V5
let version = match version_tag with `V52 -> 52
let history_version = match history_version_tag with `V5 -> 5
let invalid () = invalid_arg {js|Invalid current save envelope|js}
let object_ value = match Js.Json.decodeObject value with Some x -> x | None -> invalid ()
let field value key = match Js.Dict.get (object_ value) key with Some x -> x | None -> invalid ()
let missing value = Js.Types.test value Js.Types.Undefined

let get value key =
  Option.bind (Js.Dict.get (object_ value) key) (fun x -> if missing x then None else Some x)

let text value = match Js.Json.decodeString value with Some x -> x | None -> invalid ()

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
let json_string x = Js.Json.string x
let json_object xs = Js.Json.object_ (Js.Dict.fromArray (Array.of_list xs))

let setup_of_json value : setup =
  keys value [ {js|first|js}; {js|enemy|js}; {js|player|js} ];
  let first = parse Enum.phase_of_string (field value {js|first|js}) in
  let source = field value {js|enemy|js} in
  keys source [ {js|policy|js}; {js|diplomacy|js}; {js|blocks|js} ];
  let policy =
    match text (field source {js|policy|js}) with
    | matched_text_0 when matched_text_0 = {js|classic|js} -> `Classic
    | matched_text_1 when matched_text_1 = {js|situational|js} -> `Situational
    | matched_text_2 when matched_text_2 = {js|predictor|js} -> `Predictor
    | _ -> invalid ()
  in
  let diplomacy =
    match text (field source {js|diplomacy|js}) with
    | matched_text_3 when matched_text_3 = {js|diplomatic|js} -> Domain.Diplomacy.Diplomatic
    | matched_text_4 when matched_text_4 = {js|unrestricted|js} -> Domain.Diplomacy.Unrestricted
    | _ -> invalid ()
  in
  let blocks =
    optional
      (fun value ->
        keys value [ {js|planner|js}; {js|recruiter|js}; {js|campaign|js} ];
        if
          Array.length (Js.Dict.keys (object_ value)) <> 3
          || text (field value {js|campaign|js}) <> {js|mission|js}
        then invalid ();
        ({
           planner =
             (match parse Enum.planner_kind_of_string (field value {js|planner|js}) with
             | Domain.Planner_kind.Learned -> invalid ()
             | value -> value);
           recruiter = parse Enum.recruiter_kind_of_string (field value {js|recruiter|js});
           campaign = `Mission;
         }
          : blocks))
      source {js|blocks|js}
  in
  let player =
    match field value {js|player|js} with
    | v when v = Js.Json.null -> None
    | v ->
        Some
          (match text v with
          | matched_text_5 when matched_text_5 = {js|classic|js} -> `Classic
          | matched_text_6 when matched_text_6 = {js|situational|js} -> `Situational
          | matched_text_7 when matched_text_7 = {js|predictor|js} -> `Predictor
          | _ -> invalid ())
  in
  { first; enemy = { policy; diplomacy; blocks }; player }

let setup_to_json ({ first; enemy; player } : setup) =
  let { policy; diplomacy; blocks } = enemy in
  json_object
    [
      ({js|first|js}, json_string (Enum.string_of_phase first));
      ( {js|player|js},
        Option.fold ~none:Js.Json.null
          ~some:(fun p ->
            json_string
              (match p with
              | `Classic -> {js|classic|js}
              | `Situational -> {js|situational|js}
              | `Predictor -> {js|predictor|js}))
          player );
      ( {js|enemy|js},
        json_object
          ([
             ( {js|policy|js},
               json_string
                 (match policy with
                 | `Classic -> {js|classic|js}
                 | `Situational -> {js|situational|js}
                 | `Predictor -> {js|predictor|js}) );
             ( {js|diplomacy|js},
               json_string
                 (match diplomacy with
                 | Domain.Diplomacy.Diplomatic -> {js|diplomatic|js}
                 | Domain.Diplomacy.Unrestricted -> {js|unrestricted|js}) );
           ]
          @ Option.fold ~none:[]
              ~some:(fun ({ planner; recruiter; campaign = `Mission } : blocks) ->
                if planner = Domain.Planner_kind.Learned then invalid ();
                [
                  ( {js|blocks|js},
                    json_object
                      [
                        ({js|planner|js}, json_string (Enum.string_of_planner_kind planner));
                        ({js|recruiter|js}, json_string (Enum.string_of_recruiter_kind recruiter));
                        ({js|campaign|js}, json_string {js|mission|js});
                      ] );
                ])
              blocks) );
    ]

let entry_keys =
  [
    {js|id|js};
    {js|savedAt|js};
    {js|run|js};
    {js|mission|js};
    {js|round|js};
    {js|mapId|js};
    {js|missionStart|js};
    {js|runId|js};
    {js|branchId|js};
    {js|checkpointId|js};
    {js|ordinal|js};
    {js|kind|js};
  ]

let chapter value =
  if value = Js.Json.null then None
  else
    let number = integer value in
    if number < 1. then invalid () else Some number

let current_entry_of_json value : current_entry =
  keys value entry_keys;
  if text (field value {js|kind|js}) <> {js|flash|js} then invalid ();
  let id = text (field value {js|id|js}) and branch_id = text (field value {js|branchId|js}) in
  let mission_start = boolean (field value {js|missionStart|js}) in
  if id <> branch_id || mission_start then invalid ();
  {
    kind = `Flash;
    id;
    saved_at = number (field value {js|savedAt|js});
    run = text (field value {js|run|js});
    mission = chapter (field value {js|mission|js});
    round = number (field value {js|round|js});
    map_id = text (field value {js|mapId|js});
    mission_start;
    run_id = text (field value {js|runId|js});
    branch_id;
    checkpoint_id = text (field value {js|checkpointId|js});
    ordinal = integer (field value {js|ordinal|js});
  }

let turn_entry_of_json value : turn_entry =
  keys value (entry_keys @ [ {js|manual|js}; {js|parentId|js} ]);
  if text (field value {js|kind|js}) <> {js|turn|js} then invalid ();
  let id = text (field value {js|id|js})
  and checkpoint_id = text (field value {js|checkpointId|js}) in
  if id <> checkpoint_id then invalid ();
  {
    kind = `Turn;
    manual = boolean (field value {js|manual|js});
    parent_id = optional text value {js|parentId|js};
    id;
    saved_at = number (field value {js|savedAt|js});
    run = text (field value {js|run|js});
    mission = chapter (field value {js|mission|js});
    round = number (field value {js|round|js});
    map_id = text (field value {js|mapId|js});
    mission_start = boolean (field value {js|missionStart|js});
    run_id = text (field value {js|runId|js});
    branch_id = text (field value {js|branchId|js});
    checkpoint_id;
    ordinal = integer (field value {js|ordinal|js});
  }

let undo_change_of_json value =
  let path = Array.map text (array (field value {js|path|js})) in
  if
    Array.length path = 0
    || Array.exists
         (fun x -> List.mem x [ {js|__proto__|js}; {js|constructor|js}; {js|prototype|js} ])
         path
  then invalid ();
  match text (field value {js|op|js}) with
  | matched_text_8 when matched_text_8 = {js|set|js} ->
      Set { op = `Set; path; value = field value {js|value|js} }
  | matched_text_9 when matched_text_9 = {js|remove|js} -> Remove { op = `Remove; path }
  | _ -> invalid ()

let undo_step_of_json value : undo_step =
  {
    round = integer (field value {js|round|js});
    unit_id = text (field value {js|unitId|js});
    changes = Array.map undo_change_of_json (array (field value {js|changes|js}));
  }

let read ~checkpoint value =
  keys value
    [
      {js|version|js};
      {js|checkpoint|js};
      {js|setup|js};
      {js|progress|js};
      {js|pendingEnemy|js};
      {js|undo|js};
    ];
  if number (field value {js|version|js}) <> float_of_int version then invalid ();
  let setup = setup_of_json (field value {js|setup|js}) in
  let progress = field value {js|progress|js} in
  keys progress [ {js|entry|js} ];
  let entry = field progress {js|entry|js} in
  let pending_enemy = boolean (field value {js|pendingEnemy|js}) in
  let raw_checkpoint = field value {js|checkpoint|js} in
  keys raw_checkpoint [ {js|seed|js}; {js|state|js} ];
  let state = field raw_checkpoint {js|state|js} in
  let _ = object_ state in
  let _ = array (field state {js|factions|js}) in
  let _ = array (field state {js|groundItems|js}) in
  if
    pending_enemy
    && (text (field state {js|turn|js}) <> {js|player|js}
       || field state {js|winner|js} <> Js.Json.null)
  then invalid ();
  let map_id = text (field entry {js|mapId|js}) in
  let mission = chapter (field entry {js|mission|js}) in
  let campaign = Option.value ~default:Js.Json.null (get state {js|campaign|js}) in
  let expected =
    if campaign = Js.Json.null then None else Some (number (field campaign {js|mission|js}) +. 1.)
  in
  if map_id <> text (field state {js|mapId|js}) || mission <> expected then invalid ();
  let checkpoint = checkpoint raw_checkpoint in
  match text (field entry {js|kind|js}) with
  | matched_text_10 when matched_text_10 = {js|flash|js} ->
      Current
        {
          pending_enemy;
          checkpoint;
          setup;
          version = version_tag;
          progress = { entry = current_entry_of_json entry };
          undo = Array.map undo_step_of_json (array (field value {js|undo|js}));
        }
  | matched_text_11 when matched_text_11 = {js|turn|js} ->
      if pending_enemy || get value {js|undo|js} <> None then invalid ();
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
  keys value [ {js|version|js}; {js|entries|js} ];
  if number (field value {js|version|js}) <> float_of_int history_version then invalid ();
  let entries = Array.map turn_entry_of_json (array (field value {js|entries|js})) in
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
