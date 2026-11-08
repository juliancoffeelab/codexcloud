#[ocamljs_bindgen(throws)]
pub async fn fetch_cat_throw(api_url: &str) -> Result<Cat, CatError> {
    fetch_cat(api_url).await
}
