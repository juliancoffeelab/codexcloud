(* Frozen, migration-owned recipes. These types mirror the declarations local to
   upstream v29->30 and v33->34; no current game data participates. *)
type timer = { event : string; source : string; tile : string; turns : float }
type missing_character = { character : string; message : string }
type 'recruit branch = { flag : string; recruitment : (string * 'recruit array) array }

type 'recruit recipe = {
  active : string array;
  previous_event_ids : string array;
  objectives : (string * string) array;
  required : string array;
  old_events : string array;
  waves : string array;
  repeat : string array;
  retire_on_restore : string array;
  timers : timer array;
  seals : (string * string) array;
  named : string array;
  first_seen : (string * float) array;
  failure_units : (string * string array) array;
  departed_when_flag : (string * string) array;
  defeat_on_missing : missing_character array;
  branch : 'recruit branch option;
  base_recruitment : (string * 'recruit array) array;
}

type condition =
  | Flag of string
  | All of condition array
  | Any of condition array
  | Not of condition

type objective = { event : string; id : string; visible_when : condition }
type additions = { added : string array; order : string array }

let strings value = Array.map Migration_json.string (Migration_json.array value)

let dictionary decode value =
  Array.map (fun (key, value) -> (key, decode value)) (Migration_json.entries value)

let recipe recruit value =
  let open Migration_json in
  {
    active = strings (field value "active");
    previous_event_ids = strings (field value "previousEventIds");
    objectives = dictionary string (field value "objectives");
    required = strings (field value "required");
    old_events = strings (field value "oldEvents");
    waves = strings (field value "waves");
    repeat = strings (field value "repeat");
    retire_on_restore = strings (field value "retireOnRestore");
    timers =
      Array.map
        (fun timer ->
          ({
             event = text timer "event";
             source = text timer "source";
             tile = text timer "tile";
             turns = num timer "turns";
           }
            : timer))
        (array (field value "timers"));
    seals = dictionary string (field value "seals");
    named = strings (field value "named");
    first_seen = dictionary number (field value "firstSeen");
    failure_units = dictionary strings (field value "failureUnits");
    departed_when_flag = dictionary string (field value "departedWhenFlag");
    defeat_on_missing =
      Array.map
        (fun missing ->
          ({ character = text missing "character"; message = text missing "message" }
            : missing_character))
        (array (field value "defeatOnMissing"));
    branch =
      Option.map
        (fun branch ->
          {
            flag = text branch "flag";
            recruitment =
              dictionary (fun value -> Array.map recruit (array value)) (field branch "recruitment");
          })
        (present value "branch");
    base_recruitment =
      dictionary (fun value -> Array.map recruit (array value)) (field value "baseRecruitment");
  }

let rec condition value =
  let open Migration_json in
  match (get value "flag", get value "all", get value "any") with
  | Some flag, _, _ -> Flag (string flag)
  | _, Some values, _ -> All (Array.map condition (array values))
  | _, _, Some values -> Any (Array.map condition (array values))
  | _ -> Not (condition (field value "not"))

let objective value =
  let open Migration_json in
  {
    event = text value "event";
    id = text value "id";
    visible_when = condition (field value "visibleWhen");
  }

let additions value =
  {
    added = strings (Migration_json.field value "added");
    order = strings (Migration_json.field value "order");
  }

let get values key = Option.map snd (Array.find_opt (fun (name, _) -> name = key) values)

let set values key value =
  if Array.exists (fun (name, _) -> name = key) values then
    Array.map (fun (name, old) -> (name, if name = key then value else old)) values
  else Array.append values [| (key, value) |]

let remove values key =
  Array.of_list (List.filter (fun (name, _) -> name <> key) (Array.to_list values))

let unique values =
  List.fold_left
    (fun result value -> if List.mem value result then result else result @ [ value ])
    [] values

let starts prefix value = String.starts_with ~prefix value

let suffix prefix value =
  String.sub value (String.length prefix) (String.length value - String.length prefix)
