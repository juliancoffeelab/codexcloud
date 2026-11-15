open Json
let table tables name = Option.value ~default:[] (List.assoc_opt name tables)
let changes current previous keys fields_ =
  let index=Hashtbl.create 128 in List.iter(fun r->Hashtbl.replace index(Metrics.key r keys)r)previous;
  List.filter_map(fun r->match Hashtbl.find_opt index(Metrics.key r keys)with None->None|Some old->Some(merge r(List.map(fun k->k^"Before",get old k)fields_)))current
let match_changes current previous =
  if previous=[] then [] else let index=Hashtbl.create 2048 in List.iter(fun o->Hashtbl.add index(identity o)o)previous;
  Journal.require(List.length current=List.length previous && List.for_all(fun o->Hashtbl.mem index(identity o))current) "Unmatched configurations";
  List.map(fun o->let old=Hashtbl.find index(identity o) in
    if get o "kind"=str "campaign" then
      let summary r fields_=arr(get r "completed"::List.map(fun m->arr(List.map(get m)fields_))(list(get r "missions"))) in
      obj["matchId",str(identity o);"kind",str "campaign";"beforeCompleted",get old "completed";"completed",get o "completed";"beforeMissions",get old "missions";"missions",get o "missions";
        "resultChanged",bool(summary old["id";"winner"]<>summary o["id";"winner"]);"gameplayChanged",bool(summary old["id";"round";"stateHash"]<>summary o["id";"round";"stateHash"])]
    else obj["matchId",str(identity o);"kind",str "arena";"beforeWinner",get old "winner";"winner",get o "winner";"beforeReason",get old "reason";"reason",get o "reason";"beforeRounds",get old "rounds";"rounds",get o "rounds";
      "resultChanged",bool(List.map(get old)["winner";"reason"]<>List.map(get o)["winner";"reason"]);"gameplayChanged",bool(get old "stateHash"<>get o "stateHash")])current
let build current previous ~unit_usage ~presence ~support ~value ~leaders =
  let rows,units,ledger,campaigns=Arena.tally current.Store.outcomes in
  let before,before_units,_,_=Arena.tally previous.Store.outcomes in
  let transitions=match_changes current.outcomes previous.outcomes in
  let families=List.fold_left(fun all source->List.fold_left(fun all(k,v)->Journal.require(not(has all k)||get all k=v)("Conflicting unit family: "^k);set all k v)all(fields(get source "unitFamilies"))) (obj[])current.sources in
  let purchases=Arena.purchase_shares units families in
  let campaigns_=Campaign.tables current.outcomes in let old_campaigns=Campaign.tables previous.outcomes in
  let matchup_changes=changes rows before Arena.keys ["wins";"losses";"caps";"winRate"] |> List.map(fun r->set r "winRateDelta"(float(number(get r "winRate")-.number(get r "winRateBefore")))) in
  let campaign_changes=changes(table campaigns_ "campaign_summary")(table old_campaigns "campaign_summary")Campaign.fields_ ["completed";"completionRate"] |> List.map(fun r->set r "completionRateDelta"(float(number(get r "completionRate")-.number(get r "completionRateBefore")))) in
  let race_map opponent=Arena.combine (List.filter(fun r->get r "opponentPolicy"=str opponent && (get r "policy"<>str opponent || get r "race"<>get r "opponentRace"))rows)["race";"map";"policy"] in
  let policy_timings=List.concat_map(fun(label,sources)->List.concat_map(fun s->let timing=get s "turnTiming" in list(get timing "policies") |> List.map(fun r->merge r ["run",str label;"clock",get timing "clock";"source",get s "file"]))sources) ["current",current.sources;"baseline",previous.sources] in
  let tables=campaigns_@["matchups",rows;"unit_events",units;"matches",ledger;"campaigns",campaigns;
    "policy_matchups",Arena.combine ~cross:false (List.filter(fun r->get r "policy"<>get r "opponentPolicy")rows)["policy";"opponentPolicy"];
    "race_map_vs_classic",race_map "classic";"race_map_vs_balanced",race_map "balanced";"purchase_shares",purchases;
    "race_equal_policy",Arena.combine ~equal:true rows["race"];"race_map_equal_policy",Arena.combine ~equal:true rows["race";"map"];
    "race_map_policy",Arena.combine ~equal:true rows["race";"map";"policy"];"race_all_policies",Arena.combine rows["race"];
    "baseline_matchups",before;"matchup_changes",matchup_changes;"match_changes",transitions;"baseline_unit_events",before_units;"campaign_changes",campaign_changes;"policy_timings",policy_timings] in
  if not(unit_usage||presence||support||value||leaders) then tables else (
    if unit_usage then List.iter(fun s->Journal.require(get s "combatTelemetryVersion"=int 1) "Damage tables require fresh combat telemetry") (current.sources@previous.sources);
    let value=value || (unit_usage && List.for_all(fun s->get s "combatTelemetryVersion"=int 1)current.sources) in
    let leaders=leaders || (unit_usage && List.for_all(fun s->get s "leaderAuditVersion"=int 1)current.sources) in
    let extra,_=Telemetry.analyze current ~presence:(presence||unit_usage) ~support:(support||unit_usage) ~value ~leaders in
    let tables=tables@extra in
    let tables=if unit_usage||presence then
      let nonleaders=List.filter(fun e->not(List.exists(fun s->truth(get(get(get s "unitProfiles")(text(get e "unit")))"leader"))current.sources))units in
      tables@["army_usage",Metrics.army_usage nonleaders purchases (table tables "battlefield_presence")families] else tables in
    if not unit_usage then tables else
      let profiles=get(List.hd current.sources)"unitProfiles" in
      let old_extra=if previous.outcomes=[] then [] else fst(Telemetry.analyze previous ~presence:true ~support:false ~value:true ~leaders:false) in
      let membership=table tables "battlefield_units"@table old_extra "battlefield_units" in
      let damage=Metrics.unit_damage(table tables "battlefield_units")(table tables "battlefield_unit_value")profiles membership in
      let tables=tables@["unit_damage",damage] in
      if previous.outcomes=[] then tables else
        let old_damage=Metrics.unit_damage(table old_extra "battlefield_units")(table old_extra "battlefield_unit_value")profiles membership in
        tables@["baseline_unit_damage",old_damage;"unit_damage_changes",Metrics.compare_damage damage old_damage])
