let observe = (promise, success, failure) => {
  let settled =
    promise
    |> Js.Promise.then_(value => Js.Promise.resolve(Ok(value)))
    |> Js.Promise.catch(_ => Js.Promise.resolve(Error()))

  ignore(
    settled |> Js.Promise.then_(result => {
      switch result {
      | Ok(value) => success(value)
      | Error() => failure()
      }
      Js.Promise.resolve()
    }),
  )
}

module B = Bindings

let rejects = f =>
  try {
    f()
    false
  } catch {
  | _ => true
  }

let () = {
  let notified = ref(0.)
  assert(B.numeric_notify(
    [1., 2.],
    progress => {notified.contents = progress.records},
    [3., 4.],
  ) == 10.)
  assert(notified.contents == 2.)

  observe(
    B.numeric_sum_after_yield([1., 2.], [3., 4.]),
    value => assert(value == 10.),
    () => assert(false),
  )
  observe(B.Arithmetic.twice(21.), value => assert(value == 42.), () => assert(false))
  let corpus = B.Corpus.create() and other = B.Corpus.create() and input = B.Input.create()
  let pending = B.Index.create(corpus, input, _ => ())
  assert(rejects(() => ignore(B.Corpus.revision(corpus))))
  assert(rejects(() => B.Corpus.dispose(corpus)))
  assert(B.Corpus.revision(other) == 0.)
  B.Input.finish(input)
  observe(
    pending,
    x =>
      switch x {
      | Ok(index) =>
        B.Index.dispose(index)
        B.Input.dispose(input)
        B.Corpus.dispose(corpus)
        B.Corpus.dispose(other)
      | Error(_) => assert(false)
      },
    () => assert(false),
  )
  observe(B.digest("λ🦀"), result => assert(result == Ok(2.)), () => assert(false))
  observe(
    B.borrowed_text("λ", Bytes.of_string("AB")),
    count => assert(count == 132.),
    () => assert(false),
  )
  observe(B.reject(), _ => assert(false), () => ())
}
