open Save_json

let issue row =
  match text row "issue" with
  | "unreadable" -> "Couldn’t read this save"
  | "format" -> "This save’s format isn’t recognized"
  | "missing_snapshot" -> "Snapshot missing"
  | "identity_conflict" -> "The snapshot doesn’t match its save entry"
  | "ownership_conflict" -> "Conflicting campaign information"
  | "invalid_battle" -> "The saved battle failed validation"
  | "index_missing" -> "The save list is missing"
  | "index_invalid" -> "The save list is damaged"
  | "unlisted_snapshots" -> "Turns are missing from the save list"
  | _ -> ""

let record_name catalog row =
  match text row "kind" with
  | "flash" -> if text row "ownership" = "known" then "Flash save" else "Unidentified flash save"
  | "index" -> "Save list"
  | _ ->
      let entry = field row "entry" in
      if text entry "kind" = "turn" then Save_browser.turn_name catalog entry
      else if text row "kind" = "turn" then "Unidentified turn save"
      else "Unidentified save"

type group = { id : string; name : string; records : Js.Json.t array }

type model = {
  flash : Js.Json.t option;
  attention : Js.Json.t array;
  campaigns : group array;
  can_continue : bool;
  ready : bool;
  summary : string;
  continuation : string;
  hint : string;
}

let model catalog inventory problem =
  let records = array (field inventory "records") in
  let flash = Array.find_opt (fun r -> text r "kind" = "flash") records in
  let attention =
    Array.to_list records |> List.filter (fun r -> get r "issue" <> None) |> Array.of_list
  in
  let priority r = match text r "kind" with "flash" -> 0 | "index" -> 1 | _ -> 2 in
  Array.stable_sort (fun a b -> Int.compare (priority a) (priority b)) attention;
  let groups = Hashtbl.create 8 and order = ref [] in
  Array.iter
    (fun row ->
      if text row "ownership" = "known" && text row "kind" <> "index" then (
        let id = text row "runId" in
        if not (Hashtbl.mem groups id) then order := !order @ [ id ];
        Hashtbl.replace groups id (row :: Option.value ~default:[] (Hashtbl.find_opt groups id))))
    records;
  let campaigns =
    List.map
      (fun id ->
        let rows = Hashtbl.find groups id |> List.rev |> Array.of_list in
        Array.stable_sort (fun a b -> Save_browser.order (field a "entry") (field b "entry")) rows;
        { id; name = Save_browser.run_name catalog (field rows.(0) "entry"); records = rows })
      !order
    |> Array.of_list
  in
  let errors = array (field inventory "errors") in
  let can_continue = Array.length errors = 0 && Array.length attention = 0 in
  let ready = b inventory "authoritative" && can_continue in
  let unknown = Option.fold ~none:false ~some:(fun f -> text f "ownership" <> "known") flash in
  let kind = Option.fold ~none:"" ~some:(fun p -> text p "kind") problem in
  let summary =
    if Array.length errors > 0 then
      "Saved data couldn’t be checked completely. You can review what was readable below."
    else if ready then
      if b inventory "completedElsewhere" then "Another tab finished updating your saves."
      else if Array.exists (fun r -> text r "kind" <> "index") records then
        "Your saved games can now be read."
      else "No retained saves remain."
    else if kind = "full" then
      "There isn’t enough space to finish updating saved games. Delete saves you no longer need, \
       then finish the update."
    else if unknown then "We can’t identify the flash save’s campaign."
    else if Array.exists (fun r -> text r "kind" = "index") attention then
      "The save list needs attention."
    else if Array.length attention = 1 && text attention.(0) "issue" = "missing_snapshot" then
      "One turn is listed, but its snapshot is missing."
    else if Array.length attention = 1 && text attention.(0) "issue" = "ownership_conflict" then
      "One turn has conflicting campaign information."
    else if Array.length attention = 1 && text attention.(0) "kind" = "turn" then
      "One turn save can’t be restored."
    else if Array.length attention > 0 then
      string_of_int (Array.length attention) ^ " saved records need attention."
    else if Array.length records = 0 then "No retained saves remain."
    else "The remaining saved games are ready to update."
  in
  let summary =
    summary
    ^
    if ready && flash = None then " There is no flash save to continue."
    else if
      (not ready)
      && Option.fold ~none:false ~some:(fun f -> get f "issue" = None) flash
      && kind <> "full"
      && Array.length errors = 0
    then " Your flash save is intact."
    else ""
  in
  {
    flash;
    attention;
    campaigns;
    can_continue;
    ready;
    summary;
    continuation =
      (if flash = None then "Finish and choose new game"
       else if ready then "Continue from flash save"
       else if kind = "full" then "Finish update"
       else "Restore saved game");
    hint =
      (if can_continue then ""
       else if unknown then "Delete the unidentified flash save to continue."
       else "Delete the damaged or unlisted records above to continue.");
  }

