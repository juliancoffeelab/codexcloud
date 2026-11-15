open Json

external sha256 : string -> string = "hex_analysis_sha256"

let require yes message = if not yes then failwith message

let adjacent name =
  name <> "" && name <> "." && name <> ".." && Filename.basename name = name

let bytes directory reference =
  let name = text (get reference "file") in
  require (adjacent name) "Detail reference must name an adjacent journal";
  let offset = integer (get reference "offset")
  and length = integer (get reference "bytes") in
  require (offset >= 0 && length > 0) "Invalid detail byte range";
  let path = Filename.concat directory name in
  let channel, close =
    if Sys.file_exists path then
      let c = open_in_bin path in
      (c, fun () -> close_in c)
    else
      let c =
        Unix.open_process_in ("gzip -dc -- " ^ Filename.quote (path ^ ".gz"))
      in
      (c, fun () -> ignore (Unix.close_process_in c))
  in
  Fun.protect ~finally:close (fun () ->
      (if Sys.file_exists path then seek_in channel offset
       else
         let buffer = Bytes.create 65536 in
         let rec skip n =
           if n > 0 then (
             let size = min n 65536 in
             really_input channel buffer 0 size;
             skip (n - size))
         in
         skip offset);
      let raw = really_input_string channel length in
      require
        (sha256 raw = text (get reference "sha256"))
        ("Invalid gameplay checksum: " ^ path);
      raw)

let json_table table = Frame.rows table

