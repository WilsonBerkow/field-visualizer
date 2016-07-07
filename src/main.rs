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

use na::{ Translate, Norm, ToHomogeneous, FromHomogeneous };

use std::f64::consts::PI;
use std::ops::Add;
use std::ops::AddAssign;

const WIDTH: u32 = 600;
const HEIGHT: u32 = 600;
const WIDTHF: f64 = WIDTH as f64;
const HEIGHTF: f64 = HEIGHT as f64;
const WIDTHF_2: f64 = WIDTHF * 0.5;
const HEIGHTF_2: f64 = HEIGHTF * 0.5;

const GRID_S: f64 = 15.0;
const GRID_S_2: f64 = GRID_S * 0.5;
const GRID_DIAG: f64 = GRID_S * 1.73205080757; // 1.7... is sqrt(3)

const CHARGE_MVMT_STEP: f64 = GRID_S;

const BG_CLR: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
const ARROW_CLR: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
const LINES_CLR: [f32; 4] = [0.0, 0.0, 0.7, 0.3];

const SHOW_GRID: bool = false;

const POTENTIAL_SHADING: bool = false;
const COLORFUL_POTENTIAL: bool = false;

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
        arrow_transforms: num::One::one(),
        camera: translation3_mat(na::Vector3::new(0.0, -GRID_S_2, 200.0)),
        persp: na::PerspectiveMatrix3::new(1.0, 200.0, NEAR_PLANE_Z, FAR_PLANE_Z),
        charges: vec![
            PointCharge::new(10.0, na::Point3::new(5.0 * GRID_S_2, GRID_S_2, GRID_S_2)),
            PointCharge::new(-10.0, na::Point3::new(-5.0 * GRID_S_2, GRID_S_2, GRID_S_2)),
        ],

        // Ranges in x,y,z in which we will draw the field vectors
        // These are expressed in terms on cubes in the grid, ie.,
        // in units of GRID_S voxels.
        x_range: (-4, 6),
        y_range: (-2, 4),
        z_range: (-2, 4),
    };

    app.populate_field();

    if SHOW_GRID {
        app.populate_grid();
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
    arrow_transforms: na::Matrix4<f64>, // the product of transforms which have gotten the initial arrows to their current position
    persp: na::PerspectiveMatrix3<f64>,
    camera: na::Matrix4<f64>, // camera transform from space to locations relative to camera
    charges: Vec<PointCharge>,
    x_range: (i64, i64),
    y_range: (i64, i64),
    z_range: (i64, i64),
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
                let rot = na::Rotation3::new(na::Vector3::new(PI * 0.01, 0.0, 0.0));
                let rotmat = rot.submatrix().to_homogeneous();
                self.arrow_transforms = rotmat * self.arrow_transforms;
                self.map_transform(&rotmat);
            },
            Key::K => {
                let rot = na::Rotation3::new(na::Vector3::new(-PI * 0.01, 0.0, 0.0));
                let rotmat = rot.submatrix().to_homogeneous();
                self.arrow_transforms = rotmat * self.arrow_transforms;
                self.map_transform(&rotmat);
            },
            Key::L => {
                let rot = na::Rotation3::new(na::Vector3::new(0.0, PI * 0.01, 0.0));
                let rotmat = rot.submatrix().to_homogeneous();
                self.arrow_transforms = rotmat * self.arrow_transforms;
                self.map_transform(&rotmat);
            },
            Key::J => {
                let rot = na::Rotation3::new(na::Vector3::new(0.0, -PI * 0.01, 0.0));
                let rotmat = rot.submatrix().to_homogeneous();
                self.arrow_transforms = rotmat * self.arrow_transforms;
                self.map_transform(&rotmat);
            },
            Key::T => {
                self.charges[0].loc.y -= CHARGE_MVMT_STEP;
                self.populate_field();
                self.map_arrow_transforms();
            },
            Key::G => {
                self.charges[0].loc.y += CHARGE_MVMT_STEP;
                self.populate_field();
                self.map_arrow_transforms();
            },
            Key::H => {
                self.charges[0].loc.x += CHARGE_MVMT_STEP;
                self.populate_field();
                self.map_arrow_transforms();
            },
            Key::F => {
                self.charges[0].loc.x -= CHARGE_MVMT_STEP;
                self.populate_field();
                self.map_arrow_transforms();
            },
            Key::R => {
                self.charges[0].loc.z -= CHARGE_MVMT_STEP;
                self.populate_field();
                self.map_arrow_transforms();
            },
            Key::Y => {
                self.charges[0].loc.z += CHARGE_MVMT_STEP;
                self.populate_field();
                self.map_arrow_transforms();
            },
            _ => {},
        }
    }

    fn populate_field(&mut self) {
        self.arrows = vec![];
        let (lx, rx) = self.x_range;
        let (ly, ry) = self.y_range;
        let (lz, rz) = self.z_range;

        // Keep track of stongest value of field so we can scale all
        // field vectors later and cap the length of the longest one
        let mut max_field: f64 = std::f64::NEG_INFINITY;

        // Same for potential, for alpha of arrows
        let mut max_abs_potential: f64 = std::f64::NEG_INFINITY;

        let mut field: Vec<(na::Point3<f64>, FieldData)> = vec![];

        for i in lx..rx {
            for j in ly..ry {
                for k in lz..rz {
                    let loc = na::Point3::new(
                        i as f64 * GRID_S,
                        j as f64 * GRID_S,
                        k as f64 * GRID_S);
                    let mut field_data: FieldData = self.charges.iter()
                        .map(|chg| chg.field_data_at(&loc))
                        .fold(num::Zero::zero(), |f0, f1| f0 + f1);
                    field_data.update_norm();
                    max_field = f64_max(max_field, field_data.force_mag);
                    max_abs_potential = f64_max(max_abs_potential, field_data.potential.abs());

                    field.push((loc, field_data));
                }
            }
        }

        for (loc, field_data) in field {
            let rel_mag = field_data.force_mag / max_field;
            let rel_pot = (1.0 + field_data.potential / max_abs_potential) / 2.0;

            let length = FIELD_VEC_MIN_LEN + rel_mag * FIELD_VEC_LEN_RANGE;
            let adjusted_pot = (1.0 - 0.7 * (1.0 - rel_pot)) as f32; // So none are at 0.0
            let clr = if POTENTIAL_SHADING {
                // clr changes based on potential
                if COLORFUL_POTENTIAL {
                    // Use a scale from red to blue
                    if adjusted_pot > 0.0 {
                        [adjusted_pot, 0.0, 1.0 - adjusted_pot, 1.0]
                    } else {
                        [1.0 + adjusted_pot, 0.0, -adjusted_pot, 1.0]
                    }
                } else {
                    [0.0, 0.0, 0.0, adjusted_pot]
                }
            } else {
                // clr changes based on field magnitude
                [0.0, 0.0, 0.0, (rel_mag * 2.2) as f32]
            };

            // loc is the center of the arrow stem
            let arrow_vec = length * field_data.force_vec.normalize();
            let tail = loc - arrow_vec * 0.5;
            let head = loc + arrow_vec * 0.5;
            self.arrows.push(Arrow3::from_to_clr(tail, head, clr));
        }
        // // Scale each vector to have reasonable lengths and to be
        // // centered around the point whose field they are measuring
        // for arrow in self.arrows.iter_mut() {
        //     let len = arrow.len();
        //     let strength_lvl = len / max_field;
        //     arrow.clr = [0.0, 0.0, 0.0, 1.0 - 0.9 * (1.0 - strength_lvl.sqrt()) as f32];
        //     arrow.set_len(FIELD_VEC_MIN_LEN + strength_lvl * FIELD_VEC_LEN_RANGE);
        //     let c = arrow.tail;
        //     arrow.center_at(c);
        // }
    }

    fn populate_grid(&mut self) {
        let (lx, rx) = self.x_range;
        let (ly, ry) = self.y_range;
        let (lz, rz) = self.z_range;
        for i in lx..rx {
            for j in ly..ry {
                self.grid_arrows.push(
                    Arrow3::from_to_clr(
                        na::Point3::new(i as f64 * GRID_S, j as f64 * GRID_S, (lz - 1) as f64 * GRID_S),
                        na::Point3::new(i as f64 * GRID_S, j as f64 * GRID_S, (rz - 1) as f64 * GRID_S),
                        LINES_CLR
                        )
                    );
                self.grid_arrows.push(
                    Arrow3::from_to_clr(
                        na::Point3::new((lx - 1) as f64 * GRID_S, i as f64 * GRID_S, j as f64 * GRID_S),
                        na::Point3::new((rx - 1) as f64 * GRID_S, i as f64 * GRID_S, j as f64 * GRID_S),
                        LINES_CLR
                        )
                    );
                self.grid_arrows.push(
                    Arrow3::from_to_clr(
                        na::Point3::new(i as f64 * GRID_S, (ly - 1) as f64 * GRID_S, j as f64 * GRID_S),
                        na::Point3::new(i as f64 * GRID_S, (ry - 1) as f64 * GRID_S, j as f64 * GRID_S),
                        LINES_CLR
                        )
                    );
            }
        }
    }

    fn map_transform(&mut self, t: &na::Matrix4<f64>) {
        for arrow in self.arrows.iter_mut() {
            arrow.map_transform(t);
        }
        for arrow in self.grid_arrows.iter_mut() {
            arrow.map_transform(t);
        }
    }

    fn map_arrow_transforms(&mut self) {
        for arrow in self.arrows.iter_mut() {
            arrow.map_transform(&self.arrow_transforms);
        }
        for arrow in self.grid_arrows.iter_mut() {
            arrow.map_transform(&self.arrow_transforms);
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
    fn potential_at(&self, p: &na::Point3<f64>) -> f64;
    fn field_data_at(&self, p: &na::Point3<f64>) -> FieldData;
}

struct FieldData {
    force_vec: na::Vector3<f64>, // direction and unscaled strength of field
    force_mag: f64, // cached norm of force_vec, updated manually
    potential: f64, // potential before scaling relative to field as a whole
}

impl FieldData {
    fn new(force_vec: na::Vector3<f64>, mag: f64, pot: f64) -> FieldData {
        FieldData {
            force_vec: force_vec,
            force_mag: mag,
            potential: pot,
        }
    }

    fn update_norm(&mut self) {
        self.force_mag = self.force_vec.norm();
    }
}

impl num::Zero for FieldData {
    fn zero() -> FieldData { FieldData::new(num::Zero::zero(), 0.0, 0.0) }
    fn is_zero(&self) -> bool {
        self.force_vec.is_zero()
            && self.potential.is_zero()
    }

}

impl Add<FieldData> for FieldData {
    type Output = FieldData;
    fn add(self, right: FieldData) -> FieldData {
        FieldData::new(
            self.force_vec + right.force_vec,
            self.force_mag, // not updated
            self.potential + right.potential
        )
    }
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

impl Translate<PointCharge> for na::Vector3<f64> {
    fn translate(&self, chg: &PointCharge) -> PointCharge {
        PointCharge::new(chg.charge, self.translate(&chg.loc))
    }
    fn inverse_translate(&self, chg: &PointCharge) -> PointCharge {
        PointCharge::new(chg.charge, self.inverse_translate(&chg.loc))
    }
}

impl Add<na::Vector3<f64>> for PointCharge {
    type Output = PointCharge;
    fn add(self, rhs: na::Vector3<f64>) -> PointCharge {
        rhs.translate(&self)
    }
}

impl<'a> Add<na::Vector3<f64>> for &'a PointCharge {
    type Output = PointCharge;
    fn add(self, rhs: na::Vector3<f64>) -> PointCharge {
        rhs.translate(self)
    }
}

impl<'a> Add<&'a na::Vector3<f64>> for PointCharge {
    type Output = PointCharge;
    fn add(self, rhs: &na::Vector3<f64>) -> PointCharge {
        rhs.translate(&self)
    }
}

impl<'a, 'b> Add<&'a na::Vector3<f64>> for &'b PointCharge {
    type Output = PointCharge;
    fn add(self, rhs: &na::Vector3<f64>) -> PointCharge {
        rhs.translate(self)
    }
}

impl<'a> AddAssign<na::Vector3<f64>> for PointCharge {
    fn add_assign(&mut self, rhs: na::Vector3<f64>) {
        self.loc += rhs;
    }
}

impl<'a, 'b> AddAssign<&'a na::Vector3<f64>> for PointCharge {
    fn add_assign(&mut self, rhs: &na::Vector3<f64>) {
        // nalgebra does not implement AddAssign<&Vector3> for Point3,
        // thus this is done field by field
        self.loc.x = self.loc.x + rhs.x;
        self.loc.y = self.loc.y + rhs.y;
        self.loc.z = self.loc.z + rhs.z;
    }
}


