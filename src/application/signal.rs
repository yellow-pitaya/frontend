#[derive(Clone)]
pub enum Signal {
    AcquireRate(redpitaya_scpi::acquire::SamplingRate),
    GraphDraw,
    Level(String, i32),
    NeedDraw,
    Resize(i32, i32),
    TriggerAuto,
    TriggerNormal,
    TriggerSingle,
    Quit,
}

impl relm::DisplayVariant for Signal {
    fn display_variant(&self) -> &'static str {
        match *self {
            Signal::AcquireRate(_) => "Signal::AcquireRate",
            Signal::GraphDraw => "Signal::GraphDraw",
            Signal::Level(_, _) => "Signal::Level",
            Signal::NeedDraw => "Signal::NeedDraw",
            Signal::Resize(_, _) => "Signal::Resize",
            Signal::TriggerAuto => "Signal::TriggerAuto",
            Signal::TriggerNormal => "Signal::TriggerNormal",
            Signal::TriggerSingle => "Signal::TriggerSingle",
            Signal::Quit => "Signal::Quit",
        }
    }
}

impl relm::IntoOption<Self> for Signal {
    fn into_option(self) -> Option<Self> {
        Some(self)
    }
}
