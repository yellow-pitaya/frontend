#[derive(relm_derive::Msg, Clone)]
pub enum Signal {
    Invalidate,
    Draw,
    Level(String, i32),
    Redraw(cairo::Context, crate::application::Model),
    Resize(i32, i32),
    SetImage(cairo::ImageSurface),
    SourceStart(super::level::widget::Orientation, String),
    SourceStop(super::level::widget::Orientation, String),
}
