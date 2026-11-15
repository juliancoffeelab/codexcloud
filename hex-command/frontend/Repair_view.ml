open Save_json

let issue row =
  match text row {js|issue|js} with
  | matched_text_0 when matched_text_0 = {js|unreadable|js} -> {js|Couldn’t read this save|js}
  | matched_text_1 when matched_text_1 = {js|format|js} ->
      {js|This save’s format isn’t recognized|js}
  | matched_text_2 when matched_text_2 = {js|missing_snapshot|js} -> {js|Snapshot missing|js}
  | matched_text_3 when matched_text_3 = {js|identity_conflict|js} ->
      {js|The snapshot doesn’t match its save entry|js}
  | matched_text_4 when matched_text_4 = {js|ownership_conflict|js} ->
      {js|Conflicting campaign information|js}
  | matched_text_5 when matched_text_5 = {js|invalid_battle|js} ->
      {js|The saved battle failed validation|js}
  | matched_text_6 when matched_text_6 = {js|index_missing|js} -> {js|The save list is missing|js}
  | matched_text_7 when matched_text_7 = {js|index_invalid|js} -> {js|The save list is damaged|js}
  | matched_text_8 when matched_text_8 = {js|unlisted_snapshots|js} ->
      {js|Turns are missing from the save list|js}
  | _ -> {js||js}

let record_name catalog row =
  match text row {js|kind|js} with
  | matched_text_9 when matched_text_9 = {js|flash|js} ->
      if text row {js|ownership|js} = {js|known|js} then {js|Flash save|js}
      else {js|Unidentified flash save|js}
  | matched_text_10 when matched_text_10 = {js|index|js} -> {js|Save list|js}
  | _ ->
      let entry = field row {js|entry|js} in
      if text entry {js|kind|js} = {js|turn|js} then Save_browser.turn_name catalog entry
      else if text row {js|kind|js} = {js|turn|js} then {js|Unidentified turn save|js}
      else {js|Unidentified save|js}

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
  let records = array (field inventory {js|records|js}) in
  let flash = Array.find_opt (fun r -> text r {js|kind|js} = {js|flash|js}) records in
  let attention =
    Array.to_list records |> List.filter (fun r -> get r {js|issue|js} <> None) |> Array.of_list
  in
  let priority r =
    match text r {js|kind|js} with
    | matched_text_11 when matched_text_11 = {js|flash|js} -> 0
    | matched_text_12 when matched_text_12 = {js|index|js} -> 1
    | _ -> 2
  in
  Array.stable_sort (fun a b -> Int.compare (priority a) (priority b)) attention;
  let groups = Hashtbl.create 8 and order = ref [] in
  Array.iter
    (fun row ->
      if text row {js|ownership|js} = {js|known|js} && text row {js|kind|js} <> {js|index|js} then (
        let id = text row {js|runId|js} in
        if not (Hashtbl.mem groups id) then order := !order @ [ id ];
        Hashtbl.replace groups id (row :: Option.value ~default:[] (Hashtbl.find_opt groups id))))
    records;
  let campaigns =
    List.map
      (fun id ->
        let rows = Hashtbl.find groups id |> List.rev |> Array.of_list in
        Array.stable_sort
          (fun a b -> Save_browser.order (field a {js|entry|js}) (field b {js|entry|js}))
          rows;
        { id; name = Save_browser.run_name catalog (field rows.(0) {js|entry|js}); records = rows })
      !order
    |> Array.of_list
  in
  let errors = array (field inventory {js|errors|js}) in
  let can_continue = Array.length errors = 0 && Array.length attention = 0 in
  let ready = b inventory {js|authoritative|js} && can_continue in
  let unknown =
    Option.fold ~none:false ~some:(fun f -> text f {js|ownership|js} <> {js|known|js}) flash
  in
  let kind = Option.fold ~none:{js||js} ~some:(fun p -> text p {js|kind|js}) problem in
  let summary =
    if Array.length errors > 0 then
      {js|Saved data couldn’t be checked completely. You can review what was readable below.|js}
    else if ready then
      if b inventory {js|completedElsewhere|js} then
        {js|Another tab finished updating your saves.|js}
      else if Array.exists (fun r -> text r {js|kind|js} <> {js|index|js}) records then
        {js|Your saved games can now be read.|js}
      else {js|No retained saves remain.|js}
    else if kind = {js|full|js} then
      {js|There isn’t enough space to finish updating saved games. Delete saves you no longer need, then finish the update.|js}
    else if unknown then {js|We can’t identify the flash save’s campaign.|js}
    else if Array.exists (fun r -> text r {js|kind|js} = {js|index|js}) attention then
      {js|The save list needs attention.|js}
    else if
      Array.length attention = 1 && text attention.(0) {js|issue|js} = {js|missing_snapshot|js}
    then {js|One turn is listed, but its snapshot is missing.|js}
    else if
      Array.length attention = 1 && text attention.(0) {js|issue|js} = {js|ownership_conflict|js}
    then {js|One turn has conflicting campaign information.|js}
    else if Array.length attention = 1 && text attention.(0) {js|kind|js} = {js|turn|js} then
      {js|One turn save can’t be restored.|js}
    else if Array.length attention > 0 then
      let js_text_part_0 = string_of_int (Array.length attention) in
      {j|$(js_text_part_0) saved records need attention.|j}
    else if Array.length records = 0 then {js|No retained saves remain.|js}
    else {js|The remaining saved games are ready to update.|js}
  in
  let summary =
    let js_text_part_1 =
      if ready && flash = None then {js| There is no flash save to continue.|js}
      else if
        (not ready)
        && Option.fold ~none:false ~some:(fun f -> get f {js|issue|js} = None) flash
        && kind <> {js|full|js}
        && Array.length errors = 0
      then {js| Your flash save is intact.|js}
      else {js||js}
    in
    {j|$(summary)$(js_text_part_1)|j}
  in
  {
    flash;
    attention;
    campaigns;
    can_continue;
    ready;
    summary;
    continuation =
      (if flash = None then {js|Finish and choose new game|js}
       else if ready then {js|Continue from flash save|js}
       else if kind = {js|full|js} then {js|Finish update|js}
       else {js|Restore saved game|js});
    hint =
      (if can_continue then {js||js}
       else if unknown then {js|Delete the unidentified flash save to continue.|js}
       else {js|Delete the damaged or unlisted records above to continue.|js});
  }

