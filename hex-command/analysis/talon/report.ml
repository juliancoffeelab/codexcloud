open Json

let names rows =
  List.fold_left
    (fun acc r ->
      List.fold_left
        (fun acc (k, _) -> if List.mem k acc then acc else acc @ [ k ])
        acc (fields r))
    [] rows

let rec mkdir path =
  if path <> "." && not (Sys.file_exists path) then (
    mkdir (Filename.dirname path);
    Unix.mkdir path 0o755)

let display = function
  | "human" -> "Humans"
  | "skeletons" -> "Skeletons"
  | "woodland" -> "Woodland"
  | "coastal" -> "Merfolk"
  | "open" -> "Plains"
  | "wooded" -> "Forests"
  | "crossings" -> "Crossing"
  | "islands" -> "Islands"
  | "wood-edge" -> "Wood Edge"
  | x -> x

let faction_rank = function
  | "human" -> 0
  | "skeletons" -> 1
  | "woodland" -> 2
  | "coastal" -> 3
  | _ -> 4

let map_rank = function
  | "open" -> 0
  | "wooded" -> 1
  | "crossings" -> 2
  | "islands" -> 3
  | "wood-edge" -> 4
  | _ -> 5

let unique key rows =
  List.map (fun r -> text (get r key)) rows |> List.sort_uniq compare

let csv_value = function
  | `Null -> ""
  | `String x -> x
  | `Bool true -> "True"
  | `Bool false -> "False"
  | x -> encode x

let quote s =
  if List.exists (String.contains s) [ ','; '"'; '\n'; '\r' ] then
    "\"" ^ String.concat "\"\"" (String.split_on_char '"' s) ^ "\""
  else s

let csv path rows =
  let headers = names rows in
  let ch = open_out_bin path in
  Fun.protect
    ~finally:(fun () -> close_out ch)
    (fun () ->
      let line xs =
        output_string ch (String.concat "," (List.map quote xs));
        output_string ch "\r\n"
      in
      line headers;
      List.iter
        (fun r -> line (List.map (fun k -> csv_value (get r k)) headers))
        rows)

