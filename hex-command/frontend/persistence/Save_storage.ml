module Raw = struct
  open Save_json

  type database
  type transaction
  type store
  type index
  type 'a request

  external open_db : string -> int -> database request = "open" [@@mel.scope "indexedDB"]
  external result : 'a request -> 'a = "result" [@@mel.get]
  external error : 'a request -> Foreign_error.t Js.Nullable.t = "error" [@@mel.get]
  external success : 'a request -> (unit -> unit) -> unit = "onsuccess" [@@mel.set]
  external failed : 'a request -> (unit -> unit) -> unit = "onerror" [@@mel.set]
  external upgrade : database request -> (unit -> unit) -> unit = "onupgradeneeded" [@@mel.set]
  external blocked : database request -> (unit -> unit) -> unit = "onblocked" [@@mel.set]
  external version_change : database -> (unit -> unit) -> unit = "onversionchange" [@@mel.set]
  external close_db : database -> unit = "close" [@@mel.send]
  external create_store : database -> string -> store = "createObjectStore" [@@mel.send]

  external create_keyed : database -> string -> < keyPath : string > Js.t -> store
    = "createObjectStore"
  [@@mel.send]

  external create_index : store -> string -> string -> unit = "createIndex" [@@mel.send]

  external begin_transaction :
    database -> string array -> string -> < durability : string > Js.t -> transaction
    = "transaction"
  [@@mel.send]

  external object_store : transaction -> string -> store = "objectStore" [@@mel.send]
  external complete : transaction -> (unit -> unit) -> unit = "oncomplete" [@@mel.set]
  external abort_handler : transaction -> (unit -> unit) -> unit = "onabort" [@@mel.set]
  external abort : transaction -> unit = "abort" [@@mel.send]
  external transaction_error : transaction -> Foreign_error.t Js.Nullable.t = "error" [@@mel.get]
  external get : store -> string -> Js.Json.t request = "get" [@@mel.send]
  external get_all : store -> Js.Json.t array request = "getAll" [@@mel.send]
  external index : store -> string -> index = "index" [@@mel.send]
  external index_all : index -> string -> Js.Json.t array request = "getAll" [@@mel.send]
  external keys : store -> string array request = "getAllKeys" [@@mel.send]
  external put : store -> Js.Json.t -> string -> unit = "put" [@@mel.send]
  external put_entry : store -> Js.Json.t -> unit = "put" [@@mel.send]
  external delete : store -> string -> unit = "delete" [@@mel.send]
  external clear : store -> unit = "clear" [@@mel.send]

  let request value =
    Js.Promise.make (fun ~resolve ~reject ->
        success value (fun () -> (resolve (result value) [@u]));
        failed value (fun () ->
            (reject (Save_error.exception_ (Save_error.of_error (error value))) [@u])))

  type idb_factory

  external current_factory : idb_factory = "indexedDB"

  let owner : idb_factory option ref = ref None
  let opening : database Js.Promise.t option ref = ref None

  let database () =
    if not (Option.fold ~none:false ~some:(fun owner -> owner == current_factory) !owner) then (
      opening := None;
      owner := Some current_factory);
    match !opening with
    | Some promise -> promise
    | None ->
        attempt (fun () ->
            let value = open_db {js|hex-command-saves|js} 1 in
            let promise =
              Js.Promise.make (fun ~resolve ~reject ->
                  let abandoned = ref false in
                  upgrade value (fun () ->
                      let db = result value in
                      let _ = create_store db {js|state|js} in
                      let entries =
                        create_keyed db {js|entries|js} [%mel.obj { keyPath = {js|id|js} }]
                      in
                      create_index entries {js|runId|js} {js|runId|js};
                      create_index entries {js|savedAt|js} {js|savedAt|js};
                      let _ = create_store db {js|turns|js} in
                      ());
                  success value (fun () ->
                      let db = result value in
                      if !abandoned then close_db db
                      else (
                        version_change db (fun () ->
                            close_db db;
                            opening := None);
                        resolve db [@u]));
                  failed value (fun () ->
                      opening := None;
                      reject (Save_error.exception_ (Save_error.of_error (error value))) [@u]);
                  blocked value (fun () ->
                      abandoned := true;
                      opening := None;
                      reject
                        (Save_error.exception_
                           (Save_error.blocked
                              {js|Close other Hex Command tabs to update save storage.|js})) [@u]))
            in
            opening := Some promise;
            promise)

  let transaction stores mode work =
    let* db = database () in
    attempt (fun () ->
        let tx = begin_transaction db stores mode [%mel.obj { durability = {js|strict|js} }] in
        let failure : exn option ref = ref None in
        let finished =
          Js.Promise.make (fun ~resolve ~reject ->
              complete tx (fun () ->
                  let completed = () in
                  (resolve completed [@u]));
              abort_handler tx (fun () ->
                  (reject
                     (match !failure with
                     | Some reason -> reason
                     | None -> Save_error.exception_ (Save_error.of_error (transaction_error tx)))
                   [@u])))
        in
        (* Install rejection handling before work: abort may race the request rejection. *)
        let settled =
          Js.Promise.catch
            (fun error -> return (Some error))
            (Js.Promise.then_ (fun () -> return None) finished)
        in
        Js.Promise.catch
          (fun reason ->
            let reason = Save_error.exception_ (Save_error.of_error reason) in
            failure := Some reason;
            (try abort tx with _ -> ());
            let* _ = settled in
            reject reason)
          (let* value = attempt (fun () -> work tx) in
           let* outcome = settled in
           match outcome with None -> return value | Some error -> reject error))

  let version_tx tx =
    let* value = request (get (object_store tx {js|state|js}) {js|initialized|js}) in
    if missing value then return None
    else if List.mem (number value) [ 2.; 3.; 4. ] then return (Some (number value))
    else Save_error.reject_internal {js|Unsupported save storage version.|js}

  let initialized_tx tx =
    let* version = version_tx tx in
    return (Option.is_some version)

  let initialized () = transaction [| {js|state|js} |] {js|readonly|js} initialized_tx

  let repository stores mode work =
    transaction
      (Array.of_list (List.sort_uniq Stdlib.compare ({js|state|js} :: stores)))
      mode
      (fun tx ->
        let* version = version_tx tx in
        if version <> Some 4. then
          Save_error.reject_internal
            {js|Saved progress needs to finish migrating before it can be written.|js}
        else work tx)

  let read_session () =
    repository [] {js|readonly|js} (fun tx ->
        let state = object_store tx {js|state|js} in
        let text = request (get state {js|current|js})
        and idle = request (get state {js|choose-new-game|js}) in
        let* text = text in
        let* idle = idle in
        return ((if missing text then None else Some (host_string text)), boolean idle))

  let store_save text =
    repository [] {js|readwrite|js} (fun tx ->
        let state = object_store tx {js|state|js} in
        put state (Js.Json.string text) {js|current|js};
        delete state {js|choose-new-game|js};
        return ())

  let initialize data =
    transaction [| {js|state|js}; {js|entries|js}; {js|turns|js} |] {js|readwrite|js} (fun tx ->
        let* ready = initialized_tx tx in
        if not ready then (
          let state = object_store tx {js|state|js} in
          Option.iter
            (fun save -> put state (Js.Json.string (Js.Json.stringify save)) {js|current|js})
            (Save_json.get data {js|current|js});
          if b data {js|chooseNewGame|js} then put state (bool true) {js|choose-new-game|js};
          Array.iter
            (fun save ->
              let entry = field (field save {js|progress|js}) {js|entry|js} in
              put_entry (object_store tx {js|entries|js}) entry;
              put (object_store tx {js|turns|js})
                (Js.Json.string (Js.Json.stringify save))
                (text entry {js|id|js}))
            (array (field data {js|turns|js}));
          put state (num 4.) {js|initialized|js});
        return ())
