let throw_js : Js.Exn.t -> 'a = [%mel.raw {|error => { throw error; }|}]
let raise_original error = match Js.Exn.asJsExn error with
  | Some original -> throw_js original
  | None -> raise error
