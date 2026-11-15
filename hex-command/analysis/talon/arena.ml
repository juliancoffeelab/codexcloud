open Json
(** Compact outcomes enter typed columns once; aggregates remain Talon tables.
*)

module R = Rel
module E = Talon.Expr
module B = R.Builder

let keys = [ "race"; "opponentRace"; "policy"; "opponentPolicy"; "map" ]

let counts =
  [ "games"; "wins"; "losses"; "caps"; "leaderWins"; "territoryWins"; "rounds" ]

let events = Telemetry.events

let tally outcomes =
  let games = R.Builder.create (Dataset.strings keys @ Dataset.floats counts) in
  let units =
    R.Builder.create
      (Dataset.strings (keys @ [ "unit" ]) @ Dataset.floats events)
  in
  let ledger = ref [] and campaigns = ref [] in
  List.iter
    (fun o ->
      let config = get o "config" in
      let row =
        merge
          (remove config [ "armies" ])
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
      else
        let dim side n =
          match n with
          | "race" -> get (get config "armies") side
          | "opponentRace" -> get (get config "armies") (other side)
          | "policy" -> str (label config side)
          | "opponentPolicy" -> str (label config (other side))
          | "map" -> get config "landscape"
          | _ -> `Null
        in
        ledger :=
          merge row
            ([
               ("playerRace", get (get config "armies") "player");
               ("enemyRace", get (get config "armies") "enemy");
             ]
            @ List.map (fun n -> (n, dim "player" n)) keys)
          :: !ledger;
        List.iter
          (fun side ->
            let winner = get o "winner" in
            B.add games (fun n ->
                if List.mem n keys then dim side n
                else
                  let flag b = int (if b then 1 else 0) in
                  match n with
                  | "games" -> int 1
                  | "wins" -> flag (winner = str side)
                  | "losses" -> flag (winner = str (other side))
                  | "caps" -> flag (winner = `Null)
                  | "leaderWins" ->
                      flag (winner = str side && get o "reason" = str "leader")
                  | "territoryWins" ->
                      flag
                        (winner = str side && get o "reason" = str "territory")
                  | _ -> get o n);
            List.iter
              (fun (source, event) ->
                List.iter
                  (fun u ->
                    if get u "side" = str side then
                      B.add units (fun n ->
                          if List.mem n keys then dim side n
                          else if n = "unit" then get u "type"
                          else if source = "combatUsage" then
                            default u n (int 0)
                          else int (if n = event then 1 else 0)))
                  (list (get o source)))
              [
                ("recruits", "recruited");
                ("promotions", "promoted");
                ("casualties", "lost");
                ("combatUsage", "");
              ])
          [ "player"; "enemy" ])
    outcomes;
  let games =
    B.finish games |> R.group keys counts
    |> R.derive
         E.
           [
             "winRate" := float 100. *. R.f "wins" /. R.f "games";
             "lossRate" := float 100. *. R.f "losses" /. R.f "games";
             "capRate" := float 100. *. R.f "caps" /. R.f "games";
             "meanRounds" := R.f "rounds" /. R.f "games";
           ]
  in
  ( games,
    B.finish units |> R.group (keys @ [ "unit" ]) events,
    List.rev !ledger,
    List.rev !campaigns )

let combine ?(equal = false) ?(cross = true) rows keys =
  rows
  |> (if equal then R.filter E.(R.s "policy" = R.s "opponentPolicy") else Fun.id)
  |> (if cross then R.filter E.(R.s "race" <> R.s "opponentRace") else Fun.id)
  |> R.group keys counts
  |> R.derive [ R.out "winRate" E.(float 100. *. R.f "wins" /. R.f "games") ]

let family_table families =
  let builder = B.create (Dataset.strings [ "unit"; "family" ]) in
  List.iter
    (fun (unit, family) ->
      B.add builder (fun n -> if n = "unit" then str unit else family))
    (fields families);
  B.finish builder

let purchase_shares events families =
  let keys = [ "race"; "policy"; "opponentPolicy"; "map" ] in
  let rows =
    events
    |> R.filter E.(R.s "race" <> R.s "opponentRace")
    |> R.join ~kind:Talon.Join.Left ~one:true [ "unit" ] (family_table families)
    |> R.derive [ R.out "family" E.(coalesce [ R.s "family"; R.s "unit" ]) ]
    |> R.all_maps
    |> R.project
         E.
           [
             keep (Talon.Sel.names (keys @ [ "family" ]));
             "purchased" := R.f "recruited";
           ]
    |> R.group (keys @ [ "family" ]) [ "purchased" ]
  in
  let totals =
    rows
    |> R.aggregate keys [ R.out "totalPurchases" E.(sum (R.f "purchased")) ]
  in
  rows |> R.join keys totals
  |> R.filter E.(R.f "purchased" > float 0.)
  |> R.derive
       [
         R.out "sharePercent"
           E.(float 100. *. R.f "purchased" /. R.f "totalPurchases");
       ]
