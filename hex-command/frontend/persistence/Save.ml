open Browser_storage
open Save_json

module Storage = struct
  include Save_storage

  let repository stores mode work = Save_error.await (Save_storage.repository stores mode work)
  let initialized () = Save_error.await (Save_storage.initialized ())
  let initialize data = Save_error.await (Save_storage.initialize data)
  let read_session () = Save_error.await (Save_storage.read_session ())
  let store_save text = Save_error.await (Save_storage.store_save text)
end

let version = Save_format.version
let key = {js|hex-command-battle|js}
let rejected_key = {j|$(key)-unrestored|j}

external uuid : unit -> string = "randomUUID" [@@mel.scope "crypto"]
external now : unit -> float = "now" [@@mel.scope "Date"]

let lock name work = Save_error.catch (attempt (fun () -> request_lock name work))
let convert_storage = Storage_migrations.migrate
let inspect_storage = Storage_migrations.inspect
let repair_storage = Storage_migrations.repair
let current_save = Save_format.current_json

type snapshot = { raw : string; value : Js.Json.t; source : Domain.state option }

type t = {
  mutable suspended : bool;
  mutable generation : int;
  mutable queue : unit Js.Promise.t;
  mutable identity : Js.Json.t;
  mutable persistence : string;
  mutable failure : Save_error.t option;
  mutable history_error : string option;
  mutable problem : Js.Json.t option;
  mutable choose_new_game : bool;
  mutable undo : Save_undo.entry list;
  mutable snapshot : snapshot option;
  mutable changed : unit -> unit;
  mutable previous : string option;
  mutable start : bool;
  mutable pending : Js.Json.t option;
  mutable pending_explicit : Js.Json.t option;
}

type envelope = { checkpoint : Domain.saved_game; pending_enemy : bool; setup : Setup.t option }

type 'game loaded = {
  game : 'game;
  pending_enemy : bool;
  notice : string option;
  saves : t;
  setup : Setup.t;
  undo : Save_undo.entry list;
  entry : Js.Json.t option;
}

let identity () =
  obj
    [
      ({js|runId|js}, str (uuid ()));
      ({js|branchId|js}, str (uuid ()));
      ({js|checkpointId|js}, str (uuid ()));
      ({js|ordinal|js}, num 0.);
    ]

let identity_of entry =
  obj
    (List.map
       (fun key -> (key, field entry key))
       [ {js|runId|js}; {js|branchId|js}; {js|checkpointId|js}; {js|ordinal|js} ])

let create () =
  {
    suspended = false;
    generation = 0;
    queue = return ();
    identity = identity ();
    persistence = {js|Not saved yet|js};
    failure = None;
    history_error = None;
    problem = None;
    choose_new_game = false;
    undo = [];
    snapshot = None;
    changed = (fun () -> ());
    previous = None;
    start = true;
    pending = None;
    pending_explicit = None;
  }

let notify owner = owner.changed ()
let resume owner = owner.suspended <- false

let invalidate owner =
  owner.generation <- owner.generation + 1;
  owner.undo <- [];
  owner.snapshot <- None

let close owner =
  invalidate owner;
  owner.suspended <- true;
  owner.changed <- (fun () -> ())

let new_run owner =
  (match owner.problem with
  | Some p when text p {js|kind|js} = {js|ready|js} -> owner.problem <- None
  | _ -> ());
  invalidate owner;
  owner.identity <- identity ();
  owner.previous <- None;
  owner.start <- true;
  owner.choose_new_game <- false;
  owner.pending <- None;
  owner.pending_explicit <- None

let subscribe owner f = owner.changed <- f
let set_undo (owner : t) history = owner.undo <- history
let saved owner = owner.queue
let failure = Save_error.of_error

let failed owner error =
  owner.persistence <- {js|Local save failed|js};
  owner.failure <- Some (failure error);
  notify owner

let queue owner work =
  let next = Save_error.catch (Js.Promise.then_ (fun () -> attempt work) owner.queue) in
  owner.queue <- Js.Promise.catch (fun _ -> return ()) (Js.Promise.then_ (fun _ -> return ()) next);
  next

let current value =
  let value = Save_format.of_json value in
  {
    checkpoint = Save_format.checkpoint value;
    pending_enemy = Save_format.pending_enemy value;
    setup = Some (Setup.of_wire (Save_format.setup value));
  }

let migrate input =
  let value, changed = Save_migrations.migrate input in
  (current value, changed)

let read_source () =
  let result = obj [] in
  for index = 0 to storage_length - 1 do
    match Js.Nullable.toOption (storage_key index) with
    | Some key
      when key = {js|hex-command-battle|js} || key = rejected_key
           || key = {js|hex-command-progress-index|js}
           || key = {js|hex-command-choose-new-game|js}
           || Js.String.startsWith ~prefix:{js|hex-command-progress:|js} key ->
        Option.iter
          (fun value -> set result key (Js.Json.string value))
          (Js.Nullable.toOption (storage_get key))
    | _ -> ()
  done;
  result

