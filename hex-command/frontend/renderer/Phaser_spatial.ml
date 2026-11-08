external push : 'a array -> 'a -> int = "push" [@@mel.send]

module Map = Host_collections.Map
module Set = Host_collections.Set

type bounds = { left : float; top : float; right : float; bottom : float }

let overlaps a b = a.left < b.right && a.right > b.left && a.top < b.bottom && a.bottom > b.top

type 'value record = { bounds : bounds; keys : string array }

type 'value t = {
  size : float;
  records : ('value, 'value record) Map.t;
  buckets : (string, 'value Set.t) Map.t;
}

let create ?(size = 256.) () = { size; records = Map.create (); buckets = Map.create () }

let keys t bounds =
  let result = [||] in
  for
    y = int_of_float (floor (bounds.top /. t.size))
    to int_of_float (floor (bounds.bottom /. t.size))
  do
    for
      x = int_of_float (floor (bounds.left /. t.size))
      to int_of_float (floor (bounds.right /. t.size))
    do
      let _ = push result (string_of_int x ^ ":" ^ string_of_int y) in
      ()
    done
  done;
  result

let same_bounds a b = a.left = b.left && a.top = b.top && a.right = b.right && a.bottom = b.bottom

let delete t value =
  Option.iter
    (fun old ->
      Array.iter
        (fun key ->
          Option.iter
            (fun bucket ->
              let _ = Set.delete bucket value in
              if Set.size bucket = 0 then
                let _ = Map.delete t.buckets key in
                ())
            (Map.get t.buckets key))
        old.keys;
      let _ = Map.delete t.records value in
      ())
    (Map.get t.records value)

let set t value bounds =
  match Map.get t.records value with
  | Some old when same_bounds old.bounds bounds -> ()
  | _ ->
      delete t value;
      let keys = keys t bounds in
      let _ = Map.set t.records value { bounds; keys } in
      Array.iter
        (fun key ->
          let bucket =
            match Map.get t.buckets key with Some bucket -> bucket | None -> Set.create ()
          in
          let _ = Set.add bucket value in
          let _ = Map.set t.buckets key bucket in
          ())
        keys

let query t bounds =
  let found = Set.create () in
  Array.iter
    (fun key ->
      Option.iter
        (fun bucket ->
          Set.each bucket (fun[@u] value ->
              if not (Set.has found value) then
                match Map.get t.records value with
                | Some record when overlaps record.bounds bounds ->
                    let _ = Set.add found value in
                    ()
                | _ -> ()))
        (Map.get t.buckets key))
    (keys t bounds);
  Set.values_array found

let clear t =
  Map.clear t.records;
  Map.clear t.buckets
