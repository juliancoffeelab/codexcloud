type image = Browser_image.t
type request = Art.frame -> image Js.Promise.t
type t = { images : (string, image) Host_collections.Map.t; mutable request : request option }

open Browser_image

let create () = { images = Host_collections.Map.create (); request = None }

let bind cache request =
  cache.request <- Some request;
  fun () ->
    match cache.request with
    | Some current when current == request -> cache.request <- None
    | _ -> ()

let adopt cache url image =
  let _ = Host_collections.Map.set cache.images url image in
  ()

let ready cache (frame : Art.frame) = Host_collections.Map.get cache.images frame.url

let load cache (frame : Art.frame) =
  match ready cache frame with
  | Some image -> Js.Promise.resolve image
  | None -> (
      match cache.request with
      | Some request -> request frame
      | None ->
          Js.Promise.reject
            (Failure
               (let js_text_part_1 = frame.url in
                {j|Artwork loader unavailable: $(js_text_part_1)|j})))

let retry = load

let close cache =
  cache.request <- None;
  Host_collections.Map.clear cache.images

let mount node image =
  class_name image {js|terrain-card-art pixel-art|js};
  alt image {js||js};
  width image 80;
  height image 80;
  replace node image
