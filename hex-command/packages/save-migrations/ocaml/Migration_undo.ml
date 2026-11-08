open Migration_json

let safe key = not (List.mem key [ "__proto__"; "constructor"; "prototype" ])

let array_index key length =
  require
    (Js.Re.test ~str:(Migration_utf8.to_js key) (Js.Re.fromString "^(0|[1-9][0-9]*)$"))
    "Invalid undo index";
  let index = int_of_string key in
  require (index >= 0 && index < length) "Invalid undo index";
  index

let rec apply value change depth =
  let path = array (field change "path") in
  require (depth < Array.length path) "Invalid undo path";
  let key = string path.(depth) in
  require (safe key) "Invalid undo path";
  let final = depth = Array.length path - 1 in
  match Js.Json.decodeArray value with
  | Some xs ->
      let xs = Array.copy xs in
      let i = array_index key (Array.length xs) in
      xs.(i) <-
        (if final then (
           require (text change "op" = "set") "Invalid undo removal";
           field change "value")
         else apply xs.(i) change (depth + 1));
      Js.Json.array xs
  | None ->
      let result = copy value in
      if final then
        if text change "op" = "set" then set result key (field change "value")
        else (
          require (get result key <> None) "Invalid undo removal";
          remove result key)
      else set result key (apply (field value key) change (depth + 1));
      result

let children value =
  match Js.Json.decodeArray value with
  | Some xs -> Array.mapi (fun i v -> (string_of_int i, v)) xs
  | None -> entries value

let child value key =
  match Js.Json.decodeArray value with
  | Some xs ->
      let i = int_of_string key in
      if i < Array.length xs then Some xs.(i) else None
  | None -> if is_record value then get value key else None

let rec changes current previous path =
  if current == previous then []
  else
    let objects =
      (is_record current && is_record previous)
      ||
      match (Js.Json.decodeArray current, Js.Json.decodeArray previous) with
      | Some a, Some b -> Array.length a = Array.length b
      | _ -> false
    in
    if objects then
      let removed =
        Array.to_list (children current)
        |> List.filter_map (fun (key, v) ->
            if v <> undefined && Option.fold ~none:true ~some:is_undefined (child previous key) then
              Some (obj [ ("op", s "remove"); ("path", a (List.map s (path @ [ key ]))) ])
            else None)
      in
      removed
      @ (Array.to_list (children previous)
        |> List.concat_map (fun (key, v) ->
            if v = undefined then []
            else changes (Option.value (child current key) ~default:undefined) v (path @ [ key ])))
    else [ obj [ ("op", s "set"); ("path", a (List.map s path)); ("value", previous) ] ]

let convert value transform =
  let checkpoint = field value "checkpoint" in
  let previous = ref (field checkpoint "state") in
  let converted = ref (transform !previous) in
  let state = !converted in
  Option.iter
    (fun undo ->
      let history =
        List.rev (Array.to_list (array undo))
        |> List.map (fun step ->
            List.iter (fun change -> previous := apply !previous change 0) (list step "changes");
            let next = transform !previous in
            let result = update step [ ("changes", a (changes !converted next [])) ] in
            converted := next;
            result)
        |> List.rev
      in
      set value "undo" (a history))
    (present value "undo");
  set checkpoint "state" state
