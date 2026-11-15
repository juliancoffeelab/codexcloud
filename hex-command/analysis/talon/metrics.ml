module R = Rel
(** Exposure metrics stay in typed Talon tables, including zero-exposure scopes.
*)

module E = Talon.Expr

let dims = Telemetry.dims
let key r ks = Json.canonical (Json.arr (List.map (Json.get r) ks))

let army_usage events purchases presence families =
  let counts =
    events
    |> R.join ~kind:Talon.Join.Left [ "unit" ] (Arena.family_table families)
    |> R.derive [ R.out "family" E.(coalesce [ R.s "family"; R.s "unit" ]) ]
    |> R.all_maps
    |> R.group (dims @ [ "family" ]) Telemetry.events
    |> R.derive
         E.
           [
             "__attacks" := over ~by:dims (sum (R.f "attacks"));
             "__damage" := over ~by:dims (sum (R.f "damage"));
           ]
  in
  let buys =
    purchases
    |> R.select (dims @ [ "family"; "sharePercent" ])
    |> R.rename [ ("sharePercent", "purchaseSharePercent") ]
  in
  presence
  |> R.join ~kind:Talon.Join.Left ~one:true (dims @ [ "family" ]) counts
  |> R.join ~kind:Talon.Join.Left ~one:true (dims @ [ "family" ]) buys
  |> R.derive
       (List.map
          (fun n -> R.out n (R.numeric n))
          (Telemetry.events
          @ [ "__attacks"; "__damage"; "purchaseSharePercent" ]))
  |> R.derive
       E.
         [
           "attackSharePercent" :=
             float 100. *. R.f "attacks"
             /. R.maximum (R.f "__attacks") (float 1.);
           "damageSharePercent" :=
             float 100. *. R.f "damage" /. R.maximum (R.f "__damage") (float 1.);
           "attacksPerUnitTurn" :=
             R.f "attacks" /. R.maximum (R.f "unitTurns") (float 1.);
           "retaliationsPerUnitTurn" :=
             R.f "retaliations" /. R.maximum (R.f "unitTurns") (float 1.);
         ]
  |> R.drop [ "__attacks"; "__damage" ]
  |> R.sort (dims @ [ "family" ])

