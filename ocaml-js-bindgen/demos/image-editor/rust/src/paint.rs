//! The document and its pixels stay in Rust. A stroke takes one undo snapshot,
//! not one per pointer event. Both history stacks share a 24 MiB / 40 step cap.
use super::{Canvas, Image, ImageError, MAX_DIMENSION, copy_error, draw, open_png};
use ocamljs_bindgen::{
    HostError,
    host::{Promise, Uint8ClampedArray},
    ocamljs_bindgen,
};
use std::collections::VecDeque;

const HISTORY_BYTES: usize = 24 * 1024 * 1024;
const HISTORY_STEPS: usize = 40;

#[ocamljsify::ocamljsify]
#[derive(Clone, Copy, PartialEq)]
pub enum Tool {
    Brush,
    Eraser,
    Fill,
    Line,
    Rectangle,
    Ellipse,
}

#[ocamljsify::ocamljsify]
pub struct PaintStatus {
    pub width: u32,
    pub height: u32,
    pub can_undo: bool,
    pub can_redo: bool,
}

/// A nonempty pixel rectangle wholly inside the document.
#[ocamljsify::ocamljsify]
#[derive(Clone, Copy)]
pub struct Region {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}
impl Region {
    fn valid(self, image: &Image) -> bool {
        self.width > 0
            && self.height > 0
            && self
                .x
                .checked_add(self.width)
                .is_some_and(|n| n <= image.width)
            && self
                .y
                .checked_add(self.height)
                .is_some_and(|n| n <= image.height)
    }
}
struct Transform {
    before: Pixels,
    source: Region,
}

struct Pixels {
    width: u32,
    height: u32,
    data: Vec<u8>,
}
struct Stroke {
    before: Pixels,
    tool: Tool,
    start: (f64, f64),
    last: (f64, f64),
    color: [u8; 4],
    size: u32,
}

/// A drawing document with one active gesture and bounded undo/redo storage.
#[ocamljs_bindgen(opaque)]
pub struct Paint {
    image: Image,
    past: VecDeque<Pixels>,
    future: Vec<Pixels>,
    active: Option<Stroke>,
    transform: Option<Transform>,
}

