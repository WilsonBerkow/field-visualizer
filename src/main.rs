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

const SHOW_GRID: bool = false;

const NEAR_PLANE_Z: f64 = 1.0;
const FAR_PLANE_Z: f64 = 100.0;

// Maximum and minimum lengths of a field vector:
const FIELD_VEC_MAX_LEN: f64 = GRID_DIAG * 0.8;
const FIELD_VEC_MIN_LEN: f64 = GRID_DIAG * 0.1;

const FIELD_VEC_LEN_RANGE: f64 = FIELD_VEC_MAX_LEN - FIELD_VEC_MIN_LEN;

fn f64_max(x: f64, y: f64) -> f64 {
    if x > y { x } else { y }
}

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
        camera: translation3_mat(na::Vector3::new(0.0, -GRID_S_2, 200.0)),
        persp: na::PerspectiveMatrix3::new(1.0, 200.0, NEAR_PLANE_Z, FAR_PLANE_Z),
        charges: vec![
            PointCharge::new(10.0, na::Point3::new(5.0 * GRID_S_2, GRID_S_2, GRID_S_2)),
            PointCharge::new(-10.0, na::Point3::new(-5.0 * GRID_S_2, GRID_S_2, GRID_S_2)),
        ],
    };

    // Ranges in x,y,z in which we will draw the field vectors
    // These are expressed in terms on cubes in the grid, ie.,
    // in units of GRID_S voxels.
    let x_range: (i64, i64) = (-4, 6);
    let y_range: (i64, i64) = (-2, 4);
    let z_range: (i64, i64) = (-2, 4);

    app.populate_field(x_range, y_range, z_range);

    if SHOW_GRID {
        app.populate_grid(x_range, y_range, z_range);
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
    charges: Vec<PointCharge>,
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

    fn populate_field(&mut self, (lx, rx): (i64, i64), (ly, ry): (i64, i64), (lz, rz): (i64, i64)) {
        // Keep track of stongest value of field so we can scale all
        // field vectors later and cap the length of the longest one
        let mut max_field: f64 = std::f64::NEG_INFINITY;

        for i in lx..rx {
            for j in ly..ry {
                for k in lz..rz {
                    let loc = na::Point3::new(
                        i as f64 * GRID_S,
                        j as f64 * GRID_S,
                        k as f64 * GRID_S);
                    let net_field: na::Vector3<f64> = self.charges.iter()
                        .map(|chg| chg.force_at(&loc))
                        .fold(num::Zero::zero(), |a, b| a + b);
                    max_field = f64_max(max_field, net_field.norm());

                    // Create and push the arrow of length net_field and
                    // tail at loc.
                    self.arrows.push(arrow_from_force(&loc, &net_field));
                    // In the loop below, the arrows will be scaled and
                    // repositioned for better appearence.
                }
            }
        }

        // Scale each vector to have reasonable lengths and to be
        // centered around the point whose field they are measuring
        for arrow in self.arrows.iter_mut() {
            let len = arrow.len();
            arrow.set_len(FIELD_VEC_MIN_LEN + len / max_field * FIELD_VEC_LEN_RANGE);
            let c = arrow.tail;
            arrow.center_at(c);
        }
    }

    fn populate_grid(&mut self, (lx, rx): (i64, i64), (ly, ry): (i64, i64), (lz, rz): (i64, i64)) {
        for i in lx..rx {
            for j in ly..ry {
                self.grid_arrows.push(
                    Arrow3::new(
                        na::Point3::new(i as f64 * GRID_S, j as f64 * GRID_S, (lz - 1) as f64 * GRID_S),
                        na::Point3::new(i as f64 * GRID_S, j as f64 * GRID_S, (rz - 1) as f64 * GRID_S)
                        )
                    );
                self.grid_arrows.push(
                    Arrow3::new(
                        na::Point3::new((lx - 1) as f64 * GRID_S, i as f64 * GRID_S, j as f64 * GRID_S),
                        na::Point3::new((rx - 1) as f64 * GRID_S, i as f64 * GRID_S, j as f64 * GRID_S)
                        )
                    );
                self.grid_arrows.push(
                    Arrow3::new(
                        na::Point3::new(i as f64 * GRID_S, (ly - 1) as f64 * GRID_S, j as f64 * GRID_S),
                        na::Point3::new(i as f64 * GRID_S, (ry - 1) as f64 * GRID_S, j as f64 * GRID_S)
                        )
                    );
            }
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
        let mut v = self.head - self.tail;
        let len = v.norm();
        if len > 1.0 {
            v = v * (s / len);
        }
        self.head = v.translate(&self.tail);
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
        if headr.z <= NEAR_PLANE_Z || tailr.z <= NEAR_PLANE_Z {
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
