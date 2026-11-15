open Save_json

let run_name catalog entry =
  let id = text entry {js|run|js} in
  match List.find_opt (fun (c : Domain.campaign) -> c.id = id) (Catalog.campaigns catalog) with
  | Some c -> c.name
  | None -> (
      match List.find_opt (fun (m : Domain.map_definition) -> m.id = id) (Catalog.maps catalog) with
      | Some m -> m.name
      | None -> {js|Saved game|js})

let chapter catalog entry =
  let number = int_of_float (n entry {js|mission|js}) in
  let map = text entry {js|mapId|js} in
  (number, try (Catalog.map catalog map).name with _ -> run_name catalog entry)

let roman value =
  let remaining = ref value and result = ref {js||js} in
  List.iter
    (fun (amount, symbol) ->
      while !remaining >= amount do
        (result :=
           let js_text_part_0 = !result in
           {j|$(js_text_part_0)$(symbol)|j});
        remaining := !remaining - amount
      done)
    [
      (1000, {js|M|js});
      (900, {js|CM|js});
      (500, {js|D|js});
      (400, {js|CD|js});
      (100, {js|C|js});
      (90, {js|XC|js});
      (50, {js|L|js});
      (40, {js|XL|js});
      (10, {js|X|js});
      (9, {js|IX|js});
      (5, {js|V|js});
      (4, {js|IV|js});
      (1, {js|I|js});
    ];
  !result

let turn_name catalog entry =
  let number, name = chapter catalog entry in
  if number = 0 then
    let js_text_part_1 = string_of_int (int_of_float (n entry {js|round|js})) in
    {j|Turn $(js_text_part_1)|j}
  else
    let js_text_part_0 = roman number in
    let js_text_part_4 = string_of_int (int_of_float (n entry {js|round|js})) in
    {j|$(js_text_part_0): $(name), $(js_text_part_4)|j}

type date

external date : float -> date = "Date" [@@mel.new]
external today : unit -> date = "Date" [@@mel.new]
external date_string : date -> string = "toDateString" [@@mel.send]
external local_date : date -> string = "toLocaleDateString" [@@mel.send]

external local_time :
  date -> string array -> < hour : string ; minute : string ; second : string > Js.t -> string
  = "toLocaleTimeString"
[@@mel.send]

let save_time value =
  let date = date value in
  let js_text_part_0 =
    if date_string date = date_string (today ()) then {js|Today|js} else local_date date
  in
  let js_text_part_2 =
    local_time date [||]
      [%mel.obj { hour = {js|2-digit|js}; minute = {js|2-digit|js}; second = {js|2-digit|js} }]
  in
  {j|$(js_text_part_0), $(js_text_part_2)|j}

let order a b =
  let c = Float.compare (n b {js|savedAt|js}) (n a {js|savedAt|js}) in
  if c <> 0 then c else Stdlib.compare (text a {js|id|js}) (text b {js|id|js})

type game = { id : string; latest : Js.Json.t; earliest : Js.Json.t }

let games sources =
  let grouped = Hashtbl.create 16 in
  Array.iter
    (fun entry ->
      let id = text entry {js|runId|js} in
      match Hashtbl.find_opt grouped id with
      | None -> Hashtbl.add grouped id { id; latest = entry; earliest = entry }
      | Some game ->
          Hashtbl.replace grouped id
            {
              game with
              latest = (if order entry game.latest < 0 then entry else game.latest);
              earliest = (if order entry game.earliest > 0 then entry else game.earliest);
            })
    sources;
  Hashtbl.fold (fun _ game result -> game :: result) grouped []
  |> List.sort (fun a b -> order a.latest b.latest)
  |> Array.of_list

let first_saved entry =
  let js_text_part_0 =
    if
      text entry {js|kind|js} = {js|turn|js}
      && b entry {js|missionStart|js}
      && n entry {js|ordinal|js} = 1.
    then {js|Started|js}
    else {js|Earliest save|js}
  in
  let js_text_part_2 = save_time (n entry {js|savedAt|js}) in
  {j|$(js_text_part_0) $(js_text_part_2)|j}
