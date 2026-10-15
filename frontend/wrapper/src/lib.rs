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
        for point in self.line.get_line_pxl_coordinates_using_bresenham() {
            flat.push(point.x);
            flat.push(point.y);
        }
        return flat;
    }

    pub fn get_line_aa_pxl_coordinates(&mut self) -> Vec<i32> {
        let mut flat = Vec::new();
        for (point, intensity) in self.line.get_line_pxl_coordinates_using_wu() {
            let alpha = (intensity.clamp(0.0, 1.0) * 255.0).round() as i32;
            flat.push(point.x);
            flat.push(point.y);
            flat.push(alpha);
        }
        return flat;
    }
}
