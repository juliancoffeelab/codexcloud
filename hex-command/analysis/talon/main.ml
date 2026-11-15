open Json

let () =
  Printexc.record_backtrace true;
  try
    if Array.length Sys.argv = 4 && Sys.argv.(1) = "decode" then
      let report = read Sys.argv.(2) in
      let outcome =
        List.nth (list (get report "outcomes")) (int_of_string Sys.argv.(3))
      in
      Journal.read (Filename.dirname Sys.argv.(2)) (get outcome "detail")
      |> encode |> print_endline
    else if
      Array.length Sys.argv > 1
      && List.mem Sys.argv.(1) [ "compare-report"; "compare-dump" ]
    then (
      let mode = Sys.argv.(1) and paths = ref [] and output = ref "" in
      Arg.current := 1;
      Arg.parse
        [ ("--out", Arg.Set_string output, "Save comparison JSON") ]
        (fun path -> paths := !paths @ [ path ])
        "hex-analysis compare-report|compare-dump BASELINE CURRENT [--out \
         RESULT.json]";
      let a, b =
        match !paths with
        | [ a; b ] -> (a, b)
        | _ -> failwith "Two comparison inputs are required"
      in
      let result =
        (if mode = "compare-report" then Compare.reports else Compare.dumps) a b
      in
      if !output <> "" then Json.write !output result;
      print_endline (Json.encode result);
      if not (Json.truth (Json.get result "equal")) then exit 1)
    else
      let inputs = ref [] and baseline = ref [] and output = ref "" in
      let unit_usage = ref false
      and presence = ref false
      and support = ref false
      and value = ref false
      and leaders = ref false in
      Arg.parse
        [
          ("--out", Arg.Set_string output, "Output directory");
          ( "--baseline",
            Arg.String (fun s -> baseline := !baseline @ [ s ]),
            "Baseline report (repeatable)" );
          ("--unit-usage", Arg.Set unit_usage, "Analyze all unit telemetry");
          ("--battlefield-presence", Arg.Set presence, "Analyze presence");
          ("--support-effects", Arg.Set support, "Analyze support");
          ( "--battlefield-value",
            Arg.Set value,
            "Analyze protection, captures and casualty value" );
          ("--leader-audit", Arg.Set leaders, "Analyze leader risk");
        ]
        (fun s -> inputs := !inputs @ [ s ])
        "hex-analysis REPORT [REPORT...] --out DIRECTORY";
      Journal.require
        (!inputs <> [] && !output <> "")
        "Reports and --out are required";
      let start = Unix.gettimeofday () in
      let current = Store.load !inputs and previous = Store.load !baseline in
      let tables =
        Analysis.build current previous ~unit_usage:!unit_usage
          ~presence:!presence ~support:!support ~value:!value ~leaders:!leaders
      in
      Report.write !output current previous tables
        (Unix.gettimeofday () -. start);
      let seconds = Unix.gettimeofday () -. start in
      Json.write
        (Filename.concat !output "timings.json")
        (obj
           [
             ("reportSeconds", float seconds);
             ("physicalMatches", int (List.length current.outcomes));
           ]);
      Printf.printf "%d physical matches; %.3fs. %s/summary.md\n%!"
        (List.length current.outcomes)
        seconds !output
  with error ->
    prerr_endline (Printexc.to_string error);
    prerr_endline (Printexc.get_backtrace ());
    exit 1
