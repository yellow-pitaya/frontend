extern crate env_logger;
#[macro_use]
extern crate conrod;
extern crate glium;
#[macro_use]
extern crate log;
extern crate redpitaya_scpi;

mod application;

use redpitaya_scpi::Redpitaya;
use application::Application;

fn main() {
    env_logger::init()
        .unwrap();

    let redpitaya = Redpitaya::new("192.168.1.5:5000");

    Application::new(redpitaya)
        .run();
}
