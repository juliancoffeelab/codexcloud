type halo_data = { fixture : bool; color : int array }

type blur_data = {
  x : float;
  y : float;
  region : float array;
  extent : float array;
  capture : float array option;
}

type mask_data = {
  width : float;
  height : float;
  origin_x : float;
  origin_y : float;
  extent : float array;
}

module Halo = Phaser_pipeline.Data (struct
  type t = halo_data
end)

module Blur = Phaser_pipeline.Data (struct
  type t = blur_data
end)

module Mask = Phaser_pipeline.Data (struct
  type t = mask_data
end)

let halo game =
  let previous = ref None in
  Phaser_pipeline.create ~game ~vertex:Phaser_shaders.halo_vertex
    ~fragment:Phaser_shaders.halo_fragment
    ~bind:(fun (native : Phaser.pipeline) _ object_ ->
      Option.iter
        (fun object_ ->
          let data = Halo.get object_ in
          let same =
            Option.fold ~none:false
              ~some:(fun (old : halo_data) ->
                old.fixture = data.fixture && Array_ext.equal Int.equal old.color data.color)
              !previous
          in
          if not same then (
            native##flush ();
            previous := Some data);
          let _ = native##set1f {js|uFixture|js} (if data.fixture then 1. else 0.) in
          let _ =
            native##set3f {js|uGlow|js}
              (float_of_int data.color.(0) /. 255.)
              (float_of_int data.color.(1) /. 255.)
              (float_of_int data.color.(2) /. 255.)
          in
          ())
        object_)
    ()

let fog_blur game =
  let previous = ref None in
  let kernel = Array.init 13 (fun i -> exp (-.float_of_int (i * i) /. 32.)) in
  let sum = (Array.fold_left ( +. ) 0. kernel *. 2.) -. kernel.(0) in
  let weights = Array.make 7 0. and offsets = Array.make 7 0. in
  weights.(0) <- kernel.(0) /. sum;
  for index = 1 to 6 do
    let i = (index * 2) - 1 in
    let weight = kernel.(i) +. kernel.(i + 1) in
    weights.(index) <- weight /. sum;
    offsets.(index) <- float_of_int i +. (kernel.(i + 1) /. weight)
  done;
  Phaser_pipeline.create ~game ~vertex:Phaser_shaders.fog_blur_vertex
    ~fragment:Phaser_shaders.fog_blur_fragment
    ~bind:(fun (native : Phaser.pipeline) _ object_ ->
      Option.iter
        (fun object_ ->
          let data = Blur.get object_ in
          if not (Option.fold ~none:false ~some:(fun old -> old = data) !previous) then (
            native##flush ();
            previous := Some data);
          let _ = native##set2f {js|uStep|js} data.x data.y in
          let _ =
            native##set4f {js|uRegion|js} data.region.(0) data.region.(1) data.region.(2)
              data.region.(3)
          in
          let _ = native##set2f {js|uExtent|js} data.extent.(0) data.extent.(1) in
          let x, y =
            match data.capture with None -> (0., 0.) | Some capture -> (capture.(0), capture.(1))
          in
          let _ = native##set2f {js|uCapture|js} x y in
          let _ = native##set1fv {js|uWeights|js} weights in
          let _ = native##set1fv {js|uOffsets|js} offsets in
          ())
        object_)
    ()

let fog_composite game =
  Phaser_pipeline.create ~game ~vertex:Phaser_shaders.fog_composite_vertex
    ~fragment:Phaser_shaders.fog_composite_fragment
    ~bind:(fun _ _ _ -> ())
    ()

let fog_mask game =
  Phaser_pipeline.create ~game ~vertex:Phaser_shaders.fog_mask_vertex
    ~fragment:Phaser_shaders.fog_mask_fragment
    ~bind:(fun (native : Phaser.pipeline) _ object_ ->
      Option.iter
        (fun object_ ->
          let data = Mask.get object_ in
          let _ = native##set2f {js|uWorldSize|js} data.width data.height in
          let _ = native##set2f {js|uWorldOrigin|js} data.origin_x data.origin_y in
          let _ = native##set2f {js|uExtent|js} data.extent.(0) data.extent.(1) in
          ())
        object_)
    ()
