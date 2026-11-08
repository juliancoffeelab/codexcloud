module B = Bindings

let () = {
  let calculator = B.Calculator.create()
  List.iter(
    key => B.Calculator.press(calculator, key),
    list{B.Key.One, B.Key.Add, B.Key.Two, B.Key.Equals},
  )
  assert(B.Calculator.display(calculator) == "3")
  B.Calculator.dispose(calculator)
  assert(try {
    ignore(B.Calculator.display(calculator))
    false
  } catch {
  | _ => true
  })
  assert(B.add(3l, 7l) == 10l)
  assert(B.add(Int32.max_int, 1l) == Int32.min_int)
  switch B.calculate(12., 4., "/") {
  | Ok({value, history}) => assert(value == 3. && Array.length(history) == 1)
  | Error(_) => assert(false)
  }
  switch B.calculate(12., 0., "/") {
  | Error(B.Calculation_error.Division_by_zero) => ()
  | _ => assert(false)
  }
  switch B.calculate(1., 2., "other") {
  | Error(B.Calculation_error.Invalid_operation({operation})) => assert(operation == "other")
  | _ => assert(false)
  }
  assert(B.nested(None) == None)
  assert(B.nested(Some(None)) == Some(None))
  assert(B.nested(Some(Some("λ🙂\x00"))) == Some(Some("λ🙂\x00")))
  let batch = B.Counter.batched_output()
  assert(batch.first == "\uFEFFλ" &&
  batch.id == 17. &&
  batch.flag &&
  batch.key == B.Key.Equals &&
  batch.second == "🦀\x00" &&
  batch.tail == 513)
  assert(B.Counter.current(batch.owner) == 23.)
  B.Counter.dispose(batch.owner)
  assert(B.int64(Int64.min_int) == Int64.min_int)
  assert(B.int64(Int64.max_int) == Int64.max_int)
  assert(B.bytes_sum(Bytes.of_string("abc")) == 294.)
  assert(B.string_length(B.unicode()) == 3.)
  assert(B.high_unsigned() == 4294967295.)
  assert(B.uint64(-1L) == -1L)
  assert(B.qualified(4294967295.) == 4294967295.)
  assert(B.qualified_bool(true))
  assert(B.generic_roundtrip({value: 4}, {value: "four"}) == ({value: "four"}, {value: 4}))
  assert(B.empty(B.Empty_value) == B.Empty_value)
  let server = B.HTTPServer.create()
  assert(B.HTTPServer.value(server) == 42.)
  B.HTTPServer.dispose(server)
  let alias = B.Counter.aliased(19.)
  assert(B.Counter.current(alias) == 19.)
  B.Counter.dispose(alias)
  let h = B.Counter.create(8.)
  let server = B.Counter.server(h)
  let copy = B.HTTPServer.counter(server)
  assert(B.Counter.current(copy) == 8.)
  B.Counter.dispose(copy)
  B.HTTPServer.dispose(server)
  assert(B.Counter.increment(h, 4.) == 12.)
  assert(B.Counter.current(h) == 12.)
  let rejected = try {
    ignore(B.Counter.same(h, h))
    false
  } catch {
  | _ => true
  }

  assert(rejected)
  assert(B.Counter.increment(h, 1.) == 13.)
  B.Counter.dispose(h)
  assert(try {
    ignore(B.Counter.current(h))
    false
  } catch {
  | _ => true
  })
  assert(try {
    ignore(B.divide(1., 0.))
    false
  } catch {
  | _ => true
  })
  assert(B.divide(12., 4.) == 3.)
  assert(B.writer("owner", "buffer") == "owner:buffer")
  let batch = B.Counter.many_counters(10.)
  Array.iter(B.Counter.dispose, batch)
  try {
    ignore(B.Counter.throw_owned())
    assert(false)
  } catch {
  | B.Counter.Throw_owned_error(h) =>
    assert(B.Counter.current(h) == 42.)
    B.Counter.dispose(h)
  }
  assert(try {
    ignore(B.Counter.invalid_output())
    false
  } catch {
  | _ => true
  })
  let r = B.Counter.counter_record()
  assert(r.label == "opaque in native record" && B.Counter.current(r.counter) == 17.)
  B.Counter.dispose(r.counter)
  let v = B.Vec2D.create()
  assert(B.Vec2D.dimension(v) == 2.)
  B.Vec2D.dispose(v)
  assert(B.scratch_name(40., "hi") == 42.)
  print_endline("calculator native ABI passed")
}

