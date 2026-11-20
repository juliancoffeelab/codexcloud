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
let dictionary : unit -> 'a Js.Dict.t = [%mel.raw {js|() => Object.create(null)|js}]
let base_url : string = [%mel.raw {js|import.meta.env?.BASE_URL ?? '/' |js}]

external geometry : Js.Json.t Js.Dict.t = "pixelTerrainGeometry" [@@mel.module "@assets/pixel-art"]

let published =
  let frames = dictionary () in
  Js.Dict.entries pixel_sprites
  |> Array.iter (fun (key, path) ->
      let metadata =
        Array.find_opt
          (fun (kind, _) -> Js.String.startsWith ~prefix:{j|terrain_$(kind)_|j} key)
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
          url = {j|$(base_url)$(path)|j};
          width = field {js|width|js} 64.;
          height = field {js|height|js} 64.;
          anchor_x = field {js|anchorX|js} 32.;
          anchor_y = field {js|anchorY|js} 32.;
          z = field {js|z|js} 0.;
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

let frame_name = function
  | matched_text_0 when matched_text_0 = {js|bonearcher|js} -> {js|bone_archer|js}
  | matched_text_1 when matched_text_1 = {js|grove|js} -> {js|sacred_grove|js}
  | name -> name

let frame_key ~kind ~name ?(side = {js|player|js}) ?floor () =
  let name =
    if kind = {js|units|js} then Option.value (lookup pixel_models name) ~default:name else name
  in
  let key =
    if kind = {js|items|js} then {j|terrain_prop_items_$(name)|j}
    else
      let js_text_part_0 = if kind = {js|units|js} then side else {js|terrain|js} in
      let js_text_part_2 = frame_name name in
      {j|$(js_text_part_0)_$(js_text_part_2)|j}
  in
  match (kind, floor) with
  | matched_text_2, Some floor
    when matched_text_2 = {js|terrain|js} && lookup floor_colors floor <> None ->
      {j|$(key)__$(floor)|j}
  | _ -> key

let frame artwork ~kind ~name ?(side = {js|player|js}) ?floor () =
  find artwork (frame_key ~kind ~name ~side ?floor ())

let unit_frame artwork ~side name = frame artwork ~kind:{js|units|js} ~name ~side ()
let terrain_frame artwork ?floor name = frame artwork ~kind:{js|terrain|js} ~name ?floor ()

let portrait_frame artwork = function
  | matched_text_3 when matched_text_3 = {js|worker|js} -> find artwork {js|portrait_worker|js}
  | name -> unit_frame artwork ~side:{js|player|js} name

let icon_path = function
  | matched_text_4 when matched_text_4 = {js|move|js} ->
      {js|<path d="m5 17 4-4 3 3 7-9M14 7h5v5"/>|js}
  | matched_text_5 when matched_text_5 = {js|blade|js} ->
      {js|<path d="m5 19 3-3m-2-3 5 5m-3-4L17 4l3-1-1 3-10 9"/>|js}
  | matched_text_6 when matched_text_6 = {js|pierce|js} ->
      {js|<path d="m5 19 14-14M13 5h6v6M5 14v5h5"/>|js}
  | matched_text_7 when matched_text_7 = {js|impact|js} ->
      {js|<path d="m5 19 10-10M11 5l4-3 7 7-3 4z"/>|js}
  | matched_text_8 when matched_text_8 = {js|range|js} ->
      {js|<circle cx="12" cy="12" r="7"/><circle cx="12" cy="12" r="3"/><path d="M12 2v3m0 14v3M2 12h3m14 0h3"/>|js}
  | matched_text_9 when matched_text_9 = {js|shield|js} ->
      {js|<path d="M12 3 4 6v6c0 4 4 7 8 9 4-2 8-5 8-9V6z"/>|js}
  | matched_text_10 when matched_text_10 = {js|blocked|js} ->
      {js|<path d="M3 12h18m-4-4 4 4-4 4M8 4v16m8-16L8 20"/>|js}
  | matched_text_11 when matched_text_11 = {js|coin|js} ->
      {js|<circle cx="12" cy="12" r="8"/><path d="M14 8h-3a2 2 0 0 0 0 4h2a2 2 0 0 1 0 4h-3m2-10v12"/>|js}
  | matched_text_12 when matched_text_12 = {js|heal|js} ->
      {js|<path d="M9 3h6v6h6v6h-6v6H9v-6H3V9h6z"/>|js}
  | matched_text_13 when matched_text_13 = {js|recruit|js} ->
      {js|<circle cx="9" cy="7" r="3"/><path d="M3 21v-3a6 6 0 0 1 12 0v3m4-12v8m-4-4h8"/>|js}
  | matched_text_14 when matched_text_14 = {js|keep|js} ->
      {js|<path d="M4 21V3h4v4h3V3h3v4h3V3h3v18zM10 21v-6h4v6"/>|js}
  | matched_text_15 when matched_text_15 = {js|search|js} ->
      {js|<circle cx="10" cy="10" r="6"/><path d="m15 15 6 6"/>|js}
  | _ -> {js||js}

let icon_markup name =
  let js_text_part_1 = icon_path name in
  {j|<svg class="ui-icon" viewBox="0 0 24 24" aria-hidden="true" focusable="false">$(js_text_part_1)</svg>|j}

let terrain_portrait artwork terrain =
  let name, floor =
    match terrain with
    | matched_text_16 when matched_text_16 = {js|plains|js} ->
        ({js|plains_layout_0|js}, {js|green-plain|js})
    | matched_text_17 when matched_text_17 = {js|road|js} -> ({js|road_9|js}, {js|green-plain|js})
    | matched_text_18 when matched_text_18 = {js|forest|js} ->
        ({js|forest_layout_0|js}, {js|green-forest|js})
    | matched_text_19 when matched_text_19 = {js|hill|js} ->
        ({js|hill_layout_0|js}, {js|green-plain|js})
    | matched_text_20 when matched_text_20 = {js|water|js} -> ({js|water|js}, {js|dark-water|js})
    | kind when kind = {js|shallow-water|js} -> ({js|shallow-water|js}, {js|shallow-water|js})
    | matched_text_21 when matched_text_21 = {js|bridge|js} -> ({js|bridge_9|js}, {js|dark-water|js})
    | matched_text_22 when matched_text_22 = {js|ford|js} ->
        ({js|ford_9_layout_0|js}, {js|shallow-water|js})
    | matched_text_23 when matched_text_23 = {js|village|js} ->
        ({js|village_0|js}, {js|green-plain|js})
    | matched_text_24 when matched_text_24 = {js|keep|js} -> ({js|keep|js}, {js|green-plain|js})
    | matched_text_25 when matched_text_25 = {js|marsh|js} -> ({js|marsh|js}, {js|marsh|js})
    | matched_text_26 when matched_text_26 = {js|ruins|js} -> ({js|ruins|js}, {js|green-plain|js})
    | matched_text_27 when matched_text_27 = {js|fort|js} -> ({js|fort|js}, {js|green-plain|js})
    | matched_text_28 when matched_text_28 = {js|grove|js} -> ({js|grove|js}, {js|green-forest|js})
    | matched_text_29 when matched_text_29 = {js|landing|js} ->
        ({js|landing_1_layout_0|js}, {js|shallow-water|js})
    | matched_text_30 when matched_text_30 = {js|ferry|js} ->
        ({js|ferry_1_layout_0|js}, {js|dark-water|js})
    | matched_text_31 when matched_text_31 = {js|house|js} ->
        ({js|house_layout_0|js}, {js|green-plain|js})
    | matched_text_32 when matched_text_32 = {js|mountain|js} ->
        ({js|mountain_layout_0|js}, {js|mountain-rock|js})
    | matched_text_33 when matched_text_33 = {js|mountain-mine|js} ->
        ({js|mountain-mine_layout_0|js}, {js|mountain-rock|js})
    | matched_text_34 when matched_text_34 = {js|cave-passage|js} ->
        ({js|cave-passage_9|js}, {js|cave-stone|js})
    | matched_text_35 when matched_text_35 = {js|cave-wall|js} ->
        ({js|cave-wall|js}, {js|mountain-rock|js})
    | matched_text_36 when matched_text_36 = {js|burned-village|js} ->
        ({js|burned-village_layout_0|js}, {js|green-plain|js})
    | matched_text_37 when matched_text_37 = {js|signpost|js} ->
        ({js|signpost_9_layout_0|js}, {js|green-plain|js})
    | matched_text_38 when matched_text_38 = {js|castle-wall|js} ->
        ({js|castle-wall_63|js}, {js|castle-paving|js})
    | matched_text_39 when matched_text_39 = {js|castle-bell|js} ->
        ({js|castle-bell_63|js}, {js|castle-paving|js})
    | matched_text_40 when matched_text_40 = {js|monastery|js} ->
        ({js|monastery_layout_0|js}, {js|green-plain|js})
    | matched_text_41 when matched_text_41 = {js|church|js} ->
        ({js|church_layout_0|js}, {js|green-plain|js})
    | matched_text_42 when matched_text_42 = {js|graveyard|js} ->
        ({js|graveyard_layout_0|js}, {js|green-plain|js})
    | matched_text_43 when matched_text_43 = {js|ritual-star|js} ->
        ({js|ritual-star_layout_0|js}, {js|mountain-rock|js})
    | matched_text_44 when matched_text_44 = {js|ritual-spent|js} ->
        ({js|ritual-spent_layout_0|js}, {js|mountain-rock|js})
    | _ -> invalid_arg {j|Missing terrain portrait: $(terrain)|j}
  in
  terrain_frame artwork ~floor name

let keys artwork = Js.Dict.keys artwork.frames
