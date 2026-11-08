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

let run = done_ => {
  let before = B.live_entries()
  let corpus = B.Corpus.create() and other = B.Corpus.create() and input = B.Input.create()
  let pending = B.Index.create(corpus, input, _ => ())
  assert(rejects(() => ignore(B.Corpus.revision(corpus))))
  assert(rejects(() => B.Corpus.dispose(corpus)))
  assert(B.Corpus.revision(other) == 0.)
  assert(B.Input.push(input, "{\"text\":\"OCaml proof λ\"}\n") == Ok())
  B.Input.finish(input)
  observe(
    pending,
    x =>
      switch x {
      | Ok(index) =>
        assert(B.Index.query(index, "proof") == ["OCaml proof λ"])
        B.Index.dispose(index)
        B.Input.dispose(input)
        B.Corpus.dispose(corpus)
        B.Corpus.dispose(other)
        assert(B.live_entries() == before)
        let native_corpus = B.Corpus.create()
        let source = switch B.Blob.of_text("{\"text\":\"Blob proof λ🦀\"}\n") {
        | Ok(value) => value
        | Error(_) => assert(false)
        }
        let pending = B.Index.of_blob(native_corpus, source, _ => ())
        assert(rejects(() => B.Corpus.dispose(native_corpus)))
        observe(
          pending,
          x =>
            switch x {
            | Ok(native_index) =>
              assert(B.Index.query(native_index, "proof") == ["Blob proof λ🦀"])
              B.Index.dispose(native_index)
              B.Corpus.dispose(native_corpus)
              assert(B.live_entries() == before)
              done_(true)
            | _ => done_(false)
            },
          () => done_(false),
        )
      | _ => done_(false)
      },
    () => done_(false),
  )
}
