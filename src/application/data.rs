use ::application::color::Colorable;

#[derive(Clone)]
pub struct Widget {
    pub data: String,
}

impl Widget {
    pub fn new() -> Self {
        Widget {
            data: String::new(),
        }
    }
}

impl ::application::Panel for Widget {
    fn draw(&self, context: &::cairo::Context, _: ::application::Scales) {
        let mut data = self.data
            .trim_matches(|c: char| c == '{' || c == '}' || c == '!' || c.is_alphabetic())
            .split(",")
            .map(|s| {
                match s.parse::<f64>() {
                    Ok(f) => f,
                    Err(_) => {
                        error!("Invalid data '{}'", s);
                        0.0
                    },
                }
            });

        context.set_line_width(0.05);
        context.set_color(::application::color::IN1);

        for x in 0..16384 {
            match data.next() {
                Some(y) => {
                    context.line_to(x as f64, y);
                    context.move_to(x as f64, y);
                },
                None => (),
            }
        }
        context.stroke();
    }
}
