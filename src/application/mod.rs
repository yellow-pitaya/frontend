use conrod::{Borderable, Sizeable, Positionable, Colorable, Labelable, Widget};
use conrod::backend::glium::glium::Surface;
use glium::DisplayBuild;

mod event;

widget_ids! {
    struct Ids {
        master,
        main_panel,
        side_panel,
        side_panel_tabs,
        oscillo_panel,
        generator_panel,
        toggle_oscillo,
        toggle_generator,
        toggle_generator_sine,
        toggle_generator_img_sine,
        toggle_generator_square,
        toggle_generator_img_square,
        toggle_generator_triangle,
        toggle_generator_img_triangle,
        toggle_generator_sawu,
        toggle_generator_img_sawu,
        toggle_generator_sawd,
        toggle_generator_img_sawd,
        toggle_generator_pwm,
        toggle_generator_img_pwm,
        text_generator_amplitude,
        text_generator_frequency,
        text_generator_dcyc,
        scales,
        lines[],
        points,
    }
}

pub struct Application {
    redpitaya: ::backend::Redpitaya,
    bg_color: ::conrod::color::Color,
    width: f64,
    height: f64,
    scales: [(f64, f64); 2],
}

impl Application {
    pub fn new(redpitaya: ::backend::Redpitaya) -> Application {
        Application {
            redpitaya: redpitaya,
            bg_color: ::conrod::color::rgb(0.2, 0.35, 0.45),
            width: 400.0,
            height: 200.0,
            scales: [
                (0.0, 16384.0),
                (-5.0, 5.0),
            ],
        }
    }

    pub fn run(&mut self) {
        let display = ::glium::glutin::WindowBuilder::new()
            .with_title("Redpitaya")
            .build_glium()
            .unwrap();

        let mut ui = ::conrod::UiBuilder::new([self.width, self.height])
            .build();

        ui.fonts.insert_from_file("assets/fonts/NotoSans/NotoSans-Regular.ttf")
            .unwrap();

        let mut ids = Ids::new(ui.widget_id_generator());
        ids.lines.resize(20, &mut ui.widget_id_generator());

        let mut renderer = ::conrod::backend::glium::Renderer::new(&display)
            .unwrap();

        let image_map = ::conrod::image::Map::<::glium::texture::Texture2d>::new();

        let mut event_loop = event::Loop::new();
        'main: loop {
            for event in event_loop.next(&display) {
                if let Some(event) = ::conrod::backend::winit::convert(event.clone(), &display) {
                    ui.handle_event(event);
                    event_loop.needs_update();
                }

                match event {
                    ::glium::glutin::Event::Closed => break 'main,
                    _ => {},
                }
            }

            self.set_widgets(ui.set_widgets(), &mut ids);

            if let Some(primitives) = ui.draw_if_changed() {
                renderer.fill(&display, primitives, &image_map);
                let mut target = display.draw();
                target.clear_color(0.0, 0.0, 0.0, 1.0);
                renderer.draw(&display, &mut target, &image_map)
                    .unwrap();
                target.finish()
                    .unwrap();
            }
        }