let discard_source () =
  try Array.iter (fun (key, _) -> storage_remove key) (entries (read_source ())) with _ -> ()

let restore_text ~restore ~validate raw kind =
  let converted, migrated = Save_migrations.migrate (Js.Json.parseExn raw) in
  let save = current_save converted in
  if text (field (field save {js|progress|js}) {js|entry|js}) {js|kind|js} <> kind then
    invalid_arg {js|Unexpected saved checkpoint kind|js};
  let undo = Save_undo.restore save in
  if undo <> [] then (
    if b save {js|pendingEnemy|js} then
      invalid_arg {js|Pending enemy turns cannot retain move undo.|js};
    validate
      (obj
         [
           ({js|state|js}, field (field save {js|checkpoint|js}) {js|state|js});
           ({js|history|js}, field save {js|undo|js});
         ]));
  let envelope = current save in
  let game = restore envelope.checkpoint in
  (game, save, undo, migrated)

let database_source tx =
  let state = Storage.Raw.object_store tx {js|state|js}
  and turns = Storage.Raw.object_store tx {js|turns|js} in
  let current = Storage.Raw.request (Storage.Raw.get state {js|current|js})
  and rows = Storage.Raw.request (Storage.Raw.get_all (Storage.Raw.object_store tx {js|entries|js}))
  and keys = Storage.Raw.request (Storage.Raw.keys turns)
  and values = Storage.Raw.request (Storage.Raw.get_all turns) in
  let* storage_version = Storage.Raw.request (Storage.Raw.get state {js|initialized|js}) in
  let history_version = Storage_migrations.indexed_history_version (number storage_version) in
  let* current = current in
  let* rows = rows in
  let* keys = keys in
  let* values = values in
  let source =
    obj
      [
        ({js|hex-command-battle|js}, if missing current then Js.Json.null else current);
        ( {js|hex-command-progress-index|js},
          Js.Json.string
            (Js.Json.stringify
               (obj
                  [ ({js|version|js}, num history_version); ({js|entries|js}, Js.Json.array rows) ]))
        );
      ]
  in
  Array.iteri (fun i key -> set source {j|hex-command-progress:$(key)|j} values.(i)) keys;
  return source

let delete_database_records tx plan =
  let state = Storage.Raw.object_store tx {js|state|js}
  and rows = Storage.Raw.object_store tx {js|entries|js}
  and turns = Storage.Raw.object_store tx {js|turns|js} in
  Array.iter
    (fun value ->
      (* Storage keys are host strings; preserve their exact UTF-16 spelling. *)
      let key = host_string value in
      if key = {js|hex-command-battle|js} then (
        Storage.Raw.delete state {js|current|js};
        Storage.Raw.put state (bool true) {js|choose-new-game|js})
      else if key = {js|hex-command-progress-index|js} then Storage.Raw.clear rows
      else if Js.String.startsWith ~prefix:{js|hex-command-progress:|js} key then (
        let id = Js.String.slice ~start:21 ~end_:(21 + (Js.String.length key - 21)) key in
        Storage.Raw.delete turns id;
        Storage.Raw.delete rows id))
    (array (field plan {js|deleted|js}))

let initialize ~restore ~validate ~dispose ~stage () =
  lock {js|hex-command-storage-import|js} (fun () ->
      let* initialized = Storage.initialized () in
      if initialized then (
        let* () =
          Storage.Raw.transaction [| {js|state|js}; {js|entries|js}; {js|turns|js} |]
            {js|readwrite|js} (fun tx ->
              let state = Storage.Raw.object_store tx {js|state|js} in
              let* version = Storage.Raw.request (Storage.Raw.get state {js|initialized|js}) in
              if number version = 4. then return ()
              else
                let* () =
                  if number version <> 2. then return ()
                  else
                    let* source = database_source tx in
                    let plan = Storage_migrations.indexed_db_v2_to_v3 source in
                    delete_database_records tx plan;
                    Storage.Raw.put state (num 3.) {js|initialized|js};
                    return ()
                in
                let* source = database_source tx in
                stage := {js|data|js};
                let plan = Storage_migrations.indexed_db_v3_to_v4 source in
                stage := {js|commit|js};
                let entries = Storage.Raw.object_store tx {js|entries|js} in
                Storage.Raw.clear entries;
                Array.iter (Storage.Raw.put_entry entries) (array (field plan {js|entries|js}));
                Storage.Raw.put state (num 4.) {js|initialized|js};
                return ())
        in
        discard_source ();
        return ())
      else
        attempt (fun () ->
            let source = read_source () in
            stage := {js|data|js};
            let converted = convert_storage source in
            let validate_save save =
              let kind = text (field (field save {js|progress|js}) {js|entry|js}) {js|kind|js} in
              let game, _, _, _ = restore_text ~restore ~validate (Js.Json.stringify save) kind in
              dispose game
            in
            Option.iter validate_save (get converted {js|current|js});
            Array.iter validate_save (array (field converted {js|turns|js}));
            stage := {js|commit|js};
            let* () = Storage.initialize converted in
            discard_source ();
            return ()))

