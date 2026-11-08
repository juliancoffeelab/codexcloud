use ocamljs_bindgen::{fetch, ocamljs_bindgen};

#[derive(serde::Deserialize)]
#[ocamljsify::ocamljsify]
pub struct Cat {
    pub id: String,
    pub url: String,
}

#[derive(Debug)]
#[ocamljsify::ocamljsify]
pub enum CatError {
    Network,
    TooLarge,
    InvalidResponse,
}

// The browser fetch is ordinary asynchronous I/O in the current realm.
#[ocamljs_bindgen]
pub async fn fetch_cat(api_url: &str) -> Result<Cat, CatError> {
    let body = fetch::get_text_limited(api_url, 64 * 1024)
        .await
        .map_err(|error| match error {
            fetch::FetchError::Network => CatError::Network,
            fetch::FetchError::TooLarge => CatError::TooLarge,
        })?;
    parse_cat(&body)
}

fn parse_cat(body: &str) -> Result<Cat, CatError> {
    let cats: Vec<Cat> = serde_json::from_str(body).map_err(|_| CatError::InvalidResponse)?;
    let cat = cats.into_iter().next().ok_or(CatError::InvalidResponse)?;
    if !cat.url.starts_with("https://") {
        return Err(CatError::InvalidResponse);
    }
    Ok(cat)
}

#[cfg(test)]
mod tests {
    include!("../../tests/rust/unit.rs");
}

#[cfg(feature = "conformance")]
include!("../../tests/rust/conformance.rs");
