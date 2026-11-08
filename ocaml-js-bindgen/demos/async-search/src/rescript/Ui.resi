/** Find a required element by its HTML id. Raises Failure if it is absent. */
let by_id: string => Browser.element

/** Create an unattached element with the given HTML tag. */
let create: string => Browser.element

/** Replace the element text content. */
let text: (Browser.element, string) => unit

/** Remove all child nodes. */
let clear: Browser.element => unit

/** Append the second element to the first. */
let append: (Browser.element, Browser.element) => unit

/** Read the current form control value. */
let value: Browser.element => string

/** Register a click callback. */
let on_click: (Browser.element, unit => unit) => unit

/** Mark the document ready for interaction. */
let ready: unit => unit

/** Append the failure message and a Reload button to the page. */
let failed: string => unit
