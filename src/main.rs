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
extern crate log;
extern crate redpitaya_scpi;

mod application;

fn main() {
    env_logger::init()
        .unwrap();

    application::Application::run();
}
