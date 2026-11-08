(* Browser version-one importer. This follows the frozen legacy snapshot and
   board implementations; it operates on decoded OCaml values throughout. *)
module T = Historical_types
module D = Legacy_data
open T
module Map = T.String_map

let invalid () = invalid_arg "Invalid version-one battle"
let require value = if not value then invalid ()
let integer value = Migration_json.safe_integer value && value >= 0.
let find predicate values = Array.find_opt predicate values
let filter predicate values = Array.of_list (List.filter predicate (Array.to_list values))
let member value values = Array.exists (( = ) value) values
let concat values = Array.concat values

let unique values =
  Array.length values = List.length (List.sort_uniq compare (Array.to_list values))

external js_length : string -> int = "length" [@@mel.get]

let length value = js_length (Migration_utf8.to_js value)

let find_id id get_id values =
  match find (fun value -> get_id value = id) values with Some value -> value | None -> invalid ()

let tile_id col row = string_of_int (int_of_float col) ^ "," ^ string_of_int (int_of_float row)
let as_owner = function `Player -> `String "player" | `Enemy -> `String "enemy"
let valid_owner = function `Null -> true | `String value -> value = "player" || value = "enemy"
let no_winner = function `Null -> true | `Player | `Enemy -> false

let wave_marker mark waves =
  match
    find
      (fun (wave : D.wave) ->
        Option.exists
          (fun index -> mark = "wave:" ^ string_of_int (int_of_float index))
          wave.legacy_index)
      waves
  with
  | None -> mark
  | Some wave -> "wave:" ^ wave.id

let unit_type (value : Javascript1.type_) =
  match value with
  | `Ranger -> "ranger"
  | `Healer -> "healer"
  | `Briarweaver -> "briarweaver"
  | `Deatharcher -> "deatharcher"
  | `Direwolf -> "direwolf"
  | `Greatboar -> "greatboar"
  | `Wraith -> "wraith"
  | `Spectre -> "spectre"
  | `Lancer -> "lancer"
  | `Mage -> "mage"
  | `Marksman -> "marksman"
  | `Merfolk_maceguard_l2 -> "merfolk-maceguard-l2"
  | `Revenant -> "revenant"
  | `Skeleton_impact_caster_l2 -> "skeleton-impact-caster-l2"
  | `Soldier -> "soldier"
  | `Tidehunter -> "tidehunter"
  | `Tidesage -> "tidesage"
  | `Tridentguard -> "tridentguard"
  | `Warden -> "warden"
  | `Champion -> "champion"
  | `Hammerman -> "hammerman"
  | `Warhammer -> "warhammer"
  | `Sentinel -> "sentinel"
  | `Halberdier -> "halberdier"
  | `Halberd_master -> "halberd-master"
  | `Shadowblade -> "shadowblade"
  | `Slinger -> "slinger"
  | `Fireslinger -> "fireslinger"
  | `Sharpshooter -> "sharpshooter"
  | `Crossbowman -> "crossbowman"
  | `Arbalester -> "arbalester"
  | `Archmage -> "archmage"
  | `High_healer -> "high-healer"
  | `Knight -> "knight"
  | `Horse_bowman -> "horse-bowman"
  | `Horse_marksman -> "horse-marksman"
  | `Skeleton_dreadguard -> "skeleton-dreadguard"
  | `Skeleton_flanker_l2 -> "skeleton-flanker-l2"
  | `Skeleton_flanker_l3 -> "skeleton-flanker-l3"
  | `Skeleton_impact_caster_l3 -> "skeleton-impact-caster-l3"
  | `Skeleton_scythe_caster_l2 -> "skeleton-scythe-caster-l2"
  | `Skeleton_scythe_caster_l3 -> "skeleton-scythe-caster-l3"
  | `Skeleton_rider_l2 -> "skeleton-rider-l2"
  | `Skeleton_rider_l3 -> "skeleton-rider-l3"
  | `Skeleton_bow_master -> "skeleton-bow-master"
  | `Skeleton_axeman_l2 -> "skeleton-axeman-l2"
  | `Skeleton_axeman_l3 -> "skeleton-axeman-l3"
  | `Woodland_alpha_wolf -> "woodland-alpha-wolf"
  | `Woodland_ancient_boar -> "woodland-ancient-boar"
  | `Woodland_thorn_oracle -> "woodland-thorn-oracle"
  | `Woodland_bramble_warden -> "woodland-bramble-warden"
  | `Woodland_thornbinder -> "woodland-thornbinder"
  | `Fox_spearguard_l2 -> "fox-spearguard-l2"
  | `Fox_spearguard_l3 -> "fox-spearguard-l3"
  | `Treant_l2 -> "treant-l2"
  | `Treant_l3 -> "treant-l3"
  | `Merfolk_harpoon_master -> "merfolk-harpoon-master"
  | `Merfolk_netter_l2 -> "merfolk-netter-l2"
  | `Merfolk_netter_l3 -> "merfolk-netter-l3"
  | `Merfolk_reef_champion -> "merfolk-reef-champion"
  | `Merfolk_trident_l2 -> "merfolk-trident-l2"
  | `Merfolk_trident_l3 -> "merfolk-trident-l3"
  | `Merfolk_tide_oracle -> "merfolk-tide-oracle"
  | `Merfolk_tidecrusher_l2 -> "merfolk-tidecrusher-l2"
  | `Merfolk_tidecrusher_l3 -> "merfolk-tidecrusher-l3"
  | `Merfolk_maceguard_l3 -> "merfolk-maceguard-l3"
  | `Dragon -> "dragon"
  | `Skeletal_hound -> "skeletal-hound"
  | `Captives -> "captives"
  | `Zombie_walker -> "zombie-walker"
  | `Zombie_decay_caster -> "zombie-decay-caster"
  | `Bear -> "bear"
  | `Giant_bat -> "giant-bat"
  | `Giant_spider -> "giant-spider"
  | `Commander -> "commander"
  | `Grovewarden -> "grovewarden"
  | `Lich -> "lich"
  | `Marshal -> "marshal"
  | `Necromancer -> "necromancer"
  | `Tidekeeper -> "tidekeeper"
  | `Tideseer -> "tideseer"
  | `Wildkeeper -> "wildkeeper"
  | `Grand_marshal -> "grand-marshal"
  | `Lich_commander_l3 -> "lich-commander-l3"
  | `Woodland_grove_oracle -> "woodland-grove-oracle"
  | `Merfolk_tide_sovereign -> "merfolk-tide-sovereign"
  | `Adept -> "adept"
  | `Archer -> "archer"
  | `Boar -> "boar"
  | `Bonearcher -> "bonearcher"
  | `Ghost -> "ghost"
  | `Harpooner -> "harpooner"
  | `Merfolk_maceguard -> "merfolk-maceguard"
  | `Reefguard -> "reefguard"
  | `Rider -> "rider"
  | `Scout -> "scout"
  | `Skeleton_impact_caster -> "skeleton-impact-caster"
  | `Skeleton -> "skeleton"
  | `Spearman -> "spearman"
  | `Swordsman -> "swordsman"
  | `Thorncaller -> "thorncaller"
  | `Wavemender -> "wavemender"
  | `Wolf -> "wolf"
  | `Horse_archer -> "horse-archer"
  | `Skeleton_rider -> "skeleton-rider"
  | `Skeleton_axeman -> "skeleton-axeman"
  | `Fox_spearguard -> "fox-spearguard"
  | `Treant -> "treant"

let campaign_id (value : javascript1_state_campaign_member1_id) =
  match value with
  | `Lantern_road -> "lantern-road"
  | `Green_accord -> "green-accord"
  | `Ashen_oath -> "ashen-oath"
  | `Borrowed_tide -> "borrowed-tide"
  | `Mirror_night -> "mirror-night"
  | `Bell_night -> "bell-night"

let portrait (value : Javascript1.portrait) =
  match value with
  | `Ranger -> "ranger"
  | `Healer -> "healer"
  | `Briarweaver -> "briarweaver"
  | `Deatharcher -> "deatharcher"
  | `Direwolf -> "direwolf"
  | `Greatboar -> "greatboar"
  | `Wraith -> "wraith"
  | `Spectre -> "spectre"
  | `Lancer -> "lancer"
  | `Mage -> "mage"
  | `Marksman -> "marksman"
  | `Merfolk_maceguard_l2 -> "merfolk-maceguard-l2"
  | `Revenant -> "revenant"
  | `Skeleton_impact_caster_l2 -> "skeleton-impact-caster-l2"
  | `Soldier -> "soldier"
  | `Tidehunter -> "tidehunter"
  | `Tidesage -> "tidesage"
  | `Tridentguard -> "tridentguard"
  | `Warden -> "warden"
  | `Champion -> "champion"
  | `Hammerman -> "hammerman"
  | `Warhammer -> "warhammer"
  | `Sentinel -> "sentinel"
  | `Halberdier -> "halberdier"
  | `Halberd_master -> "halberd-master"
  | `Shadowblade -> "shadowblade"
  | `Slinger -> "slinger"
  | `Fireslinger -> "fireslinger"
  | `Sharpshooter -> "sharpshooter"
  | `Crossbowman -> "crossbowman"
  | `Arbalester -> "arbalester"
  | `Archmage -> "archmage"
  | `High_healer -> "high-healer"
  | `Knight -> "knight"
  | `Horse_bowman -> "horse-bowman"
  | `Horse_marksman -> "horse-marksman"
  | `Skeleton_dreadguard -> "skeleton-dreadguard"
  | `Skeleton_flanker_l2 -> "skeleton-flanker-l2"
  | `Skeleton_flanker_l3 -> "skeleton-flanker-l3"
  | `Skeleton_impact_caster_l3 -> "skeleton-impact-caster-l3"
  | `Skeleton_scythe_caster_l2 -> "skeleton-scythe-caster-l2"
  | `Skeleton_scythe_caster_l3 -> "skeleton-scythe-caster-l3"
  | `Skeleton_rider_l2 -> "skeleton-rider-l2"
  | `Skeleton_rider_l3 -> "skeleton-rider-l3"
  | `Skeleton_bow_master -> "skeleton-bow-master"
  | `Skeleton_axeman_l2 -> "skeleton-axeman-l2"
  | `Skeleton_axeman_l3 -> "skeleton-axeman-l3"
  | `Woodland_alpha_wolf -> "woodland-alpha-wolf"
  | `Woodland_ancient_boar -> "woodland-ancient-boar"
  | `Woodland_thorn_oracle -> "woodland-thorn-oracle"
  | `Woodland_bramble_warden -> "woodland-bramble-warden"
  | `Woodland_thornbinder -> "woodland-thornbinder"
  | `Fox_spearguard_l2 -> "fox-spearguard-l2"
  | `Fox_spearguard_l3 -> "fox-spearguard-l3"
  | `Treant_l2 -> "treant-l2"
  | `Treant_l3 -> "treant-l3"
  | `Merfolk_harpoon_master -> "merfolk-harpoon-master"
  | `Merfolk_netter_l2 -> "merfolk-netter-l2"
  | `Merfolk_netter_l3 -> "merfolk-netter-l3"
  | `Merfolk_reef_champion -> "merfolk-reef-champion"
  | `Merfolk_trident_l2 -> "merfolk-trident-l2"
  | `Merfolk_trident_l3 -> "merfolk-trident-l3"
  | `Merfolk_tide_oracle -> "merfolk-tide-oracle"
  | `Merfolk_tidecrusher_l2 -> "merfolk-tidecrusher-l2"
  | `Merfolk_tidecrusher_l3 -> "merfolk-tidecrusher-l3"
  | `Merfolk_maceguard_l3 -> "merfolk-maceguard-l3"
  | `Dragon -> "dragon"
  | `Skeletal_hound -> "skeletal-hound"
  | `Captives -> "captives"
  | `Zombie_walker -> "zombie-walker"
  | `Zombie_decay_caster -> "zombie-decay-caster"
  | `Bear -> "bear"
  | `Giant_bat -> "giant-bat"
  | `Giant_spider -> "giant-spider"
  | `Commander -> "commander"
  | `Grovewarden -> "grovewarden"
  | `Lich -> "lich"
  | `Marshal -> "marshal"
  | `Necromancer -> "necromancer"
  | `Tidekeeper -> "tidekeeper"
  | `Tideseer -> "tideseer"
  | `Wildkeeper -> "wildkeeper"
  | `Grand_marshal -> "grand-marshal"
  | `Lich_commander_l3 -> "lich-commander-l3"
  | `Woodland_grove_oracle -> "woodland-grove-oracle"
  | `Merfolk_tide_sovereign -> "merfolk-tide-sovereign"
  | `Adept -> "adept"
  | `Archer -> "archer"
  | `Boar -> "boar"
  | `Bonearcher -> "bonearcher"
  | `Ghost -> "ghost"
  | `Harpooner -> "harpooner"
  | `Merfolk_maceguard -> "merfolk-maceguard"
  | `Reefguard -> "reefguard"
  | `Rider -> "rider"
  | `Scout -> "scout"
  | `Skeleton_impact_caster -> "skeleton-impact-caster"
  | `Skeleton -> "skeleton"
  | `Spearman -> "spearman"
  | `Swordsman -> "swordsman"
  | `Thorncaller -> "thorncaller"
  | `Wavemender -> "wavemender"
  | `Wolf -> "wolf"
  | `Horse_archer -> "horse-archer"
  | `Skeleton_rider -> "skeleton-rider"
  | `Skeleton_axeman -> "skeleton-axeman"
  | `Fox_spearguard -> "fox-spearguard"
  | `Treant -> "treant"
  | `Worker -> "worker"

