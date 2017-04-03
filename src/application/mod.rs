use conrod::{Borderable, Sizeable, Positionable, Colorable, Labelable, Widget};
use conrod::backend::glium::glium::Surface;
use glium::DisplayBuild;

mod event;

widget_ids! {
    struct Ids {
        canvas,
        toggle_oscillo,
        toggle_generator,
        toggle_generator_img,
        plot,
    }
}

pub struct Application {
    oscillo_started: bool,
    generator_started: bool,
    tx: ::std::sync::mpsc::Sender<String>,
    rx: ::std::sync::mpsc::Receiver<String>,
    bg_color: ::conrod::color::Color,
}

impl Application {
    pub fn new(tx: ::std::sync::mpsc::Sender<String>, rx: ::std::sync::mpsc::Receiver<String>) -> Application {
        Application {
            oscillo_started: false,
            generator_started: false,
            tx: tx,
            rx: rx,
            bg_color: ::conrod::color::rgb(0.2, 0.35, 0.45),
        }
    }

    pub fn run(&mut self) {
        let display = ::glium::glutin::WindowBuilder::new()
            .with_title("Redpitaya")
            .build_glium()
            .unwrap();

        let mut ui = ::conrod::UiBuilder::new([400.0, 200.0])
            .build();

        ui.fonts.insert_from_file("assets/fonts/NotoSans/NotoSans-Regular.ttf")
            .unwrap();

        let ids = Ids::new(ui.widget_id_generator());

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
        ::conrod::widget::Canvas::new()
            .pad(30.0)
            .color(self.bg_color)
            .set(ids.canvas, ui);

        self.oscillo_run_button(ui, ids);
        self.generator_sin_button(ui, ids);

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

    fn oscillo_run_button(&mut self, ui: &mut ::conrod::UiCell, ids: &Ids) {
        let label = match self.oscillo_started {
            true => "Stop",
            false => "Run",
        };

        let toggle = ::conrod::widget::Toggle::new(self.oscillo_started)
            .w_h(100.0, 50.0)
            .mid_right_of(ids.canvas)
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
            .down_from(ids.toggle_oscillo, 10.0)
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

    fn draw_data<I>(&mut self, data: I, ui: &mut ::conrod::UiCell, ids: &Ids) where I: Iterator<Item=f64> {
        let data: Vec<f64> = data.collect();

        let plot = ::conrod::widget::PlotPath::new(0, data.len() - 1, -2.0, 2.0, |x| {
            return data[x];
        });

        plot.color(::conrod::color::LIGHT_BLUE)
            .padded_w_of(ids.canvas, 100.0)
            .padded_h_of(ids.canvas, 30.0)
            .top_left_of(ids.canvas)
            .set(ids.plot, ui);
    }
}
