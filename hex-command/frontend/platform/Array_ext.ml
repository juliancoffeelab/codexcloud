(* Frontend operations on the arrays supplied by Rust Vec/fixed-array bindings. *)
external append_value : 'a array -> 'a -> int = "push" [@@mel.send]
external filter_values : 'a array -> ('a -> bool) -> 'a array = "filter" [@@mel.send]

let filter predicate values = filter_values values predicate

let filter_map project values =
  let result = [||] in
  Array.iter
    (fun value ->
      match project value with
      | None -> ()
      | Some value ->
          let _ = append_value result value in
          ())
    values;
  result

let equal equal left right =
  let length = Array.length left in
  if length <> Array.length right then false
  else
    let rec loop index = index = length || (equal left.(index) right.(index) && loop (index + 1)) in
    loop 0
