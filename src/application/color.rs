type Color = (f64, f64, f64, f64);

pub const BACKGROUND: Color = (0.0, 0.0, 0.0, 1.0);
pub const MAIN_SCALE: Color = (1.0, 1.0, 1.0, 1.0);
pub const SECONDARY_SCALE: Color = (1.0, 1.0, 1.0, 0.2);
pub const IN1: Color = (1.0, 1.0, 0.0, 1.0);
pub const TRIGGER: Color = (1.0, 0.5, 0.0, 1.0);

pub trait Colorable {
    fn set_color(&self, color: Color);
}

impl Colorable for ::cairo::Context {
    fn set_color(&self, color: Color) {
        self.set_source_rgba(
            color.0, color.1, color.2, color.3
        );
    }
}
