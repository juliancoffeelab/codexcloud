(* Unknown-input adapter retained for the browser-era import tooling. The actual
   conversion and its frozen dependencies operate on typed OCaml values. *)
let import input =
  Historical_codec.Javascript1.of_json input
  |> Legacy_migrations.import |> Historical_codec.encode_save2_checkpoint_state

let wave_marker marker waves =
  Legacy_migrations.wave_marker marker (Array.map Legacy_data.wave waves)
