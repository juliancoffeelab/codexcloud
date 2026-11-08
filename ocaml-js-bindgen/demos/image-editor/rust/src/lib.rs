use ocamljs_bindgen::{
    HostError,
    host::{CopyError, Uint8ClampedArray},
    ocamljs_bindgen,
};
use std::io::Cursor;
mod paint;
const MAX_DIMENSION: u32 = 2048;
#[ocamljsify::ocamljsify]
#[derive(Debug)]
pub enum ImageError {
    InvalidPng,
    TooLarge,
    InvalidCrop,
    DimensionsDiffer,
    Browser(HostError),
}
#[ocamljs_bindgen(opaque)]
pub struct Image {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
    frame: Option<ImageData>,
}
fn copy_error(error: CopyError) -> ImageError {
    match error {
        CopyError::TooLarge => ImageError::TooLarge,
        CopyError::Host(error) => ImageError::Browser(error),
    }
}
#[ocamljs_bindgen(host)]
pub struct Canvas;
#[ocamljs_bindgen(host)]
pub struct Context;
#[ocamljs_bindgen(host)]
pub struct ImageData;
#[ocamljs_bindgen]
unsafe extern "C" {
    #[ocamljs_bindgen(js_namespace = "document", js_name = "getElementById")]
    fn get_canvas(id: &str) -> Result<Canvas, HostError>;
    #[ocamljs_bindgen(method, js_name = "getContext")]
    fn context(canvas: &Canvas, kind: &str) -> Result<Context, HostError>;
    #[ocamljs_bindgen(setter, js_name = "width")]
    fn width(canvas: &Canvas, value: u32) -> Result<(), HostError>;
    #[ocamljs_bindgen(setter, js_name = "height")]
    fn height(canvas: &Canvas, value: u32) -> Result<(), HostError>;
    #[ocamljs_bindgen(getter, js_name = "width")]
    fn canvas_width(canvas: &Canvas) -> Result<u32, HostError>;
    #[ocamljs_bindgen(getter, js_name = "height")]
    fn canvas_height(canvas: &Canvas) -> Result<u32, HostError>;
    #[ocamljs_bindgen(constructor, js_name = "ImageData")]
    fn image_data(
        pixels: &Uint8ClampedArray,
        width: u32,
        height: u32,
    ) -> Result<ImageData, HostError>;
    #[ocamljs_bindgen(method, js_name = "putImageData")]
    fn put(context: &Context, data: &ImageData, x: f64, y: f64) -> Result<(), HostError>;

}
#[ocamljs_bindgen(module=Canvas,name="by_id")]
pub fn canvas(id: &str) -> Result<Canvas, HostError> {
    get_canvas(id)
}
impl Image {
    fn frame(&mut self) -> Result<&ImageData, ImageError> {
        if self.frame.is_none() {
            let pixels = Uint8ClampedArray::copy_from(&self.pixels).map_err(copy_error)?;
            self.frame =
                Some(image_data(&pixels, self.width, self.height).map_err(ImageError::Browser)?);
        }
        Ok(self.frame.as_ref().unwrap())
    }
}
#[ocamljs_bindgen(module=Image)]
pub fn draw(image: &mut Image, canvas: &Canvas) -> Result<(), ImageError> {
    if canvas_width(canvas).map_err(ImageError::Browser)? != image.width {
        width(canvas, image.width).map_err(ImageError::Browser)?;
    }
    if canvas_height(canvas).map_err(ImageError::Browser)? != image.height {
        height(canvas, image.height).map_err(ImageError::Browser)?;
    }
    let context = context(canvas, "2d").map_err(ImageError::Browser)?;
    put(&context, image.frame()?, 0.0, 0.0).map_err(ImageError::Browser)
}
#[ocamljs_bindgen(module = Image, name = "create")]
pub fn open_png(data: &[u8]) -> Result<Image, ImageError> {
    let mut decoder = png::Decoder::new(Cursor::new(data));
    decoder.set_limits(png::Limits {
        bytes: 32 * 1024 * 1024,
    });
    decoder.set_transformations(png::Transformations::normalize_to_color8());
    let mut reader = decoder.read_info().map_err(|_| ImageError::InvalidPng)?;
    let info = reader.info();
    if info.width > MAX_DIMENSION || info.height > MAX_DIMENSION {
        return Err(ImageError::TooLarge);
    }
    let size = reader.output_buffer_size().ok_or(ImageError::TooLarge)?;
    if size > 16 * 1024 * 1024 {
        return Err(ImageError::TooLarge);
    }
    let mut pixels = vec![0; size];
    let frame = reader
        .next_frame(&mut pixels)
        .map_err(|_| ImageError::InvalidPng)?;
    pixels.truncate(frame.buffer_size());
    let rgba = if frame.color_type == png::ColorType::Rgba {
        pixels
    } else {
        let mut rgba = Vec::with_capacity((frame.width * frame.height * 4) as usize);
        for pixel in pixels.chunks_exact(frame.color_type.samples()) {
            let (r, g, b, a) = match frame.color_type {
                png::ColorType::Rgb => (pixel[0], pixel[1], pixel[2], 255),
                png::ColorType::Grayscale => (pixel[0], pixel[0], pixel[0], 255),
                png::ColorType::GrayscaleAlpha => (pixel[0], pixel[0], pixel[0], pixel[1]),
                _ => return Err(ImageError::InvalidPng),
            };
            rgba.extend_from_slice(&[r, g, b, a]);
        }
        rgba
    };
    Ok(Image {
        width: frame.width,
        height: frame.height,
        pixels: rgba,
        frame: None,
    })
}
#[cfg(test)]
mod tests {
    include!("../../tests/rust/png.rs");
}

#[cfg(feature = "conformance")]
include!("../../tests/rust/conformance.rs");
