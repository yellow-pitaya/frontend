#[derive(Msg)]
pub enum Signal {
    Amplitude(f32),
    DutyCycle(f32),
    Frequency(u32),
    Level(u32),
    Offset(f32),
    Form(::redpitaya_scpi::generator::Form),
    Start,
    Stop,
}
