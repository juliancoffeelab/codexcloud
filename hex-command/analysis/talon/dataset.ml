open Json
(** One journal decode into typed relations. Arrow snapshots are projected in
    place; phase rosters remain foreign-key joins. No expanded phase trees. *)

module R = Rel
module B = R.Builder
module E = Talon.Expr
module Q = Talon.Query

let side = function `String "ai" -> str "enemy" | x -> x
let ints ns = List.map (fun n -> (n, B.Int)) ns
let floats ns = List.map (fun n -> (n, B.Float)) ns
let strings ns = List.map (fun n -> (n, B.String)) ns
let bools ns = List.map (fun n -> (n, B.Bool)) ns
let unit_fields = [ "id"; "type"; "cell"; "side"; "faction"; "role"; "name" ]

let unit_schema =
  strings unit_fields
  @ ints [ "m"; "ch"; "p"; "hp"; "col"; "row" ]
  @ bools [ "rebuilding" ]

let unit_keys = List.map fst unit_schema

let damage_fields =
  [
    "effectiveDamage";
    "afterTerrainDamage";
    "availableHp";
    "effectiveArmourPrevented";
    "effectiveTerrainPrevented";
    "rawDamage";
    "afterArmourDamage";
  ]

let exposure_fields =
  [
    "beforeDamage";
    "chosenTileDamage";
    "minDamage";
    "reachableTiles";
    "safeTiles";
    "afterDamage";
  ]

let value_dims =
  [
    "kind";
    "campaign";
    "diplomacy";
    "objective";
    "mission";
    "race";
    "policy";
    "opponentPolicy";
    "map";
    "family";
  ]

type header = { m : int; saved : Json.t; result : Json.t; report : Json.t }

type t = {
  meta : Talon.t;
  chapters : Talon.t;
  tiles : Talon.t;
  profiles : Talon.t;
  initial : Talon.t;
  final : Talon.t;
  phases : Talon.t;
  before : Talon.t;
  units : Talon.t;
  losses : Talon.t;
  lookup : Talon.t;
  support_lookup : Talon.t;
  actions : Talon.t;
  combats : Talon.t;
  healing : Talon.t;
  audits : Talon.t;
  goals : Talon.t;
  objectives : Talon.t;
  duties : Talon.t;
  constraints : Talon.t;
  headers : header list;
  owners : (int, Json.t array) Hashtbl.t;
}

let unit_value factions u name =
  match name with
  | "cell" -> if get u "tile" = `Null then `Null else str (cell (get u "tile"))
  | "col" | "row" -> (
      let tile = get u "tile" in
      if has tile name then get tile name
      else
        match String.split_on_char ',' (cell tile) with
        | [ x; y ] -> int (int_of_string (if name = "col" then x else y))
        | _ -> `Null)
  | "side" ->
      side
        (if has u "side" then get u "side"
         else
           let f =
             List.find_opt (fun f -> get f "id" = get u "faction") factions
           in
           match f with Some f -> get f "phase" | None -> `Null)
  | _ -> get u name

let load ?readers reports ~telemetry =
  let with_readers f =
    match readers with
    | Some readers -> f readers
    | None -> Journal.with_readers f
  in
  with_readers (fun readers ->
      let meta =
        B.create
          (ints [ "m"; "seed" ]
          @ strings
              [
                "kind";
                "matchId";
                "campaign";
                "diplomacy";
                "objective";
                "first";
                "landscape";
                "player";
                "enemy";
                "playerRace";
                "enemyRace";
              ])
      in
      let chapters = B.create (ints [ "m"; "ch" ] @ strings [ "mission" ]) in
      let profiles =
        B.create
          (strings
             [ "type"; "family"; "profileRace"; "profileRole"; "profileName" ]
          @ floats [ "cost"; "healing"; "thornAura"; "thorns" ]
          @ ints [ "level" ])
      in
      let tiles =
        B.create
          (ints [ "m"; "ch"; "ordinal"; "col"; "row" ]
          @ strings [ "cell"; "terrain"; "owner" ])
      in
      let initial = B.create unit_schema
      and final = B.create unit_schema
      and losses = B.create unit_schema in
      let phases =
        B.create
          (ints [ "p"; "m"; "ch"; "phase"; "round"; "ownership" ]
          @ strings [ "side"; "goalsSignature"; "planning"; "ownerSignature" ]
          @ bools [ "hasPlanning"; "emptyActions" ]
          @ floats [ "candidates" ])
      in
      let refs =
        B.create (ints [ "p"; "m"; "ch"; "snapshot" ] @ strings [ "which" ])
      in
      let json_snapshots =
        B.create
          (ints [ "m"; "snapshot" ]
          @ strings unit_fields
          @ ints [ "hp"; "col"; "row" ]
          @ bools [ "rebuilding" ])
      in
      let actions =
        B.create
          (ints [ "p"; "action" ]
          @ strings [ "kind"; "unit"; "type"; "destination" ]
          @ floats [ "paid" ])
      in
      let combats =
        B.create
          (ints [ "p"; "combat" ]
          @ strings [ "unit"; "target" ]
          @ floats
              ([
                 "damage";
                 "retaliation";
                 "retaliationCount";
                 "raw";
                 "aura";
                 "effective";
               ]
              @ List.concat_map
                  (fun k -> List.map (fun n -> k ^ n) damage_fields)
                  [ "attack"; "response" ])
          @ bools [ "hasMitigation"; "hasResponse"; "hasThorns" ]
          @ [ ("sources", B.Strings) ])
      in
      let healing =
        B.create
          (ints [ "p" ] @ floats [ "amount" ] @ [ ("providers", B.Strings) ])
      in
      let audits =
        B.create
          (ints [ "p" ]
          @ strings
              [ "leader"; "auditSide"; "beforeTile"; "afterTile"; "minTile" ]
          @ bools
              [ "died"; "hasExposure"; "onOwnedKeep"; "chosenTileReachable" ]
          @ floats
              ([
                 "beforeHP";
                 "afterHP";
                 "armyUnitsBefore";
                 "armyUnitsAfter";
                 "armyHPValueBefore";
                 "armyHPValueAfter";
                 "opponentHPValueBefore";
                 "opponentHPValueAfter";
                 "armyHPBefore";
                 "armyHPAfter";
                 "gold";
                 "legalOptions";
                 "affordableOptions";
               ]
              @ exposure_fields))
      in
      let goals =
        B.create (ints [ "p"; "m"; "ch"; "goal" ] @ floats [ "current" ])
      in
      let objectives =
        B.create (ints [ "m"; "ch" ] @ strings [ "side"; "cell" ])
      in
      let duties =
        B.create (ints [ "p"; "m"; "ch" ] @ strings [ "status"; "reason" ])
      in
      let constraints = B.create (ints [ "p" ] @ strings [ "unit" ]) in
      let headers = ref []
      and snapshot_tables = ref []
      and phase_number = ref 0
      and action_number = ref 0
      and combat_number = ref 0 in
      let owners = Hashtbl.create 128 and catalogs = ref None in
      let add_unit dest m ch p factions u =
        B.add dest (fun n ->
            match n with
            | "m" -> int m
            | "ch" -> int ch
            | "p" -> int p
            | _ -> unit_value factions u n)
      in
      List.iter
        (fun (path, report) ->
          if telemetry then
            Journal.require
              (get report "status" = str "complete")
              "Telemetry requires a completed run";
          let legacy = Hashtbl.create 32 in
          let legacy_path =
            Filename.concat (Filename.dirname path) "details.jsonl.gz"
          in
          (if
             Filename.basename path = "outcomes.json"
             && Sys.file_exists legacy_path
           then
             let stream =
               Unix.open_process_in ("gzip -dc -- " ^ Filename.quote legacy_path)
             in
             Fun.protect
               ~finally:(fun () -> ignore (Unix.close_process_in stream))
               (fun () ->
                 try
                   while true do
                     let value = normalize (parse (input_line stream)) in
                     let value =
                       if has value "kind" then value
                       else
                         set value "kind"
                           (str
                              (if has value "missions" then "campaign"
                               else "arena"))
                     in
                     Hashtbl.add legacy (identity value) value
                   done
                 with End_of_file -> ()));
          let declared_profiles =
            if truth (get report "unitProfiles") then get report "unitProfiles"
            else
              obj
                (List.map
                   (fun (unit, family) -> (unit, obj [ ("family", family) ]))
                   (fields (get report "unitFamilies")))
          in
          (match !catalogs with
          | None ->
              catalogs := Some declared_profiles;
              List.iter
                (fun (typ, profile) ->
                  B.add profiles (fun n ->
                      match n with
                      | "type" -> str typ
                      | "profileRace" -> get profile "race"
                      | "profileRole" -> get profile "role"
                      | "profileName" -> default profile "name" (str typ)
                      | _ -> get profile n))
                (fields declared_profiles)
          | Some previous ->
              Journal.require
                ((not telemetry) || sorted previous = sorted declared_profiles)
                "Cannot pool different recorded unit catalogs");
          list (get report "outcomes")
          |> List.sort (fun a b ->
              compare
                ( text (get (get a "detail") "file"),
                  integer (get (get a "detail") "offset") )
                ( text (get (get b "detail") "file"),
                  integer (get (get b "detail") "offset") ))
          |> List.iter (fun saved ->
              let kind =
                default saved "kind"
                  (str
                     (if has (get saved "config") "campaign" then "campaign"
                      else "arena"))
              in
              let config = get saved "config" in
              let arena = kind = str "arena" in
              let mirror =
                arena
                && get (get config "armies") "player"
                   = get (get config "armies") "enemy"
              in
              if (not arena) || (telemetry && not mirror) then (
                let reference = get saved "detail" in
                let raw =
                  if
                    List.for_all (has reference)
                      [ "file"; "offset"; "bytes"; "sha256" ]
                  then
                    Some
                      (Journal.bytes ~readers (Filename.dirname path) reference)
                  else None
                in
                Journal.require
                  ((not arena) || Option.is_some raw)
                  "Telemetry requires recorded gameplay journals";
                let arrow =
                  match raw with
                  | Some raw
                    when Filename.check_suffix
                           (text (get reference "file"))
                           ".arrow" ->
                      Some (Journal.arrow raw)
                  | _ -> None
                in
                let modern =
                  match arrow with
                  | Some (version, _) ->
                      List.mem version
                        [
                          "arrow-ipc-snapshot-dictionaries-v4";
                          "arrow-ipc-snapshot-dictionaries-v5";
                        ]
                  | _ -> false
                in
                let result =
                  if modern then
                    let _, table = Option.get arrow in
                    let strings =
                      Talon.Column.options Talon.Kind.string
                        (Talon.column table "header_json")
                    in
                    normalize (parse (Option.get strings.(0)))
                  else
                    match raw with
                    | Some raw ->
                        normalize
                          (Journal.decode raw (text (get reference "file")))
                    | None ->
                        Option.value ~default:saved
                          (Hashtbl.find_opt legacy (identity saved))
                in
                let result =
                  if has result "kind" then result else set result "kind" kind
                in
                Journal.require
                  (sorted (get result "config") = sorted config)
                  "Journal configuration mismatch";
                let m = List.length !headers in
                headers := { m; saved; result; report } :: !headers;
                B.add meta (fun n ->
                    match n with
                    | "m" -> int m
                    | "kind" -> kind
                    | "matchId" -> str (identity saved)
                    | "player" | "enemy" -> str (label config n)
                    | "playerRace" -> get (get config "armies") "player"
                    | "enemyRace" -> get (get config "armies") "enemy"
                    | "campaign" | "diplomacy" | "first" | "landscape" ->
                        default config n (str "")
                    | "objective" -> default config n (str "authored")
                    | _ -> get config n);
                let missions =
                  Array.of_list
                    (if arena then [ result ] else list (get result "missions"))
                in
                Array.iteri
                  (fun ch chapter ->
                    B.add chapters (fun n ->
                        match n with
                        | "m" -> int m
                        | "ch" -> int ch
                        | _ -> default chapter "id" (str ""));
                    let state = get chapter "initial"
                    and factions =
                      list (get (get chapter "initial") "factions")
                    in
                    List.iteri
                      (fun ordinal t ->
                        B.add tiles (fun n ->
                            match n with
                            | "m" -> int m
                            | "ch" -> int ch
                            | "ordinal" -> int ordinal
                            | "cell" -> get t "id"
                            | _ -> get t n))
                      (list (get state "tiles"));
                    List.iter
                      (add_unit initial m ch (-1) factions)
                      (list (get state "units"));
                    List.iter
                      (add_unit final m ch (-1) factions)
                      (list (get (get chapter "final") "units")))
                  missions;
                let ownership =
                  if modern then (
                    let _, table = Option.get arrow in
                    List.iter
                      (fun name ->
                        let column = Talon.column table name in
                        let valid =
                          match Talon.Column.validity column with
                          | None -> true
                          | Some mask -> Nx.item [ 0 ] mask
                        in
                        Journal.require valid
                          "Invalid Arrow snapshot dictionaries")
                      [ "unit_snapshots"; "ownership_snapshots" ];
                    let v =
                      Frame.json_column
                        (Talon.column table "ownership_snapshots")
                    in
                    let snapshots = Array.of_list (list v.(0)) in
                    Array.iter
                      (fun snapshot ->
                        Journal.require
                          (match snapshot with
                          | `List values ->
                              List.for_all
                                (function
                                  | `Null | `String _ -> true | _ -> false)
                                values
                          | _ -> false)
                          "Invalid Arrow ownership snapshot")
                      snapshots;
                    snapshots)
                  else [||]
                in
                Hashtbl.add owners m ownership;
                let legacy_owners = ref [] and legacy_snapshot = ref 0 in
                let add_phase ch phase before_indices unit_indices owner_index =
                  Journal.require
                    (ch >= 0
                    && ch < Array.length missions
                    && ((not arena) || ch = 0))
                    "Invalid Arrow chapter";
                  let p = !phase_number in
                  incr phase_number;
                  let chapter = missions.(ch)
                  and factions =
                    list (get (get missions.(ch) "initial") "factions")
                  in
                  if modern then
                    Journal.require
                      (List.length (list ownership.(owner_index))
                      = List.length (list (get (get chapter "initial") "tiles"))
                      )
                      "Invalid Arrow ownership snapshot";
                  let phase_side = side (get phase "side") in
                  let plan = get phase "planning" in
                  B.add phases (fun n ->
                      match n with
                      | "p" -> int p
                      | "m" -> int m
                      | "ch" -> int ch
                      | "phase" -> int 0
                      | "side" -> phase_side
                      | "ownership" -> int owner_index
                      | "goalsSignature" -> str (canonical (get phase "goals"))
                      | "ownerSignature" ->
                          str
                            (canonical
                               (if modern then ownership.(owner_index)
                                else get phase "ownership"))
                      | "hasPlanning" -> bool (truth plan)
                      | "planning" ->
                          if truth plan then str (encode plan) else `Null
                      | "emptyActions" ->
                          bool (not (truth (get phase "actions")))
                      | "candidates" ->
                          default (get plan "report") "candidates" (int 0)
                      | _ -> get phase n);
                  List.iter
                    (fun (which, indices) ->
                      List.iter
                        (fun snapshot ->
                          B.add refs (fun n ->
                              match n with
                              | "p" -> int p
                              | "m" -> int m
                              | "ch" -> int ch
                              | "which" -> str which
                              | _ -> int snapshot))
                        indices)
                    [ ("before", before_indices); ("units", unit_indices) ];
                  List.iter
                    (add_unit losses m ch p factions)
                    (list (get phase "losses"));
                  List.iter
                    (fun a ->
                      let number = !action_number in
                      incr action_number;
                      B.add actions (fun n ->
                          match n with
                          | "p" -> int p
                          | "action" -> int number
                          | "destination" ->
                              if has a "tile" then str (cell (get a "tile"))
                              else `Null
                          | "paid" -> get a "cost"
                          | _ -> get a n))
                    (list (get phase "actions"));
                  List.iter
                    (fun event ->
                      let number = !combat_number in
                      incr combat_number;
                      let mitigation = get event "mitigation"
                      and thorns = get event "thorns" in
                      B.add combats (fun n ->
                          match n with
                          | "p" -> int p
                          | "combat" -> int number
                          | "raw" -> get thorns "damage"
                          | "aura" -> get thorns "auraDamage"
                          | "effective" -> get thorns "effectiveDamage"
                          | "sources" -> default thorns "auraSources" (arr [])
                          | "hasMitigation" -> bool (mitigation <> `Null)
                          | "hasResponse" ->
                              bool (get mitigation "retaliation" <> `Null)
                          | "hasThorns" -> bool (thorns <> `Null)
                          | _ ->
                              if String.starts_with ~prefix:"attack" n then
                                get (get mitigation "attack")
                                  (String.sub n 6 (String.length n - 6))
                              else if String.starts_with ~prefix:"response" n
                              then
                                get
                                  (get mitigation "retaliation")
                                  (String.sub n 8 (String.length n - 8))
                              else get event n))
                    (list (get phase "combats"));
                  List.iter
                    (fun event ->
                      B.add healing (fun n ->
                          if n = "p" then int p else get event n))
                    (list (get phase "healing"));
                  List.iter
                    (fun audit ->
                      let before = get audit "before"
                      and after = get audit "after"
                      and exposure = get audit "exposure" in
                      B.add audits (fun n ->
                          match n with
                          | "p" -> int p
                          | "auditSide" -> side (get audit "side")
                          | "died" -> bool (after = `Null)
                          | "hasExposure" -> bool (exposure <> `Null)
                          | "beforeHP" -> get before "hp"
                          | "afterHP" -> default after "hp" (int 0)
                          | "beforeTile" -> str (encode (get before "tile"))
                          | "afterTile" ->
                              if after = `Null then `Null
                              else str (encode (get after "tile"))
                          | "minTile" ->
                              if get exposure "minTile" = `Null then `Null
                              else str (encode (get exposure "minTile"))
                          | "armyUnitsBefore" ->
                              get (get audit "armyBefore") "units"
                          | "armyUnitsAfter" ->
                              get (get audit "armyAfter") "units"
                          | "armyHPValueBefore" ->
                              get (get audit "armyBefore") "hpValue"
                          | "armyHPValueAfter" ->
                              get (get audit "armyAfter") "hpValue"
                          | "opponentHPValueBefore" ->
                              get (get audit "opponentBefore") "hpValue"
                          | "opponentHPValueAfter" ->
                              get (get audit "opponentAfter") "hpValue"
                          | "armyHPBefore" -> get (get audit "armyBefore") "hp"
                          | "armyHPAfter" -> get (get audit "armyAfter") "hp"
                          | "gold" | "onOwnedKeep" | "legalOptions"
                          | "affordableOptions" ->
                              get (get audit "recruitmentBefore") n
                          | _ ->
                              if has exposure n then get exposure n
                              else get audit n))
                    (list (get phase "leaderAudit"));
                  List.iteri
                    (fun goal g ->
                      B.add goals (fun n ->
                          match n with
                          | "p" -> int p
                          | "m" -> int m
                          | "ch" -> int ch
                          | "goal" -> int goal
                          | _ -> get g n))
                    (list (get phase "goals"));
                  List.iter
                    (fun o ->
                      if get o "kind" = str "capture" then
                        List.iter
                          (fun tile ->
                            B.add objectives (fun n ->
                                match n with
                                | "m" -> int m
                                | "ch" -> int ch
                                | "side" -> phase_side
                                | _ -> str (cell tile)))
                          (list (get o "tiles")))
                    (list (get phase "objectives"));
                  List.iter
                    (fun duty ->
                      B.add duties (fun n ->
                          match n with
                          | "p" -> int p
                          | "m" -> int m
                          | "ch" -> int ch
                          | "reason" ->
                              let r = get duty "reason" in
                              if truth r then r else str "unspecified"
                          | _ -> get duty n))
                    (list (get (get plan "report") "duties"));
                  List.iter
                    (fun constraint_ ->
                      let actor = get (get constraint_ "Survive") "actor" in
                      if actor <> `Null then
                        B.add constraints (fun n ->
                            if n = "p" then int p
                            else str ("u" ^ string_of_int (integer actor))))
                    (list
                       (get (get (get plan "orders") "tactics") "constraints"))
                in
                if modern then (
                  let _, table = Option.get arrow in
                  let projected =
                    Q.of_table table
                    |> Q.slice ~offset:0 ~length:1
                    |> Q.select
                         [ E.keep (Talon.Sel.names [ "unit_snapshots" ]) ]
                    |> Q.unnest
                         [
                           E.(
                             "u" :=
                               Talon.Col.v
                                 (Talon.Kind.list Talon.Record.kind)
                                 "unit_snapshots");
                         ]
                    |> Q.select
                         [
                           E.(
                             "id" :=
                               field Talon.Kind.string "id"
                                 (Talon.Col.v Talon.Record.kind "u"));
                           E.(
                             "type" :=
                               field Talon.Kind.string "type"
                                 (Talon.Col.v Talon.Record.kind "u"));
                           E.(
                             "faction" :=
                               field Talon.Kind.string "faction"
                                 (Talon.Col.v Talon.Record.kind "u"));
                           E.(
                             "role" :=
                               field Talon.Kind.string "role"
                                 (Talon.Col.v Talon.Record.kind "u"));
                           E.(
                             "hp" :=
                               cast Talon.Type.int64
                                 (field Talon.Kind.int "hp"
                                    (Talon.Col.v Talon.Record.kind "u")));
                           E.(
                             "col" :=
                               cast Talon.Type.int64
                                 (field Talon.Kind.int "col"
                                    (field Talon.Record.kind "tile"
                                       (Talon.Col.v Talon.Record.kind "u"))));
                           E.(
                             "row" :=
                               cast Talon.Type.int64
                                 (field Talon.Kind.int "row"
                                    (field Talon.Record.kind "tile"
                                       (Talon.Col.v Talon.Record.kind "u"))));
                           E.(
                             "rebuilding" :=
                               field Talon.Kind.bool "rebuilding"
                                 (Talon.Col.v Talon.Record.kind "u"));
                         ]
                    |> R.run
                  in
                  let projected =
                    Talon.v
                      (List.map
                         (fun (n, _) -> (n, Talon.column projected n))
                         (Talon.Schema.columns (Talon.schema projected))
                      @ [
                          ( "snapshot",
                            Talon.Column.v Talon.Type.int64
                              (Array.init (Talon.rows projected) Fun.id) );
                        ])
                    |> R.derive
                         [
                           R.out "m" (E.int m);
                           R.out "side" E.(store Talon.Type.string null);
                           R.out "name" E.(store Talon.Type.string null);
                         ]
                    |> R.derive
                         [
                           R.out "cell"
                             E.(
                               Str.concat ","
                                 [
                                   store Talon.Type.string
                                     (const string_of_int $ R.i "col");
                                   store Talon.Type.string
                                     (const string_of_int $ R.i "row");
                                 ]);
                         ]
                  in
                  snapshot_tables := projected :: !snapshot_tables;
                  let strings =
                    Talon.Column.options Talon.Kind.string
                      (Talon.column table "phase_json")
                  in
                  let chapters =
                    Talon.Column.options Talon.Kind.int
                      (Talon.column table "chapter")
                  in
                  let owner_indices =
                    Talon.Column.options Talon.Kind.int
                      (Talon.column table "ownership")
                  in
                  let before =
                    Talon.Column.options
                      (Talon.Kind.list Talon.Kind.int)
                      (Talon.column table "before")
                  and units =
                    Talon.Column.options
                      (Talon.Kind.list Talon.Kind.int)
                      (Talon.column table "units")
                  in
                  for n = 1 to Talon.rows table - 1 do
                    let ch = Option.get chapters.(n)
                    and owner = Option.get owner_indices.(n) in
                    Journal.require
                      (owner >= 0 && owner < Array.length ownership)
                      "Invalid Arrow ownership index";
                    let indices values =
                      Array.to_list (Option.get values.(n))
                      |> List.map (fun v ->
                          Journal.require
                            (v >= 0 && v < Talon.rows projected)
                            "Invalid Arrow roster index";
                          v)
                    in
                    add_phase ch
                      (parse (Option.get strings.(n)))
                      (indices before) (indices units) owner
                  done)
                else (
                  Array.iteri
                    (fun ch chapter ->
                      let factions =
                        list (get (get chapter "initial") "factions")
                      in
                      List.iter
                        (fun phase ->
                          let indices which =
                            List.map
                              (fun u ->
                                let snapshot = !legacy_snapshot in
                                incr legacy_snapshot;
                                B.add json_snapshots (fun n ->
                                    match n with
                                    | "m" -> int m
                                    | "snapshot" -> int snapshot
                                    | _ -> unit_value factions u n);
                                snapshot)
                              (list (get phase which))
                          in
                          let before = indices "before"
                          and units = indices "units" in
                          let owner = List.length !legacy_owners in
                          legacy_owners :=
                            get phase "ownership" :: !legacy_owners;
                          add_phase ch phase before units owner)
                        (list (get chapter "phases")))
                    missions;
                  Hashtbl.replace owners m
                    (Array.of_list (List.rev !legacy_owners))))))
        reports;
      let meta = B.finish meta
      and chapters = B.finish chapters
      and profiles = B.finish profiles
      and tiles = B.finish tiles in
      let snapshot_schema =
        List.map fst
          (ints [ "m"; "snapshot" ]
          @ strings unit_fields
          @ ints [ "hp"; "col"; "row" ]
          @ bools [ "rebuilding" ])
      in
      let snapshots =
        R.concat
          (List.map (R.select snapshot_schema)
             (B.finish json_snapshots :: !snapshot_tables))
      in
      let refs = B.finish refs in
      let faction_rows =
        B.create (ints [ "m"; "ch" ] @ strings [ "faction"; "factionSide" ])
      in
      List.iter
        (fun h ->
          let missions =
            if get h.result "kind" = str "arena" then [ h.result ]
            else list (get h.result "missions")
          in
          List.iteri
            (fun ch chapter ->
              List.iter
                (fun faction ->
                  B.add faction_rows (fun n ->
                      match n with
                      | "m" -> int h.m
                      | "ch" -> int ch
                      | "factionSide" -> side (get faction "phase")
                      | _ -> get faction "id"))
                (list (get (get chapter "initial") "factions")))
            missions)
        !headers;
      let factions = B.finish faction_rows in
      let roster which =
        let result =
          Q.of_table refs
          |> Q.filter E.(R.s "which" = string which)
          |> R.qselect [ "p"; "m"; "ch"; "snapshot" ]
          |> R.qjoin ~required:true [ "m"; "snapshot" ] snapshots
          |> R.qjoin ~kind:Talon.Join.Left ~one:true [ "m"; "ch"; "faction" ]
               factions
          |> Q.derive
               [ R.out "side" E.(coalesce [ R.s "side"; R.s "factionSide" ]) ]
          |> R.qselect unit_keys |> R.run
        in
        Journal.require
          (Talon.rows (R.filter E.(is_null (R.s "side")) result) = 0)
          "Unit snapshot has no recorded faction/side mapping";
        result
      in
      let before = roster "before"
      and units = roster "units"
      and losses = B.finish losses in
      let phases =
        B.finish phases
        |> R.derive
             [ R.out "phase" E.(over ~by:[ "m"; "ch" ] (window rows) - int 1) ]
      in
      let identity_columns =
        [ "p"; "m"; "ch"; "id"; "type"; "side"; "faction"; "role"; "cell" ]
      in
      let before_keys = R.select identity_columns before
      and after_keys = R.select identity_columns units
      and loss_keys = R.select identity_columns losses in
      let lookup =
        if telemetry then
          R.concat [ before_keys; loss_keys; after_keys ]
          |> R.unique_last [ "p"; "id" ]
        else R.Q.of_table after_keys |> R.Q.slice ~offset:0 ~length:0 |> R.run
      in
      let support_lookup =
        if telemetry then
          R.concat [ loss_keys; after_keys; before_keys ]
          |> R.unique_last [ "p"; "id" ]
        else R.Q.of_table before_keys |> R.Q.slice ~offset:0 ~length:0 |> R.run
      in
      {
        meta;
        chapters;
        profiles;
        tiles;
        initial = B.finish initial;
        final = B.finish final;
        phases;
        before;
        units;
        losses;
        lookup;
        support_lookup;
        actions = B.finish actions;
        combats = B.finish combats;
        healing = B.finish healing;
        audits = B.finish audits;
        goals = B.finish goals;
        objectives =
          B.finish objectives |> R.unique [ "m"; "ch"; "side"; "cell" ];
        duties = B.finish duties;
        constraints = B.finish constraints;
        headers = List.rev !headers;
        owners;
      })

let describe ?columns d units =
  Q.of_table units
  |> R.qjoin ~kind:Talon.Join.Left ~one:true [ "m" ] d.meta
  |> R.qjoin ~kind:Talon.Join.Left ~one:true [ "m"; "ch" ] d.chapters
  |> R.qjoin ~kind:Talon.Join.Left ~one:true [ "type" ] d.profiles
  |> Q.derive [ R.out "family" E.(coalesce [ R.s "family"; R.s "type" ]) ]
  |> Q.derive
       E.
         [
           "race" :=
             if_
               (R.s "kind" = string "arena")
               (if_
                  (R.s "side" = string "player")
                  (R.s "playerRace") (R.s "enemyRace"))
               (R.s "profileRace");
           "policy" :=
             if_ (R.s "side" = string "player") (R.s "player") (R.s "enemy");
           "opponentPolicy" :=
             if_ (R.s "side" = string "player") (R.s "enemy") (R.s "player");
           "map" :=
             if_ (R.s "kind" = string "arena") (R.s "landscape") (R.s "mission");
         ]
  |> (fun q ->
  match columns with None -> q | Some columns -> R.qselect columns q)
  |> R.run
