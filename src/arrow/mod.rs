use pw;

use na::{Point2, Point3, Point4, Matrix4, PerspectiveMatrix3, ToHomogeneous, FromHomogeneous};

use util::{f64_min, ref_mat4_mul};
use consts::*;

mod arrow2;

pub use self::arrow2::*;

pub struct Arrow3 {
    pub tail: Point3<f64>,
    head: Point3<f64>,
    clr: [f32; 4],
}

impl Arrow3 {
    pub fn from_to_clr(tail: Point3<f64>, head: Point3<f64>, clr: [f32; 4]) -> Arrow3 {
        Arrow3 {
            tail: tail,
            head: head,
            clr: clr,
        }
    }

    pub fn from_to(tail: Point3<f64>, head: Point3<f64>) -> Arrow3 {
        Arrow3 {
            tail: tail,
            head: head,
            clr: ARROW_CLR,
        }
    }

    pub fn map_transform(&mut self, mat: &Matrix4<f64>) {
        self.tail = transform_in_homo(self.tail, mat);
        self.head = transform_in_homo(self.head, mat);
    }

    pub fn project_to_viewport(&self, persp: &PerspectiveMatrix3<f64>, camera: Matrix4<f64>, view: [f64; 4]) -> Option<Arrow2> {
        // Transform relative to the camera position:
        let headr: Point3<f64> = transform_in_homo(self.head, &camera);
        let tailr: Point3<f64> = transform_in_homo(self.tail, &camera);
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
                self.clr,
            ))
        }
    }

    pub fn draw(&self, c: pw::Context, gl: &mut pw::G2d, persp: &PerspectiveMatrix3<f64>, camera: Matrix4<f64>, view: [f64; 4]) {
        if let Some(a2d) = self.project_to_viewport(persp, camera, view) {
            a2d.draw(c, gl);
        }
    }

    pub fn draw_no_head(&self, c: pw::Context, gl: &mut pw::G2d, persp: &PerspectiveMatrix3<f64>, camera: Matrix4<f64>, view: [f64; 4]) {
        if let Some(a2d) = self.project_to_viewport(persp, camera, view) {
            a2d.draw_no_head(c, gl);
        }
    }
}

// Lift a Point3 to Point4, apply a Matrix4, then flatten it back to Point3
fn transform_in_homo(pt: Point3<f64>, mat: &Matrix4<f64>) -> Point3<f64> {
    <Point3<f64> as FromHomogeneous<Point4<f64>>>::from(&(ref_mat4_mul(mat, pt.to_homogeneous())))
}
