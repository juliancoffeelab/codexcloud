(* Explicit localStorage-v1 -> IndexedDB-v2 conversion and deletion of blocked
   records. Valid saves are typed; rejected data remains uninterpreted. *)
open Migration_json
module T = Historical_types
module Identity = Migration_identity

type local_source = (string * string option) list
type entry = Identity.entry = Flash_entry of T.V39.current_entry | Turn_entry of T.V39.turn_entry
type claim = Identity.claim
type ownership = Unknown | Known of entry | Conflicting of claim list
type record_kind = Flash | Turn | Index | Other

type repair_issue =
  | Unreadable
  | Format
  | Missing_snapshot
  | Identity_conflict
  | Ownership_conflict
  | Invalid_battle
  | Index_missing
  | Index_invalid
  | Unlisted_snapshots

type source_record = {
  key : string;
  kind : record_kind;
  label : string;
  ownership : ownership;
  evidence : claim list;
  mutable issue : repair_issue option;
  mutable error : string option;
  mutable unlisted_count : int option;
  save : T.V39.t option;
}

type problem_source = Local_storage | Indexed_db
type problem_kind = Data | Full | Blocked | Read | Ready
type load_problem = { source : problem_source; kind : problem_kind; detail : string }

type repair_inventory = {
  records : source_record array;
  errors : string array;
  authoritative : bool;
  completed_elsewhere : bool;
}

type repair_target = Record of string | Campaign of string | All

type imported_storage = {
  current : T.save39_member0 option;
  turns : T.save39_member1 array;
  choose_new_game : bool;
}

type repair_result = { deleted : string list; writes : local_source }

let save_key = "hex-command-battle"
let history_key = "hex-command-progress-index"
let new_game_key = "hex-command-choose-new-game"
let payload_key id = "hex-command-progress:" ^ id

let source_key key =
  List.mem key [ save_key; history_key; new_game_key; save_key ^ "-unrestored" ]
  || starts "hex-command-progress:" key

let source_of_json value : local_source =
  Array.to_list (entries value)
  |> List.map (fun (key, value) ->
      (key, if value = Js.Json.null || is_undefined value then None else Some (string value)))

let source_to_json source =
  obj (List.map (fun (key, value) -> (key, Option.fold ~none:Js.Json.null ~some:s value)) source)

let source_keys source = List.map fst source
let raw source key = Option.join (List.assoc_opt key source)
let parse raw = Js.Json.parseExn (Migration_utf8.to_js raw)
let stringify value = Migration_utf8.of_js (Js.Json.stringify value)

(* Version 37 was deployed in localStorage with the former current-save label.
   This correction belongs to this explicit storage conversion, as upstream. *)
let imported_value_typed value =
  let value = clone value in
  if is_record value && get value "version" = Some (n 37.) then
    Option.iter
      (fun progress ->
        if is_record progress then
          Option.iter
            (fun entry ->
              if is_record entry && get entry "kind" = Some (s "recovery") then
                set entry "kind" (s "flash"))
            (get progress "entry"))
      (get value "progress");
  Current_save.widen (fst (Save_migrations.migrate_typed value))

let imported_value value = Historical_codec.V39.to_json (imported_value_typed value)
let imported raw = imported_value_typed (parse raw)
let entry_of_save = Identity.entry_of_save
let entry_id = Identity.entry_id
let entry_run_id = Identity.entry_run_id
let entry_run = Identity.entry_run
let entry_round = Identity.entry_round
let entry_to_json = Identity.entry_to_json
let is_turn = Identity.is_turn

