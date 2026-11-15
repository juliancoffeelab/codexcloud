open Json
(** Typed Talon columns and reductions; no Python process or precomputed facts.
*)

let column ?(default_value = `Null) name rows =
  let values =
    Array.of_list (List.map (fun r -> default r name default_value) rows)
  in
  let nonnull = Array.to_list values |> List.filter (( <> ) `Null) in
  let c =
    match List.find_opt (function `Float _ -> true | _ -> false) nonnull with
    | Some _ ->
        Talon.Column.of_options Talon.Type.float64
          (Array.map (function `Null -> None | x -> Some (number x)) values)
    | None -> (
        match nonnull with
        | (`Int _ | `Bool _) :: _ ->
            Talon.Column.of_options Talon.Type.int64
              (Array.map
                 (function `Null -> None | x -> Some (integer x))
                 values)
        | _ ->
            Talon.Column.of_options Talon.Type.string
              (Array.map
                 (function
                   | `Null -> None | `String x -> Some x | x -> Some (encode x))
                 values))
  in
  (name, c)

let table rows =
  let names =
    List.fold_left
      (fun a r ->
        List.fold_left
          (fun a (k, _) -> if List.mem k a then a else a @ [ k ])
          a (fields r))
      [] rows
  in
  Talon.v (List.map (fun n -> column n rows) names)

let rows table =
  let buffer = Buffer.create 4096 in
  let format =
    Talon_json.format ~framing:Talon_json.Array
      (Talon.Schema.columns (Talon.schema table))
  in
  Talon.Error.get_ok
    (Talon_json.encode format
       (Talon.Query.of_table table)
       (Bytesrw.Bytes.Writer.of_buffer buffer));
  list (parse (Buffer.contents buffer))

let group facts keys measures =
  if facts = [] then []
  else
    let t =
      Talon.v
        (List.map (fun name -> column name facts) keys
        @ List.map
            (fun name -> column ~default_value:(int 0) name facts)
            measures)
    in
    let expressions =
      List.map
        (fun n ->
          let (Talon.Type.Any ty) = Talon.Column.type_ (Talon.column t n) in
          match
            Talon.Kind.provably_equal (Talon.Type.kind ty) Talon.Kind.float
          with
          | Some _ -> Talon.Expr.(n := sum (Talon.Col.float n))
          | None -> Talon.Expr.(n := sum (Talon.Col.int n)))
        measures
    in
    Talon.Query.of_table t
    |> Talon.Query.aggregate ~by:keys expressions
    |> Talon.Query.sort (List.map Talon.Order.asc keys)
    |> Talon.Query.run |> Talon.Error.get_ok |> rows

type accumulator = {
  keys : string list;
  measures : string list;
  mutable pending : Json.t list;
  mutable size : int;
  mutable chunks : Json.t list list;
}

let accumulator keys measures =
  { keys; measures; pending = []; size = 0; chunks = [] }

let flush a =
  if a.size > 0 then (
    a.chunks <- group a.pending a.keys a.measures :: a.chunks;
    a.pending <- [];
    a.size <- 0)

let add a row =
  a.pending <- row :: a.pending;
  a.size <- a.size + 1;
  if a.size >= 8192 then flush a

let finish a =
  flush a;
  group (List.concat a.chunks) a.keys a.measures
