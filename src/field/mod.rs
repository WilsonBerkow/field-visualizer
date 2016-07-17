use std;
use num::{Zero, One};

use na::{Point3, Vector3, Matrix4, PerspectiveMatrix3, Norm};

use pw;

use arrow::Arrow3;

use util;
use consts::*;

mod field_data;
mod point_charge;

pub use self::field_data::{ FieldData, VectorField3 };
pub use self::point_charge::PointCharge;

pub struct FieldView {
    // The PointCharges whose field we are visualizing
    pub charges: Vec<PointCharge>,

    // The arrows describing the field strengths
    arrows: Vec<Arrow3>,

    // The product of all transformations applied to the arrows
    // of the FieldView (not to the camera). With this we can move
    // the location of a charge, rebuild the field, and then reapply
    // arrow_transforms to put the field where the user expects it
    arrow_transforms: Matrix4<f64>,

    // The transformation from absolute positions (as in `arrows`) to
    // positions relative to the camera's position and orientation
    pub camera: Matrix4<f64>,

    // The bounds of the grid in which we are viewing the field
    pub x_range: (i64, i64),
    pub y_range: (i64, i64),
    pub z_range: (i64, i64),

    // For getting to 2-space
    persp: PerspectiveMatrix3<f64>,
}

impl VectorField3 for FieldView {
    fn field_data_at(&self, p: &Point3<f64>) -> FieldData {
        let mut field_data: FieldData = self.charges.iter()
            .map(|chg| chg.field_data_at(&p))
            .fold(Zero::zero(), |f0, f1| f0 + f1);
        field_data.update_norm();
        field_data
    }
}

impl FieldView {
    pub fn new(camera_dist: f64, charges: Vec<PointCharge>) -> FieldView {
        FieldView {
            arrows: vec![],
            arrow_transforms: One::one(),
            camera: util::translation_mat4(Vector3::new(0.0, -GRID_S_2, camera_dist)),
            persp: PerspectiveMatrix3::new(1.0, 200.0, NEAR_PLANE_Z, FAR_PLANE_Z),
            charges: charges,

            // Ranges in x,y,z in which we will draw the field vectors
            // These are expressed in terms on cubes in the grid, ie.,
            // in units of GRID_S voxels.
            x_range: (-4, 6),
            y_range: (-2, 4),
            z_range: (-2, 4),
        }
    }

    pub fn render(&self, c: pw::Context, gl: &mut pw::G2d, view: [f64; 4]) {
        pw::Rectangle::new(pw::color::WHITE).draw(view, &c.draw_state, c.transform, gl);
        let persp = &self.persp;
        for arrow in &self.arrows {
            let cam = self.camera;
            arrow.draw(c, gl, persp, cam.clone(), view);
        }
    }

    pub fn populate_field(&mut self) {
        self.arrows = vec![];
        let (lx, rx) = self.x_range;
        let (ly, ry) = self.y_range;
        let (lz, rz) = self.z_range;

        // Keep track of stongest value of field so we can scale all
        // field vectors later and cap the length of the longest one
        let mut max_field: f64 = std::f64::NEG_INFINITY;

        // Same for potential, for color or alpha
        let mut max_abs_potential: f64 = std::f64::NEG_INFINITY;

        let mut field: Vec<(Point3<f64>, FieldData)> = vec![];

        // Get data at all points in field
        for i in lx..rx {
            for j in ly..ry {
                for k in lz..rz {
                    let loc = Point3::new(
                        i as f64 * GRID_S,
                        j as f64 * GRID_S,
                        k as f64 * GRID_S);
                    let field_data = self.field_data_at(&loc);
                    max_field = util::f64_max(max_field, field_data.force_mag);
                    max_abs_potential = util::f64_max(max_abs_potential, field_data.potential.abs());

                    field.push((loc, field_data));
                }
            }
        }

        // Generate arrows based on those data
        for (loc, field_data) in field {
            let rel_mag = field_data.force_mag / max_field;
            let rel_pot = (1.0 + field_data.potential / max_abs_potential) / 2.0;

            let length = FIELD_VEC_MIN_LEN + rel_mag * FIELD_VEC_LEN_RANGE;
            let adjusted_pot = (1.0 - 0.7 * (1.0 - rel_pot)) as f32; // So none are at 0.0 (nor below 0.3)
            // Based on configured constants, make clr
            let clr = if POTENTIAL_SHADING {
                // `clr` changes based on potential
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
                // `clr` changes based on field magnitude
                [0.0, 0.0, 0.0, (rel_mag * 2.2) as f32]
            };

            // loc is the center of the arrow stem
            let arrow_vec = length * field_data.force_vec.normalize();
            let tail = loc - arrow_vec * 0.5;
            let head = loc + arrow_vec * 0.5;
            self.arrows.push(Arrow3::from_to_clr(tail, head, clr));
        }
    }

    pub fn map_transform(&mut self, t: Matrix4<f64>) {
        for arrow in self.arrows.iter_mut() {
            arrow.map_transform(&t);
        }
        // Record t in arrow_transforms
        self.arrow_transforms = t * self.arrow_transforms;
    }

    pub fn map_arrow_transforms(&mut self) {
        for arrow in self.arrows.iter_mut() {
            arrow.map_transform(&self.arrow_transforms);
        }
    }
}
