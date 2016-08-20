use na::{Point3, Matrix4, Norm};

use pw;

use arrow::Arrow;

mod vector_field;
pub use self::vector_field::*;

use util;

use consts::*;

pub trait FieldView: VectorField {
    // Each range incl. on lower bound, excl. on upper bound
    fn ranges(&self) -> ((i64, i64), (i64, i64), (i64, i64));

    fn transform_arrows(&mut self, Matrix4<f64>);
    fn reapply_arrow_transforms(&mut self);
    fn transform_camera(&mut self, Matrix4<f64>);

    fn set_arrows(&mut self, Vec<Arrow>);

    fn render(&self, c: pw::Context, gl: &mut pw::G2d, view: [f64; 4]);

    // `greatest_*` used for neat rendering:
    // field value which will correspond to the longest arrow that fits in the grid (ie., the
    // greatest field strength in the FieldView):
    fn greatest_field(&self) -> f64;
    // potential value which will have darkest color (ie., the greatest potential in the
    // FieldView)
    fn greatest_pot(&self) -> f64;
    // potential value for lightest color
    fn least_pot(&self) -> f64;

    fn populate_field(&mut self) {
        // Each range incl. on lower bound, excl. on upper bound
        let ((min_x, max_x), (min_y, max_y), (min_z, max_z)) = self.ranges();

        let mut arrows = vec![];

        // Generate arrows for each position in field
        for i in min_x..max_x {
            for j in min_y..max_y {
                for k in min_z..max_z {
                    let loc = Point3::new(
                        i as f64 * GRID_S,
                        j as f64 * GRID_S,
                        k as f64 * GRID_S);
                    let field_data = self.field_data_at(&loc);

                    let rel_field = field_data.force_mag / self.greatest_field();

                    // Map space of potential values from range
                    // [least_pot, greatest_pot] to range [0.0, 1.0]
                    let rel_pot = (field_data.potential - self.least_pot()) / (self.greatest_pot() - self.least_pot());

                    let length = FIELD_VEC_MIN_LEN + rel_field * FIELD_VEC_LEN_RANGE;

                    // loc is the center of the arrow stem
                    let arrow_vec = length * field_data.force_vec.normalize();
                    let tail = loc - arrow_vec * 0.5;
                    let head = loc + arrow_vec * 0.5;
                    arrows.push(Arrow {
                        tail: tail,
                        head: head,
                        field: rel_field,
                        potential: rel_pot,
                    });
                }
            }
        }

        self.set_arrows(arrows);
    }
}
