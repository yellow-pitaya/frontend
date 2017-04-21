use gtk::{
    BoxExt,
    ButtonExt,
    ContainerExt,
    RangeExt,
    ToggleButtonExt,
    WidgetExt,
};

#[derive(Clone)]
pub enum Signal {
    Start(::redpitaya_scpi::generator::Source),
    Stop(::redpitaya_scpi::generator::Source),
    Amplitude(::redpitaya_scpi::generator::Source, f32),
    Offset(::redpitaya_scpi::generator::Source, f32),
    Frequency(::redpitaya_scpi::generator::Source, u32),
    DutyCycle(::redpitaya_scpi::generator::Source, f32),
    Signal(::redpitaya_scpi::generator::Source, ::redpitaya_scpi::generator::Form),
}

impl ::relm::DisplayVariant for Signal {
    fn display_variant(&self) -> &'static str {
        match *self {
            Signal::Amplitude(_, _) => "Signal::Amplitude",
            Signal::Offset(_, _) => "Signal::Offset",
            Signal::DutyCycle(_, _) => "Signal::DutyCycle",
            Signal::Frequency(_, _) => "Signal::Frequency",
            Signal::Signal(_, _) => "Signal::Signal",
            Signal::Start(_) => "Signal::Start",
            Signal::Stop(_) => "Signal::Stop",
        }
    }
}

#[derive(Clone)]
pub struct Widget {
    pub page: ::gtk::Box,
    pub toggle: ::gtk::ToggleButton,
    pub amplitude_scale: ::gtk::Scale,
    pub offset_scale: ::gtk::Scale,
    pub frequency_scale: ::gtk::Scale,
    pub duty_cycle_scale: ::gtk::Scale,
    pub duty_cycle_frame: ::gtk::Frame,
}

impl ::relm::Widget for Widget {
    type Model = ();
    type Msg = Signal;
    type Root = ::gtk::Box;

    fn model() -> Self::Model {
    }

    fn root(&self) -> &Self::Root {
        &self.page
    }

    fn update(&mut self, event: Signal, _: &mut Self::Model) {
        match event {
            Signal::Start(_) => self.toggle.set_label("Stop"),
            Signal::Stop(_) => self.toggle.set_label("Run"),
            Signal::Signal(_, form) => {
                let is_pwm = form == ::redpitaya_scpi::generator::Form::PWM;
                self.duty_cycle_frame.set_visible(is_pwm);
            },
            _ => (),
        };
    }

