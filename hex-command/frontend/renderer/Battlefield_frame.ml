open Battlefield_view
module Map = Host_collections.Map
module Set = Host_collections.Set

type unit_view = {
  unit : Domain.public_unit;
  faction : Domain.unit_faction;
  team_color : string;
  frame : Art.frame option;
  sprite_size : float;
  health_width : float;
}

type tile = {
  terrain : Terrain_plan.tile;
  display : Presentation.tile_display;
  occupant : unit_view option;
}

type view = Battlefield_view.view

type t = {
  view : view;
  tiles : tile array;
  terrain_tiles : Terrain_plan.tile array;
  reduced_motion : bool;
}

class cache =
  object
    val display_cache = Presentation.create_battlefield_cache ()
    val mutable previous_game : Native.view option = None
    val mutable previous_artwork : Art.t option = None
    val mutable previous_selected : string option = None
    val mutable previous_recruit : string option = None
    val mutable terrain_source : Domain.state option = None
    val mutable terrain_plan : Terrain_plan.t option = None
    val mutable tiles : tile array = [||]
    val mutable terrain_tiles : Terrain_plan.tile array = [||]
    val old_tiles : (string, tile) Map.t = Map.create ()

    method prepare (view : view) reduced_motion =
      Performance_trace.measure "frame.prepare" (fun () ->
          let state = Native.state view.game in
          let recruit =
            if state.turn = Domain.Turn_phase.Player && view.recruitment then view.inspected_tile
            else None
          in
          let artwork_same =
            Option.fold ~none:false ~some:(fun old -> old == view.artwork) previous_artwork
          in
          if
            (not (Option.fold ~none:false ~some:(fun old -> old == view.game) previous_game))
            || (not artwork_same)
            || previous_selected <> view.selected
            || previous_recruit <> recruit
          then (
            let same_terrain =
              Option.fold ~none:false
                ~some:(fun (old : Domain.state) ->
                  old.map_id = state.map_id && old.cols = state.cols && old.rows = state.rows
                  && (old.tiles == state.tiles
                     || Array_ext.equal Terrain_plan.same_tile old.tiles state.tiles))
                terrain_source
            in
            let plan =
              match terrain_plan with
              | Some plan when same_terrain && artwork_same -> plan
              | _ ->
                  Terrain_plan.prepare
                    ~roles:(Catalog.terrain_roles view.catalog)
                    ~map_id:state.map_id ~cols:(Int32.to_float state.cols)
                    ~rows:(Int32.to_float state.rows) state.tiles view.artwork
            in
            terrain_source <- Some state;
            terrain_plan <- Some plan;
            terrain_tiles <- plan.tiles;
            let displays =
              Array.of_list
                (Presentation.battlefield ~cache:display_cache view.catalog view.game
                   ~selected:view.selected ~inspected:recruit)
            in
            tiles <-
              Array.mapi
                (fun i display ->
                  let terrain = plan.tiles.(i) in
                  match Map.get old_tiles terrain.id with
                  | Some old when old.terrain == terrain && old.display == display -> old
                  | _ ->
                      let occupant =
                        Option.bind display.Presentation.occupant
                          (fun (unit : Domain.public_unit) ->
                            Option.map
                              (fun (faction : Domain.unit_faction) ->
                                {
                                  unit;
                                  faction;
                                  team_color =
                                    (match faction.palette with
                                    | "player" -> "#6fdec0"
                                    | "enemy" -> "#ff9590"
                                    | "green" -> "#83b765"
                                    | _ -> "#efecdc");
                                  frame =
                                    Art.unit_frame view.artwork ~side:faction.palette unit.type_;
                                  sprite_size = Geometry.sprite_size;
                                  health_width = display.health_width;
                                })
                              (Native.faction view.game unit.id))
                      in
                      { terrain; display; occupant })
                displays;
            Map.clear old_tiles;
            Array.iter
              (fun tile ->
                let _ = Map.set old_tiles tile.terrain.id tile in
                ())
              tiles;
            previous_game <- Some view.game;
            previous_artwork <- Some view.artwork;
            previous_selected <- view.selected;
            previous_recruit <- recruit);
          { view; tiles; terrain_tiles; reduced_motion })
  end

