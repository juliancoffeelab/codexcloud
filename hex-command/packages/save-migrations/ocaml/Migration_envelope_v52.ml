open Migration_json

let version = 52
let history_version = 5
let only value allowed = List.for_all (fun k -> List.mem k allowed) (keys value)

let texts value names =
  List.for_all
    (fun k ->
      Option.fold ~none:false ~some:(fun x -> Option.is_some (Js.Json.decodeString x)) (get value k))
    names

let valid_setup value =
  if not (is_record value && only value [ "first"; "enemy"; "player" ]) then false
  else
    let enemy = default value "enemy" Js.Json.null in
    List.mem (default value "first" Js.Json.null) [ s "player"; s "ai" ]
    && List.mem (field value "player") [ Js.Json.null; s "classic"; s "situational"; s "predictor" ]
    && get value "player" <> None
    && is_record enemy
    && only enemy [ "policy"; "diplomacy"; "blocks" ]
    && List.mem
         (default enemy "policy" Js.Json.null)
         [ s "classic"; s "situational"; s "predictor" ]
    && List.mem (default enemy "diplomacy" Js.Json.null) [ s "diplomatic"; s "unrestricted" ]
    &&
    match present enemy "blocks" with
    | None -> Option.fold ~none:true ~some:is_undefined (get enemy "blocks")
    | Some blocks ->
        is_record blocks
        && List.length (keys blocks) = 3
        && get blocks "campaign" = Some (s "mission")
        && List.mem (default blocks "planner" Js.Json.null) [ s "classic"; s "queued"; s "full" ]
        && List.mem
             (default blocks "recruiter" Js.Json.null)
             (List.map s
                [
                  "classic";
                  "balanced";
                  "frontline";
                  "rotation";
                  "situational";
                  "predictor";
                  "aggressive-predictor";
                ])

let entry_keys =
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
    "mapId";
    "kind";
    "missionStart";
  ]

let valid_entry value =
  if not (is_record value) then false
  else
    let turn = get value "kind" = Some (s "turn") in
    List.mem (default value "kind" Js.Json.null) [ s "turn"; s "flash" ]
    && only value (entry_keys @ if turn then [ "parentId"; "manual" ] else [])
    && texts value [ "id"; "runId"; "branchId"; "checkpointId"; "run"; "mapId" ]
    && (get value "mission" = Some Js.Json.null
       || Option.fold ~none:false
            ~some:(fun chapter -> safe_integer chapter && chapter >= 1.)
            (Option.bind (get value "mission") Js.Json.decodeNumber))
    && List.for_all
         (fun k ->
           Option.fold ~none:false ~some:finite
             (Js.Json.decodeNumber (default value k Js.Json.null)))
         [ "savedAt"; "round" ]
    && Option.fold ~none:false ~some:safe_integer
         (Js.Json.decodeNumber (default value "ordinal" Js.Json.null))
    && Option.is_some (Js.Json.decodeBoolean (default value "missionStart" Js.Json.null))
    &&
    if turn then
      get value "id" = get value "checkpointId"
      && Option.is_some (Js.Json.decodeBoolean (default value "manual" Js.Json.null))
      &&
      match get value "parentId" with
      | None -> true
      | Some x -> x = undefined || Option.is_some (Js.Json.decodeString x)
    else get value "missionStart" = Some (b false) && get value "id" = get value "branchId"

let valid_undo value =
  is_record value && texts value [ "unitId" ]
  && Option.fold ~none:false ~some:safe_integer
       (Js.Json.decodeNumber (default value "round" Js.Json.null))
  && Option.fold ~none:false
       ~some:
         (Array.for_all (fun c ->
              is_record c
              && List.mem (default c "op" Js.Json.null) [ s "set"; s "remove" ]
              && Option.fold ~none:false
                   ~some:(fun xs ->
                     Array.length xs > 0
                     && Array.for_all
                          (fun x ->
                            match Js.Json.decodeString x with
                            | None -> false
                            | Some k -> not (List.mem k [ "__proto__"; "constructor"; "prototype" ]))
                          xs)
                   (Js.Json.decodeArray (default c "path" Js.Json.null))
              && (get c "op" <> Some (s "set") || get c "value" <> None)))
       (Js.Json.decodeArray (default value "changes" Js.Json.null))

let current_save value =
  require
    (is_record value
    && get value "version" = Some (n 52.)
    && only value [ "version"; "checkpoint"; "pendingEnemy"; "setup"; "undo"; "progress" ])
    "Invalid current save envelope";
  let checkpoint = field value "checkpoint" and progress = field value "progress" in
  require
    (valid_setup (field value "setup")
    && is_record progress && only progress [ "entry" ]
    && valid_entry (field progress "entry")
    && Option.is_some (Js.Json.decodeBoolean (field value "pendingEnemy"))
    && is_record checkpoint
    && only checkpoint [ "seed"; "state" ])
    "Invalid current save envelope";
  let state = field checkpoint "state" and entry = field progress "entry" in
  require
    (is_record state
    && get entry "mapId" = get state "mapId"
    && (if default state "campaign" Js.Json.null = Js.Json.null then
          get entry "mission" = Some Js.Json.null
        else
          get entry "mission"
          = Some (n (num (default state "campaign" Js.Json.null) "mission" +. 1.)))
    && Option.is_some (Js.Json.decodeArray (field state "factions"))
    && Option.is_some (Js.Json.decodeArray (field state "groundItems"))
    && (get value "pendingEnemy" <> Some (b true)
       || (get state "turn" = Some (s "player") && get state "winner" = Some Js.Json.null))
    &&
    if text entry "kind" = "turn" then
      get value "pendingEnemy" = Some (b false)
      && Option.fold ~none:true ~some:is_undefined (get value "undo")
    else
      Option.fold ~none:false ~some:(Array.for_all valid_undo)
        (Js.Json.decodeArray (field value "undo")))
    "Invalid current save envelope";
  value

let kind value kind =
  let value = current_save value in
  require
    (text (field (field value "progress") "entry") "kind" = kind)
    ("Expected " ^ kind ^ " save");
  value

let current_flash value = kind value "flash"
let current_turn value = kind value "turn"

let current_history value =
  require
    (is_record value && get value "version" = Some (n 5.) && only value [ "version"; "entries" ])
    "Invalid history index";
  let rows = list value "entries" in
  require
    (List.for_all (fun e -> valid_entry e && get e "kind" = Some (s "turn")) rows
    && List.length rows = List.length (unique (List.map (fun e -> field e "id") rows)))
    "Invalid history index";
  value

let default_setup =
  obj
    [
      ("first", s "player");
      ("enemy", obj [ ("policy", s "classic"); ("diplomacy", s "diplomatic") ]);
    ]
