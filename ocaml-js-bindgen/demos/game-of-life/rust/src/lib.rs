#[ocamljs_bindgen::ocamljs_bindgen(opaque_t = Universe)]
mod universe {
    #[ocamljsify::ocamljsify]
    #[derive(Debug, PartialEq)]
    pub enum GridError {
        InvalidDimensions,
        OutsideGrid { x: u32, y: u32 },
    }
    #[ocamljsify::ocamljsify]
    pub struct Snapshot {
        pub width: u32,
        pub height: u32,
        pub generation: u32,
        pub cells: Vec<bool>,
    }
    pub struct Universe {
        width: u32,
        height: u32,
        generation: u32,
        cells: Vec<bool>,
        next: Vec<bool>,
    }
    #[ocamljs_bindgen]
    pub fn create(width: u32, height: u32) -> Result<Universe, GridError> {
        if width < 3 || height < 3 || width > 256 || height > 256 {
            return Err(GridError::InvalidDimensions);
        }
        let n = (width * height) as usize;
        Ok(Universe {
            width,
            height,
            generation: 0,
            cells: vec![false; n],
            next: vec![false; n],
        })
    }
    #[ocamljs_bindgen]
    pub fn toggle(u: &mut Universe, x: u32, y: u32) -> Result<(), GridError> {
        if x >= u.width || y >= u.height {
            return Err(GridError::OutsideGrid { x, y });
        }
        let c = &mut u.cells[(y * u.width + x) as usize];
        *c = !*c;
        Ok(())
    }
    #[ocamljs_bindgen]
    pub fn step(u: &mut Universe) -> u32 {
        for y in 0..u.height {
            for x in 0..u.width {
                let mut n = 0;
                for dy in [-1i32, 0, 1] {
                    for dx in [-1i32, 0, 1] {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        let xx = (x as i32 + dx).rem_euclid(u.width as i32) as u32;
                        let yy = (y as i32 + dy).rem_euclid(u.height as i32) as u32;
                        n += u32::from(u.cells[(yy * u.width + xx) as usize]);
                    }
                }
                let i = (y * u.width + x) as usize;
                u.next[i] = n == 3 || u.cells[i] && n == 2;
            }
        }
        std::mem::swap(&mut u.cells, &mut u.next);
        u.generation += 1;
        u.generation
    }
    #[ocamljs_bindgen]
    pub fn alive(u: &Universe, x: u32, y: u32) -> bool {
        x < u.width && y < u.height && u.cells[(y * u.width + x) as usize]
    }
    #[ocamljs_bindgen]
    pub fn snapshot(u: &Universe) -> Snapshot {
        Snapshot {
            width: u.width,
            height: u.height,
            generation: u.generation,
            cells: u.cells.clone(),
        }
    }
}
pub use universe::*;
