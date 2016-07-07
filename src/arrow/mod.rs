use graphics;
use opengl_graphics::GlGraphics;

use na;
use na::{ ToHomogeneous, FromHomogeneous };

use consts::*;

mod arrow2;

pub use self::arrow2::*;

pub struct Arrow3 {
    tail: na::Point3<f64>,
    head: na::Point3<f64>,
    clr: [f32; 4],
}

impl Arrow3 {
    pub fn from_to_clr(tail: na::Point3<f64>, head: na::Point3<f64>, clr: [f32; 4]) -> Arrow3 {
        Arrow3 {
            tail: tail,
            head: head,
            clr: clr,
        }
    }

    pub fn from_to(tail: na::Point3<f64>, head: na::Point3<f64>) -> Arrow3 {
        Arrow3 {
            tail: tail,
            head: head,
            clr: ARROW_CLR,
        }
    }

    pub fn map_transform(&mut self, mat: &na::Matrix4<f64>) {
        self.tail = transform_in_homo(self.tail, mat);
        self.head = transform_in_homo(self.head, mat);
    }

    pub fn project_to_viewport(&self, persp: &na::PerspectiveMatrix3<f64>, camera: na::Matrix4<f64>) -> Option<Arrow2> {
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
            Some(Arrow2::from_to_clr(
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

    pub fn draw(&self, c: graphics::context::Context, gl: &mut GlGraphics, persp: &na::PerspectiveMatrix3<f64>, camera: na::Matrix4<f64>) {
        if let Some(a2d) = self.project_to_viewport(persp, camera) {
            a2d.draw(c, gl);
        }
    }

    pub fn draw_no_head(&self, c: graphics::context::Context, gl: &mut GlGraphics, persp: &na::PerspectiveMatrix3<f64>, camera: na::Matrix4<f64>) {
        if let Some(a2d) = self.project_to_viewport(persp, camera) {
            a2d.draw_no_head(c, gl);
        }
    }
}

// Lift a Point3 to Point4, apply a Matrix4, then flatten it back to Point3
fn transform_in_homo(pt: na::Point3<f64>, mat: &na::Matrix4<f64>) -> na::Point3<f64> {
    <na::Point3<f64> as FromHomogeneous<na::Point4<f64>>>::from(&(ref_mat4_mul(mat, pt.to_homogeneous())))
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