let unit_damage presence values profiles membership =
  let values =
    values
    |> R.filter E.(R.s "kind" = string "arena")
    |> R.derive
         E.
           [
             "unitTurns" := R.f "nonleaderUnitTurns";
             "weaponHPDamageDealt" := R.f "nonleaderWeaponHPDamageDealt";
             "thornHPDamageDealt" := R.f "nonleaderThornHPDamageDealt";
           ]
  in
  let known_values =
    values
    |> R.filter
         E.(
           R.f "nonleaderUnitTurns" <> float 0.
           || R.f "nonleaderWeaponHPDamageDealt" <> float 0.
           || R.f "nonleaderThornHPDamageDealt" <> float 0.)
    |> R.select [ "race"; "unit" ]
  in
  let known =
    R.concat
      [
        R.select [ "race"; "unit" ] presence;
        known_values;
        R.select [ "race"; "unit" ] membership;
      ]
    |> R.unique [ "race"; "unit" ]
  in
  let scopes =
    R.concat [ R.select dims presence; R.select dims values ] |> R.unique dims
  in
  let prof =
    profiles
    |> R.select [ "type"; "family"; "profileName"; "profileRole"; "level" ]
    |> R.rename [ ("type", "unit"); ("profileName", "name") ]
    |> R.derive
         [
           R.out "unitKind"
             E.(
               if_
                 (R.s "profileRole" = string "warrior")
                 (string "infantry")
                 (if_
                    (R.s "profileRole" = string "flank")
                    (string "flank")
                    (if_
                       (R.s "profileRole" = string "ranger")
                       (string "ranger")
                       (if_
                          (R.s "profileRole" = string "healer")
                          (string "support") (string "unknown")))));
         ]
    |> R.drop [ "profileRole" ]
  in
  let p = R.select (dims @ [ "unit"; "unitTurns" ]) presence in
  let v =
    R.select
      (dims
      @ [ "unit"; "unitTurns"; "weaponHPDamageDealt"; "thornHPDamageDealt" ])
      values
  in
  let checks =
    p
    |> R.join ~kind:Talon.Join.Full (dims @ [ "unit" ])
         (v
         |> R.select (dims @ [ "unit"; "unitTurns" ])
         |> R.rename [ ("unitTurns", "valueTurns") ])
    |> R.filter E.(R.numeric "unitTurns" <> R.numeric "valueTurns")
  in
  Journal.require (Talon.rows checks = 0) "Damage/presence census mismatch";
  let totals_p =
    p |> R.aggregate dims [ R.out "totalUnitTurns" E.(sum (R.f "unitTurns")) ]
  in
  let totals_v =
    v
    |> R.aggregate dims
         [ R.out "totalDamage" E.(sum (R.f "weaponHPDamageDealt")) ]
  in
  let q =
    scopes |> R.join [ "race" ] known
    |> R.join ~kind:Talon.Join.Left ~one:true [ "unit" ] prof
    |> R.join ~kind:Talon.Join.Left ~one:true (dims @ [ "unit" ]) p
    |> R.join ~kind:Talon.Join.Left ~one:true (dims @ [ "unit" ])
         (v |> R.drop [ "unitTurns" ]
         |> R.rename
              [
                ("weaponHPDamageDealt", "weaponDamage");
                ("thornHPDamageDealt", "thornDamage");
              ])
    |> R.join ~kind:Talon.Join.Left ~one:true dims totals_p
    |> R.join ~kind:Talon.Join.Left ~one:true dims totals_v
    |> R.derive
         (List.map
            (fun n -> R.out n (R.numeric n))
            [
              "unitTurns";
              "totalUnitTurns";
              "weaponDamage";
              "thornDamage";
              "totalDamage";
            ])
    |> R.derive
         E.
           [
             "damage" := R.f "weaponDamage";
             "presenceSharePercent" :=
               if_
                 (R.f "totalUnitTurns" > float 0.)
                 (float 100. *. R.f "unitTurns" /. R.f "totalUnitTurns")
                 (float 0.);
             "damageSharePercent" :=
               R.rate (float 100. *. R.f "weaponDamage") (R.f "totalDamage");
             "damagePerUnitTurn" :=
               R.rate (R.f "weaponDamage") (R.f "unitTurns");
             "thornDamagePerUnitTurn" :=
               R.rate (R.f "thornDamage") (R.f "unitTurns");
             "factionDamagePerUnitTurn" :=
               R.rate (R.f "totalDamage") (R.f "totalUnitTurns");
           ]
    |> R.derive
         E.
           [
             "damagePresenceRatio" :=
               R.rate (R.f "damageSharePercent") (R.f "presenceSharePercent");
             "__kind" :=
               if_
                 (R.s "unitKind" = string "infantry")
                 (int 0)
                 (if_
                    (R.s "unitKind" = string "flank")
                    (int 1)
                    (if_
                       (R.s "unitKind" = string "ranger")
                       (int 2)
                       (if_ (R.s "unitKind" = string "support") (int 3) (int 4))));
           ]
    |> R.sort (dims @ [ "__kind"; "family"; "level"; "name" ])
    |> R.drop [ "__kind" ]
  in
  Journal.require
    (Talon.rows (R.filter E.(is_null (R.s "family")) q) = 0)
    "Missing recorded unit profile";
  q

let compare_damage current baseline =
  let fields =
    [
      "unitTurns";
      "damage";
      "presenceSharePercent";
      "damageSharePercent";
      "damagePerUnitTurn";
      "damagePresenceRatio";
      "thornDamage";
      "thornDamagePerUnitTurn";
    ]
  in
  let old =
    baseline
    |> R.select (dims @ [ "unit" ] @ fields)
    |> R.rename (List.map (fun n -> (n, n ^ "Before")) fields)
  in
  current
  |> R.join ~kind:Talon.Join.Left ~one:true (dims @ [ "unit" ]) old
  |> R.derive
       (List.map
          (fun n -> R.out (n ^ "Delta") E.(R.f n -. R.f (n ^ "Before")))
          fields)
