module E = Talon.Expr
(** Small typed-table vocabulary. Queries stay columnar until Report.write. *)

module Q = Talon.Query

let s = Talon.Col.string
let i = Talon.Col.int
let f = Talon.Col.float
let b = Talon.Col.bool
let out n e = E.(n := e)
let run q = Q.run q |> Talon.Error.get_ok
let names t = List.map fst (Talon.Schema.columns (Talon.schema t))
let has t n = List.mem n (names t)

let select ns t =
  Q.of_table t |> Q.select [ E.keep (Talon.Sel.names ns) ] |> run

let drop ns t = select (List.filter (fun n -> not (List.mem n ns)) (names t)) t
let derive os t = Q.of_table t |> Q.derive os |> run
let project os t = Q.of_table t |> Q.select os |> run
let filter p t = Q.of_table t |> Q.filter p |> run
let sort ns t = Q.of_table t |> Q.sort (List.map Talon.Order.asc ns) |> run
let aggregate by os t = Q.of_table t |> Q.aggregate ~by os |> run

let rename pairs t =
  Q.of_table t
  |> Q.select
       [
         E.each Talon.Sel.all
           {
             E.column =
               (fun name e ->
                 out (Option.value ~default:name (List.assoc_opt name pairs)) e);
           };
       ]
  |> run

let qselect ns q = Q.select [ E.keep (Talon.Sel.names ns) ] q

let qjoin ?(kind = Talon.Join.Inner) ?(one = false) ?(required = false) keys
    right left =
  Q.join ~kind
    ~each_left:
      (if required then Talon.Join.One
       else if one then Talon.Join.At_most_one
       else Talon.Join.Any)
    ~on:(Talon.Join.keys keys) (Q.of_table right) left

let join ?kind ?one ?required keys right left =
  Q.of_table left |> qjoin ?kind ?one ?required keys right |> run

let join_on ?(kind = Talon.Join.Inner) on right left =
  Q.of_table left |> Q.join ~kind ~on (Q.of_table right) |> run

let append a b = Q.of_table a |> Q.append (Q.of_table b) |> run

let concat = function
  | [] -> invalid_arg "concat: no schema"
  | t :: ts ->
      List.fold_left (fun q t -> Q.append (Q.of_table t) q) (Q.of_table t) ts
      |> run

let group keys fields t =
  aggregate keys (List.map (fun n -> out n E.(sum (f n))) fields) t |> sort keys

let count keys name t =
  aggregate keys [ out name E.(cast Talon.Type.float64 rows) ] t

let unique keys t = aggregate keys [] t

let unique_last keys t =
  let source = Q.of_table t |> Q.derive [ out "__row" E.(window rows) ] in
  let latest =
    source |> Q.aggregate ~by:keys [ out "__row" E.(max (i "__row")) ]
  in
  source
  |> Q.join ~on:(Talon.Join.keys (keys @ [ "__row" ])) latest
  |> qselect (names t)
  |> run

let all_maps t =
  let source = Q.of_table t in
  source |> Q.append (Q.derive [ out "map" (E.string "all") ] source) |> run

let zero = E.float 0.
let one = E.float 1.
let numeric n = E.coalesce [ f n; zero ]
let flag e = E.if_ e one zero
let nonleader = E.(coalesce [ s "role" <> string "leader"; bool true ])
let rate a n = E.(if_ (n > float 0.) (a /. n) (store Talon.Type.float64 null))
let maximum a b = E.if_ E.(a > b) a b

let facts keys fields stats t =
  project
    (E.keep (Talon.Sel.names keys)
    :: List.map
         (fun n -> out n (Option.value ~default:zero (List.assoc_opt n stats)))
         fields)
    t

module Builder = struct
  type column =
    | I of int option list ref
    | F of float option list ref
    | S of string option list ref
    | B of bool option list ref
    | SS of string array option list ref

  type kind = Int | Float | String | Bool | Strings
  type t = (string * column) list

  let create schema =
    List.map
      (fun (n, k) ->
        ( n,
          match k with
          | Int -> I (ref [])
          | Float -> F (ref [])
          | String -> S (ref [])
          | Bool -> B (ref [])
          | Strings -> SS (ref []) ))
      schema

  let add t get =
    List.iter
      (fun (n, c) ->
        let v = get n in
        let opt f = if v = `Null then None else Some (f v) in
        match c with
        | I a -> a := opt Json.integer :: !a
        | F a -> a := opt Json.number :: !a
        | S a -> a := opt Json.text :: !a
        | B a -> a := opt Json.truth :: !a
        | SS a ->
            a :=
              opt (fun x -> Array.of_list (List.map Json.text (Json.list x)))
              :: !a)
      t

  let finish t =
    Talon.v
      (List.map
         (fun (n, c) ->
           ( n,
             match c with
             | I a ->
                 Talon.Column.of_options Talon.Type.int64
                   (Array.of_list (List.rev !a))
             | F a ->
                 Talon.Column.of_options Talon.Type.float64
                   (Array.of_list (List.rev !a))
             | S a ->
                 Talon.Column.of_options Talon.Type.string
                   (Array.of_list (List.rev !a))
             | B a ->
                 Talon.Column.of_options Talon.Type.bool
                   (Array.of_list (List.rev !a))
             | SS a ->
                 Talon.Column.of_options
                   (Talon.Type.list Talon.Type.string)
                   (Array.of_list (List.rev !a)) ))
         t)
end