let migrate_source (source : local_source) : imported_storage =
  let current =
    Option.map
      (fun raw ->
        match imported raw with
        | `Current value -> value
        | `Turn _ -> invalid "Expected the current flash save")
      (raw source save_key)
  in
  let history =
    Option.map (fun raw -> History_migrations.migrate_typed (parse raw)) (raw source history_key)
  in
  let rows =
    Option.fold ~none:[||] ~some:(fun ((index : T.History4.t), _, _) -> index.entries) history
  in
  let indexed =
    Array.to_list (Array.map (fun (entry : T.V39.turn_entry) -> payload_key entry.id) rows)
  and retired =
    Option.fold ~none:[]
      ~some:(fun (_, _, ids) -> Array.to_list (Array.map payload_key ids))
      history
  in
  List.iter
    (fun key ->
      require
        (not
           (starts "hex-command-progress:" key
            && not (List.mem key indexed || List.mem key retired)
           || key = save_key ^ "-unrestored"))
        "Unlisted saved data needs repair before importing.")
    (source_keys source);
  let turns =
    Array.map
      (fun (entry : T.V39.turn_entry) ->
        let payload =
          match raw source (payload_key entry.id) with
          | None -> invalid ("Missing saved turn " ^ entry.id ^ ".")
          | Some value -> value
        in
        let save =
          match imported payload with
          | `Turn value -> value
          | `Current _ -> invalid "Expected a turn snapshot"
        in
        let actual = save.progress.entry in
        require
          (actual.id = entry.id && actual.run_id = entry.run_id)
          ("Saved turn metadata does not match " ^ entry.id ^ ".");
        save)
      rows
  in
  { current; turns; choose_new_game = raw source new_game_key = Some "true" }

