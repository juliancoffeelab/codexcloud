(* Data-only entry points for external tools and parity probes. *)
open Migration_json

let migrate_save value =
  let save, changed = Save_migrations.migrate value in
  obj [ ("save", save); ("migrated", b changed) ]

let migrate_v1_to_v2 value =
  let converted =
    if get value "engine" = Some (s "rust-wasm") then
      Early_migrations.v1_native (Historical_codec.Native1.of_json value)
    else (
      require (get value "engine" = None) "Unknown historical engine";
      Early_migrations.v1_javascript (Historical_codec.Javascript1.of_json value))
  in
  Historical_codec.V2.to_json converted

let history = Lazy.force Frozen_data.history_stages

let campaigns =
  let rows = list Save_history.catalog "campaigns" in
  obj
    (List.map
       (fun c ->
         let c = clone c in
         List.iter
           (fun mission ->
             let history = default history (text (field mission "map") "id") (obj []) in
             set mission "legacyDialogue" (default history "dialogue" (a []));
             List.iter
               (fun wave ->
                 Option.iter
                   (fun old -> Array.iter (fun (key, value) -> set wave key value) (entries old))
                   (List.find_opt
                      (fun old -> get old "id" = get wave "id")
                      (optional_list history "waves")))
               (list mission "waves"))
           (list c "missions");
         (text c "id", c))
       rows)

let save_version = Migration_envelope.version
let history_key = Storage_migrations.history_key
let history_payload_key = Storage_migrations.payload_key
let inspect_storage = Storage_migrations.inspect
let repair_storage = Storage_migrations.repair

let import_wave_marker marker waves =
  Save_legacy.wave_marker (Migration_utf8.of_js marker) waves |> Migration_utf8.to_js
