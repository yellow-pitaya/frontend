use gtk::{
    ContainerExt,
    ToggleButtonExt,
};
use color::Colorable;
use relm::ContainerWidget;

#[derive(Clone)]
pub enum Signal {
    Data,
    Rate(::redpitaya_scpi::acquire::SamplingRate),
    Level(::redpitaya_scpi::acquire::Source, u32),
    Start,
    Stop,
    Average(bool),
}

impl ::relm::DisplayVariant for Signal {
    fn display_variant(&self) -> &'static str {
        match *self {
            Signal::Data => "Signal::Data",
            Signal::Rate(_) => "Signal::Rate",
            Signal::Level(_, _) => "Signal::Level",
            Signal::Start => "Signal::Start",
            Signal::Stop => "Signal::Stop",
            Signal::Average(_) => "Signal::Average",
        }
    }
}

#[derive(Clone)]
pub struct Widget {
    buffer: ::std::cell::RefCell<String>,
    level: ::relm::Component<::widget::PreciseScale>,
    stream: ::relm::EventStream<Signal>,
    page: ::gtk::Box,
    rate: ::relm::Component<::widget::RadioGroup<::redpitaya_scpi::acquire::SamplingRate>>,
    pub palette: ::relm::Component<::widget::Palette>,
    average: ::gtk::CheckButton,
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

    fn draw_data(&self, context: &::cairo::Context, scales: ::Scales) {
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
                    context.line_to(x, y);
                    context.move_to(x, y);
                },
                None => break,
            }
        }
        context.stroke();
    }
}

impl ::application::Panel for Widget {
    fn draw(&self, context: &::cairo::Context, scales: ::Scales) {
        if !self.is_started() {
            return;
        }

        context.set_color(::color::IN1);

        let level = self.level.widget().get_value();
        context.translate(0.0, level);

        self.draw_level(&context, scales);
        self.draw_data(&context, scales);
    }
}

impl ::relm::Widget for Widget {
    type Model = ::redpitaya_scpi::acquire::Acquire;
    type Msg = Signal;
    type Root = ::gtk::Box;
    type ModelParam = ::redpitaya_scpi::acquire::Acquire;

    fn model(acquire: Self::ModelParam) -> Self::Model {
        acquire
    }

    fn root(&self) -> &Self::Root {
        &self.page
    }

    fn update(&mut self, event: Signal, acquire: &mut Self::Model) {
        match event {
            Signal::Start => acquire.start(),
            Signal::Stop => acquire.stop(),
            Signal::Rate(rate) => acquire.set_decimation(rate.into()),
            Signal::Average(enable) => if enable {
                acquire.enable_average();
            } else {
                acquire.disable_average();
            },
            _ => (),
        };
    }

    fn view(relm: &::relm::RemoteRelm<Self>, acquire: &Self::Model) -> Self {
        let page = ::gtk::Box::new(::gtk::Orientation::Vertical, 10);

        let palette = page.add_widget::<::widget::Palette, _>(&relm, ());
        palette.widget().set_label("IN 1");
        palette.widget().set_color(::color::IN1);
        connect!(palette@::widget::palette::Signal::Expand, relm, Signal::Start);
        connect!(palette@::widget::palette::Signal::Fold, relm, Signal::Stop);

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
            Signal::Level(::redpitaya_scpi::acquire::Source::IN1, value as u32)
        );

        let args = ::widget::radio::Model {
            title: String::from("Sampling Rate"),
            options: vec![
                ::redpitaya_scpi::acquire::SamplingRate::RATE_1_9kHz,
                ::redpitaya_scpi::acquire::SamplingRate::RATE_15_2kHz,
                ::redpitaya_scpi::acquire::SamplingRate::RATE_103_8kHz,
                ::redpitaya_scpi::acquire::SamplingRate::RATE_1_9MHz,
                ::redpitaya_scpi::acquire::SamplingRate::RATE_15_6MHz,
                ::redpitaya_scpi::acquire::SamplingRate::RATE_125MHz,
            ],
            current: match acquire.get_decimation() {
                Ok(sampling_rate) => Some(sampling_rate.into()),
                Err(_) => None,
            },
        };
        let rate = vbox.add_widget::<::widget::RadioGroup<::redpitaya_scpi::acquire::SamplingRate>, _>(&relm, args);
        connect!(
            rate@::widget::radio::Signal::Change(rate),
            relm,
            Signal::Rate(rate)
        );

        let average = ::gtk::CheckButton::new_with_label("Average");
        average.set_active(acquire.is_average_enabled());
        page.add(&average);
        connect!(
            relm, average, connect_toggled(w), Signal::Average(w.get_active())
        );

        let buffer = ::std::cell::RefCell::new(String::new());
        let stream = relm.stream().clone();

        Widget {
            buffer,
            level,
            page,
            palette,
            stream,
            rate,
            average,
        }
    }
}
