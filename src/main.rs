#![warn(warnings)]

mod application;
mod color;
mod scales;
mod widget;

use clap::Parser;
use relm::Widget;
use scales::Scales;

#[derive(Parser)]
struct Opt {
    addr: String,
}

fn main() {
    env_logger::init();

    let opt = Opt::parse();

    let redpitaya = redpitaya_scpi::Redpitaya::new(opt.addr);

    application::Widget::run(redpitaya).unwrap();
}

fn create_context(widget: &gtk::DrawingArea) -> Result<gtk::cairo::Context, gtk::cairo::Error> {
    let mut draw_handler = relm::DrawHandler::new().expect("draw handler");

    draw_handler.init(widget);

    draw_handler.get_context().map(|x| x.clone())
}
