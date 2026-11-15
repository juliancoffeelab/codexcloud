open Json
(** Recorded state/event interpretation. Bounded batches are reduced by Talon.
*)

let dims = [ "race"; "policy"; "opponentPolicy"; "map" ]

let value_dims =
  [
    "kind";
    "campaign";
    "diplomacy";
    "objective";
    "mission";
    "race";
    "policy";
    "opponentPolicy";
    "map";
    "family";
  ]

let value_fields =
  [
    "unitTurns";
    "hpDamageDealt";
    "hpDamageTaken";
    "weaponHPDamageDealt";
    "weaponHPDamageTaken";
    "thornHPDamageDealt";
    "thornHPDamageTaken";
    "armourDamagePrevented";
    "terrainDamagePrevented";
    "rawArmourDelta";
    "rawTerrainDelta";
    "attacksTaken";
    "retaliationsTaken";
    "captures";
    "objectiveCaptures";
    "ownedTileTurns";
    "objectiveControlTurns";
    "capturableOccupationTurns";
    "objectiveOccupationTurns";
    "casualties";
    "recruitGoldLost";
    "casualtyReplacementValue";
  ]

let nonleader_fields =
  [
    "nonleaderUnitTurns";
    "nonleaderWeaponHPDamageDealt";
    "nonleaderThornHPDamageDealt";
  ]

let effect_fields =
  [
    "providerTurns";
    "adjacentContacts";
    "coveredRecipients";
    "overlappingRecipients";
    "healedHP";
    "healedRecipients";
    "triggeredCombats";
    "reflectedDamage";
    "effectiveReflectedDamage";
  ]

let capturable x = List.mem (text x) [ "village"; "grove"; "fort"; "ruins" ]