let item (value : Javascript1.items) : V2.item =
  {
    wire = value.wire;
    id = value.id;
    armor = value.armor;
    name = value.name;
    description = value.description;
    healing = value.healing;
    attacks =
      Array.map
        (fun (attack : Javascript1.attacks) : V2.attacks ->
          {
            wire = attack.wire;
            id = attack.id;
            name = attack.name;
            damage = attack.damage;
            spread = attack.spread;
            count = Some attack.count;
            damage_type = attack.damage_type;
            category = attack.category;
            range = attack.range;
            sound = attack.sound;
            drain = attack.drain;
          })
        value.attacks;
  }

let unit_ (value : Javascript1.unit_) : V2.unit_ =
  {
    wire =
      {
        value.wire with
        order =
          (value.wire.order
          @ if List.mem "experience" value.wire.order then [] else [ "experience" ]);
      };
    tile = value.tile;
    id = value.id;
    side = value.side;
    role = value.role;
    faction = value.faction;
    items = Option.map (Array.map item) value.items;
    ai_group = value.ai_group;
    type_ = unit_type value.type_;
    hp = value.hp;
    moved = value.moved;
    attacked = value.attacked;
    retreating = value.retreating;
    rebuilding = value.rebuilding;
    name = value.name;
    character = value.character;
    experience = Some (Option.value value.experience ~default:0.);
  }

