(* JSON is the persistence wire format, not an application-state representation.
   Every historical value is decoded here before native restoration. *)
external history : Js.Json.t = "default"
[@@mel.module "@save-history/stages.json"]
external river : Js.Json.t = "default"
[@@mel.module "@save-history/river-v1.json"]
external refuge : Js.Json.t = "default"
[@@mel.module "@save-history/ashen-refuge-v1.json"]
external developer : Js.Json.t = "default"
[@@mel.module "@save-history/developer-v1.json"]
external catalog_json : Domain.catalog_view -> Js.Json.t = "%identity"
external safe_integer : float -> bool = "isSafeInteger" [@@mel.scope "Number"]
external has_own : 'a -> string -> bool = "hasOwn" [@@mel.scope "Object"]
external from_entries : (string * Js.Json.t) array -> Js.Json.t = "fromEntries" [@@mel.scope "Object"]

let invalid () = invalid_arg "Invalid version-one battle"
let require condition = if not condition then invalid ()
let object_ value = match Js.Json.decodeObject value with Some v -> v | None -> invalid ()
let array value = match Js.Json.decodeArray value with Some v -> v | None -> invalid ()
let string value = match Js.Json.decodeString value with Some v -> v | None -> invalid ()
let number value = match Js.Json.decodeNumber value with Some v -> v | None -> invalid ()
let boolean value = match Js.Json.decodeBoolean value with Some v -> v | None -> invalid ()
let get value key = let dict = object_ value in if has_own dict key then Js.Dict.get dict key else None
let field value key = match get value key with Some v -> v | None -> invalid ()
let text value key = string (field value key)
let num value key = number (field value key)
let items value key = array (field value key)
let flag value key = boolean (field value key)
let null value = Js.Json.classify value = Js.Json.JSONNull
let present value key = match get value key with None -> false | Some _ -> true
let integer value = let n = number value in require (safe_integer n && n >= 0.); n
let count value key = integer (field value key)
let obj fields = Js.Json.object_ (Js.Dict.fromArray (Array.of_list fields))
let s = Js.Json.string
let n = Js.Json.number
let b = Js.Json.boolean
let a = Js.Json.array
let copy value = from_entries (Js.Dict.entries (object_ value))
let set value key entry = Js.Dict.set (object_ value) key entry
let default value key fallback = match get value key with Some v -> v | None -> fallback
let optional value key check = match get value key with None -> () | Some v -> check v
let texts value = Array.map string (array value)
let list value key = Array.to_list (items value key)
let filter_map f values = Array.of_list (List.filter_map f (Array.to_list values))
let filter f values = Array.of_list (List.filter f (Array.to_list values))
let has text values = List.mem text values
let unique values = require (List.length values = List.length (List.sort_uniq compare values))
let side value = let v = string value in require (v = "player" || v = "enemy"); v
let owner value = if null value then () else ignore (side value)
let find_id values id = match Array.find_opt (fun v -> text v "id" = id) values with
  | Some v -> v | None -> invalid ()
let int_string value = string_of_int (int_of_float value)
let tile_id col row = int_string col ^ "," ^ int_string row
let unit_id_pattern = Js.Re.fromString "^u[1-9]\\d*$"

let history_for map = default history map (obj ["dialogue", a [||]; "waves", a [||]])
let waves_for mission =
  let old = items (history_for (text (field mission "map") "id")) "waves" in
  Array.map (fun wave ->
      match Array.find_opt (fun previous -> text previous "id" = text wave "id") old with
      | None -> wave
      | Some previous -> let merged = copy wave in
        Array.iter (fun (key, value) -> set merged key value) (Js.Dict.entries (object_ previous)); merged)
    (items mission "waves")
let wave_marker mark waves =
  match Array.find_opt (fun wave -> match get wave "legacyIndex" with
      | Some index -> mark = "wave:" ^ int_string (number index) | None -> false) waves with
  | Some wave -> "wave:" ^ text wave "id" | None -> mark

