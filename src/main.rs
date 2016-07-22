extern crate find_folder;
extern crate piston_window as pw;
#[macro_use] extern crate conrod;
extern crate num;
extern crate nalgebra as na;

use pw::EventLoop;

use std::f64::consts::PI;

mod arrow;

mod field;
use field::FieldView;

mod point_charge;
use point_charge::{PointCharge, PointChargesFieldView};

#[macro_use] mod util;

mod consts;
use consts::*;

type Backend = (pw::G2dTexture<'static>, pw::Glyphs);
type Ui = conrod::Ui<Backend>;
type UiCell<'a> = conrod::UiCell<'a, Backend>;

fn main() {
    let opengl: pw::OpenGL = pw::OpenGL::V3_2;
    let mut window: pw::PistonWindow = pw::WindowSettings::new(
            TITLE,
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
        fields: FieldChoices {
            one_charge: PointChargesFieldView::new(
                na::Vector3::new(-GRID_S_2, 0.0, 75.0),
                vec![PointCharge::new(10.0, na::Point3::new(GRID_S_2, GRID_S_2, GRID_S_2))]
            ),
            two_charges: PointChargesFieldView::new(
                na::Vector3::new(0.0, 0.0, 75.0),
                vec![
                    PointCharge::new(10.0, na::Point3::new(5.0 * GRID_S_2, GRID_S_2, GRID_S_2)),
                    PointCharge::new(-10.0, na::Point3::new(-5.0 * GRID_S_2, GRID_S_2, GRID_S_2)),
                ]
            ),
            capacitor: PointChargesFieldView::new_capacitor(75.0),
        },
        selected: FieldChoice::TwoCharges,
        view: [VIEW_RIGHT - VIEW_W, VIEW_BOTTOM - VIEW_H, VIEW_W, VIEW_H],
        window: [WIDTH, HEIGHT],
        rebuild_queued: false,
        redraw_queued: true, // true for initial render of field
        redraw_echo_queued: false,
    };

    app.fields.one_charge.populate_field();
    app.fields.two_charges.populate_field();
    app.fields.capacitor.populate_field();

    while let Some(event) = window.next() {
        use pw::ResizeEvent;
        app.ui.handle_event(event.clone());
        match event {
            pw::Event::Render(_args) => {
                window.draw_2d(&event, |context, graphics| {
                    app.render(context, graphics);
                });
            },
            pw::Event::Update(_args) => {
                app.update();
            },
            pw::Event::Input(pw::Input::Press(pw::Button::Keyboard(key))) => {
                app.keypress(key);
            },
            _ => {
                if let Some(_) = event.resize_args() {
                    app.redraw_queued = true;
                } else {
                    app.idle();
                }
            },
        }
    }
}

struct FieldChoices {
    one_charge: PointChargesFieldView,
    two_charges: PointChargesFieldView,
    capacitor: PointChargesFieldView,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum FieldChoice {
    OneCharge,
    TwoCharges,
    Capacitor,
}

struct App {
    ui: Ui,
    fields: FieldChoices,
    selected: FieldChoice,
    view: [f64; 4], // [x, y, width, height]
    window: [u32; 2], // [width, height]
    rebuild_queued: bool, // for rebuilding field arrows after changes to, e.g., charge strengths
    redraw_queued: bool,
    // Upon drawing, we draw once more in the next frame (see
    // problem described in message of commit 93fe0b).
    redraw_echo_queued: bool,
}

impl App {
    fn update(&mut self) {
        self.set_widgets();
    }

    fn idle(&mut self) {
        if self.rebuild_queued {
            self.active_field().populate_field();
            self.active_field().reapply_arrow_transforms();
            self.rebuild_queued = false;
            // Then, redraw
            self.redraw_queued = true;
        }
    }

    fn render(&mut self, c: pw::Context, g: &mut pw::G2d) {
        self.update_view(c);
        self.ui.draw_if_changed(c, g);
        if self.redraw_queued || self.redraw_echo_queued {
            let mut context = c.clone();
            context.draw_state.scissor = Some(self.get_view_scissor());
            let view = self.view; // must copy b/c self is mutably borrowed in next line
            self.active_field().render(context, g, view);
            if self.redraw_queued {
                self.redraw_echo_queued = true;
                self.redraw_queued = false;
            } else {
                self.redraw_echo_queued = false;
            }
        }
    }

    fn update_view(&mut self, c: pw::Context) {
        if let Some(viewport) = c.viewport {
            self.window = viewport.window_size;
            let (w, h) = (viewport.window_size[0], viewport.window_size[1]);
            let x = util::f64_max(w as f64 * 0.3, CHROME_MIN_WIDTH as f64);
            self.view = [
                x,
                0.0,
                w as f64 - x,
                h as f64
            ];
        }
    }

    fn active_field(&mut self) -> &mut FieldView {
        match self.selected {
            FieldChoice::OneCharge => &mut self.fields.one_charge,
            FieldChoice::TwoCharges => &mut self.fields.two_charges,
            FieldChoice::Capacitor => &mut self.fields.capacitor,
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
        let mut queue_redraw = true;
        match key {
            pw::Key::Up => {
                self.active_field().transform_camera(util::euler_rot_mat4(PI * 0.01, 0.0, 0.0));
            },
            pw::Key::Down => {
                self.active_field().transform_camera(util::euler_rot_mat4(-PI * 0.01, 0.0, 0.0));
            },
            pw::Key::Right => {
                self.active_field().transform_camera(util::euler_rot_mat4(0.0, -PI * 0.01, 0.0));
            },
            pw::Key::Left => {
                self.active_field().transform_camera(util::euler_rot_mat4(0.0, PI * 0.01, 0.0));
            },
            pw::Key::W => {
                self.active_field().transform_camera(util::translation_mat4(na::Vector3::new(0.0, 0.0, -1.0)));
            },
            pw::Key::S => {
                self.active_field().transform_camera(util::translation_mat4(na::Vector3::new(0.0, 0.0, 1.0)));
            },
            pw::Key::D => {
                self.active_field().transform_camera(util::translation_mat4(na::Vector3::new(-1.0, 0.0, 0.0)));
            },
            pw::Key::A => {
                self.active_field().transform_camera(util::translation_mat4(na::Vector3::new(1.0, 0.0, 0.0)));
            },
            pw::Key::Q => {
                self.active_field().transform_camera(util::translation_mat4(na::Vector3::new(0.0, -1.0, 0.0)));
            },
            pw::Key::E => {
                self.active_field().transform_camera(util::translation_mat4(na::Vector3::new(0.0, 1.0, 0.0)));
            },
            pw::Key::I => {
                self.active_field().transform_arrows(util::euler_rot_mat4(PI * 0.01, 0.0, 0.0));
            },
            pw::Key::K => {
                self.active_field().transform_arrows(util::euler_rot_mat4(-PI * 0.01, 0.0, 0.0));
            },
            pw::Key::L => {
                self.active_field().transform_arrows(util::euler_rot_mat4(0.0, PI * 0.01, 0.0));
            },
            pw::Key::J => {
                self.active_field().transform_arrows(util::euler_rot_mat4(0.0, -PI * 0.01, 0.0));
            },
            pw::Key::T => {
                match self.selected {
                    FieldChoice::TwoCharges => {
                        self.fields.two_charges.charges[0].loc.y -= CHARGE_MVMT_STEP;
                        self.rebuild_queued = true;
                    },
                    _ => {},
                }
            },
            pw::Key::G => {
                match self.selected {
                    FieldChoice::TwoCharges => {
                        self.fields.two_charges.charges[0].loc.y += CHARGE_MVMT_STEP;
                        self.rebuild_queued = true;
                    },
                    _ => {},
                }
            },
            pw::Key::H => {
                match self.selected {
                    FieldChoice::TwoCharges => {
                        self.fields.two_charges.charges[0].loc.x += CHARGE_MVMT_STEP;
                        self.rebuild_queued = true;
                    },
                    _ => {},
                }
            },
            pw::Key::F => {
                match self.selected {
                    FieldChoice::TwoCharges => {
                        self.fields.two_charges.charges[0].loc.x -= CHARGE_MVMT_STEP;
                        self.rebuild_queued = true;
                    },
                    _ => {},
                }
            },
            pw::Key::R => {
                match self.selected {
                    FieldChoice::TwoCharges => {
                        self.fields.two_charges.charges[0].loc.z -= CHARGE_MVMT_STEP;
                        self.rebuild_queued = true;
                    },
                    _ => {},
                }
            },
            pw::Key::Y => {
                match self.selected {
                    FieldChoice::TwoCharges => {
                        self.fields.two_charges.charges[0].loc.z += CHARGE_MVMT_STEP;
                        self.rebuild_queued = true;
                    },
                    _ => {},
                }
            },
            _ => {
                queue_redraw = false;
            },
        }
        if queue_redraw {
            self.redraw_queued = queue_redraw;
        }
    }

    fn set_widgets(&mut self) {
        let h = self.window[1] as f64;
        let fields = &mut self.fields;
        let view = self.view;
        let mut queue_rebuild = false;
        let mut queue_redraw = false;
        let mut selected_field = self.selected;
        self.ui.set_widgets(|ref mut ui: UiCell| {
            use conrod::{color, Widget, Canvas, Text, Slider, Sizeable, Colorable, Positionable, Frameable};
            Canvas::new().flow_down(&[
                    (HEADER, Canvas::new().length(BANNER_HEIGHT).color(color::CHARCOAL).frame(0.0)),
                    (CONTENT, Canvas::new().length(h - BANNER_HEIGHT).flow_right(&[
                        (BODY_LEFT, Canvas::new().color(color::DARK_CHARCOAL).length(CHROME_PAD as f64).frame(0.0)),
                        (BODY, Canvas::new().color(color::DARK_CHARCOAL).length(view[0] - CHROME_PAD as f64).pad_top(CHROME_PAD as f64).frame(0.0))
                    ]).color(color::TRANSPARENT))
                ]).top_left().color(color::TRANSPARENT).w(view[0]).set(CANVAS, ui);
            Text::new(TITLE)
                .color(color::WHITE)
                .font_size(BANNER_FONT_SIZE)
                .mid_left_with_margin_on(HEADER, 5.0)
                .set(TITLE_TEXT, ui);

            // Instructions
            description_top("Controls:
  - WASD,QE to move the camera
  - arrow keys to look around
  - IJKL to rotate field").set(INSTRUCTIONS, ui);

            // Buttons for selecting type of field
            description("Choose field:", INSTRUCTIONS).h(15.0).set(CHOOSE_TEXT, ui);
            field_btn_top("One charge", CHOOSE_TEXT, selected_field == FieldChoice::OneCharge)
                .react(|| {
                    selected_field = FieldChoice::OneCharge;
                    queue_redraw = true;
                }).set(FIELDBTN_ONE, ui);
            field_btn("Two charges", FIELDBTN_ONE, selected_field == FieldChoice::TwoCharges)
                .react(|| {
                    selected_field = FieldChoice::TwoCharges;
                    queue_redraw = true;
                }).set(FIELDBTN_TWO, ui);
            field_btn("Capacitor", FIELDBTN_TWO, selected_field == FieldChoice::Capacitor)
                .react(|| {
                    selected_field = FieldChoice::Capacitor;
                    queue_redraw = true;
                }).set(FIELDBTN_CAP, ui);

            // Controls
            match selected_field {
                FieldChoice::OneCharge => {
                },
                FieldChoice::TwoCharges => {
                    let field = &mut fields.two_charges;
                    description("Set magnitudes of charges:", FIELDBTN_TWO).set(SLIDER_INTRO, ui);
                    // Label and slider for left charge value
                    let value0 = field.charges[1].charge;
                    slider!(
                        ids[SLIDER0, SLIDER0_LC, SLIDER0_SC, SLIDER0_L, SLIDER0_S],
                        above = SLIDER_INTRO,
                        top = false,
                        view = view,
                        ui = ui,
                        value = value0,
                        range = [0.0, -10.0],
                        text = "Left charge: ",
                        react = |c: f64| {
                            field.charges[1].charge = c;
                            queue_rebuild = true;
                        }
                    );
                    // Label and slider for right charge value
                    let value1 = field.charges[0].charge;
                    slider!(
                        ids[SLIDER1, SLIDER1_LC, SLIDER1_SC, SLIDER1_L, SLIDER1_S],
                        above = SLIDER0,
                        top = false,
                        view = view,
                        ui = ui,
                        value = value1,
                        range = [0.0, 10.0],
                        text = "Right charge: ",
                        react = |c: f64| {
                            field.charges[0].charge = c;
                            queue_rebuild = true;
                        }
                    );
                    description("Use TFGH,RY to move the positive charge around", SLIDER1).set(TFGHRY_DESCRIPTION, ui);
                },
                FieldChoice::Capacitor => {
                },
            }
        });
        if queue_redraw {
            self.redraw_queued = true;
        }
        if queue_rebuild {
            self.rebuild_queued = true;
        }
        self.selected = selected_field;
    }
}

fn field_btn_top<F: FnOnce()>(t: &str, above: conrod::WidgetId, active: bool) -> conrod::Button<F> {
    use conrod::Positionable;
    field_btn(t, above, active).down_from(above, 10.0)
}
fn field_btn<F: FnOnce()>(t: &str, above: conrod::WidgetId, active: bool) -> conrod::Button<F> {
    use conrod::{color, Button, Colorable, Labelable, Positionable, Sizeable};
    let btn = Button::new().label(t).h(18.0).down_from(above, 5.0);
    if active {
        btn.color(color::BLACK).label_color(color::WHITE)
    } else {
        btn
    }
}

fn description_top(t: &str) -> conrod::Text {
    use conrod::{color, Text, Colorable, Sizeable, Positionable};
    Text::new(t)
        .color(color::WHITE)
        .w_of(BODY)
        .mid_top_of(BODY)
}

fn description(t: &str, above: conrod::WidgetId) -> conrod::Text {
    use conrod::{color, Text, Colorable, Sizeable, Positionable};
    Text::new(t)
        .color(color::WHITE)
        .w_of(BODY)
        .down_from(above, 30.0)
}

widget_ids! {
    CANVAS,
    HEADER,
    TITLE_TEXT,
    CONTENT,
    BODY_LEFT,
    BODY,
    INSTRUCTIONS,
    CHOOSE_TEXT,
    FIELDBTN_ONE,
    FIELDBTN_TWO,
    FIELDBTN_CAP,
    SLIDER_INTRO,
    SLIDER0,
    SLIDER0_LC,
    SLIDER0_SC,
    SLIDER0_L,
    SLIDER0_S,
    SLIDER1,
    SLIDER1_LC,
    SLIDER1_SC,
    SLIDER1_L,
    SLIDER1_S,
    TFGHRY_DESCRIPTION,
}
