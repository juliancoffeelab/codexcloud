type t = Yojson.Basic.t
(** JSON is confined to the journal boundary and report presentation. *)

let obj x = `Assoc x
let arr x = `List x
let str x = `String x
let int x = `Int x
let float x = if Float.is_finite x then `Float x else `Null
let bool x = `Bool x
let fields = function `Assoc x -> x | _ -> []
let list = function `List x -> x | _ -> []
let get j k = Option.value ~default:`Null (List.assoc_opt k (fields j))
let has j k = List.mem_assoc k (fields j)
let text = function `String x -> x | `Int x -> string_of_int x | _ -> ""

let number = function
  | `Int x -> Float.of_int x
  | `Float x -> x
  | `Bool true -> 1.
  | _ -> 0.

let integer x = int_of_float (number x)

let truth = function
  | `Null | `Bool false -> false
  | `Int 0 | `Float 0. -> false
  | `String "" | `List [] | `Assoc [] -> false
  | _ -> true

let default j k d = if has j k then get j k else d
let set j k v = obj (List.filter (fun (n, _) -> n <> k) (fields j) @ [ (k, v) ])
let merge j vs = List.fold_left (fun j (k, v) -> set j k v) j vs

let remove j ks =
  obj (List.filter (fun (k, _) -> not (List.mem k ks)) (fields j))

let rec sorted = function
  | `Assoc x ->
      obj
        (List.map
           (fun (k, v) -> (k, sorted v))
           (List.sort (fun (a, _) (b, _) -> String.compare a b) x))
  | `List x -> arr (List.map sorted x)
  | x -> x

let encode = Yojson.Basic.to_string
let canonical x = encode (sorted x)
let parse = Yojson.Basic.from_string

let read path =
  if Filename.check_suffix path ".gz" then
    let ch = Unix.open_process_in ("gzip -dc -- " ^ Filename.quote path) in
    Fun.protect
      ~finally:(fun () ->
        if Unix.close_process_in ch <> Unix.WEXITED 0 then
          failwith ("gzip failed: " ^ path))
      (fun () -> Yojson.Basic.from_channel ch)
  else Yojson.Basic.from_file path

let write path j =
  let ch = open_out_bin path in
  Fun.protect
    ~finally:(fun () -> close_out ch)
    (fun () ->
      Yojson.Basic.to_channel ch j;
      output_char ch '\n')

let normalize ?(factions = []) value =
  let module Memo = Hashtbl.Make (struct
    type t = Yojson.Basic.t * Yojson.Basic.t

    let equal (a, p) (b, q) = a == b && p = q

    let hash (value, phase) =
      Hashtbl.hash
        ( get value "id",
          get value "type",
          get value "hp",
          get value "tile",
          phase )
  end) in
  let memo = Memo.create 1024 in
  let rec visit factions value =
    let cacheable = has value "faction" && has value "hp" && has value "tile" in
    let phase =
      if cacheable then
        get
          (Option.value ~default:`Null
             (List.assoc_opt (text (get value "faction")) factions))
          "phase"
      else `Null
    in
    let cache_key = (value, phase) in
    match if cacheable then Memo.find_opt memo cache_key else None with
    | Some result -> result
    | None -> (
        match value with
        | `List xs -> arr (List.map (visit factions) xs)
        | `Assoc xs ->
            let declared =
              match get value "factions" with
              | `List (_ :: _) as x -> x
              | _ -> get (get value "initial") "factions"
            in
            let factions =
              if truth declared then
                List.map (fun f -> (text (get f "id"), f)) (list declared)
              else factions
            in
            let out = obj (List.map (fun (k, v) -> (k, visit factions v)) xs) in
            let out =
              if
                has value "faction" && has value "hp" && has value "tile"
                && not (has value "side")
              then
                let f = List.assoc (text (get value "faction")) factions in
                set out "side" (get f "phase")
              else out
            in
            let out =
              List.fold_left
                (fun o k ->
                  if get o k = str "ai" then set o k (str "enemy") else o)
                out
                [ "side"; "turn"; "first" ]
            in
            let out =
              match get out "winner" with
              | `String "playerVictory" -> set out "winner" (str "player")
              | `String "playerDefeat" -> set out "winner" (str "enemy")
              | _ -> out
            in
            if cacheable then Memo.replace memo cache_key out;
            out
        | x -> x)
  in
  visit factions value

let identity o = text (get o "kind") ^ ":" ^ canonical (get o "config")

let label c side =
  let b = get c (side ^ "Blocks") in
  if truth b then
    String.concat "-"
      (List.map
         (fun k -> text (get b k))
         [ "recruiter"; "campaign"; "planner" ])
  else text (default c side (str "unrecorded"))

let other = function "player" -> "enemy" | _ -> "player"

let cell = function
  | `Assoc _ as x ->
      Printf.sprintf "%d,%d" (integer (get x "col")) (integer (get x "row"))
  | x -> text x

let sum xs = List.fold_left (fun n x -> n +. number x) 0. xs

let contains s sub =
  let n = String.length sub in
  let rec go i =
    i + n <= String.length s && (String.sub s i n = sub || go (i + 1))
  in
  go 0

let index_by key xs =
  let h = Hashtbl.create (List.length xs) in
  List.iter (fun x -> Hashtbl.replace h (text (get x key)) x) xs;
  h

let lookup h k = Option.value ~default:`Null (Hashtbl.find_opt h k)

let counts xs =
  let h = Hashtbl.create 16 in
  List.iter
    (fun x ->
      Hashtbl.replace h x (1 + Option.value ~default:0 (Hashtbl.find_opt h x)))
    xs;
  obj (Hashtbl.fold (fun k v a -> (k, int v) :: a) h [] |> List.sort compare)
