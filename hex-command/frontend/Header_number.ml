type formatter
type options = { notation : string; maximumFractionDigits : int; signDisplay : string }

external create : string -> options -> formatter = "NumberFormat" [@@mel.scope "Intl"] [@@mel.new]
external format : formatter -> float -> string = "format" [@@mel.send]

let compact =
  create {js|en|js}
    { notation = {js|compact|js}; maximumFractionDigits = 1; signDisplay = {js|auto|js} }

let signed =
  create {js|en|js}
    { notation = {js|compact|js}; maximumFractionDigits = 1; signDisplay = {js|always|js} }

let full =
  create {js|en|js}
    { notation = {js|standard|js}; maximumFractionDigits = 3; signDisplay = {js|auto|js} }

let display formatter value = if Float.is_finite value then format formatter value else {js|—|js}
let compact value = display compact value
let signed value = display signed value
let full value = display full value
