pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}

impl ::std::convert::Into<Color> for ::redpitaya_scpi::generator::Source {
    fn into(self) -> Color {
        match self {
            ::redpitaya_scpi::generator::Source::OUT1 => OUT1,
            ::redpitaya_scpi::generator::Source::OUT2 => OUT2,
        }
    }
}

impl ::std::convert::Into<Color> for ::redpitaya_scpi::acquire::Source {
    fn into(self) -> Color {
        match self {
            ::redpitaya_scpi::acquire::Source::IN1 => IN1,
            ::redpitaya_scpi::acquire::Source::IN2 => IN2,
        }
    }
}

pub const BACKGROUND: Color = Color { r: 0.0, g:0.0, b: 0.0, a: 1.0 };
pub const MAIN_SCALE: Color = Color { r: 1.0, g:1.0, b: 1.0, a: 1.0 };
pub const SECONDARY_SCALE: Color = Color { r: 1.0, g:1.0, b: 1.0, a: 0.2 };
pub const IN1: Color = Color { r: 1.0, g:1.0, b: 0.0, a: 1.0 };
pub const IN2: Color = Color { r: 0.0, g:1.0, b: 0.0, a: 1.0 };
pub const OUT1: Color = Color { r: 1.0, g:0.0, b: 1.0, a: 1.0 };
pub const OUT2: Color = Color { r: 1.0, g:0.0, b: 0.0, a: 1.0 };
pub const TRIGGER: Color = Color { r: 1.0, g:0.5, b: 0.0, a: 1.0 };

pub trait Colorable {
    fn set_color(&self, color: Color);
}

impl Colorable for ::cairo::Context {
    fn set_color(&self, color: Color) {
        self.set_source_rgba(
            color.r, color.g, color.b, color.a
        );
    }
}
