open Json
(** Telemetry joins and reductions over the once-decoded dataset. *)

module R = Rel
module E = Talon.Expr
module Q = Talon.Query

let dims = [ "race"; "policy"; "opponentPolicy"; "map" ]
let value_dims = Dataset.value_dims

let fields =
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

let events =
  [ "recruited"; "promoted"; "lost"; "attacks"; "retaliations"; "damage" ]

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

let overlap_fields =
  [
    "opportunities";
    "eligiblePairs";
    "contacts";
    "coveredRecipients";
    "overlappingRecipients";
  ]

let capturable = [ "village"; "grove"; "fort"; "ruins" ]
let without_family = List.filter (( <> ) "family") value_dims

let join_unit ?(id = "unit") lookup event =
  event
  |> R.join ~kind:Talon.Join.Left ~required:true [ "p"; id ]
       (R.rename [ ("id", id) ] lookup)

let own_phases (d : Dataset.t) =
  R.select [ "p"; "side" ] d.phases |> R.rename [ ("side", "actingSide") ]

let recruits (d : Dataset.t) =
  let newborns =
    d.units
    |> R.join ~kind:Talon.Join.Anti [ "p"; "id" ]
         (R.select [ "p"; "id" ] d.before)
    |> R.rename [ ("cell", "destination") ]
  in
  let actions =
    d.actions
    |> R.filter E.(R.s "kind" = string "recruit")
    |> R.drop [ "kind"; "unit" ]
    |> R.join [ "p" ] (R.select [ "p"; "side" ] d.phases)
  in
  let result =
    actions
    |> R.join ~kind:Talon.Join.Left ~one:true
         [ "p"; "side"; "type"; "destination" ]
         newborns
  in
  Journal.require
    (Talon.rows (R.filter E.(is_null (R.s "id")) result) = 0
    && Talon.rows result = Talon.rows actions)
    "Cannot identify recorded recruit role";
  result |> R.rename [ ("destination", "cell") ]

let presence (d : Dataset.t) ending lookup recruited =
  let arena = E.(R.s "kind" = string "arena") in
  let own =
    ending |> R.filter E.(arena && R.s "side" = R.s "actingSide" && R.nonleader)
  in
  let sides =
    List.map
      (fun (side, other) ->
        d.meta |> R.filter arena
        |> R.project
             E.
               [
                 keep (Talon.Sel.names [ "m" ]);
                 "side" := string side;
                 "race" := R.s (side ^ "Race");
                 "policy" := R.s side;
                 "opponentPolicy" := R.s other;
                 "map" := R.s "landscape";
               ])
      [ ("player", "enemy"); ("enemy", "player") ]
    |> R.concat
  in
  let battles = sides |> R.all_maps |> R.count dims "battles" in
  let phases =
    d.phases
    |> R.select [ "p"; "m"; "side" ]
    |> R.join [ "m"; "side" ] sides
    |> R.all_maps
    |> R.count dims "observedPhases"
  in
  let totals = own |> R.all_maps |> R.count dims "totalUnitTurns" in
  let seen =
    R.concat
      (List.map
         (fun units ->
           Dataset.describe d units
           |> R.select (("m" :: dims) @ [ "type"; "family"; "role"; "kind" ]))
         [ d.initial; d.before; d.losses; d.units ])
    |> R.filter E.(arena && R.nonleader)
  in
  let presence =
    List.map
      (fun (member, name, output) ->
        let p =
          seen
          |> R.select (("m" :: dims) @ [ member ])
          |> R.unique (("m" :: dims) @ [ member ])
          |> R.all_maps
          |> R.count (dims @ [ member ]) "battlesPresent"
        in
        let turns =
          own |> R.all_maps |> R.count (dims @ [ member ]) "unitTurns"
        in
        let result =
          p
          |> R.join ~kind:Talon.Join.Left (dims @ [ member ]) turns
          |> R.join ~kind:Talon.Join.Left dims battles
          |> R.join ~kind:Talon.Join.Left dims phases
          |> R.join ~kind:Talon.Join.Left dims totals
          |> R.derive
               (List.map
                  (fun n -> R.out n (R.numeric n))
                  [ "unitTurns"; "observedPhases"; "totalUnitTurns" ])
          |> R.derive
               E.
                 [
                   "sharePercent" :=
                     float 100. *. R.f "unitTurns"
                     /. R.maximum (R.f "totalUnitTurns") (float 1.);
                   "meanUnitsPerPhase" :=
                     R.f "unitTurns"
                     /. R.maximum (R.f "observedPhases") (float 1.);
                   "battlePresencePercent" :=
                     float 100. *. R.f "battlesPresent"
                     /. R.maximum (R.f "battles") (float 1.);
                 ]
          |> R.rename [ (member, output) ]
          |> R.sort (dims @ [ output ])
        in
        (name, result))
      [
        ("type", "battlefield_units", "unit");
        ("family", "battlefield_presence", "family");
      ]
  in
  let event stats table =
    table
    |> R.filter E.(arena && R.nonleader)
    |> R.rename [ ("type", "unitType") ]
    |> R.facts (dims @ [ "unitType" ]) events stats
    |> R.rename [ ("unitType", "unit") ]
  in
  let role =
    R.concat
      [
        Dataset.describe d d.losses |> event [ ("lost", R.one) ];
        Dataset.describe d recruited |> event [ ("recruited", R.one) ];
        d.actions
        |> R.filter E.(R.s "kind" = string "advance")
        |> R.select [ "p"; "unit" ]
        |> join_unit lookup
        |> event [ ("promoted", R.one) ];
        d.combats
        |> R.select [ "p"; "unit"; "damage" ]
        |> R.rename [ ("damage", "rawDamage") ]
        |> join_unit lookup
        |> event [ ("attacks", R.one); ("damage", R.f "rawDamage") ];
        d.combats
        |> R.filter
             E.(
               R.numeric "retaliationCount" > float 0.
               || R.numeric "retaliation" > float 0.)
        |> R.select [ "p"; "target"; "retaliationCount"; "retaliation" ]
        |> R.rename [ ("target", "unit") ]
        |> join_unit lookup
        |> event
             [
               ("retaliations", R.flag E.(R.f "retaliationCount" > float 0.));
               ("damage", R.f "retaliation");
             ];
      ]
    |> R.group (dims @ [ "unit" ]) events
  in
  ("nonleader_unit_events", role) :: presence

