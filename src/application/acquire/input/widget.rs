use color::Colorable;
use relm::ContainerWidget;
use super::Model;
use super::Signal;

#[derive(Clone)]
pub struct Widget {
    buffer: ::std::cell::RefCell<String>,
    level: ::relm::Component<::widget::PreciseScale>,
    stream: ::relm::EventStream<Signal>,
    page: ::gtk::Box,
    pub palette: ::relm::Component<::widget::Palette>,
    source: ::redpitaya_scpi::acquire::Source,
    attenuation: ::relm::Component<::widget::RadioGroup<u8>>,
}

impl Widget {
    fn is_started(&self) -> bool {
        self.palette.widget().get_active()
    }

    pub fn set_buffer(&self, buffer: String) {
        *self.buffer.borrow_mut() = buffer;
        self.stream.emit(Signal::Data);
    }

    fn draw_level(&self, context: &::cairo::Context, scales: ::Scales) {
        context.move_to(scales.h.0, 0.0);
        context.line_to(scales.h.1, 0.0);

        context.stroke();
    }

    fn draw_data(&self, context: &::cairo::Context, scales: ::Scales, attenuation: u8) {
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

        for sample in 0..scales.n_samples {
            let x = scales.sample_to_ms(sample);

            match data.next() {
                Some(y) => {
                    context.line_to(x, y * attenuation as f64);
                    context.move_to(x, y * attenuation as f64);
                },
                None => break,
            }
        }
        context.stroke();
    }
}

impl ::application::Panel for Widget {
    fn draw(&self, context: &::cairo::Context, model: &::application::Model) {
        if !self.is_started() {
            return;
        }

        context.set_color(self.source.into());

        let level = self.level.widget().get_value();
        context.translate(0.0, level);

        self.draw_level(&context, model.scales);

        let attenuation = match self.attenuation.widget().get_current() {
            Some(attenuation) => attenuation,
            None => 1,
        };
        self.draw_data(&context, model.scales, attenuation);
    }
}

impl ::relm::Widget for Widget {
    type Model = Model;
    type Msg = Signal;
    type Root = ::gtk::Box;
    type ModelParam = Model;

    fn model(model: Self::ModelParam) -> Self::Model {
        model
    }

    fn root(&self) -> &Self::Root {
        &self.page
    }

    fn update(&mut self, event: Signal, model: &mut Self::Model) {
        match event {
            Signal::Gain(gain) => model.acquire.set_gain(model.source, gain),
            _ => (),
        };
    }

    fn view(relm: &::relm::RemoteRelm<Self>, model: &Self::Model) -> Self {
        let page = ::gtk::Box::new(::gtk::Orientation::Vertical, 10);

        let palette = page.add_widget::<::widget::Palette, _>(&relm, ());
        palette.widget().set_label(format!("{}", model.source).as_str());
        palette.widget().set_color(model.source.into());

        let vbox  = ::gtk::Box::new(::gtk::Orientation::Vertical, 10);
        palette.widget().add(&vbox);

        let level = vbox.add_widget::<::widget::PreciseScale, _>(&relm, ());
        level.widget().set_label("Level (V)");
        level.widget().set_adjustment(::gtk::Adjustment::new(
            0.0, -10.0, 10.0, 0.1, 1.0, 0.0
        ));
        connect!(
            level@::widget::precise::Signal::Changed(value),
            relm,
            Signal::Level(value as u32)
        );

        let args = ::widget::radio::Model {
            title: String::from("Gain"),
            options: vec![
                ::redpitaya_scpi::acquire::Gain::LV,
                ::redpitaya_scpi::acquire::Gain::HV,
            ],
            current: match model.acquire.get_gain(model.source) {
                Ok(gain) => Some(gain),
                Err(_) => None,
            },
        };
        let gain = vbox.add_widget::<::widget::RadioGroup<::redpitaya_scpi::acquire::Gain>, _>(&relm, args);
        connect!(
            gain@::widget::radio::Signal::Change(gain),
            relm,
            Signal::Gain(gain)
        );

        let args = ::widget::radio::Model {
            title: String::from("Probe attenuation"),
            options: vec![1, 10, 100],
            current: Some(1),
        };
        let attenuation = vbox.add_widget::<::widget::RadioGroup<u8>, _>(&relm, args);
        connect!(
            attenuation@::widget::radio::Signal::Change(attenuation),
            relm,
            Signal::Attenuation(attenuation)
        );

        let buffer = ::std::cell::RefCell::new(String::new());
        let stream = relm.stream().clone();
        let source = model.source;

        Widget {
            buffer,
            level,
            page,
            palette,
            stream,
            source,
            attenuation,
        }
    }
}
