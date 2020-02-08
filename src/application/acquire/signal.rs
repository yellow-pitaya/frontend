#[derive(Clone)]
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

impl relm::DisplayVariant for Signal {
    fn display_variant(&self) -> &'static str {
        match *self {
            Signal::Attenuation(_, _) => "Signal::Attenuation",
            Signal::Average(_) => "Signal::Average",
            Signal::Start(_) => "Signal::Start",
            Signal::Gain(_, _) => "Signal::Gain",
            Signal::Rate(_) => "Signal::Rate",
            Signal::SetData(_, _) => "Signal::SetData",
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
