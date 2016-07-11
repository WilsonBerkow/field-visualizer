extern crate piston_window as pw;
extern crate num;
extern crate nalgebra as na;

use std::f64::consts::PI;

mod arrow;
pub use arrow::Arrow3;

mod util;

mod consts;
use consts::*;

mod field;
use field::FieldView;

fn main() {
    let opengl: pw::OpenGL = pw::OpenGL::V3_2;
    let mut window: pw::PistonWindow = pw::WindowSettings::new(
            "Field Visualizer",
            [WIDTH, HEIGHT]
        )
        .opengl(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut app: App = App {
        field: FieldView::new(75.0, vec![
                field::PointCharge::new(10.0, na::Point3::new(5.0 * GRID_S_2, GRID_S_2, GRID_S_2)),
                field::PointCharge::new(-10.0, na::Point3::new(-5.0 * GRID_S_2, GRID_S_2, GRID_S_2)),
            ]),
    };

    app.field.populate_field();

    if SHOW_GRID {
        app.field.populate_grid();
    }

    while let Some(e) = window.next() {
        match e {
            pw::Event::Render(r_args) => {
                window.draw_2d(&e, |c, g| {
                    app.render(&r_args, c, g);
                });
            },
            pw::Event::Update(u_args) => {
                app.update(&u_args);
            },
            pw::Event::Input(pw::Input::Press(pw::Button::Keyboard(key))) => {
                app.keypress(key);
            },
            _ => {},
        }
    }
}

struct App {
    field: FieldView,
}

impl App {
    fn update(&mut self, args: &pw::UpdateArgs) {
    }

    fn render(&mut self, args: &pw::RenderArgs, c: pw::Context, g: &mut pw::G2d) {
        self.field.render(args, c, g);
    }

    fn keypress(&mut self, key: pw::Key) {
        match key {
            pw::Key::Up => {
                // TODO: Define a mutating leftMultiply for Matrix4
                self.field.camera = util::euler_rot_mat4(PI * 0.01, 0.0, 0.0) * self.field.camera;
            },
            pw::Key::Down => {
                self.field.camera = util::euler_rot_mat4(-PI * 0.01, 0.0, 0.0) * self.field.camera;
            },
            pw::Key::Right => {
                self.field.camera = util::euler_rot_mat4(0.0, -PI * 0.01, 0.0) * self.field.camera;
            },
            pw::Key::Left => {
                self.field.camera = util::euler_rot_mat4(0.0, PI * 0.01, 0.0) * self.field.camera;
            },
            pw::Key::W => {
                let transmat = util::translation_mat4(na::Vector3::new(0.0, 0.0, -1.0));
                self.field.camera = transmat * self.field.camera;
            },
            pw::Key::S => {
                let transmat = util::translation_mat4(na::Vector3::new(0.0, 0.0, 1.0));
                self.field.camera = transmat * self.field.camera;
            },
            pw::Key::D => {
                let transmat = util::translation_mat4(na::Vector3::new(-1.0, 0.0, 0.0));
                self.field.camera = transmat * self.field.camera;
            },
            pw::Key::A => {
                let transmat = util::translation_mat4(na::Vector3::new(1.0, 0.0, 0.0));
                self.field.camera = transmat * self.field.camera;
            },
            pw::Key::Q => {
                let transmat = util::translation_mat4(na::Vector3::new(0.0, -1.0, 0.0));
                self.field.camera = transmat * self.field.camera;
            },
            pw::Key::E => {
                let transmat = util::translation_mat4(na::Vector3::new(0.0, 1.0, 0.0));
                self.field.camera = transmat * self.field.camera;
            },
            pw::Key::I => {
                self.field.map_transform(util::euler_rot_mat4(PI * 0.01, 0.0, 0.0));
            },
            pw::Key::K => {
                self.field.map_transform(util::euler_rot_mat4(-PI * 0.01, 0.0, 0.0));
            },
            pw::Key::L => {
                self.field.map_transform(util::euler_rot_mat4(0.0, PI * 0.01, 0.0));
            },
            pw::Key::J => {
                self.field.map_transform(util::euler_rot_mat4(0.0, -PI * 0.01, 0.0));
            },
            pw::Key::T => {
                self.field.charges[0].loc.y -= CHARGE_MVMT_STEP;
                self.field.populate_field();
                self.field.map_arrow_transforms();
            },
            pw::Key::G => {
                self.field.charges[0].loc.y += CHARGE_MVMT_STEP;
                self.field.populate_field();
                self.field.map_arrow_transforms();
            },
            pw::Key::H => {
                self.field.charges[0].loc.x += CHARGE_MVMT_STEP;
                self.field.populate_field();
                self.field.map_arrow_transforms();
            },
            pw::Key::F => {
                self.field.charges[0].loc.x -= CHARGE_MVMT_STEP;
                self.field.populate_field();
                self.field.map_arrow_transforms();
            },
            pw::Key::R => {
                self.field.charges[0].loc.z -= CHARGE_MVMT_STEP;
                self.field.populate_field();
                self.field.map_arrow_transforms();
            },
            pw::Key::Y => {
                self.field.charges[0].loc.z += CHARGE_MVMT_STEP;
                self.field.populate_field();
                self.field.map_arrow_transforms();
            },
            _ => {},
        }
    }
}
