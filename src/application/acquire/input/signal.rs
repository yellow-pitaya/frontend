#[derive(relm_derive::Msg, Clone)]
pub enum Signal {
    Attenuation(u8),
    Gain(redpitaya_scpi::acquire::Gain),
    SetData(Vec<f64>),
    Start,
    Stop,
    Redraw(cairo::Context, crate::application::Model),
}
