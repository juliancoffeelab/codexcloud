val v29 : Historical_types.V29.t -> Historical_types.V30.t
(** Frozen typed save transitions. Unknown wire data enters through Historical_codec. *)

val v30 : Historical_types.V30.t -> Historical_types.V31.t
val v31 : Historical_types.V31.t -> Historical_types.V32.t
val v32 : Historical_types.V32.t -> Historical_types.V33.t
val v33 : Historical_types.V33.t -> Historical_types.V34.t
val v34 : Historical_types.V34.t -> Historical_types.V35.t
val v35 : Historical_types.V35.t -> Historical_types.V36.t
val v36 : Historical_types.V36.t -> Historical_types.V37.t
val v37 : Historical_types.V37.t -> Historical_types.V38.t
val v38 : Historical_types.V38.t -> Historical_types.V39.t

val convert_state_and_undo :
  encode_old:('a -> Js.Json.t) ->
  decode_old:(Js.Json.t -> 'a) ->
  encode_new:('b -> Js.Json.t) ->
  ('a -> 'b) ->
  'a ->
  Historical_types.V39.current_undo array option ->
  'b * Historical_types.V39.current_undo array option
(** Reconstruct each historical undo state before applying a typed state conversion. *)

val convert_state_and_undo_with_codecs :
  encode_change:(Historical_types.save34_undo_member1_item_changes_item -> Js.Json.t) ->
  decode_change:(Js.Json.t -> Historical_types.save34_undo_member1_item_changes_item) ->
  encode_old:('a -> Js.Json.t) ->
  decode_old:(Js.Json.t -> 'a) ->
  encode_new:('b -> Js.Json.t) ->
  ('a -> 'b) ->
  'a ->
  Historical_types.V39.current_undo array option ->
  'b * Historical_types.V39.current_undo array option
