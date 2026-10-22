use arch_engine::{Camera, Canvas, Mat3, Scene, Sphere as EngineSphere, Vec3};
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

        let canvas = Canvas::new(2, 2, 800, 800, distance);
        Ok(Self {
            scene: Scene::new(Vec::new(), canvas, Camera::default()),
            width,
            height,
        })
    }

    #[wasm_bindgen(js_name = "setCamera")]
    pub fn set_camera(
        &mut self,
        px: f32,
        py: f32,
        pz: f32,
        tx: f32,
        ty: f32,
        tz: f32,
        ux: f32,
        uy: f32,
        uz: f32,
    ) -> Result<(), JsValue> {
        let values = [px, py, pz, tx, ty, tz, ux, uy, uz];
        if values.iter().any(|value| !value.is_finite()) {
            return Err(JsValue::from_str("Camera values must be finite"));
        }

        self.scene.camera = Camera::looking_at(
            Vec3::new(px, py, pz),
            Vec3::new(tx, ty, tz),
            Vec3::new(ux, uy, uz),
        );

        Ok(())
    }

    #[wasm_bindgen(js_name = "setCameraRotation")]
    pub fn set_camera_rotation(
        &mut self,
        m00: f32,
        m01: f32,
        m02: f32,
        m10: f32,
        m11: f32,
        m12: f32,
        m20: f32,
        m21: f32,
        m22: f32,
    ) -> Result<(), JsValue> {
        let values = [m00, m01, m02, m10, m11, m12, m20, m21, m22];
        if values.iter().any(|value| !value.is_finite()) {
            return Err(JsValue::from_str("Rotation matrix values must be finite"));
        }

        self.scene.camera.rotation = Mat3::from_cols_array(&values);

        Ok(())
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
        s: u32,
        reflective: f32,
    ) -> Result<(), JsValue> {
        if !x.is_finite() || !y.is_finite() || !z.is_finite() || !radius.is_finite() || radius <= 0.
        {
            return Err(JsValue::from_str(
                "Sphere coordinates must be finite and radius must be positive",
            ));
        }
        if !reflective.is_finite() || !(0.0..=1.0).contains(&reflective) {
            return Err(JsValue::from_str(
                "Reflectivity must be finite and between 0 and 1",
            ));
        }
        self.scene.spheres.push(EngineSphere::new(
            (x, y, z),
            radius,
            Vec3::new(r as f32, g as f32, b as f32),
            s,
            reflective,
        ));

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
