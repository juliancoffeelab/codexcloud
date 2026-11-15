type entry

let entries : string -> entry array =
  [%mel.raw
    {|kind => typeof performance.getEntriesByType === "function" ? performance.getEntriesByType(kind) : []|}]

external start_time : entry -> float = "startTime" [@@mel.get]
external json : entry -> Js.Json.t = "toJSON" [@@mel.send]

let obj fields = Js.Json.object_ (Js.Dict.fromArray (Array.of_list fields))
let text value = Js.Json.string value

let field value name =
  Option.bind (Js.Json.decodeObject value) (fun value -> Js.Dict.get value name)

let number value name = Option.bind (field value name) Js.Json.decodeNumber
let string value name = Option.bind (field value name) Js.Json.decodeString
let nullable_number = Option.fold ~none:Js.Json.null ~some:Js.Json.number

let interval value first last =
  match (number value first, number value last) with
  | Some first, Some last when first >= 0. && last >= first -> Some (last -. first)
  | _ -> None

let navigation_phases navigation recorder_started_at_ms =
  let tls =
    match number navigation {js|secureConnectionStart|js} with
    | Some start when start > 0. ->
        interval navigation {js|secureConnectionStart|js} {js|connectEnd|js}
    | _ -> None
  in
  let before_recorder =
    match number navigation {js|responseEnd|js} with
    | Some finish when finish > 0. && recorder_started_at_ms >= finish ->
        Some (recorder_started_at_ms -. finish)
    | _ -> None
  in
  obj
    [
      ( {js|dnsMs|js},
        nullable_number (interval navigation {js|domainLookupStart|js} {js|domainLookupEnd|js}) );
      ( {js|connectionMs|js},
        nullable_number (interval navigation {js|connectStart|js} {js|connectEnd|js}) );
      ({js|tlsWithinConnectionMs|js}, nullable_number tls);
      ( {js|requestToFirstByteMs|js},
        nullable_number (interval navigation {js|requestStart|js} {js|responseStart|js}) );
      ( {js|responseBodyMs|js},
        nullable_number (interval navigation {js|responseStart|js} {js|responseEnd|js}) );
      ({js|responseEndToRecorderMs|js}, nullable_number before_recorder);
    ]

let resource_evidence resources =
  let zero_transfer_with_body = ref 0
  and cache_label_with_transfer = ref 0
  and other_transfer = ref 0
  and unknown = ref 0 in
  Array.iter
    (fun resource ->
      match number resource {js|transferSize|js} with
      | Some transfer when transfer > 0. ->
          if string resource {js|deliveryType|js} = Some {js|cache|js} then
            incr cache_label_with_transfer
          else incr other_transfer
      | Some 0.
        when Option.fold ~none:false
               ~some:(fun size -> size > 0.)
               (number resource {js|encodedBodySize|js}) ->
          incr zero_transfer_with_body
      | _ -> incr unknown)
    resources;
  let count value = Js.Json.number (float_of_int !value) in
  obj
    [
      ({js|cacheLabelWithReportedTransfer|js}, count cache_label_with_transfer);
      ({js|zeroReportedTransferWithBody|js}, count zero_transfer_with_body);
      ({js|otherReportedTransfer|js}, count other_transfer);
      ({js|insufficientData|js}, count unknown);
    ]

let snapshot ~recorder_started_at_ms ~until_ms =
  let collected_at = Browser_time.now () in
  let navigation = entries {js|navigation|js} in
  let navigation = if Array.length navigation = 0 then Js.Json.null else json navigation.(0) in
  let resources =
    entries {js|resource|js}
    |> Array_ext.filter (fun entry -> start_time entry <= until_ms)
    |> Array.map json
  in
  let result =
    obj
      [
        ({js|collectedAtMs|js}, Js.Json.number collected_at);
        ({js|startupCutoffAtMs|js}, Js.Json.number until_ms);
        ({js|navigation|js}, navigation);
        ({js|navigationPhases|js}, navigation_phases navigation recorder_started_at_ms);
        ({js|resources|js}, Js.Json.array resources);
        ({js|resourceEvidence|js}, resource_evidence resources);
        ( {js|headerCoverage|js},
          obj
            [
              ({js|mode|js}, text {js|passive-existing-responses|js});
              ({js|extraRequests|js}, Js.Json.number 0.);
              ( {js|capturedFrom|js},
                Js.Json.array [| text {js|hud-fetch|js}; text {js|phaser-xhr|js} |] );
              ( {js|notExposedFor|js},
                Js.Json.array
                  [|
                    text {js|document|js};
                    text {js|image-element|js};
                    text {js|module-script|js};
                    text {js|wasm-runtime|js};
                  |] );
            ] );
        ( {js|interpretationLimits|js},
          Js.Json.array
            [|
              text
                {js|deliveryType=cache with positive transferSize does not establish an instant local-cache hit.|js};
              text
                {js|Zero sizes may mean unavailable timing data; they are not automatically cache hits.|js};
              text
                {js|Response headers are browser-exposed values, not a raw network or validation transcript.|js};
              text
                {js|Request-to-first-byte time includes the delivery path and does not identify Cloudflare or origin processing separately.|js};
              text
                {js|TLS time is included in connection time; intervals must not all be added together.|js};
            |] );
      ]
  in
  Option.iter
    (fun fields ->
      Js.Dict.set fields {js|collectionMs|js} (Js.Json.number (Browser_time.now () -. collected_at)))
    (Js.Json.decodeObject result);
  result
