extern crate conrod;
extern crate glium;

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
        self.socket.write(
            format!("{}\r\n", command).as_bytes()
        );
    }

    fn receive(&mut self) -> String {
        let mut message = String::new();
        let mut reader = std::io::BufReader::new(self.socket.try_clone().unwrap());

        reader.read_line(&mut message);

        message.trim_right_matches("\r\n")
            .into()
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

struct Application {
}

impl Application {
    pub fn new() -> Application {
        Application {
        }
    }

    pub fn run(&self) {
        let display = glium::glutin::WindowBuilder::new()
            .with_title("Redpitaya")
            .build_glium()
            .unwrap();

        let mut ui = conrod::UiBuilder::new([400.0, 200.0])
            .build();

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
    }
}

fn main() {
    let redpitaya = Redpitaya::new("192.168.1.5", 5000);

    Application::new()
        .run();
}