let load ~restore ~validate ~fresh ~dispose =
  let owner = create () in
  let blocked source kind error =
    owner.problem <-
      Some
        (obj
           [
             ({js|source|js}, str source);
             ({js|kind|js}, str kind);
             ({js|detail|js}, str (detail error));
           ]);
    owner.persistence <- {js|Saved data needs attention|js};
    return
      {
        game = fresh ();
        pending_enemy = false;
        notice = None;
        saves = owner;
        setup = Setup.default;
        undo = [];
        entry = None;
      }
  in
  let access error =
    let f = failure error in
    match f.kind with Save_error.Internal -> {js|read|js} | kind -> Save_error.kind_name kind
  in
  let stage = ref {js|access|js} in
  let* prepared =
    Js.Promise.catch
      (fun error -> return (Some error))
      (let* () = initialize ~restore ~validate ~dispose ~stage () in
       return None)
  in
  match prepared with
  | Some error ->
      let source =
        try
          if Array.length (entries (read_source ())) > 0 then {js|localStorage|js}
          else {js|indexedDB|js}
        with _ -> {js|localStorage|js}
      in
      blocked source (if !stage = {js|data|js} then {js|data|js} else access error) error
  | None -> (
      let* session =
        Js.Promise.catch
          (fun error -> return (Error error))
          (let* session = Storage.read_session () in
           return (Ok session))
      in
      match session with
      | Error error -> blocked {js|indexedDB|js} (access error) error
      | Ok (None, choose) ->
          owner.choose_new_game <- choose;
          return
            {
              game = fresh ();
              pending_enemy = false;
              notice = None;
              saves = owner;
              setup = Setup.default;
              undo = [];
              entry = None;
            }
      | Ok (Some raw, _) -> (
          let restored =
            try Ok (restore_text ~restore ~validate raw {js|flash|js}) with error -> Error error
          in
          match restored with
          | Error error -> blocked {js|indexedDB|js} {js|data|js} error
          | Ok (game, save, undo, migrated) -> (
              let* written =
                Js.Promise.catch
                  (fun error -> return (Some error))
                  (let* () =
                     if migrated then Storage.store_save (Js.Json.stringify save) else return ()
                   in
                   return None)
              in
              match written with
              | Some error ->
                  dispose game;
                  blocked {js|indexedDB|js} (access error) error
              | None ->
                  owner.identity <- identity_of (field (field save {js|progress|js}) {js|entry|js});
                  owner.persistence <- {js|Saved locally|js};
                  owner.start <- false;
                  owner.undo <- undo;
                  let state = field (field save {js|checkpoint|js}) {js|state|js} in
                  owner.previous <-
                    Some
                      (let js_text_part_0 = text state {js|mapId|js} in
                       let js_text_part_2 = string_of_float (n state {js|round|js}) in
                       {j|$(js_text_part_0):$(js_text_part_2)|j});
                  return
                    {
                      game;
                      pending_enemy = b save {js|pendingEnemy|js};
                      notice = None;
                      saves = owner;
                      setup = Setup.of_json (field save {js|setup|js});
                      undo;
                      entry = Some (field (field save {js|progress|js}) {js|entry|js});
                    })))

let describe state owner kind ~manual ~mission_start id parent =
  let campaign = field state {js|campaign|js} in
  let entry = copy owner.identity in
  List.iter
    (fun (key, value) -> set entry key value)
    [
      ({js|id|js}, str id);
      ({js|savedAt|js}, num (now ()));
      ( {js|run|js},
        if campaign = Js.Json.null then field state {js|mapId|js} else field campaign {js|id|js} );
      ({js|mapId|js}, field state {js|mapId|js});
      ( {js|mission|js},
        if campaign = Js.Json.null then Js.Json.null else num (n campaign {js|mission|js} +. 1.) );
      ({js|round|js}, field state {js|round|js});
      ({js|kind|js}, str kind);
      ({js|missionStart|js}, bool mission_start);
    ];
  if kind = {js|turn|js} then (
    set entry {js|manual|js} (bool manual);
    set entry {js|parentId|js} (str parent));
  entry

