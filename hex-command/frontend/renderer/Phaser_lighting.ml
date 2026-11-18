type data = {
  light_anchor : Phaser.vector option;
  light_point : Phaser.vector option;
  unlit : bool;
  brightness : float option;
  lighting_camera : Phaser.camera option;
  light_cell : Phaser_light_field.cell option;
}

type binding = { unlit : bool; brightness : float option; x : float option; y : float option }

module Pipeline_data = Phaser_pipeline.Data (struct
  type t = data
end)

let plain =
  {
    light_anchor = None;
    light_point = None;
    unlit = false;
    brightness = None;
    lighting_camera = None;
    light_cell = None;
  }

let same_cell a b = match (a, b) with None, None -> true | Some a, Some b -> a == b | _ -> false

class lighting (game : Phaser.game) (camera : Phaser.camera)
  (resources : Phaser_resources.resources) (scene : Phaser.scene) =
  let renderer =
    match Js.Nullable.toOption game##renderer with
    | Some renderer -> renderer
    | None -> failwith {js|Lighting requires the battlefield WebGL renderer|js}
  in
  let count = max 1 (min 32 ((Webgl.parameter renderer##gl 0x8dfd - 24) / 3)) in
  let field = new Phaser_light_field.light_field scene resources in
  object (self)
    val mutable pipeline : Phaser_pipeline.t option = None
    val mutable signature = {js||js}
    val mutable factors : Phaser.texture option = None
    val mutable offsets : Phaser.texture option = None
    val mutable zone_signature = {js||js}
    val mutable zone_texture : Phaser.texture option = None
    val mutable zone_mask : Underground.mask option = None
    val mutable map_width = 1.
    val mutable map_height = 1.
    val mutable origin_x = 0.
    val mutable origin_y = 0.
    val mutable lighting = Phaser_actor_lighting.daylight
    val mutable binding : binding option = None
    val mutable normalization = 1.
    val mutable bound_cell : Phaser_light_field.cell option = None
    val view = Phaser.Geom.Vector2.create ()

    initializer
      pipeline <-
        Some
          (Phaser_pipeline.create ~game ~vertex:Phaser_shaders.lighting_vertex
             ~fragment:(Phaser_pipeline.shader_count Phaser_shaders.lighting_fragment count)
             ~bind:(fun native _ object_ -> Option.iter (self#bind native) object_)
             ~dispose:(fun () ->
               self#release;
               renderer##pipelines##remove {js|BattlefieldLightField|js} false;
               field#native##destroy ())
             ());
      let _ = renderer##pipelines##add {js|BattlefieldLightField|js} field#native in
      ()

    method native =
      match pipeline with
      | Some pipeline -> Phaser_pipeline.native pipeline
      | None -> failwith {js|Lighting pipeline is not initialized|js}

    method cells = field#cells

    method configure ?(x = 0.) ?(y = 0.) (next : Domain.battlefield_lighting) width height =
      let next_signature =
        Js.Array.join ~sep:{js|:|js}
          (Array.of_list
             [
               Js.Float.toString x;
               Js.Float.toString y;
               Js.Float.toString width;
               Js.Float.toString height;
               Lighting.key next;
             ])
      in
      if next_signature <> signature then (
        signature <- next_signature;
        map_width <- width;
        map_height <- height;
        origin_x <- x;
        origin_y <- y;
        lighting <- next;
        binding <- None;
        self#native##flush ();
        factors <- None;
        offsets <- None;
        self#configure_zones next;
        if Lighting.daylight_only next then field#release
        else
          let built =
            Performance_trace.measure {js|Phaser lighting field|js} (fun () ->
                field#build next width height x y)
          in
          factors <- built.factors;
          offsets <- built.offsets;
          normalization <- built.normalization)

    method private release_zones =
      Option.iter
        (fun (_ : Phaser.texture) ->
          let _ = scene##textures##remove {js|battlefield-underground|js} in
          ())
        zone_texture;
      zone_texture <- None;
      zone_mask <- None;
      zone_signature <- {js||js};
      resources#remove {js|underground-mask|js}

    method private configure_zones (next : Domain.battlefield_lighting) =
      let key = Underground.key next.unlit_tiles in
      if key <> zone_signature then (
        self#native##flush ();
        self#release_zones;
        zone_signature <- key;
        zone_mask <- Underground.get next.unlit_tiles;
        Option.iter
          (fun (mask : Underground.mask) ->
            let _ =
              resources#check {js|underground ambience mask|js} (float_of_int mask.width)
                (float_of_int mask.height)
            in
            let bytes =
              Array.init
                (mask.width * mask.height * 4)
                (fun i -> if i mod 4 = 3 then 255 else int_of_float mask.data.(i / 4))
            in
            let texture =
              Phaser.SceneMethods.white_texture scene##textures {js|battlefield-underground|js}
                (Phaser.SceneMethods.bytes bytes) mask.width mask.height
            in
            let _ = texture##setFilter Phaser.Textures.linear in
            zone_texture <- Some texture;
            resources#set {js|underground-mask|js} {js|underground ambience|js}
              (float_of_int (Array.length mask.data * 8))
              (float_of_int (mask.width * mask.height * 4)))
          zone_mask)

    method private bind (native : Phaser.pipeline) object_ =
      let data = Pipeline_data.get object_ in
      let point =
        match data.light_anchor with Some point -> Some point | None -> data.light_point
      in
      let next =
        {
          unlit = data.unlit;
          brightness = data.brightness;
          x = Option.map (fun (point : Phaser.vector) -> point##x) point;
          y = Option.map (fun (point : Phaser.vector) -> point##y) point;
        }
      in
      if binding <> Some next || not (same_cell bound_cell data.light_cell) then (
        native##flush ();
        binding <- Some next;
        bound_cell <- data.light_cell);
      let direct =
        Option.fold ~none:false
          ~some:(fun (cell : Phaser_light_field.cell) ->
            Array.length cell.lights <= field#batch_size)
          data.light_cell
      in
      let _ = native##set1f {js|uDirect|js} (if direct then 1. else 0.) in
      (match data.light_cell with
      | Some cell when not direct ->
          let width, height = field#packed_size in
          let _ = native##set2f {js|uPackedSize|js} (float_of_int width) (float_of_int height) in
          let _ =
            native##set2f {js|uPackedShift|js}
              (float_of_int (cell.packed_x - cell.x))
              (float_of_int (cell.packed_y - cell.y))
          in
          ()
      | _ -> ());
      (match data.light_cell with
      | Some cell when direct ->
          let positions = Webgl.floats (field#batch_size * 4)
          and warm = Webgl.floats (field#batch_size * 3)
          and offsets = Webgl.floats (field#batch_size * 3) in
          Array.iteri
            (fun i (light : Domain.battlefield_light) ->
              let x, y = Lighting.position light in
              Webgl.set positions (i * 4) x;
              Webgl.set positions ((i * 4) + 1) y;
              Webgl.set positions ((i * 4) + 2) light.radius;
              Webgl.set positions ((i * 4) + 3) light.power;
              for channel = 0 to 2 do
                Webgl.set warm
                  ((i * 3) + channel)
                  (match light.color with
                  | Some colors -> float_of_int colors.(channel) /. 255. *. 1.06
                  | None -> Phaser_light_field.default_warm.(channel));
                Webgl.set offsets
                  ((i * 3) + channel)
                  (match light.color with
                  | Some colors -> float_of_int colors.(channel) /. 255. *. 30.
                  | None -> Phaser_light_field.default_offset.(channel))
              done)
            cell.lights;
          let _ = native##set1f {js|uCount|js} (float_of_int (Array.length cell.lights)) in
          let _ = native##set4fv {js|uLights|js} positions in
          let _ = native##set3fv {js|uWarm|js} warm in
          let _ = native##set3fv {js|uOffsetsDirect|js} offsets in
          ()
      | _ -> ());
      let camera = Option.value ~default:camera data.lighting_camera in
      let _ = camera##getWorldPoint 0. 0. view in
      let _ = native##set2f {js|uWorldView|js} view##x view##y in
      let _ = native##set2f {js|uZoom|js} camera##zoomX camera##zoomY in
      let _ = native##set2f {js|uMapSize|js} map_width map_height in
      let _ = native##set2f {js|uFieldOrigin|js} origin_x origin_y in
      let _ =
        native##set2f {js|uFieldSize|js}
          (floor ((map_width *. Geometry.native_pixel_scale) +. 0.5) *. 4.)
          (floor ((map_height *. Geometry.native_pixel_scale) +. 0.5))
      in
      let _ = native##set1f {js|uNativeScale|js} Geometry.native_pixel_scale in
      let _ = native##set1f {js|uNormalization|js} normalization in
      let _ = native##set1f {js|uDay|js} (if not data.unlit then 1. else 0.) in
      let _ = native##set1f {js|uBrightness|js} (Option.value ~default:1. data.brightness) in
      let _ = native##set1f {js|uGround|js} (if Option.is_some point then 1. else 0.) in
      (match point with
      | Some point when not (Lighting.daylight_only lighting) ->
          let matrix = Lighting.matrix point##x point##y lighting in
          let _ = native##set3f {js|uRowR|js} matrix.(0) matrix.(1) matrix.(2) in
          let _ = native##set3f {js|uRowG|js} matrix.(5) matrix.(6) matrix.(7) in
          let _ = native##set3f {js|uRowB|js} matrix.(10) matrix.(11) matrix.(12) in
          let _ = native##set3f {js|uOffset|js} matrix.(4) matrix.(9) matrix.(14) in
          ()
      | _ -> ());
      let _ =
        native##set1f {js|uUnderground|js}
          (if (not data.unlit) && Option.is_some zone_mask then 1. else 0.)
      in
      (match zone_mask with
      | None ->
          let _ = native##set4f {js|uUndergroundBounds|js} 0. 0. 1. 1. in
          ()
      | Some mask ->
          let _ =
            native##set4f {js|uUndergroundBounds|js} mask.x mask.y
              (float_of_int mask.width *. mask.step)
              (float_of_int mask.height *. mask.step)
          in
          ());
      let _ = native##set1i {js|uUndergroundMask|js} 3 in
      Phaser_pipeline.bind_texture native
        (Option.bind zone_texture Phaser_pipeline.source_texture)
        3;
      let _ = native##set1i {js|uFactors|js} 1 in
      let _ = native##set1i {js|uOffsets|js} 2 in
      Phaser_pipeline.bind_texture native (Option.bind factors Phaser_pipeline.source_texture) 1;
      Phaser_pipeline.bind_texture native (Option.bind offsets Phaser_pipeline.source_texture) 2;
      Phaser_pipeline.restore_texture native

    method release =
      signature <- {js||js};
      binding <- None;
      field#release;
      self#release_zones;
      factors <- None;
      offsets <- None
  end
