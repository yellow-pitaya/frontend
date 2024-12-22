#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Edge {
    Positive,
    Negative,
}

impl std::fmt::Display for Edge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match *self {
            Self::Positive => "Positive",
            Self::Negative => "Negative",
        };

        f.write_str(s)
    }
}