type unit_actor = {
  id : string;
  tile : string;
  x : float;
  y : float;
  url : string option;
  size : float;
  color : string;
  health : float;
  hp : int32;
  spent : bool;
  retreating : bool;
  concealment : Domain.Unit_concealment_appearance.t;
}

type ground_item = { key : string; x : float; y : float; url : string }
type scene_data = { units : unit_actor array; items : ground_item array; fog : string array }

(* A private cache identity over a closed, serializable OCaml value. This is
   never used as a save/wire encoding, just as upstream uses JSON.stringify(item). *)
external item_identity : Domain.item -> string = "stringify" [@@mel.scope "JSON"]

let scene_data frame =
  let ongoing = (Native.state frame.view.game).winner = None in
  let fog = Set.create () in
  let presentation = Native.presentation frame.view.game in
  let appearances = Map.create () in
  Array.iter
    (fun (a : Domain.unit_appearance) -> ignore (Map.set appearances a.unit a.concealment))
    presentation.unit_appearances;
  let fog_ids = Array.map Hex.key presentation.fog in
  Array.iter
    (fun id ->
      let _ = Set.add fog id in
      ())
    fog_ids;
  let centers = Map.create () in
  Array.iter
    (fun tile ->
      let _ = Map.set centers tile.terrain.id tile in
      ())
    frame.tiles;
  let piles = Map.create () in
  Array.iter
    (fun (item : Domain.ground_item) ->
      let key = Hex.key item.tile in
      let pile =
        match Map.get piles key with
        | Some pile -> pile
        | None ->
            let pile = [||] in
            let _ = Map.set piles key pile in
            pile
      in
      let _ = Array_ext.append_value pile item in
      ())
    (Native.ground_items frame.view.game);
  let units =
    Array_ext.filter_map
      (fun tile ->
        match tile.occupant with
        | None -> None
        | Some occupant
          when Set.has fog tile.terrain.id && occupant.faction.relation = Domain.Relation.Hostile ->
            None
        | Some occupant ->
            Some
              {
                id = occupant.unit.id;
                tile = tile.terrain.id;
                x = tile.terrain.x;
                y = tile.terrain.y;
                url = Option.map (fun (frame : Art.frame) -> frame.url) occupant.frame;
                size = occupant.sprite_size;
                color = occupant.team_color;
                health = occupant.health_width;
                hp = occupant.unit.hp;
                concealment =
                  Option.value ~default:Domain.Unit_concealment_appearance.Normal
                    (Map.get appearances occupant.unit.id);
                spent = ongoing && occupant.unit.attack_spent;
                retreating = ongoing && occupant.unit.retreating = Some true;
              })
      frame.tiles
  in
  let items = [||] in
  Map.each piles (fun[@u] pile id ->
      match Map.get centers id with
      | Some tile when not (Set.has fog id) ->
          let occurrences = Map.create () in
          Array.iteri
            (fun index (ground : Domain.ground_item) ->
              let identity = item_identity ground.item in
              let occurrence = Option.value ~default:0 (Map.get occurrences identity) in
              let _ = Map.set occurrences identity (occurrence + 1) in
              let frame = Phaser_resource_plan.item_frame frame.view.artwork ground.item.id in
              let _ =
                Array_ext.append_value items
                  {
                    key = id ^ ":" ^ identity ^ ":" ^ string_of_int occurrence;
                    x =
                      tile.terrain.x
                      +. (float_of_int index -. (float_of_int (Array.length pile - 1) /. 2.))
                         *. 12.;
                    y = tile.terrain.y;
                    url = frame.url;
                  }
              in
              ())
            pile
      | _ -> ());
  { units; items; fog = fog_ids }
