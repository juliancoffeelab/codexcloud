open Json

let keys = [ "race"; "opponentRace"; "policy"; "opponentPolicy"; "map" ]

let counts_ =
  [ "games"; "wins"; "losses"; "caps"; "leaderWins"; "territoryWins"; "rounds" ]

let event_counts =
  [ "recruited"; "promoted"; "lost"; "attacks"; "retaliations"; "damage" ]

let dims c side =
  let other = other side in
  obj
    [
      ("race", get (get c "armies") side);
      ("opponentRace", get (get c "armies") other);
      ("policy", str (label c side));
      ("opponentPolicy", str (label c other));
      ("map", get c "landscape");
    ]

let rates row =
  let ratio key =
    float (100. *. number (get row key) /. number (get row "games"))
  in
  merge row
    [
      ("winRate", ratio "wins");
      ("lossRate", ratio "losses");
      ("capRate", ratio "caps");
      ( "meanRounds",
        float (number (get row "rounds") /. number (get row "games")) );
    ]

let tally outcomes =
  let games = ref []
  and events = ref []
  and ledger = ref []
  and campaigns = ref [] in
  List.iter
    (fun o ->
      let c = get o "config" in
      let row =
        merge (remove c [ "armies" ])
          ([
             ("matchId", str (identity o));
             ("kind", get o "kind");
             ("winnerSeat", get o "winner");
           ]
          @ List.map
              (fun k -> (k, get o k))
              [
                "reason";
                "rounds";
                "stateHash";
                "detail";
                "recruits";
                "promotions";
                "casualties";
                "combatUsage";
              ])
      in
      if get o "kind" <> str "arena" then (
        let row =
          merge row
            [ ("missions", get o "missions"); ("completed", get o "completed") ]
        in
        ledger := row :: !ledger;
        campaigns := row :: !campaigns)
      else (
        ledger :=
          merge row
            ([
               ("playerRace", get (get c "armies") "player");
               ("enemyRace", get (get c "armies") "enemy");
             ]
            @ fields (dims c "player"))
          :: !ledger;
        List.iter
          (fun side ->
            let d = dims c side in
            let winner = get o "winner" in
            let flag x = int (if x then 1 else 0) in
            games :=
              merge d
                [
                  ("games", int 1);
                  ("wins", flag (winner = str side));
                  ("losses", flag (winner = str (other side)));
                  ("caps", flag (winner = `Null));
                  ( "leaderWins",
                    flag (winner = str side && get o "reason" = str "leader") );
                  ( "territoryWins",
                    flag (winner = str side && get o "reason" = str "territory")
                  );
                  ("rounds", get o "rounds");
                ]
              :: !games;
            List.iter
              (fun (field, count) ->
                List.iter
                  (fun u ->
                    if get u "side" = str side then
                      events :=
                        merge d [ ("unit", get u "type"); (count, int 1) ]
                        :: !events)
                  (list (get o field)))
              [
                ("recruits", "recruited");
                ("promotions", "promoted");
                ("casualties", "lost");
              ];
            List.iter
              (fun u ->
                if get u "side" = str side then
                  events :=
                    merge d
                      ([ ("unit", get u "type") ]
                      @ List.map
                          (fun k -> (k, get u k))
                          [ "attacks"; "retaliations"; "damage" ])
                    :: !events)
              (list (get o "combatUsage")))
          [ "player"; "enemy" ]))
    outcomes;
  ( Frame.group !games keys counts_ |> List.map rates,
    Frame.group !events (keys @ [ "unit" ]) event_counts,
    List.rev !ledger,
    List.rev !campaigns )

let combine ?(equal = false) ?(cross = true) rows keys =
  let rows =
    List.filter
      (fun r ->
        ((not equal) || get r "policy" = get r "opponentPolicy")
        && ((not cross) || get r "race" <> get r "opponentRace"))
      rows
  in
  Frame.group rows keys counts_
  |> List.map (fun r ->
      set r "winRate"
        (float (100. *. number (get r "wins") /. number (get r "games"))))

let purchase_shares events families =
  let facts =
    events
    |> List.filter (fun r -> get r "race" <> get r "opponentRace")
    |> List.concat_map (fun r ->
        List.map
          (fun map ->
            obj
              [
                ("race", get r "race");
                ("policy", get r "policy");
                ("opponentPolicy", get r "opponentPolicy");
                ("map", map);
                ("family", default families (text (get r "unit")) (get r "unit"));
                ("purchased", get r "recruited");
              ])
          [ get r "map"; str "all" ])
  in
  let keys = [ "race"; "policy"; "opponentPolicy"; "map" ] in
  let rows = Frame.group facts (keys @ [ "family" ]) [ "purchased" ] in
  let totals = Frame.group rows keys [ "purchased" ] in
  List.filter_map
    (fun r ->
      let total =
        List.find
          (fun t -> List.for_all (fun k -> get r k = get t k) keys)
          totals
        |> fun t -> get t "purchased"
      in
      if number (get r "purchased") = 0. then None
      else
        Some
          (merge r
             [
               ("totalPurchases", total);
               ( "sharePercent",
                 float (100. *. number (get r "purchased") /. number total) );
             ]))
    rows
