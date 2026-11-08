module Map = Host_collections.Map
module Set = Host_collections.Set

let detach_root (parent : Phaser.container) (root : Phaser.game_object) =
  if Option.is_some (Js.Undefined.toOption root##scene) then (
    let _ = parent##remove root false in
    root##removeFromDisplayList ();
    let _ = root##off "destroy" parent##onChildDestroyed parent in
    ())

type entry = {
  key : string;
  parent : Phaser.container;
  root : Phaser.game_object;
  mutable order : float;
  tie : int;
}

class visibility =
  object
    val index : entry Phaser_spatial.t = Phaser_spatial.create ()
    val records : (string, entry) Map.t = Map.create ()
    val mutable active : entry Set.t = Set.create ()
    val mutable tie = 0
    val mutable dirty = true
    val mutable previous : Phaser_spatial.bounds option = None

    method set key parent root bounds order =
      let entry =
        match Map.get records key with
        | Some entry -> entry
        | None ->
            let entry = { key; parent; root; order; tie } in
            tie <- tie + 1;
            let _ = Map.set records key entry in
            if
              Option.fold ~none:false
                ~some:(fun current -> current == parent)
                (Js.Nullable.toOption root##parentContainer)
            then detach_root parent root;
            entry
      in
      entry.order <- order;
      Phaser_spatial.set index entry bounds;
      dirty <- true

    method delete key =
      match Map.get records key with
      | None -> ()
      | Some entry ->
          Phaser_spatial.delete index entry;
          let _ = Map.delete records key in
          let _ = Set.delete active entry in
          dirty <- true

    method update bounds =
      if dirty || not (Option.fold ~none:false ~some:(Phaser_spatial.same_bounds bounds) previous)
      then (
        previous <- Some bounds;
        dirty <- false;
        let entries = Phaser_spatial.query index bounds in
        let visible = Set.create () and parents = Set.create () in
        Array.iter
          (fun entry ->
            let _ = Set.add visible entry in
            ())
          entries;
        Set.each active (fun[@u] entry ->
            if not (Set.has visible entry) then (
              detach_root entry.parent entry.root;
              let _ = Set.add parents entry.parent in
              ()));
        let _ =
          Host_collections.sort entries (fun[@u] a b ->
              let order = Float.compare a.order b.order in
              if order = 0 then a.tie - b.tie else order)
        in
        Array.iter
          (fun entry ->
            (if Js.Nullable.isNullable entry.root##parentContainer then
               let _ = entry.parent##add entry.root in
               ());
            let _ = Set.add parents entry.parent in
            ())
          entries;
        Set.each parents (fun[@u] parent ->
            let ordinal = ref 0 in
            Array.iter
              (fun entry ->
                if entry.parent == parent then (
                  (if (parent##list).(!ordinal) != entry.root then
                     let _ = parent##moveTo entry.root !ordinal in
                     ());
                  incr ordinal))
              entries);
        active <- visible)

    method clear =
      Set.each active (fun[@u] entry -> detach_root entry.parent entry.root);
      Map.clear records;
      Set.clear active;
      Phaser_spatial.clear index;
      dirty <- true
  end
