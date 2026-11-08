type blob
type anchor
type options = { kind : string [@mel.as "type"] }

external blob : string array -> options -> blob = "Blob" [@@mel.new]
external object_url : blob -> string = "createObjectURL" [@@mel.scope "URL"]
external revoke : string -> unit = "revokeObjectURL" [@@mel.scope "URL"]
external anchor : string -> anchor = "createElement" [@@mel.scope "document"]
external set_href : anchor -> string -> unit = "href" [@@mel.set]
external set_download : anchor -> string -> unit = "download" [@@mel.set]
external append : anchor -> unit = "append" [@@mel.scope "document", "body"]
external click : anchor -> unit = "click" [@@mel.send]
external remove : anchor -> unit = "remove" [@@mel.send]
external later : (unit -> unit) -> int -> unit = "setTimeout"

let json ?(revoke_after = 60000) filename text =
  let url = object_url (blob [| text |] { kind = "application/json" }) in
  let link = anchor "a" in
  try
    set_href link url;
    set_download link filename;
    append link;
    click link;
    remove link;
    later (fun () -> revoke url) revoke_after
  with error ->
    (try remove link with _ -> ());
    revoke url;
    Fault.raise_original error