let confirmation title subject copy action destructive notice warning =
  obj
    [
      ("title", str title);
      ("subject", str subject);
      ("copy", str copy);
      ("action", str action);
      ("destructive", bool destructive);
      ("notice", str notice);
      ("warning", bool warning);
    ]

let record_confirmation catalog row =
  let missing = text row "issue" = "missing_snapshot" in
  let kind = text row "kind" in
  let action =
    if missing then "Remove entry"
    else if kind = "flash" then "Delete flash save"
    else if kind = "turn" then "Delete turn save"
    else if kind = "index" then "Delete save list"
    else "Delete retained save"
  in
  let entry = field row "entry" in
  confirmation
    (if missing then "Remove missing turn from list?" else action ^ "?")
    (if entry = Js.Json.null then record_name catalog row
     else
       Save_browser.run_name catalog entry
       ^ " · " ^ record_name catalog row ^ " · "
       ^ Save_browser.save_time (n entry "savedAt"))
    (if missing then "Only its list entry remains. It will be removed."
     else if kind = "index" then
       "Only the save list will be deleted. Saved game payloads will remain. Unlisted saves must \
        be deleted separately before import can finish."
     else "This save will be deleted. Other saves will remain.")
    action true "" false

let campaign_confirmation catalog inventory id =
  let records = array (field inventory "records") in
  let own =
    Array.to_list records
    |> List.filter (fun r -> text r "ownership" = "known" && text r "runId" = id)
  in
  let count = List.length (List.filter (fun r -> text r "kind" = "turn") own) in
  let flash = Array.find_opt (fun r -> text r "kind" = "flash") records in
  let includes =
    Option.fold ~none:false
      ~some:(fun f -> text f "ownership" = "known" && text f "runId" = id)
      flash
  in
  let unknown = Option.fold ~none:false ~some:(fun f -> text f "ownership" <> "known") flash in
  let name =
    match own with
    | [] -> "Selected campaign"
    | row :: _ -> Save_browser.run_name catalog (field row "entry")
  in
  let copy =
    if count > 0 then
      string_of_int count ^ " turn "
      ^ (if count = 1 then "save" else "saves")
      ^ (if includes then " and its flash save" else "")
      ^ " will be deleted."
    else if includes then "The flash save will be deleted."
    else "No identified saves remain for this campaign."
  in
  let notice =
    if unknown then
      "The unidentified flash save will be kept. We strongly recommend deleting it separately."
    else
      match flash with
      | Some f when not includes ->
          "The flash save for " ^ Save_browser.run_name catalog (field f "entry") ^ " will be kept."
      | _ -> ""
  in
  confirmation "Delete campaign saves?" name copy "Delete campaign saves" true notice unknown

let index_message row =
  let count = int_of_float (n row "unlistedCount") in
  if count > 0 then
    string_of_int count ^ " identifiable "
    ^ (if count = 1 then "turn is" else "turns are")
    ^ " missing from the list."
  else if text row "issue" = "index_missing" then
    "Delete the unlisted turn saves individually or clear saved data to continue."
  else "You can delete this save list. Saved game payloads will remain for individual deletion."
