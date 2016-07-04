extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;
extern crate num;
extern crate nalgebra as na;

use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{ GlGraphics, OpenGL };
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
        camera: {
            //use std::num::One;
            //use std::num::Zero;
            //use std::ops::Add;
            //use std::ops::Mul;
            //use std::ops::Div;
            //use std::ops::Rem;
            //use std::ops::AddAssign;
            //use std::ops::SubAssign;
            //use std::ops::MulAssign;
            //use std::ops::DivAssign;
            //use std::ops::RemAssign;
            //use std::cmp::PartialEq;
            //use na::Axpy;
            //use na::Absolute;
            //use na::BaseNum;
            //use std::marker::Copy;
            //use
            //let uno = One::one() + Zero::zero();
            use num::One;
            One::one()
        },
        persp: na::PerspectiveMatrix3::new(1.0, 200.0, 0.0, 100.0),
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

fn translation3_mat<T: na::BaseNum + Copy>(v: na::Vector3<T>) -> na::Matrix4<T> {
    let mut res: na::Matrix4<T> = num::One::one();
    res.m14 = v.x;
    res.m24 = v.y;
    res.m34 = v.z;
    res
}

struct App {
    gl: GlGraphics,
    arrows: Vec<Arrow3>,
    persp: na::PerspectiveMatrix3<f64>,
    camera: na::Matrix4<f64>, // camera transform from space to locations relative to camera
    rot_x: f64,
    rot_y: f64,
    rot_z: f64,
}

impl App {
    fn update(&mut self, args: &UpdateArgs) {
        //self.rot_y += args.dt; // One radian per second
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
            let cam = self.camera;
            self.gl.draw(args.viewport(), |c, gl| {
                arrow.draw(c, gl, persp, rot, cam.clone());
            });
        }
    }

    fn keypress(&mut self, key: Key) {
        use na::ToHomogeneous;
        match key {
            Key::Up => {
                let rotmat = na::Rotation3::new(na::Vector3::new(PI * 0.01, 0.0, 0.0));
                self.camera *= rotmat.submatrix().to_homogeneous();
            },
            Key::Down => {
                let rotmat = na::Rotation3::new(na::Vector3::new(-PI * 0.01, 0.0, 0.0));
                self.camera *= rotmat.submatrix().to_homogeneous();
            },
            Key::Right => {
                let rotmat = na::Rotation3::new(na::Vector3::new(0.0, -PI * 0.01, 0.0));
                self.camera *= rotmat.submatrix().to_homogeneous();
            },
            Key::Left => {
                let rotmat = na::Rotation3::new(na::Vector3::new(0.0, PI * 0.01, 0.0));
                self.camera *= rotmat.submatrix().to_homogeneous();
            },
            Key::W => {
                let transmat = translation3_mat(na::Vector3::new(0.0, 0.0, -1.0));
                self.camera *= transmat;
            },
            Key::S => {
                let transmat = translation3_mat(na::Vector3::new(0.0, 0.0, 1.0));
                self.camera *= transmat;
            },
            Key::D => {
                let transmat = translation3_mat(na::Vector3::new(-1.0, 0.0, 0.0));
                self.camera *= transmat;
            },
            Key::A => {
                let transmat = translation3_mat(na::Vector3::new(1.0, 0.0, 0.0));
                self.camera *= transmat;
            },
            Key::Q => {
                let transmat = translation3_mat(na::Vector3::new(0.0, -1.0, 0.0));
                self.camera *= transmat;
            },
            Key::E => {
                let transmat = translation3_mat(na::Vector3::new(0.0, 1.0, 0.0));
                self.camera *= transmat;
            },
            Key::I => {
                self.rot_x += PI * 0.01;
            },
            Key::K => {
                self.rot_x -= PI * 0.01;
            },
            Key::L => {
                self.rot_y += PI * 0.01;
            },
            Key::J => {
                self.rot_y -= PI * 0.01;
            },
            _ => {},
        }
    }
}

struct Arrow3 {
    tail: na::Point3<f64>,
    head: na::Point3<f64>,
}

impl Arrow3 {
    fn new(tx: f64, ty: f64, tz: f64, hx: f64, hy: f64, hz: f64) -> Arrow3 {
        Arrow3 {
            tail: na::Point3::new(tx, ty, tz),
            head: na::Point3::new(hx, hy, hz),
        }
    }

    fn project_to_viewport(&self, rot: na::Rotation3<f64>, persp: &na::PerspectiveMatrix3<f64>, camera: na::Matrix4<f64>) -> Arrow {
        use na::{ Rotate, ToHomogeneous, FromHomogeneous };
        // Apply cube's rotation:
        let mut headr = rot.rotate(&self.head);
        let mut tailr = rot.rotate(&self.tail);
        // Transform relative to the camera position:
        headr.z += 51.0 + 40.0;
        tailr.z += 51.0 + 40.0;
        let headr: na::Point3<f64> = <na::Point3<f64> as FromHomogeneous<na::Point4<f64>>>::from(&(camera * headr.to_homogeneous()));
        let tailr: na::Point3<f64> = <na::Point3<f64> as FromHomogeneous<na::Point4<f64>>>::from(&(camera * tailr.to_homogeneous()));
        // Project onto "device" surface:
        let head_prime = persp.project_point(&headr);
        let tail_prime = persp.project_point(&tailr);
        // Trasform to viewport surface:
        Arrow::new(
            na::Point2::new(
                tail_prime.x * 150.0 + WIDTHF_2,
                tail_prime.y * 150.0 + HEIGHTF_2,
            ),
            na::Point2::new(
                head_prime.x * 150.0 + WIDTHF_2,
                head_prime.y * 150.0 + HEIGHTF_2,
            )
        )
    }

    fn draw(&self, c: graphics::context::Context, gl: &mut GlGraphics, persp: &na::PerspectiveMatrix3<f64>, rot: na::Rotation3<f64>, camera: na::Matrix4<f64>) {
        let a2: Arrow = self.project_to_viewport(rot, persp, camera);
        a2.draw(c, gl);
    }
}

struct Arrow {
    tail: na::Point2<f64>,
    head: na::Point2<f64>,
}

impl Arrow {
    fn new(tail: na::Point2<f64>, head: na::Point2<f64>) -> Arrow {
        Arrow {
            tail: tail,
            head: head,
        }
    }

    fn draw(&self, c: graphics::context::Context, gl: &mut GlGraphics) {
        let path = [self.tail.x, self.tail.y, self.head.x, self.head.y];
        let line_style = graphics::Line::new([1.0, 1.0, 1.0, 1.0], 1.0);
        line_style.draw_arrow(path, 5.0, &c.draw_state, c.transform, gl);
    }
}
