use na;
use na::{ Vector3, Rotation3, Point4, Matrix4, ToHomogeneous };
use num::One;

pub fn f64_min(x: f64, y: f64) -> f64 {
    if x < y { x } else { y }
}

pub fn f64_max(x: f64, y: f64) -> f64 {
    if x > y { x } else { y }
}

pub fn translation_mat4<T: na::BaseNum>(v: Vector3<T>) -> Matrix4<T> {
    let mut res: Matrix4<T> = One::one();
    res.m14 = v.x;
    res.m24 = v.y;
    res.m34 = v.z;
    res
}

pub fn euler_rot_mat4<T: na::BaseFloat>(x: T, y: T, z: T) -> Matrix4<T> {
    let rot = Rotation3::new_with_euler_angles(x, y, z);
    rot.submatrix().to_homogeneous()
}

// The following should be in nalgebra, which implements
// Mul<Point4<N>> for Matrix4<N> but not also for &'a Matrix<N>.
// The definition mirrors nalgebra's definition of the method
// `mul` in `impl... for Matrix4<N>`.
#[inline]
pub fn ref_mat4_mul(mat: &Matrix4<f64>, right: Point4<f64>) -> Point4<f64> {
    let mut res: Point4<f64> = Point4::new(0.0, 0.0, 0.0, 0.0);
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

macro_rules! slider {
    (
        ids [ $self_id:ident, $text_canv:ident, $slider_canv:ident, $text_id:ident, $slider_id:ident ],
        above = $above:expr,
        top = $top:expr,
        view = $view:ident, ui = $ui:ident,
        value = $value:expr,
        range = [ $leftbound:expr, $rightbound:expr ],
        text = $text_prefix:expr,
        react = $react:expr
    ) => ({
        let value = $value;
        let label = format!("{}{:.*}", $text_prefix, 1, value);
        {
            let items = [
                ($text_canv, Canvas::new().color(color::DARK_CHARCOAL).length_weight(1.3).frame(0.0)),
                ($slider_canv, Canvas::new().color(color::DARK_CHARCOAL).frame(0.0)),
            ];
            let mut c = Canvas::new().flow_right(&items);
            c = if $top { c.mid_top_of($above) } else { c.down_from($above, 5.0) };
            c.w($view[0]).set($self_id, $ui);
        }
        Text::new(&label)
            .color(color::WHITE)
            .align_text_right()
            .top_right_of($text_canv)
            .padded_w_of($text_canv, 7.0)
            .set($text_id, $ui);
        Slider::new(value, $leftbound, $rightbound)
            .mid_bottom_of($slider_canv)
            .padded_w_of($slider_canv, 10.0)
            .h(CHROME_SLIDER_HEIGHT as f64)
            .react($react)
            .set($slider_id, $ui);
    });
}
