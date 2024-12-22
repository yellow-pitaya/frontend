#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Mode {
    Auto,
    Normal,
    Single,
}

impl std::fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match *self {
            Self::Auto => "Auto",
            Self::Normal => "Normal",
            Self::Single => "Single",
        };

        f.write_str(s)
    }
}
