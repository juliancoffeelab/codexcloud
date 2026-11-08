module B = Bindings

let ok = function Ok x -> x | Error _ -> failwith "unexpected image error"

let rejects f =
  try
    f ();
    false
  with _ -> true

let () =
  assert (
    B.Image.create (Bytes.of_string "invalid") = Error B.Image_error.Invalid_png);
  let doomed = ok (B.Image.fixture ()) in
  let before_drop = B.drops () in
  assert (B.Texture.reject_upload doomed = Error B.Image_error.Too_large);
  assert (B.drops () = before_drop +. 1.);
  assert (
    rejects (fun () ->
        let _ = B.Image.checksum doomed in
        ()));
  let image = ok (B.Image.fixture ()) in
  let alias = image in
  let before = B.Image.checksum image in
  assert (
    rejects (fun () ->
        let _ = B.Image.blend image alias in
        ()));
  assert (B.Image.checksum image = before);
  assert (B.Image.crop image 0. 0. 0. 4. = Error B.Image_error.Invalid_crop);
  let other = ok (B.Image.fixture ()) in
  let _ = ok (B.Image.blend image other) in
  let cropped = ok (B.Image.crop image 0. 0. 4. 4.) in
  B.Image.dispose image;
  assert (
    rejects (fun () ->
        let _ = B.Image.checksum alias in
        ()));
  assert ((B.Image.snapshot cropped).width = 4.);
  assert (
    rejects (fun () ->
        let _ = B.Image.invalid_upload cropped cropped in
        ()));
  assert ((B.Image.snapshot cropped).height = 4.);
  B.Image.dispose cropped;
  B.Image.dispose other;
  for _ = 1 to 64 do
    let v = ok (B.Image.fixture ()) in
    B.Image.dispose v
  done