let analyze store ~presence ~support ~value ~leaders =
  let role_events = Frame.accumulator (dims @ [ "unit" ]) Arena.event_counts in
  let census = Frame.accumulator (dims @ [ "unit" ]) [ "unitTurns" ] in
  let totals = Frame.accumulator dims [ "battles"; "phases"; "unitTurns" ] in
  let present =
    Frame.accumulator (dims @ [ "grouping"; "member" ]) [ "battlesPresent" ]
  in
  let effects =
    Frame.accumulator (dims @ [ "unit"; "ability" ]) effect_fields
  in
  let overlaps =
    Frame.accumulator
      (dims @ [ "ability"; "providerCount"; "recipientsPerProvider" ])
      [
        "opportunities";
        "eligiblePairs";
        "contacts";
        "coveredRecipients";
        "overlappingRecipients";
      ]
  in
  let values = Frame.accumulator value_dims value_fields
  and unit_values =
    Frame.accumulator
      (List.filter (( <> ) "family") value_dims @ [ "unit" ])
      (value_fields @ nonleader_fields)
  in
  let leader_rows = ref []
  and catalogs = ref `Null
  and family_catalog = ref `Null in
  let add_zero cols r =
    merge r (List.map (fun k -> (k, default r k (int 0))) cols)
  in
  let isleader u = get u "role" = str "leader" in
  let family unit =
    default (get !catalogs unit) "family"
      (default !family_catalog unit (str unit))
  in
  let matches = ref 0 in
  Store.journals store (fun report result ->
      let kind = text (get result "kind") and config = get result "config" in
      let arena = kind = "arena" in
      let mirror =
        arena
        && get (get config "armies") "player"
           = get (get config "armies") "enemy"
      in
      if not mirror then (
        let profiles = default report "unitProfiles" (obj []) in
        if truth !catalogs then
          Journal.require
            (sorted !catalogs = sorted profiles)
            "Cannot pool different recorded unit catalogs";
        catalogs := profiles;
        family_catalog := default report "unitFamilies" (obj []);
        if arena && support then
          Journal.require
            (get report "supportTelemetryVersion" = int 1 && truth profiles)
            "Support effects require current telemetry and unitProfiles";
        if value then
          Journal.require
            (get report "combatTelemetryVersion" = int 1)
            "Battlefield value requires combat telemetry";
        if leaders then
          Journal.require
            (get report "leaderAuditVersion" = int 1)
            "Leader audit requires leader-risk telemetry";
        let group side map =
          obj
            [
              ("race", get (get config "armies") side);
              ("policy", str (label config side));
              ("opponentPolicy", str (label config (other side)));
              ("map", map);
            ]
        in
        let landscape = get config "landscape" in
        let seen = Hashtbl.create 32 in
        let record_seen u =
          if not (isleader u) then
            Hashtbl.replace seen (text (get u "side"), text (get u "type")) ()
        in
        if arena && (presence || support) then (
          incr matches;
          List.iter record_seen (list (get (get result "initial") "units"));
          List.iter
            (fun side ->
              List.iter
                (fun map ->
                  Frame.add totals (set (group side map) "battles" (int 1)))
                [ landscape; str "all" ])
            [ "player"; "enemy" ]);
        let add_effect unit ability stats =
          List.iter
            (fun map ->
              List.iter
                (fun name ->
                  Frame.add effects
                    (merge
                       (group (text (get unit "side")) map)
                       ([ ("unit", name); ("ability", str ability) ] @ stats)))
                [ get unit "type"; str "all" ])
            [ landscape; str "all" ]
        in
        let chapters =
          if arena then [ result ] else list (get result "missions")
        in
        List.iter
          (fun chapter ->
            let mission = if arena then str "" else get chapter "id" in
            let phases = list (get chapter "phases")
            and initial = get chapter "initial" in
            let tiles = Hashtbl.create 128 in
            List.iter
              (fun t -> Hashtbl.replace tiles (text (get t "id")) t)
              (list (get initial "tiles"));
            let positions = Hashtbl.create 128 in
            List.iter
              (fun t ->
                Hashtbl.replace positions
                  (text (get t "id"))
                  (integer (get t "col"), integer (get t "row")))
              (list (get initial "tiles"));
            let adjacent a b =
              let x, y = Hashtbl.find positions (cell a)
              and u, v = Hashtbl.find positions (cell b) in
              let q = x - (y / 2) - u + (v / 2) and r = y - v in
              abs q + abs r + abs (q + r) = 2
            in
            let factions = index_by "id" (list (get initial "factions")) in
            let owners = Hashtbl.create 128 in
            Hashtbl.iter
              (fun id t -> Hashtbl.replace owners id (get t "owner"))
              tiles;
            let faction u =
              if truth (get u "faction") then get u "faction" else get u "side"
            in
            let hostile a b =
              if a = b then false
              else
                let relation =
                  get (get (lookup factions (text a)) "relations") (text b)
                in
                let relation =
                  if relation = `Null then
                    get (get (lookup factions (text b)) "relations") (text a)
                  else relation
                in
                if relation <> `Null then relation = str "hostile"
                else List.sort compare [ a; b ] = [ str "enemy"; str "player" ]
            in
            let cost u =
              let profile = get profiles (text (get u "type")) in
              let recruits =
                list (get (lookup factions (text (faction u))) "recruits")
              in
              match
                List.find_opt (fun r -> get r "unit" = get u "type") recruits
              with
              | Some r -> default r "cost" (get profile "cost")
              | None -> get profile "cost"
            in
            let captors = Hashtbl.create 128
            and investments = Hashtbl.create 128
            and objectives = Hashtbl.create 128 in
            List.iter
              (fun phase ->
                List.iter
                  (fun o ->
                    if get o "kind" = str "capture" then
                      List.iter
                        (fun tile ->
                          Hashtbl.replace objectives
                            (text (get phase "side"), cell tile)
                            ())
                        (list (get o "tiles")))
                  (list (get phase "objectives")))
              phases;
            let objective side tile = Hashtbl.mem objectives (side, tile) in
            let identity u map =
              let side = text (get u "side") in
              obj
                [
                  ("kind", str kind);
                  ("campaign", default config "campaign" (str ""));
                  ("diplomacy", default config "diplomacy" (str ""));
                  ("objective", default config "objective" (str "authored"));
                  ("mission", mission);
                  ( "race",
                    if arena then get (get config "armies") side
                    else get (get profiles (text (get u "type"))) "race" );
                  ("policy", str (label config side));
                  ("opponentPolicy", str (label config (other side)));
                  ("map", map);
                  ("family", family (text (get u "type")));
                ]
            in
            let add_value u stats =
              let map = if arena then landscape else mission in
              List.iter
                (fun map ->
                  let d = identity u map in
                  Frame.add values (merge d stats);
                  let nonleader = not (isleader u) in
                  let pairs =
                    List.map2
                      (fun source target ->
                        ( target,
                          if nonleader then default (obj stats) source (int 0)
                          else int 0 ))
                      [
                        "unitTurns"; "weaponHPDamageDealt"; "thornHPDamageDealt";
                      ]
                      nonleader_fields
                  in
                  Frame.add unit_values
                    (merge (remove d [ "family" ])
                       ([ ("unit", get u "type") ] @ stats @ pairs)))
                [ map; str "all" ]
            in
            List.iteri
              (fun phase_index phase ->
                let side = text (get phase "side") in
                let before = list (get phase "before")
                and units = list (get phase "units")
                and losses = list (get phase "losses") in
                let lookup_ = index_by "id" (before @ losses @ units) in
                let lookup_support = index_by "id" (losses @ units @ before) in
                if arena && presence then (
                  let own =
                    List.filter
                      (fun u -> get u "side" = str side && not (isleader u))
                      units
                  in
                  List.iter
                    (fun map ->
                      Frame.add totals
                        (merge (group side map)
                           [
                             ("phases", int 1);
                             ("unitTurns", int (List.length own));
                           ]);
                      List.iter
                        (fun u ->
                          Frame.add census
                            (merge (group side map)
                               [ ("unit", get u "type"); ("unitTurns", int 1) ]))
                        own)
                    [ landscape; str "all" ];
                  List.iter record_seen (before @ units @ losses));
                if arena && presence then (
                  let add u stats =
                    if not (isleader u) then
                      Frame.add role_events
                        (merge
                           (group (text (get u "side")) landscape)
                           ([ ("unit", get u "type") ] @ stats))
                  in
                  List.iter (fun u -> add u [ ("lost", int 1) ]) losses;
                  List.iter
                    (fun action ->
                      if get action "kind" = str "advance" then
                        add
                          (lookup lookup_ (text (get action "unit")))
                          [ ("promoted", int 1) ]
                      else if get action "kind" = str "recruit" then (
                        let previous = List.map (fun u -> get u "id") before in
                        let spawned =
                          List.filter
                            (fun u ->
                              (not (List.mem (get u "id") previous))
                              && get u "type" = get action "type"
                              && get u "tile" = get action "tile"
                              && get u "side" = str side)
                            units
                        in
                        Journal.require
                          (List.length spawned = 1)
                          "Cannot identify recorded recruit role";
                        add (List.hd spawned) [ ("recruited", int 1) ]))
                    (list (get phase "actions"));
                  List.iter
                    (fun c ->
                      add
                        (lookup lookup_ (text (get c "unit")))
                        [
                          ("attacks", int 1);
                          ("damage", default c "damage" (int 0));
                        ];
                      if
                        integer (get c "retaliationCount") > 0
                        || number (get c "retaliation") <> 0.
                      then
                        add
                          (lookup lookup_ (text (get c "target")))
                          [
                            ( "retaliations",
                              int
                                (if integer (get c "retaliationCount") > 0 then
                                   1
                                 else 0) );
                            ("damage", default c "retaliation" (int 0));
                          ])
                    (list (get phase "combats")));
                if arena && support then (
                  List.iter
                    (fun ability ->
                      let provider_side =
                        if ability = "healing" then side else other side
                      in
                      let friends =
                        List.filter
                          (fun u -> get u "side" = str provider_side)
                          before
                      in
                      let providers =
                        List.filter
                          (fun u ->
                            number
                              (get (get profiles (text (get u "type"))) ability)
                            > 0.)
                          friends
                      in
                      List.iter
                        (fun p ->
                          add_effect p ability [ ("providerTurns", int 1) ])
                        providers;
                      if ability <> "thorns" then (
                        let contacts = ref 0
                        and covered = ref 0
                        and overlapping = ref 0 in
                        List.iter
                          (fun ally ->
                            let touching =
                              List.filter
                                (fun p ->
                                  get p "id" <> get ally "id"
                                  && adjacent (get p "tile") (get ally "tile"))
                                providers
                            in
                            let n = List.length touching in
                            contacts := !contacts + n;
                            if n > 0 then incr covered;
                            if n > 1 then incr overlapping;
                            List.iter
                              (fun p ->
                                add_effect p ability
                                  [
                                    ("adjacentContacts", int 1);
                                    ( "coveredRecipients",
                                      float (1. /. Float.of_int n) );
                                    ( "overlappingRecipients",
                                      float
                                        (if n > 1 then 1. /. Float.of_int n
                                         else 0.) );
                                  ])
                              touching)
                          friends;
                        if providers <> [] then
                          List.iter
                            (fun map ->
                              Frame.add overlaps
                                (merge (group provider_side map)
                                   [
                                     ("ability", str ability);
                                     ( "providerCount",
                                       int (List.length providers) );
                                     ( "recipientsPerProvider",
                                       int (max 0 (List.length friends - 1)) );
                                     ("opportunities", int 1);
                                     ( "eligiblePairs",
                                       int
                                         (List.length providers
                                         * max 0 (List.length friends - 1)) );
                                     ("contacts", int !contacts);
                                     ("coveredRecipients", int !covered);
                                     ("overlappingRecipients", int !overlapping);
                                   ]))
                            [ landscape; str "all" ]))
                    [ "healing"; "thornAura"; "thorns" ];
                  List.iter
                    (fun event ->
                      let providers = list (get event "providers") in
                      Journal.require (providers <> [])
                        "Healing event has no provider";
                      List.iter
                        (fun p ->
                          add_effect
                            (lookup lookup_support (text p))
                            "healing"
                            [
                              ( "healedHP",
                                float
                                  (number (get event "amount")
                                  /. Float.of_int (List.length providers)) );
                              ( "healedRecipients",
                                float
                                  (1. /. Float.of_int (List.length providers))
                              );
                            ])
                        providers)
                    (list (get phase "healing"));
                  List.iter
                    (fun combat ->
                      let thorns = get combat "thorns" in
                      Journal.require (thorns <> `Null)
                        "Support telemetry missing";
                      let raw = number (get thorns "damage") in
                      if raw > 0. then (
                        let own = raw -. number (get thorns "auraDamage")
                        and effective = number (get thorns "effectiveDamage") in
                        if own <> 0. then
                          add_effect
                            (lookup lookup_support (text (get combat "target")))
                            "thorns"
                            [
                              ("triggeredCombats", int 1);
                              ("reflectedDamage", float own);
                              ( "effectiveReflectedDamage",
                                float (effective *. own /. raw) );
                            ];
                        let sources = list (get thorns "auraSources") in
                        Journal.require
                          (number (get thorns "auraDamage") = 0.
                          || sources <> [])
                          "Thorn aura has no provider";
                        List.iter
                          (fun source ->
                            let damage =
                              number (get thorns "auraDamage")
                              /. Float.of_int (List.length sources)
                            in
                            add_effect
                              (lookup lookup_support (text source))
                              "thornAura"
                              [
                                ( "triggeredCombats",
                                  float
                                    (1. /. Float.of_int (List.length sources))
                                );
                                ("reflectedDamage", float damage);
                                ( "effectiveReflectedDamage",
                                  float (effective *. damage /. raw) );
                              ])
                          sources))
                    (list (get phase "combats")));
                if value then (
                  List.iter
                    (fun u ->
                      if get u "side" = str side then (
                        add_value u [ ("unitTurns", int 1) ];
                        let tile = cell (get u "tile") in
                        if capturable (get (lookup tiles tile) "terrain") then
                          add_value u [ ("capturableOccupationTurns", int 1) ];
                        if objective side tile then
                          add_value u [ ("objectiveOccupationTurns", int 1) ]))
                    units;
                  List.iter
                    (fun action ->
                      if get action "kind" = str "recruit" then (
                        let previous = List.map (fun u -> get u "id") before in
                        let recruits =
                          Hashtbl.fold
                            (fun _ u acc ->
                              if
                                (not (List.mem (get u "id") previous))
                                && get u "side" = str side
                                && get u "tile" = get action "tile"
                                && get u "type" = get action "type"
                                && not
                                     (Hashtbl.mem investments
                                        (text (get u "id")))
                              then u :: acc
                              else acc)
                            lookup_ []
                        in
                        Journal.require
                          (List.length recruits = 1)
                          "Cannot identify recorded new recruit";
                        let unit = List.hd recruits in
                        Hashtbl.replace investments
                          (text (get unit "id"))
                          (if get action "cost" <> `Null then get action "cost"
                           else cost unit))
                      else if get action "kind" = str "move" then
                        let u = lookup lookup_ (text (get action "unit")) in
                        let tile = cell (get action "tile") in
                        let owner = lookup owners tile in
                        if
                          capturable (get (lookup tiles tile) "terrain")
                          && (owner = `Null || hostile (faction u) owner)
                        then (
                          Hashtbl.replace owners tile (faction u);
                          Hashtbl.replace captors tile u;
                          add_value u
                            [
                              ("captures", int 1);
                              ( "objectiveCaptures",
                                int
                                  (if objective (text (get u "side")) tile then
                                     1
                                   else 0) );
                            ]))
                    (list (get phase "actions"));
                  List.iter
                    (fun loss ->
                      let id = text (get loss "id") in
                      let paid =
                        default
                          (obj [ ("paid", lookup investments id) ])
                          "paid" (int 0)
                      in
                      add_value loss
                        [
                          ("casualties", int 1);
                          ( "recruitGoldLost",
                            if paid = `Null then int 0 else paid );
                          ("casualtyReplacementValue", cost loss);
                        ];
                      Hashtbl.remove investments id)
                    losses;
                  List.iter
                    (fun combat ->
                      let mitigation = get combat "mitigation" in
                      Journal.require (mitigation <> `Null)
                        "Combat mitigation telemetry missing";
                      let attacker = lookup lookup_ (text (get combat "unit"))
                      and target =
                        lookup lookup_ (text (get combat "target"))
                      in
                      List.iter
                        (fun (event, actor, defender, response) ->
                          if event <> `Null then (
                            let actual =
                              if get event "effectiveDamage" <> `Null then
                                get event "effectiveDamage"
                              else
                                float
                                  (min
                                     (number (get event "afterTerrainDamage"))
                                     (number (get event "availableHp")))
                            in
                            add_value actor
                              [
                                ("hpDamageDealt", actual);
                                ("weaponHPDamageDealt", actual);
                              ];
                            add_value defender
                              [
                                ("hpDamageTaken", actual);
                                ("weaponHPDamageTaken", actual);
                                ( "armourDamagePrevented",
                                  get event "effectiveArmourPrevented" );
                                ( "terrainDamagePrevented",
                                  get event "effectiveTerrainPrevented" );
                                ( "rawArmourDelta",
                                  float
                                    (number (get event "rawDamage")
                                    -. number (get event "afterArmourDamage"))
                                );
                                ( "rawTerrainDelta",
                                  float
                                    (number (get event "afterArmourDamage")
                                    -. number (get event "afterTerrainDamage"))
                                );
                                ("attacksTaken", int (if response then 0 else 1));
                                ( "retaliationsTaken",
                                  int (if response then 1 else 0) );
                              ]))
                        [
                          (get mitigation "attack", attacker, target, false);
                          (get mitigation "retaliation", target, attacker, true);
                        ];
                      let thorns = get combat "thorns" in
                      let effective = number (get thorns "effectiveDamage") in
                      if effective <> 0. then (
                        let raw = number (get thorns "damage") in
                        let own = raw -. number (get thorns "auraDamage") in
                        add_value attacker
                          [
                            ("hpDamageTaken", float effective);
                            ("thornHPDamageTaken", float effective);
                          ];
                        if own <> 0. then
                          add_value target
                            [
                              ("hpDamageDealt", float (effective *. own /. raw));
                              ( "thornHPDamageDealt",
                                float (effective *. own /. raw) );
                            ];
                        let sources = list (get thorns "auraSources") in
                        Journal.require
                          (number (get thorns "auraDamage") = 0.
                          || sources <> [])
                          "Thorn aura has no provider";
                        List.iter
                          (fun source ->
                            let reflected =
                              effective
                              *. number (get thorns "auraDamage")
                              /. raw
                              /. Float.of_int (List.length sources)
                            in
                            add_value
                              (lookup lookup_ (text source))
                              [
                                ("hpDamageDealt", float reflected);
                                ("thornHPDamageDealt", float reflected);
                              ])
                          sources))
                    (list (get phase "combats"));
                  let observed = Hashtbl.create 128 in
                  List.iter
                    (fun o ->
                      Hashtbl.replace observed
                        (cell (get o "tile"))
                        (get o "owner"))
                    (list (get phase "ownership"));
                  Hashtbl.iter
                    (fun tile captor ->
                      if
                        lookup observed tile = faction captor
                        && get captor "side" = str side
                      then
                        add_value captor
                          [
                            ("ownedTileTurns", int 1);
                            ( "objectiveControlTurns",
                              int (if objective side tile then 1 else 0) );
                          ])
                    captors;
                  Hashtbl.iter (Hashtbl.replace owners) observed);
                if leaders then
                  List.iter
                    (fun audit ->
                      let leader = lookup lookup_ (text (get audit "leader")) in
                      let d =
                        remove
                          (identity leader
                             (if arena then landscape else mission))
                          [ "family" ]
                      in
                      let before = get audit "armyBefore"
                      and after = get audit "armyAfter"
                      and foe = get audit "opponentBefore"
                      and exposure = get audit "exposure" in
                      let last_combat =
                        List.rev (list (get phase "combats"))
                        |> List.find_opt (fun c ->
                            List.mem (get audit "leader")
                              [ get c "unit"; get c "target" ])
                        |> Option.value ~default:`Null
                      in
                      let died = get audit "after" = `Null in
                      let ratio a b =
                        if number b <> 0. then float (number a /. number b)
                        else `Null
                      in
                      let row =
                        merge d
                          ([
                             ("matchId", str (Json.identity result));
                             ("seed", get config "seed");
                             ("first", default config "first" (str ""));
                             ("phase", int phase_index);
                             ("round", get phase "round");
                             ("actingSide", str side);
                             ("side", get audit "side");
                             ("leader", get audit "leader");
                             ("unit", get leader "type");
                             ("died", bool died);
                             ("beforeHP", get (get audit "before") "hp");
                             ( "afterHP",
                               default (get audit "after") "hp" (int 0) );
                             ("beforeTile", get (get audit "before") "tile");
                             ("afterTile", get (get audit "after") "tile");
                             ("ownDecision", bool (str side = get audit "side"));
                             ("armyUnitsBefore", get before "units");
                             ("armyUnitsAfter", get after "units");
                             ("armyHPValueBefore", get before "hpValue");
                             ("armyHPValueAfter", get after "hpValue");
                             ("opponentHPValueBefore", get foe "hpValue");
                             ( "opponentHPValueAfter",
                               get (get audit "opponentAfter") "hpValue" );
                             ( "armyValueRatioBefore",
                               ratio (get before "hpValue") (get foe "hpValue")
                             );
                             ( "armyRetainedFraction",
                               ratio (get after "hpValue")
                                 (get before "hpValue") );
                             ("armyHPBefore", get before "hp");
                             ("armyHPAfter", get after "hp");
                             ( "deathDuringOwnAttack",
                               bool
                                 (died
                                 && get last_combat "unit" = get audit "leader"
                                 ) );
                             ( "forecastLethalWithSafeAlternative",
                               if
                                 get exposure "chosenTileDamage" <> `Null
                                 && get exposure "minDamage" <> `Null
                               then
                                 bool
                                   (number (get exposure "chosenTileDamage")
                                    >= number (get (get audit "before") "hp")
                                   && number (get exposure "minDamage")
                                      < number (get (get audit "before") "hp"))
                               else `Null );
                             ( "endForecastLethal",
                               if
                                 get exposure "afterDamage" <> `Null && not died
                               then
                                 bool
                                   (number (get exposure "afterDamage")
                                   >= number (get (get audit "after") "hp"))
                               else `Null );
                           ]
                          @ fields (get audit "recruitmentBefore")
                          @ fields exposure)
                      in
                      leader_rows := row :: !leader_rows)
                    (list (get phase "leaderAudit")))
              phases)
          chapters;
        if arena && presence then (
          let memberships = Hashtbl.create 32 in
          Hashtbl.iter
            (fun (side, unit) () ->
              Hashtbl.replace memberships (side, "unit", unit) ();
              Hashtbl.replace memberships
                (side, "family", text (family unit))
                ())
            seen;
          Hashtbl.iter
            (fun (side, grouping, member) () ->
              List.iter
                (fun map ->
                  Frame.add present
                    (merge (group side map)
                       [
                         ("grouping", str grouping);
                         ("member", str member);
                         ("battlesPresent", int 1);
                       ]))
                [ landscape; str "all" ])
            memberships)));
  let output = ref [] in
  let put k v = output := !output @ [ (k, v) ] in
  if presence then (
    put "nonleader_unit_events" (Frame.finish role_events);
    let census = Frame.finish census
    and totals = Frame.finish totals
    and present = Frame.finish present in
    let match_dims a b = List.for_all (fun k -> get a k = get b k) dims in
    List.iter
      (fun grouping ->
        let rows =
          List.filter (fun r -> get r "grouping" = str grouping) present
          |> List.map (fun r ->
              let total = List.find (fun t -> match_dims t r) totals in
              let turns =
                census
                |> List.filter (fun c ->
                    match_dims c r
                    && (if grouping = "unit" then get c "unit"
                        else family (text (get c "unit")))
                       = get r "member")
                |> List.map (fun c -> get c "unitTurns")
                |> sum
              in
              let denominator = max 1. (number (get total "unitTurns"))
              and phases = max 1. (number (get total "phases"))
              and battles = max 1. (number (get total "battles")) in
              merge
                (remove r [ "grouping"; "member" ])
                [
                  (grouping, get r "member");
                  ("unitTurns", int (int_of_float turns));
                  ("totalUnitTurns", get total "unitTurns");
                  ("sharePercent", float (100. *. turns /. denominator));
                  ("observedPhases", get total "phases");
                  ("meanUnitsPerPhase", float (turns /. phases));
                  ("battles", get total "battles");
                  ( "battlePresencePercent",
                    float (100. *. number (get r "battlesPresent") /. battles)
                  );
                ])
        in
        put
          (if grouping = "unit" then "battlefield_units"
           else "battlefield_presence")
          rows)
      [ "unit"; "family" ]);
  if support then (
    let rows =
      Frame.finish effects
      |> List.map (fun r ->
          let turns = number (get r "providerTurns")
          and ability = text (get r "ability") in
          let rate field applies =
            if turns > 0. && applies then float (number (get r field) /. turns)
            else `Null
          in
          merge r
            [
              ( "family",
                if get r "unit" = str "all" then str "all"
                else family (text (get r "unit")) );
              ( "adjacentAlliesPerTurn",
                rate "adjacentContacts" (ability <> "thorns") );
              ( "usefulRecipientsPerTurn",
                rate "healedRecipients" (ability = "healing") );
              ("healedHPPerTurn", rate "healedHP" (ability = "healing"));
              ( "reflectedDamagePerTurn",
                rate "reflectedDamage" (ability <> "healing") );
              ( "overlapPercent",
                float
                  (if number (get r "coveredRecipients") = 0. then 0.
                   else
                     100.
                     *. number (get r "overlappingRecipients")
                     /. number (get r "coveredRecipients")) );
            ])
    in
    put "support_effects" rows;
    put "support_overlap"
      (Frame.finish overlaps
      |> List.map (fun r ->
          merge r
            [
              ( "pairCoverageProbability",
                float
                  (number (get r "contacts")
                  /. max 1. (number (get r "eligiblePairs"))) );
              ( "overlapPercent",
                float
                  (100.
                  *. number (get r "overlappingRecipients")
                  /. max 1. (number (get r "coveredRecipients"))) );
              ( "redundantContactPercent",
                float
                  (100.
                  *. (number (get r "contacts")
                     -. number (get r "coveredRecipients"))
                  /. max 1. (number (get r "contacts"))) );
            ])));
  if value then (
    put "battlefield_value" (Frame.finish values);
    put "battlefield_unit_value"
      (Frame.finish unit_values
      |> List.map (fun r -> set r "family" (family (text (get r "unit"))))));
  if leaders then (
    let decisions = Hashtbl.create 128
    and peaks = Hashtbl.create 128
    and deaths = ref [] in
    let rows =
      List.rev !leader_rows
      |> List.map (fun row ->
          let key =
            canonical
              (arr
                 (List.map (get row) [ "matchId"; "mission"; "side"; "leader" ]))
          in
          let peak =
            max
              (Option.value ~default:0. (Hashtbl.find_opt peaks key))
              (max
                 (number (get row "armyHPValueBefore"))
                 (number (get row "armyHPValueAfter")))
          in
          Hashtbl.replace peaks key peak;
          let row =
            merge row
              [
                ("armyPeakHPValue", float peak);
                ( "armyFractionOfPeakAfter",
                  if peak <> 0. then
                    float (number (get row "armyHPValueAfter") /. peak)
                  else `Null );
              ]
          in
          if truth (get row "ownDecision") then
            Hashtbl.replace decisions key row;
          (if truth (get row "died") then
             let previous = lookup decisions key in
             deaths :=
               merge row
                 (List.map
                    (fun k ->
                      ( "lastDecision" ^ String.capitalize_ascii k,
                        get previous k ))
                    [
                      "round";
                      "beforeHP";
                      "armyUnitsBefore";
                      "armyHPValueBefore";
                      "armyValueRatioBefore";
                      "beforeDamage";
                      "chosenTileDamage";
                      "minDamage";
                      "safeTiles";
                      "forecastLethalWithSafeAlternative";
                      "afterHP";
                      "afterDamage";
                      "endForecastLethal";
                    ])
               :: !deaths);
          row)
    in
    put "leader_audit" rows;
    put "leader_deaths" (List.rev !deaths));
  ignore add_zero;
  (!output, !matches)
