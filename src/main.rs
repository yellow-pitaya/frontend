#![feature(proc_macro)]

extern crate cairo;
extern crate env_logger;
extern crate glib;
extern crate gdk_sys;
extern crate gtk;
#[macro_use]
extern crate relm;
extern crate relm_attributes;
#[macro_use]
extern crate relm_derive;
#[macro_use]
extern crate log;
extern crate redpitaya_scpi;

mod application;
mod widget;

use ::relm::Widget;

#[derive(Copy, Clone)]
struct Scales {
    h: (f64, f64),
    v: (f64, f64),
    pub n_samples: u32,
}

impl Scales {
    pub fn get_width(&self) -> f64 {
        self.h.1 - self.h.0
    }

    pub fn get_height(&self) -> f64 {
        self.v.1 - self.v.0
    }

    pub fn from_decimation(&mut self, decimation: ::redpitaya_scpi::acquire::Decimation) {
        let duration = decimation.get_buffer_duration();
        let h1 = (duration.as_secs() * 1_000_000 + duration.subsec_nanos() as u64 / 1_000) as f64;

        self.h.1 = h1;
    }

    pub fn v_div(&self) -> f64 {
        (self.v.1 - self.v.0) / 10.0
    }

    pub fn h_div(&self) -> f64 {
        (self.h.1 - self.h.0) / 10.0
    }

    pub fn sample_to_ms(&self, sample: u32) -> f64 {
        sample as f64 / self.n_samples as f64 * self.h.1
    }
}

fn main() {
    env_logger::init()
        .unwrap();

    let redpitaya = ::redpitaya_scpi::Redpitaya::new("192.168.1.5:5000");

    application::Application::run(redpitaya)
        .unwrap();
}
