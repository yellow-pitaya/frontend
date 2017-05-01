#[derive(Clone)]
pub enum Signal {
    Start(::redpitaya_scpi::generator::Source),
    Stop(::redpitaya_scpi::generator::Source),
    Amplitude(::redpitaya_scpi::generator::Source, f32),
    Offset(::redpitaya_scpi::generator::Source, f32),
    Frequency(::redpitaya_scpi::generator::Source, u32),
    Level(::redpitaya_scpi::generator::Source, u32),
    DutyCycle(::redpitaya_scpi::generator::Source, f32),
    Signal(::redpitaya_scpi::generator::Source, ::redpitaya_scpi::generator::Form),
}

impl ::relm::DisplayVariant for Signal {
    fn display_variant(&self) -> &'static str {
        match *self {
            Signal::Amplitude(_, _) => "Signal::Amplitude",
            Signal::Offset(_, _) => "Signal::Offset",
            Signal::DutyCycle(_, _) => "Signal::DutyCycle",
            Signal::Frequency(_, _) => "Signal::Frequency",
            Signal::Level(_, _) => "Signal::Level",
            Signal::Signal(_, _) => "Signal::Signal",
            Signal::Start(_) => "Signal::Start",
            Signal::Stop(_) => "Signal::Stop",
        }
    }
}
