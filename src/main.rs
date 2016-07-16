extern crate find_folder;
extern crate piston_window as pw;
#[macro_use] extern crate conrod;
extern crate num;
extern crate nalgebra as na;

use pw::EventLoop;

use std::f64::consts::PI;

mod arrow;
pub use arrow::Arrow3;

mod util;

mod consts;
use consts::*;

mod field;
use field::FieldView;

type Backend = (pw::G2dTexture<'static>, pw::Glyphs);
type Ui = conrod::Ui<Backend>;
type UiCell<'a> = conrod::UiCell<'a, Backend>;

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

    window.set_ups(60);

    let mut app: App = App {
        ui: {
            use conrod::Theme;
            let assets = find_folder::Search::ParentsThenKids(3, 3)
                .for_folder("assets").unwrap();
            let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
            let theme = Theme::default();
            let glyph_cache = pw::Glyphs::new(
                    &font_path, window.factory.clone()
                ).unwrap();
            Ui::new(glyph_cache, theme)
        },
        field: FieldView::new(75.0, vec![
            field::PointCharge::new(10.0, na::Point3::new(5.0 * GRID_S_2, GRID_S_2, GRID_S_2)),
            field::PointCharge::new(-10.0, na::Point3::new(-5.0 * GRID_S_2, GRID_S_2, GRID_S_2)),
        ]),
        view: [VIEW_RIGHT - VIEW_W, VIEW_BOTTOM - VIEW_H, VIEW_W, VIEW_H],
        window: [WIDTH, HEIGHT],
        rebuild_queued: false,
    };

    app.field.populate_field();

    if SHOW_GRID {
        app.field.populate_grid();
    }

    while let Some(event) = window.next() {
        app.ui.handle_event(event.clone());
        match event {
            pw::Event::Render(r_args) => {
                window.draw_2d(&event, |context, graphics| {
                    app.render(&r_args, context, graphics);
                });
            },
            pw::Event::Update(u_args) => {
                app.update(&u_args);
            },
            pw::Event::Input(pw::Input::Press(pw::Button::Keyboard(key))) => {
                app.keypress(key);
            },
            _ => {
                app.idle();
            },
        }
    }
}

struct App {
    ui: Ui,
    field: FieldView,
    view: [f64; 4], // [x, y, width, height]
    window: [u32; 2], // [width, height]
    rebuild_queued: bool, // for rebuilding field arrows after changes to, e.g., charge strengths
}

impl App {
    fn update(&mut self, _args: &pw::UpdateArgs) {
        self.set_widgets();
    }

    fn idle(&mut self) {
        if self.rebuild_queued {
            self.field.populate_field();
            self.field.map_arrow_transforms();
            self.rebuild_queued = false;
        }
    }

    fn render(&mut self, args: &pw::RenderArgs, c: pw::Context, g: &mut pw::G2d) {
        self.update_view(c);
        self.ui.draw_if_changed(c, g);
        let mut context = c.clone();
        context.draw_state.scissor = Some(self.get_view_scissor());
        self.field.render(args, context, g, self.view);
    }

    fn update_view(&mut self, c: pw::Context) {
        if let Some(viewport) = c.viewport {
            self.window = viewport.window_size;
            let (w, h) = (viewport.window_size[0], viewport.window_size[1]);
            let x = util::f64_max(w as f64 * 0.3, CHROME_MIN_WIDTH as f64);
            self.view = [
                x,
                BANNER_HEIGHT,
                w as f64 - x,
                h as f64 - BANNER_HEIGHT,
            ];
        }
    }

