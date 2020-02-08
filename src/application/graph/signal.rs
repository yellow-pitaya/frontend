#[derive(Clone)]
pub enum Signal {
    Invalidate,
    Draw,
    Level(String, i32),
    Redraw(::cairo::Context, ::application::Model),
    Resize(i32, i32),
    SetImage(::cairo::ImageSurface),
    SourceStart(super::level::widget::Orientation, String),
    SourceStop(super::level::widget::Orientation, String),
}

impl ::relm::DisplayVariant for Signal {
    fn display_variant(&self) -> &'static str {
        match *self {
            Signal::Invalidate => "Signal::Invalidate",
            Signal::Draw => "Signal::Draw",
            Signal::Level(_, _) => "Signal::Level",
            Signal::Redraw(_, _) => "Signal::Redraw",
            Signal::Resize(_, _) => "Signal::Resize",
            Signal::SetImage(_) => "Signal::SetImage",
            Signal::SourceStart(_, _) => "Signal::SourceStart",
            Signal::SourceStop(_, _) => "Signal::SourceStop",
        }
    }
}

impl ::relm::IntoOption<Self> for Signal {
    fn into_option(self) -> Option<Self> {
        Some(self)
    }
}
