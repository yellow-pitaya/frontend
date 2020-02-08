#[derive(relm_derive::Msg, Clone)]
pub enum Signal {
    Amplitude(f32),
    DutyCycle(f32),
    Frequency(u32),
    Offset(f32),
    Form(redpitaya_scpi::generator::Form),
    Redraw(cairo::Context, crate::application::Model),
    Start,
    Stop,
}
