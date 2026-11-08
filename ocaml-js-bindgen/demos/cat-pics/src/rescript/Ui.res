let by_id = id =>
  switch Browser.find_id(Browser.document_of(Browser.global), id) {
  | Some(element) => element
  | None => failwith("Missing element " ++ id)
  }
let create = tag => Browser.create_element(Browser.document_of(Browser.global), tag)
let text = Browser.set_text
let append = Browser.append_child
let value = Browser.get_value
let on_click = (element, callback) => Browser.listen_raw(element, "click", _ => callback())
let set_attribute = Browser.set_attribute
let button = (label, action) => {
  let element = create("button")
  text(element, label)
  on_click(element, action)
  element
}
let ready = () =>
  set_attribute(Browser.document_element(Browser.document_of(Browser.global)), "data-ready", "true")
let failed = reason => {
  let document = Browser.document_of(Browser.global)
  let error = create("pre")
  text(error, reason)
  append(Browser.body(document), error)
  append(Browser.body(document), button("Reload", Browser.reload))
}
