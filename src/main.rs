extern crate env_logger;
#[macro_use]
extern crate conrod;
extern crate glium;
#[macro_use]
extern crate log;

mod redpitaya;
mod application;

use redpitaya::Redpitaya;
use application::Application;

fn main() {
    env_logger::init()
        .unwrap();

    let (redpitaya_tx, redpitaya_rx) = std::sync::mpsc::channel::<String>();
    let (application_tx, application_rx) = std::sync::mpsc::channel::<String>();

    let mut redpitaya = Redpitaya::new("192.168.1.5", 5000);

    std::thread::spawn(move || {
        for message in redpitaya_rx {
            match message.as_str() {
                "oscillo/start" => {
                    redpitaya.acquire_start();
                    redpitaya.acquire_set_units("VOLTS");
                },
                "oscillo/stop" => redpitaya.acquire_stop(),
                "oscillo/data" => if redpitaya.acquire_is_started() {
                    let data = redpitaya.get_data();

                    application_tx.send(data);
                },
                "generator/start" => redpitaya.generator_start(),
                "generator/stop" => redpitaya.generator_stop(),
                "generator/sinc" => redpitaya.generator_set_form("sine"),
                message => warn!("Invalid action: '{}'", message),
            };
        }
    });

    Application::new(redpitaya_tx, application_rx)
        .run();
}
