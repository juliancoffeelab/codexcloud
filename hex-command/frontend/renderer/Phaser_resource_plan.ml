type kind = Image | Audio

type resource = {
  key : string;
  url : string;
  kind : kind;
  priority : int;
  hud : bool;
  width : float option;
  height : float option;
}

module Map = Host_collections.Map
module Set = Host_collections.Set

let skipped =
  [
    {js|recruitment|js};
    {js|recruits|js};
    {js|unlockRecruits|js};
    {js|advancesTo|js};
    {js|recall|js};
  ]

let content_types ?(include_recruitment = false) catalog walk content =
  let found = Set.create () in
  let known = Set.create () in
  List.iter
    (fun (rule : Domain.unit_rule) ->
      let _ = Set.add known rule.id in
      ())
    (Catalog.units catalog);
  let rec visitor =
    {
      Domain_resource_walk.string =
        (fun value ->
          if Set.has known value then
            let _ = Set.add found value in
            ());
      field =
        (fun name visit -> if include_recruitment || not (List.mem name skipped) then visit visitor);
      terrain_change = (fun _ -> ());
    }
  in
  walk visitor content;
  found

let item_frame artwork id =
  match Art.frame artwork ~kind:{js|items|js} ~name:id () with
  | Some frame -> frame
  | None -> failwith {j|Missing item artwork: $(id)|j}

let current_map_audio game catalog =
  let state = Native.state game in
  let families = Set.create () in
  Array.iter
    (fun cue ->
      let _ = Set.add families cue in
      ())
    Phaser_sound.action_families;
  Array.iter
    (fun cue ->
      let _ = Set.add families cue in
      ())
    (Phaser_sound.battle_sounds
       (Array.map (fun (unit : Domain.public_unit) -> unit.type_) (Native.units game))
       (Array.map (fun (tile : Domain.tile) -> tile.terrain) state.tiles));
  Array.iter
    (fun (unit : Domain.public_unit) ->
      List.iter
        (fun (weapon : Domain.weapon) ->
          let family =
            Phaser_sound.sound_for_interaction
              {
                Phaser_sound.interaction with
                attack_type = Some unit.type_;
                attack_id = Some weapon.id;
                attack_sound = Some weapon.sound;
                attack_category = Some (Enum.string_of_category weapon.category);
                attack_damage_type = Some (Enum.string_of_damage_type weapon.damage_type);
              }
          in
          let _ = Set.add families family in
          ())
        (Presentation.weapons catalog unit))
    (Native.units game);
  Array_ext.filter
    (fun (sample : Phaser_sound.sample) -> Set.has families sample.family)
    Phaser_sound.samples

let texture map_id revision =
  match Terrain_pages.find (Terrain_pages.asset_key map_id revision) with
  | Some texture -> texture
  | None -> failwith {j|Missing prebaked terrain: $(map_id). Rebuild map assets.|j}

let initial_terrain catalog map_id revision artwork =
  let map =
    Array.find_opt
      (fun (map : Domain.map_definition) -> map.id = map_id && map.revision = revision)
      (Catalog.raw catalog).maps
  in
  let map =
    match map with Some map -> map | None -> failwith {j|Missing authored map: $(map_id)|j}
  in
  (Terrain_plan.prepare ~roles:(Catalog.terrain_roles catalog) ~map_id
     ~cols:(Int32.to_float map.cols) ~rows:(Int32.to_float map.rows) map.terrain artwork)
    .tiles

