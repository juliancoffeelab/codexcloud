/** Find a required element by its HTML id. Raises Failure if it is absent. */
let by_id: string => Browser.element

/** Replace the element text content. */
let text: (Browser.element, string) => unit

/** Remove all child nodes. */
let clear: Browser.element => unit

/** Append the second element to the first. */
let append: (Browser.element, Browser.element) => unit

/** Register a click callback. */
let on_click: (Browser.element, unit => unit) => unit

/** Replace the element CSS class list. */
let class_name: (Browser.element, string) => unit

/** Set an HTML attribute: element, name, value. */
let set_attribute: (Browser.element, string, string) => unit

/** Create an unattached button with a label and click callback. */
let button: (string, unit => unit) => Browser.element

/** Mark the document ready for interaction. */
let ready: unit => unit

/** Append the failure message and a Reload button to the page. */
let failed: string => unit
