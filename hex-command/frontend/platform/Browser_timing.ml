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
    match number navigation "secureConnectionStart" with
    | Some start when start > 0. -> interval navigation "secureConnectionStart" "connectEnd"
    | _ -> None
  in
  let before_recorder =
    match number navigation "responseEnd" with
    | Some finish when finish > 0. && recorder_started_at_ms >= finish ->
        Some (recorder_started_at_ms -. finish)
    | _ -> None
  in
  obj
    [
      ("dnsMs", nullable_number (interval navigation "domainLookupStart" "domainLookupEnd"));
      ("connectionMs", nullable_number (interval navigation "connectStart" "connectEnd"));
      ("tlsWithinConnectionMs", nullable_number tls);
      ("requestToFirstByteMs", nullable_number (interval navigation "requestStart" "responseStart"));
      ("responseBodyMs", nullable_number (interval navigation "responseStart" "responseEnd"));
      ("responseEndToRecorderMs", nullable_number before_recorder);
    ]

let resource_evidence resources =
  let zero_transfer_with_body = ref 0
  and cache_label_with_transfer = ref 0
  and other_transfer = ref 0
  and unknown = ref 0 in
  Array.iter
    (fun resource ->
      match number resource "transferSize" with
      | Some transfer when transfer > 0. ->
          if string resource "deliveryType" = Some "cache" then incr cache_label_with_transfer
          else incr other_transfer
      | Some 0.
        when Option.fold ~none:false
               ~some:(fun size -> size > 0.)
               (number resource "encodedBodySize") ->
          incr zero_transfer_with_body
      | _ -> incr unknown)
    resources;
  let count value = Js.Json.number (float_of_int !value) in
  obj
    [
      ("cacheLabelWithReportedTransfer", count cache_label_with_transfer);
      ("zeroReportedTransferWithBody", count zero_transfer_with_body);
      ("otherReportedTransfer", count other_transfer);
      ("insufficientData", count unknown);
    ]

let snapshot ~recorder_started_at_ms ~until_ms =
  let collected_at = Browser_time.now () in
  let navigation = entries "navigation" in
  let navigation = if Array.length navigation = 0 then Js.Json.null else json navigation.(0) in
  let resources =
    entries "resource"
    |> Array_ext.filter (fun entry -> start_time entry <= until_ms)
    |> Array.map json
  in
  let result =
    obj
      [
        ("collectedAtMs", Js.Json.number collected_at);
        ("startupCutoffAtMs", Js.Json.number until_ms);
        ("navigation", navigation);
        ("navigationPhases", navigation_phases navigation recorder_started_at_ms);
        ("resources", Js.Json.array resources);
        ("resourceEvidence", resource_evidence resources);
        ( "headerCoverage",
          obj
            [
              ("mode", text "passive-existing-responses");
              ("extraRequests", Js.Json.number 0.);
              ("capturedFrom", Js.Json.array [| text "hud-fetch"; text "phaser-xhr" |]);
              ( "notExposedFor",
                Js.Json.array
                  [|
                    text "document"; text "image-element"; text "module-script"; text "wasm-runtime";
                  |] );
            ] );
        ( "interpretationLimits",
          Js.Json.array
            [|
              text
                "deliveryType=cache with positive transferSize does not establish an instant \
                 local-cache hit.";
              text
                "Zero sizes may mean unavailable timing data; they are not automatically cache \
                 hits.";
              text
                "Response headers are browser-exposed values, not a raw network or validation \
                 transcript.";
              text
                "Request-to-first-byte time includes the delivery path and does not identify \
                 Cloudflare or origin processing separately.";
              text
                "TLS time is included in connection time; intervals must not all be added together.";
            |] );
      ]
  in
  Option.iter
    (fun fields ->
      Js.Dict.set fields "collectionMs" (Js.Json.number (Browser_time.now () -. collected_at)))
    (Js.Json.decodeObject result);
  result
