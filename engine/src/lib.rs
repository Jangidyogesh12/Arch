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
        // Magnitudes + travel direction per axis: one derivation (first
        // octant, study.md section 3.3) covers all 8 octants via sign steps
        // (study.md section 3.4). Works y-down as-is: the grid is symmetric.
        let dx = (self.p1.x - self.p0.x).abs();
        let dy = (self.p1.y - self.p0.y).abs();

        let sx = if self.p0.x < self.p1.x { 1 } else { -1 };
        let sy = if self.p0.y < self.p1.y { 1 } else { -1 };

        let mut x = self.p0.x;
        let mut y = self.p0.y;

        self.arr.clear();

        if dx >= dy {
            // Shallow: x drives, decision controls y. D0 = 2*dy - dx.
            let mut d = 2 * dy - dx;
            for _ in 0..=dx {
                self.arr.push(Point { x, y });
                if d >= 0 {
                    y += sy;
                    d += 2 * (dy - dx);
                } else {
                    d += 2 * dy;
                }
                x += sx;
            }
        } else {
            // Steep: y drives, decision controls x (axes swapped).
            let mut d = 2 * dx - dy;
            for _ in 0..=dy {
                self.arr.push(Point { x, y });
                if d >= 0 {
                    x += sx;
                    d += 2 * (dx - dy);
                } else {
                    d += 2 * dx;
                }
                y += sy;
            }
        }

        return self.arr.clone();
    }
}