        self.redpitaya.acquire.stop();
        self.redpitaya.generator.stop();
    }

    fn set_widgets(&mut self, ref mut ui: ::conrod::UiCell, ids: &mut Ids) {
        let main_panel = ::conrod::widget::Canvas::new();
        let side_panel = ::conrod::widget::Canvas::new()
            .length(400.0);

        ::conrod::widget::Canvas::new()
            .flow_right(&[
                (ids.main_panel, main_panel),
                (ids.side_panel, side_panel),
            ])
            .color(self.bg_color)
            .set(ids.master, ui);

        self.main_panel(ui, ids);
        self.side_panel(ui, ids);
    }

    fn main_panel(&mut self, ui: &mut ::conrod::UiCell, ids: &mut Ids) {
        self.draw_scales(ui, ids);

        if self.redpitaya.acquire.is_started() {
            let message = self.redpitaya.acquire.get_data();

            let mut data = message
                .trim_matches(|c: char| c == '{' || c == '}' || c == '!' || c.is_alphabetic())
                .split(",")
                .map(|s| {
                    match s.parse::<f64>() {
                        Ok(f) => f,
                        Err(_) => {
                            error!("Invalid data '{}'", s);
                            0.0
                        },
                    }
                });

            self.draw_data(&mut data, ui, ids);
        }
    }

    fn draw_scales(&mut self, ui: &mut ::conrod::UiCell, ids: &Ids) {
        ::conrod::widget::Canvas::new()
            .border(1.0)
            .border_color(self.bg_color.invert())
            .wh_of(ids.main_panel)
            .top_left_of(ids.main_panel)
            .border(1.0)
            .set(ids.scales, ui);

        let canvas = ui.rect_of(ids.scales)
            .unwrap();

        for i in 1..10 {
            let x = canvas.x.start + (canvas.x.end - canvas.x.start) / 10.0 * (i as f64);

            let mut color = ::conrod::color::WHITE;

            if i != 5 {
                color = color.alpha(0.1);
            }

            ::conrod::widget::Line::new(
                [x, canvas.y.start],
                [x, canvas.y.end]
            )
                .color(color)
                .bottom_left_of(ids.scales)
                .set(ids.lines[i], ui);

            let y = canvas.y.start + (canvas.y.end - canvas.y.start) / 10.0 * (i as f64);

            ::conrod::widget::Line::new(
                [canvas.x.start, y],
                [canvas.x.end, y]
            )
                .color(color)
                .bottom_left_of(ids.scales)
                .set(ids.lines[i + 9], ui);
        }
    }

    fn draw_data<I>(&mut self, data: &mut I, ui: &mut ::conrod::UiCell, ids: &mut Ids) where I: Iterator<Item=f64> {
        let rect = ui.rect_of(ids.scales)
            .unwrap();

        let point_iter = (0..16384).map(|x| {
            match data.next() {
                Some(y) => self.scale(rect, x as f64, y),
                None => [0.0, 0.0],
            }
        });

        ::conrod::widget::PointPath::new(point_iter)
            .wh_of(ids.scales)
            .middle_of(ids.scales)
            .color(::conrod::color::YELLOW)
            .parent(ids.scales)
            .graphics_for(ids.scales)
            .set(ids.points, ui);
    }

    fn scale(&self, rect: ::conrod::position::rect::Rect, x: f64, y: f64) -> ::conrod::position::Point {
        [
            self.scale_coord(x, self.scales[0], (rect.x.start, rect.x.end)),
            self.scale_coord(y, self.scales[1], (rect.y.start, rect.y.end)),
        ]
    }

    fn scale_coord(&self, x: f64, from: (f64, f64), to: (f64, f64)) -> f64 {
        ((x - from.0) / (from.1 - from.0)) * (to.1 - to.0) + to.0
    }

    fn side_panel(&mut self, ui: &mut ::conrod::UiCell, ids: &Ids) {
        ::conrod::widget::Tabs::new(&[
            (ids.oscillo_panel, "Oscilloscope"),
            (ids.generator_panel, "Generator"),
        ])
            .starting_canvas(ids.oscillo_panel)
            .wh_of(ids.side_panel)
            .middle_of(ids.side_panel)
            .color(self.bg_color.complement())
            .set(ids.side_panel_tabs, ui);

        self.oscillo_run_button(ui, ids);

        self.generator_run_button(ui, ids);
        self.generator_sine_button(ui, ids);
        self.generator_square_button(ui, ids);
        self.generator_triangle_button(ui, ids);
        self.generator_sawu_button(ui, ids);
        self.generator_sawd_button(ui, ids);
        self.generator_pwm_button(ui, ids);
        self.generator_amplitude(ui, ids);
        self.generator_frequency(ui, ids);

        if self.redpitaya.generator.get_form() == "pwm" {
            self.generator_dcyc(ui, ids);
        }
    }

    fn oscillo_run_button(&mut self, ui: &mut ::conrod::UiCell, ids: &Ids) {
        let label = match self.redpitaya.acquire.is_started() {
            true => "Stop",
            false => "Run",
        };

        let toggle = ::conrod::widget::Toggle::new(self.redpitaya.acquire.is_started())
            .w_h(100.0, 50.0)
            .middle_of(ids.oscillo_panel)
            .color(self.bg_color.plain_contrast())
            .label(label)
            .label_color(self.bg_color)
            .set(ids.toggle_oscillo, ui);

        if let Some(value) = toggle.last() {
            if value {
                self.redpitaya.acquire.reset();
                self.redpitaya.acquire.set_decimation(1);
                self.redpitaya.trigger.set_level(0);
                self.redpitaya.acquire.start();
                self.redpitaya.trigger.enable("CH1_PE");
                self.redpitaya.acquire.set_units("VOLTS");
            } else {
                self.redpitaya.acquire.stop();
            }
        }
    }

    fn generator_run_button(&mut self, ui: &mut ::conrod::UiCell, ids: &Ids) {
        let label = match self.redpitaya.generator.is_started() {
            true => "Stop",
            false => "Run",
        };

        let toggle = ::conrod::widget::Toggle::new(self.redpitaya.generator.is_started())
            .w_h(100.0, 50.0)
            .mid_top_of(ids.generator_panel)
            .color(self.bg_color.plain_contrast())
            .label(label)
            .label_color(self.bg_color)
            .set(ids.toggle_generator, ui);

        if let Some(value) = toggle.last() {
            if value {
                self.redpitaya.generator.start();
            } else {
                self.redpitaya.generator.stop();
            }
        }
    }

    fn generator_sine_button(&mut self, ui: &mut ::conrod::UiCell, ids: &Ids) {
        self.generator_button("sine", ui, ids.toggle_generator, ids.toggle_generator_sine, ids.toggle_generator_img_sine, f64::sin);
    }

    fn generator_square_button(&mut self, ui: &mut ::conrod::UiCell, ids: &Ids) {
        self.generator_button("square", ui, ids.toggle_generator_sine, ids.toggle_generator_square, ids.toggle_generator_img_square, |x| {
            (x as f64).signum()
        });
    }

    fn generator_triangle_button(&mut self, ui: &mut ::conrod::UiCell, ids: &Ids) {
        self.generator_button("triangle", ui, ids.toggle_generator_square, ids.toggle_generator_triangle, ids.toggle_generator_img_triangle, |x| {
            if x.is_sign_negative() {
                x * 2.0 / ::std::f64::consts::PI + 1.0
            } else {
                x * -2.0 / ::std::f64::consts::PI + 1.0
            }
        });
    }

    fn generator_sawu_button(&mut self, ui: &mut ::conrod::UiCell, ids: &Ids) {
        self.generator_button("sawu", ui, ids.toggle_generator_triangle, ids.toggle_generator_sawu, ids.toggle_generator_img_sawu, |x| {
            x / ::std::f64::consts::PI
        });
    }

    fn generator_sawd_button(&mut self, ui: &mut ::conrod::UiCell, ids: &Ids) {
        self.generator_button("sawd", ui, ids.toggle_generator_sawu, ids.toggle_generator_sawd, ids.toggle_generator_img_sawd, |x| {
            -x / ::std::f64::consts::PI
        });
    }

    fn generator_pwm_button(&mut self, ui: &mut ::conrod::UiCell, ids: &Ids) {
        self.generator_button("pwm", ui, ids.toggle_generator_sawd, ids.toggle_generator_pwm, ids.toggle_generator_img_pwm, |x| {
            if x.is_sign_negative() {
                if x.abs().fract() > 0.5 {
                    1.0
                } else {
                    -1.0
                }
            } else {
                if x.abs().fract() > 0.5 {
                    -1.0
                } else {
                    1.0
                }
            }
        });
    }

    fn generator_button<F>(&mut self, name: &str, ui: &mut ::conrod::UiCell, parent: ::conrod::widget::Id, id: ::conrod::widget::Id, img_id: ::conrod::widget::Id, f: F)
        where F: Fn(f64) -> f64 {
        let active = self.redpitaya.generator.get_form() == name;

        let toggle = ::conrod::widget::Toggle::new(active)
            .w_h(100.0, 50.0)
            .down_from(parent, 10.0)
            .color(self.bg_color.plain_contrast())
            .label_color(self.bg_color)
            .set(id, ui);

        ::conrod::widget::PlotPath::new(-::std::f64::consts::PI, ::std::f64::consts::PI, -1.0, 1.0, f)
            .padded_wh_of(id, 10.0)
            .middle_of(id)
            .parent(id)
            .graphics_for(id)
            .set(img_id, ui);

        if let Some(value) = toggle.last() {
            if value {
                self.redpitaya.generator.set_form(name);
            }
        }
    }

    fn generator_amplitude(&mut self, ui: &mut ::conrod::UiCell, ids: &Ids) {
        let amplitude = self.redpitaya.generator.get_amplitude();

        let slider = ::conrod::widget::Slider::new(amplitude, -1.0, 1.0)
            .w_h(200.0, 50.0)
            .down_from(ids.toggle_generator_pwm, 10.0)
            .color(self.bg_color.plain_contrast())
            .label_color(self.bg_color)
            .label(format!("{} V", amplitude).as_str())
            .set(ids.text_generator_amplitude, ui);

        if let Some(value) = slider {
            self.redpitaya.generator.set_amplitude(value);
        }
    }

    fn generator_frequency(&mut self, ui: &mut ::conrod::UiCell, ids: &Ids) {
        let frequency = self.redpitaya.generator.get_frequency();

        let slider = ::conrod::widget::Slider::new(frequency as f32, 0.0, 62_500_000.0)
            .w_h(200.0, 50.0)
            .down_from(ids.text_generator_amplitude, 10.0)
            .color(self.bg_color.plain_contrast())
            .label_color(self.bg_color)
            .label(format!("{} Hz", frequency as u32).as_str())
            .set(ids.text_generator_frequency, ui);

        if let Some(value) = slider {
            self.redpitaya.generator.set_frequency(value as u32);
        }
    }

    fn generator_dcyc(&mut self, ui: &mut ::conrod::UiCell, ids: &Ids) {
        let dcyc = self.redpitaya.generator.get_dcyc();

        let slider = ::conrod::widget::Slider::new(dcyc as f32, 0.0, 100.0)
            .w_h(200.0, 50.0)
            .down_from(ids.text_generator_frequency, 10.0)
            .color(self.bg_color.plain_contrast())
            .label_color(self.bg_color)
            .label(format!("{} Hz", dcyc as u32).as_str())
            .set(ids.text_generator_dcyc, ui);

        if let Some(value) = slider {
            self.redpitaya.generator.set_dcyc(value as u32);
        }
    }
}
