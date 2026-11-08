module B = Bindings
let get = function Ok value -> value | Error _ -> failwith "unexpected host error"
let run () =
  let value = get (B.parse_json "{\"name\":\"OCaml λ🦀\"}") in
  assert (B.identity value == value);
  assert (B.alias_identity value == value);
  assert (get (B.same value value));
  let holder = B.Holder.create value in
  assert (B.Holder.value holder == value);
  B.Holder.dispose holder;
  (match B.parse_json "{" with Error _ -> () | Ok _ -> assert false);
  (match B.fail () with Error _ -> () | Ok _ -> assert false);
  assert (get (B.unsigned ()) = Int64.minus_one);
  let buffer = get (B.buffer 16.) in
  let bytes = get (B.bytes buffer) in
  assert (get (B.paint_bytes bytes 237) == bytes);
  assert (get (B.read_byte bytes 15.) = 237);
  let counter = get (B.library_counter 40.) in
  assert (get (B.increment counter 2.) = 42.);
  let calls = ref 0 in
  B.notify value (fun (event : B.host_event) ->
    assert (event.value == value);
    assert (event.label = "\239\187\191Host λ🦀");
    assert (event.sequence = 1.);
    incr calls);
  assert (!calls = 1);
  let native = get (B.text "OCaml λ🦀") in
  assert (get (B.copy_text native 12.) = "OCaml λ🦀");
  assert (B.copy_text native 11. = Error B.Copy_failure.Too_large);
  value