let support (d : Dataset.t) before lookup =
  let arena_ids =
    d.phases
    |> R.select [ "p"; "m" ]
    |> R.join [ "m" ]
         (d.meta |> R.filter E.(R.s "kind" = string "arena") |> R.select [ "m" ])
    |> R.select [ "p" ]
  in
  let friends =
    before
    |> R.filter E.(R.s "kind" = string "arena")
    |> R.derive
         E.
           [
             "q" := R.i "col" - ((R.i "row" - (R.i "row" mod int 2)) / int 2);
             "r" := R.i "row";
           ]
  in
  let effect_rows ability stats table =
    table
    |> R.derive [ R.out "ability" (E.string ability) ]
    |> R.rename [ ("type", "providerType") ]
    |> R.facts
         (dims @ [ "providerType"; "family"; "ability" ])
         effect_fields stats
    |> R.rename [ ("providerType", "unit") ]
  in
  let effects = ref [] and overlaps = ref [] in
  List.iter
    (fun ability ->
      let opportunities =
        friends
        |> R.filter
             (if ability = "healing" then E.(R.s "side" = R.s "actingSide")
              else E.(R.s "side" <> R.s "actingSide"))
      in
      let providers = opportunities |> R.filter E.(R.f ability > float 0.) in
      effects :=
        effect_rows ability [ ("providerTurns", R.one) ] providers :: !effects;
      if ability <> "thorns" then (
        let sizes = opportunities |> R.count [ "p"; "side" ] "friends" in
        let counts =
          providers
          |> R.count ([ "p"; "side" ] @ dims) "providerCount"
          |> R.join [ "p"; "side" ] sizes
        in
        let recipients =
          opportunities
          |> R.select [ "p"; "side"; "id"; "q"; "r" ]
          |> R.rename [ ("id", "recipient"); ("q", "rq"); ("r", "rr") ]
        in
        let touching =
          providers
          |> R.join [ "p"; "side" ] recipients
          |> R.filter E.(R.s "id" <> R.s "recipient")
          |> R.derive
               E.[ "dq" := R.i "q" - R.i "rq"; "dr" := R.i "r" - R.i "rr" ]
          |> R.filter
               E.(
                 nx { f = Nx.abs } (R.i "dq")
                 + nx { f = Nx.abs } (R.i "dr")
                 + nx { f = Nx.abs } (R.i "dq" + R.i "dr")
                 = int 2)
          |> R.derive
               [
                 R.out "touching"
                   E.(
                     cast Talon.Type.float64
                       (over ~by:[ "p"; "recipient" ] rows));
               ]
        in
        effects :=
          effect_rows ability
            E.
              [
                ("adjacentContacts", float 1.);
                ("coveredRecipients", float 1. /. R.f "touching");
                ( "overlappingRecipients",
                  if_
                    (R.f "touching" > float 1.)
                    (float 1. /. R.f "touching")
                    (float 0.) );
              ]
            touching
          :: !effects;
        let contacts = touching |> R.count [ "p"; "side" ] "contacts" in
        let covered =
          touching
          |> R.unique [ "p"; "side"; "recipient" ]
          |> R.count [ "p"; "side" ] "coveredRecipients"
        in
        let overlapping =
          touching
          |> R.filter E.(R.f "touching" > float 1.)
          |> R.unique [ "p"; "side"; "recipient" ]
          |> R.count [ "p"; "side" ] "overlappingRecipients"
        in
        let measured =
          counts
          |> R.join ~kind:Talon.Join.Left [ "p"; "side" ] contacts
          |> R.join ~kind:Talon.Join.Left [ "p"; "side" ] covered
          |> R.join ~kind:Talon.Join.Left [ "p"; "side" ] overlapping
          |> R.derive
               [
                 R.out "ability" (E.string ability);
                 R.out "recipientsPerProvider"
                   E.(R.maximum (R.f "friends" -. float 1.) (float 0.));
               ]
          |> R.facts
               (dims @ [ "ability"; "providerCount"; "recipientsPerProvider" ])
               overlap_fields
               E.
                 [
                   ("opportunities", float 1.);
                   ( "eligiblePairs",
                     R.f "providerCount"
                     *. R.maximum (R.f "friends" -. float 1.) (float 0.) );
                   ("contacts", R.numeric "contacts");
                   ("coveredRecipients", R.numeric "coveredRecipients");
                   ("overlappingRecipients", R.numeric "overlappingRecipients");
                 ]
        in
        overlaps := measured :: !overlaps))
    [ "healing"; "thornAura"; "thorns" ];
  let list_size name =
    E.(
      cast Talon.Type.float64
        (per_list
           [
             "__element" := Talon.Col.v (Talon.Kind.list Talon.Kind.string) name;
           ]
           rows))
  in
  let healing =
    d.healing
    |> R.join ~kind:Talon.Join.Semi [ "p" ] arena_ids
    |> R.derive [ R.out "n" (list_size "providers") ]
  in
  Journal.require
    (Talon.rows (R.filter E.(R.f "n" = float 0.) healing) = 0)
    "Healing event has no provider";
  let healing =
    Q.of_table healing
    |> Q.unnest
         [
           E.(
             "unit" :=
               Talon.Col.v (Talon.Kind.list Talon.Kind.string) "providers");
         ]
    |> Q.select [ E.keep (Talon.Sel.names [ "p"; "unit"; "amount"; "n" ]) ]
    |> R.run |> join_unit lookup
  in
  effects :=
    effect_rows "healing"
      E.
        [
          ("healedHP", R.f "amount" /. R.f "n");
          ("healedRecipients", float 1. /. R.f "n");
        ]
      healing
    :: !effects;
  let combats = d.combats |> R.join ~kind:Talon.Join.Semi [ "p" ] arena_ids in
  Journal.require
    (Talon.rows (R.filter E.(not (R.b "hasThorns")) combats) = 0)
    "Support telemetry missing from combat record";
  let thorns =
    combats
    |> R.filter E.(R.f "raw" > float 0.)
    |> R.derive
         [
           R.out "n" (list_size "sources");
           R.out "own" E.(R.f "raw" -. R.f "aura");
         ]
  in
  Journal.require
    (Talon.rows
       (R.filter E.(R.f "aura" <> float 0. && R.f "n" = float 0.) thorns)
    = 0)
    "Thorn aura event has no provider";
  let own =
    thorns
    |> R.filter E.(R.f "own" <> float 0.)
    |> R.select [ "p"; "target"; "own"; "raw"; "effective" ]
    |> R.rename [ ("target", "unit") ]
    |> join_unit lookup
  in
  effects :=
    effect_rows "thorns"
      E.
        [
          ("triggeredCombats", float 1.);
          ("reflectedDamage", R.f "own");
          ("effectiveReflectedDamage", R.f "effective" *. R.f "own" /. R.f "raw");
        ]
      own
    :: !effects;
  let aura =
    Q.of_table thorns
    |> Q.unnest
         [
           E.(
             "provider" :=
               Talon.Col.v (Talon.Kind.list Talon.Kind.string) "sources");
         ]
    |> Q.select
         [
           E.keep
             (Talon.Sel.names
                [ "p"; "provider"; "aura"; "raw"; "effective"; "n" ]);
         ]
    |> R.run
    |> R.rename [ ("provider", "unit") ]
    |> join_unit lookup
  in
  effects :=
    effect_rows "thornAura"
      E.
        [
          ("triggeredCombats", float 1. /. R.f "n");
          ("reflectedDamage", R.f "aura" /. R.f "n");
          ( "effectiveReflectedDamage",
            R.f "effective" *. R.f "aura" /. R.f "n" /. R.f "raw" );
        ]
      aura
    :: !effects;
  let effects = R.concat !effects in
  let effects =
    R.append effects
      (R.derive E.[ "unit" := string "all"; "family" := string "all" ] effects)
    |> R.all_maps
    |> R.group (dims @ [ "unit"; "family"; "ability" ]) effect_fields
  in
  let rate field_name condition =
    E.(
      if_
        (R.f "providerTurns" > float 0. && condition)
        (R.f field_name /. R.f "providerTurns")
        (store Talon.Type.float64 null))
  in
  let effects =
    effects
    |> R.derive
         E.
           [
             "adjacentAlliesPerTurn" :=
               rate "adjacentContacts" (R.s "ability" <> string "thorns");
             "usefulRecipientsPerTurn" :=
               rate "healedRecipients" (R.s "ability" = string "healing");
             "healedHPPerTurn" :=
               rate "healedHP" (R.s "ability" = string "healing");
             "reflectedDamagePerTurn" :=
               rate "reflectedDamage" (R.s "ability" <> string "healing");
             "overlapPercent" :=
               if_
                 (R.f "coveredRecipients" > float 0.)
                 (float 100.
                 *. R.f "overlappingRecipients"
                 /. R.f "coveredRecipients")
                 (float 0.);
           ]
  in
  let overlaps =
    R.concat !overlaps |> R.all_maps
    |> R.group
         (dims @ [ "ability"; "providerCount"; "recipientsPerProvider" ])
         overlap_fields
    |> R.derive
         E.
           [
             "pairCoverageProbability" :=
               R.f "contacts" /. R.maximum (R.f "eligiblePairs") (float 1.);
             "overlapPercent" :=
               float 100.
               *. R.f "overlappingRecipients"
               /. R.maximum (R.f "coveredRecipients") (float 1.);
             "redundantContactPercent" :=
               float 100.
               *. (R.f "contacts" -. R.f "coveredRecipients")
               /. R.maximum (R.f "contacts") (float 1.);
           ]
  in
  [ ("support_effects", effects); ("support_overlap", overlaps) ]

