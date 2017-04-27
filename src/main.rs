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

#[derive(Copy, Clone)]
struct Scales {
    h: (f64, f64),
    v: (f64, f64),
}

impl Scales {
    pub fn get_width(&self) -> f64 {
        self.h.1 - self.h.0
    }

    pub fn get_height(&self) -> f64 {
        self.v.1 - self.v.0
    }

    pub fn x(&self) -> ::std::ops::Range<u64> {
        ::std::ops::Range {
            start: self.h.0 as u64,
            end: self.h.1 as u64,
        }
    }

    pub fn from_decimation(&mut self, decimation: ::redpitaya_scpi::acquire::Decimation) {
        let duration = decimation.get_buffer_duration();
        let h1 = (duration.as_secs() * 1_000_000 + duration.subsec_nanos() as u64 / 1_000) as f64;

        self.h.1 = h1;
    }
}

fn main() {
    env_logger::init()
        .unwrap();

    application::Application::run();
}