let envelope _owner setup checkpoint pending entry undo =
  let fields =
    [
      ({js|version|js}, num (float_of_int version));
      ({js|setup|js}, Setup.to_json setup);
      ({js|checkpoint|js}, checkpoint);
      ({js|pendingEnemy|js}, bool pending);
      ({js|progress|js}, obj [ ({js|entry|js}, entry) ]);
    ]
  in
  obj (if text entry {js|kind|js} = {js|flash|js} then ({js|undo|js}, undo) :: fields else fields)

let list owner =
  let* () = saved owner in
  if Option.fold ~none:false ~some:(fun p -> text p {js|kind|js} <> {js|ready|js}) owner.problem
  then
    Save_error.reject_internal
      {js|Review blocked saved data before opening normal save management.|js}
  else
    Storage.repository [ {js|entries|js} ] {js|readonly|js} (fun tx ->
        let* rows = Storage.entries tx in
        Array.sort
          (fun a b ->
            let c = Float.compare (Storage.Entry.saved_at b) (Storage.Entry.saved_at a) in
            if c <> 0 then c else Stdlib.compare (Storage.Entry.id a) (Storage.Entry.id b))
          rows;
        return (Array.map Storage.Entry.to_json rows))

let progress_text id =
  Storage.repository [ {js|turns|js} ] {js|readonly|js} (fun tx ->
      let* raw = Storage.turn tx id in
      match raw with
      | None -> Save_error.reject_internal {js|Saved turn is unavailable.|js}
      | Some text -> return text)

let add_turn save =
  Storage.repository [ {js|entries|js}; {js|turns|js} ] {js|readwrite|js} (fun tx ->
      Storage.write_turn tx
        (Storage.Entry.of_json (field (field save {js|progress|js}) {js|entry|js}))
        (Js.Json.stringify save);
      return ())

let update_turn save =
  Storage.repository [ {js|entries|js}; {js|turns|js} ] {js|readwrite|js} (fun tx ->
      let entry = Storage.Entry.of_json (field (field save {js|progress|js}) {js|entry|js}) in
      let* previous = Storage.entry tx (Storage.Entry.id entry) in
      match previous with
      | None -> Save_error.reject_internal {js|Saved turn is unavailable.|js}
      | Some _ ->
          Storage.write_turn tx entry (Js.Json.stringify save);
          return ())

let entry_id = Storage.Entry.id
let entry_run = Storage.Entry.run_id
let entry_time = Storage.Entry.saved_at
let entry_manual = Storage.Entry.manual
let entry_start = Storage.Entry.mission_start

type 'a entry_map
type id_set

external entry_map : unit -> 'a entry_map = "Map" [@@mel.new]
external map_get : 'a entry_map -> string -> 'a Js.Undefined.t = "get" [@@mel.send]
external map_set : 'a entry_map -> string -> 'a -> unit = "set" [@@mel.send]
external map_each : 'a entry_map -> (('a -> string -> unit)[@u]) -> unit = "forEach" [@@mel.send]
external id_set : unit -> id_set = "Set" [@@mel.new]
external keep_id : id_set -> string -> unit = "add" [@@mel.send]
external kept_id : id_set -> string -> bool = "has" [@@mel.send]

let prune ?run () =
  let time = now () in
  Storage.repository [ {js|entries|js}; {js|turns|js} ] {js|readwrite|js} (fun tx ->
      let* data = Storage.entries ?run tx in
      let keep = id_set () and latest = entry_map () and buckets = entry_map () in
      let newer a b =
        entry_time a > entry_time b || (entry_time a = entry_time b && entry_id a < entry_id b)
      in
      let update table key entry =
        match Js.Undefined.toOption (map_get table key) with
        | Some old when not (newer entry old) -> ()
        | _ -> map_set table key entry
      in
      Array.iter
        (fun value ->
          let e = value in
          let run = entry_run e in
          if not (entry_manual e) then update latest run e;
          if entry_manual e || entry_start e then keep_id keep (entry_id e)
          else
            let age = max 0. (time -. entry_time e) in
            if age < 300000. then keep_id keep (entry_id e)
            else
              let interval =
                if age < 3600000. then 300000.
                else if age < 86400000. then 3600000.
                else if age < 604800000. then 86400000.
                else if age < 2592000000. then 604800000.
                else 0.
              in
              if interval > 0. then
                update buckets
                  (let js_text_part_2 = string_of_float interval in
                   let js_text_part_4 = string_of_float (floor (entry_time e /. interval)) in
                   {j|$(run):$(js_text_part_2):$(js_text_part_4)|j})
                  e)
        data;
      List.iter
        (fun table -> map_each table (fun[@u] entry _ -> keep_id keep (entry_id entry)))
        [ latest; buckets ];
      Array.iter
        (fun value ->
          let id = entry_id value in
          if not (kept_id keep id) then Storage.delete_turn tx id)
        data;
      return ())