let veteran (value : Javascript1.veteran) : V2.veteran =
  let promoted decode key =
    match List.assoc_opt key value.wire.extra with
    | None -> None
    | Some value when Migration_json.is_undefined value -> None
    | Some value -> Some (decode value)
  in
  let items =
    promoted Historical_codec.read_fragments_native_state38_units_item_items_member1 "items"
  in
  let character = promoted Historical_codec.read_json_string "character" in
  {
    wire = value.wire;
    type_ = unit_type value.type_;
    experience = value.experience;
    name = value.name;
    role = value.role;
    items;
    character;
  }

let wire order = { empty_wire with order }

let counter turns round : fragments_native_state38_campaign_member1_vigil_member1 =
  {
    wire = wire [ "heldTurns"; "lastCountedRound" ];
    held_turns = turns;
    last_counted_round = round;
  }

let campaign_state catalog (value : Javascript1.campaign) map_id revision winner round named :
    V2.campaign =
  let id = campaign_id value.id in
  let definition = find_id id (fun (value : D.campaign) -> value.id) catalog.D.campaigns in
  require (integer value.mission);
  let mission_index = int_of_float value.mission in
  require (mission_index < Array.length definition.missions);
  let mission =
    if map_id = "lantern-ferry" && revision = 1. then Lazy.force D.ferry_mission
    else definition.missions.(mission_index)
  in
  require
    (mission.map.id = map_id
    && Array.length value.recall <= 4
    && integer value.carry_gold && value.carry_gold <= 8.);
  let rule id = find_id id (fun (value : D.rule) -> value.id) catalog.D.rules in
  let previous =
    definition.missions |> Array.to_list
    |> List.filteri (fun index _ -> index < mission_index)
    |> List.concat_map (fun (mission : D.mission) -> Array.to_list mission.map.units)
    |> List.filter_map (fun (unit_ : D.placement) ->
        if unit_.side = `Player && unit_.role <> Some `Leader then unit_.name else None)
    |> Array.of_list
  in
  Option.iter
    (fun names -> require (unique names && Array.for_all (fun name -> member name previous) names))
    value.fallen_companions;
  let validate_veteran (value : Javascript1.veteran) =
    require
      (integer value.experience
      && value.experience <= (rule (unit_type value.type_)).experience_needed);
    Option.iter (fun name -> require (length name <= 80)) value.name;
    Option.iter (function `Leader | `Courier -> () | _ -> invalid ()) value.role
  in
  Option.iter
    (fun entries ->
      require (Array.length entries = Array.length value.recall);
      Array.iteri
        (fun index entry ->
          validate_veteran entry;
          require (entry.type_ = value.recall.(index) && entry.role = None))
        entries)
    value.veterans;
  Option.iter
    (fun entries ->
      require (Array.length entries <= 8);
      Array.iter
        (fun (entry : Javascript1.veteran) ->
          validate_veteran entry;
          require
            (Array.exists
               (fun (incoming : D.placement) ->
                 incoming.side = `Player
                 && (if entry.role = Some `Leader then incoming.role = Some `Leader
                     else
                       Option.fold ~none:false
                         ~some:(fun name -> name <> "" && incoming.name = Some name)
                         entry.name)
                 && ((rule incoming.type_).family = (rule (unit_type entry.type_)).family
                    || (map_id = "lantern-ferry" || map_id = "lantern-villages")
                       && (entry.role = Some `Leader
                          || entry.name = Some (Lazy.force D.ferry_leader_name))
                       && Option.fold ~none:false
                            ~some:(fun projected ->
                              (rule projected).family = (rule incoming.type_).family)
                            (D.leader_projection (unit_type entry.type_))))
               mission.map.units))
        entries)
    value.story_veterans;
  let allowed =
    concat
      [
        Array.map (fun name -> "fallen:" ^ name) previous;
        mission.map.units |> Array.to_list
        |> List.filter_map (fun (unit_ : D.placement) ->
            if unit_.side = `Player then
              Option.bind unit_.name (fun name ->
                  if name = "" then None else Some ("fallen:" ^ name))
            else None)
        |> Array.of_list;
        (if id = "green-accord" then [| "dain-fell" |] else [||]);
        (if mission.refuge <> None then [| "held:5,4" |] else [||]);
        Array.map (fun (event : D.event) -> "event:" ^ event.id) mission.events;
        Array.map (fun (wave : D.wave) -> "wave:" ^ wave.id) mission.waves;
        (if named then [||]
         else
           mission.waves |> Array.to_list
           |> List.filter_map (fun (wave : D.wave) ->
               Option.map
                 (fun index -> "wave:" ^ string_of_int (int_of_float index))
                 wave.legacy_index)
           |> Array.of_list);
        mission.goals |> Array.to_list
        |> List.concat_map (function
          | D.Seal ids -> Array.to_list (Array.map (fun id -> "sealed:" ^ id) ids)
          | D.Hold { tile; _ } | D.Vigil { tile; _ } -> [ "held:" ^ tile ]
          | _ -> [])
        |> Array.of_list;
      ]
  in
  require (unique value.marks && Array.for_all (fun mark -> member mark allowed) value.marks);
  let marks =
    ref (Array.map (fun mark -> if named then mark else wave_marker mark mission.waves) value.marks)
  in
  require (unique !marks);
  let hold_goals =
    mission.goals |> Array.to_list
    |> List.filter_map (function
      | D.Hold value -> Some (value.tile, value.turns, value.start_event)
      | _ -> None)
  in
  let vigil_goal =
    mission.goals |> Array.to_list
    |> List.find_map (function D.Vigil value -> Some value.turns | _ -> None)
  in
  let validate_counter (value : fragments_native_state38_campaign_member1_vigil_member1) turns =
    require
      (integer value.held_turns && value.held_turns <= turns && integer value.last_counted_round
     && value.last_counted_round <= round)
  in
  Option.iter
    (fun holds ->
      Map.iter
        (fun tile progress ->
          match List.find_opt (fun (id, _, _) -> id = tile) hold_goals with
          | None -> invalid ()
          | Some (_, turns, _) -> validate_counter progress turns)
        holds)
    value.holds;
  if winner = `Player then
    List.iter
      (fun (_, _, event) ->
        Option.iter
          (fun event ->
            if event <> "" && not (member ("event:" ^ event) !marks) then
              marks := Array.append !marks [| "event:" ^ event |])
          event)
      hold_goals;
  Option.iter
    (fun progress ->
      match vigil_goal with None -> invalid () | Some turns -> validate_counter progress turns)
    value.vigil;
  Option.iter
    (fun (progress : Javascript1.refuge) ->
      require
        (mission.refuge <> None && integer progress.held_turns && progress.held_turns <= 6.
        && integer progress.last_counted_round))
    value.refuge;
  Option.iter
    (fun choices ->
      require
        (Array.length choices <= 1
        && Array.for_all
             (fun choice ->
               if id = "ashen-oath" then
                 mission_index >= 1 && (choice = "refuge-peace" || choice = "refuge-war")
               else id = "green-accord" && choice = "green-dain-dead")
             choices))
    value.choices;
  let failure =
    Array.append
      [|
        "Your leader fell. The company cannot continue without them.";
        "The courier fell before reaching safety.";
      |]
      (Array.map
         (fun name -> name ^ " fell. The company cannot continue without them.")
         mission.required_companions)
  in
  let known =
    concat
      [
        mission.intro;
        concat (Array.to_list (Array.map (fun (event : D.event) -> event.dialogue) mission.events));
        concat
          (Array.to_list (Array.map (fun (event : D.event) -> event.war_dialogue) mission.events));
        mission.victory;
        Option.fold ~none:[||] ~some:(fun (refuge : D.refuge) -> refuge.escalation) mission.refuge;
        Option.fold ~none:[||] ~some:(fun (refuge : D.refuge) -> refuge.war_victory) mission.refuge;
        Option.fold ~none:[||]
          ~some:(fun (patch : D.dialogue_patch) -> patch.intro)
          mission.without_dain;
        Option.fold ~none:[||]
          ~some:(fun (patch : D.dialogue_patch) -> patch.victory)
          mission.without_dain;
        concat
          (Array.to_list
             (Array.map
                (fun (patch : D.dialogue_patch) -> Array.append patch.intro patch.victory)
                mission.companion_losses));
        mission.legacy_dialogue;
        definition.ending;
        Array.map
          (fun text : V2.dialogue ->
            {
              wire = empty_wire;
              speaker = Some "The company";
              portrait = definition.leader;
              text;
              character = None;
              sound = None;
            })
          failure;
      ]
  in
  require (Array.length value.dialogue <= Array.length known);
  let dialogue =
    Array.map
      (fun (line : Javascript1.dialogue) ->
        let matches (known : V2.dialogue) =
          known.speaker = Some line.speaker && known.text = line.text
        in
        require
          (Array.exists
             (fun (known : V2.dialogue) ->
               matches known
               && (known.portrait = portrait line.portrait
                  || (known.portrait = "worker" && portrait line.portrait = "scout")))
             known);
        let original = Option.get (find matches known) in
        ({
           wire = wire [ "speaker"; "portrait"; "text" ];
           speaker = Some line.speaker;
           portrait = original.portrait;
           text = line.text;
           sound = None;
           character = None;
         }
          : V2.dialogue))
      value.dialogue
  in
  require
    (integer value.dialogue_index
    && value.dialogue_index <= float_of_int (Array.length dialogue)
    && value.completed = (winner = `Player && mission_index = Array.length definition.missions - 1)
    );
  require
    (match (winner, value.outcome) with
    | `Null, `Null -> true
    | `Player, `String outcome -> outcome = "Mission complete. The company can rest and regroup."
    | `Enemy, `String outcome -> member outcome failure
    | _ -> false);
  let holds =
    Map.of_list
      (List.map
         (fun (tile, turns, event) ->
           let previous = Option.bind value.holds (Map.find_opt tile) in
           let started =
             Option.fold ~none:true
               ~some:(fun event -> event = "" || member ("event:" ^ event) !marks)
               event
           in
           ( tile,
             if started && previous <> None then Option.get previous
             else counter (if winner = `Player then turns else 0.) round ))
         hold_goals)
  in
  {
    wire =
      wire
        [
          "id";
          "mission";
          "recall";
          "veterans";
          "storyVeterans";
          "fallenCompanions";
          "carryGold";
          "marks";
          "dialogue";
          "dialogueIndex";
          "outcome";
          "completed";
          "refuge";
          "vigil";
          "holds";
          "choices";
        ];
    id;
    mission = value.mission;
    recall = Array.map unit_type value.recall;
    veterans = Option.map (Array.map veteran) value.veterans;
    story_veterans = Option.map (Array.map veteran) value.story_veterans;
    fallen_companions = value.fallen_companions;
    carry_gold = value.carry_gold;
    marks = !marks;
    dialogue;
    dialogue_index = value.dialogue_index;
    outcome = value.outcome;
    completed = value.completed;
    refuge = value.refuge;
    vigil = Option.map (fun _ -> Option.value value.vigil ~default:(counter 0. round)) vigil_goal;
    holds = Some holds;
    choices = value.choices;
    evacuated = None;
  }

