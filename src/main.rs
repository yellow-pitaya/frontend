#![warn(warnings)]

mod application;
mod color;
mod scales;
mod widget;

use clap::Parser;
use color::Color;
use scales::Scales;

#[derive(Parser)]
struct Opt {
    #[clap(default_value = "127.0.0.1:5000")]
    addr: String,
}

fn main() {
    envir::init();

    let opt = Opt::parse();

    let redpitaya = redpitaya_scpi::Redpitaya::new(opt.addr);

    let app = relm4::RelmApp::new("com.yellow-pitaya.frontend").with_args(Vec::new());
    app.run::<application::Model>(redpitaya);
}
