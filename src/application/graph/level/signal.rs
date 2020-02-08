#[derive(relm_derive::Msg, Clone)]
pub enum Signal {
    Click(f64, f64),
    Draw,
    Invalidate,
    Move(f64, f64),
    Release,
    SourceStart(String),
    SourceStop(String),
    Level(String, i32),
}
