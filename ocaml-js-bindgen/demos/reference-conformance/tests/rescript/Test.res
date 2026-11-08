module B = Bindings
module P = Js.Promise
let bind = (promise, continuation) => P.then_(continuation, promise)
let return = P.resolve
@scope("OcamljsTest") external done_: unit => unit = "done"
let run = () => {
  let value = Checks.run()
  let native = Checks.get(B.native_promise(value))
  bind(B.promise_identity(native), received => {
    assert(received === value)
    bind(B.alias_promise(native), received => {
      assert(received === value)
      bind(
        B.await_object(native),
        result => {
          assert(Checks.get(result) === value)
          bind(
            B.await_borrowed(native),
            result => {
              assert(Checks.get(result) === value)
              let calls = ref(0)
              bind(
                B.notify_after(
                  native,
                  (event: B.host_event) => {
                    assert(event.value === value)
                    assert(event.label == "\uFEFFHost λ🦀")
                    assert(event.sequence == 2.)
                    incr(calls)
                  },
                ),
                result => {
                  assert(calls.contents == 1)
                  assert(Checks.get(result) === value)
                  done_()
                  return()
                },
              )
            },
          )
        },
      )
    })
  })
}
let () = ignore(run())
