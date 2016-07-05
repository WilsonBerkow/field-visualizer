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
use std::ops::Mul;

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
        camera: translation3_mat(na::Vector3::new(0.0, 0.0, 91.0)),
        persp: na::PerspectiveMatrix3::new(1.0, 200.0, 0.0, 100.0),
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

fn translation3_mat<T: na::BaseNum>(v: na::Vector3<T>) -> na::Matrix4<T> {
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
}

impl App {
    fn update(&mut self, args: &UpdateArgs) {
    }

    fn render(&mut self, args: &RenderArgs) {
        graphics::clear(BG_CLR, &mut self.gl);
        let persp = &self.persp;
        for arrow in &self.arrows {
            let cam = self.camera;
            self.gl.draw(args.viewport(), |c, gl| {
                arrow.draw(c, gl, persp, cam.clone());
            });
        }
    }

    fn keypress(&mut self, key: Key) {
        use na::ToHomogeneous;
        match key {
            Key::Up => {
                let rotmat = na::Rotation3::new(na::Vector3::new(PI * 0.01, 0.0, 0.0));
                self.camera = rotmat.submatrix().to_homogeneous() * self.camera;
            },
            Key::Down => {
                let rotmat = na::Rotation3::new(na::Vector3::new(-PI * 0.01, 0.0, 0.0));
                self.camera = rotmat.submatrix().to_homogeneous() * self.camera;
            },
            Key::Right => {
                let rotmat = na::Rotation3::new(na::Vector3::new(0.0, -PI * 0.01, 0.0));
                self.camera = rotmat.submatrix().to_homogeneous() * self.camera;
            },
            Key::Left => {
                let rotmat = na::Rotation3::new(na::Vector3::new(0.0, PI * 0.01, 0.0));
                self.camera = rotmat.submatrix().to_homogeneous() * self.camera;
            },
            Key::W => {
                let transmat = translation3_mat(na::Vector3::new(0.0, 0.0, -1.0));
                self.camera = transmat * self.camera;
            },
            Key::S => {
                let transmat = translation3_mat(na::Vector3::new(0.0, 0.0, 1.0));
                self.camera = transmat * self.camera;
            },
            Key::D => {
                let transmat = translation3_mat(na::Vector3::new(-1.0, 0.0, 0.0));
                self.camera = transmat * self.camera;
            },
            Key::A => {
                let transmat = translation3_mat(na::Vector3::new(1.0, 0.0, 0.0));
                self.camera = transmat * self.camera;
            },
            Key::Q => {
                let transmat = translation3_mat(na::Vector3::new(0.0, -1.0, 0.0));
                self.camera = transmat * self.camera;
            },
            Key::E => {
                let transmat = translation3_mat(na::Vector3::new(0.0, 1.0, 0.0));
                self.camera = transmat * self.camera;
            },
            Key::I => {
                let rotmat = na::Rotation3::new(na::Vector3::new(PI * 0.01, 0.0, 0.0));
                for arrow in self.arrows.iter_mut() {
                    arrow.map_transform(&rotmat.submatrix().to_homogeneous());
                }
            },
            Key::K => {
                let rotmat = na::Rotation3::new(na::Vector3::new(-PI * 0.01, 0.0, 0.0));
                for arrow in self.arrows.iter_mut() {
                    arrow.map_transform(&rotmat.submatrix().to_homogeneous());
                }
            },
            Key::L => {
                let rotmat = na::Rotation3::new(na::Vector3::new(0.0, PI * 0.01, 0.0));
                for arrow in self.arrows.iter_mut() {
                    arrow.map_transform(&rotmat.submatrix().to_homogeneous());
                }
            },
            Key::J => {
                let rotmat = na::Rotation3::new(na::Vector3::new(0.0, -PI * 0.01, 0.0));
                for arrow in self.arrows.iter_mut() {
                    arrow.map_transform(&rotmat.submatrix().to_homogeneous());
                }
            },
            _ => {},
        }
    }
}

// The following should be in nalgebra, which implements
// Mul<Point4<N>> for Matrix4<N> but not also for &'a Matrix<N>.
// The definition mirrors nalgebra's definition of the method
// `mul` in `impl... for Matrix4<N>`.
#[inline]
fn ref_mat4_mul(mat: &na::Matrix4<f64>, right: na::Point4<f64>) -> na::Point4<f64> {
    let mut res: na::Point4<f64> = na::Point4::new(0.0, 0.0, 0.0, 0.0);
    for i in 0..4 {
        for j in 0..4 {
            unsafe {
                let val = res.at_fast(i) + mat.at_fast((i, j)) * right.at_fast(j);
                res.set_fast(i, val);
            }
        }
    }
    res
}

fn transform_in_homo(pt: na::Point3<f64>, mat: &na::Matrix4<f64>) -> na::Point3<f64> {
    use na::{ ToHomogeneous, FromHomogeneous };
    <na::Point3<f64> as FromHomogeneous<na::Point4<f64>>>::from(&(ref_mat4_mul(mat, pt.to_homogeneous())))
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

    fn map_transform(&mut self, mat: &na::Matrix4<f64>) {
        self.tail = transform_in_homo(self.tail, mat);
        self.head = transform_in_homo(self.head, mat);
    }

    fn project_to_viewport(&self, persp: &na::PerspectiveMatrix3<f64>, camera: na::Matrix4<f64>) -> Arrow {
        use na::{ ToHomogeneous, FromHomogeneous };
        // Transform relative to the camera position:
        let headr: na::Point3<f64> = transform_in_homo(self.head, &camera);
        let tailr: na::Point3<f64> = transform_in_homo(self.tail, &camera);
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

    fn draw(&self, c: graphics::context::Context, gl: &mut GlGraphics, persp: &na::PerspectiveMatrix3<f64>, camera: na::Matrix4<f64>) {
        let a2: Arrow = self.project_to_viewport(persp, camera);
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
