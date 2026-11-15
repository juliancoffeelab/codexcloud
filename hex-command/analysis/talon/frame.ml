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

(* Presentation/strict-decoder boundary: read cells directly. Serializing a
   table to JSON merely to parse it again doubles the work and allocations. *)
let rec json_column col =
  let open Talon in
  let (Type.Any ty) = Column.type_ col in
  let scalar kind encode =
    Array.map
      (function None -> `Null | Some v -> encode v)
      (Column.options kind col)
  in
  let values =
    match ty with
    | Type.Bool -> scalar Kind.bool Json.bool
    | Type.Int8 | Type.Int16 | Type.Int32 | Type.Int64 | Type.Uint8
    | Type.Uint16 | Type.Uint32 | Type.Uint64 ->
        scalar Kind.int Json.int
    | Type.Float16 | Type.Float32 | Type.Float64 -> scalar Kind.float Json.float
    | Type.String | Type.Categorical _ -> scalar Kind.string Json.str
    | Type.List _ -> (
        match Column.layout col with
        | Column.Varsize { offsets; child; _ } ->
            let child = json_column child in
            Array.init (Column.length col) (fun i ->
                let first = Int64.to_int (Nx.item [ i ] offsets) in
                let stop = Int64.to_int (Nx.item [ i + 1 ] offsets) in
                Json.arr (Array.to_list (Array.sub child first (stop - first))))
        | _ -> failwith "Invalid list layout")
    | Type.Record _ -> (
        match Column.layout col with
        | Column.Children { fields; length; _ } ->
            let fields = List.map (fun (k, c) -> (k, json_column c)) fields in
            Array.init length (fun i ->
                Json.obj (List.map (fun (k, v) -> (k, v.(i))) fields))
        | _ -> failwith "Invalid record layout")
    | _ -> failwith "Unsupported analysis output type"
  in
  match Column.validity col with
  | None -> values
  | Some mask ->
      Array.mapi (fun i v -> if Nx.item [ i ] mask then v else `Null) values

let rows table =
  let columns =
    List.map
      (fun (name, _) -> (name, json_column (Talon.column table name)))
      (Talon.Schema.columns (Talon.schema table))
  in
  List.init (Talon.rows table) (fun i ->
      obj (List.map (fun (k, v) -> (k, v.(i))) columns))

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

(* Boundary adapter for compact chapter observations; never used for journal
   event batches. Measures have a fixed numeric schema, including empty input. *)
let ingest rows keys measures =
  Talon.v
    (List.map (fun n -> column n rows) keys
    @ List.map
        (fun n ->
          ( n,
            Talon.Column.of_options Talon.Type.float64
              (Array.of_list
                 (List.map
                    (fun r ->
                      let v = get r n in
                      if v = `Null then None else Some (number v))
                    rows)) ))
        measures)
