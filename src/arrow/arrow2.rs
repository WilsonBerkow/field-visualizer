use graphics;
use opengl_graphics::GlGraphics;

use na;

use consts::*;

pub struct Arrow2 {
    tail: na::Point2<f64>,
    head: na::Point2<f64>,
    clr: [f32; 4],
}

impl Arrow2 {
    pub fn from_to_clr(tail: na::Point2<f64>, head: na::Point2<f64>, clr: [f32; 4]) -> Arrow2 {
        Arrow2 {
            tail: tail,
            head: head,
            clr: clr,
        }
    }

    pub fn from_to(tail: na::Point2<f64>, head: na::Point2<f64>) -> Arrow2 {
        Arrow2 {
            tail: tail,
            head: head,
            clr: ARROW_CLR,
        }
    }

    pub fn draw(&self, c: graphics::context::Context, gl: &mut GlGraphics) {
        let path = [self.tail.x, self.tail.y, self.head.x, self.head.y];
        let line_style = graphics::Line::new(self.clr, 1.0);
        line_style.draw_arrow(path, 5.0, &c.draw_state, c.transform, gl);
    }

    pub fn draw_no_head(&self, c: graphics::context::Context, gl: &mut GlGraphics) {
        let path = [self.tail.x, self.tail.y, self.head.x, self.head.y];
        let line_style = graphics::Line::new(self.clr, 1.0);
        line_style.draw(path, &c.draw_state, c.transform, gl);
    }
}