end

open Save_json

(** Normal repository records are written only after current-format loading or migration. Repair
    deliberately uses Raw, including malformed records. *)
module Entry = struct
  type t =
    < id : string
    ; runId : string
    ; branchId : string
    ; checkpointId : string
    ; ordinal : float
    ; savedAt : float
    ; run : string
    ; mission : float Js.Nullable.t
    ; round : float
    ; mapId : string
    ; kind : string
    ; missionStart : bool
    ; manual : bool
    ; parentId : string Js.Undefined.t >
    Js.t

  external id : t -> string = "id" [@@mel.get]
  external run_id : t -> string = "runId" [@@mel.get]
  external saved_at : t -> float = "savedAt" [@@mel.get]
  external manual : t -> bool = "manual" [@@mel.get]
  external mission_start : t -> bool = "missionStart" [@@mel.get]

  let of_json value : t =
    [%mel.obj
      {
        id = host_string (field value {js|id|js});
        runId = host_string (field value {js|runId|js});
        branchId = host_string (field value {js|branchId|js});
        checkpointId = host_string (field value {js|checkpointId|js});
        ordinal = n value {js|ordinal|js};
        savedAt = n value {js|savedAt|js};
        run = host_string (field value {js|run|js});
        mission = Js.Nullable.fromOption (Js.Json.decodeNumber (field value {js|mission|js}));
        round = n value {js|round|js};
        mapId = host_string (field value {js|mapId|js});
        kind = {js|turn|js};
        missionStart = b value {js|missionStart|js};
        manual = b value {js|manual|js};
        parentId =
          Js.Undefined.fromOption (Option.bind (get value {js|parentId|js}) Js.Json.decodeString);
      }]

  let to_json (value : t) =
    let fields =
      [
        ({js|id|js}, Js.Json.string value##id);
        ({js|runId|js}, Js.Json.string value##runId);
        ({js|branchId|js}, Js.Json.string value##branchId);
        ({js|checkpointId|js}, Js.Json.string value##checkpointId);
        ({js|ordinal|js}, num value##ordinal);
        ({js|savedAt|js}, num value##savedAt);
        ({js|run|js}, Js.Json.string value##run);
        ( {js|mission|js},
          Option.fold ~none:Js.Json.null ~some:Js.Json.number (Js.Nullable.toOption value##mission)
        );
        ({js|round|js}, num value##round);
        ({js|mapId|js}, Js.Json.string value##mapId);
        ({js|kind|js}, Js.Json.string value##kind);
        ({js|missionStart|js}, bool value##missionStart);
        ({js|manual|js}, bool value##manual);
      ]
    in
    obj
      (match Js.Undefined.toOption value##parentId with
      | None -> fields
      | Some parent -> ({js|parentId|js}, Js.Json.string parent) :: fields)
end

external get_string : Raw.store -> string -> string Js.Undefined.t Raw.request = "get" [@@mel.send]
external get_boolean : Raw.store -> string -> bool Js.Undefined.t Raw.request = "get" [@@mel.send]
external get_entry : Raw.store -> string -> Entry.t Js.Undefined.t Raw.request = "get" [@@mel.send]
external all_entries : Raw.store -> Entry.t array Raw.request = "getAll" [@@mel.send]
external indexed_entries : Raw.index -> string -> Entry.t array Raw.request = "getAll" [@@mel.send]
external put_string : Raw.store -> string -> string -> unit = "put" [@@mel.send]
external put_boolean : Raw.store -> bool -> string -> unit = "put" [@@mel.send]
external put_entry : Raw.store -> Entry.t -> unit = "put" [@@mel.send]

let outcome work =
  let pending = Save_error.attempt work in
  Js.Promise.catch
    (fun cause -> return (Error (Save_error.of_error cause)))
    (let* value = pending in
     return (Ok value))

let repository stores mode work = outcome (fun () -> Raw.repository stores mode work)
let initialized () = outcome Raw.initialized
let initialize data = outcome (fun () -> Raw.initialize data)

let current tx =
  let* value = Raw.request (get_string (Raw.object_store tx {js|state|js}) {js|current|js}) in
  return (Js.Undefined.toOption value)

let choose_new_game tx =
  let state = Raw.object_store tx {js|state|js} in
  Raw.delete state {js|current|js};
  put_boolean state true {js|choose-new-game|js}

let read_session () =
  repository [] {js|readonly|js} (fun tx ->
      let text = current tx in
      let idle =
        Raw.request (get_boolean (Raw.object_store tx {js|state|js}) {js|choose-new-game|js})
      in
      let* text = text in
      let* idle = idle in
      return (text, Js.Undefined.toOption idle = Some true))

let store_save text =
  repository [] {js|readwrite|js} (fun tx ->
      let state = Raw.object_store tx {js|state|js} in
      put_string state text {js|current|js};
      Raw.delete state {js|choose-new-game|js};
      return ())

let entries ?run tx =
  let store = Raw.object_store tx {js|entries|js} in
  Raw.request
    (match run with
    | None -> all_entries store
    | Some run -> indexed_entries (Raw.index store {js|runId|js}) run)

let entry tx id =
  let* value = Raw.request (get_entry (Raw.object_store tx {js|entries|js}) id) in
  return (Js.Undefined.toOption value)

let turn tx id =
  let* value = Raw.request (get_string (Raw.object_store tx {js|turns|js}) id) in
  return (Js.Undefined.toOption value)

let write_turn tx entry text =
  put_entry (Raw.object_store tx {js|entries|js}) entry;
  put_string (Raw.object_store tx {js|turns|js}) text (Entry.id entry)

let delete_turn tx id =
  Raw.delete (Raw.object_store tx {js|entries|js}) id;
  Raw.delete (Raw.object_store tx {js|turns|js}) id
