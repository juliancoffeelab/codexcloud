open Json
let dims=Telemetry.dims
let key r ks=canonical(arr(List.map(get r)ks))
let ratio a b=if number b=0. then `Null else float(number a/.number b)
let army_usage events purchases presence families =
  let facts=List.filter(fun e->get e "race"<>get e "opponentRace")events |> List.concat_map(fun e->List.map(fun map->
    obj(List.map(fun k->k,(if k="map" then map else get e k))dims@["family",default families(text(get e "unit"))(get e "unit")]@List.map(fun k->k,get e k)Arena.event_counts)) [get e "map";str "all"]) in
  let rows=Frame.group facts (dims@["family"]) Arena.event_counts and totals=Frame.group facts dims Arena.event_counts in
  let index rows keys=let h=Hashtbl.create 64 in List.iter(fun r->Hashtbl.replace h (key r keys) r)rows;h in
  let own=index rows(dims@["family"]) and total=index totals dims and buys=index purchases(dims@["family"]) in
  List.map(fun r->let k=key r(dims@["family"]) in let stats=lookup own k and total=lookup total(key r dims) and purchase=lookup buys k in
    let safe n=max 1.(number n) in
    merge r (["purchaseSharePercent",default purchase "sharePercent" (int 0);"attackSharePercent",float(100.*.number(get stats "attacks")/.safe(get total "attacks"));"damageSharePercent",float(100.*.number(get stats "damage")/.safe(get total "damage"));"attacksPerUnitTurn",float(number(get stats "attacks")/.safe(get r "unitTurns"));"retaliationsPerUnitTurn",float(number(get stats "retaliations")/.safe(get r "unitTurns"))]@List.map(fun k->k,default stats k(int 0))Arena.event_counts))presence
let unit_kind p=match text(get p "role") with "warrior"->"infantry"|"flank"->"flank"|"ranger"->"ranger"|"healer"->"support"|_->"unknown"
let damage_order r = let rank=match text(get r "unitKind")with "infantry"->0|"flank"->1|"ranger"->2|"support"->3|_->4 in rank,text(get r "family"),integer(get r "level"),text(get r "name")
let unit_damage presence values profiles membership =
  let known=Hashtbl.create 64 in
  List.iter(fun rows->List.iter(fun r->let u=text(get r "unit") in
    if default r "kind"(str "arena")=str "arena" && has profiles u && not(truth(get(get profiles u)"leader")) then Hashtbl.replace known (text(get r "race"),u) ())rows)[presence;values;membership];
  let groups=Hashtbl.create 64 in
  let add field rows=List.iter(fun r->if field="presence" || get r "kind"=str "arena" then (
    let u=text(get r "unit") in Journal.require(has profiles u)("Missing recorded unit profile: "^u);
    let k=key r dims in let d,p,v=Option.value ~default:(obj(List.map(fun k->k,get r k)dims),[],[]) (Hashtbl.find_opt groups k) in
    if not(truth(get(get profiles u)"leader")) then Hashtbl.replace groups k (d,(if field="presence" then r::p else p),(if field="damage" then r::v else v))))rows in
  add "presence" presence;add "damage" values;
  let result=Hashtbl.fold(fun _ (d,census,values) acc->
    let turns=sum(List.map(fun r->get r "unitTurns")census) and total_damage=sum(List.map(fun r->get r "weaponHPDamageDealt")values) in
    let p=Hashtbl.create 32 and v=Hashtbl.create 32 in List.iter(fun r->Hashtbl.replace p(text(get r "unit"))r)census;List.iter(fun r->Hashtbl.replace v(text(get r "unit"))r)values;
    Hashtbl.iter(fun u _->Journal.require(number(get(lookup p u)"unitTurns")=number(get(lookup v u)"unitTurns")) ("Damage/presence census mismatch: "^u))p;
    Hashtbl.iter(fun u _->Journal.require(number(get(lookup p u)"unitTurns")=number(get(lookup v u)"unitTurns")) ("Damage/presence census mismatch: "^u))v;
    Hashtbl.fold(fun(race,u)()acc->if race<>text(get d "race") then acc else
      let profile=get profiles u and observed=default(lookup p u)"unitTurns"(int 0) and stats=lookup v u in
      let dealt=default stats "weaponHPDamageDealt"(int 0) in
      let presence_share=if turns<>0. then 100.*.number observed/.turns else 0. in
      let damage_share=if total_damage<>0. then float(100.*.number dealt/.total_damage) else `Null in
      merge d ["unit",str u;"name",default profile "name"(str u);"level",get profile "level";"family",get profile "family";"unitKind",str(unit_kind profile);
        "unitTurns",observed;"totalUnitTurns",int(int_of_float turns);"damage",dealt;"totalDamage",float total_damage;"weaponDamage",dealt;"thornDamage",default stats "thornHPDamageDealt"(int 0);
        "thornDamagePerUnitTurn",ratio(default stats "thornHPDamageDealt"(int 0))observed;"presenceSharePercent",float presence_share;"damageSharePercent",damage_share;"damagePerUnitTurn",ratio dealt observed;
        "factionDamagePerUnitTurn",(if turns<>0. then float(total_damage/.turns) else `Null);"damagePresenceRatio",(if presence_share<>0. && damage_share<>`Null then float(number damage_share/.presence_share) else `Null)]::acc)known acc) groups [] in
  List.sort(fun a b->compare(List.map(get a)dims,damage_order a)(List.map(get b)dims,damage_order b))result
let compare_damage current baseline =
  let index=Hashtbl.create 512 in List.iter(fun r->Hashtbl.replace index(key r(dims@["unit"]))r)baseline;
  let fields_=["unitTurns";"damage";"presenceSharePercent";"damageSharePercent";"damagePerUnitTurn";"damagePresenceRatio";"thornDamage";"thornDamagePerUnitTurn"] in
  List.map(fun r->let before=lookup index(key r(dims@["unit"])) in
    merge r(List.concat_map(fun k->let old=get before k and now=get r k in [k^"Before",old;k^"Delta",(if old<>`Null && now<>`Null then float(number now-.number old) else `Null)])fields_))current
