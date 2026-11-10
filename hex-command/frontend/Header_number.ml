type formatter
type options = { notation : string; maximumFractionDigits : int; signDisplay : string }

external create : string -> options -> formatter = "NumberFormat" [@@mel.scope "Intl"] [@@mel.new]
external format : formatter -> float -> string = "format" [@@mel.send]

let compact = create "en" { notation = "compact"; maximumFractionDigits = 1; signDisplay = "auto" }
let signed = create "en" { notation = "compact"; maximumFractionDigits = 1; signDisplay = "always" }
let full = create "en" { notation = "standard"; maximumFractionDigits = 3; signDisplay = "auto" }

let display formatter value =
  if Float.is_finite value then Utf8.of_js (format formatter value) else "—"

let compact value = display compact value
let signed value = display signed value
let full value = display full value
