val v1_to_v2 : Historical_types.History1.t -> Historical_types.History2.t * string array
val v2_to_v3 : Historical_types.History2.t -> Historical_types.History3.t
val v3_to_v4 : Historical_types.History3.t -> Historical_types.History4.t
val migrate_typed : Js.Json.t -> Historical_types.History4.t * bool * string array
val migrate : Js.Json.t -> Js.Json.t
