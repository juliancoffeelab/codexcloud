type command = Build of string | Cancel | Query of string | Validate | Shutdown

type event =
  | Ready
  | Progress of int
  | Indexed
  | Cancelled
  | Search_error of string
  | Query_error of string
  | Worker_failed of string
  | Clear
  | Match of string
  | Queried
  | Validation of bool
  | Closed

let command_to_wire = function
  | Build url -> ("build", url)
  | Cancel -> ("cancel", "")
  | Query needle -> ("query", needle)
  | Validate -> ("validate", "")
  | Shutdown -> ("shutdown", "")

let command_of_wire = function
  | "build", url -> Some (Build url)
  | "cancel", "" -> Some Cancel
  | "query", needle -> Some (Query needle)
  | "validate", "" -> Some Validate
  | "shutdown", "" -> Some Shutdown
  | _ -> None

let event_to_wire = function
  | Ready -> ("ready", "")
  | Progress count -> ("progress", string_of_int count)
  | Indexed -> ("index", "")
  | Cancelled -> ("cancelled", "")
  | Search_error reason -> ("error", reason)
  | Query_error reason -> ("query_error", reason)
  | Worker_failed reason -> ("failed", reason)
  | Clear -> ("clear", "")
  | Match text -> ("match", text)
  | Queried -> ("queried", "")
  | Validation passed -> ("validation", if passed then "PASS" else "FAILED")
  | Closed -> ("closed", "")

let event_of_wire = function
  | "ready", "" -> Some Ready
  | "progress", count -> (
      match int_of_string_opt count with
      | Some count when count >= 0 && count <= 4096 -> Some (Progress count)
      | _ -> None)
  | "index", "" -> Some Indexed
  | "cancelled", "" -> Some Cancelled
  | "error", reason -> Some (Search_error reason)
  | "query_error", reason -> Some (Query_error reason)
  | "failed", reason -> Some (Worker_failed reason)
  | "clear", "" -> Some Clear
  | "match", text -> Some (Match text)
  | "queried", "" -> Some Queried
  | "validation", "PASS" -> Some (Validation true)
  | "validation", "FAILED" -> Some (Validation false)
  | "closed", "" -> Some Closed
  | _ -> None

let worker : Browser.worker option ref = ref None

let message (kind, value) : Browser.message =
  { kind = Browser.to_host kind; value = Browser.to_host value }

let payload (data : Browser.message) =
  (Browser.of_host data.kind, Browser.of_host data.value)

let start receive =
  if !worker <> None then failwith "Search worker already started";
  let session =
    Browser.new_worker (Browser.to_host "./worker.mjs") (Browser.worker_options ())
  in
  worker := Some session;
  let current () =
    match !worker with Some current -> current == session | None -> false
  in
  Browser.worker_message session (fun event ->
      match Browser.data event with
      | Some data when current () && Browser.message_valid data (1024 * 1024) -> (
          match event_of_wire (payload data) with
          | Some event ->
              (match event with
              | Closed | Worker_failed _ ->
                  worker := None;
                  Browser.terminate session
              | _ -> ());
              receive event
          | None -> ())
      | _ -> ());
  let failed _ =
    if current () then (
      worker := None;
      Browser.terminate session;
      receive (Worker_failed "Search worker failed; reload to restart"))
  in
  Browser.worker_error session failed;
  Browser.worker_message_error session failed

let send command =
  match !worker with
  | None -> failwith "Search worker is closed"
  | Some worker ->
      let data = message (command_to_wire command) in
      if not (Browser.message_valid data 32768) then failwith "Invalid search command";
      Browser.post worker data

let listen receive =
  Browser.listen_raw Browser.global (Browser.to_host "message") (fun event ->
      match Browser.data event with
      | Some data when Browser.message_valid data 32768 ->
          Option.iter receive (command_of_wire (payload data))
      | _ -> ())

let emit event = Browser.post_global (message (event_to_wire event))
let close () = Browser.close_worker ()
