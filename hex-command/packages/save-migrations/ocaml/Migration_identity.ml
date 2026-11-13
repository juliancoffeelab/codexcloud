(* Only a fully migrated save establishes ownership. Literal metadata claims
   remain untrusted evidence; they can prevent deletion, never authorize it. *)
open Migration_json
module T = Historical_types

type entry = Flash_entry of T.V45.current_entry | Turn_entry of T.V45.turn_entry
type origin = Index | Payload | Migrated_payload

type claim = {
  origin : origin;
  version : float option;
  raw : Js.Json.t;
  id : string option;
  run_id : string option;
}

let entry_id = function Flash_entry entry -> entry.id | Turn_entry entry -> entry.id
let entry_run_id = function Flash_entry entry -> entry.run_id | Turn_entry entry -> entry.run_id
let entry_run = function Flash_entry entry -> entry.run | Turn_entry entry -> entry.run
let entry_round = function Flash_entry entry -> entry.round | Turn_entry entry -> entry.round
let is_turn = function Turn_entry _ -> true | Flash_entry _ -> false

let entry_to_json = function
  | Flash_entry value -> Historical_codec.encode_save37_progress_entry_member0 value
  | Turn_entry value -> Historical_codec.encode_history4_entries_item value

let entry_of_save : T.V45.t -> entry = function
  | `Current value -> Flash_entry value.progress.entry
  | `Turn value -> Turn_entry value.progress.entry

let readable_string value key =
  if not (is_record value) then None
  else Option.map Migration_utf8.of_js (Option.bind (get value key) Js.Json.decodeString)

let version value =
  if not (is_record value) then None else Option.bind (get value "version") Js.Json.decodeNumber

(* Read literal claims only to display disputes and prevent destructive actions.
   These are untrusted evidence, never current entries or migration output. *)
let claim ~origin ~version raw =
  { origin; version; raw; id = readable_string raw "id"; run_id = readable_string raw "runId" }

let from_payload value =
  if not (is_record value) then None
  else
    Option.bind (get value "progress") (fun progress ->
        if not (is_record progress) then None
        else Option.map (claim ~origin:Payload ~version:(version value)) (get progress "entry"))

let from_index version raw = claim ~origin:Index ~version raw

let from_migrated value =
  let entry = entry_of_save value in
  {
    origin = Migrated_payload;
    version = Some 39.;
    raw = entry_to_json entry;
    id = Some (entry_id entry);
    run_id = Some (entry_run_id entry);
  }

let owners claims = unique (List.filter_map (fun (claim : claim) -> claim.run_id) claims)

let to_json (claim : claim) =
  let fields =
    if is_record claim.raw then Array.to_list (entries claim.raw) else [ ("raw", claim.raw) ]
  in
  obj
    (fields
    @ [
        ("partial", b (claim.origin <> Migrated_payload));
        ( "source",
          s
            (match claim.origin with
            | Index -> "index"
            | Payload -> "payload"
            | Migrated_payload -> "migratedPayload") );
      ]
    @ Option.fold ~none:[] ~some:(fun version -> [ ("version", n version) ]) claim.version)
