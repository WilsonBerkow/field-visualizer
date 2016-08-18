use num::{One, Zero};

use pw;

use na::{Point3, Vector3, Matrix4, PerspectiveMatrix3};

use field::{FieldData, VectorField, FieldView};

mod charge;
pub use self::charge::*;

use arrow::Arrow3;

use util;
use consts::*;

pub struct PointChargesFieldView {
    // The PointCharges whose field we are visualizing
    pub charges: Vec<PointCharge>,

    greatest_field: f64,
    greatest_pot: f64,

    // The arrows describing the field strengths
    arrows: Vec<Arrow3>,

    // The product of all transformations applied to the arrows
    // of the PointChargesFieldView (not to the camera). With this we can move
    // the location of a charge, rebuild the field, and then reapply
    // arrow_transforms to put the field where the user expects it
    arrow_transforms: Matrix4<f64>,

    // The transformation from absolute positions (as in `arrows`) to
    // positions relative to the camera's position and orientation
    camera: Matrix4<f64>,

    // The bounds of the grid in which we are viewing the field
    x_range: (i64, i64),
    y_range: (i64, i64),
    z_range: (i64, i64),

    // For getting to 2-space
    persp: PerspectiveMatrix3<f64>,
}

impl VectorField for PointChargesFieldView {
    fn field_data_at(&self, p: &Point3<f64>) -> FieldData {
        let mut field_data: FieldData = self.charges.iter()
            .map(|chg| chg.field_data_at(&p))
            .fold(Zero::zero(), |f0, f1| f0 + f1);
        field_data.update_norm();
        field_data
    }
}

impl PointChargesFieldView {
    pub fn new(camera_offset: Vector3<f64>, greatest_field: f64, greatest_pot: f64, charges: Vec<PointCharge>) -> PointChargesFieldView {
        PointChargesFieldView {
            arrows: vec![],
            arrow_transforms: One::one(),
            camera: util::translation_mat4(camera_offset + Vector3::new(0.0, -GRID_S_2, 0.0)),
            persp: PerspectiveMatrix3::new(1.0, 200.0, NEAR_PLANE_Z, FAR_PLANE_Z),
            charges: charges,

            // Ranges in x,y,z in which we will draw the field vectors
            // These are expressed in terms on cubes in the grid, ie.,
            // in units of GRID_S voxels.
            x_range: (-4, 6),
            y_range: (-2, 4),
            z_range: (-2, 4),

            greatest_field: greatest_field,
            greatest_pot: greatest_pot,
        }
    }

    pub fn new_capacitor(camera_dist: f64, greatest_field: f64, greatest_pot: f64) -> PointChargesFieldView {
        let x_range = (-2, 3);
        let y_range = (-3, 3);
        let z_range = (-1, 2);
        // fact used to place charges more densely
        let fact = 3;
        // extra margin of charges around visible area to sraighten out field
        let (buffer_l, buffer_r) = (3, 2);
        let mut charges = vec![];
        for i in fact * x_range.0 - buffer_l..fact * x_range.1 + buffer_r {
            let x = i as f64 * GRID_S / fact as f64;
            for j in fact * z_range.0 - 3..fact * z_range.1 + 2 {
                let z = j as f64 * GRID_S_2;
                charges.push(
                    PointCharge::new(1.0, Point3::new(x, -4.0 * GRID_S, z))
                );
                charges.push(
                    PointCharge::new(-1.0, Point3::new(x, 4.0 * GRID_S, z))
                );
            }
        }
        PointChargesFieldView {
            arrows: vec![],
            arrow_transforms: One::one(),
            camera: util::translation_mat4(Vector3::new(0.0, 0.0, camera_dist)),
            persp: PerspectiveMatrix3::new(1.0, 200.0, NEAR_PLANE_Z, FAR_PLANE_Z),
            charges: charges,

            // Ranges in x,y,z in which we will draw the field vectors
            // These are expressed in terms on cubes in the grid, ie.,
            // in units of GRID_S voxels.
            x_range: x_range,
            y_range: y_range,
            z_range: z_range,

            greatest_field: greatest_field,
            greatest_pot: greatest_pot,
        }
    }
}

impl FieldView for PointChargesFieldView {
    fn ranges(&self) -> ((i64, i64), (i64, i64), (i64, i64)) {
        (self.x_range, self.y_range, self.z_range)
    }

    fn set_arrows(&mut self, arrows: Vec<Arrow3>) {
        self.arrows = arrows;
    }

    fn render(&self, c: pw::Context, gl: &mut pw::G2d, view: [f64; 4]) {
        // Clear the section on which we will draw
        pw::Rectangle::new(pw::color::WHITE).draw(view, &c.draw_state, c.transform, gl);

        let persp = &self.persp;
        let cam = &self.camera;
        for arrow in &self.arrows {
            arrow.draw(c, gl, persp, cam, view);
        }
    }

    fn transform_arrows(&mut self, t: Matrix4<f64>) {
        for arrow in self.arrows.iter_mut() {
            arrow.map_transform(&t);
        }
        // Record t in arrow_transforms
        self.arrow_transforms = t * self.arrow_transforms;
    }

    fn reapply_arrow_transforms(&mut self) {
        for arrow in self.arrows.iter_mut() {
            arrow.map_transform(&self.arrow_transforms);
        }
    }

    fn transform_camera(&mut self, t: Matrix4<f64>) {
        self.camera = t * self.camera;
    }

    fn greatest_field(&self) -> f64 {
        self.greatest_field
    }

    fn greatest_pot(&self) -> f64 {
        self.greatest_pot
    }
}