    fn get_view_scissor(&self) -> [u32; 4] {
        // scissor (clip) mask is a [u32; 4] of [x, y, w, h] where the
        // origin is at the bottom left of the window and y increases up
        [
            self.view[0] as u32,
            (self.window[1] as f64 - self.view[1] - self.view[3]) as u32,
            self.view[2] as u32,
            self.view[3] as u32
        ]
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
                self.rebuild_queued = true;
            },
            pw::Key::G => {
                self.field.charges[0].loc.y += CHARGE_MVMT_STEP;
                self.rebuild_queued = true;
            },
            pw::Key::H => {
                self.field.charges[0].loc.x += CHARGE_MVMT_STEP;
                self.rebuild_queued = true;
            },
            pw::Key::F => {
                self.field.charges[0].loc.x -= CHARGE_MVMT_STEP;
                self.rebuild_queued = true;
            },
            pw::Key::R => {
                self.field.charges[0].loc.z -= CHARGE_MVMT_STEP;
                self.rebuild_queued = true;
            },
            pw::Key::Y => {
                self.field.charges[0].loc.z += CHARGE_MVMT_STEP;
                self.rebuild_queued = true;
            },
            _ => {},
        }
    }

    fn set_widgets(&mut self) {
        let (w, h) = (self.window[0] as f64, self.window[1] as f64);
        let field = &mut self.field;
        let view = self.view;
        let mut queue_rebuild = false;
        self.ui.set_widgets(|ref mut ui: UiCell| {
            use conrod::{color, Widget, Canvas, Text, Slider, Sizeable, Colorable, Positionable, Frameable};
            Canvas::new().flow_down(&[
                    (HEADER, Canvas::new().length(BANNER_HEIGHT).color(color::CHARCOAL)),
                    (CONTENT, Canvas::new().length(h - BANNER_HEIGHT).flow_right(&[
                        (BODY, Canvas::new().color(color::DARK_CHARCOAL).length(view[0])),
                        (BODY_RIGHT, Canvas::new().length(view[2]))
                    ]))
                ]).top_left().set(CANVAS, ui);
            Text::new("Fancy Fields")
                .color(color::WHITE)
                .font_size(BANNER_FONT_SIZE)
                .mid_left_with_margin_on(HEADER, 5.0)
                .set(TITLE, ui);
            // Label and slider for right charge value
            {
                let value = field.charges[0].charge;
                let label = format!("Right charge: {:.*}", 1, value);
                Canvas::new().flow_right(&[
                    (SLIDER1_LC, Canvas::new().color(color::DARK_CHARCOAL).length_weight(1.3).frame(0.0)),
                    (SLIDER1_SC, Canvas::new().color(color::DARK_CHARCOAL).frame(0.0)),
                ]).mid_top_of(BODY).w(view[0]).set(SLIDER1, ui);
                Text::new(&label)
                    .align_text_right()
                    .top_right_of(SLIDER1_LC)
                    .padded_w_of(SLIDER1_LC, 3.0)
                    .set(SLIDER1_L, ui);
                Slider::new(value, 0.0, 10.0)
                    .middle_of(SLIDER1_SC)
                    .padded_w_of(SLIDER1_SC, 10.0)
                    .h(CHROME_SLIDER_HEIGHT as f64)
                    .react(|c: f64| {
                            field.charges[0].charge = c;
                            queue_rebuild = true;
                        })
                    .set(SLIDER1_S, ui);
            }
            // Label and slider for left charge value
            {
                let value = field.charges[1].charge;
                let label = format!("Left charge: {:.*}", 1, value);
                Canvas::new().flow_right(&[
                    (SLIDER2_LC, Canvas::new().color(color::DARK_CHARCOAL).length_weight(1.3).frame(0.0)),
                    (SLIDER2_SC, Canvas::new().color(color::DARK_CHARCOAL).frame(0.0)),
                ]).down_from(SLIDER1_LC, 5.0).w(view[0]).set(SLIDER2, ui);
                Text::new(&label)
                    .align_text_right()
                    .top_right_of(SLIDER2_LC)
                    .padded_w_of(SLIDER2_LC, 3.0)
                    .set(SLIDER2_L, ui);
                Slider::new(value, 0.0, -10.0)
                    .middle_of(SLIDER2_SC)
                    .padded_w_of(SLIDER2_SC, 10.0)
                    .h(CHROME_SLIDER_HEIGHT as f64)
                    .react(|c: f64| {
                            field.charges[1].charge = c;
                            queue_rebuild = true;
                        })
                    .set(SLIDER2_S, ui);
            }
        });
        if queue_rebuild {
            self.rebuild_queued = true;
        }
    }
}

widget_ids! {
    CANVAS,
    HEADER,
    TITLE,
    CONTENT,
    BODY,
    BODY_RIGHT,
    SLIDER1,
    SLIDER1_LC,
    SLIDER1_SC,
    SLIDER1_L,
    SLIDER1_S,
    SLIDER2,
    SLIDER2_LC,
    SLIDER2_SC,
    SLIDER2_L,
    SLIDER2_S,
}