let share_array name before after previous current =
  match (Js.Json.decodeArray (field previous name), Js.Json.decodeArray (field current name)) with
  | Some old_json, Some new_json ->
      let unchanged = ref (Array.length before = Array.length after) in
      for index = 0 to Array.length after - 1 do
        if index < Array.length before && before.(index) == after.(index) then
          new_json.(index) <- old_json.(index)
        else unchanged := false
      done;
      if !unchanged then set current name (field previous name)
  | _ -> ()

let snapshot ?source owner raw =
  match owner.snapshot with
  | Some previous when previous.raw = raw ->
      owner.snapshot <- Some { previous with source };
      previous.value
  | previous ->
      let value = Js.Json.parseExn raw in
      (match (previous, source) with
      | Some { source = Some before; value = previous; _ }, Some after ->
          let previous = field previous {js|state|js} and current = field value {js|state|js} in
          share_array {js|tiles|js} before.Domain.tiles after.Domain.tiles previous current;
          share_array {js|units|js} before.units after.units previous current;
          share_array {js|factions|js} before.factions after.factions previous current;
          share_array {js|groundItems|js} before.ground_items after.ground_items previous current;
          share_array {js|log|js} before.log after.log previous current;
          share_array {js|conditionMemory|js} before.condition_memory after.condition_memory
            previous current
      | _ -> ());
      owner.snapshot <- Some { raw; value; source };
      value

let write_json ?(setup = Setup.default) ?source owner raw ~pending_enemy =
  if (not owner.suspended) && owner.problem = None && not owner.choose_new_game then
    try
      let checkpoint = snapshot ?source owner raw in
      let state = field checkpoint {js|state|js} in
      if
        text state {js|turn|js} = {js|player|js}
        || field state {js|winner|js} <> Js.Json.null
        || (n state {js|round|js} = 1. && Setup.first setup = Domain.Turn_phase.Ai)
      then (
        let generation = owner.generation in
        let undo =
          if pending_enemy then Js.Json.array [||] else Save_undo.saved checkpoint owner.undo
        in
        let point =
          let js_text_part_0 = text state {js|mapId|js} in
          let js_text_part_2 = string_of_float (n state {js|round|js}) in
          {j|$(js_text_part_0):$(js_text_part_2)|j}
        in
        let history =
          if (not pending_enemy) && owner.previous <> Some point then (
            let parent = text owner.identity {js|checkpointId|js} in
            let next = copy owner.identity in
            set next {js|checkpointId|js} (str (uuid ()));
            set next {js|ordinal|js} (num (n next {js|ordinal|js} +. 1.));
            let original = owner.identity in
            owner.identity <- next;
            let e =
              describe state owner {js|turn|js} ~manual:false ~mission_start:owner.start
                (text next {js|checkpointId|js}) parent
            in
            owner.identity <- original;
            let captured = envelope owner setup checkpoint false e (Js.Json.array [||]) in
            owner.identity <- next;
            owner.previous <- Some point;
            owner.start <- false;
            Some (next, captured))
          else None
        in
        let entry =
          describe state owner {js|flash|js} ~manual:false ~mission_start:false
            (text owner.identity {js|branchId|js})
            {js||js}
        in
        let flash = Js.Json.stringify (envelope owner setup checkpoint pending_enemy entry undo) in

        owner.persistence <- {js|Saving…|js};
        let current () = generation = owner.generation && not owner.suspended in
        let _ =
          queue owner (fun () ->
              if not (current ()) then return ()
              else
                let* () =
                  Js.Promise.catch
                    (fun error ->
                      if current () then failed owner error;
                      return ())
                    (let* () = Storage.store_save flash in
                     if current () then (
                       owner.persistence <-
                         (if owner.history_error = None then {js|Saved locally|js}
                          else {js|History not saved|js});
                       if owner.history_error = None then owner.failure <- None);
                     return ())
                in
                let* () =
                  if not (current ()) then return ()
                  else
                    match history with
                    | None -> return ()
                    | Some (_next, save) ->
                        Js.Promise.catch
                          (fun error ->
                            if current () then (
                              owner.pending <- Some save;
                              owner.history_error <- Some (detail error);
                              failed owner error);
                            return ())
                          (let* () = Js.Promise.catch (fun _ -> return ()) (prune ()) in
                           if not (current ()) then return ()
                           else
                             let* () = add_turn save in
                             if not (current ()) then return ()
                             else (
                               owner.pending <- None;
                               owner.history_error <- None;
                               let* () = Storage.store_save flash in
                               if current () then (
                                 owner.persistence <- {js|Saved locally|js};
                                 owner.failure <- None);
                               if current () then Js.Promise.catch (fun _ -> return ()) (prune ())
                               else return ()))
                in
                if current () then notify owner;
                return ())
        in
        ())
    with error -> failed owner error

