open Migration_json

(* Decode unknown storage data once, then keep every intermediate version typed. *)
let from_v38 (value : Historical_types.V38.t) : Historical_types.V39.t = Migration_late.v38 value

let from_v37 (value : Historical_types.V37.t) : Historical_types.V39.t =
  from_v38 (Migration_late.v37 value)

let from_v36 (value : Historical_types.V36.t) : Historical_types.V39.t =
  from_v37 (Migration_late.v36 value)

let from_v35 (value : Historical_types.V35.t) : Historical_types.V39.t =
  from_v36 (Migration_late.v35 value)

let from_v34 (value : Historical_types.V34.t) : Historical_types.V39.t =
  from_v35 (Migration_late.v34 value)

let from_v33 (value : Historical_types.V33.t) : Historical_types.V39.t =
  from_v34 (Migration_late.v33 value)

let from_v32 (value : Historical_types.V32.t) : Historical_types.V39.t =
  from_v33 (Migration_late.v32 value)

let from_v31 (value : Historical_types.V31.t) : Historical_types.V39.t =
  from_v32 (Migration_late.v31 value)

let from_v30 (value : Historical_types.V30.t) : Historical_types.V39.t =
  from_v31 (Migration_late.v30 value)

let from_v29 (value : Historical_types.V29.t) : Historical_types.V39.t =
  from_v30 (Migration_late.v29 value)

let from_v28 (value : Historical_types.V28.t) : Historical_types.V39.t =
  from_v29 (Migration_middle.v28 value)

let from_v27 (value : Historical_types.V27.t) : Historical_types.V39.t =
  from_v28 (Migration_middle.v27 value)

let from_v26 (value : Historical_types.V26.t) : Historical_types.V39.t =
  from_v27 (Migration_middle.v26 value)

let from_v25 (value : Historical_types.V25.t) : Historical_types.V39.t =
  from_v26 (Migration_middle.v25 value)

let from_v24 (value : Historical_types.V24.t) : Historical_types.V39.t =
  from_v25 (Migration_middle.v24 value)

let from_v23 (value : Historical_types.V23.t) : Historical_types.V39.t =
  from_v24 (Migration_middle.v23 value)

let from_v22 (value : Historical_types.V22.t) : Historical_types.V39.t =
  from_v23 (Migration_middle.v22 value)

let from_v21 (value : Historical_types.V21.t) : Historical_types.V39.t =
  from_v22 (Migration_middle.v21 value)

let from_v20 (value : Historical_types.V20.t) : Historical_types.V39.t =
  from_v21 (Migration_middle.v20 value)

let from_v19 (value : Historical_types.V19.t) : Historical_types.V39.t =
  from_v20 (Migration_middle.v19 value)

let from_v18 (value : Historical_types.V18.t) : Historical_types.V39.t =
  from_v19 (Migration_middle.v18 value)

let from_v17 (value : Historical_types.V17.t) : Historical_types.V39.t =
  from_v18 (Early_migrations.v17 value)

let from_v16 (value : Historical_types.V16.t) : Historical_types.V39.t =
  from_v17 (Early_migrations.v16 value)

let from_v15 (value : Historical_types.V15.t) : Historical_types.V39.t =
  from_v16 (Early_migrations.v15 value)

let from_v14 (value : Historical_types.V14.t) : Historical_types.V39.t =
  from_v15 (Early_migrations.v14 value)

let from_v13 (value : Historical_types.V13.t) : Historical_types.V39.t =
  from_v14 (Early_migrations.v13 value)

let from_v12 (value : Historical_types.V12.t) : Historical_types.V39.t =
  from_v13 (Early_migrations.v12 value)

let from_v11 (value : Historical_types.V11.t) : Historical_types.V39.t =
  from_v12 (Early_migrations.v11 value)

let from_v10 (value : Historical_types.V10.t) : Historical_types.V39.t =
  from_v11 (Early_migrations.v10 value)

let from_v9 (value : Historical_types.V9.t) : Historical_types.V39.t =
  from_v10 (Early_migrations.v9 value)

let from_v8 (value : Historical_types.V8.t) : Historical_types.V39.t =
  from_v9 (Early_migrations.v8 value)

let from_v7 (value : Historical_types.V7.t) : Historical_types.V39.t =
  from_v8 (Early_migrations.v7 value)

let from_v6 (value : Historical_types.V6.t) : Historical_types.V39.t =
  from_v7 (Early_migrations.v6 value)

let from_v5 (value : Historical_types.V5.t) : Historical_types.V39.t =
  from_v6 (Early_migrations.v5 value)

let from_v4 (value : Historical_types.V4.t) : Historical_types.V39.t =
  from_v5 (Early_migrations.v4 value)

