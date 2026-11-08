val v1_native : Historical_types.Native1.t -> Historical_types.V2.t
(** Pure sequential conversions over the complete frozen historical contracts. *)

val v1_javascript : Historical_types.Javascript1.t -> Historical_types.V2.t
val v2 : Historical_types.V2.t -> Historical_types.V3.t
val v3 : Historical_types.V3.t -> Historical_types.V4.t
val v4 : Historical_types.V4.t -> Historical_types.V5.t
val v5 : Historical_types.V5.t -> Historical_types.V6.t
val v6 : Historical_types.V6.t -> Historical_types.V7.t
val v7 : Historical_types.V7.t -> Historical_types.V8.t
val v8 : Historical_types.V8.t -> Historical_types.V9.t
val v9 : Historical_types.V9.t -> Historical_types.V10.t
val v10 : Historical_types.V10.t -> Historical_types.V11.t
val v11 : Historical_types.V11.t -> Historical_types.V12.t
val v12 : Historical_types.V12.t -> Historical_types.V13.t
val v13 : Historical_types.V13.t -> Historical_types.V14.t
val v14 : Historical_types.V14.t -> Historical_types.V15.t
val v15 : Historical_types.V15.t -> Historical_types.V16.t
val v16 : Historical_types.V16.t -> Historical_types.V17.t
val v17 : Historical_types.V17.t -> Historical_types.V18.t

val v6_with_preplaced_sella : bool -> Historical_types.V6.t -> Historical_types.V7.t
(** The boolean records whether the source was the original browser snapshot, whose Sella was
    preplaced. Native absence alone does not establish a death. *)