let write ?setup owner checkpoint ~pending_enemy =
  write_json ?setup owner (Domain.checkpoint_to_json checkpoint) ~pending_enemy

let manual owner setup raw =
  attempt (fun () ->
      if owner.problem <> None || owner.choose_new_game then
        Save_error.reject_internal {js|Choose a game or delete blocked saved data before saving.|js}
      else
        let checkpoint = Js.Json.parseExn raw in
        let state = field checkpoint {js|state|js} in
        let generation = owner.generation in
        let entry =
          describe state owner {js|turn|js} ~manual:true ~mission_start:false (uuid ())
            (text owner.identity {js|checkpointId|js})
        in
        set entry {js|checkpointId|js} (field entry {js|id|js});
        let save = envelope owner setup checkpoint false entry (Js.Json.array [||]) in
        queue owner (fun () ->
            if owner.suspended || generation <> owner.generation then
              Save_error.reject_internal {js|This play session has changed.|js}
            else (
              owner.pending_explicit <- Some save;
              Js.Promise.catch
                (fun error ->
                  if generation = owner.generation then failed owner error;
                  reject error)
                (let* () = add_turn save in
                 if generation = owner.generation then owner.pending_explicit <- None;
                 return ()))))

let managed owner =
  if owner.suspended then Save_error.raise_internal {js|This play session has closed.|js};
  if Option.fold ~none:false ~some:(fun p -> text p {js|kind|js} <> {js|ready|js}) owner.problem
  then
    Save_error.raise_internal
      {js|Review blocked saved data before opening normal save management.|js}

let delete_progress owner id =
  queue owner (fun () ->
      managed owner;
      Storage.repository [ {js|entries|js}; {js|turns|js} ] {js|readwrite|js} (fun tx ->
          let* entry = Storage.entry tx id in
          match entry with
          | None -> Save_error.reject_internal {js|Saved turn is unavailable.|js}
          | Some _ ->
              Storage.delete_turn tx id;
              return ()))

let delete_run owner run =
  queue owner (fun () ->
      managed owner;
      Storage.repository [ {js|entries|js}; {js|turns|js} ] {js|readwrite|js} (fun tx ->
          let* source = database_source tx in
          let plan =
            repair_storage source
              (obj [ ({js|kind|js}, str {js|campaign|js}); ({js|runId|js}, str run) ])
          in
          delete_database_records tx plan;
          return ()))

let status owner =
  ( owner.persistence,
    owner.failure,
    owner.problem,
    owner.choose_new_game,
    text owner.identity {js|runId|js} )

let problem owner = owner.problem
let awaiting owner = owner.choose_new_game

let active_run owner =
  if owner.choose_new_game then None else Some (text owner.identity {js|runId|js})

let adopt_identity owner entry =
  owner.identity <- identity_of entry;
  owner.start <- false;
  owner.previous <-
    Some
      (let js_text_part_0 = text entry {js|mapId|js} in
       let js_text_part_2 = string_of_float (n entry {js|round|js}) in
       {j|$(js_text_part_0):$(js_text_part_2)|j})

let restored owner entry =
  adopt_identity owner entry;
  owner.problem <- None;
  owner.choose_new_game <- false;
  owner.pending <- None;
  owner.history_error <- None;
  owner.persistence <- {js|Saved locally|js};
  owner.failure <- None

let mark_idle owner =
  invalidate owner;
  owner.choose_new_game <- true;
  owner.pending <- None;
  owner.pending_explicit <- None;
  owner.failure <- None;
  owner.history_error <- None;
  owner.persistence <- {js|No saved game|js};
  notify owner

let flash owner =
  let* () = saved owner in
  let* raw, _ = Storage.read_session () in
  return
    (Option.map
       (fun raw ->
         let save, _ = Save_migrations.migrate (Js.Json.parseExn raw) in
         field (field save {js|progress|js}) {js|entry|js})
       raw)

