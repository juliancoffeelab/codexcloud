module B = Bindings

let () =
  assert (B.hypotenuse 3. 4. = 5.);
  assert (B.hypotenuse 0. 0. = 0.);
  assert (B.hypotenuse (-3.) 4. = 5.)