impl Paint {
    fn from_image(image: Image) -> Self {
        Self {
            image,
            past: VecDeque::new(),
            future: Vec::new(),
            active: None,
            transform: None,
        }
    }
    fn snapshot(&self) -> Pixels {
        Pixels {
            width: self.image.width,
            height: self.image.height,
            data: self.image.pixels.clone(),
        }
    }
    fn replace(&mut self, pixels: Pixels) -> Pixels {
        let mut old = std::mem::replace(
            &mut self.image,
            Image {
                width: pixels.width,
                height: pixels.height,
                pixels: pixels.data,
                frame: None,
            },
        );
        Pixels {
            width: old.width,
            height: old.height,
            data: std::mem::take(&mut old.pixels),
        }
    }
    fn trim(&mut self) {
        while self.past.len() + self.future.len() > HISTORY_STEPS
            || self
                .past
                .iter()
                .chain(&self.future)
                .map(|s| s.data.len())
                .sum::<usize>()
                > HISTORY_BYTES
        {
            if self.past.pop_front().is_none() {
                self.future.remove(0);
            }
        }
    }
    fn commit(&mut self, before: Pixels) {
        if before.width != self.image.width
            || before.height != self.image.height
            || before.data != self.image.pixels
        {
            self.future.clear();
            self.past.push_back(before);
            self.trim();
        }
    }
    fn point(&self, x: f64, y: f64) -> (f64, f64) {
        (
            x.clamp(0.0, self.image.width.saturating_sub(1) as f64),
            y.clamp(0.0, self.image.height.saturating_sub(1) as f64),
        )
    }
    fn stamp(&mut self, x: f64, y: f64, size: u32, color: [u8; 4]) {
        let radius = size as f64 / 2.0;
        let (cx, cy) = (x.floor() + 0.5, y.floor() + 0.5);
        let left = (cx - radius).floor().max(0.0) as u32;
        let top = (cy - radius).floor().max(0.0) as u32;
        let right = (cx + radius).ceil().min(self.image.width as f64) as u32;
        let bottom = (cy + radius).ceil().min(self.image.height as f64) as u32;
        for row in top..bottom {
            for col in left..right {
                if (col as f64 + 0.5 - cx).powi(2) + (row as f64 + 0.5 - cy).powi(2)
                    <= radius * radius
                {
                    let at = ((row * self.image.width + col) * 4) as usize;
                    self.image.pixels[at..at + 4].copy_from_slice(&color);
                }
            }
        }
    }
    fn line(&mut self, a: (f64, f64), b: (f64, f64), size: u32, color: [u8; 4]) {
        let a = (a.0.floor() + 0.5, a.1.floor() + 0.5);
        let b = (b.0.floor() + 0.5, b.1.floor() + 0.5);
        let (dx, dy) = (b.0 - a.0, b.1 - a.1);
        let squared = dx * dx + dy * dy;
        if squared == 0.0 {
            self.stamp(a.0, a.1, size, color);
            return;
        }
        // Rasterize the swept disk once, rather than stamping a large brush at
        // every integer coordinate along the segment.
        let radius = size as f64 / 2.0;
        let top = (a.1.min(b.1) - radius).floor().max(0.0) as u32;
        let bottom = (a.1.max(b.1) + radius).ceil().min(self.image.height as f64) as u32;
        for y in top..bottom {
            // Bound candidates per row, so thin diagonals do not scan the
            // segment's full bounding rectangle.
            let (first, last) = if dy == 0.0 {
                (0.0, 1.0)
            } else {
                let lo = (y as f64 + 0.5 - radius - a.1) / dy;
                let hi = (y as f64 + 0.5 + radius - a.1) / dy;
                (lo.min(hi).clamp(0.0, 1.0), lo.max(hi).clamp(0.0, 1.0))
            };
            let x0 = a.0 + first * dx;
            let x1 = a.0 + last * dx;
            let left = (x0.min(x1) - radius).floor().max(0.0) as u32;
            let right = (x0.max(x1) + radius).ceil().min(self.image.width as f64) as u32;
            for x in left..right {
                let (px, py) = (x as f64 + 0.5 - a.0, y as f64 + 0.5 - a.1);
                let t = ((px * dx + py * dy) / squared).clamp(0.0, 1.0);
                if (px - t * dx).powi(2) + (py - t * dy).powi(2) <= radius * radius {
                    let at = ((y * self.image.width + x) * 4) as usize;
                    self.image.pixels[at..at + 4].copy_from_slice(&color);
                }
            }
        }
    }
    fn fill(&mut self, x: u32, y: u32, color: [u8; 4]) {
        let width = self.image.width;
        let at = ((y * width + x) * 4) as usize;
        let target: [u8; 4] = self.image.pixels[at..at + 4].try_into().unwrap();
        if target == color {
            return;
        }
        let mut stack = vec![(x, y)];
        while let Some((x, y)) = stack.pop() {
            let same = |pixels: &[u8], col: u32, row: u32| {
                let at = ((row * width + col) * 4) as usize;
                pixels[at..at + 4] == target
            };
            if !same(&self.image.pixels, x, y) {
                continue;
            }
            let mut left = x;
            while left > 0 && same(&self.image.pixels, left - 1, y) {
                left -= 1;
            }
            let mut above = false;
            let mut below = false;
            for col in left..width {
                if !same(&self.image.pixels, col, y) {
                    break;
                }
                let at = ((y * width + col) * 4) as usize;
                self.image.pixels[at..at + 4].copy_from_slice(&color);
                let top = y > 0 && same(&self.image.pixels, col, y - 1);
                let bottom = y + 1 < self.image.height && same(&self.image.pixels, col, y + 1);
                if top && !above {
                    stack.push((col, y - 1));
                }
                if bottom && !below {
                    stack.push((col, y + 1));
                }
                above = top;
                below = bottom;
            }
        }
    }
}

