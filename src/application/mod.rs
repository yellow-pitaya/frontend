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
        toggle_generator_img,
        scales,
        plot,
        lines[],
    }
}

pub struct Application {
    oscillo_started: bool,
    generator_started: bool,
    tx: ::std::sync::mpsc::Sender<String>,
    rx: ::std::sync::mpsc::Receiver<String>,
    bg_color: ::conrod::color::Color,
    width: f64,
    height: f64,
}

impl Application {
    pub fn new(tx: ::std::sync::mpsc::Sender<String>, rx: ::std::sync::mpsc::Receiver<String>) -> Application {
        Application {
            oscillo_started: false,
            generator_started: false,
            tx: tx,
            rx: rx,
            bg_color: ::conrod::color::rgb(0.2, 0.35, 0.45),
            width: 400.0,
            height: 200.0,
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

            self.set_widgets(ui.set_widgets(), &ids);

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

        self.tx.send("oscillo/stop".into());
        self.tx.send("generator/stop".into());
    }

    fn set_widgets(&mut self, ref mut ui: ::conrod::UiCell, ids: &Ids) {
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

    fn main_panel(&mut self, ui: &mut ::conrod::UiCell, ids: &Ids) {
        self.draw_scales(ui, ids);

        if self.oscillo_started {
            self.tx.send("oscillo/data".into());
            if let Ok(message) = self.rx.recv() {
                let data = message
                    .trim_matches(|c| c == '{' || c == '}')
                    .split(",")
                    .map(|s| s.parse::<f64>().unwrap());

                self.draw_data(data, ui, ids);
            }
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

    fn draw_data<I>(&mut self, data: I, ui: &mut ::conrod::UiCell, ids: &Ids) where I: Iterator<Item=f64> {
        let data: Vec<f64> = data.collect();

        let plot = ::conrod::widget::PlotPath::new(0, data.len() - 1, -2.0, 2.0, |x| {
            return data[x];
        });

        plot.color(::conrod::color::LIGHT_BLUE)
            .top_left_of(ids.scales)
            .wh_of(ids.scales)
            .set(ids.plot, ui);
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
        self.generator_sin_button(ui, ids);
    }

    fn oscillo_run_button(&mut self, ui: &mut ::conrod::UiCell, ids: &Ids) {
        let label = match self.oscillo_started {
            true => "Stop",
            false => "Run",
        };

        let toggle = ::conrod::widget::Toggle::new(self.oscillo_started)
            .w_h(100.0, 50.0)
            .middle_of(ids.oscillo_panel)
            .color(self.bg_color.plain_contrast())
            .label(label)
            .label_color(self.bg_color)
            .set(ids.toggle_oscillo, ui);

        if let Some(value) = toggle.last() {
            if value {
                self.tx.send("oscillo/start".into());
            } else {
                self.tx.send("oscillo/stop".into());
            }

            self.oscillo_started = value;
        }
    }

    fn generator_sin_button(&mut self, ui: &mut ::conrod::UiCell, ids: &Ids) {
        let toggle = ::conrod::widget::Toggle::new(self.generator_started)
            .w_h(100.0, 50.0)
            .middle_of(ids.generator_panel)
            .label("Sin")
            .color(self.bg_color.plain_contrast())
            .label_color(self.bg_color)
            .set(ids.toggle_generator, ui);

        if let Some(value) = toggle.last() {
            if value {
                self.tx.send("generator/start".into());
            } else {
                self.tx.send("generator/stop".into());
            }

            self.generator_started = value;
        }
    }
}
