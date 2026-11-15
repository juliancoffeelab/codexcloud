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
  dataset : Dataset.t;
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

let load ?(telemetry = false) paths =
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
  let dataset = Dataset.load reports ~telemetry in
  let observations = Campaign_observations.build dataset in
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
  { outcomes; sources = List.rev !sources; reports; dataset }
