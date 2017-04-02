extern crate env_logger;
#[macro_use]
extern crate conrod;
extern crate glium;
#[macro_use]
extern crate log;

use conrod::backend::glium::glium::Surface;
use glium::DisplayBuild;
use std::io::prelude::*;

struct Redpitaya {
    socket: std::net::TcpStream,
}

impl Redpitaya {
    pub fn new(ip: &str, port: u16) -> Redpitaya {
        let socket = match std::net::TcpStream::connect((ip, port)) {
            Ok(socket) => socket,
            Err(_) => panic!("Unable to connect to {}:{}", ip, port),
        };

        Redpitaya {
            socket: socket,
        }
    }

    pub fn aquire_start(&mut self) {
        self.send("ACQ:START");
    }

    pub fn aquire_stop(&mut self) {
        self.send("ACQ:STOP");
    }

    pub fn aquire_reset(&mut self) {
        self.send("ACQ:RST");
    }

    fn send(&mut self, command: &str) {
        info!("> {}", command);

        self.socket.write(
            format!("{}\r\n", command).as_bytes()
        );
    }

    fn receive(&mut self) -> String {
        let mut message = String::new();
        let mut reader = std::io::BufReader::new(self.socket.try_clone().unwrap());

        reader.read_line(&mut message);

        let message = message.trim_right_matches("\r\n");

        debug!("< {}", message);

        message.into()
    }
}

pub struct EventLoop {
    ui_needs_update: bool,
    last_update: std::time::Instant,
}

impl EventLoop {
    pub fn new() -> Self {
        EventLoop {
            last_update: std::time::Instant::now(),
            ui_needs_update: true,
        }
    }

    pub fn next(&mut self, display: &glium::Display) -> Vec<glium::glutin::Event> {
        // We don't want to loop any faster than 60 FPS, so wait until it has been at least 16ms
        // since the last yield.
        let last_update = self.last_update;
        let sixteen_ms = std::time::Duration::from_millis(16);
        let duration_since_last_update = std::time::Instant::now().duration_since(last_update);
        if duration_since_last_update < sixteen_ms {
            std::thread::sleep(sixteen_ms - duration_since_last_update);
        }

        // Collect all pending events.
        let mut events = Vec::new();
        events.extend(display.poll_events());

        // If there are no events and the `Ui` does not need updating, wait for the next event.
        if events.is_empty() && !self.ui_needs_update {
            events.extend(display.wait_events().next());
        }

        self.ui_needs_update = false;
        self.last_update = std::time::Instant::now();

        events
    }

    pub fn needs_update(&mut self) {
        self.ui_needs_update = true;
    }
}

widget_ids! {
    struct Ids {
        canvas,
        toggle,
    }
}

struct Application {
    started: bool,
    tx: std::sync::mpsc::Sender<String>,
}

impl Application {
    pub fn new(tx: std::sync::mpsc::Sender<String>) -> Application {
        Application {
            started: false,
            tx: tx,
        }
    }

    pub fn run(&mut self) {
        let display = glium::glutin::WindowBuilder::new()
            .with_title("Redpitaya")
            .build_glium()
            .unwrap();

        let mut ui = conrod::UiBuilder::new([400.0, 200.0])
            .build();

        ui.fonts.insert_from_file("assets/fonts/NotoSans/NotoSans-Regular.ttf")
            .unwrap();

        let ids = Ids::new(ui.widget_id_generator());

        let mut renderer = conrod::backend::glium::Renderer::new(&display)
            .unwrap();

        let image_map = conrod::image::Map::<glium::texture::Texture2d>::new();

        let mut event_loop = EventLoop::new();
        'main: loop {
            for event in event_loop.next(&display) {
                if let Some(event) = conrod::backend::winit::convert(event.clone(), &display) {
                    ui.handle_event(event);
                    event_loop.needs_update();
                }

                match event {
                    glium::glutin::Event::Closed => break 'main,
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
    }

    fn set_widgets(&mut self, ref mut ui: conrod::UiCell, ids: &Ids) {
        use conrod::{Sizeable, Positionable, Colorable, Labelable, Widget};

        let bg_color = conrod::color::rgb(0.2, 0.35, 0.45);

        conrod::widget::Canvas::new()
            .pad(30.0)
            .color(bg_color)
            .set(ids.canvas, ui);

        let label = match self.started {
            true => "Stop",
            false => "Run",
        };

        let toggle = conrod::widget::Toggle::new(self.started)
            .w_h(200.0, 50.0)
            .mid_right_of(ids.canvas)
            .color(bg_color.plain_contrast())
            .label(label)
            .label_color(bg_color)
            .set(ids.toggle, ui);

        if let Some(value) = toggle.last() {
            if value {
                self.tx.send("oscillo/start".into());
            } else {
                self.tx.send("oscillo/stop".into());
            }

            self.started = value;
        }
    }
}

fn main() {
    env_logger::init()
        .unwrap();

    let (tx, rx) = std::sync::mpsc::channel::<String>();

    let mut redpitaya = Redpitaya::new("192.168.1.5", 5000);

    std::thread::spawn(move || {
        for message in rx {
            match message.as_str() {
                "oscillo/start" => redpitaya.aquire_start(),
                "oscillo/stop" => redpitaya.aquire_stop(),
                message => warn!("Invalid action: '{}'", message),
            };
        }
    });

    Application::new(tx)
        .run();
}
