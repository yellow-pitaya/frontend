#[derive(Copy, Clone, PartialEq)]
pub enum Channel {
    CH1,
    CH2,
    EXT,
}

impl std::fmt::Display for Channel {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let display = match self {
            &Channel::CH1 => "CH1",
            &Channel::CH2 => "CH2",
            &Channel::EXT => "EXT",
        };

        write!(f, "{}", display)
    }
}