let prepare ~game ~catalog ~artwork =
  let state = Native.state game in
  let resources = Map.create () in
  let add (resource : resource) =
    let resource =
      match Map.get resources resource.key with
      | None -> resource
      | Some old ->
          {
            resource with
            priority = min old.priority resource.priority;
            hud = old.hud || resource.hud;
          }
    in
    let _ = Map.set resources resource.key resource in
    ()
  in
  let add_frame ?(hud = false) priority (frame : Art.frame) =
    add
      {
        key = frame.url;
        url = frame.url;
        kind = Image;
        priority;
        hud;
        width = Some frame.width;
        height = Some frame.height;
      }
  in
  let image key priority =
    match Art.find artwork key with
    | Some frame -> add_frame ~hud:true priority frame
    | None -> failwith {j|Missing artwork: $(key)|j}
  in
  let add_sample priority (sample : Phaser_sound.sample) =
    add
      {
        key = sample.key;
        url = sample.url;
        kind = Audio;
        priority;
        hud = false;
        width = None;
        height = None;
      }
  in
  let audio cues priority =
    let wanted = Set.create () in
    Array.iter
      (fun cue ->
        let _ = Set.add wanted cue in
        ())
      cues;
    Array.iter
      (fun (sample : Phaser_sound.sample) ->
        if Set.has wanted sample.family then add_sample priority sample)
      Phaser_sound.samples
  in
  let palettes = Set.create () in
  let _ = Set.add palettes {js|player|js} in
  Array.iter
    (fun (faction : Domain.faction) ->
      let _ = Set.add palettes faction.palette in
      ())
    state.factions;
  let units ?(sides = palettes) types priority =
    Set.each types (fun[@u] kind ->
        Set.each sides (fun[@u] palette ->
            let key = Art.frame_key ~kind:{js|units|js} ~name:kind ~side:palette () in
            if Option.is_some (Art.find artwork key) then image key priority));
    audio (Phaser_sound.battle_sounds (Set.values_array types) [||]) priority
  in
  let campaign =
    Option.map
      (fun (progress : Domain.campaign_state) -> (Catalog.campaign catalog progress.id, progress))
      state.campaign
  in
  let mission =
    Option.bind campaign (fun ((campaign : Domain.campaign), progress) ->
        let index = int_of_float progress.mission in
        if index >= 0 && index < Array.length campaign.missions then Some campaign.missions.(index)
        else None)
  in
  let prepare_terrain tiles map_id walk content priority =
    let changes = [||] in
    let rec visitor =
      {
        Domain_resource_walk.string = (fun _ -> ());
        field = (fun _ visit -> visit visitor);
        terrain_change =
          (fun change ->
            let _ = Array_ext.append_value changes change in
            ());
      }
    in
    walk visitor content;
    if Array.length changes > 0 then (
      let frame tile =
        let plan =
          Terrain_plan.prepare ~roles:(Catalog.terrain_roles catalog) ~map_id
            ~cols:(Int32.to_float state.cols) ~rows:(Int32.to_float state.rows) [| tile |] artwork
        in
        match plan.tiles.(0).frame with
        | Some frame -> add_frame priority frame
        | None ->
            failwith
              (let js_text_part_3 = tile.Domain.id in
               {j|Missing terrain artwork: $(map_id)/$(js_text_part_3)|j})
      in
      Array.iter frame tiles;
      Array.iter
        (fun (change : Domain.terrain_change) ->
          let literal =
            Array.find_opt
              (fun (tile : Domain.tile) ->
                match change.tile with
                | Domain.Tile_ref.Hex hex -> tile.col = hex.col && tile.row = hex.row
                | Domain.Tile_ref.Location location -> tile.location = Some location
                | Domain.Tile_ref.Binding _ -> false)
              tiles
          in
          let changed = match literal with Some tile -> [| tile |] | None -> tiles in
          Array.iter
            (fun (tile : Domain.tile) ->
              frame
                {
                  tile with
                  terrain = change.terrain;
                  connections = None;
                  walls = None;
                  dead_ends = None;
                })
            changed)
        changes)
  in
  Array.iter (add_sample 1) (current_map_audio game catalog);
  Array.iter
    (fun (page : Terrain_pages.page) -> add_frame 1 (Terrain_pages.frame page))
    (texture state.map_id state.map_revision).pages;
  Array.iter
    (fun (unit : Domain.public_unit) ->
      Array.iter
        (fun (item : Domain.item) -> add_frame ~hud:true 1 (item_frame artwork item.id))
        unit.items)
    (Native.units game);
  image {js|portrait_worker|js} 1;
  let current_map = Catalog.map catalog state.map_id in
  let current_types = content_types catalog Domain_resource_walk.walk_map_definition current_map in
  Array.iter
    (fun (unit : Domain.public_unit) ->
      let _ = Set.add current_types unit.type_ in
      ())
    (Native.units game);
  units current_types 1;
  let terrains = Set.create () in
  Array.iter
    (fun (tile : Domain.tile) ->
      let _ = Set.add terrains tile.terrain in
      ())
    state.tiles;
  Set.each terrains (fun[@u] terrain ->
      match Art.terrain_portrait artwork terrain with
      | Some frame -> add_frame ~hud:true 1 frame
      | None -> failwith {j|Missing terrain portrait: $(terrain)|j});
  audio
    (Array.append Phaser_sound.action_families
       (Array.map
          (fun terrain ->
            Phaser_sound.sound_for_interaction
              { Phaser_sound.interaction with terrain = Some terrain })
          (Set.values_array terrains)))
    1;
  Array.iter
    (fun (item : Domain.item) -> add_frame ~hud:true 1 (item_frame artwork item.id))
    (Catalog.raw catalog).items;
  Option.iter
    (fun (mission : Domain.mission) ->
      let scripted = content_types catalog Domain_resource_walk.walk_mission mission in
      Set.each scripted (fun[@u] kind ->
          let _ = Set.add current_types kind in
          ());
      units scripted 1;
      let ending =
        match campaign with
        | Some (campaign, progress)
          when int_of_float progress.mission = Array.length campaign.missions - 1 ->
            campaign.ending
        | _ -> [||]
      in
      audio (Campaign_sound.mission mission ending) 1)
    mission;
  Option.iter
    (fun (_, (progress : Domain.campaign_state)) ->
      Option.iter
        (fun veterans ->
          let types = Set.create () in
          Array.iter
            (fun veteran ->
              let found = content_types catalog Domain_resource_walk.walk_veteran veteran in
              Set.each found (fun[@u] kind ->
                  let _ = Set.add types kind in
                  ()))
            veterans;
          units types 1)
        progress.veterans;
      Option.iter
        (fun veterans ->
          let types = Set.create () in
          Array.iter
            (fun veteran ->
              let found = content_types catalog Domain_resource_walk.walk_veteran veteran in
              Set.each found (fun[@u] kind ->
                  let _ = Set.add types kind in
                  ()))
            veterans;
          units types 1)
        progress.story_veterans)
    campaign;
  let recruits = Set.create () in
  Array.iter
    (fun (faction : Domain.faction) ->
      Array.iter
        (fun kind ->
          let _ = Set.add recruits kind in
          ())
        (Native.recruitment_options game faction.id))
    state.factions;
  Option.iter
    (fun mission ->
      let rec collector =
        {
          Domain_resource_walk.string = (fun _ -> ());
          terrain_change = (fun _ -> ());
          field =
            (fun name visit ->
              if List.mem name [ {js|recruitment|js}; {js|recruits|js}; {js|unlockRecruits|js} ]
              then
                let rec inside =
                  {
                    Domain_resource_walk.string =
                      (fun value ->
                        if
                          List.exists
                            (fun (rule : Domain.unit_rule) -> rule.id = value)
                            (Catalog.units catalog)
                        then
                          let _ = Set.add recruits value in
                          ());
                    terrain_change = (fun _ -> ());
                    field = (fun name visit -> if not (List.mem name skipped) then visit inside);
                  }
                in
                visit inside
              else visit collector);
        }
      in
      Domain_resource_walk.walk_mission collector mission)
    mission;
  units recruits 2;
  let promotions = Set.create () in
  Set.each current_types (fun[@u] kind ->
      let _ = Set.add promotions kind in
      ());
  Set.each recruits (fun[@u] kind ->
      let _ = Set.add promotions kind in
      ());
  Set.each promotions (fun[@u] kind ->
      Array.iter
        (fun next ->
          let _ = Set.add promotions next in
          ())
        (Catalog.unit catalog kind).advances_to);
  units promotions 3;
  (match campaign with
  | Some (campaign, progress)
    when int_of_float progress.mission + 1 < Array.length campaign.missions ->
      let next = campaign.missions.(int_of_float progress.mission + 1) in
      let map = Catalog.mission_map catalog next in
      Array.iter
        (fun (page : Terrain_pages.page) -> add_frame 4 (Terrain_pages.frame page))
        (texture map.id map.revision).pages;
      prepare_terrain map.terrain map.id Domain_resource_walk.walk_mission next 4;
      let next_palettes = Set.create () in
      Set.each palettes (fun[@u] palette ->
          let _ = Set.add next_palettes palette in
          ());
      Array.iter
        (fun key ->
          match Array.to_list (Js.String.split ~sep:{js|_|js} key) with
          | (matched_text_0 as palette) :: _
            when matched_text_0 = {js|player|js} || matched_text_0 = {js|enemy|js}
                 || matched_text_0 = {js|passive|js} || matched_text_0 = {js|green|js} ->
              let _ = Set.add next_palettes palette in
              ()
          | _ -> ())
        (Art.keys artwork);
      units ~sides:next_palettes
        (content_types ~include_recruitment:true catalog Domain_resource_walk.walk_mission next)
        4;
      audio (Campaign_sound.mission next [||]) 4;
      audio
        (Array.map
           (fun (tile : Domain.tile) ->
             Phaser_sound.sound_for_interaction
               { Phaser_sound.interaction with terrain = Some tile.terrain })
           map.terrain)
        4
  | _ -> ());
  (match mission with
  | Some mission ->
      prepare_terrain state.tiles state.map_id Domain_resource_walk.walk_mission mission 1
  | None ->
      prepare_terrain state.tiles state.map_id Domain_resource_walk.walk_map_definition current_map
        1);
  Array.iter
    (fun key ->
      match Array.to_list (Js.String.split ~sep:{js|_|js} key) with
      | matched_text_1 :: _
        when matched_text_1 = {js|player|js} || matched_text_1 = {js|enemy|js}
             || matched_text_1 = {js|passive|js} || matched_text_1 = {js|green|js}
             || matched_text_1 = {js|portrait|js} ->
          image key 5
      | _ -> ())
    (Art.keys artwork);
  List.iter
    (fun (terrain : Domain.terrain_definition) ->
      Option.iter (add_frame ~hud:true 5)
        (Art.terrain_portrait artwork (Enum.string_of_terrain terrain.id)))
    (Catalog.terrain catalog);
  Array.iter
    (fun (sample : Phaser_sound.sample) ->
      if
        Js.String.startsWith ~prefix:{js|palette:|js} sample.family
        || Array.exists (( = ) sample.family) Phaser_sound.action_families
      then add_sample 5 sample)
    Phaser_sound.samples;
  let ordered = Map.create () and values = Map.values_array resources in
  for priority = 1 to 5 do
    let images =
      Array_ext.filter
        (fun resource -> resource.priority = priority && resource.kind = Image)
        values
    and sounds =
      Array_ext.filter
        (fun resource -> resource.priority = priority && resource.kind = Audio)
        values
    in
    for i = 0 to max (Array.length images) (Array.length sounds) - 1 do
      (if i < Array.length sounds then
         let resource = sounds.(i) in
         let _ = Map.set ordered resource.key resource in
         ());
      if i < Array.length images then
        let resource = images.(i) in
        let _ = Map.set ordered resource.key resource in
        ()
    done
  done;
  ordered

(* The same invalidation inputs as upstream, retained as typed OCaml values
   rather than serialized to JSON on every publication. *)
type plan_key = {
  session_version : int;
  map_id : string;
  factions : Domain.faction array;
  audio : string array;
  unit_types : string array;
  terrain_types : string array;
  recruits : string array array;
  mission : float option;
}

let key ~session_version ~game ~catalog =
  let state = Native.state game in
  let unique values =
    let set = Set.create () in
    Array.iter
      (fun value ->
        let _ = Set.add set value in
        ())
      values;
    Set.values_array set
  in
  {
    session_version;
    map_id = state.map_id;
    factions = state.factions;
    audio =
      Array.map (fun (sample : Phaser_sound.sample) -> sample.key) (current_map_audio game catalog);
    unit_types =
      unique (Array.map (fun (unit : Domain.public_unit) -> unit.type_) (Native.units game));
    terrain_types = unique (Array.map (fun (tile : Domain.tile) -> tile.terrain) state.tiles);
    recruits =
      Array.map
        (fun (faction : Domain.faction) -> Native.recruitment_options game faction.id)
        state.factions;
    mission = Option.map (fun (campaign : Domain.campaign_state) -> campaign.mission) state.campaign;
  }

let same_key left right = left = right
