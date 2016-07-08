extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;
extern crate num;
extern crate nalgebra as na;

use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{ GlGraphics, OpenGL };

use std::f64::consts::PI;

mod arrow;
pub use arrow::Arrow3;

mod util;

mod consts;
use consts::*;

mod field;
use field::FieldView;

fn main() {
    let opengl: OpenGL = OpenGL::V3_2;
    let mut window: Window = WindowSettings::new(
            "Field Visualizer",
            [WIDTH, HEIGHT]
        )
        .opengl(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();
    let mut events = window.events();

    let mut app: App = App {
        gl: GlGraphics::new(opengl),
        field: FieldView::new(200.0, vec![
                field::PointCharge::new(10.0, na::Point3::new(5.0 * GRID_S_2, GRID_S_2, GRID_S_2)),
                field::PointCharge::new(-10.0, na::Point3::new(-5.0 * GRID_S_2, GRID_S_2, GRID_S_2)),
            ]),
    };

    app.field.populate_field();

    if SHOW_GRID {
        app.field.populate_grid();
    }

    while let Some(e) = events.next(&mut window) {
        if let Some(r) = e.render_args() {
            app.render(&r);
        }
        if let Some(u) = e.update_args() {
            app.update(&u);
        }
        if let Some(Button::Keyboard(k)) = e.press_args() {
            app.keypress(k);
        }
    }
}

struct App {
    gl: GlGraphics,
    field: FieldView,
}

impl App {
    fn update(&mut self, args: &UpdateArgs) {
    }

    fn render(&mut self, args: &RenderArgs) {
        self.field.render(&mut self.gl, args);
    }

    fn keypress(&mut self, key: Key) {
        match key {
            Key::Up => {
                // TODO: Define a mutating leftMultiply for Matrix4
                self.field.camera = util::euler_rot_mat4(PI * 0.01, 0.0, 0.0) * self.field.camera;
            },
            Key::Down => {
                self.field.camera = util::euler_rot_mat4(-PI * 0.01, 0.0, 0.0) * self.field.camera;
            },
            Key::Right => {
                self.field.camera = util::euler_rot_mat4(0.0, -PI * 0.01, 0.0) * self.field.camera;
            },
            Key::Left => {
                self.field.camera = util::euler_rot_mat4(0.0, PI * 0.01, 0.0) * self.field.camera;
            },
            Key::W => {
                let transmat = util::translation_mat4(na::Vector3::new(0.0, 0.0, -1.0));
                self.field.camera = transmat * self.field.camera;
            },
            Key::S => {
                let transmat = util::translation_mat4(na::Vector3::new(0.0, 0.0, 1.0));
                self.field.camera = transmat * self.field.camera;
            },
            Key::D => {
                let transmat = util::translation_mat4(na::Vector3::new(-1.0, 0.0, 0.0));
                self.field.camera = transmat * self.field.camera;
            },
            Key::A => {
                let transmat = util::translation_mat4(na::Vector3::new(1.0, 0.0, 0.0));
                self.field.camera = transmat * self.field.camera;
            },
            Key::Q => {
                let transmat = util::translation_mat4(na::Vector3::new(0.0, -1.0, 0.0));
                self.field.camera = transmat * self.field.camera;
            },
            Key::E => {
                let transmat = util::translation_mat4(na::Vector3::new(0.0, 1.0, 0.0));
                self.field.camera = transmat * self.field.camera;
            },
            Key::I => {
                self.field.map_transform(util::euler_rot_mat4(PI * 0.01, 0.0, 0.0));
            },
            Key::K => {
                self.field.map_transform(util::euler_rot_mat4(-PI * 0.01, 0.0, 0.0));
            },
            Key::L => {
                self.field.map_transform(util::euler_rot_mat4(0.0, PI * 0.01, 0.0));
            },
            Key::J => {
                self.field.map_transform(util::euler_rot_mat4(0.0, -PI * 0.01, 0.0));
            },
            Key::T => {
                self.field.charges[0].loc.y -= CHARGE_MVMT_STEP;
                self.field.populate_field();
                self.field.map_arrow_transforms();
            },
            Key::G => {
                self.field.charges[0].loc.y += CHARGE_MVMT_STEP;
                self.field.populate_field();
                self.field.map_arrow_transforms();
            },
            Key::H => {
                self.field.charges[0].loc.x += CHARGE_MVMT_STEP;
                self.field.populate_field();
                self.field.map_arrow_transforms();
            },
            Key::F => {
                self.field.charges[0].loc.x -= CHARGE_MVMT_STEP;
                self.field.populate_field();
                self.field.map_arrow_transforms();
            },
            Key::R => {
                self.field.charges[0].loc.z -= CHARGE_MVMT_STEP;
                self.field.populate_field();
                self.field.map_arrow_transforms();
            },
            Key::Y => {
                self.field.charges[0].loc.z += CHARGE_MVMT_STEP;
                self.field.populate_field();
                self.field.map_arrow_transforms();
            },
            _ => {},
        }
    }
}