let migrate source =
  let value = migrate_source (source_of_json source) in
  obj
    ([
       ( "turns",
         Js.Json.array
           (Array.map (fun value -> Historical_codec.V39.to_json (`Turn value)) value.turns) );
       ("chooseNewGame", b value.choose_new_game);
     ]
    @ Option.fold ~none:[]
        ~some:(fun value -> [ ("current", Historical_codec.V39.to_json (`Current value)) ])
        value.current)

let error_text error =
  match Js.Exn.asJsExn error with
  | Some error -> Option.value (Js.Exn.message error) ~default:"Invalid saved data"
  | None -> (
      match error with Invalid_argument message -> message | _ -> Printexc.to_string error)

let record ?issue ?error ?unlisted_count ?save ?(evidence = []) ~key ~kind ~label ownership =
  { key; kind; label; ownership; evidence; issue; error; unlisted_count; save }

let turn_label entry = "Turn " ^ Js.Float.toString (entry_round entry)
let conflicting claims = List.length (Identity.owners claims) > 1

let reconcile claims payload_entry =
  if conflicting claims then Conflicting claims
  else Option.fold ~none:Unknown ~some:(fun entry -> Known entry) payload_entry

let add_claim claims (claim : claim) = claims @ [ claim ]

let inspect_source (source : local_source) : source_record list =
  let records = ref [] and metadata = ref [] in
  let push value = records := !records @ [ value ] in
  let index_record = ref None in
  let index_problem record error =
    record.issue <- Some Index_invalid;
    record.error <- Some (error_text error)
  in
  (match raw source history_key with
  | Some text ->
      let row = record ~key:history_key ~kind:Index ~label:"Save list" Unknown in
      index_record := Some row;
      (try
         let value = parse text in
         require
           (is_record value
           && Option.is_some (Option.bind (get value "entries") Js.Json.decodeArray))
           "Save list is unreadable.";
         List.iter
           (fun raw ->
             let claim = Identity.from_index (Identity.version value) raw in
             match claim.id with
             | None -> ()
             | Some id ->
                 let claims = Option.value (List.assoc_opt id !metadata) ~default:[] @ [ claim ] in
                 metadata :=
                   if List.mem_assoc id !metadata then
                     List.map (fun (key, old) -> (key, if key = id then claims else old)) !metadata
                   else !metadata @ [ (id, claims) ])
           (list value "entries");
         let _ = History_migrations.migrate_typed value in
         ()
       with error -> index_problem row error);
      push row
  | None ->
      if List.exists (starts "hex-command-progress:") (source_keys source) then (
        let row =
          record ~key:history_key ~kind:Index ~label:"Save list" ~issue:Index_missing
            ~error:"No history index is stored." Unknown
        in
        index_record := Some row;
        push row));
  let payloads =
    unique
      (List.filter
         (fun key ->
           (key = save_key && raw source key <> None) || starts "hex-command-progress:" key)
         (source_keys source)
      @ List.map (fun (id, _) -> payload_key id) !metadata)
  in
  let unlisted = ref 0 in
  List.iter
    (fun key ->
      let flash = key = save_key in
      let id = if flash then "" else suffix "hex-command-progress:" key in
      let claims = if flash then [] else Option.value (List.assoc_opt id !metadata) ~default:[] in
      let kind = if flash then Flash else Turn in
      let label = if flash then "Flash save" else "Unidentified turn save" in
      (* Index rows are hints, never deletion authority. Until the payload proves
       its own complete identity, a single index owner remains unknown. *)
      let initial = reconcile claims None in
      let current =
        ref
          (match initial with
          | Conflicting _ ->
              record ~key ~kind ~label ~evidence:claims ~issue:Ownership_conflict
                ~error:"Index entries assign this snapshot to different campaigns." initial
          | _ -> record ~key ~kind ~label ~evidence:claims initial)
      in
      let stage = ref `Parse in
      (try
         let text =
           match raw source key with
           | Some value -> value
           | None ->
               if not (conflicting claims) then !current.issue <- Some Missing_snapshot;
               invalid "Snapshot missing; only its list entry remains."
         in
         let value = parse text in
         stage := `Convert;
         let payload_claim = Identity.from_payload value in
         let evidence = claims @ Option.to_list payload_claim in
         let ownership = reconcile evidence None in
         current := { !current with ownership; evidence };
         if conflicting evidence then (
           !current.issue <- Some Ownership_conflict;
           !current.error <- Some "The payload and save list contain conflicting ownership claims.");
         (* The ordinary typed migration chain is the only conversion path.
            Failure retains the raw claims above without constructing a save. *)
         let payload = imported_value_typed value in
         let actual_entry = entry_of_save payload in
         let evidence = add_claim evidence (Identity.from_migrated payload) in
         let ownership = reconcile evidence (Some actual_entry) in
         current := { !current with ownership; evidence; save = Some payload };
         if conflicting evidence then (
           !current.issue <- Some Ownership_conflict;
           !current.error <- Some "The payload and save list contain conflicting ownership claims.");
         stage := `Identity;
         require
           (if flash then not (is_turn actual_entry)
            else is_turn actual_entry && entry_id actual_entry = id)
           "Snapshot identity does not match its storage key.";
         match ownership with
         | Known entry ->
             current := { !current with label = (if flash then "Flash save" else turn_label entry) };
             if (not flash) && not (List.mem_assoc id !metadata) then incr unlisted
         | _ -> ()
       with error ->
         if !current.issue = None then
           !current.issue <-
             Some
               (match !stage with
               | `Parse -> Unreadable
               | `Convert -> Format
               | `Identity -> Identity_conflict);
         !current.error <-
           Some (String.concat "\n" (Option.to_list !current.error @ [ error_text error ])));
      push !current)
    payloads;
  Option.iter
    (fun (record : source_record) ->
      if !unlisted > 0 then (
        record.unlisted_count <- Some !unlisted;
        if record.issue = None then (
          record.issue <- Some Unlisted_snapshots;
          record.error <- Some "Identifiable snapshots are absent from the save list.")))
    !index_record;
  if List.mem_assoc (save_key ^ "-unrestored") source then
    push
      (record ~key:(save_key ^ "-unrestored") ~kind:Other ~label:"Unidentified retained save"
         ~issue:Format ~error:"This extra record is not part of the current import." Unknown);
  !records