let import (saved : Javascript1.t) : V2.state =
  let catalog = Lazy.force D.catalog in
  let source = saved.state in
  let map_id = Option.value source.map_id ~default:"mirror" in
  let current = find_id map_id (fun (value : D.map) -> value.id) catalog.maps in
  let revision =
    match source.map_revision with
    | None | Some `Null | Some `Undefined -> 1.
    | Some (`Number value) -> value
  in
  require (revision = 1. || revision = current.revision);
  let layout =
    if revision <> 1. then current
    else
      match map_id with
      | "ashen-refuge" -> Lazy.force D.ashen_refuge
      | "lantern-ferry" -> Lazy.force D.ferry_source
      | _ -> current
  in
  let cols = layout.cols and rows = layout.rows in
  let tiles : V2.tile array =
    Array.init
      (int_of_float (cols *. rows))
      (fun index ->
        let col = float_of_int (index mod int_of_float cols)
        and row = float_of_int (index / int_of_float cols) in
        ({
           wire = wire [ "id"; "col"; "row"; "terrain"; "owner" ];
           id = tile_id col row;
           col;
           row;
           terrain = "plains";
           owner = `Null;
           location = None;
           connections = None;
           dead_ends = None;
           walls = None;
           floor = None;
           fog = None;
           light = None;
           wall_corners = None;
         }
          : V2.tile))
  in
  let indices =
    Array.mapi (fun index (tile : V2.tile) -> (tile.id, index)) tiles
    |> Array.to_list |> Map.of_list
  in
  let tile id =
    match Map.find_opt id indices with None -> invalid () | Some index -> tiles.(index)
  in
  let update id f =
    match Map.find_opt id indices with
    | None -> invalid ()
    | Some index -> tiles.(index) <- f tiles.(index)
  in
  Array.iter
    (fun (authored : D.terrain) ->
      update (tile_id authored.col authored.row) (fun (tile : V2.tile) ->
          { tile with terrain = authored.terrain; owner = authored.owner }))
    layout.terrain;
  let rule id = find_id id (fun (value : D.rule) -> value.id) catalog.rules in
  let cost unit_type (tile : V2.tile) =
    let index = ref None in
    Array.iteri
      (fun i terrain -> if terrain = tile.terrain then index := Some i)
      catalog.terrain_order;
    match !index with
    | None -> false
    | Some index ->
        let costs = (rule unit_type).costs in
        index < Array.length costs && costs.(index) <> None
  in
  require
    (integer source.round && source.round >= 1.
    && (source.turn = `Player || not (no_winner source.winner))
    && ((not saved.pending_enemy) || (source.turn = `Player && no_winner source.winner))
    && integer source.gold.player && integer source.gold.enemy && integer source.next_id
    && source.next_id >= 1.
    && Array.length source.log <= 8
    && Array.for_all (fun line -> length line <= 1000) source.log);
  let definition =
    find
      (fun (definition : D.campaign) ->
        Array.exists (fun (mission : D.mission) -> mission.map.id = map_id) definition.missions)
      catalog.campaigns
  in
  let campaign =
    match (definition, source.campaign) with
    | None, None -> None
    | Some _, Some value ->
        Some
          (campaign_state catalog value map_id revision source.winner source.round
             (saved.campaign_deployment_format <> None))
    | _ -> invalid ()
  in
  let ids = ref Map.empty and occupied = ref Map.empty in
  let units =
    Array.map
      (fun (value : Javascript1.unit_) ->
        let kind = unit_type value.type_ in
        require (Js.Re.test ~str:(Migration_utf8.to_js value.id) (Js.Re.fromString "^u[1-9]\\d*$"));
        require
          (float_of_string (String.sub value.id 1 (String.length value.id - 1)) < source.next_id
          && (not (Map.mem value.id !ids))
          && cost kind (tile value.tile)
          && (not (Map.mem value.tile !occupied))
          && integer value.hp && value.hp >= 1. && value.hp <= (rule kind).hp);
        Option.iter
          (fun experience ->
            require (integer experience && experience <= (rule kind).experience_needed))
          value.experience;
        Option.iter (fun name -> require (length name <= 80)) value.name;
        Option.iter (function `Leader | `Courier -> () | _ -> invalid ()) value.role;
        Option.iter (function `True | `False -> () | `Undefined -> invalid ()) value.rebuilding;
        Option.iter
          (function `True | `False -> require (value.side = `Enemy) | `Undefined -> invalid ())
          value.retreating;
        ids := Map.add value.id () !ids;
        occupied := Map.add value.tile () !occupied;
        unit_ value)
      source.units
  in
  let units = ref units
  and campaign = ref campaign
  and revision = ref revision
  and next_id = ref source.next_id
  and winner = ref source.winner
  and log = ref source.log in
  (match (!campaign, definition) with
  | Some state, Some definition when state.id = "borrowed-tide" && state.mission < 2. ->
      let old_name = if state.mission = 0. then "Captain Vale" else "Factor Senn" in
      let mission = definition.missions.(int_of_float state.mission) in
      let officer =
        Option.get
          (find
             (fun (unit_ : D.placement) -> unit_.side = `Enemy && unit_.role = Some `Leader)
             mission.map.units)
      in
      units :=
        Array.map
          (fun (unit_ : V2.unit_) ->
            if unit_.side = `Enemy && unit_.role = Some `Leader && unit_.name = Some old_name then
              { unit_ with name = officer.name }
            else unit_)
          !units;
      Option.iter
        (fun current_line ->
          campaign :=
            Some
              {
                state with
                dialogue =
                  Array.map
                    (fun (line : V2.dialogue) ->
                      if line.speaker = Some old_name then current_line else line)
                    state.dialogue;
              })
        (find (fun (line : V2.dialogue) -> line.speaker = officer.name) mission.intro)
  | _ -> ());
  let villages = filter (fun (tile : V2.tile) -> tile.terrain = "village") tiles in
  require
    (Array.length source.villages = Array.length villages
    && unique (Array.map (fun (value : Javascript1.villages) -> value.id) source.villages));
  let set_owners saved places =
    Array.iter
      (fun (value : Javascript1.villages) ->
        require
          (Array.exists (fun (tile : V2.tile) -> tile.id = value.id) places
          && valid_owner value.owner);
        update value.id (fun (tile : V2.tile) -> { tile with owner = value.owner }))
      saved
  in
  set_owners source.villages villages;
  if map_id = "ashen-refuge" && !revision = 1. then (
    List.iter
      (fun row ->
        let old = tile ("5," ^ row) and fresh = tile ("3," ^ row) in
        let owner =
          Option.fold ~none:old.owner
            ~some:(fun (unit_ : V2.unit_) -> as_owner unit_.side)
            (find (fun (unit_ : V2.unit_) -> unit_.tile = fresh.id) !units)
        in
        update fresh.id (fun tile -> { tile with terrain = "village"; owner });
        update old.id (fun tile -> { tile with terrain = "plains"; owner = `Null }))
      [ "1"; "7" ];
    revision := 2.);
  let sites =
    filter (fun (tile : V2.tile) -> member tile.terrain [| "grove"; "ruins"; "fort" |]) tiles
  in
  if !campaign <> None || source.sites <> None then (
    let saved_sites = match source.sites with None -> invalid () | Some sites -> sites in
    let saved_sites =
      if map_id = "lantern-bell" && !revision = 1. then
        Array.map
          (fun (site : Javascript1.sites) ->
            if site.id <> "4,3" then site
            else
              {
                site with
                id = "5,1";
                owner =
                  Option.fold ~none:site.owner
                    ~some:(fun (unit_ : V2.unit_) -> as_owner unit_.side)
                    (find (fun (unit_ : V2.unit_) -> unit_.tile = "5,1") !units);
              })
          saved_sites
      else saved_sites
    in
    require
      (Array.length saved_sites = Array.length sites
      && unique (Array.map (fun (value : Javascript1.sites) -> value.id) saved_sites));
    set_owners saved_sites sites;
    if map_id = "lantern-bell" && !revision = 1. then revision := 2.)
  else
    Array.iter
      (fun (site : V2.tile) -> update site.id (fun tile -> { tile with owner = `Null }))
      sites;
  (match (!campaign, definition) with
  | Some state, Some definition ->
      List.iter
        (fun side ->
          require
            (Array.length
               (filter
                  (fun (unit_ : V2.unit_) -> unit_.side = side && unit_.role = Some `Leader)
                  !units)
            <= 1))
        [ `Player; `Enemy ];
      if no_winner source.winner then (
        require
          (Array.exists
             (fun (unit_ : V2.unit_) -> unit_.side = `Player && unit_.role = Some `Leader)
             !units);
        if
          Array.exists
            (function D.Escort -> true | _ -> false)
            definition.missions.(int_of_float state.mission).goals
        then
          require
            (Array.exists
               (fun (unit_ : V2.unit_) -> unit_.side = `Player && unit_.role = Some `Courier)
               !units))
  | _ ->
      if no_winner source.winner then
        List.iter
          (fun side ->
            require
              (Array.exists
                 (fun (unit_ : V2.unit_) ->
                   unit_.side = side && (rule unit_.type_).family = "commander")
                 !units))
          [ `Player; `Enemy ]);
  (match (!campaign, definition) with
  | Some state, Some definition when state.id = "green-accord" ->
      let state = ref state in
      let has_dain () =
        Array.exists (fun (unit_ : V2.unit_) -> unit_.name = Some "Captain Dain") !units
      in
      let dead = Option.fold ~none:false ~some:(member "green-dain-dead") !state.choices in
      let add_mark mark =
        if not (member mark !state.marks) then
          state := { !state with marks = Array.append !state.marks [| mark |] }
      in
      if
        (!state.mission <> 1. || !revision >= 2.)
        && (not dead)
        && (not (has_dain ()))
        && not (member "dain-fell" !state.marks)
      then add_mark "dain-fell";
      if !state.mission = 1. && !revision = 1. then (
        (if (not dead) && (not (member "dain-fell" !state.marks)) && not (has_dain ()) then
           let origin = tile "5,5" in
           let distance (tile : V2.tile) =
             let q = tile.col -. floor (tile.row /. 2.) -. origin.col +. floor (origin.row /. 2.) in
             let r = tile.row -. origin.row in
             (abs_float q +. abs_float r +. abs_float (q +. r)) /. 2.
           in
           let destinations =
             tiles |> Array.to_list
             |> List.filter (fun (tile : V2.tile) ->
                 cost "warden" tile
                 && not (Array.exists (fun (unit_ : V2.unit_) -> unit_.tile = tile.id) !units))
             |> List.stable_sort (fun first second ->
                 Float.compare (distance first) (distance second))
           in
           match destinations with
           | [] -> ()
           | destination :: _ ->
               let unit_ : V2.unit_ =
                 {
                   wire =
                     wire
                       [
                         "id";
                         "type";
                         "side";
                         "tile";
                         "hp";
                         "moved";
                         "attacked";
                         "experience";
                         "name";
                       ];
                   id = "u" ^ string_of_int (int_of_float !next_id);
                   type_ = "warden";
                   side = `Player;
                   tile = destination.id;
                   hp = (rule "warden").hp;
                   moved = false;
                   attacked = false;
                   experience = Some 0.;
                   name = Some "Captain Dain";
                   role = None;
                   faction = None;
                   items = None;
                   ai_group = None;
                   retreating = None;
                   rebuilding = None;
                   character = None;
                 }
               in
               next_id := !next_id +. 1.;
               units := Array.append !units [| unit_ |]);
        revision := 2.);
      let mission = definition.missions.(int_of_float !state.mission) in
      let missing =
        find
          (fun name ->
            not
              (Array.exists
                 (fun (unit_ : V2.unit_) -> unit_.side = `Player && unit_.name = Some name)
                 !units))
          mission.required_companions
      in
      (match (!winner, missing) with
      | `Player, Some name ->
          winner := `Enemy;
          let outcome = name ^ " fell. The company cannot continue without them." in
          let line : V2.dialogue =
            {
              wire = wire [ "speaker"; "portrait"; "text" ];
              speaker = Some "The company";
              portrait = definition.leader;
              text = outcome;
              sound = None;
              character = None;
            }
          in
          state :=
            {
              !state with
              completed = false;
              outcome = `String outcome;
              dialogue_index = float_of_int (Array.length !state.dialogue);
              dialogue = Array.append !state.dialogue [| line |];
            };
          log :=
            Array.of_list (List.filteri (fun index _ -> index < 8) (outcome :: Array.to_list !log))
      | _ -> ());
      campaign := Some !state
  | _ -> ());
  (match (!campaign, definition) with
  | Some state, Some definition ->
      let state = ref state and migrated = ref false in
      Array.iter
        (fun (wave : D.wave) ->
          match wave.previously_deployed_before_revision with
          | Some before when !revision < before ->
              let mark = "wave:" ^ wave.id in
              if not (member mark !state.marks) then
                state := { !state with marks = Array.append !state.marks [| mark |] };
              migrated := true
          | _ -> ())
        definition.missions.(int_of_float !state.mission).waves;
      if !migrated then revision := definition.missions.(int_of_float !state.mission).map.revision;
      campaign := Some !state
  | _ -> ());
  {
    wire =
      wire
        [
          "mapId";
          "mapRevision";
          "cols";
          "rows";
          "round";
          "turn";
          "gold";
          "winner";
          "nextId";
          "log";
          "units";
          "tiles";
          "campaign";
        ];
    map_id;
    map_revision = !revision;
    cols;
    rows;
    round = source.round;
    turn = source.turn;
    gold = source.gold;
    winner = !winner;
    next_id = !next_id;
    log = !log;
    units = !units;
    tiles;
    campaign = !campaign;
    armies = None;
    condition_memory = None;
    last_combat = None;
    factions = None;
    ground_items = None;
    combat_rng = None;
  }
