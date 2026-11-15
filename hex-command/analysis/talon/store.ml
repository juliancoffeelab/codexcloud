open Json

let source_fields =
  [
    "sourceCommit";
    "fingerprint";
    "status";
    "mode";
    "schedule";
    "seeds";
    "elapsedSeconds";
    "detailFormat";
    "turnTiming";
    "recruitmentLimits";
    "recruitmentProfiles";
    "unitFamilies";
    "unitProfiles";
    "supportTelemetryVersion";
    "combatTelemetryVersion";
    "leaderAuditVersion";
  ]

type t = {
  outcomes : Json.t list;
  sources : Json.t list;
  reports : (string * Json.t) list;
  telemetry : (string * Talon.t) list;
  profiles : Talon.t;
}

let with_kind result =
  if has result "kind" then result
  else
    set result "kind"
      (str
         (if has (get result "config") "campaign" then "campaign" else "arena"))

let is_dir path = Sys.file_exists path && Sys.is_directory path

let files paths =
  let rec expand path =
    if not (is_dir path) then [ path ]
    else
      match
        List.find_opt
          (fun n -> Sys.file_exists (Filename.concat path n))
          [ "report.json"; "report.json.gz"; "outcomes.json" ]
      with
      | Some n -> [ Filename.concat path n ]
      | None ->
          Sys.readdir path |> Array.to_list |> List.sort String.compare
          |> List.filter (fun n -> is_dir (Filename.concat path n))
          |> List.concat_map (fun n -> expand (Filename.concat path n))
  in
  List.concat_map expand paths

let load ?(unit_usage = false) ?(presence = false) ?(support = false)
    ?(value = false) ?(leaders = false) paths =
  let filenames = files paths in
  Journal.require
    (paths = [] || filenames <> [])
    "No recorded reports in the selected inputs";
  let seen = Hashtbl.create 2048 in
  let outcomes = ref [] and sources = ref [] and reports = ref [] in
  List.iter
    (fun path ->
      let directory = Filename.dirname path in
      let raw = read path in
      let raw =
        if Filename.basename path = "outcomes.json" then
          let parent =
            Filename.concat (Filename.dirname directory) "metadata.json"
          in
          merge
            (if Sys.file_exists parent then read parent else obj [])
            [ ("outcomes", raw); ("mode", str "campaign") ]
        else raw
      in
      let report = normalize raw in
      let declared = List.map with_kind (list (get report "outcomes")) in
      let report = set report "outcomes" (arr declared) in
      List.iter
        (fun o ->
          let key = identity o in
          Journal.require
            (not (Hashtbl.mem seen key))
            ("Duplicate match configuration: " ^ key);
          Hashtbl.add seen key ();
          if get o "kind" = str "arena" then (
            Journal.require
              (List.mem (get o "winner") [ `Null; str "player"; str "enemy" ])
              ("Invalid winner: " ^ key);
            Journal.require
              (get o "winner" = `Null = (get o "reason" = str "cap"))
              ("Winner/cap inconsistency: " ^ key));
          outcomes := o :: !outcomes)
        declared;
      let source = obj (List.map (fun k -> (k, get report k)) source_fields) in
      sources :=
        merge source
          [
            ( "fingerprint",
              default report "fingerprint" (get report "fingerprints") );
            ( "file",
              str
                (Unix.realpath
                   (if Filename.basename path = "outcomes.json" then directory
                    else path)) );
            ("matches", int (List.length declared));
          ]
        :: !sources;
      reports := (path, report) :: !reports)
    filenames;
  let reports = List.rev !reports in
  let sources = List.rev !sources in
  if unit_usage then
    List.iter
      (fun source ->
        Journal.require
          (get source "combatTelemetryVersion" = int 1)
          "Damage tables require fresh combat telemetry")
      sources;
  let presence = unit_usage || presence and support = unit_usage || support in
  let value =
    value
    || unit_usage
       && List.for_all
            (fun source -> get source "combatTelemetryVersion" = int 1)
            sources
  in
  let leaders =
    leaders
    || unit_usage
       && List.for_all
            (fun source -> get source "leaderAuditVersion" = int 1)
            sources
  in
  let detailed = presence || support || value || leaders in
  (match sources with
  | first :: rest when detailed ->
      List.iter
        (fun source ->
          Journal.require
            (sorted (get source "unitProfiles")
            = sorted (get first "unitProfiles"))
            "Cannot pool telemetry across different recorded catalogs")
        rest
  | _ -> ());
  let observations = Hashtbl.create 128
  and partials = ref []
  and profiles = ref None in
  Journal.with_readers (fun readers ->
      let batch reports =
        let dataset = Dataset.load ~readers reports ~telemetry:detailed in
        if Option.is_none !profiles then profiles := Some dataset.profiles;
        Hashtbl.iter
          (Hashtbl.replace observations)
          (Campaign_observations.build dataset);
        if detailed then
          partials :=
            Telemetry.analyze dataset ~presence ~support ~value ~leaders
            :: !partials
      in
      List.iter
        (fun (path, report) ->
          if detailed then
            Journal.require
              (get report "status" = str "complete")
              "Telemetry requires a completed run";
          let selected =
            list (get report "outcomes")
            |> List.filter (fun o ->
                get o "kind" = str "campaign"
                || detailed
                   && get (get (get o "config") "armies") "player"
                      <> get (get (get o "config") "armies") "enemy")
            |> List.sort (fun a b ->
                compare
                  ( text (get (get a "detail") "file"),
                    integer (get (get a "detail") "offset") )
                  ( text (get (get b "detail") "file"),
                    integer (get (get b "detail") "offset") ))
          in
          let size =
            if Filename.basename path = "outcomes.json" then
              max 1 (List.length selected)
            else 32
          in
          let rec chunks remaining =
            let rec take n rest acc =
              if n = 0 then (List.rev acc, rest)
              else
                match rest with
                | [] -> (List.rev acc, [])
                | x :: xs -> take (n - 1) xs (x :: acc)
            in
            let current, rest = take size remaining [] in
            batch [ (path, set report "outcomes" (arr current)) ];
            if rest <> [] then chunks rest
          in
          chunks selected)
        reports;
      if reports = [] then batch []);
  let outcomes =
    List.rev !outcomes
    |> List.map (fun o ->
        if get o "kind" = str "campaign" then
          set o "analysis"
            (arr
               (Option.value ~default:[]
                  (Hashtbl.find_opt observations (identity o))))
        else o)
  in
  {
    outcomes;
    sources;
    reports;
    profiles = Option.get !profiles;
    telemetry = (if detailed then Telemetry.merge (List.rev !partials) else []);
  }
