pub const WIDTH: u32 = 600;
pub const HEIGHT: u32 = 600;
pub const WIDTHF: f64 = WIDTH as f64;
pub const HEIGHTF: f64 = HEIGHT as f64;

pub const BANNER_HEIGHT: f64 = 40.0;
pub const BANNER_FONT_SIZE: u32 = 20;

pub const CHROME_MIN_WIDTH: u32 = 300;
pub const CHROME_PAD: u32 = 5;
pub const CHROME_SLIDER_HEIGHT: u32 = 15;

pub const VIEW_W: f64 = 400.0;
pub const VIEW_RIGHT: f64 = WIDTHF;
pub const VIEW_H: f64 = 400.0;
pub const VIEW_BOTTOM: f64 = HEIGHTF;

pub const GRID_S: f64 = 15.0;
pub const GRID_S_2: f64 = GRID_S * 0.5;
pub const GRID_DIAG: f64 = GRID_S * 1.73205080757; // 1.7... is sqrt(3)

pub const CHARGE_MVMT_STEP: f64 = GRID_S;

pub const ARROW_CLR: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

pub const POTENTIAL_SHADING: bool = false;
pub const COLORFUL_POTENTIAL: bool = false;

pub const NEAR_PLANE_Z: f64 = 1.0;
pub const FAR_PLANE_Z: f64 = 100.0;

// Maximum and minimum lengths of a field vector:
pub const FIELD_VEC_MAX_LEN: f64 = GRID_DIAG * 0.8;
pub const FIELD_VEC_MIN_LEN: f64 = GRID_DIAG * 0.1;

pub const FIELD_VEC_LEN_RANGE: f64 = FIELD_VEC_MAX_LEN - FIELD_VEC_MIN_LEN;
