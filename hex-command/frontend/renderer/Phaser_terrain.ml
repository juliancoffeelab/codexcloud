module Map = Host_collections.Map
module Set = Host_collections.Set

let scale = Geometry.native_pixel_scale

let bounds (tile : Terrain_plan.tile) =
  {
    Phaser_spatial.left = tile.sprite_x *. scale;
    top = tile.sprite_y *. scale;
    right = (tile.sprite_x +. tile.sprite_width) *. scale;
    bottom = (tile.sprite_y +. tile.sprite_height) *. scale;
  }

let same (a : Terrain_plan.tile) (b : Terrain_plan.tile) =
  (match (a.frame, b.frame) with None, None -> true | Some a, Some b -> a == b | _ -> false)
  && a.terrain = b.terrain && a.sprite_x = b.sprite_x && a.sprite_y = b.sprite_y
  && a.sprite_width = b.sprite_width && a.sprite_height = b.sprite_height && a.x = b.x && a.y = b.y

let region (page : Terrain_pages.page) =
  {
    Phaser_spatial.left = float_of_int (page.x - page.gutter);
    top = float_of_int (page.y - page.gutter);
    right = float_of_int (page.x + page.interior_width + page.gutter);
    bottom = float_of_int (page.y + page.interior_height + page.gutter);
  }

type source = Image of Phaser.image | Surface of Phaser.render_texture

let object_ = function
  | Image image -> Phaser.Upcast.image image
  | Surface surface -> Phaser.Upcast.render_texture surface

let texture = function Image image -> image##texture | Surface surface -> surface##texture

