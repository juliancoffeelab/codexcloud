type handle = Surface | North_west | North_east | South_west | South_east

/** Find a required element by its HTML id. Raises Failure if it is absent. */
let by_id: string => Browser.element

/** Replace the element text content. */
let text: (Browser.element, string) => unit

/** Read the current form control value. */
let value: Browser.element => string

/** Set the form control value. */
let set_value: (Browser.element, string) => unit

/** Register a click callback. */
let on_click: (Browser.element, unit => unit) => unit

/** Set an HTML attribute: element, name, value. */
let set_attribute: (Browser.element, string, string) => unit

/** Mark the document ready for interaction. */
let ready: unit => unit

/** Append the failure message and a Reload button to the page. */
let failed: string => unit

/** Register a form change callback. */
let on_change: (Browser.element, unit => unit) => unit

/** Register a window resize callback. */
let on_resize: (unit => unit) => unit

/** Schedule one callback for the next animation frame. */
let frame: (unit => unit) => unit

/** Set whether a form control is disabled. */
let disabled: (Browser.element, bool) => unit

/** Set whether an element is hidden. */
let hidden: (Browser.element, bool) => unit

/** Read the last pointer position in canvas pixels, along the x axis. */
let pointer_x: unit => float

/** Read the last pointer position in canvas pixels, along the y axis. */
let pointer_y: unit => float

/** Read the last pointer x coordinate in viewport CSS pixels. */
let pointer_screen_x: unit => float

/** Read the last pointer y coordinate in viewport CSS pixels. */
let pointer_screen_y: unit => float

/** Read the canvas surface or selection corner associated with the current pointer. */
let pointer_handle: unit => handle

/** Read the pinch center x offset from the interaction element center, in CSS pixels. */
let pinch_x: unit => float

/** Read the pinch center y offset from the interaction element center, in CSS pixels. */
let pinch_y: unit => float

/** Read the distance between the two pinch contacts, in CSS pixels. */
let pinch_distance: unit => float

/** Read the last wheel displacement, normalized to CSS pixels. */
let wheel_delta: unit => float

/** Read the current normalized keyboard shortcut. */
let key: unit => string

/** Register callbacks in order: down, move, up, cancel, pinch start, pinch move, pinch end. Coordinate getters are updated before callbacks run. */
let on_pointer: (
  Browser.element,
  unit => unit,
  unit => unit,
  unit => unit,
  unit => unit,
  unit => unit,
  unit => unit,
  unit => unit,
) => unit

/** Set canvas translation x, y in CSS pixels and its zoom factor. */
let view: (float, float, float) => unit

/** Compute a zoom factor that fits the canvas into the workspace. */
let fit_scale: unit => float

/** Display the selection rectangle: x, y, width, height in canvas pixels. */
let selection: (float, float, float, float) => unit

/** Register a wheel callback and suppress native scrolling over this element. */
let on_wheel: (Browser.element, unit => unit) => unit

/** Register a keyboard callback outside editable controls. Return true to prevent the browser default. */
let on_key: (unit => bool) => unit

/** Associate a normalized keyboard shortcut with a button click. */
let shortcut: (Browser.element, string) => unit

/** Download the supplied PNG bytes. */
let save_png: Bindings.Uint8ClampedArray.t => unit
