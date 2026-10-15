use arch_engine::DrawLine;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]

pub struct DrawLineWrapper {
    line: DrawLine,
}

#[wasm_bindgen]
impl DrawLineWrapper {
    #[wasm_bindgen(constructor)]
    pub fn new(x0: i32, y0: i32, x1: i32, y1: i32) -> Self {
        Self {
            line: DrawLine::new(x0, y0, x1, y1),
        }
    }

    pub fn get_line_pxl_coordinates(&mut self) -> Vec<i32> {
        let mut flat = Vec::new();
        for point in self.line.get_line_pxl_coordinates_using_dda() {
            flat.push(point.x);
            flat.push(point.y);
        }
        return flat;
    }
}