let position = function
  | Image image -> (image##x, image##y)
  | Surface surface -> (surface##x, surface##y)

let destroy_source = function
  | Image image -> image##destroy ()
  | Surface surface -> surface##destroy ()

let light_source source data =
  match source with
  | Image image ->
      let _ = Phaser_lighting.Pipeline_data.image image "BattlefieldLighting" data false in
      ()
  | Surface surface ->
      let _ = Phaser_lighting.Pipeline_data.target surface "BattlefieldLighting" data false in
      ()

type page_record = {
  page : Terrain_pages.page;
  mutable unlit : source option;
  mutable lit : Phaser.render_texture option;
  mutable display : Phaser.image option;
  mutable dirty : bool;
}

let make_display scene (page : Terrain_pages.page) texture x y =
  let display = Phaser.TextureImage.create scene x y texture in
  let _ = display##setOrigin 0. 0. in
  let _ =
    display##setDisplaySize (float_of_int page.width /. scale) (float_of_int page.height /. scale)
  in
  let _ =
    display##setCrop (float_of_int page.gutter) (float_of_int page.gutter)
      (float_of_int page.interior_width)
      (float_of_int page.interior_height)
  in
  display

class terrain (scene : Phaser.scene) (manifest : Terrain_pages.sprite)
  (initial : Terrain_plan.tile array) (pipeline : Phaser_lighting.lighting)
  (resources : Phaser_resources.resources) =
  object (self)
    val retired_sources : string Set.t = Set.create ()
    val mutable visibility_signature : (Phaser_spatial.bounds * int) option = None
    val mutable version = 0
    val root = scene##add##container 0. 0.

    val pages =
      Array.map
        (fun page -> { page; unlit = None; lit = None; display = None; dirty = false })
        manifest.pages

    val mutable active : page_record Set.t = Set.create ()
    val page_index : page_record Phaser_spatial.t = Phaser_spatial.create ~size:1020. ()
    val tile_index : Terrain_plan.tile Phaser_spatial.t = Phaser_spatial.create ~size:128. ()
    val cells : (string, Terrain_plan.tile) Map.t = Map.create ()
    val dirty : page_record Set.t = Set.create ()
    val mutable lighting : Domain.battlefield_lighting option = None
    val mutable signature = ""
    val light_index : int Phaser_spatial.t = Phaser_spatial.create ~size:256. ()

    initializer
      let _ =
        resources#area "terrain manifest" (float_of_int manifest.width)
          (float_of_int manifest.height)
      in
      Array.iter
        (fun record ->
          let page = record.page in
          let _ =
            resources#check ~target:true "terrain page" (float_of_int page.width)
              (float_of_int page.height)
          in
          let _ =
            resources#check "double-width coefficient page"
              (float_of_int (page.width * 2))
              (float_of_int page.height)
          in
          if
            page.x < 0 || page.y < 0 || page.interior_width <= 0 || page.interior_height <= 0
            || page.x + page.interior_width > manifest.width
            || page.y + page.interior_height > manifest.height
          then failwith ("Invalid terrain page extent: " ^ page.url);
          if
            page.width <> page.interior_width + (page.gutter * 2)
            || page.height <> page.interior_height + (page.gutter * 2)
            || page.gutter < 2
          then failwith ("Invalid padded terrain manifest: " ^ page.url);
          Phaser_spatial.set page_index record (region page))
        pages;
      Array.iter
        (fun (tile : Terrain_plan.tile) ->
          let _ = Map.set cells tile.id tile in
          Phaser_spatial.set tile_index tile (bounds tile))
        initial

    method root = root
    method retired_sources = retired_sources

    method plan (current : Terrain_plan.tile array) =
      let live = Set.create () in
      Array.iter
        (fun (tile : Terrain_plan.tile) ->
          let _ = Set.add live tile.id in
          let previous = Map.get cells tile.id in
          if not (Option.fold ~none:false ~some:(fun previous -> same previous tile) previous) then (
            Option.iter
              (fun previous ->
                Array.iter
                  (fun page ->
                    let _ = Set.add dirty page in
                    ())
                  (Phaser_spatial.query page_index (bounds previous));
                Phaser_spatial.delete tile_index previous)
              previous;
            Array.iter
              (fun page ->
                let _ = Set.add dirty page in
                ())
              (Phaser_spatial.query page_index (bounds tile));
            let _ = Map.set cells tile.id tile in
            Phaser_spatial.set tile_index tile (bounds tile)))
        current;
      Map.each cells (fun[@u] tile id ->
          if not (Set.has live id) then (
            Array.iter
              (fun page ->
                let _ = Set.add dirty page in
                ())
              (Phaser_spatial.query page_index (bounds tile));
            let _ = Map.delete cells id in
            Phaser_spatial.delete tile_index tile));
      let urls = Set.create () in
      Array.iter
        (fun record ->
          if not (Set.has retired_sources record.page.url) then
            let _ = Set.add urls record.page.url in
            ())
        pages;
      Set.each dirty (fun[@u] record ->
          Array.iter
            (fun (tile : Terrain_plan.tile) ->
              match tile.frame with
              | None -> failwith ("Missing terrain frame: " ^ tile.id)
              | Some frame ->
                  let _ = Set.add urls frame.url in
                  ())
            (Phaser_spatial.query tile_index (region record.page)));
      urls

    method private make_source (page : Terrain_pages.page) =
      let image = Phaser.Texture.source_image (scene##textures##get page.url) in
      if
        Phaser_dom.natural_width image <> page.width
        || Phaser_dom.natural_height image <> page.height
      then failwith ("Invalid natural terrain page dimensions: " ^ page.url);
      let source =
        Phaser.GameObjects.image scene
          (float_of_int (page.x - page.gutter) /. scale)
          (float_of_int (page.y - page.gutter) /. scale)
          page.url
      in
      let _ = source##setOrigin 0. 0. in
      let _ =
        source##setDisplaySize (float_of_int page.width /. scale) (float_of_int page.height /. scale)
      in
      Image source

    method private rebuild record =
      let page = record.page in
      let surface =
        match record.unlit with
        | Some (Surface surface) -> surface
        | previous ->
            let surface =
              Phaser.GameObjects.render_texture scene
                (float_of_int (page.x - page.gutter) /. scale)
                (float_of_int (page.y - page.gutter) /. scale)
                page.width page.height false
            in
            let _ = surface##setOrigin 0. 0. in
            let _ =
              surface##setDisplaySize
                (float_of_int page.width /. scale)
                (float_of_int page.height /. scale)
            in
            Option.iter destroy_source previous;
            let _ = Set.add retired_sources page.url in
            record.unlit <- Some (Surface surface);
            surface
      in
      let _ = surface##camera##setOrigin 0. 0. in
      let _ = surface##camera##setZoom scale in
      let _ =
        surface##camera##setScroll
          (float_of_int (page.x - page.gutter) /. scale)
          (float_of_int (page.y - page.gutter) /. scale)
      in
      let images = [||] in
      Fun.protect
        ~finally:(fun () -> Array.iter (fun (image : Phaser.image) -> image##destroy ()) images)
        (fun () ->
          let tiles = Phaser_spatial.query tile_index (region page) in
          let _ =
            Host_collections.sort tiles (fun[@u] a b ->
                let z (tile : Terrain_plan.tile) =
                  Option.fold ~none:0. ~some:(fun (frame : Art.frame) -> frame.z) tile.frame
                in
                let order = Float.compare (z a) (z b) in
                if order <> 0 then order
                else
                  let order = Float.compare a.y b.y in
                  if order <> 0 then order else Float.compare a.x b.x)
          in
          Array.iter
            (fun (tile : Terrain_plan.tile) ->
              let frame =
                match tile.frame with
                | Some frame -> frame
                | None -> failwith ("Missing terrain frame: " ^ tile.id)
              in
              let image = Phaser.GameObjects.image scene tile.sprite_x tile.sprite_y frame.url in
              let _ = image##setOrigin 0. 0. in
              let _ = image##setDisplaySize tile.sprite_width tile.sprite_height in
              let _ =
                Phaser_lighting.Pipeline_data.image image "BattlefieldLighting"
                  {
                    Phaser_lighting.plain with
                    unlit = true;
                    brightness = Some (if tile.role = Domain.Terrain_role.Forest then 0.93 else 1.);
                  }
                  false
              in
              let _ = Array_ext.append_value images image in
              ())
            tiles;
          let _ = surface##clear () in
          Phaser.RenderTexture.draw_many surface (Array.map Phaser.Upcast.image images);
          self#clip_outer surface page;
          record.dirty <- true;
          resources#set ("terrain-edit:" ^ page.url) "edited unlit terrain" 0.
            (float_of_int page.width *. float_of_int page.height *. 4.))

    method private clip_outer (target : Phaser.render_texture) (page : Terrain_pages.page) =
      let texture = target##texture in
      let left = max 0 (page.gutter - page.x)
      and top = max 0 (page.gutter - page.y)
      and right = max 0 (page.x - page.gutter + page.width - manifest.width)
      and bottom = max 0 (page.y - page.gutter + page.height - manifest.height) in
      let clear x y width height =
        Phaser.DynamicTexture.dirty texture true;
        Phaser.DynamicTexture.clear texture x y width height
      in
      if left > 0 then clear 0 0 left page.height;
      if top > 0 then clear 0 0 page.width top;
      if right > 0 then clear (page.width - right) 0 right page.height;
      if bottom > 0 then clear 0 (page.height - bottom) page.width bottom;
      Phaser.DynamicTexture.dirty texture true

    method update (next : Domain.battlefield_lighting) =
      let next_signature = if Lighting.daylight_only next then "day" else Lighting.key next in
      if
        next_signature <> signature
        || Set.size dirty > 0
        || Array.exists (fun page -> Option.is_none page.unlit) pages
      then (
        signature <- next_signature;
        let before = lighting in
        let global =
          not
            (Option.fold ~none:false
               ~some:(fun (old : Domain.battlefield_lighting) ->
                 old.time = next.time && old.underground = next.underground)
               before)
        in
        let changed = Set.create () in
        (match before with
        | None ->
            Array.iter
              (fun page ->
                let _ = Set.add changed page in
                ())
              pages
        | Some _ when global ->
            Array.iter
              (fun page ->
                let _ = Set.add changed page in
                ())
              pages
        | Some previous ->
            Array.iter
              (fun (light : Domain.battlefield_light) ->
                let x, y = Lighting.position light in
                let radius = max light.radius 25. in
                Array.iter
                  (fun page ->
                    let _ = Set.add changed page in
                    ())
                  (Phaser_spatial.query page_index
                     {
                       left = (x -. radius) *. scale;
                       right = (x +. radius) *. scale;
                       top = (y -. radius) *. scale;
                       bottom = (y +. radius) *. scale;
                     }))
              (Lighting.changed_lights previous.lights next.lights));
        lighting <- Some next;
        Phaser_spatial.clear light_index;
        if not (Lighting.daylight_only next) then
          Array.iteri
            (fun index (light : Domain.battlefield_light) ->
              let x, y = Lighting.position light in
              let radius = max light.radius 25. in
              Phaser_spatial.set light_index index
                { left = x -. radius; right = x +. radius; top = y -. radius; bottom = y +. radius })
            next.lights;
        Array.iter
          (fun record ->
            if Option.is_none record.unlit then record.unlit <- Some (self#make_source record.page);
            if Set.has dirty record then (
              Performance_trace.measure "Phaser terrain page rebuild" (fun () ->
                  self#rebuild record);
              let _ = Set.add changed record in
              ());
            if Set.has changed record then
              Performance_trace.measure "Phaser terrain page lighting" (fun () ->
                  self#shade record next))
          pages;
        Set.clear dirty;
        if Lighting.daylight_only next then pipeline#release;
        resources#publish)

    method private target record x y =
      let page = record.page in
      let target =
        match record.lit with
        | Some target -> target
        | None ->
            let target = Phaser.GameObjects.render_texture scene x y page.width page.height false in
            let _ = target##setOrigin 0. 0. in
            let _ =
              target##setDisplaySize
                (float_of_int page.width /. scale)
                (float_of_int page.height /. scale)
            in
            record.lit <- Some target;
            target
      in
      let _ = target##camera##setOrigin 0. 0. in
      let _ = target##camera##setZoom scale in
      let _ = target##camera##setScroll x y in
      target

    method private shade record (lighting : Domain.battlefield_lighting) =
      version <- version + 1;
      let page = record.page and source = Option.get record.unlit in
      Option.iter (fun (image : Phaser.image) -> image##destroy ()) record.display;
      record.display <- None;
      let x, y = position source in
      let target = self#target record x y in
      if Lighting.daylight_only lighting then (
        (* Daylight is stationary in world coordinates. Shade the padded page
           once at native resolution; display the cached result with Phaser's
           ordinary image pipeline at every camera position and pixel density. *)
        pipeline#configure ~x ~y lighting
          (float_of_int page.width /. scale)
          (float_of_int page.height /. scale);
        light_source source { Phaser_lighting.plain with lighting_camera = Some target##camera };
        let _ = target##clear () in
        Phaser.RenderTexture.draw_many target [| object_ source |];
        self#clip_outer target page;
        record.display <- Some (make_display scene page target##texture x y);
        resources#set ("terrain-lit:" ^ page.url) "lit terrain finals" 0.
          (float_of_int page.width *. float_of_int page.height *. 4.))
      else
        let local_bounds = region page in
        let indices =
          Phaser_spatial.query light_index
            {
              left = local_bounds.left /. scale;
              top = local_bounds.top /. scale;
              right = local_bounds.right /. scale;
              bottom = local_bounds.bottom /. scale;
            }
        in
        let _ = Host_collections.sort indices (fun[@u] a b -> compare a b) in
        let local = { lighting with lights = Array.map (Array.get lighting.lights) indices } in
        pipeline#configure ~x ~y local
          (float_of_int page.width /. scale)
          (float_of_int page.height /. scale);
        light_source source { Phaser_lighting.plain with lighting_camera = Some target##camera };
        let entries = [||] and temporary = [||] in
        Array.iter
          (fun (cell : Phaser_light_field.cell) ->
            let piece = Phaser.TextureImage.create scene x y (texture source) in
            let _ = piece##setOrigin 0. 0. in
            let _ =
              piece##setDisplaySize
                (float_of_int page.width /. scale)
                (float_of_int page.height /. scale)
            in
            let _ =
              piece##setCrop (float_of_int cell.x) (float_of_int cell.y) (float_of_int cell.width)
                (float_of_int cell.height)
            in
            let _ =
              Phaser_lighting.Pipeline_data.image piece "BattlefieldLighting"
                {
                  Phaser_lighting.plain with
                  lighting_camera = Some target##camera;
                  light_cell = Some cell;
                }
                false
            in
            let piece = Phaser.Upcast.image piece in
            let _ = Array_ext.append_value entries piece in
            let _ = Array_ext.append_value temporary piece in
            ())
          pipeline#cells;
        Fun.protect
          ~finally:(fun () ->
            Array.iter (fun (object_ : Phaser.game_object) -> object_##destroy ()) temporary)
          (fun () ->
            Array.iter
              (fun (light : Domain.battlefield_light) ->
                let x, y = Lighting.position light in
                let fixture = Option.value ~default:true light.fixture in
                (if fixture then
                   let brush = Phaser.GameObjects.graphics scene in
                   let _ = Array_ext.append_value temporary (Phaser.Upcast.graphics brush) in
                   let _ = Array_ext.append_value entries (Phaser.Upcast.graphics brush) in
                   let _ = brush##fillStyle 0x49361f 1. in
                   let _ = brush##fillRect (x -. 3.) (y -. 9.) 6. 12. in
                   let _ = brush##fillStyle 0xf6aa46 1. in
                   let _ = brush##fillRect (x -. 2.) (y -. 7.) 4. 5. in
                   let _ = brush##fillStyle 0xfff0b0 1. in
                   let _ = brush##fillRect (x -. 1.) (y -. 6.) 2. 3. in
                   let _ = brush##fillStyle 0x50402e 1. in
                   let _ = brush##fillRect (x -. 1.) (y +. 3.) 2. 8. in
                   ());
                let radius = if fixture then 20. else 25. in
                let halo =
                  Phaser.GameObjects.image scene x
                    (y -. if fixture then 5. else 0.)
                    "battlefield-white"
                in
                let _ = halo##setDisplaySize (radius *. 2.) (radius *. 2.) in
                let color =
                  match light.color with Some color -> color | None -> [| 255; 170; 70 |]
                in
                let _ =
                  Phaser_atmosphere.Halo.image halo "BattlefieldHalo" { fixture; color } false
                in
                let _ = halo##setBlendMode 3 in
                let halo = Phaser.Upcast.image halo in
                let _ = Array_ext.append_value temporary halo in
                let _ = Array_ext.append_value entries halo in
                ())
              local.lights;
            let _ = target##clear () in
            let _ = target##beginDraw () in
            Fun.protect
              ~finally:(fun () ->
                let _ = target##endDraw () in
                ())
              (fun () -> Phaser.RenderTexture.batch_many target entries));
        self#clip_outer target page;
        record.display <- Some (make_display scene page target##texture x y);
        resources#set ("terrain-lit:" ^ page.url) "lit terrain finals" 0.
          (float_of_int page.width *. float_of_int page.height *. 4.)

    method visible (bounds : Phaser_spatial.bounds) =
      let same =
        match visibility_signature with
        | Some (old, old_version) -> old_version = version && Phaser_spatial.same_bounds old bounds
        | None -> false
      in
      if not same then (
        visibility_signature <- Some (bounds, version);
        let records =
          Phaser_spatial.query page_index
            {
              left = bounds.left *. scale;
              top = bounds.top *. scale;
              right = bounds.right *. scale;
              bottom = bounds.bottom *. scale;
            }
        in
        let visible = Set.create () in
        Array.iter
          (fun record ->
            let _ = Set.add visible record in
            ())
          records;
        Set.each active (fun[@u] record ->
            if not (Set.has visible record) then
              Option.iter
                (fun (display : Phaser.image) ->
                  if not (Js.Nullable.isNullable display##parentContainer) then
                    Phaser_visibility.detach_root root (Phaser.Upcast.image display))
                record.display);
        Array.iter
          (fun record ->
            Option.iter
              (fun (display : Phaser.image) ->
                if Js.Nullable.isNullable display##parentContainer then
                  let _ = root##add (Phaser.Upcast.image display) in
                  ())
              record.display)
          records;
        active <- visible)

    method destroy =
      Array.iter
        (fun record ->
          Option.iter destroy_source record.unlit;
          Option.iter (fun (target : Phaser.render_texture) -> target##destroy ()) record.lit;
          Option.iter (fun (image : Phaser.image) -> image##destroy ()) record.display;
          resources#remove ("terrain-edit:" ^ record.page.url);
          resources#remove ("terrain-lit:" ^ record.page.url))
        pages;
      root##destroy ()
  end