const FIELD_SCALE_FACTOR: f64 = 10000.0;
impl VectorField3 for PointCharge {
    fn force_at(&self, p: &na::Point3<f64>) -> na::Vector3<f64> {
        let unit_vec = (p.clone() - self.loc).normalize();
        let magnitude = self.charge / na::distance_squared(&self.loc, p);
        FIELD_SCALE_FACTOR * magnitude * unit_vec
    }
    fn potential_at(&self, p: &na::Point3<f64>) -> f64 {
        FIELD_SCALE_FACTOR * self.charge / na::distance(&self.loc, p)
    }
    fn field_data_at(&self, p: &na::Point3<f64>) -> FieldData {
        let unit_vec = (p.clone() - self.loc).normalize();
        let d_squared = na::distance_squared(&self.loc, p);
        let force_mag = FIELD_SCALE_FACTOR * self.charge / d_squared;
        let potential = FIELD_SCALE_FACTOR * self.charge / d_squared.sqrt();
        FieldData {
            force_vec: unit_vec * force_mag,
            force_mag: force_mag,
            potential: potential,
        }
    }
}

struct Arrow3 {
    tail: na::Point3<f64>,
    head: na::Point3<f64>,
    clr: [f32; 4],
}

impl Arrow3 {
    fn from_to_clr(tail: na::Point3<f64>, head: na::Point3<f64>, clr: [f32; 4]) -> Arrow3 {
        Arrow3 {
            tail: tail,
            head: head,
            clr: clr,
        }
    }

