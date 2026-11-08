module B = Bindings

let ok = x =>
  switch x {
  | Ok(x) => x
  | Error(_) => failwith("unexpected image error")
  }

let rejects = f =>
  try {
    f()
    false
  } catch {
  | _ => true
  }

let () = {
  assert(B.Image.create(Bytes.of_string("invalid")) == Error(B.Image_error.Invalid_png))
  let doomed = ok(B.Image.fixture())
  let before_drop = B.drops()
  assert(B.Texture.reject_upload(doomed) == Error(B.Image_error.Too_large))
  assert(B.drops() == before_drop +. 1.)
  assert(rejects(() => ignore(B.Image.checksum(doomed))))
  let image = ok(B.Image.fixture())
  let alias = image
  let before = B.Image.checksum(image)
  assert(rejects(() => ignore(B.Image.blend(image, alias))))
  assert(B.Image.checksum(image) == before)
  assert(B.Image.crop(image, 0., 0., 0., 4.) == Error(B.Image_error.Invalid_crop))
  let other = ok(B.Image.fixture())
  ignore(ok(B.Image.blend(image, other)))
  let cropped = ok(B.Image.crop(image, 0., 0., 4., 4.))
  B.Image.dispose(image)
  assert(rejects(() => ignore(B.Image.checksum(alias))))
  assert(B.Image.snapshot(cropped).width == 4.)
  assert(rejects(() => ignore(B.Image.invalid_upload(cropped, cropped))))
  assert(B.Image.snapshot(cropped).height == 4.)
  B.Image.dispose(cropped)
  B.Image.dispose(other)
  for _ in 1 to 64 {
    let v = ok(B.Image.fixture())
    B.Image.dispose(v)
  }
}
