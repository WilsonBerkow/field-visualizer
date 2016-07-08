use na;
use num;

pub fn f64_max(x: f64, y: f64) -> f64 {
    if x > y { x } else { y }
}

pub fn translation3_mat<T: na::BaseNum>(v: na::Vector3<T>) -> na::Matrix4<T> {
    let mut res: na::Matrix4<T> = num::One::one();
    res.m14 = v.x;
    res.m24 = v.y;
    res.m34 = v.z;
    res
}

// The following should be in nalgebra, which implements
// Mul<Point4<N>> for Matrix4<N> but not also for &'a Matrix<N>.
// The definition mirrors nalgebra's definition of the method
// `mul` in `impl... for Matrix4<N>`.
#[inline]
pub fn ref_mat4_mul(mat: &na::Matrix4<f64>, right: na::Point4<f64>) -> na::Point4<f64> {
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
