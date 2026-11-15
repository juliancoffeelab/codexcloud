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
}

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

let load paths =
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
          let meta = if Sys.file_exists parent then read parent else obj [] in
          merge meta [ ("outcomes", raw); ("mode", str "campaign") ]
        else raw
      in
      let report = normalize raw in
      let declared = list (get report "outcomes") in
      let legacy = Hashtbl.create 16 in
      let journal = Filename.concat directory "details.jsonl.gz" in
      (if Filename.basename path = "outcomes.json" && Sys.file_exists journal
       then
         let channel =
           Unix.open_process_in ("gzip -dc -- " ^ Filename.quote journal)
         in
         Fun.protect
           ~finally:(fun () -> ignore (Unix.close_process_in channel))
           (fun () ->
             try
               while true do
                 let o = normalize (parse (input_line channel)) in
                 let o =
                   if has o "kind" then o
                   else
                     set o "kind"
                       (str (if has o "missions" then "campaign" else "arena"))
                 in
                 Hashtbl.replace legacy (identity o)
                   (arr
                      (List.map Campaign.diagnostics (list (get o "missions"))))
               done
             with End_of_file -> ()));
      let completed = Hashtbl.create (List.length declared) in
      let physical_order =
        List.sort
          (fun a b ->
            compare
              ( text (get (get a "detail") "file"),
                integer (get (get a "detail") "offset") )
              ( text (get (get b "detail") "file"),
                integer (get (get b "detail") "offset") ))
          declared
      in
      List.iter
        (fun o ->
          let o =
            if has o "kind" then o
            else
              set o "kind"
                (str
                   (if has (get o "config") "campaign" then "campaign"
                    else "arena"))
          in
          let key = identity o in
          Journal.require
            (not (Hashtbl.mem seen key))
            ("Duplicate match configuration: " ^ key);
          Hashtbl.add seen key ();
          let o =
            if get o "kind" = str "arena" then (
              Journal.require
                (List.mem (get o "winner") [ `Null; str "player"; str "enemy" ])
                ("Invalid winner: " ^ key);
              Journal.require
                (get o "winner" = `Null = (get o "reason" = str "cap"))
                ("Winner/cap inconsistency: " ^ key);
              o)
            else
              let analysis =
                match Hashtbl.find_opt legacy key with
                | Some a -> a
                | None ->
                    let ref_ = get o "detail" in
                    let detail =
                      if
                        List.for_all (has ref_)
                          [ "file"; "offset"; "bytes"; "sha256" ]
                      then (
                        let result = normalize (Journal.read directory ref_) in
                        Journal.require
                          (identity result = key)
                          "Gameplay journal configuration mismatch";
                        result)
                      else o
                    in
                    arr
                      (List.map Campaign.diagnostics
                         (list (get detail "missions")))
              in
              set o "analysis" analysis
          in
          Hashtbl.add completed key o)
        physical_order;
      List.iter
        (fun saved ->
          let saved =
            if has saved "kind" then saved
            else
              set saved "kind"
                (str
                   (if has (get saved "config") "campaign" then "campaign"
                    else "arena"))
          in
          outcomes := Hashtbl.find completed (identity saved) :: !outcomes)
        declared;
      let source =
        obj (List.map (fun k -> (k, get report k)) source_fields) |> fun s ->
        merge s
          [
            ( "fingerprint",
              default report "fingerprint" (get report "fingerprints") );
            ("file", str (Unix.realpath path));
            ("matches", int (List.length declared));
          ]
      in
      sources := source :: !sources;
      reports := (path, report) :: !reports)
    (files paths);
  {
    outcomes = List.rev !outcomes;
    sources = List.rev !sources;
    reports = List.rev !reports;
  }

let journals store f =
  List.iter
    (fun (path, report) ->
      Journal.require
        (get report "status" = str "complete")
        "Unit usage requires a completed run";
      list (get report "outcomes")
      |> List.sort (fun a b ->
          compare
            ( text (get (get a "detail") "file"),
              integer (get (get a "detail") "offset") )
            ( text (get (get b "detail") "file"),
              integer (get (get b "detail") "offset") ))
      |> List.iter (fun saved ->
          let result =
            normalize
              (Journal.read (Filename.dirname path) (get saved "detail"))
          in
          Journal.require
            (sorted (get result "config") = sorted (get saved "config"))
            "Unit usage journal configuration mismatch";
          f report result))
    store.reports