let from_v3 (value : Historical_types.V3.t) : Historical_types.V39.t =
  from_v4 (Early_migrations.v3 value)

let from_v2 (value : Historical_types.V2.t) : Historical_types.V39.t =
  from_v3 (Early_migrations.v2 value)

(* Original browser saves already contained Sella; retain that provenance through
   the explicit historical board repair without changing direct/native v6 saves. *)
let from_browser_v2 value =
  let value = Early_migrations.v2 value in
  let value = Early_migrations.v3 value in
  let value = Early_migrations.v4 value in
  let value = Early_migrations.v5 value in
  from_v7 (Early_migrations.v6_with_preplaced_sella true value)

let convert input =
  let _ = object_ input in
  let version =
    match get input "version" with
    | None when get input "engine" = Some (s "rust-wasm") -> 1.
    | None -> invalid "Unsupported save version"
    | Some value -> number value
  in
  require (safe_integer version && version >= 1. && version <= 39.) "Unsupported save version";
  let converted =
    match int_of_float version with
    | 1 ->
        if get input "engine" = Some (s "rust-wasm") then
          from_v2 (Early_migrations.v1_native (Historical_codec.Native1.of_json input))
        else (
          require (get input "engine" = None) "Unknown historical engine";
          from_browser_v2
            (Early_migrations.v1_javascript (Historical_codec.Javascript1.of_json input)))
    | 2 -> from_v2 (Historical_codec.V2.of_json input)
    | 3 -> from_v3 (Historical_codec.V3.of_json input)
    | 4 -> from_v4 (Historical_codec.V4.of_json input)
    | 5 -> from_v5 (Historical_codec.V5.of_json input)
    | 6 -> from_v6 (Historical_codec.V6.of_json input)
    | 7 -> from_v7 (Historical_codec.V7.of_json input)
    | 8 -> from_v8 (Historical_codec.V8.of_json input)
    | 9 -> from_v9 (Historical_codec.V9.of_json input)
    | 10 -> from_v10 (Historical_codec.V10.of_json input)
    | 11 -> from_v11 (Historical_codec.V11.of_json input)
    | 12 -> from_v12 (Historical_codec.V12.of_json input)
    | 13 -> from_v13 (Historical_codec.V13.of_json input)
    | 14 -> from_v14 (Historical_codec.V14.of_json input)
    | 15 -> from_v15 (Historical_codec.V15.of_json input)
    | 16 -> from_v16 (Historical_codec.V16.of_json input)
    | 17 -> from_v17 (Historical_codec.V17.of_json input)
    | 18 -> from_v18 (Historical_codec.V18.of_json input)
    | 19 -> from_v19 (Historical_codec.V19.of_json input)
    | 20 -> from_v20 (Historical_codec.V20.of_json input)
    | 21 -> from_v21 (Historical_codec.V21.of_json input)
    | 22 -> from_v22 (Historical_codec.V22.of_json input)
    | 23 -> from_v23 (Historical_codec.V23.of_json input)
    | 24 -> from_v24 (Historical_codec.V24.of_json input)
    | 25 -> from_v25 (Historical_codec.V25.of_json input)
    | 26 -> from_v26 (Historical_codec.V26.of_json input)
    | 27 -> from_v27 (Historical_codec.V27.of_json input)
    | 28 -> from_v28 (Historical_codec.V28.of_json input)
    | 29 -> from_v29 (Historical_codec.V29.of_json input)
    | 30 -> from_v30 (Historical_codec.V30.of_json input)
    | 31 -> from_v31 (Historical_codec.V31.of_json input)
    | 32 -> from_v32 (Historical_codec.V32.of_json input)
    | 33 -> from_v33 (Historical_codec.V33.of_json input)
    | 34 -> from_v34 (Historical_codec.V34.of_json input)
    | 35 -> from_v35 (Historical_codec.V35.of_json input)
    | 36 -> from_v36 (Historical_codec.V36.of_json input)
    | 37 -> from_v37 (Historical_codec.V37.of_json input)
    | 38 -> from_v38 (Historical_codec.V38.of_json input)
    | 39 -> Historical_codec.V39.of_json (Migration_envelope.current_save input)
    | _ -> invalid "Unsupported save version"
  in
  (converted, version <> 39.)

(* Relational rules (identity, flash/turn and replay invariants) complement the
   static wire model and apply to every consumer, including storage import. *)
let validated input =
  let value, changed = convert input in
  let json = Migration_envelope.current_save (Historical_codec.V39.to_json value) in
  (Current_save.narrow value, json, changed)

let migrate_typed input =
  let value, _, changed = validated input in
  (value, changed)

(* JSON is the storage/public parsing boundary, not an intermediate model. *)
let migrate input =
  let _, json, changed = validated input in
  (json, changed)
