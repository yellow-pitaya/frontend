extern crate env_logger;
#[macro_use]
extern crate conrod;
extern crate glium;
#[macro_use]
extern crate log;

mod backend;
mod application;

use backend::Redpitaya;
use application::Application;

fn main() {
    env_logger::init()
        .unwrap();

    let redpitaya = Redpitaya::new("192.168.1.5", 5000);

    Application::new(redpitaya)
        .run();
}