let capture_facts (d : Dataset.t) lookup =
  (* Only capture ownership is stateful. Iterate projected move/ownership
     columns; the roster, combat and numeric calculations remain relational. *)
  let moves =
    d.actions
    |> R.filter E.(R.s "kind" = string "move")
    |> R.select [ "p"; "action"; "unit"; "destination" ]
    |> join_unit lookup
    |> R.sort [ "p"; "action" ]
    |> Frame.rows
  in
  let by_phase = Hashtbl.create 2048 in
  List.iter
    (fun r ->
      let p = integer (get r "p") in
      Hashtbl.replace by_phase p
        (r :: Option.value ~default:[] (Hashtbl.find_opt by_phase p)))
    moves;
  let objectives = Hashtbl.create (Talon.rows d.objectives) in
  Frame.rows d.objectives
  |> List.iter (fun r ->
      Hashtbl.replace objectives
        ( integer (get r "m"),
          integer (get r "ch"),
          text (get r "side"),
          text (get r "cell") )
        ());
  let is_objective m ch side tile =
    Hashtbl.mem objectives (m, ch, side, tile)
  in
  let states = Hashtbl.create 128 in
  let output =
    R.Builder.create
      (Dataset.strings (value_dims @ [ "type"; "role" ])
      @ Dataset.floats
          [
            "captures";
            "objectiveCaptures";
            "ownedTileTurns";
            "objectiveControlTurns";
          ])
  in
  let emit row stats =
    R.Builder.add output (fun n ->
        match List.assoc_opt n stats with
        | Some v -> float v
        | None ->
            if
              List.mem n
                [
                  "captures";
                  "objectiveCaptures";
                  "ownedTileTurns";
                  "objectiveControlTurns";
                ]
            then int 0
            else get row n)
  in
  d.phases
  |> R.select [ "p"; "m"; "ch"; "side"; "ownership" ]
  |> Frame.rows
  |> List.iter (fun phase ->
      let p = integer (get phase "p")
      and m = integer (get phase "m")
      and ch = integer (get phase "ch")
      and acting = text (get phase "side") in
      let owners, terrain, captors, factions, tiles =
        match Hashtbl.find_opt states (m, ch) with
        | Some state -> state
        | None ->
            let h = List.find (fun (h : Dataset.header) -> h.m = m) d.headers in
            let chapter =
              if get h.result "kind" = str "arena" then h.result
              else List.nth (list (get h.result "missions")) ch
            in
            let initial = get chapter "initial" in
            let tiles = list (get initial "tiles") in
            let owners = Hashtbl.create 64
            and terrain = Hashtbl.create 64
            and captors = Hashtbl.create 16 in
            List.iter
              (fun t ->
                Hashtbl.add owners (text (get t "id")) (get t "owner");
                Hashtbl.add terrain (text (get t "id")) (text (get t "terrain")))
              tiles;
            let state =
              (owners, terrain, captors, list (get initial "factions"), tiles)
            in
            Hashtbl.add states (m, ch) state;
            state
      in
      let faction row =
        let f = get row "faction" in
        if truth f then f else get row "side"
      in
      let relation a b =
        match List.find_opt (fun f -> get f "id" = a) factions with
        | Some f -> get (get f "relations") (text b)
        | None -> `Null
      in
      let hostile a b =
        if a = b then false
        else
          let r = relation a b in
          let r = if r = `Null then relation b a else r in
          if r <> `Null then r = str "hostile"
          else
            List.sort compare [ a; b ]
            = List.sort compare [ str "player"; str "enemy" ]
      in
      Option.value ~default:[] (Hashtbl.find_opt by_phase p)
      |> List.rev
      |> List.iter (fun r ->
          let tile = text (get r "destination") in
          let owner =
            Option.value ~default:`Null (Hashtbl.find_opt owners tile)
          in
          if
            List.mem (Hashtbl.find terrain tile) capturable
            && (owner = `Null || hostile (faction r) owner)
          then (
            Hashtbl.replace owners tile (faction r);
            Hashtbl.replace captors tile r;
            emit r
              [
                ("captures", 1.);
                ( "objectiveCaptures",
                  if is_objective m ch (text (get r "side")) tile then 1.
                  else 0. );
              ]));
      let saved = (Hashtbl.find d.owners m).(integer (get phase "ownership")) in
      let values = list saved in
      Journal.require
        (List.length values = List.length tiles)
        "Invalid Arrow ownership snapshot";
      let observed = Hashtbl.create 64 in
      List.iter2
        (fun tile value ->
          let owner = if has value "owner" then get value "owner" else value in
          Hashtbl.add observed (text (get tile "id")) owner)
        tiles values;
      Hashtbl.iter
        (fun tile r ->
          if
            Hashtbl.find_opt observed tile = Some (faction r)
            && text (get r "side") = acting
          then
            emit r
              [
                ("ownedTileTurns", 1.);
                ( "objectiveControlTurns",
                  if is_objective m ch acting tile then 1. else 0. );
              ])
        captors;
      Hashtbl.iter (Hashtbl.replace owners) observed);
  R.Builder.finish output

let value (d : Dataset.t) ending lookup recruited =
  let part stats table =
    R.facts (value_dims @ [ "type"; "role" ]) fields stats table
  in
  let own =
    ending
    |> R.filter E.(R.s "side" = R.s "actingSide")
    |> R.join ~kind:Talon.Join.Left [ "m"; "ch"; "cell" ]
         (R.select [ "m"; "ch"; "cell"; "terrain" ] d.tiles)
    |> R.join ~kind:Talon.Join.Left
         [ "m"; "ch"; "side"; "cell" ]
         (R.derive [ R.out "isObjective" (E.bool true) ] d.objectives)
  in
  let parts =
    ref
      [
        part
          [
            ("unitTurns", R.one);
            ( "capturableOccupationTurns",
              R.flag (E.is_in capturable (R.s "terrain")) );
            ("objectiveOccupationTurns", R.flag (R.b "isObjective"));
          ]
          own;
      ]
  in
  let costs =
    R.Builder.create
      (Dataset.ints [ "m"; "ch" ]
      @ Dataset.strings [ "faction"; "type" ]
      @ Dataset.floats [ "replacement" ])
  in
  List.iter
    (fun (h : Dataset.header) ->
      let chapters =
        if get h.result "kind" = str "arena" then [ h.result ]
        else list (get h.result "missions")
      in
      List.iteri
        (fun ch chapter ->
          List.iter
            (fun faction ->
              List.iter
                (fun recruit ->
                  R.Builder.add costs (fun n ->
                      match n with
                      | "m" -> int h.m
                      | "ch" -> int ch
                      | "faction" -> get faction "id"
                      | "type" -> get recruit "unit"
                      | _ -> get recruit "cost"))
                (list (get faction "recruits")))
            (list (get (get chapter "initial") "factions")))
        chapters)
    d.headers;
  let costs = R.Builder.finish costs in
  let bought =
    Dataset.describe d recruited
    |> R.join ~kind:Talon.Join.Left [ "m"; "ch"; "faction"; "type" ] costs
    |> R.project
         E.
           [
             keep (Talon.Sel.names [ "m"; "ch"; "id" ]);
             "invested" :=
               coalesce [ R.f "paid"; R.f "replacement"; R.f "cost" ];
           ]
  in
  let losses =
    Dataset.describe d d.losses
    |> R.join ~kind:Talon.Join.Left [ "m"; "ch"; "faction"; "type" ] costs
    |> R.join ~kind:Talon.Join.Left ~one:true [ "m"; "ch"; "id" ] bought
  in
  parts :=
    part
      E.
        [
          ("casualties", float 1.);
          ("recruitGoldLost", R.numeric "invested");
          ( "casualtyReplacementValue",
            coalesce [ R.f "replacement"; R.f "cost" ] );
        ]
      losses
    :: !parts;
  Journal.require
    (Talon.rows (R.filter E.(not (R.b "hasMitigation")) d.combats) = 0)
    "Combat mitigation telemetry missing from combat record";
  List.iter
    (fun (response, prefix, actor, defender) ->
      let event =
        d.combats
        |> (if response then R.filter (R.b "hasResponse") else Fun.id)
        |> R.select
             ([ "p"; actor; defender ]
             @ List.map (fun n -> prefix ^ n) Dataset.damage_fields)
        |> R.rename
             ([ (actor, "actor"); (defender, "defender") ]
             @ List.map (fun n -> (prefix ^ n, n)) Dataset.damage_fields)
        |> R.derive
             [
               R.out "actual"
                 E.(
                   coalesce
                     [
                       R.f "effectiveDamage";
                       if_
                         (R.f "afterTerrainDamage" < R.f "availableHp")
                         (R.f "afterTerrainDamage") (R.f "availableHp");
                     ]);
             ]
      in
      let attack =
        event |> R.rename [ ("actor", "unit") ] |> join_unit lookup
      in
      parts :=
        part
          [
            ("hpDamageDealt", R.f "actual");
            ("weaponHPDamageDealt", R.f "actual");
          ]
          attack
        :: !parts;
      let defense =
        event |> R.rename [ ("defender", "unit") ] |> join_unit lookup
      in
      parts :=
        part
          E.
            [
              ("hpDamageTaken", R.f "actual");
              ("weaponHPDamageTaken", R.f "actual");
              ("armourDamagePrevented", R.f "effectiveArmourPrevented");
              ("terrainDamagePrevented", R.f "effectiveTerrainPrevented");
              ("rawArmourDelta", R.f "rawDamage" -. R.f "afterArmourDamage");
              ( "rawTerrainDelta",
                R.f "afterArmourDamage" -. R.f "afterTerrainDamage" );
              ("attacksTaken", float (if response then 0. else 1.));
              ("retaliationsTaken", float (if response then 1. else 0.));
            ]
          defense
        :: !parts)
    [
      (false, "attack", "unit", "target"); (true, "response", "target", "unit");
    ];
  let list_size name =
    E.(
      cast Talon.Type.float64
        (per_list
           [
             "__source" := Talon.Col.v (Talon.Kind.list Talon.Kind.string) name;
           ]
           rows))
  in
  let thorns =
    d.combats
    |> R.filter E.(R.f "effective" > float 0.)
    |> R.derive
         [
           R.out "own" E.(R.f "raw" -. R.f "aura");
           R.out "n" (list_size "sources");
         ]
  in
  Journal.require
    (Talon.rows
       (R.filter E.(R.f "aura" <> float 0. && R.f "n" = float 0.) thorns)
    = 0)
    "Thorn aura event has no provider";
  parts :=
    part
      [
        ("hpDamageTaken", R.f "effective");
        ("thornHPDamageTaken", R.f "effective");
      ]
      (thorns |> R.select [ "p"; "unit"; "effective" ] |> join_unit lookup)
    :: !parts;
  let reflected =
    thorns
    |> R.filter E.(R.f "own" <> float 0.)
    |> R.select [ "p"; "target"; "raw"; "effective"; "own" ]
    |> R.rename [ ("target", "unit") ]
    |> join_unit lookup
  in
  parts :=
    part
      E.
        [
          ("hpDamageDealt", R.f "effective" *. R.f "own" /. R.f "raw");
          ("thornHPDamageDealt", R.f "effective" *. R.f "own" /. R.f "raw");
        ]
      reflected
    :: !parts;
  let aura =
    Q.of_table thorns
    |> Q.unnest
         [
           E.(
             "provider" :=
               Talon.Col.v (Talon.Kind.list Talon.Kind.string) "sources");
         ]
    |> Q.select
         [
           E.keep
             (Talon.Sel.names
                [ "p"; "provider"; "raw"; "effective"; "aura"; "n" ]);
         ]
    |> R.run
    |> R.rename [ ("provider", "unit") ]
    |> join_unit lookup
  in
  parts :=
    part
      E.
        [
          ( "hpDamageDealt",
            R.f "effective" *. R.f "aura" /. R.f "raw" /. R.f "n" );
          ( "thornHPDamageDealt",
            R.f "effective" *. R.f "aura" /. R.f "raw" /. R.f "n" );
        ]
      aura
    :: !parts;
  let captures = capture_facts d lookup in
  parts :=
    part
      (List.map
         (fun n -> (n, R.f n))
         [
           "captures";
           "objectiveCaptures";
           "ownedTileTurns";
           "objectiveControlTurns";
         ])
      captures
    :: !parts;
  let facts = R.concat !parts |> R.all_maps in
  let families = R.group value_dims fields facts in
  let units =
    facts
    |> R.derive
         (List.map2
            (fun source target ->
              R.out target E.(if_ R.nonleader (R.f source) (float 0.)))
            [ "unitTurns"; "weaponHPDamageDealt"; "thornHPDamageDealt" ]
            nonleader_fields)
    |> R.group (without_family @ [ "type" ]) (fields @ nonleader_fields)
    |> R.join ~kind:Talon.Join.Left [ "type" ]
         (R.select [ "type"; "family" ] d.profiles)
    |> R.rename [ ("type", "unit") ]
  in
  [ ("battlefield_value", families); ("battlefield_unit_value", units) ]

let leaders (d : Dataset.t) lookup =
  let phase =
    d.phases
    |> R.select [ "p"; "phase"; "round"; "side" ]
    |> R.rename [ ("side", "actingSide") ]
  in
  let audits =
    d.audits |> join_unit ~id:"leader" lookup |> R.join [ "p" ] phase
  in
  let last =
    R.concat
      [
        d.combats
        |> R.project
             E.
               [
                 keep (Talon.Sel.names [ "p"; "combat" ]);
                 "leader" := R.s "unit";
                 "lastOwnAttack" := bool true;
               ];
        d.combats
        |> R.project
             E.
               [
                 keep (Talon.Sel.names [ "p"; "combat" ]);
                 "leader" := R.s "target";
                 "lastOwnAttack" := bool false;
               ];
      ]
    |> R.sort [ "combat" ]
    |> R.unique_last [ "p"; "leader" ]
    |> R.drop [ "combat" ]
  in
  let rows =
    audits
    |> R.join ~kind:Talon.Join.Left [ "p"; "leader" ] last
    |> R.derive
         E.
           [
             "side" := R.s "auditSide";
             "ownDecision" := R.s "actingSide" = R.s "auditSide";
             "armyValueRatioBefore" :=
               R.rate (R.f "armyHPValueBefore") (R.f "opponentHPValueBefore");
             "armyRetainedFraction" :=
               R.rate (R.f "armyHPValueAfter") (R.f "armyHPValueBefore");
             "deathDuringOwnAttack" :=
               R.b "died" && coalesce [ R.b "lastOwnAttack"; bool false ];
             "forecastLethalWithSafeAlternative" :=
               if_
                 ((not (is_null (R.f "chosenTileDamage")))
                 && not (is_null (R.f "minDamage")))
                 (R.f "chosenTileDamage" >= R.f "beforeHP"
                 && R.f "minDamage" < R.f "beforeHP")
                 (store Talon.Type.bool null);
             "endForecastLethal" :=
               if_
                 (not (R.b "died"))
                 (R.f "afterDamage" >= R.f "afterHP")
                 (store Talon.Type.bool null);
           ]
    |> R.rename [ ("type", "unit") ]
  in
  let keys = [ "matchId"; "mission"; "side"; "leader" ] in
  let rows =
    rows
    |> R.derive
         [
           R.out "armyPeakHPValue"
             E.(
               over ~by:keys
                 ~order:[ Talon.Order.asc "p" ]
                 (window
                    (max
                       (R.maximum (R.f "armyHPValueBefore")
                          (R.f "armyHPValueAfter")))));
         ]
    |> R.derive
         [
           R.out "armyFractionOfPeakAfter"
             (R.rate (R.f "armyHPValueAfter") (R.f "armyPeakHPValue"));
         ]
  in
  let keep =
    without_family
    @ [
        "matchId";
        "seed";
        "first";
        "phase";
        "round";
        "actingSide";
        "side";
        "leader";
        "unit";
        "died";
        "beforeHP";
        "afterHP";
        "beforeTile";
        "afterTile";
        "ownDecision";
        "armyUnitsBefore";
        "armyUnitsAfter";
        "armyHPValueBefore";
        "armyHPValueAfter";
        "opponentHPValueBefore";
        "opponentHPValueAfter";
        "armyValueRatioBefore";
        "armyRetainedFraction";
        "armyHPBefore";
        "armyHPAfter";
        "gold";
        "onOwnedKeep";
        "legalOptions";
        "affordableOptions";
      ]
    @ Dataset.exposure_fields
    @ [
        "minTile";
        "chosenTileReachable";
        "deathDuringOwnAttack";
        "forecastLethalWithSafeAlternative";
        "endForecastLethal";
        "armyPeakHPValue";
        "armyFractionOfPeakAfter";
      ]
  in
  let rows = rows |> R.select ("p" :: keep) in
  let decision_fields =
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
    ]
  in
  let title n =
    "lastDecision"
    ^ String.uppercase_ascii (String.sub n 0 1)
    ^ String.sub n 1 (String.length n - 1)
  in
  let decisions =
    rows
    |> R.filter (R.b "ownDecision")
    |> R.select (keys @ [ "p" ] @ decision_fields)
    |> R.rename
         (("p", "__decision")
         :: List.map (fun n -> (n, title n)) decision_fields)
  in
  let deaths =
    Q.of_table (R.filter (R.b "died") rows)
    |> Talon.Kit.join_asof ~by:keys ~on:("p", "__decision")
         (Q.of_table decisions)
    |> R.run
    |> R.drop [ "p"; "__decision" ]
  in
  let deaths =
    if Talon.rows (R.filter E.(R.b "died" && R.b "hasExposure") audits) = 0 then
      R.drop
        (Dataset.exposure_fields @ [ "minTile"; "chosenTileReachable" ])
        deaths
    else deaths
  in
  [ ("leader_audit", R.drop [ "p" ] rows); ("leader_deaths", deaths) ]

let analyze (d : Dataset.t) ~presence:want_presence ~support:want_support
    ~value:want_value ~leaders:want_leaders =
  List.iter
    (fun (h : Dataset.header) ->
      if want_support && get h.result "kind" = str "arena" then
        Journal.require
          (get h.report "supportTelemetryVersion" = int 1
          && truth (get h.report "unitProfiles"))
          "Support effects require fresh support telemetry and recorded \
           unitProfiles";
      if want_value then
        Journal.require
          (get h.report "combatTelemetryVersion" = int 1)
          "Battlefield value requires fresh combat telemetry";
      if want_leaders then
        Journal.require
          (get h.report "leaderAuditVersion" = int 1)
          "Leader audit requires fresh leader-risk telemetry")
    d.headers;
  let ending = Dataset.describe d d.units |> R.join [ "p" ] (own_phases d) in
  let before = Dataset.describe d d.before |> R.join [ "p" ] (own_phases d) in
  let lookup = Dataset.describe d d.lookup
  and support_lookup = Dataset.describe d d.support_lookup in
  let recruited = recruits d in
  (if want_presence then presence d ending lookup recruited else [])
  @ (if want_support then support d before support_lookup else [])
  @ (if want_value then value d ending lookup recruited else [])
  @ if want_leaders then leaders d lookup else []
