// Minimal feature builds exercise reference support without any declared host type.
#[cfg(not(any(
    feature = "primitive-only",
    feature = "builtin-only",
    feature = "error-only",
    feature = "text-only",
    feature = "pixels-only"
)))]
mod demo;
#[cfg(feature = "primitive-only")]
mod primitives {
    use ocamljs_bindgen::{HostError, ocamljs_bindgen};
    #[ocamljs_bindgen]
    unsafe extern "C" {
        #[ocamljs_bindgen(js_namespace = "Math", js_name = "abs")]
        fn absolute(value: f64) -> Result<f64, HostError>;
        // A disabled import must not introduce an ES dependency or ABI metadata.
        #[cfg(any())]
        #[ocamljs_bindgen(js_module = "./must-not-load.mjs", js_name = "disabled")]
        fn disabled(value: f64) -> Result<f64, HostError>;
    }
    #[ocamljs_bindgen]
    pub fn abs(value: f64) -> Result<f64, HostError> {
        absolute(value)
    }
}
#[cfg(all(
    any(feature = "builtin-only", feature = "error-only"),
    not(feature = "primitive-only")
))]
mod builtins {
    use ocamljs_bindgen::{HostError, ocamljs_bindgen};
    #[cfg(feature = "builtin-only")]
    #[ocamljs_bindgen]
    pub type Pending = ocamljs_bindgen::host::Promise<HostError>;
    #[ocamljs_bindgen]
    pub fn identity(error: HostError) -> HostError {
        error
    }
    #[cfg(feature = "builtin-only")]
    #[ocamljs_bindgen]
    pub fn pending(value: Pending) -> Pending {
        value
    }
}

#[cfg(feature = "text-only")]
mod text {
    use ocamljs_bindgen::{host::JsString, ocamljs_bindgen};
    #[ocamljs_bindgen]
    pub type Text = JsString;
    #[ocamljs_bindgen]
    pub fn identity(value: Text) -> Text {
        value
    }
}
#[cfg(feature = "pixels-only")]
mod pixels {
    use ocamljs_bindgen::{host::Uint8ClampedArray, ocamljs_bindgen};
    #[ocamljs_bindgen]
    pub fn identity(value: Uint8ClampedArray) -> Uint8ClampedArray {
        value
    }
}