let cell k v =
  let s =
    match v with
    | `Null -> if k = "recruitLimit" then "policy default" else "—"
    | `Float n -> Printf.sprintf "%.2f" n
    | `String s ->
        if List.mem k [ "race"; "opponentRace"; "Faction"; "map" ] then
          display s
        else s
    | _ -> csv_value v
  in
  String.concat "\\|" (String.split_on_char '|' s)
  |> String.split_on_char '\n' |> String.concat " "

let markdown rows headers =
  String.concat "\n"
    ([
       "| " ^ String.concat " | " headers ^ " |";
       "| " ^ String.concat " | " (List.map (fun _ -> "---") headers) ^ " |";
     ]
    @ List.map
        (fun r ->
          "| "
          ^ String.concat " | " (List.map (fun k -> cell k (get r k)) headers)
          ^ " |")
        rows)

let findings tables physical =
  let get_table = Analysis.table tables in
  let rows = get_table "matchups" in
  let factions =
    unique "race" rows
    |> List.sort (fun a b -> compare (faction_rank a, a) (faction_rank b, b))
  in
  let maps =
    unique "map" rows
    |> List.sort (fun a b -> compare (map_rank a, a) (map_rank b, b))
  in
  let pairs =
    List.map
      (fun r -> (text (get r "policy"), text (get r "opponentPolicy")))
      rows
    |> List.sort_uniq compare
  in
  let lines =
    ref
      [
        "# Combat findings";
        "";
        Printf.sprintf
          "Coverage: %d physical matches. Caps remain in all win-rate \
           denominators. Participant views are not additional games."
          (List.length physical);
        "";
        "## Arena faction matchups";
        "";
      ]
  in
  List.iter
    (fun (p, q) ->
      List.iter
        (fun map ->
          let selected =
            List.filter
              (fun r ->
                get r "map" = str map
                && get r "policy" = str p
                && get r "opponentPolicy" = str q)
              rows
          in
          if selected <> [] then
            let matrix =
              List.map
                (fun race ->
                  obj
                    (("Faction", str race)
                    :: List.map
                         (fun enemy ->
                           let item =
                             List.find_opt
                               (fun r ->
                                 get r "race" = str race
                                 && get r "opponentRace" = str enemy)
                               selected
                           in
                           ( enemy,
                             str
                               (if race = enemy then "Mirror"
                                else
                                  match item with
                                  | None -> "—"
                                  | Some r ->
                                      Printf.sprintf "%.1f%% (%d)"
                                        (number (get r "winRate"))
                                        (integer (get r "caps"))) ))
                         factions))
                factions
            in
            lines :=
              !lines
              @ [
                  "### " ^ display map ^ ": " ^ p ^ " against " ^ q;
                  "";
                  markdown matrix ("Faction" :: factions);
                  "";
                ])
        maps)
    pairs;
  let mirrors =
    physical
    |> List.filter (fun o ->
        let c = get o "config" in
        get o "kind" = str "arena"
        && get (get c "armies") "player" = get (get c "armies") "enemy"
        && label c "player" = label c "enemy")
    |> List.map (fun o ->
        let c = get o "config" in
        obj
          [
            ("map", get c "landscape");
            ("race", get (get c "armies") "player");
            ("policy", str (label c "player"));
            ("games", int 1);
            ( "firstWins",
              int
                (if get o "winner" <> `Null && get o "winner" = get c "first"
                 then 1
                 else 0) );
            ("caps", int (if get o "winner" = `Null then 1 else 0));
          ])
  in
  let mirrors =
    Frame.group mirrors
      [ "map"; "race"; "policy" ]
      [ "games"; "firstWins"; "caps" ]
    |> List.map (fun r ->
        set r "firstWinRate"
          (float (100. *. number (get r "firstWins") /. number (get r "games"))))
  in
  lines :=
    !lines
    @ [
        "## Matching-policy mirrors: first to act";
        "";
        "Each physical game is counted once, including caps.";
        "";
        markdown mirrors
          [ "map"; "race"; "policy"; "games"; "firstWinRate"; "caps" ];
        "";
        "## Campaign completion";
        "";
        markdown
          (get_table "campaign_summary")
          [
            "campaign";
            "player";
            "enemy";
            "diplomacy";
            "objective";
            "maxRounds";
            "recruitLimit";
            "attempts";
            "completed";
            "defeats";
            "caps";
            "unclassified";
          ];
        "";
        "## Campaign stages";
        "";
        "Stage denominators include only attempts reaching that stage. Failure \
         observations and forecast exposure do not establish causes.";
        "";
        markdown
          (get_table "campaign_stages")
          [
            "campaign";
            "objective";
            "diplomacy";
            "player";
            "enemy";
            "stage";
            "map";
            "reached";
            "victories";
            "defeats";
            "caps";
            "idleCaps";
            "repeatedCaps";
            "zeroProgressDefeats";
            "actorAttempts";
            "actorArrivals";
            "actorDefeatsBeforeArrival";
            "defeatsWithSurvivors";
            "playerDamage";
            "enemyDamage";
          ];
        "";
        "## Recorded terminal reasons";
        "";
        markdown
          (get_table "campaign_failures")
          [
            "campaign";
            "objective";
            "diplomacy";
            "player";
            "enemy";
            "map";
            "category";
            "reason";
            "attempts";
            "exampleSeeds";
          ];
        "";
      ];
  if get_table "unit_damage" <> [] then
    lines :=
      !lines
      @ [
          "## Per-unit damage";
          "";
          "Presence is living nonleader unit-turn exposure. Weapon HP damage \
           includes retaliation and excludes overkill and thorns. Thorns are \
           attributed separately. D/P is damage share divided by presence \
           share; undefined rates remain unknown.";
          "";
          markdown
            (get_table
               (if get_table "unit_damage_changes" = [] then "unit_damage"
                else "unit_damage_changes"))
            [
              "race";
              "policy";
              "opponentPolicy";
              "map";
              "name";
              "unitKind";
              "unitTurns";
              "presenceSharePercent";
              "damagePerUnitTurn";
              "damagePresenceRatio";
              "thornDamagePerUnitTurn";
            ];
          "";
        ];
  if get_table "battlefield_presence" = [] then
    lines :=
      !lines
      @ [
          "Field composition is unavailable. Purchases measure recruitment \
           flow and do not establish composition or survival.";
          "";
        ];
  String.concat "\n" !lines

let write output current previous tables seconds =
  mkdir output;
  List.iter
    (fun (name, rows) -> csv (Filename.concat output (name ^ ".csv")) rows)
    tables;
  let rows = Analysis.table tables "matchups"
  and campaigns = Analysis.table tables "campaigns" in
  let factions =
    unique "race" rows
    |> List.sort (fun a b -> compare (faction_rank a, a) (faction_rank b, b))
  in
  let maps =
    List.sort_uniq compare
      (unique "map" rows
      @ unique "map" (Analysis.table tables "campaign_stages"))
  in
  let policies =
    List.sort_uniq compare
      (unique "policy" rows
      @ unique "player" (Analysis.table tables "campaign_summary"))
  in
  let metadata =
    obj
      [
        ("sources", arr current.Store.sources);
        ("baselineSources", arr previous.Store.sources);
        ("physicalMatches", int (List.length current.outcomes));
        ( "arenaMatches",
          int
            (List.length
               (List.filter
                  (fun o -> get o "kind" = str "arena")
                  current.outcomes)) );
        ("campaignAttempts", int (List.length campaigns));
        ("factions", arr (List.map str factions));
        ("maps", arr (List.map str maps));
        ("policies", arr (List.map str policies));
        ( "participantRows",
          int (List.fold_left (fun n r -> n + integer (get r "games")) 0 rows)
        );
      ]
  in
  Json.write (Filename.concat output "metadata.json") metadata;
  Json.write
    (Filename.concat output "tables.json")
    (obj (List.map (fun (k, v) -> (k, arr v)) tables));
  let findings = findings tables current.outcomes in
  let textfile name content =
    let ch = open_out (Filename.concat output name) in
    output_string ch content;
    close_out ch
  in
  textfile "findings.md" findings;
  textfile "summary.md"
    (findings ^ "\n# Complete combat data\n\n"
    ^ String.concat "\n\n"
        (List.map
           (fun (name, rows) ->
             "## " ^ name ^ "\n\n" ^ markdown rows (names rows))
           (List.filter
              (fun (name, _) ->
                List.mem name
                  [
                    "matchups";
                    "unit_events";
                    "purchase_shares";
                    "race_equal_policy";
                    "race_map_policy";
                  ])
              tables))
    ^ "\n\n## Complete exports\n\n"
    ^ String.concat "\n"
        (List.map
           (fun (name, _) -> "- [" ^ name ^ ".csv](" ^ name ^ ".csv)")
           tables)
    ^ "\n");
  Json.write
    (Filename.concat output "timings.json")
    (obj
       [
         ("reportSeconds", float seconds);
         ("physicalMatches", int (List.length current.outcomes));
       ])