let () = {
  let data = B.sample_collections()
  assert(B.String_map.find("__proto__", data.words) == [1.])
  assert(B.String_map.find("constructor", data.words) == [2.])
  assert(List.map(fst, B.String_map.bindings(data.words)) ==
    list{"__proto__", "constructor", "\u{e000}", "\u{10000}"})
  assert(B.String_map.find("\u{10000}", data.words) == [4.])
  assert(List.map(fst, B.U64_map.bindings(data.unsigned)) == list{0L, -1L})
  assert(B.Collection_key_map.find({label: "same", rank: -1L}, data.points) == "max")
  assert(List.map(fst, B.Mixed_key_map.bindings(data.mixed)) ==
    list{B.Mixed_key.Payload(9.), B.Mixed_key.Empty})
  let unitKeys = B.unit_key_order()
  assert(List.map(fst, B.Unit_key_map.bindings(unitKeys)) ==
    list{B.Unit_key.Zed, B.Unit_key.Alpha, B.Unit_key.Middle})
  assert(B.Unit_key_map.find(B.Unit_key.Alpha, unitKeys) == 2.)
  assert(Hashtbl.find(data.counts, "__proto__") == 11.)
  Hashtbl.replace(data.counts, "constructor", 99.)
  let roundtrip = B.collections(data)
  assert(Hashtbl.find(roundtrip.counts, "constructor") == 99.)
  assert(B.String_map.bindings(roundtrip.words) == B.String_map.bindings(data.words))
  Hashtbl.add(data.counts, "constructor", 100.)
  assert(try {
    ignore(B.collections(data))
    false
  } catch {
  | _ => true
  })
  let owned = B.Counter.owned_map()
  assert(B.Counter.current(B.String_map.find("first", owned)) == 10.)
  B.String_map.iter((_, value) => B.Counter.dispose(value), owned)
  assert(B.Wasm.failure() == None)
}

module G = B.Geometry
module I = B.IntegerGeometry
module A = B.Arithmetic
module L = B.Labels
module E = B.EmptyNamespace
module D = B.Direction

let () = {
  assert(B.tuple_distance((0., 0.), (3., 4.)) == 5.)
  assert(B.tuple_bits((-1L, true)) == -1L)
  assert(B.tuple_bits((-1L, false)) == 0L)
  assert(B.nested_tuple(((-10l, 12), 0.5)) == 2.5)
  let text = "\u{feff}native λ🦀"
  assert(B.echo_text(text) == text)
  let a: G.point = {x: 0., y: 0.}
  let b: G.point = {x: 3., y: 4.}
  assert(G.distance(a, b) == 5.)
  assert(G.checked_distance(a, b) == 5.)
  assert(G.contains({min: a, max: b}, b))
  assert(!G.contains({min: a, max: b}, {x: 8., y: 9.}))
  assert(G.project(b, G.Y) == 4.)
  assert(G.echo([a, b]) == [a, b])
  assert(B.geometry_identity(b) == b)
  assert(G.origin() == a)
  let i: I.t = {x: 3, y: 4}
  assert(I.project(i, I.Y) == 4)
  assert(I.origin() == {x: 0, y: 0})
  assert(A.add(3., 4.) == 7. && A.dispose())
  let label: L.label<string> = {value: "native generic"}
  assert(label.value == "native generic")
  assert(B.label_identity(label) == label)
  assert(G.integer(b) == i && I.floating(i) == b)
  assert(D.flip(D.X) == D.Y)
  try {
    ignore(G.checked_distance({x: infinity, y: 0.}, b))
    assert(false)
  } catch {
  | G.Checked_distance_error(G.X) => ()
  }
  print_endline("ordinary module exports passed")
}

module T = B.Token
module Ts = B.Tokens
let () = {
  let before = T.drops()
  assert(Ts.parse("invalid") == Error("invalid token"))
  assert(Ts.optional(false) == None)
  let token = switch Ts.parse("13") {
  | Ok(token) => token
  | Error(_) => assert(false)
  }
  assert(T.value(token) == 13.)
  let other = switch Ts.optional(true) {
  | Some(token) => token
  | None => assert(false)
  }
  assert(T.value(other) == 42.)
  let batch = Ts.batch()
  assert(Array.map(T.value, batch) == [7., 9.])
  T.dispose(token)
  T.dispose(other)
  Array.iter(T.dispose, batch)
  assert(T.drops() == before +. 4.)
  print_endline("constructor-free owners and Drop passed")
}
