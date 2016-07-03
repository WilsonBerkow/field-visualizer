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

    let mut app: App = App {
        gl: GlGraphics::new(opengl),
        arrows: vec![
            // Front square:
            Arrow3::new(0.0, 0.0, 0.0,
                        50.0, 0.0, 0.0),
            Arrow3::new(50.0, 0.0, 0.0,
                        0.0, 50.0, 0.0),
            Arrow3::new(50.0, 50.0, 0.0,
                        -50.0, 0.0, 0.0),
            Arrow3::new(0.0, 50.0, 0.0,
                        0.0, -50.0, 0.0),
            // Connecting edges:
            Arrow3::new(0.0, 0.0, 0.0,
                        0.0, 0.0, 50.0),
            Arrow3::new(50.0, 0.0, 0.0,
                        0.0, 0.0, 50.0),
            Arrow3::new(0.0, 50.0, 0.0,
                        0.0, 0.0, 50.0),
            Arrow3::new(50.0, 50.0, 0.0,
                        0.0, 0.0, 50.0),
            // Back square:
            Arrow3::new(0.0, 0.0, 50.0,
                        50.0, 0.0, 0.0),
            Arrow3::new(50.0, 0.0, 50.0,
                        0.0, 50.0, 0.0),
            Arrow3::new(50.0, 50.0, 50.0,
                        -50.0, 0.0, 0.0),
            Arrow3::new(0.0, 50.0, 50.0,
                        0.0, -50.0, 0.0),
        ],
        persp: PerspectiveMatrix3::new(1.0, 200.0/*2 * std::f64::consts::PI* /4.0*/, 0.0, 100.0),
        rot: 0.0,
    };
    //Arrow3::new(0.0, 0.0, 1.0, 20.0, 50.0, 10.0)

    while let Some(e) = events.next(&mut window) {
        if let Some(r) = e.render_args() {
            app.render(&r);
        }
        if let Some(u) = e.update_args() {
            app.update(&u);
        }
    }
}

struct App {
    gl: GlGraphics,
    arrows: Vec<Arrow3>,
    persp: PerspectiveMatrix3<f64>,
    rot: f64,
}

impl App {
    fn update(&mut self, args: &UpdateArgs) {
        print!("Increasing rot by {} radians to {}", args.dt, self.rot);
        self.rot += args.dt; // One radian per second
    }

    fn render(&mut self, args: &RenderArgs) {
        graphics::clear(BG_CLR, &mut self.gl);
        let persp = &self.persp;
        let rot = na::Rotation3::new(na::Vector3::new(0.0, self.rot, 0.0)); //std::f64::consts::PI * 0.25, std::f64::consts::PI * 0.25, 0.0));
        for arrow in &self.arrows {
            self.gl.draw(args.viewport(), |c, gl| {
                use graphics::*;
                arrow.draw(c, gl, persp, rot);
                //Line::new([1.0, 0.0, 0.0, 1.0], 5.0).draw([-10.0, -5.0, -20.0, -10.0], &c.draw_state, c.transform, gl);
            });
        }
    }
}

struct Arrow3 {
    x: f64,
    y: f64,
    z: f64,
    dx: f64,
    dy: f64,
    dz: f64,
}

impl Arrow3 {
    fn new(x: f64, y: f64, z: f64, dx: f64, dy: f64, dz: f64) -> Arrow3 {
        Arrow3 { x: x - 25.0, y: y - 25.0, z: z - 25.0, dx: dx, dy: dy, dz: dz }
    }

    fn point0(&self) -> Point3<f64> { Point3::new(self.x, self.y, self.z) }

    fn point1(&self) -> Point3<f64> { Point3::new(self.x + self.dx, self.y + self.dy, self.z + self.dz) }

    fn rot_project(&self, rot: na::Rotation3<f64>, persp: &PerspectiveMatrix3<f64>) -> Arrow {
        let pt0 = self.point0();
        let pt1 = self.point1();
        let mut pt0r = rot.rotate(&pt0);
        let mut pt1r = rot.rotate(&pt1);
        pt0r.z += 51.0 + 40.0;
        pt1r.z += 51.0 + 40.0;
        let pt0_prime = persp.project_point(&pt0r);
        let pt1_prime = persp.project_point(&pt1r);
        Arrow {
            x: pt0_prime.x * 150.0 + WIDTHF_2,
            y: pt0_prime.y * 150.0 + HEIGHTF_2,
            dx: (pt1_prime.x - pt0_prime.x) * 150.0,
            dy: (pt1_prime.y - pt0_prime.y) * 150.0,
        }
    }

    fn draw(&self, c: graphics::context::Context, gl: &mut GlGraphics, persp: &PerspectiveMatrix3<f64>, rot: na::Rotation3<f64>) {
        use graphics::*;
        let a2: Arrow = self.rot_project(rot, persp);
        a2.draw(c, gl);
    }

    fn update(&mut self, args: &UpdateArgs) {
    }
}

struct Arrow {
    x: f64,
    y: f64,
    dx: f64,
    dy: f64,
}

impl Arrow {
    fn new(x: f64, y: f64, dx: f64, dy: f64) -> Arrow { Arrow { x: x, y: y, dx: dx, dy: dy } }

    fn draw(&self, c: graphics::context::Context, gl: &mut GlGraphics) {
        use graphics::*;
        let path = [self.x, self.y, self.x + self.dx, self.y + self.dy];
        let line_style = Line::new([1.0, 1.0, 1.0, 1.0], 1.0);
        line_style.draw_arrow(path, 5.0, &c.draw_state, c.transform, gl);
        println!("({}, {}) by ({}, {})", self.x, self.y, self.dx, self.dy);
    }

    fn update(&mut self, args: &UpdateArgs) {
    }

}
