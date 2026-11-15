type cell = {
  x : int;
  y : int;
  width : int;
  height : int;
  lights : Domain.battlefield_light array;
  mutable packed_x : int;
  mutable packed_y : int;
}

type data = {
  mutable cell : cell;
  mutable origin_x : float;
  mutable origin_y : float;
  mutable pass : int;
  mutable kind : int;
}

type field = {
  factors : Phaser.texture option;
  offsets : Phaser.texture option;
  normalization : float;
}

module Pipeline_data = Phaser_pipeline.Data (struct
  type t = data
end)

let scale = Geometry.native_pixel_scale
let cell_size = 128
let default_warm = [| 1.02; 0.78; 0.49 |]
let default_offset = [| 22.; 14.; 7. |]

class light_field (scene : Phaser.scene) (resources : Phaser_resources.resources) =
  let renderer =
    match Js.Nullable.toOption scene##game##renderer with
    | Some renderer -> renderer
    | None -> failwith {js|Lighting requires the battlefield WebGL renderer|js}
  in
  let batch_size = max 1 (min 32 ((Webgl.parameter renderer##gl 0x8dfd - 24) / 3)) in
  object (self)
    val mutable pipeline : Phaser_pipeline.t option = None
    val mutable targets : Phaser.render_texture array array = [||]
    val mutable stamp : Phaser.image option = None
    val positions = Webgl.floats (batch_size * 4)
    val warm = Webgl.floats (batch_size * 3)
    val offsets = Webgl.floats (batch_size * 3)
    val mutable field_width = 0
    val mutable field_height = 0
    val mutable cells : cell array = [||]
    val mutable previous : Phaser.render_texture array option = None
    val mutable normalization = 1.

    initializer
      pipeline <-
        Some
          (Phaser_pipeline.create ~game:scene##game ~vertex:Phaser_shaders.light_field_vertex
             ~fragment:(Phaser_pipeline.shader_count Phaser_shaders.light_field_fragment batch_size)
             ~bind:(fun native _ object_ -> Option.iter (self#bind native) object_)
             ())

    method native =
      match pipeline with
      | Some pipeline -> Phaser_pipeline.native pipeline
      | None -> failwith {js|Lighting field is not initialized|js}

    method batch_size = batch_size
    method cells = cells
    method packed_size = (field_width * 4, field_height)

    method build (lighting : Domain.battlefield_lighting) width height origin_x origin_y =
      let cols = int_of_float (floor ((width *. scale) +. 0.5))
      and rows = int_of_float (floor ((height *. scale) +. 0.5)) in
      let lights = lighting.lights in
      normalization <-
        max 1.
          (Array.fold_left
             (fun sum (light : Domain.battlefield_light) -> sum +. light.power)
             0. lights);
      let next = [||] in
      let y = ref 0 in
      while !y < rows do
        let x = ref 0 in
        while !x < cols do
          let width = min cell_size (cols - !x) and height = min cell_size (rows - !y) in
          let local =
            Array_ext.filter
              (fun (light : Domain.battlefield_light) ->
                let px, py = Lighting.position light in
                (px +. light.radius -. origin_x) *. scale > float_of_int !x
                && (px -. light.radius -. origin_x) *. scale < float_of_int (!x + width)
                && (py +. (light.radius /. 1.12) -. origin_y) *. scale > float_of_int !y
                && (py -. (light.radius /. 1.12) -. origin_y) *. scale < float_of_int (!y + height))
              lights
          in
          let _ =
            Array_ext.append_value next
              { x = !x; y = !y; width; height; lights = local; packed_x = 0; packed_y = 0 }
          in
          x := !x + cell_size
        done;
        y := !y + cell_size
      done;
      cells <- next;
      let crowded = Array_ext.filter (fun cell -> Array.length cell.lights > batch_size) cells in
      Performance_trace.instant {js|Battlefield GPU lighting workload|js}
        ~args:
          (Performance_trace.obj
             [
               ({js|lights|js}, Performance_trace.int (Array.length lights));
               ({js|cells|js}, Performance_trace.int (Array.length cells));
               ({js|crowdedCells|js}, Performance_trace.int (Array.length crowded));
               ( {js|lightCellReferences|js},
                 Performance_trace.int
                   (Array.fold_left (fun count cell -> count + Array.length cell.lights) 0 cells) );
               ({js|batchSize|js}, Performance_trace.int batch_size);
             ]);
      if Array.length crowded = 0 then { factors = None; offsets = None; normalization }
      else
        let across =
          min
            (int_of_float (ceil (sqrt (float_of_int (Array.length crowded)))))
            (max 1 (resources#target / (cell_size * 4)))
        in
        let packed_width = across * cell_size
        and packed_height = (Array.length crowded + across - 1) / across * cell_size in
        let _ =
          resources#check ~target:true {js|packed GPU lighting scratch|js}
            (float_of_int (packed_width * 4))
            (float_of_int packed_height)
        in
        Array.iteri
          (fun index cell ->
            cell.packed_x <- index mod across * cell_size;
            cell.packed_y <- index / across * cell_size)
          crowded;
        if packed_width <> field_width || packed_height <> field_height then (
          self#release;
          field_width <- packed_width;
          field_height <- packed_height;
          targets <-
            Array.init 2 (fun _ ->
                Array.init 2 (fun _ ->
                    let target =
                      Phaser.GameObjects.render_texture scene 0. 0. (packed_width * 4) packed_height
                        false
                    in
                    let _ = target##setOrigin 0. 0. in
                    let _ = target##camera##setOrigin 0. 0. in
                    let _ = target##camera##setScroll 0. 0. in
                    let _ = target##texture##setFilter 1 in
                    target));
          resources#set {js|light-gpu-scratch|js} {js|GPU lighting scratch|js} 0.
            (float_of_int packed_width *. float_of_int packed_height *. 64.));
        let stamp =
          match stamp with
          | Some stamp -> stamp
          | None ->
              let created = Phaser.GameObjects.image scene 0. 0. {js|battlefield-white|js} in
              let _ = created##setOrigin 0. 0. in
              let _ =
                Pipeline_data.image created {js|BattlefieldLightField|js}
                  { cell = crowded.(0); origin_x; origin_y; pass = 0; kind = 0 }
                  false
              in
              stamp <- Some created;
              created
        in
        let data = Pipeline_data.get (Phaser.Upcast.image stamp) in
        data.origin_x <- origin_x;
        data.origin_y <- origin_y;
        let passes =
          Array.fold_left
            (fun passes cell ->
              max passes ((Array.length cell.lights + batch_size - 1) / batch_size))
            1 crowded
        in
        let output = ref targets.(0) in
        Performance_trace.measure {js|Phaser GPU lighting fields|js} (fun () ->
            for pass = 0 to passes - 1 do
              output := targets.(pass mod 2);
              previous <- (if pass > 0 then Some targets.((pass - 1) mod 2) else None);
              for kind = 0 to 1 do
                let target = !output.(kind) in
                Fun.protect
                  ~finally:(fun () ->
                    let _ = target##endDraw () in
                    ())
                  (fun () ->
                    let _ = target##clear () in
                    let _ = target##beginDraw () in
                    data.pass <- pass;
                    data.kind <- kind;
                    Array.iter
                      (fun cell ->
                        data.cell <- cell;
                        let _ =
                          stamp##setPosition
                            (float_of_int (cell.packed_x * 4))
                            (float_of_int cell.packed_y)
                        in
                        let _ =
                          stamp##setDisplaySize
                            (float_of_int (cell.width * 4))
                            (float_of_int cell.height)
                        in
                        let _ = target##batchDraw (Phaser.Upcast.image stamp) in
                        ())
                      crowded)
              done
            done);
        previous <- None;
        { factors = Some !output.(0)##texture; offsets = Some !output.(1)##texture; normalization }

    method private bind (native : Phaser.pipeline) object_ =
      native##flush ();
      let { cell; origin_x; origin_y; pass; kind } = Pipeline_data.get object_ in
      let start = pass * batch_size in
      let count = max 0 (min batch_size (Array.length cell.lights - start)) in
      for i = 0 to count - 1 do
        let light = cell.lights.(start + i) in
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
            | None -> default_warm.(channel));
          Webgl.set offsets
            ((i * 3) + channel)
            (match light.color with
            | Some colors -> float_of_int colors.(channel) /. 255. *. 30.
            | None -> default_offset.(channel))
        done
      done;
      let _ = native##set2f {js|uSize|js} (float_of_int field_width) (float_of_int field_height) in
      let _ =
        native##set4f {js|uCell|js} (float_of_int cell.x) (float_of_int cell.y)
          (float_of_int cell.width) (float_of_int cell.height)
      in
      let _ = native##set2f {js|uOrigin|js} origin_x origin_y in
      let _ =
        native##set2f {js|uAtlas|js} (float_of_int cell.packed_x) (float_of_int cell.packed_y)
      in
      let _ = native##set1f {js|uCount|js} (float_of_int count) in
      let _ = native##set1f {js|uKind|js} (float_of_int kind) in
      let _ = native##set1f {js|uNormalization|js} normalization in
      let _ = native##set1f {js|uHasPrevious|js} (if Option.is_some previous then 1. else 0.) in
      let _ = native##set4fv {js|uLights|js} positions in
      let _ = native##set3fv {js|uWarm|js} warm in
      let _ = native##set3fv {js|uOffset|js} offsets in
      let _ = native##set1i {js|uPrevious|js} 1 in
      let previous_texture =
        Option.bind previous (fun pair -> Phaser_pipeline.source_texture pair.(kind)##texture)
      in
      Phaser_pipeline.bind_texture native previous_texture 1;
      Phaser_pipeline.restore_texture native

    method release =
      Option.iter (fun (image : Phaser.image) -> image##destroy ()) stamp;
      stamp <- None;
      Array.iter (Array.iter (fun (target : Phaser.render_texture) -> target##destroy ())) targets;
      targets <- [||];
      field_width <- 0;
      field_height <- 0;
      resources#remove {js|light-gpu-scratch|js}
  end
