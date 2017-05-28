#[derive(Clone)]
pub enum Signal {
    Attenuation(::redpitaya_scpi::acquire::Source, u8),
    Average(bool),
    Data(::redpitaya_scpi::acquire::Source),
    Gain(::redpitaya_scpi::acquire::Source, ::redpitaya_scpi::acquire::Gain),
    Rate(::redpitaya_scpi::acquire::SamplingRate),
    Start(::redpitaya_scpi::acquire::Source),
    Stop(::redpitaya_scpi::acquire::Source),
}

impl ::relm::DisplayVariant for Signal {
    fn display_variant(&self) -> &'static str {
        match *self {
            Signal::Attenuation(_, _) => "Signal::Attenuation",
            Signal::Average(_) => "Signal::Average",
            Signal::Data(_) => "Signal::Data",
            Signal::Start(_) => "Signal::Start",
            Signal::Gain(_, _) => "Signal::Gain",
            Signal::Rate(_) => "Signal::Rate",
            Signal::Stop(_) => "Signal::Stop",
        }
    }
}