/// Creates a white drawing surface. Dimensions are limited to 2048 × 2048.
#[ocamljs_bindgen(module=Paint,name="create")]
pub fn create(width: u32, height: u32) -> Result<Paint, ImageError> {
    if width == 0 || height == 0 || width > MAX_DIMENSION || height > MAX_DIMENSION {
        return Err(ImageError::TooLarge);
    }
    Ok(Paint::from_image(Image {
        width,
        height,
        pixels: vec![255; (width * height * 4) as usize],
        frame: None,
    }))
}

#[ocamljs_bindgen(module=Paint)]
pub fn status(paint: &Paint) -> PaintStatus {
    PaintStatus {
        width: paint.image.width,
        height: paint.image.height,
        can_undo: !paint.past.is_empty(),
        can_redo: !paint.future.is_empty(),
    }
}

/// Start one undoable gesture. RGB colors are 24-bit values; erasing paints white.
#[ocamljs_bindgen(module=Paint)]
pub fn begin(paint: &mut Paint, x: f64, y: f64, tool: Tool, rgb: u32, size: u32) {
    cancel(paint);
    if !x.is_finite() || !y.is_finite() {
        return;
    }
    let point = paint.point(x, y);
    let color = if tool == Tool::Eraser {
        [255; 4]
    } else {
        [(rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8, 255]
    };
    let before = paint.snapshot();
    paint.active = Some(Stroke {
        before,
        tool,
        start: point,
        last: point,
        color,
        size: size.clamp(1, 64),
    });
    if tool == Tool::Fill {
        paint.fill(point.0 as u32, point.1 as u32, color);
        paint.image.frame = None;
    } else {
        move_to(paint, x, y);
    }
}

/// Continue the current gesture without allocating a history snapshot.
#[ocamljs_bindgen(module=Paint)]
pub fn move_to(paint: &mut Paint, x: f64, y: f64) {
    if !x.is_finite() || !y.is_finite() {
        return;
    }
    let Some(mut stroke) = paint.active.take() else {
        return;
    };
    let point = paint.point(x, y);
    match stroke.tool {
        Tool::Brush | Tool::Eraser => paint.line(stroke.last, point, stroke.size, stroke.color),
        Tool::Fill => {
            paint.active = Some(stroke);
            return;
        }
        Tool::Line | Tool::Rectangle | Tool::Ellipse => {
            paint.image.pixels.copy_from_slice(&stroke.before.data);
            match stroke.tool {
                Tool::Line => paint.line(stroke.start, point, stroke.size, stroke.color),
                Tool::Rectangle => {
                    let a = stroke.start;
                    let b = point;
                    for (p, q) in [
                        (a, (b.0, a.1)),
                        ((b.0, a.1), b),
                        (b, (a.0, b.1)),
                        ((a.0, b.1), a),
                    ] {
                        paint.line(p, q, stroke.size, stroke.color);
                    }
                }
                Tool::Ellipse => {
                    let (cx, cy) = (
                        (stroke.start.0 + point.0) / 2.0,
                        (stroke.start.1 + point.1) / 2.0,
                    );
                    let (rx, ry) = (
                        (point.0 - stroke.start.0).abs() / 2.0,
                        (point.1 - stroke.start.1).abs() / 2.0,
                    );
                    let spacing = (stroke.size as f64 / 4.0).max(1.0);
                    let steps = (((rx + ry) * std::f64::consts::TAU) / spacing)
                        .ceil()
                        .max(8.0) as u32;
                    let mut previous = (cx + rx, cy);
                    for i in 0..=steps {
                        let t = i as f64 / steps as f64 * std::f64::consts::TAU;
                        let next = (cx + rx * t.cos(), cy + ry * t.sin());
                        paint.line(previous, next, stroke.size, stroke.color);
                        previous = next;
                    }
                }
                _ => unreachable!(),
            }
        }
    }
    stroke.last = point;
    paint.active = Some(stroke);
    paint.image.frame = None;
}

#[ocamljs_bindgen(module=Paint)]
pub fn finish(paint: &mut Paint) {
    if let Some(stroke) = paint.active.take() {
        paint.commit(stroke.before);
    }
    if let Some(transform) = paint.transform.take() {
        paint.commit(transform.before);
    }
}

#[ocamljs_bindgen(module=Paint)]
pub fn cancel(paint: &mut Paint) {
    if let Some(stroke) = paint.active.take() {
        paint.replace(stroke.before);
    }
    if let Some(transform) = paint.transform.take() {
        paint.replace(transform.before);
    }
}

/// Begin moving or resizing selected pixels. One snapshot covers all previews.
#[ocamljs_bindgen(module=Paint)]
pub fn transform_start(paint: &mut Paint, source: Region) -> Result<(), ImageError> {
    if !source.valid(&paint.image) {
        return Err(ImageError::InvalidCrop);
    }
    cancel(paint);
    paint.transform = Some(Transform {
        before: paint.snapshot(),
        source,
    });
    Ok(())
}

fn erase(image: &mut Image, region: Region) {
    for y in region.y..region.y + region.height {
        let start = ((y * image.width + region.x) * 4) as usize;
        image.pixels[start..start + region.width as usize * 4].fill(255);
    }
    image.frame = None;
}

/// Preview a move or nearest-neighbor resize from the original selection.
/// The vacated area is white. Previews reuse storage and do not add undo steps.
#[ocamljs_bindgen(module=Paint)]
pub fn transform_to(paint: &mut Paint, destination: Region) -> Result<(), ImageError> {
    if !destination.valid(&paint.image) {
        return Err(ImageError::InvalidCrop);
    }
    let Some(edit) = paint.transform.as_ref() else {
        return Err(ImageError::InvalidCrop);
    };
    let source = edit.source;
    paint.image.pixels.copy_from_slice(&edit.before.data);
    erase(&mut paint.image, source);
    for y in 0..destination.height {
        let source_y = source.y + y * source.height / destination.height;
        let out = (((destination.y + y) * paint.image.width + destination.x) * 4) as usize;
        let input = ((source_y * paint.image.width + source.x) * 4) as usize;
        if source.width == destination.width {
            let count = source.width as usize * 4;
            paint.image.pixels[out..out + count]
                .copy_from_slice(&edit.before.data[input..input + count]);
        } else {
            for x in 0..destination.width {
                let sample = input + (x * source.width / destination.width) as usize * 4;
                let target = out + x as usize * 4;
                paint.image.pixels[target..target + 4]
                    .copy_from_slice(&edit.before.data[sample..sample + 4]);
            }
        }
    }
    Ok(())
}

/// Delete the selected pixels as one undoable edit.
#[ocamljs_bindgen(module=Paint)]
pub fn delete_region(paint: &mut Paint, region: Region) -> Result<(), ImageError> {
    if !region.valid(&paint.image) {
        return Err(ImageError::InvalidCrop);
    }
    finish(paint);
    let before = paint.snapshot();
    erase(&mut paint.image, region);
    paint.commit(before);
    Ok(())
}

#[ocamljs_bindgen(module=Paint)]
pub fn undo(paint: &mut Paint) {
    finish(paint);
    if let Some(previous) = paint.past.pop_back() {
        let current = paint.replace(previous);
        paint.future.push(current);
        paint.trim();
    }
}
#[ocamljs_bindgen(module=Paint)]
pub fn redo(paint: &mut Paint) {
    finish(paint);
    if let Some(next) = paint.future.pop() {
        let current = paint.replace(next);
        paint.past.push_back(current);
        paint.trim();
    }
}
#[ocamljs_bindgen(module=Paint)]
pub fn clear(paint: &mut Paint) {
    finish(paint);
    let before = paint.snapshot();
    paint.image.pixels.fill(255);
    paint.image.frame = None;
    paint.commit(before);
}
#[ocamljs_bindgen(module=Paint,name="draw")]
pub fn draw_paint(paint: &mut Paint, canvas: &Canvas) -> Result<(), ImageError> {
    draw(&mut paint.image, canvas)
}

fn png_bytes(paint: &Paint) -> Result<Vec<u8>, ImageError> {
    let mut out = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut out, paint.image.width, paint.image.height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().map_err(|_| ImageError::InvalidPng)?;
        writer
            .write_image_data(&paint.image.pixels)
            .map_err(|_| ImageError::InvalidPng)?;
    }
    Ok(out)
}

