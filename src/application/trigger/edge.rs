#[derive(Copy, Clone, PartialEq)]
pub enum Edge {
    Positive,
    Negative,
}

impl ::std::fmt::Display for Edge {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        let display = match self {
            &Edge::Positive => "Positive",
            &Edge::Negative => "Negative",
        };

        write!(f, "{}", display)
    }
}
