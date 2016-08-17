use na::{Point3, Matrix4, Norm};

use pw;

use arrow::Arrow3;

mod vector_field;
pub use self::vector_field::*;

use util;
use consts::*;

pub trait FieldView: VectorField {
    fn ranges(&self) -> ((i64, i64), (i64, i64), (i64, i64));
    fn set_arrows(&mut self, Vec<Arrow3>);
    fn render(&self, c: pw::Context, gl: &mut pw::G2d, view: [f64; 4]);
    fn transform_arrows(&mut self, Matrix4<f64>);
    fn reapply_arrow_transforms(&mut self);
    fn transform_camera(&mut self, Matrix4<f64>);

    // max values are used in rendering: vectors with field strength
    // field_max() are as long as they can be, and similar with pot_max()
    // for shading of vectors. None indicates max should be dynamically found
    fn field_max(&self) -> Option<f64>;
    fn pot_max(&self) -> Option<f64>;

    fn populate_field(&mut self) {
        let ((lx, rx), (ly, ry), (lz, rz)) = self.ranges();

        // Keep track of stongest value of field so we can scale all
        // field vectors later and cap the length of the longest one
        let mut max_field: f64 = match self.field_max() {
            None => 0.0, // will be dynamically calculated
            Some(max) => max,
        };

        // Same for potential, for color or alpha
        let mut max_abs_potential: f64 = match self.pot_max() {
            None => 0.0, // will be dynamically calculated
            Some(max) => max,
        };

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
                    if self.field_max().is_none() {
                        max_field = util::f64_max(max_field, field_data.force_mag);
                    }
                    if self.pot_max().is_none() {
                        max_abs_potential = util::f64_max(max_abs_potential, field_data.potential.abs());
                    }

                    field.push((loc, field_data));
                }
            }
        }

        let mut arrows = vec![];
        // Generate arrows based on those data
        for (loc, field_data) in field {
            let rel_mag = field_data.force_mag / max_field;
            // Average pot with 1.0 to shift negative values to the range (ie [0.0, 1.0])
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
            arrows.push(Arrow3::from_to_clr(tail, head, clr));
        }
        self.set_arrows(arrows);
    }
}
