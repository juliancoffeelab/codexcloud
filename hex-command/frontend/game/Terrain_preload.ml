(* Warm bytes only. Phaser owns decoding, admission and GPU resources. *)
open Browser_image

let map catalog id =
  match
    List.find_opt (fun (map : Domain.browser_map) -> map.id = id) (Catalog.scenario_maps catalog)
  with
  | Some map ->
      Option.iter
        (fun (sprite : Terrain_pages.sprite) ->
          Array.iter (fun (page : Terrain_pages.page) -> source (image ()) page.url) sprite.pages)
        (Js.Dict.get Terrain_pages.sprites (Terrain_pages.asset_key id map.revision))
  | None ->
      (* Campaign maps are intentionally absent from the scenario picker. *)
      Array.iter
        (fun (map : Domain.browser_map) ->
          if map.id = id then
            Option.iter
              (fun (sprite : Terrain_pages.sprite) ->
                Array.iter
                  (fun (page : Terrain_pages.page) -> source (image ()) page.url)
                  sprite.pages)
              (Js.Dict.get Terrain_pages.sprites (Terrain_pages.asset_key id map.revision)))
        (Catalog.raw catalog).maps