(* The gameplay contract is exactly one IPC stream record batch. Query.run
   canonicalizes batches, so count the validated message envelopes before
   materializing the table. The Arrow Message FlatBuffer fields are header_type
   (slot 1) and bodyLength (slot 3); Talon decodes all schemas and values.
   https://arrow.apache.org/docs/format/Columnar.html#encapsulated-message-format *)
let record_batches raw =
  let total = String.length raw in
  let bad () = failwith "Invalid Arrow gameplay message framing" in
  let rec loop position count =
    if position = total then count
    else (
      require (position >= 0 && position <= total - 8) "Truncated Arrow message";
      require
        (String.get_int32_le raw position = Int32.minus_one)
        "Gameplay journals require IPC stream framing";
      let size = Int32.to_int (String.get_int32_le raw (position + 4)) in
      if size = 0 then count
      else (
        require
          (size > 0 && size <= total - position - 8)
          "Invalid Arrow metadata length";
        let first = position + 8 in
        let last = first + size in
        let check p width =
          if p < first || width < 0 || p > last - width then bad ()
        in
        let u16 p =
          check p 2;
          String.get_uint16_le raw p
        in
        let i32 p =
          check p 4;
          Int32.to_int (String.get_int32_le raw p)
        in
        let message = first + i32 first in
        let vtable = message - i32 message in
        let vsize = u16 vtable in
        let slot index =
          let p = vtable + 4 + (2 * index) in
          if p + 2 > vtable + vsize then 0 else u16 p
        in
        let header_offset = slot 1 in
        let header =
          if header_offset = 0 then 0
          else (
            check (message + header_offset) 1;
            Char.code raw.[message + header_offset])
        in
        let body_offset = slot 3 in
        let body =
          if body_offset = 0 then 0L
          else (
            check (message + body_offset) 8;
            String.get_int64_le raw (message + body_offset))
        in
        require
          (body >= 0L && body <= Int64.of_int (total - last))
          "Invalid Arrow body length";
        loop (last + Int64.to_int body) (count + if header = 3 then 1 else 0)))
  in
  loop 0 0

let arrow raw =
  let buffer = Rig.Buffer.of_string raw in
  let format = Talon_arrow.sniff buffer |> Talon.Error.get_ok in
  let metadata = Talon_arrow.metadata format in
  let version =
    Option.value ~default:""
      (List.assoc_opt "hex-command.detail-format" metadata)
  in
  require
    (List.mem version
       (List.init 5 (fun i ->
            "arrow-ipc-snapshot-dictionaries-v" ^ string_of_int (i + 1))))
    "Unknown Arrow gameplay format";
  require (record_batches raw = 1) "Invalid Arrow gameplay batch count";
  let table =
    Talon.Query.of_source (Talon_arrow.source format buffer)
    |> Talon.Query.run |> Talon.Error.get_ok
  in
  require (Talon.rows table > 0) "Empty Arrow gameplay batch";
  require
    (List.map fst (Talon.Schema.columns (Talon.schema table))
    = [
        "header_json";
        "chapter";
        "phase_json";
        "before";
        "units";
        "ownership";
        "unit_snapshots";
        "ownership_snapshots";
      ])
    "Unexpected Arrow gameplay columns";
  (version, table)

let decode raw name =
  if not (Filename.check_suffix name ".arrow") then parse raw
  else
    let version, table = arrow raw in
    let strings name =
      Talon.Column.options Talon.Kind.string (Talon.column table name)
    in
    let header_strings = strings "header_json"
    and phase_strings = strings "phase_json" in
    let ints name =
      Talon.Column.options Talon.Kind.int (Talon.column table name)
    in
    let chapters = ints "chapter" and owner_indices = ints "ownership" in
    let indices name =
      Talon.Column.options
        (Talon.Kind.list Talon.Kind.int)
        (Talon.column table name)
    in
    let before_indices = indices "before" and unit_indices = indices "units" in
    let snapshots =
      Talon.Query.of_table table
      |> Talon.Query.select
           [
             Talon.Expr.keep
               (Talon.Sel.names [ "unit_snapshots"; "ownership_snapshots" ]);
           ]
      |> Talon.Query.slice ~offset:0 ~length:1
      |> Talon.Query.run |> Talon.Error.get_ok |> Frame.rows |> List.hd
    in
    require
      (List.for_all
         (fun name ->
           match get snapshots name with `List _ -> true | _ -> false)
         [ "unit_snapshots"; "ownership_snapshots" ])
      "Invalid Arrow snapshot dictionaries";
    let header = parse (Option.get header_strings.(0)) in
    let typed = version <> "arrow-ipc-snapshot-dictionaries-v1" in
    let units =
      Array.of_list
        (List.map
           (fun u ->
             if not typed then parse (text u)
             else
               fields u
               |> List.filter_map (fun (k, v) ->
                   if v = `Null then None
                   else if Filename.check_suffix k "_json" then (
                     let key = String.sub k 0 (String.length k - 5) in
                     let values = parse (text v) in
                     require
                       (match values with `List _ -> true | _ -> false)
                       "Invalid Arrow unit array";
                     if key = "items" && values = arr [] then None
                     else Some (key, values))
                   else Some (k, v))
               |> obj)
           (list (get snapshots "unit_snapshots")))
    in
    let ownership =
      Array.of_list
        (List.map
           (fun v -> if typed then v else parse (text v))
           (list (get snapshots "ownership_snapshots")))
    in
    let missions = Array.of_list (list (get header "missions")) in
    let phases = Array.make (max 1 (Array.length missions)) [] in
    for i = 1 to Array.length phase_strings - 1 do
      let phase = parse (Option.get phase_strings.(i)) in
      let roster indices =
        Option.get indices.(i)
        |> Array.to_list
        |> List.map (fun idx ->
            require
              (idx >= 0 && idx < Array.length units)
              "Invalid Arrow roster index";
            units.(idx))
        |> arr
      in
      let chapter = Option.get chapters.(i) in
      require
        (chapter >= 0
        && chapter < Array.length phases
        && (text (get header "kind") <> "arena" || chapter = 0))
        "Invalid Arrow chapter";
      let index = Option.get owner_indices.(i) in
      require
        (index >= 0 && index < Array.length ownership)
        "Invalid Arrow ownership index";
      let owners =
        if not typed then ownership.(index)
        else
          let initial =
            if text (get header "kind") = "arena" then get header "initial"
            else get missions.(chapter) "initial"
          in
          let tiles = list (get initial "tiles")
          and values = list ownership.(index) in
          require
            (List.length tiles = List.length values)
            "Invalid Arrow ownership snapshot";
          arr
            (List.map2
               (fun tile owner ->
                 let tile =
                   if
                     List.mem version
                       [
                         "arrow-ipc-snapshot-dictionaries-v4";
                         "arrow-ipc-snapshot-dictionaries-v5";
                       ]
                   then obj [ ("col", get tile "col"); ("row", get tile "row") ]
                   else get tile "id"
                 in
                 let owner =
                   if version = "arrow-ipc-snapshot-dictionaries-v2" then
                     match owner with
                     | `Null -> `Null
                     | `Int 0 -> str "player"
                     | `Int 1 -> str "enemy"
                     | _ -> failwith "Invalid Arrow owner"
                   else
                     match owner with
                     | `Null | `String _ -> owner
                     | _ -> failwith "Invalid Arrow owner"
                 in
                 obj [ ("tile", tile); ("owner", owner) ])
               tiles values)
      in
      phases.(chapter) <-
        merge phase
          [
            ("before", roster before_indices);
            ("units", roster unit_indices);
            ("ownership", owners);
          ]
        :: phases.(chapter)
    done;
    if text (get header "kind") = "arena" then
      set header "phases" (arr (List.rev phases.(0)))
    else
      set header "missions"
        (arr
           (Array.to_list
              (Array.mapi
                 (fun i m -> set m "phases" (arr (List.rev phases.(i))))
                 missions)))

let read directory reference =
  decode (bytes directory reference) (text (get reference "file"))
