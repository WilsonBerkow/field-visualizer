use pw;

use na::{Point3, Point4, Matrix4, PerspectiveMatrix3, ToHomogeneous, FromHomogeneous};

use util;
use consts::*;

pub struct Arrow {
    pub tail: Point3<f64>,
    pub head: Point3<f64>,

    // field and potential scaled down to range [0.0, 1.0]
    pub field: f64,
    pub potential: f64,
}

impl Arrow {
    pub fn map_transform(&mut self, mat: &Matrix4<f64>) {
        self.tail = transform_in_homo(self.tail, mat);
        self.head = transform_in_homo(self.head, mat);
    }

    pub fn draw(&self, c: pw::Context, gl: &mut pw::G2d, persp: &PerspectiveMatrix3<f64>, camera: &Matrix4<f64>, view: [f64; 4]) {
        if let Some(path) = self.project_to_viewport(persp, camera, view) {
            let line_style =
                pw::Line::new(self.color(), 1.0);
            line_style.draw_arrow(path, 5.0, &c.draw_state, c.transform, gl);
        }
    }

    fn project_to_viewport(
        &self,
        persp: &PerspectiveMatrix3<f64>,
        camera: &Matrix4<f64>,
        viewport: [f64; 4]
    )
    -> Option<[f64; 4]> {
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
            let scale_factor = 0.3 * util::f64_min(viewport[2], viewport[3]);
            let cx = viewport[0] + viewport[2] * 0.5;
            let cy = viewport[1] + viewport[3] * 0.5;
            Some([
                tail_prime.x * scale_factor + cx, // x0
                tail_prime.y * scale_factor + cy, // y0
                head_prime.x * scale_factor + cx, // x1
                head_prime.y * scale_factor + cy, // y1
            ])
        }
    }

    fn color(&self) -> [f32; 4] {
        // Calls to f##_max(..., 0.0) ensure slight imprecision will not
        // result in a negative channel value, which makes the color trip out
        let pot = util::f64_max(self.potential, 0.0) as f32;
        if POTENTIAL_SHADING {
            // `clr` depends on potential
            if COLORFUL_POTENTIAL {
                // Use a scale from red to blue
                [pot, 0.0, util::f32_max(1.0 - pot, 0.0), 1.0]
            } else {
                // Use an alpha scale, adjusting such that alpha is never below 0.3
                let adjusted_potential = 0.7 * pot + 0.3;
                [0.0, 0.0, 0.0, adjusted_potential]
            }
        } else {
            // `clr` depends on field magnitude
            [0.0, 0.0, 0.0, (self.field * 2.2) as f32]
        }
    }
}

// Lift a Point3 to Point4, apply a Matrix4, then flatten it back to Point3
fn transform_in_homo(pt: Point3<f64>, mat: &Matrix4<f64>) -> Point3<f64> {
    <Point3<f64> as FromHomogeneous<Point4<f64>>>::from(&(util::ref_mat4_mul(mat, pt.to_homogeneous())))
}
