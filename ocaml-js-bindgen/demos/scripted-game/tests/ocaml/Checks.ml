module B = Bindings

let rejects f =
  try
    f ();
    false
  with _ -> true

let run () =
  let world = B.World.create () and hits = ref 0 and queued = ref 0 in
  B.World.set_handler world (fun (event : B.collision) ->
      incr hits;
      assert (event.wall = B.Wall.Right);
      assert (
        rejects (fun () ->
            let _ = B.World.snapshot world in
            ()));
      assert (rejects (fun () -> B.World.award world 1.));
      assert (rejects (fun () -> B.World.dispose world));
      queued := 10);
  for _ = 1 to 6 do
    let _ = B.World.tick world in
    ()
  done;
  assert (!hits = 1);
  B.World.award world (float_of_int !queued);
  assert ((B.World.snapshot world).score = 10.);
  B.World.clear_handler world;
  B.World.set_handler world (fun _ -> failwith "script exception");
  for _ = 1 to 6 do
    let _ = B.World.tick world in
    ()
  done;
  assert (
    rejects (fun () ->
        let _ = B.World.tick world in
        ()));
  let _ = B.World.snapshot world in
  assert (B.Wasm.failure () = None);
  B.World.clear_handler world;
  B.World.set_handler world (fun _ -> assert false);
  B.World.set_handler world (fun _ -> ());
  for _ = 1 to 14 do
    let _ = B.World.tick world in
    ()
  done;
  B.World.dispose world
