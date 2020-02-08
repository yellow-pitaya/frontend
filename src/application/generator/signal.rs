#[derive(Clone)]
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

impl relm::DisplayVariant for Signal {
    fn display_variant(&self) -> &'static str {
        match *self {
            Signal::Amplitude(_, _) => "Signal::Amplitude",
            Signal::DutyCycle(_, _) => "Signal::DutyCycle",
            Signal::Frequency(_, _) => "Signal::Frequency",
            Signal::Offset(_, _) => "Signal::Offset",
            Signal::Form(_, _) => "Signal::Signal",
            Signal::Start(_) => "Signal::Start",
            Signal::Stop(_) => "Signal::Stop",
            Signal::Redraw(_, _) => "Signal::Redraw",
        }
    }
}

impl relm::IntoOption<Self> for Signal {
    fn into_option(self) -> Option<Self> {
        Some(self)
    }
}
