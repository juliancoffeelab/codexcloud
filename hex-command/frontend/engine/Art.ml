type frame = {
  url : string;
  width : float;
  height : float;
  anchor_x : float;
  anchor_y : float;
  z : float;
}

type t = { frames : frame Js.Dict.t }

external pixel_sprites : string Js.Dict.t = "pixelSprites" [@@mel.module "@assets/pixel-art"]
external pixel_models : string Js.Dict.t = "pixelUnitModels" [@@mel.module "@assets/pixel-art"]
external floor_colors : string Js.Dict.t = "pixelFloorColors" [@@mel.module "@assets/pixel-art"]
external has_own : 'a Js.Dict.t -> string -> bool = "hasOwn" [@@mel.scope "Object"]

let lookup dictionary key = if has_own dictionary key then Js.Dict.get dictionary key else None
let dictionary : unit -> 'a Js.Dict.t = [%mel.raw "() => Object.create(null)"]
let base_url : string = [%mel.raw "import.meta.env?.BASE_URL ?? '/' "]

external geometry : Js.Json.t Js.Dict.t = "pixelTerrainGeometry" [@@mel.module "@assets/pixel-art"]

let published =
  let frames = dictionary () in
  Js.Dict.entries pixel_sprites
  |> Array.iter (fun (key, path) ->
      let metadata =
        Array.find_opt
          (fun (kind, _) -> String.starts_with ~prefix:("terrain_" ^ kind ^ "_") key)
          (Js.Dict.entries geometry)
      in
      let field name default =
        Option.value ~default
          (Option.bind metadata (fun (_, json) ->
               Option.bind (Js.Json.decodeObject json) (fun fields ->
                   Option.bind (Js.Dict.get fields name) Js.Json.decodeNumber)))
      in
      Js.Dict.set frames key
        {
          url = base_url ^ path;
          width = field "width" 64.;
          height = field "height" 64.;
          anchor_x = field "anchorX" 32.;
          anchor_y = field "anchorY" 32.;
          z = field "z" 0.;
        });
  { frames }

let current () = published

let frames (artwork : t) =
  let result = dictionary () in
  Array.iter (fun (key, frame) -> Js.Dict.set result key frame) (Js.Dict.entries artwork.frames);
  result

let with_overrides artwork overrides =
  let frames = frames artwork in
  Array.iter (fun (key, frame) -> Js.Dict.set frames key frame) overrides;
  { frames }

let find (artwork : t) key = lookup artwork.frames key
let frame_name = function "bonearcher" -> "bone_archer" | "grove" -> "sacred_grove" | name -> name

let frame_key ~kind ~name ?(side = "player") ?floor () =
  let name =
    if kind = "units" then Option.value (lookup pixel_models name) ~default:name else name
  in
  let key =
    if kind = "items" then "terrain_prop_items_" ^ name
    else (if kind = "units" then side else "terrain") ^ "_" ^ frame_name name
  in
  match (kind, floor) with
  | "terrain", Some floor when lookup floor_colors floor <> None -> key ^ "__" ^ floor
  | _ -> key

let frame artwork ~kind ~name ?(side = "player") ?floor () =
  find artwork (frame_key ~kind ~name ~side ?floor ())

let unit_frame artwork ~side name = frame artwork ~kind:"units" ~name ~side ()
let terrain_frame artwork ?floor name = frame artwork ~kind:"terrain" ~name ?floor ()

let portrait_frame artwork = function
  | "worker" -> find artwork "portrait_worker"
  | name -> unit_frame artwork ~side:"player" name