    fn view(relm: ::relm::RemoteRelm<Signal>, _: &Self::Model) -> Self {
        let page = ::gtk::Box::new(::gtk::Orientation::Vertical, 10);

        let toggle = ::gtk::ToggleButton::new_with_label("Run");
        page.pack_start(&toggle, false, false, 0);

        let stream = relm.stream().clone();
        toggle.connect_toggled(move |w| {
            if w.get_active() {
                stream.emit(Signal::Start(::redpitaya_scpi::generator::Source::OUT1));
            } else {
                stream.emit(Signal::Stop(::redpitaya_scpi::generator::Source::OUT1));
            }
        });

        let frame = ::gtk::Frame::new("Form");
        page.pack_start(&frame, false, true, 0);

        let vbox = ::gtk::Box::new(::gtk::Orientation::Vertical, 0);
        frame.add(&vbox);

        let forms = vec![
            ::redpitaya_scpi::generator::Form::SINE,
            ::redpitaya_scpi::generator::Form::SQUARE,
            ::redpitaya_scpi::generator::Form::TRIANGLE,
            ::redpitaya_scpi::generator::Form::SAWU,
            ::redpitaya_scpi::generator::Form::SAWD,
            ::redpitaya_scpi::generator::Form::PWM,
            // @TODO ::redpitaya_scpi::generator::Form::ARBITRARY,
        ];

        let mut group_member = None;

        for form in forms {
            let button = ::gtk::RadioButton::new_with_label_from_widget(
                group_member.as_ref(),
                format!("{}", form).as_str()
            );
            vbox.pack_start(&button, false, true, 0);

            let stream = relm.stream().clone();
            button.connect_toggled(move |f| {
                if f.get_active() {
                    stream.emit(
                        Signal::Signal(::redpitaya_scpi::generator::Source::OUT1, form.clone())
                    );
                }
            });

            if group_member == None {
                group_member = Some(button);
            }
        }

        let frame = ::gtk::Frame::new("Amplitude");
        page.pack_start(&frame, false, true, 0);

        let amplitude_scale = ::gtk::Scale::new_with_range(::gtk::Orientation::Horizontal, -1.0, 1.0, 0.01);
        amplitude_scale.add_mark(0.0, ::gtk::PositionType::Top, None);

        amplitude_scale.connect_format_value(move |_, value| {
            format!("{:.2} V", value)
        });

        let stream = relm.stream().clone();
        amplitude_scale.connect_change_value(move |_, _, value| {
            stream.emit(Signal::Amplitude(::redpitaya_scpi::generator::Source::OUT1, value as f32));

            ::gtk::Inhibit(false)
        });
        frame.add(&amplitude_scale);

        let frame = ::gtk::Frame::new("Offset");
        page.pack_start(&frame, false, true, 0);

        let offset_scale = ::gtk::Scale::new_with_range(::gtk::Orientation::Horizontal, -1.0, 1.0, 0.01);
        offset_scale.add_mark(0.0, ::gtk::PositionType::Top, None);

        offset_scale.connect_format_value(move |_, value| {
            format!("{:.2} V", value)
        });

        let stream = relm.stream().clone();
        offset_scale.connect_change_value(move |_, _, value| {
            stream.emit(Signal::Offset(::redpitaya_scpi::generator::Source::OUT1, value as f32));

            ::gtk::Inhibit(false)
        });
        frame.add(&offset_scale);

        let frame = ::gtk::Frame::new("Frequency");
        page.pack_start(&frame, false, true, 0);

        let frequency_scale = ::gtk::Scale::new_with_range(::gtk::Orientation::Horizontal, 0.0, 62_500_000.0, 1_000.0);
        frequency_scale.add_mark(0.0, ::gtk::PositionType::Top, None);

        frequency_scale.connect_format_value(move |_, value| {
            format!("{:0} Hz", value)
        });

        let stream = relm.stream().clone();
        frequency_scale.connect_change_value(move |_, _, value| {
            stream.emit(Signal::Frequency(::redpitaya_scpi::generator::Source::OUT1, value as u32));

            ::gtk::Inhibit(false)
        });
        frame.add(&frequency_scale);

        let duty_cycle_frame = ::gtk::Frame::new("Duty cycle");
        page.pack_start(&duty_cycle_frame, false, true, 0);

        let duty_cycle_scale = ::gtk::Scale::new_with_range(::gtk::Orientation::Horizontal, 0.0, 1.0, 0.01);
        duty_cycle_scale.add_mark(0.0, ::gtk::PositionType::Top, None);

        duty_cycle_scale.connect_format_value(move |_, value| {
            format!("{:.0} %", value * 100.0)
        });

        let stream = relm.stream().clone();
        duty_cycle_scale.connect_change_value(move |_, _, value| {
            stream.emit(Signal::DutyCycle(::redpitaya_scpi::generator::Source::OUT1, value as f32));

            ::gtk::Inhibit(false)
        });
        duty_cycle_frame.add(&duty_cycle_scale);

        Widget {
            page: page,
            toggle: toggle,
            amplitude_scale: amplitude_scale,
            offset_scale: offset_scale,
            frequency_scale: frequency_scale,
            duty_cycle_scale: duty_cycle_scale,
            duty_cycle_frame: duty_cycle_frame,
        }
    }
}
