use arch_engine::{Canvas, Scene, Sphere as EngineSphere};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct PaintScene {
    scene: Scene,
    width: i32,
    height: i32,
}

#[wasm_bindgen]

impl PaintScene {
    #[wasm_bindgen(constructor)]
    pub fn new(width: i32, height: i32, distance: i32) -> Result<PaintScene, JsValue> {
        if width <= 0 || height <= 0 || distance <= 0 {
            return Err(JsValue::from_str(
                "Width, height, and distance must be positive",
            ));
        }

        let canvas = Canvas::new(1, 1, width, height, distance);
        Ok(Self {
            scene: Scene::new(Vec::new(), canvas),
            width,
            height,
        })
    }

    #[wasm_bindgen(js_name = "addSphere")]
    pub fn add_sphere(
        &mut self,
        x: f32,
        y: f32,
        z: f32,
        radius: f32,
        r: u8,
        g: u8,
        b: u8,
    ) -> Result<(), JsValue> {
        if !x.is_finite() || !y.is_finite() || !z.is_finite() || !radius.is_finite() || radius <= 0.
        {
            return Err(JsValue::from_str(
                "Sphere coordinates must be finite and radius must be positive",
            ));
        }
        self.scene
            .spheres
            .push(EngineSphere::new((x, y, z), radius, (r, g, b)));

        Ok(())
    }

    pub fn clear(&mut self) {
        self.scene.spheres.clear();
    }

    pub fn render(&self) -> Vec<u8> {
        let mut pixels = vec![255; (self.width * self.height * 4) as usize];

        for (x, y, (r, g, b)) in self.scene.paint_canvas() {
            let px = x + (self.width / 2);
            let py = (self.height / 2) - (1 + y);

            if px < 0 || px >= self.width || py < 0 || py >= self.height {
                continue;
            }

            let index = ((py * self.width + px) * 4) as usize;

            pixels[index] = r.clamp(0, 255) as u8;
            pixels[index + 1] = g.clamp(0, 255) as u8;
            pixels[index + 2] = b.clamp(0, 255) as u8;
            pixels[index + 3] = 255;
        }

        pixels
    }
}
