use ocamljs_bindgen::{fetch, ocamljs_bindgen};

/// Euclidean geometry with native OCaml records.
#[ocamljs_bindgen]
pub mod geometry {
    #[ocamljsify::ocamljsify]
    /// A point in two-dimensional space.
    pub struct Point {
        /// Horizontal coordinate.
        pub x: f64,
        /// Vertical coordinate.
        pub y: f64,
    }

    #[ocamljs_bindgen::ocamljs_bindgen]
    /// Returns the Euclidean distance between two points.
    pub fn distance(a: Point, b: Point) -> f64 {
        (a.x - b.x).hypot(a.y - b.y)
    }
}

#[ocamljs_bindgen(opaque_t = Token)]
pub mod token {
    /// A parsed numeric token whose identity remains in Rust.
    pub struct Token {
        pub(crate) value: u32,
    }

    #[ocamljs_bindgen::ocamljs_bindgen]
    /// Reads the numeric value without transferring ownership.
    pub fn value(token: &Token) -> u32 {
        token.value
    }
}

#[ocamljs_bindgen]
pub mod tokens {
    #[ocamljs_bindgen::ocamljs_bindgen]
    /// Parses a decimal token; invalid input returns an error message.
    pub fn parse(text: String) -> Result<super::token::Token, String> {
        text.parse()
            .map(|value| super::token::Token { value })
            .map_err(|_| "invalid token".into())
    }
}

#[derive(serde::Deserialize)]
#[ocamljsify::ocamljsify]
/// One cat picture returned by the public API.
pub struct Cat {
    /// Identifier assigned by the API.
    pub id: String,
    /// HTTPS image URL.
    pub url: String,
}

#[ocamljsify::ocamljsify]
/// Expected failures when loading a cat picture.
pub enum CatError {
    /// The request failed.
    Network,
    /// The response exceeded the configured byte limit.
    TooLarge,
    /// The response did not contain a valid HTTPS picture.
    InvalidResponse,
}

#[ocamljs_bindgen]
/// Fetches a picture description from the supplied JSON API URL.
pub async fn fetch_cat(api_url: &str) -> Result<Cat, CatError> {
    let body = fetch::get_text_limited(api_url, 64 * 1024)
        .await
        .map_err(|error| match error {
            fetch::FetchError::Network => CatError::Network,
            fetch::FetchError::TooLarge => CatError::TooLarge,
        })?;
    let cats: Vec<Cat> = serde_json::from_str(&body).map_err(|_| CatError::InvalidResponse)?;
    let cat = cats.into_iter().next().ok_or(CatError::InvalidResponse)?;
    if !cat.url.starts_with("https://") {
        return Err(CatError::InvalidResponse);
    }
    Ok(cat)
}
