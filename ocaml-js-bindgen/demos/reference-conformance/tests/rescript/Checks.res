module B = Bindings
let get = x =>
  switch x {
  | Ok(value) => value
  | Error(_) => failwith("unexpected host error")
  }
let run = () => {
  let value = get(B.parse_json("{\"name\":\"OCaml λ🦀\"}"))
  assert(B.identity(value) === value)
  assert(B.alias_identity(value) === value)
  assert(get(B.same(value, value)))
  let holder = B.Holder.create(value)
  assert(B.Holder.value(holder) === value)
  B.Holder.dispose(holder)
  switch B.parse_json("{") {
  | Error(_) => ()
  | Ok(_) => assert(false)
  }
  switch B.fail() {
  | Error(_) => ()
  | Ok(_) => assert(false)
  }
  assert(get(B.unsigned()) == Int64.minus_one)
  let buffer = get(B.buffer(16.))
  let bytes = get(B.bytes(buffer))
  assert(get(B.paint_bytes(bytes, 237)) === bytes)
  assert(get(B.read_byte(bytes, 15.)) == 237)
  let counter = get(B.library_counter(40.))
  assert(get(B.increment(counter, 2.)) == 42.)
  let calls = ref(0)
  B.notify(value, (event: B.host_event) => {
    assert(event.value === value)
    assert(event.label == "\uFEFFHost λ🦀")
    assert(event.sequence == 1.)
    incr(calls)
  })
  assert(calls.contents == 1)
  let native = get(B.text("OCaml λ🦀"))
  assert(get(B.copy_text(native, 12.)) == "OCaml λ🦀")
  assert(B.copy_text(native, 11.) == Error(B.Copy_failure.Too_large))
  value
}