#[ocamljs_bindgen(host)]
pub struct FileInput;
#[ocamljs_bindgen(host)]
pub struct FileList;
#[ocamljs_bindgen(host)]
pub struct File;
#[ocamljs_bindgen(host)]
pub struct ArrayBuffer;
#[ocamljs_bindgen]
unsafe extern "C" {
    #[ocamljs_bindgen(js_namespace = "document", js_name = "getElementById")]
    fn input(id: &str) -> Result<FileInput, HostError>;
    #[ocamljs_bindgen(getter, js_name = "files")]
    fn files(input: &FileInput) -> Result<FileList, HostError>;
    #[ocamljs_bindgen(getter, js_name = "length")]
    fn file_count(list: &FileList) -> Result<u32, HostError>;
    #[ocamljs_bindgen(indexing_getter)]
    fn file_at(list: &FileList, index: u32) -> Result<File, HostError>;
    #[ocamljs_bindgen(getter, js_name = "size")]
    fn file_size(file: &File) -> Result<f64, HostError>;
    #[ocamljs_bindgen(method, js_name = "arrayBuffer")]
    fn array_buffer(file: &File) -> Result<Promise<ArrayBuffer>, HostError>;
    #[ocamljs_bindgen(constructor, js_name = "Uint8ClampedArray")]
    fn bytes(buffer: &ArrayBuffer) -> Result<Uint8ClampedArray, HostError>;
}

