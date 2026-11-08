external storage_length : int = "length" [@@mel.scope "localStorage"]
external storage_key : int -> string Js.Nullable.t = "key" [@@mel.scope "localStorage"]
external storage_get : string -> string Js.Nullable.t = "getItem" [@@mel.scope "localStorage"]
external storage_set : string -> string -> unit = "setItem" [@@mel.scope "localStorage"]
external storage_remove : string -> unit = "removeItem" [@@mel.scope "localStorage"]

external request_lock : string -> (unit -> 'a Js.Promise.t) -> 'a Js.Promise.t = "request"
[@@mel.scope "navigator.locks"]