let issue_name = function
  | Unreadable -> "unreadable"
  | Format -> "format"
  | Missing_snapshot -> "missing_snapshot"
  | Identity_conflict -> "identity_conflict"
  | Ownership_conflict -> "ownership_conflict"
  | Invalid_battle -> "invalid_battle"
  | Index_missing -> "index_missing"
  | Index_invalid -> "index_invalid"
  | Unlisted_snapshots -> "unlisted_snapshots"

let record_to_json (record : source_record) =
  let ownership =
    match record.ownership with
    | Unknown ->
        [ ("ownership", s "unknown") ]
        @
        if record.evidence = [] then []
        else [ ("evidence", a (List.map Identity.to_json record.evidence)) ]
    | Known entry ->
        [
          ("ownership", s "known");
          ("runId", s (entry_run_id entry));
          ("runName", s (entry_run entry));
          ("entry", entry_to_json entry);
        ]
    | Conflicting claims ->
        [ ("ownership", s "conflicting"); ("claims", a (List.map Identity.to_json claims)) ]
  in
  obj
    ([
       ("key", s record.key);
       ( "kind",
         s
           (match record.kind with
           | Flash -> "flash"
           | Turn -> "turn"
           | Index -> "index"
           | Other -> "other") );
       ("label", s record.label);
     ]
    @ ownership
    @ Option.fold ~none:[] ~some:(fun issue -> [ ("issue", s (issue_name issue)) ]) record.issue
    @ Option.fold ~none:[] ~some:(fun error -> [ ("error", s error) ]) record.error
    @ Option.fold ~none:[]
        ~some:(fun count -> [ ("unlistedCount", n (float_of_int count)) ])
        record.unlisted_count
    @ Option.fold ~none:[]
        ~some:(fun save -> [ ("save", Historical_codec.V39.to_json save) ])
        record.save)

let inspect source =
  Array.of_list (List.map record_to_json (inspect_source (source_of_json source)))

(* Damaged rows remain verbatim. A validated string ID alone permits removing a reference. *)
type repair_index_row = Identified of string * Js.Json.t | Unidentified of Js.Json.t

let repair_index_row value =
  match if is_record value then Option.bind (get value "id") Js.Json.decodeString else None with
  | Some id -> Identified (Migration_utf8.of_js id, value)
  | None -> Unidentified value

let target_of_json value =
  match text value "kind" with
  | "all" -> All
  | "record" -> Record (text value "key")
  | "campaign" -> Campaign (text value "runId")
  | _ -> invalid "Invalid save repair target"

let repair_source source target : repair_result =
  let selected =
    match target with
    | All -> source_keys source
    | Record key -> [ key ]
    | Campaign run_id ->
        List.filter_map
          (fun (record : source_record) ->
            match record.ownership with
            | Known entry when entry_run_id entry = run_id -> Some record.key
            | _ -> None)
          (inspect_source source)
  in
  require (List.for_all source_key selected) "This is not a save storage record.";
  let writes = ref [] in
  let write key value = writes := !writes @ [ (key, Some value) ] in
  if target = All || List.mem save_key selected then write new_game_key "true";
  if target <> All && not (List.mem history_key selected) then
    Option.iter
      (fun raw ->
        try
          let index = parse raw in
          if is_record index then
            Option.iter
              (fun rows ->
                let retained =
                  Array.to_list rows
                  |> List.filter_map (fun row ->
                      match repair_index_row row with
                      | Unidentified raw -> Some raw
                      | Identified (id, raw) ->
                          if List.mem (payload_key id) selected then None else Some raw)
                in
                if Array.length rows <> List.length retained then
                  write history_key (stringify (update index [ ("entries", a retained) ])))
              (Option.bind (get index "entries") Js.Json.decodeArray)
        with _ -> ())
      (raw source history_key);
  { deleted = selected; writes = !writes }

let repair source target =
  let value = repair_source (source_of_json source) (target_of_json target) in
  obj [ ("deleted", a (List.map s value.deleted)); ("writes", source_to_json value.writes) ]
