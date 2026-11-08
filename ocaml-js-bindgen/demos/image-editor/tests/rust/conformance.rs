use std::cell::Cell;
use ocamljs_bindgen::host::Promise;

thread_local! { static DROPS: Cell<u32> = const { Cell::new(0) }; }

impl Drop for Image {
    fn drop(&mut self) {
        DROPS.set(DROPS.get() + 1);
    }
}

#[ocamljs_bindgen(host)]
pub struct Bitmap;

#[ocamljs_bindgen]
unsafe extern "C" {
    #[ocamljs_bindgen(js_name = "createImageBitmap")]
    fn bitmap(data: &ImageData) -> Result<Promise<Bitmap>, HostError>;
    #[ocamljs_bindgen(method, js_name = "drawImage")]
    fn draw_bitmap(context: &Context, bitmap: &Bitmap, x: f64, y: f64) -> Result<(), HostError>;
    #[ocamljs_bindgen(getter, js_name = "width")]
    fn bitmap_width(bitmap: &Bitmap) -> Result<u32, HostError>;
    #[ocamljs_bindgen(getter, js_name = "height")]
    fn bitmap_height(bitmap: &Bitmap) -> Result<u32, HostError>;
    #[ocamljs_bindgen(method, js_name = "close")]
    fn close_bitmap(bitmap: &Bitmap) -> Result<(), HostError>;
}
#[ocamljsify::ocamljsify]
pub struct Snapshot {
    pub width: u32,
    pub height: u32,
}

#[ocamljs_bindgen(opaque)]
pub struct Texture {
    bitmap: Bitmap,
    checksum: u32,
}
impl Drop for Texture {
    fn drop(&mut self) {
        let _ = close_bitmap(&self.bitmap);
    }
}

#[ocamljs_bindgen(module=Texture,name="draw")]
pub fn draw_texture(texture: &Texture, canvas: &Canvas) -> Result<(), HostError> {
    width(canvas, bitmap_width(&texture.bitmap)?)?;
    height(canvas, bitmap_height(&texture.bitmap)?)?;
    draw_bitmap(&context(canvas, "2d")?, &texture.bitmap, 0.0, 0.0)
}
#[ocamljs_bindgen(module=Texture,name="bitmap")]
pub fn texture_bitmap(texture: &Texture) -> Bitmap {
    texture.bitmap.clone()
}


#[ocamljs_bindgen(module = Image)]
pub fn fixture() -> Result<Image, ImageError> {
    open_png(include_bytes!("../fixture.png"))
}
#[ocamljs_bindgen(module = Image)]
pub fn snapshot(image: &Image) -> Snapshot {
    Snapshot {
        width: image.width,
        height: image.height,
    }
}
#[ocamljs_bindgen(module = Image)]
pub fn checksum(image: &Image) -> u32 {
    image
        .pixels
        .iter()
        .fold(0u32, |a, b| a.wrapping_mul(31).wrapping_add(*b as u32))
}
#[ocamljs_bindgen(module = Image)]
pub fn crop(image: &Image, x: u32, y: u32, width: u32, height: u32) -> Result<Image, ImageError> {
    if width == 0
        || height == 0
        || x.checked_add(width).is_none_or(|n| n > image.width)
        || y.checked_add(height).is_none_or(|n| n > image.height)
    {
        return Err(ImageError::InvalidCrop);
    }
    let mut pixels = Vec::with_capacity((width * height * 4) as usize);
    for row in y..y + height {
        let start = ((row * image.width + x) * 4) as usize;
        pixels.extend_from_slice(&image.pixels[start..start + (width * 4) as usize]);
    }
    Ok(Image {
        width,
        height,
        pixels,
        frame: None,
    })
}
#[ocamljs_bindgen(module = Image)]
pub fn blend(destination: &mut Image, source: &Image) -> Result<(), ImageError> {
    if destination.width != source.width || destination.height != source.height {
        return Err(ImageError::DimensionsDiffer);
    }
    destination.frame = None;
    for (a, b) in destination.pixels.iter_mut().zip(&source.pixels) {
        *a = ((*a as u16 + *b as u16) / 2) as u8;
    }
    Ok(())
}
#[ocamljs_bindgen(consume(image), module = Texture, name = "create")]
pub async fn upload(mut image: Image) -> Result<Texture, ImageError> {
    let checksum = checksum(&image);
    let bitmap = bitmap(image.frame()?)
        .map_err(ImageError::Browser)?
        .await
        .map_err(ImageError::Browser)?;
    Ok(Texture { bitmap, checksum })
}
#[ocamljs_bindgen(consume(image), module = Image)]
pub fn invalid_upload(image: Image, other: &Image) -> u32 {
    checksum(&image) ^ checksum(other)
}
#[ocamljs_bindgen(module = Texture, name = "checksum")]
pub fn texture_checksum(texture: &Texture) -> u32 {
    texture.checksum
}
#[ocamljs_bindgen]
pub fn drops() -> u32 {
    DROPS.get()
}

#[ocamljs_bindgen(consume(image), module = Texture)]
pub fn reject_upload(image: Image) -> Result<Texture, ImageError> {
    drop(image);
    Err(ImageError::TooLarge)
}
