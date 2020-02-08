#[derive(relm_derive::Msg, Clone)]
pub enum Signal {
    Amplitude(redpitaya_scpi::generator::Source, f32),
    DutyCycle(redpitaya_scpi::generator::Source, f32),
    Frequency(redpitaya_scpi::generator::Source, u32),
    Offset(redpitaya_scpi::generator::Source, f32),
    Form(redpitaya_scpi::generator::Source, redpitaya_scpi::generator::Form),
    Start(redpitaya_scpi::generator::Source),
    Stop(redpitaya_scpi::generator::Source),
    Redraw(cairo::Context, crate::application::Model),
}
