open Json
module R = Rel
module E = Talon.Expr

let fields_ =
  [
    "campaign";
    "player";
    "enemy";
    "diplomacy";
    "objective";
    "maxRounds";
    "recruitLimit";
  ]

let config c =
  obj
    (List.map
       (fun k ->
         ( k,
           if List.mem k [ "player"; "enemy" ] then str (label c k)
           else
             default c k
               (str (if k = "objective" then "authored" else "unrecorded")) ))
       fields_)

let tables outcomes =
  let attempts = ref []
  and chapters = ref []
  and failures = ref []
  and events = ref []
  and runs = ref [] in
  List.iter
    (fun o ->
      if get o "kind" = str "campaign" then (
        let c = get o "config" in
        let dims = config c in
        let ms = list (get o "analysis") in
        let completed = truth (get o "completed") in
        let ending =
          match List.rev ms with
          | x :: _ -> get x "winner"
          | _ -> str "unclassified"
        in
        let flag x = int (if x then 1 else 0) in
        attempts :=
          merge dims
            [
              ("attempts", int 1);
              ("completed", flag completed);
              ( "defeats",
                flag ((not completed) && ms <> [] && ending = str "enemy") );
              ("caps", flag ((not completed) && ms <> [] && ending = `Null));
              ( "unclassified",
                flag
                  ((not completed)
                  && (ms = [] || (ending <> str "enemy" && ending <> `Null))) );
            ]
          :: !attempts;
        List.iteri
          (fun i m ->
            let d =
              merge dims [ ("stage", int (i + 1)); ("map", get m "id") ]
            in
            let win = get m "winner" in
            let actors = truth (get m "namedGoalActors") in
            let arrived =
              List.for_all
                (fun (_, v) -> truth v)
                (fields (get m "actorArrivals"))
            in
            let progress = list (get m "maxGoalProgress") in
            let counts_ =
              [
                ("reached", int 1);
                ("victories", flag (win = str "player"));
                ("defeats", flag (win = str "enemy"));
                ("caps", flag (win = `Null));
                ("rounds", get m "round");
                ("detailed", flag (truth (get m "recordedPhases")));
                ( "idleCaps",
                  flag (win = `Null && integer (get m "idleTail") >= 5) );
                ( "repeatedCaps",
                  flag (win = `Null && integer (get m "repeatedTail") >= 5) );
                ( "zeroProgressDefeats",
                  flag
                    (win = str "enemy"
                    && progress <> []
                    && List.for_all (fun x -> number x = 0.) progress) );
                ("actorAttempts", flag actors);
                ("actorArrivals", flag (actors && arrived));
                ( "actorDefeatsBeforeArrival",
                  flag (win = str "enemy" && actors && not arrived) );
                ( "defeatsWithSurvivors",
                  flag
                    (win = str "enemy" && integer (get m "finalPlayerUnits") > 0)
                );
                ( "survivorsInDefeats",
                  if win = str "enemy" then get m "finalPlayerUnits" else int 0
                );
                ("playerDamage", get m "playerDamage");
                ("enemyDamage", get m "enemyDamage");
              ]
            in
            chapters := merge d counts_ :: !chapters;
            let diagnostics =
              [
                "winner";
                "category";
                "reason";
                "finalPlayerUnits";
                "idleTail";
                "repeatedTail";
                "maxGoalProgress";
                "goals";
                "namedLosses";
                "actorArrivals";
                "unmetOrders";
                "deferredOrders";
                "rescueCandidates";
                "protectedRetaliationExchanges";
                "recordedPlanning";
                "stalledActors";
              ]
            in
            runs :=
              merge d
                ([
                   ("seed", get c "seed");
                   ("round", get m "round");
                   ("detail", get o "detail");
                 ]
                @ List.map (fun k -> (k, get m k)) diagnostics)
              :: !runs;
            if win <> str "player" then
              failures :=
                merge d
                  [
                    ("category", get m "category");
                    ( "reason",
                      if truth (get m "reason") then get m "reason"
                      else str "No terminal explanation recorded" );
                    ("attempts", int 1);
                    ("seed", get c "seed");
                  ]
                :: !failures;
            List.iter
              (fun (event, field) ->
                List.iter
                  (fun (unit, count) ->
                    events :=
                      merge d
                        [
                          ("event", str event);
                          ("unit", str unit);
                          ("count", count);
                        ]
                      :: !events)
                  (fields (get m field)))
              [
                ("recruited", "recruits");
                ("lost", "casualties");
                ("namedLost", "namedLosses");
              ])
          ms))
    outcomes;
  let attempt_fields =
    [ "attempts"; "completed"; "defeats"; "caps"; "unclassified" ]
  in
  let campaigns =
    Frame.ingest !attempts fields_ attempt_fields
    |> R.group fields_ attempt_fields
    |> R.derive
         [
           R.out "completionRate"
             E.(float 100. *. R.f "completed" /. R.f "attempts");
         ]
  in
  let stage_keys = fields_ @ [ "stage"; "map" ] in
  let sums =
    [
      "reached";
      "victories";
      "defeats";
      "caps";
      "rounds";
      "detailed";
      "idleCaps";
      "repeatedCaps";
      "zeroProgressDefeats";
      "actorAttempts";
      "actorArrivals";
      "actorDefeatsBeforeArrival";
      "defeatsWithSurvivors";
      "survivorsInDefeats";
      "playerDamage";
      "enemyDamage";
    ]
  in
  let attempts =
    campaigns
    |> R.select (fields_ @ [ "attempts" ])
    |> R.rename [ ("attempts", "campaignAttempts") ]
  in
  let stages =
    Frame.ingest !chapters stage_keys sums
    |> R.group stage_keys sums |> R.join fields_ attempts
    |> R.derive
         E.
           [
             "victoryRate" := float 100. *. R.f "victories" /. R.f "reached";
             "meanRounds" := R.f "rounds" /. R.f "reached";
           ]
  in
  let failure_keys = stage_keys @ [ "category"; "reason" ] in
  let failures = Frame.ingest !failures failure_keys [ "seed"; "attempts" ] in
  let seeds =
    failures
    |> R.filter E.(not (is_null (R.f "seed")))
    |> R.unique (failure_keys @ [ "seed" ])
    |> R.sort (failure_keys @ [ "seed" ])
    |> R.derive [ R.out "__rank" E.(over ~by:failure_keys (window rows)) ]
    |> R.filter E.(R.i "__rank" <= int 3)
    |> R.aggregate failure_keys
         [ R.out "exampleSeeds" E.(collect (R.f "seed")) ]
  in
  let failures =
    failures
    |> R.group failure_keys [ "attempts" ]
    |> R.join ~kind:Talon.Join.Left failure_keys seeds
  in
  ( [
      ("campaign_summary", campaigns);
      ("campaign_stages", stages);
      ("campaign_failures", failures);
      ( "campaign_unit_events",
        Frame.ingest !events (stage_keys @ [ "event"; "unit" ]) [ "count" ]
        |> R.group (stage_keys @ [ "event"; "unit" ]) [ "count" ] );
    ],
    [ ("campaign_runs", List.rev !runs) ] )