let inspect owner validate =
  queue owner (fun () ->
      lock {js|hex-command-storage-import|js} (fun () ->
          let errors = ref [] in
          let* authoritative =
            Js.Promise.catch
              (fun error ->
                errors := [ str (detail error) ];
                return false)
              (Storage.initialized ())
          in
          let source =
            if
              authoritative
              || Option.fold ~none:false
                   ~some:(fun p -> text p {js|source|js} = {js|indexedDB|js})
                   owner.problem
            then
              Storage.Raw.transaction
                [| {js|state|js}; {js|entries|js}; {js|turns|js} |]
                {js|readonly|js} database_source
            else attempt (fun () -> return (read_source ()))
          in
          Js.Promise.catch
            (fun error ->
              return
                (obj
                   [
                     ({js|records|js}, arr []);
                     ({js|errors|js}, arr (!errors @ [ str (detail error) ]));
                     ({js|authoritative|js}, bool authoritative);
                     ({js|completedElsewhere|js}, bool false);
                   ]))
            (let* source = source in
             let rows = inspect_storage source in
             Array.iter
               (fun row ->
                 Option.iter
                   (fun save ->
                     (try validate save
                      with error ->
                        if get row {js|issue|js} = None then
                          set row {js|issue|js} (str {js|invalid_battle|js});
                        set row {js|error|js}
                          (str
                             (let js_text_part_0 =
                                let previous = text row {js|error|js} in
                                if previous = {js||js} then {js||js} else {j|$(previous)\n|j}
                              in
                              let js_text_part_1 = detail error in
                              {j|$(js_text_part_0)$(js_text_part_1)|j})));
                     let _ = remove row {js|save|js} in
                     ())
                   (get row {js|save|js}))
               rows;
             let completed =
               authoritative
               && Option.fold ~none:false
                    ~some:(fun p -> text p {js|source|js} = {js|localStorage|js})
                    owner.problem
             in
             if
               authoritative && !errors = []
               && Array.for_all (fun row -> get row {js|issue|js} = None) rows
             then (
               owner.problem <-
                 Some
                   (obj
                      [
                        ({js|kind|js}, str {js|ready|js});
                        ({js|source|js}, str {js|indexedDB|js});
                        ({js|detail|js}, str {js||js});
                      ]);
               notify owner);
             return
               (obj
                  [
                    ({js|records|js}, Js.Json.array rows);
                    ({js|errors|js}, arr !errors);
                    ({js|authoritative|js}, bool authoritative);
                    ({js|completedElsewhere|js}, bool completed);
                  ]))))

let repair owner target =
  queue owner (fun () ->
      lock {js|hex-command-storage-import|js} (fun () ->
          match owner.problem with
          | None -> Save_error.reject_internal {js|There is no blocked import to manage.|js}
          | Some problem ->
              if text problem {js|source|js} = {js|localStorage|js} then
                let* authoritative =
                  Js.Promise.catch (fun _ -> return false) (Storage.initialized ())
                in
                if authoritative then
                  Save_error.reject_internal
                    {js|Import finished in another tab. Import the saved game here before deleting anything.|js}
                else
                  attempt (fun () ->
                      let plan = repair_storage (read_source ()) target in
                      Array.iter
                        (fun key -> storage_remove (host_string key))
                        (array (field plan {js|deleted|js}));
                      Array.iter
                        (fun (key, value) ->
                          if value <> Js.Json.null then storage_set key (host_string value))
                        (entries (field plan {js|writes|js}));
                      return ())
              else
                Storage.Raw.transaction [| {js|state|js}; {js|entries|js}; {js|turns|js} |]
                  {js|readwrite|js} (fun tx ->
                    let state = Storage.Raw.object_store tx {js|state|js}
                    and rows = Storage.Raw.object_store tx {js|entries|js}
                    and turns = Storage.Raw.object_store tx {js|turns|js} in
                    if text target {js|kind|js} = {js|all|js} then (
                      Storage.Raw.clear state;
                      Storage.Raw.clear rows;
                      Storage.Raw.clear turns;
                      Storage.Raw.put state (num 4.) {js|initialized|js};
                      Storage.Raw.put state (bool true) {js|choose-new-game|js};
                      return ())
                    else
                      let* source = database_source tx in
                      let plan = repair_storage source target in
                      delete_database_records tx plan;
                      return ())))

type retry_policy = Before_restore | After_cleanup

let retry_pending policy owner =
  let explicit () =
    match owner.pending_explicit with
    | None -> return ()
    | Some save ->
        let* () = add_turn save in
        owner.pending_explicit <- None;
        return ()
  in
  let automatic () =
    match owner.pending with
    | None -> return ()
    | Some save ->
        let entry = field (field save {js|progress|js}) {js|entry|js} in
        if
          policy = After_cleanup
          && text entry {js|branchId|js} <> text owner.identity {js|branchId|js}
        then return ()
        else
          let* () = add_turn save in
          owner.pending <- None;
          owner.identity <- identity_of entry;
          owner.start <- false;
          owner.history_error <- None;
          return ()
  in
  let work () =
    match policy with
    | Before_restore when owner.problem <> None -> return ()
    | Before_restore ->
        let* () = explicit () in
        automatic ()
    | After_cleanup ->
        let* () = automatic () in
        explicit ()
  in
  (* A failed retry is data for the caller's policy, not failure of its load or
     already committed deletion. Keep unsuccessful snapshots pending. *)
  Js.Promise.catch
    (fun error -> return (Error (failure error)))
    (let* () = attempt work in
     return (Ok ()))

let chapter_start owner =
  owner.start <- true;
  owner.previous <- None

