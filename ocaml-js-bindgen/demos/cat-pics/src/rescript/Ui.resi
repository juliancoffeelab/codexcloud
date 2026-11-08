/** Find a required element by its HTML id. Raises Failure if it is absent. */
let by_id: string => Browser.element

/** Replace the element text content. */
let text: (Browser.element, string) => unit

/** Read the current form control value. */
let value: Browser.element => string

/** Register a click callback. */
let on_click: (Browser.element, unit => unit) => unit

/** Set an HTML attribute: element, name, value. */
let set_attribute: (Browser.element, string, string) => unit

/** Mark the document ready for interaction. */
let ready: unit => unit

/** Append the failure message and a Reload button to the page. */
let failed: string => unit
