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

fn main() {
    let redpitaya = Redpitaya::new("192.168.1.5", 5000);

    let display = glium::glutin::WindowBuilder::new()
        .with_title("Redpitaya")
        .build_glium()
        .unwrap();

    let mut ui = conrod::UiBuilder::new([400.0, 200.0])
        .build();

    let mut renderer = conrod::backend::glium::Renderer::new(&display)
        .unwrap();

    let image_map = conrod::image::Map::<glium::texture::Texture2d>::new();

    'main: loop {
        for event in display.poll_events() {
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
