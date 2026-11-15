open Json

let table tables name = Option.value ~default:[] (List.assoc_opt name tables)

module R = Rel
module E = Talon.Expr

let changes current previous keys fields =
  let before =
    previous
    |> R.select (keys @ fields)
    |> R.rename (List.map (fun n -> (n, n ^ "Before")) fields)
  in
  current |> R.join ~one:true keys before

let match_changes current previous =
  if previous = [] then []
  else
    let index = Hashtbl.create 2048 in
    List.iter (fun o -> Hashtbl.add index (identity o) o) previous;
    Journal.require
      (List.length current = List.length previous
      && List.for_all (fun o -> Hashtbl.mem index (identity o)) current)
      "Unmatched configurations";
    List.map
      (fun o ->
        let old = Hashtbl.find index (identity o) in
        if get o "kind" = str "campaign" then
          let summary r fields_ =
            arr
              (get r "completed"
              :: List.map
                   (fun m -> arr (List.map (get m) fields_))
                   (list (get r "missions")))
          in
          obj
            [
              ("matchId", str (identity o));
              ("kind", str "campaign");
              ("beforeCompleted", get old "completed");
              ("completed", get o "completed");
              ("beforeMissions", get old "missions");
              ("missions", get o "missions");
              ( "resultChanged",
                bool
                  (summary old [ "id"; "winner" ]
                  <> summary o [ "id"; "winner" ]) );
              ( "gameplayChanged",
                bool
                  (summary old [ "id"; "round"; "stateHash" ]
                  <> summary o [ "id"; "round"; "stateHash" ]) );
            ]
        else
          obj
            [
              ("matchId", str (identity o));
              ("kind", str "arena");
              ("beforeWinner", get old "winner");
              ("winner", get o "winner");
              ("beforeReason", get old "reason");
              ("reason", get o "reason");
              ("beforeRounds", get old "rounds");
              ("rounds", get o "rounds");
              ( "resultChanged",
                bool
                  (List.map (get old) [ "winner"; "reason" ]
                  <> List.map (get o) [ "winner"; "reason" ]) );
              ( "gameplayChanged",
                bool (get old "stateHash" <> get o "stateHash") );
            ])
      current

