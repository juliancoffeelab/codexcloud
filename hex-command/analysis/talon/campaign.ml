open Json

let diagnostics chapter =
  let phases = list (get chapter "phases") in
  let own = List.filter (fun p -> get p "side" = str "player") phases in
  let final = get chapter "final" in
  let units = list (get final "units") in
  let winner = get chapter "winner" in
  let reason = get (get final "campaign") "outcome" in
  let reason =
    if winner = `Null && not (truth reason) then
      str "Round limit reached with unfinished objectives"
    else reason
  in
  let goals = list (get chapter "goals") in
  let named =
    List.filter
      (fun u ->
        truth (get u "name")
        && get u "side" = str "player"
        && List.exists
             (fun g -> contains (text (get g "label")) (text (get u "name")))
             goals)
      (list (get (get chapter "initial") "units"))
  in
  let arrivals =
    List.map
      (fun actor ->
        let name = text (get actor "name") in
        let arrived =
          List.exists
            (fun p ->
              List.exists
                (fun u ->
                  get u "name" = str name
                  && List.exists
                       (fun g ->
                         contains (text (get g "label")) name
                         && List.mem (get u "tile") (list (get g "tiles")))
                       goals)
                (list (get p "units")))
            phases
        in
        (name, bool arrived))
      named
  in
  let losses =
    List.concat_map
      (fun p ->
        list (get p "losses")
        |> List.filter (fun u -> get u "side" = str "player"))
      phases
  in
  let rec idle = function
    | p :: rest when not (truth (get p "actions")) -> 1 + idle rest
    | _ -> 0
  in
  let last_own = match List.rev own with x :: _ -> x | _ -> `Null in
  let signatures =
    let rev = List.rev own in
    List.mapi (fun i x -> (i, x)) rev
    |> List.filter (fun (i, _) -> i < 10)
    |> List.map (fun (_, p) ->
        canonical
          (arr
             [
               arr
                 (List.map
                    (fun u ->
                      arr (List.map (get u) [ "id"; "type"; "tile"; "hp" ]))
                    (list (get p "units")));
               get p "ownership";
               get p "goals";
             ]))
  in
  let repeats =
    fields (counts signatures)
    |> List.fold_left (fun n (_, v) -> max n (integer v)) 0
  in
  let named_names = List.map (fun u -> get u "name") named in
  let category =
    if winner = str "player" then "victory"
    else if winner = `Null then "round cap"
    else if
      has final "units"
      && not (List.exists (fun u -> get u "side" = str "player") units)
    then "all player units lost"
    else if List.exists (fun u -> get u "role" = str "leader") losses then
      "leader lost"
    else if
      List.exists
        (fun u ->
          List.mem (get u "name") named_names || get u "role" = str "courier")
        losses
      || truth reason
         && List.exists
              (contains (String.lowercase_ascii (text reason)))
              [ "courier"; "companion" ]
    then "required actor lost"
    else if truth reason then "defeat; see recorded reason"
    else "defeat; cause not recorded"
  in
  let stalled =
    if winner <> `Null || not (truth (get last_own "planning")) then []
    else
      let planning = get last_own "planning" in
      let roster = index_by "id" (list (get last_own "units")) in
      let duties =
        index_by "id" (list (get (get planning "report") "duties"))
      in
      list (get (get (get planning "orders") "tactics") "objectives")
      |> List.filter_map (fun order ->
          let objective = get order "objective" in
          let body =
            if truth (get objective "GoTo") then get objective "GoTo"
            else get objective "Occupy"
          in
          if not (truth body) then None
          else
            let actor_id = "u" ^ text (get body "actor") in
            let actor = lookup roster actor_id in
            let name =
              if truth (get actor "name") then get actor "name"
              else if truth (get actor "type") then get actor "type"
              else str actor_id
            in
            Some
              (obj
                 [
                   ("actor", name);
                   ("tile", get actor "tile");
                   ("hp", get actor "hp");
                   ("target", default body "destination" (get body "location"));
                   ( "reason",
                     get (lookup duties (text (get order "id"))) "reason" );
                   ( "allies",
                     arr
                       (List.filter
                          (fun u -> get u "side" = str "player")
                          (list (get last_own "units"))
                       |> List.map (fun u ->
                           obj
                             (List.map
                                (fun k -> (k, get u k))
                                [ "id"; "type"; "tile"; "rebuilding" ]))) );
                 ]))
  in
  let recruits = ref []
  and casualties = ref []
  and unmet = ref []
  and deferred = ref [] in
  let player_damage = ref 0
  and enemy_damage = ref 0
  and rescue = ref 0
  and retaliation = ref 0 in
  List.iter
    (fun p ->
      let side = text (get p "side") in
      let roster =
        index_by "id"
          (List.concat_map
             (fun k -> list (get p k))
             [ "before"; "units"; "losses" ])
      in
      List.iter
        (fun a ->
          if get a "kind" = str "recruit" && side = "player" then
            recruits := text (get a "type") :: !recruits)
        (list (get p "actions"));
      List.iter
        (fun u ->
          if get u "side" = str "player" then
            casualties := text (get u "type") :: !casualties)
        (list (get p "losses"));
      List.iter
        (fun c ->
          let attacker = lookup roster (text (get c "unit")) in
          let own = default attacker "side" (get p "side") = str "player" in
          player_damage :=
            !player_damage
            + integer (get c (if own then "damage" else "retaliation"));
          enemy_damage :=
            !enemy_damage
            + integer (get c (if own then "retaliation" else "damage")))
        (list (get p "combats"));
      if side = "player" && truth (get p "planning") then (
        let planning = get p "planning" in
        let report = get planning "report" in
        List.iter
          (fun d ->
            let reason =
              if truth (get d "reason") then text (get d "reason")
              else "unspecified"
            in
            if get d "status" = str "Unmet" then unmet := reason :: !unmet
            else if get d "status" = str "Deferred" then
              deferred := reason :: !deferred)
          (list (get report "duties"));
        rescue := !rescue + integer (get report "candidates");
        let protected =
          list (get (get (get planning "orders") "tactics") "constraints")
          |> List.filter_map (fun c ->
              if has c "Survive" then
                Some (str ("u" ^ text (get (get c "Survive") "actor")))
              else None)
        in
        List.iter
          (fun c ->
            if
              List.mem (get c "unit") protected
              && integer (get c "retaliation") > 0
            then incr retaliation)
          (list (get p "combats"))))
    phases;
  obj
    [
      ("id", get chapter "id");
      ("round", get chapter "round");
      ("winner", winner);
      ("category", str category);
      ("reason", reason);
      ("idleTail", int (idle (List.rev own)));
      ("repeatedTail", int repeats);
      ( "finalPlayerUnits",
        int
          (List.length
             (List.filter (fun u -> get u "side" = str "player") units)) );
      ( "finalEnemyUnits",
        int
          (List.length
             (List.filter (fun u -> get u "side" = str "enemy") units)) );
      ( "maxGoalProgress",
        arr
          (List.mapi
             (fun i g ->
               let seen =
                 List.filter_map
                   (fun p ->
                     List.nth_opt (list (get p "goals")) i
                     |> Option.map (fun x -> integer (get x "current")))
                   phases
               in
               int
                 (match seen with
                 | [] -> integer (get g "current")
                 | x :: xs -> List.fold_left max x xs))
             goals) );
      ("goals", arr goals);
      ( "namedLosses",
        counts
          (List.filter_map
             (fun u ->
               if truth (get u "name") then Some (text (get u "name")) else None)
             losses) );
      ("recruits", counts !recruits);
      ("casualties", counts !casualties);
      ("playerDamage", int !player_damage);
      ("enemyDamage", int !enemy_damage);
      ("namedGoalActors", arr named_names);
      ("actorArrivals", obj arrivals);
      ("stalledActors", arr stalled);
      ("recordedPhases", bool (phases <> []));
      ("unmetOrders", counts !unmet);
      ("deferredOrders", counts !deferred);
      ("rescueCandidates", int !rescue);
      ("protectedRetaliationExchanges", int !retaliation);
      ( "recordedPlanning",
        bool (List.exists (fun p -> truth (get p "planning")) own) );
    ]

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
  let campaigns =
    Frame.group !attempts fields_
      [ "attempts"; "completed"; "defeats"; "caps"; "unclassified" ]
    |> List.map (fun r ->
        set r "completionRate"
          (float
             (100. *. number (get r "completed") /. number (get r "attempts"))))
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
  let group_key r keys = canonical (arr (List.map (get r) keys)) in
  let lookup = Hashtbl.create 16 in
  List.iter
    (fun r -> Hashtbl.replace lookup (group_key r fields_) (get r "attempts"))
    campaigns;
  let stages =
    Frame.group !chapters stage_keys sums
    |> List.map (fun r ->
        merge r
          [
            ( "victoryRate",
              float
                (100. *. number (get r "victories") /. number (get r "reached"))
            );
            ("campaignAttempts", Hashtbl.find lookup (group_key r fields_));
            ( "meanRounds",
              float (number (get r "rounds") /. number (get r "reached")) );
          ])
  in
  let failure_keys = stage_keys @ [ "category"; "reason" ] in
  let failures =
    Frame.group !failures failure_keys [ "attempts" ]
    |> List.map (fun r ->
        let seeds =
          List.filter
            (fun x -> group_key x failure_keys = group_key r failure_keys)
            !failures
          |> List.map (fun x -> get x "seed")
          |> List.filter (( <> ) `Null)
          |> List.sort_uniq compare
        in
        set r "exampleSeeds" (arr (List.filteri (fun i _ -> i < 3) seeds)))
  in
  [
    ("campaign_summary", campaigns);
    ("campaign_stages", stages);
    ("campaign_failures", failures);
    ( "campaign_unit_events",
      Frame.group !events (stage_keys @ [ "event"; "unit" ]) [ "count" ] );
    ("campaign_runs", List.rev !runs);
  ]
