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
use na::{ Norm, ToHomogeneous };

use std::f64::consts::PI;

mod arrow;
pub use arrow::Arrow3;

mod consts;
use consts::*;

mod field;
use field::*;

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
        field: FieldView {
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
        },
    };

    app.field.populate_field();

    if SHOW_GRID {
        app.field.populate_grid();
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
    field: FieldView,
}

impl App {
    fn update(&mut self, args: &UpdateArgs) {
    }

    fn render(&mut self, args: &RenderArgs) {
        self.field.render(&mut self.gl, args);
    }

    fn keypress(&mut self, key: Key) {
        match key {
            Key::Up => {
                // TODO: Define a mutating leftMultiply for Matrix4
                let rotmat = na::Rotation3::new(na::Vector3::new(PI * 0.01, 0.0, 0.0));
                self.field.camera = rotmat.submatrix().to_homogeneous() * self.field.camera;
            },
            Key::Down => {
                let rotmat = na::Rotation3::new(na::Vector3::new(-PI * 0.01, 0.0, 0.0));
                self.field.camera = rotmat.submatrix().to_homogeneous() * self.field.camera;
            },
            Key::Right => {
                let rotmat = na::Rotation3::new(na::Vector3::new(0.0, -PI * 0.01, 0.0));
                self.field.camera = rotmat.submatrix().to_homogeneous() * self.field.camera;
            },
            Key::Left => {
                let rotmat = na::Rotation3::new(na::Vector3::new(0.0, PI * 0.01, 0.0));
                self.field.camera = rotmat.submatrix().to_homogeneous() * self.field.camera;
            },
            Key::W => {
                let transmat = translation3_mat(na::Vector3::new(0.0, 0.0, -1.0));
                self.field.camera = transmat * self.field.camera;
            },
            Key::S => {
                let transmat = translation3_mat(na::Vector3::new(0.0, 0.0, 1.0));
                self.field.camera = transmat * self.field.camera;
            },
            Key::D => {
                let transmat = translation3_mat(na::Vector3::new(-1.0, 0.0, 0.0));
                self.field.camera = transmat * self.field.camera;
            },
            Key::A => {
                let transmat = translation3_mat(na::Vector3::new(1.0, 0.0, 0.0));
                self.field.camera = transmat * self.field.camera;
            },
            Key::Q => {
                let transmat = translation3_mat(na::Vector3::new(0.0, -1.0, 0.0));
                self.field.camera = transmat * self.field.camera;
            },
            Key::E => {
                let transmat = translation3_mat(na::Vector3::new(0.0, 1.0, 0.0));
                self.field.camera = transmat * self.field.camera;
            },
            Key::I => {
                let rot = na::Rotation3::new(na::Vector3::new(PI * 0.01, 0.0, 0.0));
                let rotmat = rot.submatrix().to_homogeneous();
                self.field.arrow_transforms = rotmat * self.field.arrow_transforms;
                self.field.map_transform(&rotmat);
            },
            Key::K => {
                let rot = na::Rotation3::new(na::Vector3::new(-PI * 0.01, 0.0, 0.0));
                let rotmat = rot.submatrix().to_homogeneous();
                self.field.arrow_transforms = rotmat * self.field.arrow_transforms;
                self.field.map_transform(&rotmat);
            },
            Key::L => {
                let rot = na::Rotation3::new(na::Vector3::new(0.0, PI * 0.01, 0.0));
                let rotmat = rot.submatrix().to_homogeneous();
                self.field.arrow_transforms = rotmat * self.field.arrow_transforms;
                self.field.map_transform(&rotmat);
            },
            Key::J => {
                let rot = na::Rotation3::new(na::Vector3::new(0.0, -PI * 0.01, 0.0));
                let rotmat = rot.submatrix().to_homogeneous();
                self.field.arrow_transforms = rotmat * self.field.arrow_transforms;
                self.field.map_transform(&rotmat);
            },
            Key::T => {
                self.field.charges[0].loc.y -= CHARGE_MVMT_STEP;
                self.field.populate_field();
                self.field.map_arrow_transforms();
            },
            Key::G => {
                self.field.charges[0].loc.y += CHARGE_MVMT_STEP;
                self.field.populate_field();
                self.field.map_arrow_transforms();
            },
            Key::H => {
                self.field.charges[0].loc.x += CHARGE_MVMT_STEP;
                self.field.populate_field();
                self.field.map_arrow_transforms();
            },
            Key::F => {
                self.field.charges[0].loc.x -= CHARGE_MVMT_STEP;
                self.field.populate_field();
                self.field.map_arrow_transforms();
            },
            Key::R => {
                self.field.charges[0].loc.z -= CHARGE_MVMT_STEP;
                self.field.populate_field();
                self.field.map_arrow_transforms();
            },
            Key::Y => {
                self.field.charges[0].loc.z += CHARGE_MVMT_STEP;
                self.field.populate_field();
                self.field.map_arrow_transforms();
            },
            _ => {},
        }
    }
}

struct FieldView {
    // The PointCharges whose field we are visualizing
    charges: Vec<PointCharge>,

    // The arrows describing the field strengths
    arrows: Vec<Arrow3>,

    // (Optionally, see SHOW_GRID) translucent line segments
    // drawing the grid on which the arrows reside
    grid_arrows: Vec<Arrow3>,

    // The product of all transformations applied to the arrows
    // of the FieldView (not to the camera). With this we can move
    // the location of a charge, rebuild the field, and then reapply
    // arrow_transforms to put the field where the user expects it
    arrow_transforms: na::Matrix4<f64>, // the product of transforms which have gotten the initial arrows to their current position

    // The transformation from absolute positions (as in `arrows`) to
    // positions relative to the camera's position and orientation
    camera: na::Matrix4<f64>, // camera transform from space to locations relative to camera

    // The bounds of the grid in which we are viewing the field
    x_range: (i64, i64),
    y_range: (i64, i64),
    z_range: (i64, i64),

    // For getting to 2-space
    persp: na::PerspectiveMatrix3<f64>,
}

impl VectorField3 for FieldView {
    fn field_data_at(&self, p: &na::Point3<f64>) -> FieldData {
        let mut field_data: FieldData = self.charges.iter()
            .map(|chg| chg.field_data_at(&p))
            .fold(num::Zero::zero(), |f0, f1| f0 + f1);
        field_data.update_norm();
        field_data
    }
}

impl FieldView {
    fn render(&self, gl: &mut GlGraphics, args: &RenderArgs) {
        graphics::clear(BG_CLR, gl);
        let persp = &self.persp;
        for arrow in &self.arrows {
            let cam = self.camera;
            gl.draw(args.viewport(), |c, gl| {
                arrow.draw(c, gl, persp, cam.clone());
            });
        }
        for arrow in &self.grid_arrows {
            let cam = self.camera;
            gl.draw(args.viewport(), |c, gl| {
                arrow.draw_no_head(c, gl, persp, cam.clone());
            });
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

        // Same for potential, for color or alpha
        let mut max_abs_potential: f64 = std::f64::NEG_INFINITY;

        let mut field: Vec<(na::Point3<f64>, FieldData)> = vec![];

        for i in lx..rx {
            for j in ly..ry {
                for k in lz..rz {
                    let loc = na::Point3::new(
                        i as f64 * GRID_S,
                        j as f64 * GRID_S,
                        k as f64 * GRID_S);
                    let field_data = self.field_data_at(&loc);
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
