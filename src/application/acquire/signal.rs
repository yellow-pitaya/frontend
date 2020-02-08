#[derive(relm_derive::Msg, Clone)]
pub enum Signal {
    Attenuation(redpitaya_scpi::acquire::Source, u8),
    Average(bool),
    Gain(redpitaya_scpi::acquire::Source, redpitaya_scpi::acquire::Gain),
    Rate(redpitaya_scpi::acquire::SamplingRate),
    SetData(redpitaya_scpi::acquire::Source, Vec<f64>),
    Start(redpitaya_scpi::acquire::Source),
    Stop(redpitaya_scpi::acquire::Source),
    Redraw(cairo::Context, crate::application::Model),
}
