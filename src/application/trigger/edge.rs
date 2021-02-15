#[derive(Copy, Clone, PartialEq)]
pub enum Edge {
    Positive,
    Negative,
}

impl std::fmt::Display for Edge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let display = match *self {
            Self::Positive => "Positive",
            Self::Negative => "Negative",
        };

        write!(f, "{}", display)
    }
}
