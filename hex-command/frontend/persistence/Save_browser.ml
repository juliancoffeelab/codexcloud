open Save_json

let run_name catalog entry =
  let id = text entry "run" in
  match List.find_opt (fun (c : Domain.campaign) -> c.id = id) (Catalog.campaigns catalog) with
  | Some c -> c.name
  | None -> (
      match List.find_opt (fun (m : Domain.map_definition) -> m.id = id) (Catalog.maps catalog) with
      | Some m -> m.name
      | None -> "Saved game")

let chapter catalog entry =
  let mission = text entry "mission" in
  let pieces = String.split_on_char ' ' mission in
  let number, map =
    match pieces with
    | "Mission" :: number :: "·" :: map -> (int_of_string number, String.concat " " map)
    | _ -> (0, mission)
  in
  (number, try (Catalog.map catalog map).name with _ -> run_name catalog entry)

let roman value =
  let remaining = ref value and result = ref "" in
  List.iter
    (fun (amount, symbol) ->
      while !remaining >= amount do
        result := !result ^ symbol;
        remaining := !remaining - amount
      done)
    [
      (1000, "M");
      (900, "CM");
      (500, "D");
      (400, "CD");
      (100, "C");
      (90, "XC");
      (50, "L");
      (40, "XL");
      (10, "X");
      (9, "IX");
      (5, "V");
      (4, "IV");
      (1, "I");
    ];
  !result

let turn_name catalog entry =
  let number, name = chapter catalog entry in
  if number = 0 then "Turn " ^ string_of_int (int_of_float (n entry "round"))
  else roman number ^ ": " ^ name ^ ", " ^ string_of_int (int_of_float (n entry "round"))

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
  Utf8.of_js
    ((if date_string date = date_string (today ()) then "Today" else local_date date)
    ^ ", "
    ^ local_time date [||] [%mel.obj { hour = "2-digit"; minute = "2-digit"; second = "2-digit" }])

let order a b =
  let c = Float.compare (n b "savedAt") (n a "savedAt") in
  if c <> 0 then c else String.compare (text a "id") (text b "id")

type game = { id : string; latest : Js.Json.t; earliest : Js.Json.t }

let games sources =
  let grouped = Hashtbl.create 16 in
  Array.iter
    (fun entry ->
      let id = text entry "runId" in
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
  (if text entry "kind" = "turn" && b entry "missionStart" && n entry "ordinal" = 1. then "Started"
   else "Earliest save")
  ^ " "
  ^ save_time (n entry "savedAt")
