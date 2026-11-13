(* One white silhouette edge per loaded native sprite, shared by all its actors.
   Generated textures belong to the same Phaser game as their source texture. *)
type canvas
type context
type pixels

external canvas : string -> canvas = "createElement" [@@mel.scope "document"]
external width : canvas -> int -> unit = "width" [@@mel.set]
external height : canvas -> int -> unit = "height" [@@mel.set]
external context : canvas -> string -> context = "getContext" [@@mel.send]
external draw : context -> Phaser.image_element -> int -> int -> unit = "drawImage" [@@mel.send]
external read : context -> int -> int -> int -> int -> pixels = "getImageData" [@@mel.send]
external write : context -> pixels -> int -> int -> unit = "putImageData" [@@mel.send]
external data : pixels -> Lighting.pixels = "data" [@@mel.get]
external get : Lighting.pixels -> int -> float = "" [@@mel.get_index]
external set : Lighting.pixels -> int -> float -> unit = "" [@@mel.set_index]
external add : Phaser.textures -> string -> canvas -> Phaser.texture = "addCanvas" [@@mel.send]

let texture textures source =
  let key = "concealment-outline:" ^ source in
  if not (textures##exists key) then (
    let size = 64 in
    let image = Phaser.Texture.source_image (textures##get source) in
    let surface = canvas "canvas" in
    width surface size;
    height surface size;
    let ctx = context surface "2d" in
    draw ctx image 0 0;
    let original = data (read ctx 0 0 size size) in
    let edge = read ctx 0 0 size size in
    let output = data edge in
    let opaque x y =
      x >= 0 && y >= 0 && x < size && y < size && get original ((((y * size) + x) * 4) + 3) > 40.
    in
    for y = 0 to size - 1 do
      for x = 0 to size - 1 do
        let i = ((y * size) + x) * 4 in
        let border =
          (not (opaque x y))
          && (opaque (x - 1) y || opaque (x + 1) y || opaque x (y - 1) || opaque x (y + 1))
        in
        set output i 255.;
        set output (i + 1) 255.;
        set output (i + 2) 255.;
        set output (i + 3) (if border then 235. else 0.)
      done
    done;
    write ctx edge 0 0;
    ignore (add textures key surface));
  key
