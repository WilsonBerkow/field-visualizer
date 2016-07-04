extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;
extern crate nalgebra as na;

use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{ GlGraphics, OpenGL };
use na::{ Rotate, Point3, PerspectiveMatrix3 };
use std::f64::consts::PI;

const WIDTH: u32 = 600;
const HEIGHT: u32 = 600;
const WIDTHF: f64 = WIDTH as f64;
const HEIGHTF: f64 = HEIGHT as f64;
const WIDTHF_2: f64 = WIDTHF * 0.5;
const HEIGHTF_2: f64 = HEIGHTF * 0.5;

const BG_CLR: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

fn main() {
    let opengl: OpenGL = OpenGL::V3_2;
    let mut window: Window = WindowSettings::new(
            "vectors-starting",
            [WIDTH, HEIGHT]
        )
        .opengl(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();
    let mut events = window.events();

    let s: f64 = 25.0;
    let mut app: App = App {
        gl: GlGraphics::new(opengl),
        arrows: vec![
            // Front square:
            Arrow3::new(-s, -s, -s,
                        s, -s, -s),
            Arrow3::new(s, -s, -s,
                        s, s, -s),
            Arrow3::new(s, s, -s,
                        -s, s, -s),
            Arrow3::new(-s, s, -s,
                        -s, -s, -s),
            // Connecting edges:
            Arrow3::new(-s, -s, -s,
                        -s, -s, s),
            Arrow3::new(s, -s, -s,
                        s, -s, s),
            Arrow3::new(-s, s, -s,
                        -s, s, s),
            Arrow3::new(s, s, -s,
                        s, s, s),
            // Back square:
            Arrow3::new(-s, -s, s,
                        s, -s, s),
            Arrow3::new(s, -s, s,
                        s, s, s),
            Arrow3::new(s, s, s,
                        -s, s, s),
            Arrow3::new(-s, s, s,
                        -s, -s, s),
        ],
        persp: PerspectiveMatrix3::new(1.0, 200.0, 0.0, 100.0),
        rot_x: 0.0,
        rot_y: 0.0,
        rot_z: 0.0,
    };

    while let Some(e) = events.next(&mut window) {
        if let Some(r) = e.render_args() {
            app.render(&r);
        }
        if let Some(u) = e.update_args() {
            app.update(&u);
        }
        if let Some(Button::Keyboard(k)) = e.press_args() {
            app.keypress(k);
        }
    }
}

struct App {
    gl: GlGraphics,
    arrows: Vec<Arrow3>,
    persp: PerspectiveMatrix3<f64>,
    rot_x: f64,
    rot_y: f64,
    rot_z: f64,
}

impl App {
    fn update(&mut self, args: &UpdateArgs) {
        self.rot_y += args.dt; // One radian per second
    }

    fn render(&mut self, args: &RenderArgs) {
        graphics::clear(BG_CLR, &mut self.gl);
        let persp = &self.persp;
        let rot = na::Rotation3::new_with_euler_angles(
                self.rot_x,
                self.rot_y,
                self.rot_z
            );
        for arrow in &self.arrows {
            self.gl.draw(args.viewport(), |c, gl| {
                use graphics::*;
                arrow.draw(c, gl, persp, rot);
            });
        }
    }

    fn keypress(&mut self, key: Key) {
        match key {
            Key::Up => {
                self.rot_x += PI * 0.01;
            },
            Key::Down => {
                self.rot_x -= PI * 0.01;
            },
            Key::Right => {
                self.rot_z += PI * 0.01;
            },
            Key::Left => {
                self.rot_z -= PI * 0.01;
            },
            _ => {},
        }
    }
}

struct Arrow3 {
    tail: Point3,
    head: Point3,
}

impl Arrow3 {
    fn new(tx: f64, ty: f64, tz: f64, hx: f64, hy: f64, hz: f64) -> Arrow3 {
        Arrow3 {
            tail: Point3::new(tx, ty, tz),
            head: Point3::new(hx, hy, hz),
        }
    }

    fn project_to_viewport(&self, rot: na::Rotation3<f64>, persp: &PerspectiveMatrix3<f64>) -> Arrow {
        // Apply cube's rotation:
        let mut headr = rot.rotate(&self.head);
        let mut tailr = rot.rotate(&self.tail);
        // Transform relative to the camera position:
        headr.z += 51.0 + 40.0;
        tailr.z += 51.0 + 40.0;
        // Project onto "device" surface:
        let head_prime = persp.project_point(&headr);
        let tail_prime = persp.project_point(&tailr);
        // Trasform to viewport surface:
        Arrow {
            tx: head_prime.x * 150.0 + WIDTHF_2,
            ty: head_prime.y * 150.0 + HEIGHTF_2,
            hx: tail_prime.x * 150.0 + WIDTHF_2,
            hy: tail_prime.y * 150.0 + HEIGHTF_2,
        }
    }

    fn draw(&self, c: graphics::context::Context, gl: &mut GlGraphics, persp: &PerspectiveMatrix3<f64>, rot: na::Rotation3<f64>) {
        use graphics::*;
        let a2: Arrow = self.project_to_viewport(rot, persp);
        a2.draw(c, gl);
    }
}

struct Arrow {
    tx: f64,
    ty: f64,
    hx: f64,
    hy: f64,
}

impl Arrow {
    fn new(tx: f64, ty: f64, hx: f64, hy: f64) -> Arrow { Arrow { tx: tx, ty: ty, hx: hx, hy: hy } }

    fn draw(&self, c: graphics::context::Context, gl: &mut GlGraphics) {
        use graphics::*;
        let path = [self.tx, self.ty, self.hx, self.hy];
        let line_style = Line::new([1.0, 1.0, 1.0, 1.0], 1.0);
        line_style.draw_arrow(path, 5.0, &c.draw_state, c.transform, gl);
    }
}
