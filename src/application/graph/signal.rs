#[derive(Clone)]
pub enum Signal {
    Draw,
    Level(String, i32),
}

impl ::relm::DisplayVariant for Signal {
    fn display_variant(&self) -> &'static str {
        match *self {
            Signal::Draw => "Signal::Draw",
            Signal::Level(_, _) => "Signal::Level",
        }
    }
}
