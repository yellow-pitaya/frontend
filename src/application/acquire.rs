use gtk::{
    BoxExt,
    ContainerExt,
    ToggleButtonExt,
};
use color::Colorable;
use relm::ContainerWidget;

#[derive(Clone)]
pub enum Signal {
    Data,
    Decimation(::redpitaya_scpi::acquire::Decimation),
    Level(::redpitaya_scpi::acquire::Source, u32),
    Start,
    Stop,
}

impl ::relm::DisplayVariant for Signal {
    fn display_variant(&self) -> &'static str {
        match *self {
            Signal::Data => "Signal::Data",
            Signal::Decimation(_) => "Signal::Decimation",
            Signal::Level(_, _) => "Signal::Level",
            Signal::Start => "Signal::Start",
            Signal::Stop => "Signal::Stop",
        }
    }
}

#[derive(Clone)]
pub struct Widget {
    buffer: ::std::cell::RefCell<String>,
    level: ::relm::Component<::widget::PreciseScale>,
    stream: ::relm::EventStream<Signal>,
    page: ::gtk::Box,
    pub palette: ::relm::Component<::widget::Palette>,
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
    type Model = ();
    type Msg = Signal;
    type Root = ::gtk::Box;
    type ModelParam = ();

    fn model(_: Self::ModelParam) -> Self::Model {
    }

    fn root(&self) -> &Self::Root {
        &self.page
    }

    fn update(&mut self, _: Signal, _: &mut Self::Model) {
    }

    fn view(relm: &::relm::RemoteRelm<Self>, _: &Self::Model) -> Self {
        let page = ::gtk::Box::new(::gtk::Orientation::Vertical, 10);

        let palette = page.add_widget::<::widget::Palette, _>(&relm, ());
        palette.widget().set_label("IN 1");
        palette.widget().set_color(::color::IN1);
        connect!(palette@::widget::Signal::Expand, relm, Signal::Start);
        connect!(palette@::widget::Signal::Fold, relm, Signal::Stop);

        let vbox  = ::gtk::Box::new(::gtk::Orientation::Vertical, 10);
        palette.widget().add(&vbox);

        let level = vbox.add_widget::<::widget::PreciseScale, _>(&relm, ());
        level.widget().set_label("Level (V)");
        level.widget().set_adjustment(::gtk::Adjustment::new(
            0.0, -10.0, 10.0, 0.1, 1.0, 0.0
        ));
        connect!(
            level@::widget::Signal::Changed(value),
            relm,
            Signal::Level(::redpitaya_scpi::acquire::Source::IN1, value as u32)
        );

        let frame = ::gtk::Frame::new("Decimation");
        page.pack_start(&frame, false, true, 0);

        let flow_box = ::gtk::FlowBox::new();
        frame.add(&flow_box);

        let decimations = vec![
            ::redpitaya_scpi::acquire::Decimation::DEC_1,
            ::redpitaya_scpi::acquire::Decimation::DEC_8,
            ::redpitaya_scpi::acquire::Decimation::DEC_64,
            ::redpitaya_scpi::acquire::Decimation::DEC_1024,
            ::redpitaya_scpi::acquire::Decimation::DEC_8192,
            ::redpitaya_scpi::acquire::Decimation::DEC_65536,
        ];

        let mut group_member = None;

        for decimation in decimations {
            let button = ::gtk::RadioButton::new_with_label_from_widget(
                group_member.as_ref(),
                decimation.get_sampling_rate()
            );
            flow_box.add(&button);

            let stream = relm.stream().clone();
            button.connect_toggled(move |f| {
                if f.get_active() {
                    stream.emit(
                        Signal::Decimation(decimation.clone())
                    );
                }
            });

            if group_member == None {
                group_member = Some(button);
            }
        }

        let buffer = ::std::cell::RefCell::new(String::new());
        let stream = relm.stream().clone();

        Widget {
            buffer,
            level,
            page,
            palette,
            stream,
        }
    }
}