let capture () =
  let report = obj [] and errors = ref [] in
  let record operation error =
    errors :=
      !errors @ [ obj [ ({js|operation|js}, str operation); ({js|detail|js}, str (detail error)) ] ]
  in
  let* () =
    Js.Promise.catch
      (fun error ->
        record {js|indexedDB|js} error;
        return ())
      (let* stores =
         Storage.Raw.transaction [| {js|state|js}; {js|entries|js}; {js|turns|js} |]
           {js|readonly|js} (fun tx ->
             let result = obj [] in
             let rec read = function
               | [] -> return result
               | name :: rest ->
                   let store = Storage.Raw.object_store tx name in
                   let keys = Storage.Raw.request (Storage.Raw.keys store)
                   and values = Storage.Raw.request (Storage.Raw.get_all store) in
                   let* keys = keys in
                   let* values = values in
                   set result name
                     (obj (Array.to_list (Array.mapi (fun i key -> (key, values.(i))) keys)));
                   read rest
             in
             read [ {js|state|js}; {js|entries|js}; {js|turns|js} ])
       in
       set report {js|indexedDB|js} stores;
       return ())
  in
  (try set report {js|localStorage|js} (read_source ())
   with error -> record {js|localStorage migration source|js} error);
  set report {js|errors|js} (arr !errors);
  return report

let position owner map round =
  owner.previous <-
    Some
      (let js_text_part_2 = string_of_float round in
       {j|$(map):$(js_text_part_2)|j})

let owns_position owner entry =
  text owner.identity {js|runId|js} = text entry {js|runId|js}
  && text owner.identity {js|branchId|js} = text entry {js|branchId|js}

let inspect_progress owner ~current ~restore ~validate ~dispose ~inspect id =
  queue owner (fun () ->
      current ();
      let* raw = progress_text id in
      current ();
      let game, save, _, migrated = restore_text ~restore ~validate raw {js|turn|js} in
      Js.Promise.catch
        (fun error ->
          dispose game;
          reject error)
        (let* () = if migrated then update_turn save else return () in
         attempt (fun () ->
             let result = inspect game in
             dispose game;
             return result)))

(* The durable restoration transaction belongs to persistence. The embedding keeps
   its epoch guard and live battle adoption; no caller constructs storage identity. *)
let restore_progress owner ~current ~restore ~validate ~dispose ~install id =
  queue owner (fun () ->
      current ();
      let* raw = progress_text id in
      current ();
      let game, save, _, migrated = restore_text ~restore ~validate raw {js|turn|js} in
      Js.Promise.catch
        (fun error ->
          dispose game;
          failed owner error;
          reject error)
        (let* () = if migrated then update_turn save else return () in
         let* retry = retry_pending Before_restore owner in
         (match retry with
         | Ok () -> ()
         | Error failure -> failed owner (Save_error.exception_ failure));
         current ();
         let original = field (field save {js|progress|js}) {js|entry|js} in
         let identity = copy original in
         set identity {js|branchId|js} (str (uuid ()));
         let old_owner = create () in
         adopt_identity old_owner identity;
         let checkpoint = field save {js|checkpoint|js} in
         let state = field checkpoint {js|state|js} in
         let entry =
           describe state old_owner {js|flash|js} ~manual:false ~mission_start:false
             (text identity {js|branchId|js}) {js||js}
         in
         let setup = Setup.of_json (field save {js|setup|js}) in
         let flash = envelope old_owner setup checkpoint false entry (arr []) in
         let* () = Storage.store_save (Js.Json.stringify flash) in
         current ();
         restored owner identity;
         install game setup;
         return ()))

let reload_target owner source =
  let* current = flash owner in
  match current with
  | None -> return (Error {js|No saved checkpoint is available. Reloading would lose this game.|js})
  | Some entry when not (owns_position owner entry) ->
      return
        (Error
           {js|The saved checkpoint belongs to another game or branch. Open Saved games to choose it explicitly.|js})
  | Some _ when source = {js|checkpoint|js} -> return (Ok None)
  | Some _ when source <> {js|turn-start|js} -> return (Error {js|Unknown reload source.|js})
  | Some entry -> (
      let* history = list owner in
      let rec find seen id =
        if List.mem id seen then None
        else
          match Array.find_opt (fun turn -> text turn {js|id|js} = id) history with
          | Some turn when b turn {js|manual|js} -> (
              match Js.Json.decodeString (field turn {js|parentId|js}) with
              | Some parent -> find (id :: seen) parent
              | None -> None)
          | turn -> turn
      in
      match find [] (text entry {js|checkpointId|js}) with
      | Some turn
        when (not (b turn {js|manual|js})) && text turn {js|runId|js} = text entry {js|runId|js} ->
          return (Ok (Some (text turn {js|id|js})))
      | _ ->
          return
            (Error
               {js|The start of this turn is unavailable. Choose another turn in Saved games.|js}))
