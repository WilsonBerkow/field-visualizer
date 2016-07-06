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

use na::{ Translate, Norm, ToHomogeneous, FromHomogeneous };

const WIDTH: u32 = 600;
const HEIGHT: u32 = 600;
const WIDTHF: f64 = WIDTH as f64;
const HEIGHTF: f64 = HEIGHT as f64;
const WIDTHF_2: f64 = WIDTHF * 0.5;
const HEIGHTF_2: f64 = HEIGHTF * 0.5;

const GRID_S: f64 = 30.0;
const GRID_S_2: f64 = GRID_S * 0.5;
const GRID_DIAG: f64 = GRID_S * 1.73205080757;//std::f64::consts::SQRT_3;

const BG_CLR: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
const ARROW_CLR: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
const LINES_CLR: [f32; 4] = [0.0, 0.0, 0.7, 0.3];

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
        arrows: vec![],
        grid_arrows: vec![],
        camera: translation3_mat(na::Vector3::new(0.0, 0.0, 91.0)),
        persp: na::PerspectiveMatrix3::new(1.0, 200.0, 0.0, 100.0),
        chg: PointCharge::new(10.0, na::Point3::new(3.0 * GRID_S_2, GRID_S_2, GRID_S_2)),
        chg1: PointCharge::new(-10.0, na::Point3::new(-3.0 * GRID_S_2, GRID_S_2, GRID_S_2)),
    };
    {
        let mut max_force: f64 = std::f64::NEG_INFINITY;
        let l = -2;
        let r = 4;
        for i in l..r {
            for j in l..r {
                for k in l..r {
                    let loc = na::Point3::new(
                        (i as f64 - 0.0) * GRID_S,
                        (j as f64 - 0.0) * GRID_S,
                        (k as f64 - 0.0) * GRID_S);
                    let force = app.chg.force_at(&loc) + app.chg1.force_at(&loc);
                    let mut arrow = arrow_from_force(&loc, &force);
                    let norm = force.norm();
                    max_force = f64_max(max_force, norm);
                    app.arrows.push(arrow);
                }
            }
        }
        for arrow in app.arrows.iter_mut() {
            let len = arrow.len();
            arrow.set_len(len / max_force * (GRID_DIAG * 0.6) + GRID_DIAG * 0.2);
            let c = arrow.tail;
            arrow.center_at(c);
        }
        for i in l..r {
            for j in l..r {
                app.grid_arrows.push(
                    Arrow3::new(
                        na::Point3::new(i as f64 * GRID_S, j as f64 * GRID_S, (l - 1) as f64 * GRID_S),
                        na::Point3::new(i as f64 * GRID_S, j as f64 * GRID_S, (r - 1) as f64 * GRID_S)
                    )
                );
                app.grid_arrows.push(
                    Arrow3::new(
                        na::Point3::new((l - 1) as f64 * GRID_S, i as f64 * GRID_S, j as f64 * GRID_S),
                        na::Point3::new((r - 1) as f64 * GRID_S, i as f64 * GRID_S, j as f64 * GRID_S)
                    )
                );
                app.grid_arrows.push(
                    Arrow3::new(
                        na::Point3::new(i as f64 * GRID_S, (l - 1) as f64 * GRID_S, j as f64 * GRID_S),
                        na::Point3::new(i as f64 * GRID_S, (r - 1) as f64 * GRID_S, j as f64 * GRID_S)
                    )
                );
            }
        }
    }

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
    grid_arrows: Vec<Arrow3>,
    persp: na::PerspectiveMatrix3<f64>,
    camera: na::Matrix4<f64>, // camera transform from space to locations relative to camera
    chg: PointCharge,
    chg1: PointCharge,
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
        for arrow in &self.grid_arrows {
            let cam = self.camera;
            self.gl.draw(args.viewport(), |c, gl| {
                arrow.draw_no_head(c, gl, persp, cam.clone());
            });
        }
    }

    fn keypress(&mut self, key: Key) {
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
                for arrow in self.grid_arrows.iter_mut() {
                    arrow.map_transform(&rotmat.submatrix().to_homogeneous());
                }
            },
            Key::K => {
                let rotmat = na::Rotation3::new(na::Vector3::new(-PI * 0.01, 0.0, 0.0));
                for arrow in self.arrows.iter_mut() {
                    arrow.map_transform(&rotmat.submatrix().to_homogeneous());
                }
                for arrow in self.grid_arrows.iter_mut() {
                    arrow.map_transform(&rotmat.submatrix().to_homogeneous());
                }
            },
            Key::L => {
                let rotmat = na::Rotation3::new(na::Vector3::new(0.0, PI * 0.01, 0.0));
                for arrow in self.arrows.iter_mut() {
                    arrow.map_transform(&rotmat.submatrix().to_homogeneous());
                }
                for arrow in self.grid_arrows.iter_mut() {
                    arrow.map_transform(&rotmat.submatrix().to_homogeneous());
                }
            },
            Key::J => {
                let rotmat = na::Rotation3::new(na::Vector3::new(0.0, -PI * 0.01, 0.0));
                for arrow in self.arrows.iter_mut() {
                    arrow.map_transform(&rotmat.submatrix().to_homogeneous());
                }
                for arrow in self.grid_arrows.iter_mut() {
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
    <na::Point3<f64> as FromHomogeneous<na::Point4<f64>>>::from(&(ref_mat4_mul(mat, pt.to_homogeneous())))
}

trait VectorField3 {
    fn force_at(&self, p: &na::Point3<f64>) -> na::Vector3<f64>;
    fn arrow_at(&self, p: &na::Point3<f64>) -> Arrow3;
}

struct PointCharge {
    charge: f64,
    loc: na::Point3<f64>,
}

impl PointCharge {
    fn new(charge: f64, loc: na::Point3<f64>) -> PointCharge {
        PointCharge { charge: charge, loc: loc }
    }
}

fn f64_max(x: f64, y: f64) -> f64 {
    if x > y { x } else { y }
}

fn pull_inward(p: &na::Point3<f64>, v: &na::Vector3<f64>) -> na::Point3<f64> {
    na::Point3::new(
        f64_max(0.0, p.x.abs() - v.x) * p.x.signum(),
        f64_max(0.0, p.y.abs() - v.y) * p.y.signum(),
        f64_max(0.0, p.z.abs() - v.z) * p.z.signum()
    )
}

fn arrow_from_force(p: &na::Point3<f64>, f: &na::Vector3<f64>) -> Arrow3 {
    let tail = p.clone();
    let head = f.translate(&tail);
    println!("Force: {}", f);
    Arrow3 { tail: tail, head: head }
}

impl VectorField3 for PointCharge {
    fn force_at(&self, p: &na::Point3<f64>) -> na::Vector3<f64> {
        let unit_vec = (p.clone() - self.loc).normalize(); // ownership error likely
        let magnitude = self.charge / na::distance_squared(&self.loc, p);
        10000.0 * magnitude * unit_vec
    }

    fn arrow_at(&self, p: &na::Point3<f64>) -> Arrow3 {
        //let to_cube_center = na::Vector3::new(GRID_S_2, GRID_S_2, GRID_S_2);
        //let f = self.force_at(&pull_inward(&p, &to_cube_center));
        let f = self.force_at(&p);
        //let tail = pull_inward(&p, &(2.0 * to_cube_center));
        let tail = p.clone();
        let head = f.translate(&tail);
        println!("Force: {}", f);
        Arrow3 { tail: tail, head: head }
    }
}

struct Arrow3 {
    tail: na::Point3<f64>,
    head: na::Point3<f64>,
}

impl Arrow3 {
    fn new(tail: na::Point3<f64>, head: na::Point3<f64>) -> Arrow3 {
        Arrow3 {
            tail: tail,
            head: head,
        }
    }

    fn scale_len(&mut self, s: f64) {
        self.head = ((self.head - self.tail) * s).translate(&self.tail);
    }

    fn set_len(&mut self, s: f64) {
        self.head = ((self.head - self.tail).normalize() * s).translate(&self.tail);
    }

    fn len(&self) -> f64 {
        (self.head - self.tail).norm()
    }

    fn map_transform(&mut self, mat: &na::Matrix4<f64>) {
        self.tail = transform_in_homo(self.tail, mat);
        self.head = transform_in_homo(self.head, mat);
    }

    fn project_to_viewport(&self, persp: &na::PerspectiveMatrix3<f64>, camera: na::Matrix4<f64>) -> Option<Arrow> {
        // Transform relative to the camera position:
        let headr: na::Point3<f64> = transform_in_homo(self.head, &camera);
        let tailr: na::Point3<f64> = transform_in_homo(self.tail, &camera);
        if headr.z <= 1.0 && tailr.z <= 1.0 {
            None
        } else {
            // Project onto "device" surface:
            let head_prime = persp.project_point(&headr);
            let tail_prime = persp.project_point(&tailr);
            // Trasform to viewport surface:
            Some(Arrow::new(
                na::Point2::new(
                    tail_prime.x * 150.0 + WIDTHF_2,
                    tail_prime.y * 150.0 + HEIGHTF_2,
                ),
                na::Point2::new(
                    head_prime.x * 150.0 + WIDTHF_2,
                    head_prime.y * 150.0 + HEIGHTF_2,
                )
            ))
        }
    }

    fn draw(&self, c: graphics::context::Context, gl: &mut GlGraphics, persp: &na::PerspectiveMatrix3<f64>, camera: na::Matrix4<f64>) {
        if let Some(a2d) = self.project_to_viewport(persp, camera) {
            a2d.draw(c, gl);
        }
    }

    fn draw_no_head(&self, c: graphics::context::Context, gl: &mut GlGraphics, persp: &na::PerspectiveMatrix3<f64>, camera: na::Matrix4<f64>) {
        if let Some(a2d) = self.project_to_viewport(persp, camera) {
            a2d.draw_no_head(c, gl);
        }
    }

    fn center_at(&mut self, c: na::Point3<f64>) {
        let diff = (self.head - self.tail) * 0.5;
        self.head = c + diff;
        self.tail = c - diff;
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
        let line_style = graphics::Line::new(ARROW_CLR, 1.0);
        line_style.draw_arrow(path, 5.0, &c.draw_state, c.transform, gl);
    }

    fn draw_no_head(&self, c: graphics::context::Context, gl: &mut GlGraphics) {
        let path = [self.tail.x, self.tail.y, self.head.x, self.head.y];
        let line_style = graphics::Line::new(LINES_CLR, 1.0);
        line_style.draw(path, &c.draw_state, c.transform, gl);
    }
}
