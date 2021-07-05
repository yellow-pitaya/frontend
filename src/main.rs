#![warn(rust_2018_idioms)]

mod application;
mod color;
mod scales;
mod widget;

use relm::Widget;
use scales::Scales;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Opt {
    addr: String,
}

fn main() {
    env_logger::init();

    let opt = Opt::from_args();

    let redpitaya = redpitaya_scpi::Redpitaya::new(opt.addr);

    application::Widget::run(redpitaya).unwrap();
}

fn create_context(widget: &gtk::DrawingArea) -> Result<gtk::cairo::Context, gtk::cairo::Error> {
    let mut draw_handler = relm::DrawHandler::new().expect("draw handler");

    draw_handler.init(widget);

    draw_handler.get_context().map(|x| x.clone())
}
