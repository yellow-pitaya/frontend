#[derive(Clone)]
pub enum Signal {
    Attenuation(::redpitaya_scpi::acquire::Source, u8),
    Average(bool),
    Data(::redpitaya_scpi::acquire::Source),
    Gain(::redpitaya_scpi::acquire::Source, ::redpitaya_scpi::acquire::Gain),
    Level(::redpitaya_scpi::acquire::Source, u32),
    Rate(::redpitaya_scpi::acquire::SamplingRate),
}

impl ::relm::DisplayVariant for Signal {
    fn display_variant(&self) -> &'static str {
        match *self {
            Signal::Attenuation(_, _) => "Signal::Attenuation",
            Signal::Average(_) => "Signal::Average",
            Signal::Data(_) => "Signal::Data",
            Signal::Gain(_, _) => "Signal::Gain",
            Signal::Level(_, _) => "Signal::Level",
            Signal::Rate(_) => "Signal::Rate",
        }
    }
}
