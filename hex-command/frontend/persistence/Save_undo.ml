open Save_json

external property : Js.Json.t -> string -> Js.Json.t = "" [@@mel.get_index]
external has : Js.Json.t -> string -> bool = "hasOwn" [@@mel.scope "Object"]
external same : Js.Json.t -> Js.Json.t -> bool = "is" [@@mel.scope "Object"]
external push : 'a array -> 'a -> int = "push" [@@mel.send]
external keys : Js.Json.t -> string array = "keys" [@@mel.scope "Object"]
external pop : 'a array -> unit = "pop" [@@mel.send]
external is_array : Js.Json.t -> bool = "isArray" [@@mel.scope "Array"]
external array_length : Js.Json.t -> int = "length" [@@mel.get]

let changes current previous initial_path =
  let result = [||] and path = Array.copy initial_path in
  let emit fields =
    let value = obj fields in
    set value "path" (Js.Json.array (Array.map Js.Json.string path));
    let _ = push result value in
    ()
  in
  let rec walk current previous =
    if current != previous then
      let ca = is_array current and pa = is_array previous in
      let record value array = array || Js.Json.decodeObject value <> None in
      if
        record current ca && record previous pa && ca = pa
        && ((not ca) || array_length current = array_length previous)
      then (
        let current_keys = keys current and previous_keys = keys previous in
        for index = 0 to Array.length current_keys - 1 do
          let key = current_keys.(index) in
          if not (has previous key) then (
            let _ = push path key in
            emit [ ("op", Js.Json.string "remove") ];
            pop path)
        done;
        for index = 0 to Array.length previous_keys - 1 do
          let key = previous_keys.(index) in
          let _ = push path key in
          walk (property current key) (property previous key);
          pop path
        done)
      else emit [ ("op", Js.Json.string "set"); ("value", previous) ]
  in
  walk current previous;
  Js.Json.array result

let rec apply value path replacement =
  match path with
  | [] -> Option.get replacement
  | key :: rest -> (
      if List.mem key [ "__proto__"; "constructor"; "prototype" ] then
        invalid_arg "Invalid undo key";
      match Js.Json.decodeArray value with
      | Some source ->
          let result = Array.copy source in
          let index = int_of_string key in
          result.(index) <- apply result.(index) rest replacement;
          Js.Json.array result
      | None ->
          let result = copy value in
          (match (rest, replacement) with
          | [], None ->
              let _ = remove result key in
              ()
          | _ -> set result key (apply (field result key) rest replacement));
          result)

let apply_changes state entries =
  Array.fold_left
    (fun state change ->
      apply state
        (Array.to_list (array (field change "path")) |> List.map host_string)
        (if text change "op" = "set" then Some (field change "value") else None))
    state entries

let checkpoint value =
  match Saved_game_json.decode value with Ok value -> value | Error error -> invalid_arg error

type entry = {
  checkpoint : Domain.saved_game;
  state : Js.Json.t;
  unit : string;
  round : float;
  mutable following : Js.Json.t option;
  mutable encoded : Js.Json.t option;
}

let checkpoint_of_entry entry = entry.checkpoint
let unit_id entry = entry.unit
let round entry = entry.round

let create checkpoint state unit round =
  { checkpoint; state; unit; round; following = None; encoded = None }

let restore save =
  let saved = field save "checkpoint" in
  let state = ref (field saved "state") in
  Array.to_list (array (field save "undo"))
  |> List.rev
  |> List.map (fun step ->
      let following = !state in
      state := apply_changes !state (array (field step "changes"));
      let snapshot = copy saved in
      set snapshot "state" !state;
      {
        checkpoint = checkpoint snapshot;
        state = !state;
        unit = text step "unitId";
        round = n step "round";
        following = Some following;
        encoded = Some step;
      })

let saved checkpoint history =
  let current = ref (field checkpoint "state") in
  let entries =
    List.map
      (fun entry ->
        let encoded =
          match (entry.following, entry.encoded) with
          | Some following, Some encoded when same following !current -> encoded
          | _ ->
              let encoded =
                obj
                  [
                    ("unitId", str entry.unit);
                    ("round", num entry.round);
                    ("changes", changes !current entry.state [||]);
                  ]
              in
              entry.following <- Some !current;
              entry.encoded <- Some encoded;
              encoded
        in
        current := entry.state;
        encoded)
      history
  in
  arr (List.rev entries)

let checkpoint = checkpoint_of_entry
