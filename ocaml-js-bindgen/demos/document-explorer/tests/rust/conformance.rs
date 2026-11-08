#[ocamljs_bindgen(host)]
pub struct Parts;
#[ocamljs_bindgen]
unsafe extern "C" {
    #[ocamljs_bindgen(js_namespace = "Array", js_name = "of")]
    fn parts(text: &str) -> Result<Parts, HostError>;
    #[ocamljs_bindgen(constructor, js_name = "Blob")]
    fn new_blob(parts: &Parts) -> Result<Blob, HostError>;
}
#[ocamljs_bindgen(module=Blob,name="of_text")]
pub fn blob(text: &str) -> Result<Blob, HostError> {
    new_blob(&parts(text)?)
}
