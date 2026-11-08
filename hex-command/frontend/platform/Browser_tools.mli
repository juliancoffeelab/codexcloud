type registry
type document

type tool =
  < name : string
  ; description : string
  ; inputSchema : Js.Json.t
  ; annotations : Js.Json.t Js.undefined
  ; execute : Js.Json.t Js.undefined -> Js.Json.t Js.Promise.t >
  Js.t

external document : document = "document"
external model_context : document -> registry Js.Nullable.t = "modelContext" [@@mel.get]
external registration : registry -> Js.Json.t = "registerTool" [@@mel.get]
external unregistration : registry -> Js.Json.t = "unregisterTool" [@@mel.get]
external register_tool : registry -> tool -> Js.Json.t = "registerTool" [@@mel.send]
external unregister_tool : registry -> string -> Js.Json.t = "unregisterTool" [@@mel.send]
external tool_name : tool -> string = "name" [@@mel.get]
