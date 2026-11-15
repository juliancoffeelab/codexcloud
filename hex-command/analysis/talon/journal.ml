open Json
external sha256 : string -> string = "hex_analysis_sha256"
let require yes message = if not yes then failwith message
let adjacent name = name<>"" && name<>"." && name<>".." && Filename.basename name=name
let bytes directory reference =
  let name=text(get reference "file") in require (adjacent name) "Detail reference must name an adjacent journal";
  let offset=integer(get reference "offset") and length=integer(get reference "bytes") in
  require(offset>=0 && length>0) "Invalid detail byte range";
  let path=Filename.concat directory name in
  let channel,close = if Sys.file_exists path then let c=open_in_bin path in c,(fun()->close_in c)
    else let c=Unix.open_process_in("gzip -dc -- "^Filename.quote(path^".gz")) in c,(fun()->ignore(Unix.close_process_in c)) in
  Fun.protect ~finally:close (fun()->
    if Sys.file_exists path then seek_in channel offset else (let buffer=Bytes.create 65536 in let rec skip n=if n>0 then let size=min n 65536 in really_input channel buffer 0 size;skip(n-size) in skip offset);
    let raw=really_input_string channel length in
    require(sha256 raw=text(get reference "sha256")) ("Invalid gameplay checksum: "^path); raw)
let json_table table = Frame.rows table
let decode raw name =
  if not (Filename.check_suffix name ".arrow") then parse raw else
  let buffer=Rig.Buffer.of_string raw in
  let format=Talon_arrow.sniff buffer |> Talon.Error.get_ok in
  let metadata=Talon_arrow.metadata format in
  let version=Option.value ~default:"" (List.assoc_opt "hex-command.detail-format" metadata) in
  require(List.mem version (List.init 5(fun i->"arrow-ipc-snapshot-dictionaries-v"^string_of_int(i+1)))) "Unknown Arrow gameplay format";
  let table=Talon.Query.of_source (Talon_arrow.source format buffer) |> Talon.Query.run |> Talon.Error.get_ok in
  let rows=Array.of_list (json_table table) in require(Array.length rows>0) "Empty Arrow gameplay batch";
  let names=List.map fst(fields rows.(0)) in
  require(names=["header_json";"chapter";"phase_json";"before";"units";"ownership";"unit_snapshots";"ownership_snapshots"]) "Unexpected Arrow gameplay columns";
  let header=parse(text(get rows.(0) "header_json")) in
  let typed=version<>"arrow-ipc-snapshot-dictionaries-v1" in
  let units=Array.of_list(List.map(fun u->
    if not typed then parse(text u) else
    fields u |> List.filter_map(fun(k,v)->
      if v=`Null then None else if Filename.check_suffix k "_json" then
        let key=String.sub k 0 (String.length k-5) in let values=parse(text v) in
        require(match values with `List _ ->true|_->false) "Invalid Arrow unit array";
        if key="items" && values=arr [] then None else Some(key,values)
      else Some(k,v)) |> obj) (list(get rows.(0) "unit_snapshots"))) in
  let ownership=Array.of_list(List.map(fun v->if typed then v else parse(text v)) (list(get rows.(0) "ownership_snapshots"))) in
  let missions=Array.of_list(list(get header "missions")) in
  let phases=Array.make (max 1 (Array.length missions)) [] in
  for i=1 to Array.length rows-1 do
    let row=rows.(i) in let phase=parse(text(get row "phase_json")) in
    let roster key = get row key |> list |> List.map(fun idx->require(idx<>`Null && integer idx>=0 && integer idx<Array.length units) "Invalid Arrow roster index";units.(integer idx)) |> arr in
    let chapter=integer(get row "chapter") in
    require(chapter>=0 && chapter<Array.length phases && (text(get header "kind")<>"arena" || chapter=0)) "Invalid Arrow chapter";
    let index=integer(get row "ownership") in require(get row "ownership"<>`Null && index>=0 && index<Array.length ownership) "Invalid Arrow ownership index";
    let owners=if not typed then ownership.(index) else
      let initial=if text(get header "kind")="arena" then get header "initial" else get missions.(chapter) "initial" in
      let tiles=list(get initial "tiles") and values=list ownership.(index) in require(List.length tiles=List.length values) "Invalid Arrow ownership snapshot";
      arr(List.map2(fun tile owner ->
        let tile=if List.mem version ["arrow-ipc-snapshot-dictionaries-v4";"arrow-ipc-snapshot-dictionaries-v5"] then obj["col",get tile "col";"row",get tile "row"] else get tile "id" in
        let owner=if version="arrow-ipc-snapshot-dictionaries-v2" then (match owner with `Null->`Null|`Int 0->str "player"|`Int 1->str "enemy"|_->failwith "Invalid Arrow owner") else (match owner with `Null|`String _->owner|_->failwith "Invalid Arrow owner") in
        obj["tile",tile;"owner",owner]) tiles values) in
    phases.(chapter)<-merge phase ["before",roster "before";"units",roster "units";"ownership",owners]::phases.(chapter)
  done;
  if text(get header "kind")="arena" then set header "phases" (arr(List.rev phases.(0))) else
    set header "missions" (arr(Array.to_list(Array.mapi(fun i m->set m "phases" (arr(List.rev phases.(i)))) missions)))
let read directory reference = decode (bytes directory reference) (text(get reference "file"))
