use ocamljs_bindgen::ocamljs_bindgen;

/// The hypotenuse of a right triangle with sides a and b.
#[ocamljs_bindgen]
pub fn hypotenuse(a: f64, b: f64) -> f64 {
    a.hypot(b)
}
