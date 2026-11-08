let by_id id =
  match Browser.find_id (Browser.document_of Browser.global) (Browser.to_host id) with
  | Some element -> element
  | None -> failwith ("Missing element " ^ id)

let create tag =
  Browser.create_element (Browser.document_of Browser.global) (Browser.to_host tag)

let text element value = Browser.set_text element (Browser.to_host value)
let clear = Browser.replace_children
let append = Browser.append_child

let on_click element callback =
  Browser.listen_raw element (Browser.to_host "click") (fun _ -> callback ())

let class_name element value = Browser.set_class element (Browser.to_host value)

let set_attribute element name value =
  Browser.set_attribute element (Browser.to_host name) (Browser.to_host value)

let button label action =
  let element = create "button" in
  text element label;
  on_click element action;
  element

let ready () =
  set_attribute
    (Browser.document_element (Browser.document_of Browser.global))
    "data-ready" "true"

let failed reason =
  let document = Browser.document_of Browser.global in
  let error = create "pre" in
  text error reason;
  append (Browser.body document) error;
  append (Browser.body document) (button "Reload" Browser.reload)
