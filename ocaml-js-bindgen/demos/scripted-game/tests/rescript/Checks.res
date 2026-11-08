module B = Bindings

let rejects = f =>
  try {
    f()
    false
  } catch {
  | _ => true
  }

let run = () => {
  let world = B.World.create() and hits = ref(0) and queued = ref(0)
  B.World.set_handler(world, (event: B.collision) => {
    incr(hits)
    assert(event.wall == B.Wall.Right)
    assert(rejects(() => ignore(B.World.snapshot(world))))
    assert(rejects(() => B.World.award(world, 1.)))
    assert(rejects(() => B.World.dispose(world)))
    queued := 10
  })
  for _ in 1 to 6 {
    ignore(B.World.tick(world))
  }
  assert(hits.contents == 1)
  B.World.award(world, float_of_int(queued.contents))
  assert(B.World.snapshot(world).score == 10.)
  B.World.clear_handler(world)
  B.World.set_handler(world, _ => failwith("script exception"))
  for _ in 1 to 6 {
    ignore(B.World.tick(world))
  }
  assert(rejects(() => ignore(B.World.tick(world))))
  ignore(B.World.snapshot(world))
  assert(B.Wasm.failure() == None)
  B.World.clear_handler(world)
  B.World.set_handler(world, _ => assert(false))
  B.World.set_handler(world, _ => ())
  for _ in 1 to 14 {
    ignore(B.World.tick(world))
  }
  B.World.dispose(world)
}
