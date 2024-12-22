#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Channel {
    CH1,
    CH2,
    Ext,
}

impl std::fmt::Display for Channel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match *self {
            Self::CH1 => "CH1",
            Self::CH2 => "CH2",
            Self::Ext => "EXT",
        };

        f.write_str(s)
    }
}
