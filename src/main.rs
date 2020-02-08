extern crate cairo;
extern crate env_logger;
extern crate glib;
extern crate gdk;
extern crate gtk;
#[macro_use]
extern crate log;
#[macro_use]
extern crate relm;
#[macro_use]
extern crate relm_derive;
extern crate redpitaya_scpi;
extern crate structopt;

mod application;
mod color;
mod scales;
mod widget;

use relm::Widget;
use scales::Scales;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Opt
{
    addr: String,
}

fn main() {
    env_logger::init();

    let opt = ::Opt::from_args();

    let redpitaya = ::redpitaya_scpi::Redpitaya::new(opt.addr);

    application::Widget::run(redpitaya)
        .unwrap();
}

fn create_context(widget: &::gtk::DrawingArea) -> ::cairo::Context {
    let mut draw_handler = relm::DrawHandler::new()
        .expect("draw handler");

    draw_handler.init(widget);

    draw_handler.get_context()
        .clone()
}
