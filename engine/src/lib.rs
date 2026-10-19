use glam::Vec3;

pub struct Sphere {
    pub center: (f32, f32, f32),
    pub radius: f32,
    pub color: (u8, u8, u8),
}

impl Sphere {
    pub fn new(center: (f32, f32, f32), radius: f32, color: (u8, u8, u8)) -> Self {
        Self {
            center,
            radius,
            color,
        }
    }
}

pub struct Canvas {
    pub v_w: i32,
    pub v_h: i32,
    pub c_w: i32,
    pub c_h: i32,
    pub d: i32,
}

impl Canvas {
    pub fn new(v_w: i32, v_h: i32, c_w: i32, c_h: i32, d: i32) -> Self {
        Self {
            v_w,
            v_h,
            c_w,
            c_h,
            d,
        }
    }
}

pub struct Scene {
    pub spheres: Vec<Sphere>,
    pub canvas: Canvas,
}

impl Scene {
    const ORIGIN: Vec3 = Vec3::new(0.0, 0.0, 0.0);
    const BACK_GROUND_COLOR: (u8, u8, u8) = (255, 255, 255);
    pub fn new(spheres: Vec<Sphere>, canvas: Canvas) -> Self {
        Self {
            spheres: spheres,
            canvas,
        }
    }

    pub fn trace_ray(&self, d: Vec3, t_min: f32, t_max: f32) -> (u8, u8, u8) {
        let mut closes_t = f32::MAX;
        let mut closest_sphere = None;

        for sphere in &self.spheres {
            let (t1, t2) = self.find_intersection_points(sphere, d);
            if t1 >= t_min && t2 <= t_max && t1 < closes_t {
                closes_t = t1;
                closest_sphere = Some(sphere);
            }

            if t2 >= t_min && t2 <= t_max && t2 < closes_t {
                closes_t = t2;
                closest_sphere = Some(sphere);
            }
        }

        match closest_sphere {
            Some(sphere) => sphere.color,
            None => Self::BACK_GROUND_COLOR,
        }
    }

    pub fn canvas_to_viewport(&self, x: f32, y: f32) -> Vec3 {
        let scale1 = self.canvas.v_w as f32 / self.canvas.c_w as f32;
        let scale2 = self.canvas.v_h as f32 / self.canvas.c_h as f32;
        Vec3::new(x * scale1, y * scale2, self.canvas.d as f32)
    }

    pub fn find_intersection_points(&self, sphere: &Sphere, d: Vec3) -> (f32, f32) {
        let r = sphere.radius;
        let co = Self::ORIGIN - Vec3::from(sphere.center);
        let a = d.dot(d);
        let b = 2.0 * co.dot(d);
        let c = co.dot(co) - r * r;

        let discriminent = b * b - 4.0 * a * c;

        if discriminent < 0.0 {
            return (f32::MAX, f32::MAX);
        }

        let t1 = (-b + discriminent.sqrt()) / (2.0 * a);
        let t2 = (-b - discriminent.sqrt()) / (2.0 * a);

        return (t1, t2);
    }

    pub fn paint_canvas(&self) -> Vec<(i32, i32, (u8, u8, u8))> {
        let mut result: Vec<(i32, i32, (u8, u8, u8))> = Vec::new();
        for x in (-self.canvas.c_w / 2)..(self.canvas.c_w / 2) {
            for y in (-self.canvas.c_h / 2)..(self.canvas.c_h / 2) {
                let d = self.canvas_to_viewport(x as f32, y as f32);
                let color = self.trace_ray(d, 1e-4, f32::MAX);

                result.push((x, y, color));
            }
        }

        return result;
    }
}
