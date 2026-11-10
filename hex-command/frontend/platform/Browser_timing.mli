val snapshot : recorder_started_at_ms:float -> until_ms:float -> Js.Json.t
(** Read the browser's existing navigation and resource timing entries on demand. Resources are
    limited to requests started by the first battlefield frame (or current stalled startup). Native
    transfer/body sizes and delivery type are retained as evidence, without guessing cache hits or
    mistaking CPU texture admission for GPU completion. *)
