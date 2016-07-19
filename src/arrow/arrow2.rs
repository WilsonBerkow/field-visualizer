use pw;

use na::Point2;

pub struct Arrow2 {
    pub tail: Point2<f64>,
    pub head: Point2<f64>,
    clr: [f32; 4],
}

impl Arrow2 {
    pub fn from_to_clr(tail: Point2<f64>, head: Point2<f64>, clr: [f32; 4]) -> Arrow2 {
        Arrow2 {
            tail: tail,
            head: head,
            clr: clr,
        }
    }

    pub fn draw(&self, c: pw::Context, gl: &mut pw::G2d) {
        let path = [self.tail.x, self.tail.y, self.head.x, self.head.y];
        let line_style = pw::Line::new(self.clr, 1.0);
        line_style.draw_arrow(path, 5.0, &c.draw_state, c.transform, gl);
    }
}
