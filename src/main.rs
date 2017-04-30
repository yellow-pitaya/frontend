#![feature(proc_macro)]

extern crate cairo;
extern crate docopt;
extern crate env_logger;
extern crate glib;
extern crate gdk;
extern crate gdk_sys;
extern crate gtk;
extern crate gtk_sys;
#[macro_use]
extern crate log;
#[macro_use]
extern crate relm;
extern crate relm_attributes;
#[macro_use]
extern crate relm_derive;
extern crate redpitaya_scpi;
extern crate rustc_serialize;

mod application;
mod color;
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

    pub fn from_sampling_rate(&mut self, rate: ::redpitaya_scpi::acquire::SamplingRate) {
        let duration = rate.get_buffer_duration();
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

static USAGE: &'static str = "Usage: yellow-pitaya <addr>";

#[derive(RustcDecodable)]
struct Args
{
    arg_addr: String,
}

fn main() {
    env_logger::init()
        .unwrap();

    let docopt = match ::docopt::Docopt::new(USAGE) {
        Ok(docopt) => docopt,
        Err(error) => error.exit(),
    };

    let args: Args = match docopt.decode() {
        Ok(args) => args,
        Err(e) => e.exit(),
    };

    let redpitaya = ::redpitaya_scpi::Redpitaya::new(args.arg_addr);

    application::Application::run(redpitaya)
        .unwrap();
}
