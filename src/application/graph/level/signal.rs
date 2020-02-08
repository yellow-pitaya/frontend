#[derive(Clone)]
pub enum Signal {
    Click(f64, f64),
    Draw,
    Invalidate,
    Move(f64, f64),
    Release,
    SourceStart(String),
    SourceStop(String),
    Level(String, i32),
}

impl ::relm::DisplayVariant for Signal {
    fn display_variant(&self) -> &'static str {
        match *self {
            Signal::Click(_, _) => "Signal::Click",
            Signal::Draw => "Signal::Draw",
            Signal::Invalidate => "Signal::Invalidate",
            Signal::Move(_, _) => "Signal::Move",
            Signal::Release => "Signal::Release",
            Signal::SourceStart(_) => "Signal::SourceStart",
            Signal::SourceStop(_) => "Signal::SourceStop",
            Signal::Level(_, _) => "Signal::Level",
        }
    }
}

impl ::relm::IntoOption<Self> for Signal {
    fn into_option(self) -> Option<Self> {
        Some(self)
    }
}