/// Read a selected PNG asynchronously. A late result is a new, independent owner.
#[ocamljs_bindgen(module=Paint,name="open")]
pub async fn open_file(id: &str) -> Result<Paint, ImageError> {
    let list = files(&input(id).map_err(ImageError::Browser)?).map_err(ImageError::Browser)?;
    if file_count(&list).map_err(ImageError::Browser)? == 0 {
        return Err(ImageError::InvalidPng);
    }
    let file = file_at(&list, 0).map_err(ImageError::Browser)?;
    if file_size(&file).map_err(ImageError::Browser)? > 8.0 * 1024.0 * 1024.0 {
        return Err(ImageError::TooLarge);
    }
    let buffer = array_buffer(&file)
        .map_err(ImageError::Browser)?
        .await
        .map_err(ImageError::Browser)?;
    let bytes = bytes(&buffer)
        .map_err(ImageError::Browser)?
        .to_vec(8 * 1024 * 1024)
        .map_err(copy_error)?;
    Ok(Paint::from_image(open_png(&bytes)?))
}

/// Encode the current pixels as a native byte array for browser download.
#[ocamljs_bindgen(module=Paint)]
pub fn png(paint: &Paint) -> Result<Uint8ClampedArray, ImageError> {
    Uint8ClampedArray::copy_from(&png_bytes(paint)?).map_err(copy_error)
}

#[cfg(test)]
mod tests {
    include!("../../tests/rust/paint.rs");
}
