#[derive(Debug, Clone)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

pub struct DrawLine {
    p0: Point,
    p1: Point,
    arr: Vec<Point>,
}

impl DrawLine {
    pub fn new(x0: i32, y0: i32, x1: i32, y1: i32) -> Self {
        Self {
            p0: Point { x: x0, y: y0 },
            p1: Point { x: x1, y: y1 },
            arr: Vec::new(),
        }
    }

    pub fn get_line_pxl_coordinates_using_dda(&mut self) -> Vec<Point> {
        let dx = self.p1.x - self.p0.x;
        let dy = self.p1.y - self.p0.y;

        let steps = dy.abs().max(dx.abs());

        self.arr.clear();

        if steps == 0 {
            self.arr.push(self.p0.clone());
            return self.arr.clone();
        }

        let x_inc = dx as f64 / steps as f64;
        let y_inc = dy as f64 / steps as f64;

        let mut x = self.p0.x as f64;
        let mut y = self.p0.y as f64;

        for _ in 0..=steps {
            self.arr.push(Point {
                x: x.round() as i32,
                y: y.round() as i32,
            });
            x += x_inc;
            y += y_inc;
        }

        return self.arr.clone();
    }

    pub fn get_line_pxl_coordinates_using_bresenham(&mut self) -> Vec<Point> {
        let mut x = self.p0.x;
        let mut y = self.p0.y;

        let dx = (self.p1.x - self.p0.x).abs();
        let dy = (self.p1.y - self.p0.y).abs();

        let sx = if self.p0.x < self.p1.x { 1 } else { -1 };
        let sy = if self.p0.y < self.p1.y { 1 } else { -1 };

        let mut err = dx - dy;

        self.arr.clear();

        loop {
            self.arr.push(Point { x, y });

            if x == self.p1.x && y == self.p1.y {
                break;
            }

            let e2 = 2 * err;

            if e2 > -dy {
                err -= dy;
                x += sx;
            }

            if e2 < dx {
                err += dx;
                y += sy;
            }
        }

        return self.arr.clone();
    }

    pub fn get_line_pxl_coordinates_using_wu(&mut self) -> Vec<(Point, f64)> {
        // Xiaolin Wu anti-aliasing (study.md section 3.6): instead of one
        // pixel per column, plot the two pixels straddling the true line,
        // weighted by coverage. Each column's pair sums to ~1.0, endpoints
        // are dimmed by their fractional coverage (xgap). Returns pixels
        // with intensity in 0.0..=1.0; blending happens on the frontend.
        fn frac(v: f64) -> f64 {
            return v - v.floor();
        }
        fn rfrac(v: f64) -> f64 {
            return 1.0 - frac(v);
        }

        let mut pixels: Vec<(Point, f64)> = Vec::new();
        let mut plot = |x: i32, y: i32, steep: bool, intensity: f64| {
            if intensity <= 0.0 {
                return;
            }
            if steep {
                pixels.push((Point { x: y, y: x }, intensity));
            } else {
                pixels.push((Point { x, y }, intensity));
            }
        };

        let mut x0 = self.p0.x as f64;
        let mut y0 = self.p0.y as f64;
        let mut x1 = self.p1.x as f64;
        let mut y1 = self.p1.y as f64;

        if x0 == x1 && y0 == y1 {
            plot(x0 as i32, y0 as i32, false, 1.0);
            return pixels;
        }

        // Work in "x drives" space: transpose steep lines, order leftward.
        let steep = (y1 - y0).abs() > (x1 - x0).abs();
        if steep {
            std::mem::swap(&mut x0, &mut y0);
            std::mem::swap(&mut x1, &mut y1);
        }
        if x0 > x1 {
            std::mem::swap(&mut x0, &mut x1);
            std::mem::swap(&mut y0, &mut y1);
        }

        // dx > 0 guaranteed: dot returned early, vertical became horizontal.
        let dx = x1 - x0;
        let gradient = (y1 - y0) / dx;

        // First endpoint, dimmed by fractional coverage.
        let xend = x0.round();
        let yend = y0 + gradient * (xend - x0);
        let xgap = rfrac(x0 + 0.5);
        let xpxl1 = xend as i32;
        plot(xpxl1, yend.floor() as i32, steep, rfrac(yend) * xgap);
        plot(xpxl1, yend.floor() as i32 + 1, steep, frac(yend) * xgap);
        let mut intery = yend + gradient;

        // Second endpoint (drawn after the main loop).
        let xend2 = x1.round();
        let yend2 = y1 + gradient * (xend2 - x1);
        let xgap2 = frac(x1 + 0.5);
        let xpxl2 = xend2 as i32;

        let mut x = xpxl1 + 1;
        while x < xpxl2 {
            plot(x, intery.floor() as i32, steep, rfrac(intery));
            plot(x, intery.floor() as i32 + 1, steep, frac(intery));
            intery += gradient;
            x += 1;
        }

        plot(xpxl2, yend2.floor() as i32, steep, rfrac(yend2) * xgap2);
        plot(xpxl2, yend2.floor() as i32 + 1, steep, frac(yend2) * xgap2);

        return pixels;
    }
}