    fn from_to(tail: na::Point3<f64>, head: na::Point3<f64>) -> Arrow3 {
        Arrow3 {
            tail: tail,
            head: head,
            clr: ARROW_CLR,
        }
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
            Some(Arrow::from_to_clr(
                na::Point2::new(
                    tail_prime.x * 150.0 + WIDTHF_2,
                    tail_prime.y * 150.0 + HEIGHTF_2,
                ),
                na::Point2::new(
                    head_prime.x * 150.0 + WIDTHF_2,
                    head_prime.y * 150.0 + HEIGHTF_2,
                ),
                self.clr,
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
}

struct Arrow {
    tail: na::Point2<f64>,
    head: na::Point2<f64>,
    clr: [f32; 4],
}

impl Arrow {
    fn from_to_clr(tail: na::Point2<f64>, head: na::Point2<f64>, clr: [f32; 4]) -> Arrow {
        Arrow {
            tail: tail,
            head: head,
            clr: clr,
        }
    }

    fn from_to(tail: na::Point2<f64>, head: na::Point2<f64>) -> Arrow {
        Arrow {
            tail: tail,
            head: head,
            clr: ARROW_CLR,
        }
    }

    fn draw(&self, c: graphics::context::Context, gl: &mut GlGraphics) {
        let path = [self.tail.x, self.tail.y, self.head.x, self.head.y];
        let line_style = graphics::Line::new(self.clr, 1.0);
        line_style.draw_arrow(path, 5.0, &c.draw_state, c.transform, gl);
    }

    fn draw_no_head(&self, c: graphics::context::Context, gl: &mut GlGraphics) {
        let path = [self.tail.x, self.tail.y, self.head.x, self.head.y];
        let line_style = graphics::Line::new(self.clr, 1.0);
        line_style.draw(path, &c.draw_state, c.transform, gl);
    }
}
