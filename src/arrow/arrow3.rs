use pw;

use na::{Point2, Point3, Point4, Matrix4, PerspectiveMatrix3, ToHomogeneous, FromHomogeneous};

use arrow::Arrow2;
use util::{f64_min, ref_mat4_mul};
use consts::*;

pub struct Arrow3 {
    pub tail: Point3<f64>,
    pub head: Point3<f64>,

    // field and potential scaled down to range [0.0, 1.0]
    pub field: f64,
    pub potential: f64,
}

impl Arrow3 {
    pub fn map_transform(&mut self, mat: &Matrix4<f64>) {
        self.tail = transform_in_homo(self.tail, mat);
        self.head = transform_in_homo(self.head, mat);
    }

    fn project_to_viewport(&self, persp: &PerspectiveMatrix3<f64>, camera: &Matrix4<f64>, view: [f64; 4]) -> Option<Arrow2> {
        // Transform relative to the camera position:
        let headr: Point3<f64> = transform_in_homo(self.head, camera);
        let tailr: Point3<f64> = transform_in_homo(self.tail, camera);
        if headr.z <= NEAR_PLANE_Z || tailr.z <= NEAR_PLANE_Z {
            None
        } else {
            // Project onto "device" surface:
            let head_prime = persp.project_point(&headr);
            let tail_prime = persp.project_point(&tailr);
            // Trasform to viewport surface:
            let scale_factor = 0.3 * f64_min(view[2], view[3]);
            let cx = view[0] + view[2] * 0.5;
            let cy = view[1] + view[3] * 0.5 + GRID_S;
            Some(Arrow2::from_to_clr(
                Point2::new(
                    tail_prime.x * scale_factor + cx,
                    tail_prime.y * scale_factor + cy,
                ),
                Point2::new(
                    head_prime.x * scale_factor + cx,
                    head_prime.y * scale_factor + cy,
                ),
                arrow_color(self.field, self.potential),
            ))
        }
    }

    pub fn draw(&self, c: pw::Context, gl: &mut pw::G2d, persp: &PerspectiveMatrix3<f64>, camera: &Matrix4<f64>, view: [f64; 4]) {
        if let Some(a2d) = self.project_to_viewport(persp, camera, view) {
            a2d.draw(c, gl);
        }
    }
}

// Lift a Point3 to Point4, apply a Matrix4, then flatten it back to Point3
fn transform_in_homo(pt: Point3<f64>, mat: &Matrix4<f64>) -> Point3<f64> {
    <Point3<f64> as FromHomogeneous<Point4<f64>>>::from(&(ref_mat4_mul(mat, pt.to_homogeneous())))
}

fn arrow_color(field: f64, potential: f64) -> [f32; 4] {
    // Adjust so pot is never below 0.3
    let adjusted_pot = (1.0 - 0.7 * (1.0 - potential)) as f32;
    if POTENTIAL_SHADING {
        // `clr` depends on potential
        if COLORFUL_POTENTIAL {
            // Use a scale from red to blue
            if adjusted_pot > 0.0 {
                [adjusted_pot, 0.0, 1.0 - adjusted_pot, 1.0]
            } else {
                [1.0 + adjusted_pot, 0.0, -adjusted_pot, 1.0]
            }
        } else {
            // Use an alpha scale
            [0.0, 0.0, 0.0, adjusted_pot]
        }
    } else {
        // `clr` depends on field magnitude
        [0.0, 0.0, 0.0, (field * 2.2) as f32]
    }
}
