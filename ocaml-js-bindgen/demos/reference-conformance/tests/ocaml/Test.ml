module B = Bindings
module P = Js.Promise

let bind promise continuation = P.then_ continuation promise
let return = P.resolve

external done_ : unit -> unit = "done" [@@mel.scope "OcamljsTest"]

let run () =
  let value = Checks.run () in
  let native = Checks.get (B.native_promise value) in
  bind (B.promise_identity native) (fun received ->
      assert (received == value);
      bind (B.alias_promise native) (fun received ->
          assert (received == value);
          bind (B.await_object native) (fun result ->
              assert (Checks.get result == value);
              bind (B.await_borrowed native) (fun result ->
                  assert (Checks.get result == value);
                  let calls = ref 0 in
                  bind
                    (B.notify_after native (fun (event : B.host_event) ->
                         assert (event.value == value);
                         assert (event.label = "\239\187\191Host λ🦀");
                         assert (event.sequence = 2.);
                         incr calls))
                    (fun result ->
                      assert (!calls = 1);
                      assert (Checks.get result == value);
                      done_ ();
                      return ())))))

let () =
  let _ = run () in
  ()
