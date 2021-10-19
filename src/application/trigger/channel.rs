#[derive(Copy, Clone, PartialEq)]
pub enum Channel {
    CH1,
    CH2,
    Ext,
}

impl std::fmt::Display for Channel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let display = match *self {
            Self::CH1 => "CH1",
            Self::CH2 => "CH2",
            Self::Ext => "EXT",
        };

        write!(f, "{}", display)
    }
}
