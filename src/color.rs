#[derive(Clone)]
pub struct Color {
    name: &'static str,
    r: f64,
    g: f64,
    b: f64,
    a: f64,
}

impl std::convert::Into<Color> for redpitaya_scpi::generator::Source {
    fn into(self) -> Color {
        match self {
            redpitaya_scpi::generator::Source::OUT1 => OUT1,
            redpitaya_scpi::generator::Source::OUT2 => OUT2,
        }
    }
}

impl std::convert::Into<Color> for redpitaya_scpi::acquire::Source {
    fn into(self) -> Color {
        match self {
            redpitaya_scpi::acquire::Source::IN1 => IN1,
            redpitaya_scpi::acquire::Source::IN2 => IN2,
        }
    }
}

impl std::convert::Into<Color> for String {
    fn into(self) -> Color {
        match self.as_str() {
            "IN 1" => IN1,
            "IN 2" => IN2,
            "OUT 1" => OUT1,
            "OUT 2" => OUT2,
            "TRIG" | "DELAY" => TRIGGER,
            _ => MAIN_SCALE,
        }
    }
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "color-{}", self.name)
    }
}

impl Color {
    fn to_css(&self) -> String {
        format!(
            ".{} {{ background-color: rgba({}, {}, {}, {}); }}",
            self,
            self.r * 255.,
            self.g * 255.,
            self.b * 255.,
            self.a * 255.
        )
    }
}

pub const BACKGROUND: Color = Color {
    name: "background",
    r: 0.0,
    g: 0.0,
    b: 0.0,
    a: 1.0,
};

pub const MAIN_SCALE: Color = Color {
    name: "main_scale",
    r: 1.0,
    g: 1.0,
    b: 1.0,
    a: 1.0,
};

pub const SECONDARY_SCALE: Color = Color {
    name: "secondary_scale",
    r: 1.0,
    g: 1.0,
    b: 1.0,
    a: 0.2,
};

pub const IN1: Color = Color {
    name: "in1",
    r: 1.0,
    g: 1.0,
    b: 0.0,
    a: 1.0,
};

pub const IN2: Color = Color {
    name: "in2",
    r: 0.0,
    g: 1.0,
    b: 0.0,
    a: 1.0,
};

pub const OUT1: Color = Color {
    name: "out1",
    r: 1.0,
    g: 0.0,
    b: 1.0,
    a: 1.0,
};

pub const OUT2: Color = Color {
    name: "out2",
    r: 1.0,
    g: 0.0,
    b: 0.0,
    a: 1.0,
};

pub const TRIGGER: Color = Color {
    name: "trigger",
    r: 1.0,
    g: 0.5,
    b: 0.0,
    a: 1.0,
};

impl Color {
    pub fn init() {
        use gtk::CssProviderExt;

        let colors = [
            BACKGROUND,
            MAIN_SCALE,
            SECONDARY_SCALE,
            IN1,
            IN2,
            OUT1,
            OUT2,
            TRIGGER,
        ];
        let mut styles = String::new();

        for color in &colors {
            styles.push_str(&color.to_css());
        }

        let provider = gtk::CssProvider::new();

        provider
            .load_from_data(styles.as_bytes())
            .expect("Failed to load CSS");

        gtk::StyleContext::add_provider_for_screen(
            &gdk::Screen::get_default().expect("Error initializing gtk css provider."),
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

pub trait Colorable {
    fn set_color(&self, color: Color);
}

impl Colorable for cairo::Context {
    fn set_color(&self, color: Color) {
        self.set_source_rgba(color.r, color.g, color.b, color.a);
    }
}
