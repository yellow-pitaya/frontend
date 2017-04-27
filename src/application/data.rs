use application::color::Colorable;

#[derive(Msg)]
pub enum Signal {
    Data,
}

#[derive(Clone)]
pub struct Widget {
    buffer: ::std::cell::RefCell<String>,
    label: ::gtk::Label,
    stream: ::relm::EventStream<Signal>,
}

impl Widget {
    pub fn set_buffer(&self, buffer: String) {
        *self.buffer.borrow_mut() = buffer;
        self.stream.emit(Signal::Data);
    }
}

impl ::application::Panel for Widget {
    fn draw(&self, context: &::cairo::Context, scales: ::Scales) {
        let buffer = self.buffer.borrow();
        let mut data = buffer
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

        for sample in 0..16_384 {
            let t = sample as f64 / 16_384.0 * scales.h.1;

            match data.next() {
                Some(y) => {
                    context.line_to(t, y);
                    context.move_to(t, y);
                },
                None => (),
            }
        }
        context.stroke();
    }
}

impl ::relm::Widget for Widget {
    type Model = ();
    type Msg = Signal;
    type Root = ::gtk::Label;

    fn model() -> Self::Model {
    }

    fn root(&self) -> &Self::Root {
        &self.label
    }

    fn update(&mut self, _: Signal, _: &mut Self::Model) {
    }

    fn view(relm: ::relm::RemoteRelm<Signal>, _: &Self::Model) -> Self {
        Widget {
            buffer: ::std::cell::RefCell::new(String::new()),
            label: ::gtk::Label::new(""),
            stream: relm.stream().clone(),
        }
    }
}
