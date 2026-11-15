open Json
(** Strict regression checks. Gameplay comparison retains ordered arrays and
    missing-versus-null distinctions; only documented run metadata is ignored.
*)

let run_metadata =
  [
    "fingerprint";
    "sourceCommit";
    "elapsedSeconds";
    "detailFormat";
    "turnTiming";
  ]

let outcome_metadata = [ "elapsedMs"; "detail" ]
let equal a b = sorted a = sorted b

let reports baseline current =
  let start = Unix.gettimeofday () in
  let a = read baseline and b = read current in
  Journal.require
    (get a "status" = str "complete" && get b "status" = str "complete")
    "Both reports must be complete";
  let index report =
    let seen = Hashtbl.create 2048 in
    List.iter
      (fun o ->
        let key = identity o in
        Journal.require
          (not (Hashtbl.mem seen key))
          ("Duplicate match configuration: " ^ key);
        Hashtbl.add seen key o)
      (list (get report "outcomes"));
    seen
  in
  let left = index a and right = index b in
  Journal.require
    (Hashtbl.length left = Hashtbl.length right
    && Hashtbl.fold (fun key _ yes -> yes && Hashtbl.mem right key) left true)
    "Match configurations differ";
  Journal.require
    (List.map identity (list (get a "outcomes"))
    = List.map identity (list (get b "outcomes")))
    "Outcome array order differs";
  Journal.require
    (equal
       (remove a ("outcomes" :: run_metadata))
       (remove b ("outcomes" :: run_metadata)))
    "Report aggregates or configuration differ";
  let identical = ref 0 in
  List.iter
    (fun o ->
      let other = Hashtbl.find right (identity o) in
      Journal.require
        (equal (remove o outcome_metadata) (remove other outcome_metadata))
        ("Compact gameplay differs: " ^ identity o);
      let old_ref = get o "detail" and new_ref = get other "detail" in
      let old = Journal.bytes (Filename.dirname baseline) old_ref
      and now = Journal.bytes (Filename.dirname current) new_ref in
      if old = now then incr identical
      else
        Journal.require
          (equal
             (Journal.decode old (text (get old_ref "file")))
             (Journal.decode now (text (get new_ref "file"))))
          ("Detailed gameplay differs: " ^ identity o))
    (list (get a "outcomes"));
  obj
    [
      ("equal", bool true);
      ("matches", int (Hashtbl.length left));
      ("byteIdenticalDetails", int !identical);
      ("outcomeOrderEqual", bool true);
      ( "comparison",
        str
          "all gameplay fields and ordered arrays; JSON object key order \
           ignored" );
      ("excludedRunMetadata", arr (List.map str run_metadata));
      ("excludedOutcomeMetadata", arr (List.map str outcome_metadata));
      ("baselineWallSeconds", get a "elapsedSeconds");
      ("currentWallSeconds", get b "elapsedSeconds");
      ("comparisonWallSeconds", float (Unix.gettimeofday () -. start));
    ]

let dump_metadata = [ "stateHash"; "elapsedMs"; "elapsedSeconds"; "detail" ]

let dumps baseline current =
  let open Talon in
  let load path =
    Query.of_source (Error.get_ok (Talon_arrow.file path))
    |> Query.run |> Error.get_ok
  in
  let clean table =
    let fields =
      Schema.columns (schema table)
      |> List.filter (fun (n, _) -> not (List.mem n dump_metadata))
      |> List.map (fun (n, _) -> (n, column table n))
    in
    let fields =
      List.map
        (fun (n, c) ->
          if n <> "missions" then (n, c)
          else
            match Column.layout c with
            | Column.Varsize list_layout -> (
                match Column.layout list_layout.child with
                | Column.Children record_layout ->
                    let fields =
                      List.filter
                        (fun (n, _) -> not (List.mem n dump_metadata))
                        record_layout.fields
                    in
                    let record_type =
                      Type.record
                        (List.map (fun (n, c) -> (n, Column.type_ c)) fields)
                    in
                    let get = function
                      | Ok x -> x
                      | Error (_, message) -> failwith message
                    in
                    let child =
                      get
                        (Column.of_layout (Type.Any record_type)
                           (Column.Children { record_layout with fields }))
                    in
                    ( n,
                      get
                        (Column.of_layout
                           (Type.Any (Type.list record_type))
                           (Column.Varsize { list_layout with child })) )
                | _ -> failwith "Invalid Arrow missions record")
            | _ -> failwith "Invalid Arrow missions list")
        fields
    in
    Talon.v ~rows:(Talon.rows table) fields
  in
  let a = load baseline and b = load current in
  let a = clean a and b = clean b in
  let keys = [ "kind"; "config" ] in
  let check table =
    let counts =
      Query.of_table table
      |> Query.aggregate ~by:keys Expr.[ "__count" := rows ]
      |> Query.run |> Error.get_ok
    in
    let values = Column.values Kind.int (column counts "__count") in
    Journal.require
      (Array.for_all (( = ) 1) values)
      "Duplicate match configuration";
    List.iter
      (fun k ->
        Journal.require
          (Column.null_count (column table k) = 0)
          "Missing match identity")
      keys
  in
  check a;
  check b;
  let sorted table =
    Query.of_table table
    |> Query.sort (List.map Order.asc keys)
    |> Query.run |> Error.get_ok
  in
  obj
    [
      ("equal", bool (Talon.equal (sorted a) (sorted b)));
      ("matches", int (Talon.rows a));
      ( "comparison",
        str "gameplay fields with ordered arrays; unordered match records" );
    ]
