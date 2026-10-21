use glam::Vec3;

pub struct Sphere {
    pub center: (f32, f32, f32),
    pub radius: f32,
    pub color: (u8, u8, u8),
    pub specular: u32,
}

impl Sphere {
    pub fn new(center: (f32, f32, f32), radius: f32, color: (u8, u8, u8), specular: u32) -> Self {
        Self {
            center,
            radius,
            color,
            specular,
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

#[derive(Clone, Copy)]
pub enum LightKind {
    Ambient,
    Point { position: Vec3 },
    Direction { direction: Vec3 },
}

pub struct Light {
    pub kind: LightKind,
    pub intensity: f32,
}

pub struct Scene {
    pub spheres: Vec<Sphere>,
    pub canvas: Canvas,
}

impl Scene {
    const ORIGIN: Vec3 = Vec3::new(0.0, 0.0, 0.0);
    const BACK_GROUND_COLOR: (u8, u8, u8) = (255, 255, 255);
    const LIGHTS: [Light; 3] = [
        Light {
            kind: LightKind::Ambient,
            intensity: 0.1,
        },
        Light {
            kind: LightKind::Point {
                position: Vec3::new(2.0, 1.0, 0.0),
            },
            intensity: 0.6,
        },
        Light {
            kind: LightKind::Direction {
                direction: Vec3::new(1.0, 4.0, 4.0),
            },
            intensity: 0.2,
        },
    ];

    pub fn new(spheres: Vec<Sphere>, canvas: Canvas) -> Self {
        Self {
            spheres: spheres,
            canvas,
        }
    }

    pub fn closest_sphere(
        &self,
        origin: Vec3,
        d: Vec3,
        t_min: f32,
        t_max: f32,
    ) -> (Option<&Sphere>, f32) {
        let mut closest_t = f32::MAX;
        let mut closest_sphere = None;
        for sphere in &self.spheres {
            let (t1, t2) = self.find_intersection_points(origin, sphere, d);
            if t1 >= t_min && t2 <= t_max && t1 < closest_t {
                closest_t = t1;
                closest_sphere = Some(sphere);
            }

            if t2 >= t_min && t2 <= t_max && t2 < closest_t {
                closest_t = t2;
                closest_sphere = Some(sphere);
            }
        }

        (closest_sphere, closest_t)
    }

    pub fn trace_ray(&self, d: Vec3, t_min: f32, t_max: f32) -> (u8, u8, u8) {
        let (closest_sphere, closest_t) = self.closest_sphere(Self::ORIGIN, d, t_min, t_max);

        match closest_sphere {
            Some(sphere) => {
                let p = Self::ORIGIN + closest_t * d;
                let n = (p - Vec3::from(sphere.center)).normalize_or_zero();
                let intensity = self.compute_lighting(p, n, -d, sphere.specular);
                let a = intensity * (sphere.color.0 as f32);
                let b = intensity * (sphere.color.1 as f32);
                let c = intensity * (sphere.color.2 as f32);

                (a as u8, b as u8, c as u8)
            }
            None => Self::BACK_GROUND_COLOR,
        }
    }

    pub fn canvas_to_viewport(&self, x: f32, y: f32) -> Vec3 {
        let scale1 = self.canvas.v_w as f32 / self.canvas.c_w as f32;
        let scale2 = self.canvas.v_h as f32 / self.canvas.c_h as f32;
        Vec3::new(x * scale1, y * scale2, self.canvas.d as f32)
    }

    pub fn find_intersection_points(&self, origin: Vec3, sphere: &Sphere, d: Vec3) -> (f32, f32) {
        let r = sphere.radius;
        let co = origin - Vec3::from(sphere.center);
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

    pub fn compute_lighting(&self, p: Vec3, n: Vec3, v: Vec3, s: u32) -> f32 {
        let mut intensity = 0.0;

        for light in Self::LIGHTS {
            let (l, t_max) = match light.kind {
                LightKind::Ambient => {
                    intensity += light.intensity;
                    continue;
                }
                LightKind::Point { position } => (position - p, 1.0),
                LightKind::Direction { direction } => (direction, f32::MAX),
            };

            // Shadow on the surface
            let (closest_sphere, _) = self.closest_sphere(p, l, 1e-4, t_max);
            if closest_sphere.is_some() {
                continue;
            }

            // Diffuse Reflection
            let n_dot_l = n.dot(l);
            if n_dot_l > 0.0 {
                intensity += light.intensity * n_dot_l / (n.length() * l.length());
            }

            // Specular Reflection
            let r = 2.0 * n * n.dot(l) - l;
            let r_dot_v = r.dot(v);

            if r_dot_v > 0.0 {
                intensity += light.intensity * (r_dot_v / (r.length() * v.length())).powf(s as f32);
            }
        }

        intensity
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
