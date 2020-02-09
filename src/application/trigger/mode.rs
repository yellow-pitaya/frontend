#[derive(Copy, Clone, PartialEq)]
pub enum Mode {
    Auto,
    Normal,
    Single,
}

impl std::fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let display = match *self {
            Self::Auto => "Auto",
            Self::Normal => "Normal",
            Self::Single => "Single",
        };

        write!(f, "{}", display)
    }
}