let import catalog saved =
  let catalog = catalog_json catalog in
  let maps = items catalog "maps" and rules = items catalog "units" in
  let campaigns = items catalog "campaigns" in
  let rule id = find_id rules id in
  let experience id = let v = field (rule id) "experienceNeeded" in
    if null v then 0. else number v in
  let leader id = match get (rule id) "leader" with Some v -> boolean v | None -> false in
  let unit_type value = let id = string value in ignore (rule id); id in
  let terrain_order = Array.map (fun t -> text t "id") (items catalog "terrain") in
  let cost unit_type tile =
    let terrain = text tile "terrain" in
    let index = ref (-1) in Array.iteri (fun i t -> if t = terrain then index := i) terrain_order;
    require (!index >= 0);
    let costs = items (rule unit_type) "costs" in
    !index < Array.length costs && not (null costs.(!index)) in
  require (num saved "version" = 1.);
  optional saved "campaignDeploymentFormat" (fun v -> require (string v = "named-waves-v1"));
  let named = match get saved "campaignDeploymentFormat" with Some _ -> true | None -> false in
  let state = field saved "state" in
  let map_id = string (default state "mapId" (s "mirror")) in
  let map = find_id maps map_id in
  let current_revision = num map "revision" in
  let revision_value = default state "mapRevision" (n 1.) in
  let revision = if null revision_value then 1. else number revision_value in
  require (revision = 1. || revision = current_revision);
  let layout = if revision = 1. && map_id = "river" then river
    else if revision = 1. && map_id = "ashen-refuge" then refuge
    else if revision = 1. && map_id = "developer" then developer else map in
  let cols = num layout "cols" and rows = num layout "rows" in
  let tiles = Array.init (int_of_float (cols *. rows)) (fun index ->
      let col = float_of_int (index mod int_of_float cols) and row = float_of_int (index / int_of_float cols) in
      obj ["id", s (tile_id col row); "col", n col; "row", n row; "terrain", s "plains"; "owner", Js.Json.null]) in
  let tiles_by_id = Js.Dict.empty () in Array.iter (fun t -> Js.Dict.set tiles_by_id (text t "id") t) tiles;
  let tile id = match Js.Dict.get tiles_by_id id with Some t -> t | None -> invalid () in
  let authored_tiles = items layout (if layout == river || layout == refuge then "tiles" else "terrain") in
  Array.iter (fun authored -> let t = tile (tile_id (num authored "col") (num authored "row")) in
               set t "terrain" (field authored "terrain"); set t "owner" (default authored "owner" Js.Json.null)) authored_tiles;
  let round = count state "round" and next_id = count state "nextId" in
  require (round >= 1. && next_id >= 1.);
  let turn = side (field state "turn") and winner = field state "winner" in
  owner winner; require (turn = "player" || not (null winner));
  let pending = flag saved "pendingEnemy" in require (not pending || (turn = "player" && null winner));
  let gold = field state "gold" in ignore (count gold "player"); ignore (count gold "enemy");
  let log = items state "log" in require (Array.length log <= 8);
  Array.iter (fun v -> require (String.length (string v) <= 1000)) log;
  let units = Array.map copy (items state "units") in
  let occupied = Js.Dict.empty () and ids = Js.Dict.empty () in
  Array.iter (fun unit ->
      let id = text unit "id" in
      require (Js.Re.test ~str:id unit_id_pattern);
      require (float_of_string (String.sub id 1 (String.length id - 1)) < next_id);
      require (Js.Dict.get ids id = None);
      let kind = unit_type (field unit "type") and location = text unit "tile" in
      require (cost kind (tile location) && Js.Dict.get occupied location = None);
      ignore (side (field unit "side"));
      let hp = count unit "hp" in require (hp >= 1. && hp <= num (rule kind) "hp");
      optional unit "experience" (fun v -> require (integer v <= experience kind));
      optional unit "name" (fun v -> require (String.length (string v) <= 80));
      optional unit "role" (fun v -> let role = string v in
                             require (role = "courier" || (role = "leader" && leader kind)));
      ignore (flag unit "moved"); ignore (flag unit "attacked");
      optional unit "rebuilding" (fun v -> ignore (boolean v); require (leader kind));
      optional unit "retreating" (fun v -> ignore (boolean v); require (text unit "side" = "enemy"));
      set unit "experience" (default unit "experience" (n 0.));
      Js.Dict.set ids id true; Js.Dict.set occupied location unit) units;
  let units = ref units and revision = ref revision and next_id = ref next_id and winner = ref winner and log = ref log in
  let units_named name = Array.exists (fun u -> get u "name" = Some (s name)) !units in
  let campaign_definition = Array.find_opt (fun c -> Array.exists (fun m ->
      text (field m "map") "id" = map_id) (items c "missions")) campaigns in
  let campaign = match campaign_definition with
    | None -> require (not (present state "campaign")); None
    | Some definition ->
      let c = copy (field state "campaign") in
      require (text c "id" = text definition "id");
      let mission_index = int_of_float (count c "mission") in
      let missions = items definition "missions" in require (mission_index < Array.length missions);
      let mission = missions.(mission_index) in require (text (field mission "map") "id" = map_id);
      let placements = items (field mission "map") "units" in
      let recall = items c "recall" in require (Array.length recall <= 4);
      Array.iter (fun v -> require (not (leader (unit_type v)))) recall;
      require (count c "carryGold" <= 8.);
      let previous = ref [] in for i = 0 to mission_index - 1 do
        Array.iter (fun u -> if text u "side" = "player" && get u "role" <> Some (s "leader") then
                       match get u "name" with Some v -> previous := string v :: !previous | None -> ()) (items (field missions.(i) "map") "units") done;
      let validate_veteran entry = let kind = unit_type (field entry "type") in
        require (count entry "experience" <= experience kind);
        optional entry "name" (fun v -> require (String.length (string v) <= 80));
        optional entry "role" (fun v -> require (string v = "courier" || (string v = "leader" && leader kind))) in
      optional c "fallenCompanions" (fun v -> let names = Array.to_list (texts v) in unique names;
                                      List.iter (fun name -> require (has name !previous)) names);
      optional c "veterans" (fun v -> let entries = array v in require (Array.length entries = Array.length recall);
                              Array.iteri (fun i entry -> validate_veteran entry; require (field entry "type" = recall.(i) && not (present entry "role"))) entries);
      optional c "storyVeterans" (fun v -> let entries = array v in require (Array.length entries <= 8);
                                   Array.iter (fun entry -> validate_veteran entry; require (Array.exists (fun incoming ->
                                       text incoming "side" = "player" &&
                                       (if get entry "role" = Some (s "leader") then get incoming "role" = Some (s "leader")
                                        else present entry "name" && get entry "name" = get incoming "name") &&
                                       field (rule (text incoming "type")) "family" = field (rule (text entry "type")) "family") placements)) entries);
      let goals = list mission "goals" in
      let hold_goals = List.filter (fun goal -> text goal "kind" = "hold") goals in
      let vigil_goal = List.find_opt (fun goal -> text goal "kind" = "vigil") goals in
      let history = history_for map_id and historical_waves = waves_for mission in
      let allowed = List.map (fun name -> "fallen:" ^ name) !previous @
                    Array.to_list (filter_map (fun u -> if text u "side" = "player" then
                                                  Option.map (fun v -> "fallen:" ^ string v) (get u "name") else None) placements) @
                    (if text c "id" = "green-accord" then ["dain-fell"] else []) @
                    (if present mission "refuge" then ["held:5,4"] else []) @
                    List.map (fun event -> "event:" ^ text event "id") (list mission "events") @
                    List.map (fun wave -> "wave:" ^ text wave "id") (list mission "waves") @
                    (if named then [] else Array.to_list (filter_map (fun wave -> Option.map
                                                                         (fun index -> "wave:" ^ int_string (number index)) (get wave "legacyIndex")) historical_waves)) @
                    List.concat_map (fun goal -> match text goal "kind" with
                        | "seal" -> Array.to_list (Array.map (fun v -> "sealed:" ^ string v) (items goal "tiles"))
                        | "hold" | "vigil" -> ["held:" ^ text goal "tile"] | _ -> []) goals in
      let marks = Array.to_list (texts (field c "marks")) in unique marks;
      List.iter (fun mark -> require (has mark allowed)) marks;
      let marks = ref (List.map (fun mark -> if named then mark else wave_marker mark historical_waves) marks) in unique !marks;
      let validate_counter value turns = require (count value "heldTurns" <= turns && count value "lastCountedRound" <= round) in
      optional c "holds" (fun holds -> Array.iter (fun (id, progress) -> match List.find_opt
                                                                                 (fun goal -> text goal "tile" = id) hold_goals with
                                                  | None -> invalid () | Some goal -> validate_counter progress (num goal "turns")) (Js.Dict.entries (object_ holds)));
      if !winner = s "player" then List.iter (fun goal -> match get goal "startEvent" with
          | Some event when not (null event) -> let mark = "event:" ^ string event in if not (has mark !marks) then marks := !marks @ [mark]
          | _ -> ()) hold_goals;
      optional c "vigil" (fun v -> match vigil_goal with None -> invalid () | Some goal -> validate_counter v (num goal "turns"));
      optional c "refuge" (fun v -> require (present mission "refuge" && count v "heldTurns" <= 6.);
                            ignore (count v "lastCountedRound"); ignore (flag v "escalated"));
      optional c "choices" (fun v -> let choices = texts v in require (Array.length choices <= 1);
                             Array.iter (fun choice -> require (if text c "id" = "ashen-oath" then mission_index >= 1 && has choice ["refuge-peace"; "refuge-war"]
                                                                else text c "id" = "green-accord" && choice = "green-dain-dead")) choices);
      let failure = ["Your leader fell. The company cannot continue without them."; "The courier fell before reaching safety."] @
                    (match get mission "requiredCompanions" with None -> [] | Some v -> Array.to_list (Array.map
                                                                                                         (fun name -> string name ^ " fell. The company cannot continue without them.") (array v))) in
      let lines key value = match get value key with None -> [] | Some v -> Array.to_list (array v) in
      let known = lines "intro" mission @ List.concat_map (fun event -> lines "dialogue" event @ lines "warDialogue" event) (list mission "events") @
                  lines "victory" mission @ (match get mission "refuge" with None -> [] | Some r -> lines "escalation" r @ lines "warVictory" r) @
                  (match get mission "withoutDain" with None -> [] | Some patch -> lines "intro" patch @ lines "victory" patch) @
                  (match get mission "companionLosses" with None -> [] | Some losses -> Array.to_list (Js.Dict.values (object_ losses)) |> List.concat_map
                                                                                          (fun patch -> lines "intro" patch @ lines "victory" patch)) @ lines "dialogue" history @ lines "ending" definition @
                  List.map (fun line -> obj ["speaker", s "The company"; "portrait", field definition "leader"; "text", s line]) failure in
      let dialogue = items c "dialogue" in require (Array.length dialogue <= List.length known);
      let dialogue = Array.map (fun line -> let matches known = text known "speaker" = text line "speaker" && text known "text" = text line "text" in
                                 require (List.exists (fun known -> matches known && (field known "portrait" = field line "portrait" ||
                                                                                      (text known "portrait" = "worker" && text line "portrait" = "scout"))) known);
                                 let original = List.find matches known in obj ["speaker", field line "speaker"; "portrait", field original "portrait"; "text", field line "text"]) dialogue in
      require (count c "dialogueIndex" <= float_of_int (Array.length dialogue));
      require (flag c "completed" = (!winner = s "player" && mission_index = Array.length missions - 1));
      let outcome = field c "outcome" in require (if null !winner then null outcome else if !winner = s "player" then
                                                    outcome = s "Mission complete. The company can rest and regroup." else has (string outcome) failure);
      let c = copy c in
      set c "dialogue" (a dialogue); set c "marks" (a (Array.of_list (List.map s !marks)));
      let counter turns = obj ["heldTurns", n turns; "lastCountedRound", n round] in
      (match vigil_goal with None -> () | Some _ -> set c "vigil" (copy (default c "vigil" (counter 0.))));
      let holds = obj [] in List.iter (fun goal ->
          let started = match get goal "startEvent" with None -> true | Some v -> null v || has ("event:" ^ string v) !marks in
          let saved_holds = match get c "holds" with None -> None | Some h -> get h (text goal "tile") in
          let progress = match started, saved_holds with true, Some v -> copy v | _ -> counter (if !winner = s "player" then num goal "turns" else 0.) in
          set holds (text goal "tile") progress) hold_goals; set c "holds" holds;
      Some (c, definition, mission, historical_waves)
  in
  let at location = Js.Dict.get occupied location in
  let update_ownership entries terrain =
    let expected = Array.to_list tiles |> List.filter (fun t -> has (text t "terrain") terrain) in
    let entries = array entries in require (Array.length entries = List.length expected);
    let ids = Array.to_list (Array.map (fun v -> text v "id") entries) in unique ids;
    Array.iter (fun entry -> let t = tile (text entry "id") in require (List.exists (fun e -> e == t) expected);
                 owner (field entry "owner"); set t "owner" (field entry "owner")) entries in
  update_ownership (field state "villages") ["village"];
  if map_id = "ashen-refuge" && !revision = 1. then (
    List.iter (fun row -> let old = tile ("5," ^ row) and fresh = tile ("3," ^ row) in
                set fresh "terrain" (s "village"); set fresh "owner" (match at (text fresh "id") with Some u -> field u "side" | None -> field old "owner");
                set old "terrain" (s "plains"); set old "owner" Js.Json.null) ["1"; "7"]; revision := 2.);
  if campaign <> None || present state "sites" then (
    let sites = items state "sites" in
    let sites = if map_id = "lantern-bell" && !revision = 1. then Array.map (fun site ->
        if text site "id" <> "4,3" then site else let updated = copy site in set updated "id" (s "5,1");
          set updated "owner" (match at "5,1" with Some u -> field u "side" | None -> field site "owner"); updated) sites else sites in
    update_ownership (a sites) ["grove"; "ruins"; "fort"];
    if map_id = "lantern-bell" && !revision = 1. then revision := 2.)
  else Array.iter (fun tile -> if has (text tile "terrain") ["grove"; "ruins"; "fort"] then set tile "owner" Js.Json.null) tiles;
  (match campaign with
   | None -> if null !winner then List.iter (fun side -> require (Array.exists (fun u -> text u "side" = side &&
                                                                                         text (rule (text u "type")) "family" = "commander") !units)) ["player"; "enemy"]
   | Some (c, definition, mission, waves) ->
     List.iter (fun side -> require (Array.length (filter (fun u -> text u "side" = side && get u "role" = Some (s "leader")) !units) <= 1)) ["player"; "enemy"];
     if null !winner then (
       require (Array.exists (fun u -> text u "side" = "player" && get u "role" = Some (s "leader")) !units);
       if Array.exists (fun goal -> text goal "kind" = "escort") (items mission "goals") then
         require (Array.exists (fun u -> text u "side" = "player" && get u "role" = Some (s "courier")) !units));
     if text c "id" = "borrowed-tide" && num c "mission" < 2. then (
       let old_name = if num c "mission" = 0. then "Captain Vale" else "Factor Senn" in
       let officer = Array.find_opt (fun u -> text u "side" = "enemy" && get u "role" = Some (s "leader")) (items (field mission "map") "units") in
       match officer with None -> invalid () | Some officer ->
         Array.iter (fun u -> if text u "side" = "enemy" && get u "role" = Some (s "leader") && get u "name" = Some (s old_name) then
                        set u "name" (field officer "name")) !units;
         match Array.find_opt (fun line -> get line "speaker" = get officer "name") (items mission "intro") with None -> () | Some current ->
           set c "dialogue" (a (Array.map (fun line -> if text line "speaker" = old_name then copy current else line) (items c "dialogue"))));
     let add_mark mark = let marks = Array.to_list (texts (field c "marks")) in if not (has mark marks) then set c "marks" (a (Array.of_list (List.map s (marks @ [mark])))) in
     if text c "id" = "green-accord" then (
       let dead = match get c "choices" with None -> false | Some choices -> Array.exists ((=) (s "green-dain-dead")) (array choices) in
       if (num c "mission" <> 1. || !revision >= 2.) && not dead && not (units_named "Captain Dain") then add_mark "dain-fell";
       if num c "mission" = 1. && !revision = 1. then (
         let marked_dead = Array.exists ((=) (s "dain-fell")) (items c "marks") in
         if not dead && not marked_dead && not (units_named "Captain Dain") then (
           let origin = tile "5,5" in
           let axial tile = let col = num tile "col" and row = num tile "row" in col -. (row -. float_of_int (int_of_float row mod 2)) /. 2., row in
           let distance tile = let q,r = axial tile and oq,or_ = axial origin in max (abs_float (q -. oq)) (max (abs_float (r -. or_)) (abs_float ((q +. r) -. (oq +. or_)))) in
           let candidates = Array.to_list tiles |> List.filter (fun t -> cost "warden" t && at (text t "id") = None) in
           let candidates = List.stable_sort (fun a b -> compare (distance a) (distance b)) candidates in
           match candidates with [] -> () | location :: _ ->
             let unit = obj ["id", s ("u" ^ int_string !next_id); "type", s "warden"; "side", s "player"; "tile", field location "id";
                             "hp", field (rule "warden") "hp"; "moved", b false; "attacked", b false; "experience", n 0.; "name", s "Captain Dain"] in
             units := Array.append !units [|unit|]; next_id := !next_id +. 1.);
         revision := 2.);
       let missing = match get mission "requiredCompanions" with None -> None | Some names -> Array.find_opt
                                                                                                (fun name -> not (Array.exists (fun u -> text u "side" = "player" && get u "name" = Some name) !units)) (array names) in
       match !winner = s "player", missing with true, Some name ->
         let outcome = string name ^ " fell. The company cannot continue without them." in
         winner := s "enemy"; set c "completed" (b false); set c "outcome" (s outcome);
         let dialogue = items c "dialogue" in set c "dialogueIndex" (n (float_of_int (Array.length dialogue)));
         set c "dialogue" (a (Array.append dialogue [|obj ["speaker", s "The company"; "portrait", field definition "leader"; "text", s outcome]|]));
         let new_log = Array.append [|s outcome|] !log in log := Array.sub new_log 0 (min 8 (Array.length new_log))
                                              | _ -> ());
     let migrated = ref false in Array.iter (fun wave -> match get wave "previouslyDeployedBeforeRevision" with
         | Some before when !revision < number before -> add_mark ("wave:" ^ text wave "id"); migrated := true | _ -> ()) waves;
     if !migrated then revision := current_revision);
  let result = obj ["mapId", s map_id; "mapRevision", n !revision; "cols", n cols; "rows", n rows;
                    "round", n round; "turn", s turn; "winner", !winner; "gold", copy gold; "nextId", n !next_id;
                    "log", a !log; "units", a !units; "tiles", a tiles] in
  (match campaign with None -> () | Some (c,_,_,_) -> set result "campaign" c);
  result
