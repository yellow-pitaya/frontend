#[derive(Clone)]
pub enum Signal {
    Click((f64, f64)),
    Draw,
    Move((f64, f64)),
    SourceStart(String),
    SourceStop(String),
    Level(String, i32),
}

impl ::relm::DisplayVariant for Signal {
    fn display_variant(&self) -> &'static str {
        match *self {
            Signal::Click(_) => "Signal::Click",
            Signal::Draw => "Signal::Draw",
            Signal::Move(_) => "Signal::Move",
            Signal::SourceStart(_) => "Signal::SourceStart",
            Signal::SourceStop(_) => "Signal::SourceStop",
            Signal::Level(_, _) => "Signal::Level",
        }
    }
}