let confirmation title subject copy action destructive notice warning =
  obj
    [
      ({js|title|js}, str title);
      ({js|subject|js}, str subject);
      ({js|copy|js}, str copy);
      ({js|action|js}, str action);
      ({js|destructive|js}, bool destructive);
      ({js|notice|js}, str notice);
      ({js|warning|js}, bool warning);
    ]

let record_confirmation catalog row =
  let missing = text row {js|issue|js} = {js|missing_snapshot|js} in
  let kind = text row {js|kind|js} in
  let action =
    if missing then {js|Remove entry|js}
    else if kind = {js|flash|js} then {js|Delete flash save|js}
    else if kind = {js|turn|js} then {js|Delete turn save|js}
    else if kind = {js|index|js} then {js|Delete save list|js}
    else {js|Delete retained save|js}
  in
  let entry = field row {js|entry|js} in
  confirmation
    (if missing then {js|Remove missing turn from list?|js} else {j|$(action)?|j})
    (if entry = Js.Json.null then record_name catalog row
     else
       let js_text_part_0 = Save_browser.run_name catalog entry in
       let js_text_part_2 = record_name catalog row in
       let js_text_part_4 = Save_browser.save_time (n entry {js|savedAt|js}) in
       {j|$(js_text_part_0) · $(js_text_part_2) · $(js_text_part_4)|j})
    (if missing then {js|Only its list entry remains. It will be removed.|js}
     else if kind = {js|index|js} then
       {js|Only the save list will be deleted. Saved game payloads will remain. Unlisted saves must be deleted separately before import can finish.|js}
     else {js|This save will be deleted. Other saves will remain.|js})
    action true {js||js} false

let campaign_confirmation catalog inventory id =
  let records = array (field inventory {js|records|js}) in
  let own =
    Array.to_list records
    |> List.filter (fun r -> text r {js|ownership|js} = {js|known|js} && text r {js|runId|js} = id)
  in
  let count = List.length (List.filter (fun r -> text r {js|kind|js} = {js|turn|js}) own) in
  let flash = Array.find_opt (fun r -> text r {js|kind|js} = {js|flash|js}) records in
  let includes =
    Option.fold ~none:false
      ~some:(fun f -> text f {js|ownership|js} = {js|known|js} && text f {js|runId|js} = id)
      flash
  in
  let unknown =
    Option.fold ~none:false ~some:(fun f -> text f {js|ownership|js} <> {js|known|js}) flash
  in
  let name =
    match own with
    | [] -> {js|Selected campaign|js}
    | row :: _ -> Save_browser.run_name catalog (field row {js|entry|js})
  in
  let copy =
    if count > 0 then
      let js_text_part_0 = string_of_int count in
      let js_text_part_2 = if count = 1 then {js|save|js} else {js|saves|js} in
      let js_text_part_3 = if includes then {js| and its flash save|js} else {js||js} in
      {j|$(js_text_part_0) turn $(js_text_part_2)$(js_text_part_3) will be deleted.|j}
    else if includes then {js|The flash save will be deleted.|js}
    else {js|No identified saves remain for this campaign.|js}
  in
  let notice =
    if unknown then
      {js|The unidentified flash save will be kept. We strongly recommend deleting it separately.|js}
    else
      match flash with
      | Some f when not includes ->
          let js_text_part_1 = Save_browser.run_name catalog (field f {js|entry|js}) in
          {j|The flash save for $(js_text_part_1) will be kept.|j}
      | _ -> {js||js}
  in
  confirmation {js|Delete campaign saves?|js} name copy {js|Delete campaign saves|js} true notice
    unknown

let index_message row =
  let count = int_of_float (n row {js|unlistedCount|js}) in
  if count > 0 then
    let js_text_part_0 = string_of_int count in
    let js_text_part_2 = if count = 1 then {js|turn is|js} else {js|turns are|js} in
    {j|$(js_text_part_0) identifiable $(js_text_part_2) missing from the list.|j}
  else if text row {js|issue|js} = {js|index_missing|js} then
    {js|Delete the unlisted turn saves individually or clear saved data to continue.|js}
  else
    {js|You can delete this save list. Saved game payloads will remain for individual deletion.|js}
