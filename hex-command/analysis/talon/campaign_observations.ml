open Json
(** Campaign reductions share the same decoded tables as telemetry. *)

module R = Rel
module E = Talon.Expr

let keys = [ "m"; "ch" ]

let index rows =
  let h = Hashtbl.create 128 in
  List.iter
    (fun r -> Hashtbl.replace h (integer (get r "m"), integer (get r "ch")) r)
    rows;
  h

let row h key = Option.value ~default:(obj []) (Hashtbl.find_opt h key)
let number_at h key name = get (row h key) name

let build (d : Dataset.t) =
  let result = Hashtbl.create 64 in
  let campaign_ids =
    d.meta |> R.filter E.(R.s "kind" = string "campaign") |> R.select [ "m" ]
  in
  let campaign table =
    R.join ~kind:Talon.Join.Semi [ "m" ] campaign_ids table
  in
  let phases = campaign d.phases
  and units = campaign d.units
  and before = campaign d.before
  and casualties = campaign d.losses
  and final_units = campaign d.final
  and goal_rows = campaign d.goals in
  let own = phases |> R.filter E.(R.s "side" = string "player") in
  let own_keys = R.select [ "p"; "m"; "ch" ] own in
  let losses = casualties |> R.filter E.(R.s "side" = string "player") in
  let final =
    final_units
    |> R.aggregate keys
         [
           R.out "player" E.(sum (R.flag (R.s "side" = string "player")));
           R.out "enemy" E.(sum (R.flag (R.s "side" = string "enemy")));
         ]
    |> Frame.rows |> index
  in
  let counts = phases |> R.count keys "n" |> Frame.rows |> index in
  let last = own |> R.unique_last keys |> Frame.rows |> index in
  let empty_tail =
    own |> R.Q.of_table
    |> R.Q.sort
         [ Talon.Order.asc "m"; Talon.Order.asc "ch"; Talon.Order.desc "p" ]
    |> R.Q.derive
         [
           R.out "tail"
             E.(
               over ~by:keys
                 (window (min (cast Talon.Type.int64 (R.b "emptyActions")))));
         ]
    |> R.Q.aggregate ~by:keys [ R.out "idle" E.(sum (R.i "tail")) ]
    |> R.run |> Frame.rows |> index
  in
  let roster =
    units
    |> R.aggregate [ "p" ]
         [
           R.out "roster"
             E.(
               collect
                 (record
                    [ keep (Talon.Sel.names [ "id"; "type"; "cell"; "hp" ]) ]));
         ]
  in
  let signatures =
    own
    |> R.derive
         [
           R.out "reverseRank"
             E.(over ~by:keys ~order:[ Talon.Order.desc "p" ] (window rows));
         ]
    |> R.filter E.(R.i "reverseRank" <= int 10)
    |> R.join ~kind:Talon.Join.Left [ "p" ] roster
    |> R.count
         (keys @ [ "roster"; "goalsSignature"; "ownerSignature" ])
         "repeated"
    |> R.aggregate keys [ R.out "repeated" E.(max (R.f "repeated")) ]
    |> Frame.rows |> index
  in
  let diag_lookup =
    R.concat [ before; units; casualties ] |> R.unique_last [ "p"; "id" ]
  in
  let combats =
    d.combats
    |> R.join ~kind:Talon.Join.Left [ "p"; "unit" ]
         (diag_lookup
         |> R.select [ "p"; "id"; "side" ]
         |> R.rename [ ("id", "unit"); ("side", "actorSide") ])
    |> R.join [ "p" ] (R.select [ "p"; "m"; "ch"; "side" ] phases)
  in
  let damage =
    combats
    |> R.aggregate keys
         E.
           [
             "player" :=
               sum
                 (if_
                    (coalesce [ R.s "actorSide"; R.s "side" ] = string "player")
                    (R.f "damage") (R.f "retaliation"));
             "enemy" :=
               sum
                 (if_
                    (coalesce [ R.s "actorSide"; R.s "side" ] = string "player")
                    (R.f "retaliation") (R.f "damage"));
           ]
    |> Frame.rows |> index
  in
  let maxima =
    goal_rows
    |> R.aggregate (keys @ [ "goal" ])
         [ R.out "current" E.(max (R.f "current")) ]
    |> Frame.rows
  in
  let frequency field_name table =
    let h = Hashtbl.create 128 in
    table
    |> R.filter E.(not (is_null (R.s field_name)))
    |> R.count (keys @ [ field_name ]) "n"
    |> R.sort (keys @ [ field_name ])
    |> Frame.rows
    |> List.iter (fun r ->
        let key = (integer (get r "m"), integer (get r "ch")) in
        Hashtbl.replace h key
          (set (row h key) (text (get r field_name)) (get r "n")));
    h
  in
  let named = frequency "name" losses
  and casualties = frequency "type" losses in
  let recruits =
    d.actions
    |> R.filter E.(R.s "kind" = string "recruit")
    |> R.join [ "p" ] own_keys |> frequency "type"
  in
  let planning = own |> R.filter (R.b "hasPlanning") in
  let duties =
    d.duties |> R.join ~kind:Talon.Join.Semi [ "p" ] (R.select [ "p" ] planning)
  in
  let unmet =
    duties |> R.filter E.(R.s "status" = string "Unmet") |> frequency "reason"
  in
  let deferred =
    duties
    |> R.filter E.(R.s "status" = string "Deferred")
    |> frequency "reason"
  in
  let rescue =
    planning
    |> R.aggregate keys
         E.[ "candidates" := sum (R.f "candidates"); "recorded" := rows ]
    |> Frame.rows |> index
  in
  let protected =
    combats
    |> R.filter E.(R.s "side" = string "player" && R.f "retaliation" > float 0.)
    |> R.join ~kind:Talon.Join.Semi [ "p"; "unit" ]
         (R.unique [ "p"; "unit" ] d.constraints)
    |> R.count keys "n" |> Frame.rows |> index
  in
  let actor_tiles =
    R.Builder.create
      (Dataset.ints [ "m"; "ch" ] @ Dataset.strings [ "name"; "cell" ])
  in
  List.iter
    (fun (h : Dataset.header) ->
      if get h.result "kind" = str "campaign" then
        List.iteri
          (fun ch chapter ->
            let goals = list (get chapter "goals") in
            List.iter
              (fun u ->
                if truth (get u "name") && get u "side" = str "player" then
                  List.iter
                    (fun g ->
                      if contains (text (get g "label")) (text (get u "name"))
                      then
                        List.iter
                          (fun t ->
                            R.Builder.add actor_tiles (fun n ->
                                match n with
                                | "m" -> int h.m
                                | "ch" -> int ch
                                | "name" -> get u "name"
                                | _ -> str (cell t)))
                          (list (get g "tiles")))
                    goals)
              (list (get (get chapter "initial") "units")))
          (list (get h.result "missions")))
    d.headers;
  let arrivals =
    units
    |> R.select [ "m"; "ch"; "name"; "cell" ]
    |> R.join [ "m"; "ch"; "name"; "cell" ] (R.Builder.finish actor_tiles)
    |> R.unique [ "m"; "ch"; "name" ]
    |> Frame.rows
  in
  let loss_flags =
    losses
    |> R.aggregate keys
         [
           R.out "leader"
             E.(max (cast Talon.Type.int64 (R.s "role" = string "leader")));
           R.out "courier"
             E.(max (cast Talon.Type.int64 (R.s "role" = string "courier")));
         ]
    |> Frame.rows |> index
  in
  let last_ids = R.Builder.create (Dataset.ints [ "p" ]) in
  Hashtbl.iter (fun _ r -> R.Builder.add last_ids (fun _ -> get r "p")) last;
  let final_rosters =
    units
    |> R.join ~kind:Talon.Join.Semi [ "p" ] (R.Builder.finish last_ids)
    |> Frame.rows
  in
  let by_phase = Hashtbl.create 128 in
  List.iter
    (fun u ->
      let p = integer (get u "p") in
      Hashtbl.replace by_phase p
        (u :: Option.value ~default:[] (Hashtbl.find_opt by_phase p)))
    final_rosters;
  let unit_presentation u =
    let tile = obj [ ("col", get u "col"); ("row", get u "row") ] in
    merge u [ ("tile", tile) ]
  in
  List.iter
    (fun (h : Dataset.header) ->
      if get h.result "kind" = str "campaign" then
        let observations =
          List.mapi
            (fun ch chapter ->
              let key = (h.m, ch)
              and winner = get chapter "winner"
              and state = get chapter "final" in
              let reason = get (get state "campaign") "outcome" in
              let reason =
                if winner = `Null && not (truth reason) then
                  str "Round limit reached with unfinished objectives"
                else reason
              in
              let goals = list (get chapter "goals") in
              let actors =
                list (get (get chapter "initial") "units")
                |> List.filter (fun u ->
                    truth (get u "name")
                    && get u "side" = str "player"
                    && List.exists
                         (fun g ->
                           contains (text (get g "label")) (text (get u "name")))
                         goals)
              in
              let arrived name =
                List.exists
                  (fun r ->
                    get r "m" = int h.m
                    && get r "ch" = int ch
                    && get r "name" = name)
                  arrivals
              in
              let category =
                if winner = str "player" then "victory"
                else if winner = `Null then "round cap"
                else if
                  has state "units"
                  && not (truth (number_at final key "player"))
                then "all player units lost"
                else if truth (number_at loss_flags key "leader") then
                  "leader lost"
                else if
                  truth (number_at loss_flags key "courier")
                  || List.exists
                       (fun u -> has (row named key) (text (get u "name")))
                       actors
                  || List.exists
                       (contains (String.lowercase_ascii (text reason)))
                       [ "courier"; "companion" ]
                then "required actor lost"
                else if truth reason then "defeat; see recorded reason"
                else "defeat; cause not recorded"
              in
              let stalled =
                if
                  winner <> `Null
                  || not (truth (number_at last key "hasPlanning"))
                then []
                else
                  let last = row last key in
                  let plan = parse (text (get last "planning")) in
                  let roster =
                    Option.value ~default:[]
                      (Hashtbl.find_opt by_phase (integer (get last "p")))
                    |> List.rev |> List.map unit_presentation
                  in
                  list (get (get (get plan "orders") "tactics") "objectives")
                  |> List.filter_map (fun order ->
                      let objective = get order "objective" in
                      let body =
                        if truth (get objective "GoTo") then
                          get objective "GoTo"
                        else get objective "Occupy"
                      in
                      if not (truth body) then None
                      else
                        let id =
                          "u" ^ string_of_int (integer (get body "actor"))
                        in
                        let actor =
                          Option.value ~default:(obj [])
                            (List.find_opt
                               (fun u -> get u "id" = str id)
                               roster)
                        in
                        let duty =
                          List.find_opt
                            (fun duty -> get duty "id" = get order "id")
                            (list (get (get plan "report") "duties"))
                        in
                        Some
                          (obj
                             [
                               ( "actor",
                                 if truth (get actor "name") then
                                   get actor "name"
                                 else default actor "type" (str id) );
                               ("tile", get actor "tile");
                               ("hp", get actor "hp");
                               ( "target",
                                 default body "destination"
                                   (get body "location") );
                               ( "reason",
                                 match duty with
                                 | Some duty -> get duty "reason"
                                 | None -> `Null );
                               ( "allies",
                                 arr
                                   (List.filter_map
                                      (fun u ->
                                        if get u "side" = str "player" then
                                          Some
                                            (obj
                                               (List.map
                                                  (fun k -> (k, get u k))
                                                  [
                                                    "id";
                                                    "type";
                                                    "tile";
                                                    "rebuilding";
                                                  ]))
                                        else None)
                                      roster) );
                             ]))
              in
              let zero h n = default (row h key) n (int 0) in
              obj
                [
                  ("id", get chapter "id");
                  ("round", get chapter "round");
                  ("winner", winner);
                  ("category", str category);
                  ("reason", reason);
                  ("idleTail", zero empty_tail "idle");
                  ("repeatedTail", zero signatures "repeated");
                  ("finalPlayerUnits", zero final "player");
                  ("finalEnemyUnits", zero final "enemy");
                  ( "maxGoalProgress",
                    arr
                      (List.mapi
                         (fun goal g ->
                           match
                             List.find_opt
                               (fun r ->
                                 get r "m" = int h.m
                                 && get r "ch" = int ch
                                 && get r "goal" = int goal)
                               maxima
                           with
                           | Some r -> get r "current"
                           | None -> default g "current" (int 0))
                         goals) );
                  ("goals", arr goals);
                  ("namedLosses", row named key);
                  ("recruits", row recruits key);
                  ("casualties", row casualties key);
                  ("playerDamage", zero damage "player");
                  ("enemyDamage", zero damage "enemy");
                  ( "namedGoalActors",
                    arr (List.map (fun u -> get u "name") actors) );
                  ( "actorArrivals",
                    obj
                      (List.map
                         (fun u ->
                           (text (get u "name"), bool (arrived (get u "name"))))
                         actors) );
                  ("stalledActors", arr stalled);
                  ("recordedPhases", bool (truth (zero counts "n")));
                  ("unmetOrders", row unmet key);
                  ("deferredOrders", row deferred key);
                  ("rescueCandidates", zero rescue "candidates");
                  ("protectedRetaliationExchanges", zero protected "n");
                  ("recordedPlanning", bool (truth (zero rescue "recorded")));
                ])
            (list (get h.result "missions"))
        in
        Hashtbl.add result (identity h.saved) observations)
    d.headers;
  result
