#[derive(Clone, Debug)]
pub struct Level {
    pub enable: bool,
    pub offset: i32,
}

#[derive(Clone)]
pub struct Model {
    pub current: Option<String>,
    pub orientation: super::widget::Orientation,
    pub levels: ::std::collections::HashMap<String, Level>,
}