let icon_path = function
  | "move" -> {js|<path d="m5 17 4-4 3 3 7-9M14 7h5v5"/>|js}
  | "blade" -> {js|<path d="m5 19 3-3m-2-3 5 5m-3-4L17 4l3-1-1 3-10 9"/>|js}
  | "pierce" -> {js|<path d="m5 19 14-14M13 5h6v6M5 14v5h5"/>|js}
  | "impact" -> {js|<path d="m5 19 10-10M11 5l4-3 7 7-3 4z"/>|js}
  | "range" ->
      {js|<circle cx="12" cy="12" r="7"/><circle cx="12" cy="12" r="3"/><path d="M12 2v3m0 14v3M2 12h3m14 0h3"/>|js}
  | "shield" -> {js|<path d="M12 3 4 6v6c0 4 4 7 8 9 4-2 8-5 8-9V6z"/>|js}
  | "blocked" -> {js|<path d="M3 12h18m-4-4 4 4-4 4M8 4v16m8-16L8 20"/>|js}
  | "coin" ->
      {js|<circle cx="12" cy="12" r="8"/><path d="M14 8h-3a2 2 0 0 0 0 4h2a2 2 0 0 1 0 4h-3m2-10v12"/>|js}
  | "heal" -> {js|<path d="M9 3h6v6h6v6h-6v6H9v-6H3V9h6z"/>|js}
  | "recruit" ->
      {js|<circle cx="9" cy="7" r="3"/><path d="M3 21v-3a6 6 0 0 1 12 0v3m4-12v8m-4-4h8"/>|js}
  | "keep" -> {js|<path d="M4 21V3h4v4h3V3h3v4h3V3h3v18zM10 21v-6h4v6"/>|js}
  | "search" -> {js|<circle cx="10" cy="10" r="6"/><path d="m15 15 6 6"/>|js}
  | _ -> ""

let icon_markup name =
  "<svg class=\"ui-icon\" viewBox=\"0 0 24 24\" aria-hidden=\"true\" focusable=\"false\">"
  ^ icon_path name ^ "</svg>"

let terrain_portrait artwork terrain =
  let name, floor =
    match terrain with
    | "plains" -> ("plains_layout_0", "green-plain")
    | "road" -> ("road_9", "green-plain")
    | "forest" -> ("forest_layout_0", "green-forest")
    | "hill" -> ("hill_layout_0", "green-plain")
    | "water" -> ("water", "dark-water")
    | "bridge" -> ("bridge_9", "dark-water")
    | "ford" -> ("ford_9_layout_0", "shallow-water")
    | "village" -> ("village_0", "green-plain")
    | "keep" -> ("keep", "green-plain")
    | "marsh" -> ("marsh", "marsh")
    | "ruins" -> ("ruins", "green-plain")
    | "fort" -> ("fort", "green-plain")
    | "grove" -> ("grove", "green-forest")
    | "landing" -> ("landing_1_layout_0", "shallow-water")
    | "ferry" -> ("ferry_1_layout_0", "dark-water")
    | "house" -> ("house_layout_0", "green-plain")
    | "mountain" -> ("mountain_layout_0", "mountain-rock")
    | "cave" -> ("cave_layout_0", "mountain-rock")
    | "cave-passage" -> ("cave-passage_9", "cave-stone")
    | "cave-wall" -> ("cave-wall", "mountain-rock")
    | "burned-village" -> ("burned-village_layout_0", "green-plain")
    | "signpost" -> ("signpost_9_layout_0", "green-plain")
    | "castle-wall" -> ("castle-wall_63", "castle-paving")
    | "castle-bell" -> ("castle-bell_63", "castle-paving")
    | "monastery" -> ("monastery_layout_0", "green-plain")
    | "church" -> ("church_layout_0", "green-plain")
    | "graveyard" -> ("graveyard_layout_0", "green-plain")
    | "ritual-star" -> ("ritual-star_layout_0", "mountain-rock")
    | "ritual-spent" -> ("ritual-spent_layout_0", "mountain-rock")
    | _ -> invalid_arg ("Missing terrain portrait: " ^ terrain)
  in
  terrain_frame artwork ~floor name

let keys artwork = Js.Dict.keys artwork.frames