let build current previous ~unit_usage ~presence ~support ~value ~leaders =
  let rows, units, ledger, campaigns = Arena.tally current.Store.outcomes in
  let before, before_units, _, _ = Arena.tally previous.Store.outcomes in
  let transitions = match_changes current.outcomes previous.outcomes in
  let families =
    List.fold_left
      (fun all source ->
        List.fold_left
          (fun all (k, v) ->
            Journal.require
              ((not (has all k)) || get all k = v)
              ("Conflicting unit family: " ^ k);
            set all k v)
          all
          (fields (get source "unitFamilies")))
      (obj []) current.sources
  in
  let purchases = Arena.purchase_shares units families in
  let campaign_tables, presentation = Campaign.tables current.outcomes in
  let old_campaigns, _ = Campaign.tables previous.outcomes in
  let tab name tables = List.assoc name tables in
  let matchup_changes =
    changes rows before Arena.keys [ "wins"; "losses"; "caps"; "winRate" ]
    |> R.derive
         [ R.out "winRateDelta" E.(R.f "winRate" -. R.f "winRateBefore") ]
  in
  (* Empty baselines retain the current key types rather than inventing a
     different empty schema for optional numeric campaign configuration. *)
  let old_summary = tab "campaign_summary" old_campaigns in
  let old_summary =
    if Talon.rows old_summary = 0 then
      R.Q.of_table (tab "campaign_summary" campaign_tables)
      |> R.Q.slice ~offset:0 ~length:0
      |> R.run
    else old_summary
  in
  let campaign_changes =
    changes
      (tab "campaign_summary" campaign_tables)
      old_summary Campaign.fields_
      [ "completed"; "completionRate" ]
    |> R.derive
         [
           R.out "completionRateDelta"
             E.(R.f "completionRate" -. R.f "completionRateBefore");
         ]
  in
  let race_map opponent =
    rows
    |> R.filter
         E.(
           R.s "opponentPolicy" = string opponent
           && (R.s "policy" <> string opponent
              || R.s "race" <> R.s "opponentRace"))
    |> fun rows -> Arena.combine rows [ "race"; "map"; "policy" ]
  in
  let policy_timings =
    List.concat_map
      (fun (label, sources) ->
        List.concat_map
          (fun source ->
            let timing = get source "turnTiming" in
            List.map
              (fun r ->
                merge r
                  [
                    ("run", str label);
                    ("clock", get timing "clock");
                    ("source", get source "file");
                  ])
              (list (get timing "policies")))
          sources)
      [ ("current", current.sources); ("baseline", previous.sources) ]
  in
  let tables =
    campaign_tables
    @ [
        ("matchups", rows);
        ("unit_events", units);
        ("baseline_matchups", before);
        ("baseline_unit_events", before_units);
        ( "policy_matchups",
          Arena.combine ~cross:false
            (R.filter E.(R.s "policy" <> R.s "opponentPolicy") rows)
            [ "policy"; "opponentPolicy" ] );
        ("race_map_vs_classic", race_map "classic");
        ("race_map_vs_balanced", race_map "balanced");
        ("purchase_shares", purchases);
        ("race_equal_policy", Arena.combine ~equal:true rows [ "race" ]);
        ( "race_map_equal_policy",
          Arena.combine ~equal:true rows [ "race"; "map" ] );
        ( "race_map_policy",
          Arena.combine ~equal:true rows [ "race"; "map"; "policy" ] );
        ("race_all_policies", Arena.combine rows [ "race" ]);
        ("matchup_changes", matchup_changes);
        ("campaign_changes", campaign_changes);
      ]
  in
  let presentation =
    presentation
    @ [
        ("matches", ledger);
        ("campaigns", campaigns);
        ("match_changes", transitions);
        ("policy_timings", policy_timings);
      ]
  in
  let tables =
    if not (unit_usage || presence || support || value || leaders) then tables
    else (
      if unit_usage then
        List.iter
          (fun source ->
            Journal.require
              (get source "combatTelemetryVersion" = int 1)
              "Damage tables require fresh combat telemetry")
          (current.sources @ previous.sources);
      let extra = current.telemetry in
      let tables = tables @ extra in
      let tables =
        if unit_usage || presence then
          tables
          @ [
              ( "army_usage",
                Metrics.army_usage
                  (tab "nonleader_unit_events" tables)
                  purchases
                  (tab "battlefield_presence" tables)
                  families );
            ]
        else tables
      in
      if not unit_usage then tables
      else
        let old_extra = previous.telemetry in
        let membership =
          if previous.outcomes = [] then tab "battlefield_units" tables
          else
            R.append
              (tab "battlefield_units" tables)
              (tab "battlefield_units" old_extra)
        in
        let damage =
          Metrics.unit_damage
            (tab "battlefield_units" tables)
            (tab "battlefield_unit_value" tables)
            current.profiles membership
        in
        let tables = tables @ [ ("unit_damage", damage) ] in
        if previous.outcomes = [] then tables
        else
          let old_damage =
            Metrics.unit_damage
              (tab "battlefield_units" old_extra)
              (tab "battlefield_unit_value" old_extra)
              current.profiles membership
          in
          tables
          @ [
              ("baseline_unit_damage", old_damage);
              ("unit_damage_changes", Metrics.compare_damage damage old_damage);
            ])
  in
  (* This is the single analysis-table to presentation-object boundary. *)
  List.map
    (fun (name, table) ->
      let rows = Frame.rows table in
      let rows =
        if List.mem name [ "leader_audit"; "leader_deaths" ] then
          List.map
            (fun row ->
              obj
                (List.map
                   (fun (k, v) ->
                     ( k,
                       if
                         List.mem k [ "beforeTile"; "afterTile"; "minTile" ]
                         && v <> `Null
                       then parse (text v)
                       else v ))
                   (fields row)))
            rows
        else rows
      in
      (name, rows))
    tables
  @ presentation
