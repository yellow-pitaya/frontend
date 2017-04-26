use gtk::{
    BoxExt,
    ButtonExt,
    ContainerExt,
    ToggleButtonExt,
};
use relm::ContainerWidget;

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
    pub amplitude: ::relm::Component<::widget::PreciseScale>,
    pub offset: ::relm::Component<::widget::PreciseScale>,
    pub frequency: ::relm::Component<::widget::PreciseScale>,
    pub duty_cycle: ::relm::Component<::widget::PreciseScale>,
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
                self.duty_cycle.widget().set_visible(is_pwm);
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

        let amplitude = page.add_widget::<::widget::PreciseScale, _>(&relm);
        amplitude.widget().set_label("Amplitude (V)");
        amplitude.widget().set_digits(2);
        amplitude.widget().set_adjustment(::gtk::Adjustment::new(
            0.0, -1.0, 1.0, 0.1, 1.0, 0.0
        ));
        connect!(
            amplitude@::widget::Signal::Changed(value),
            relm,
            Signal::Amplitude(::redpitaya_scpi::generator::Source::OUT1, value as f32)
        );

        let offset = page.add_widget::<::widget::PreciseScale, _>(&relm);
        offset.widget().set_label("Offset (V)");
        offset.widget().set_digits(2);
        offset.widget().set_adjustment(::gtk::Adjustment::new(
            0.0, -1.0, 1.0, 0.1, 1.0, 0.0
        ));
        connect!(
            offset@::widget::Signal::Changed(value),
            relm,
            Signal::Offset(::redpitaya_scpi::generator::Source::OUT1, value as f32)
        );

        let frequency = page.add_widget::<::widget::PreciseScale, _>(&relm);
        frequency.widget().set_label("Frequency (Hz)");
        frequency.widget().set_adjustment(::gtk::Adjustment::new(
            0.0, 0.0, 62_500_000.0, 1_000.0, 10_000.0, 0.0
        ));
        connect!(
            frequency@::widget::Signal::Changed(value),
            relm,
            Signal::Frequency(::redpitaya_scpi::generator::Source::OUT1, value as u32)
        );

        let duty_cycle = page.add_widget::<::widget::PreciseScale, _>(&relm);
        duty_cycle.widget().set_label("Duty cycle (%)");
        duty_cycle.widget().set_digits(2);
        duty_cycle.widget().set_adjustment(::gtk::Adjustment::new(
            0.0, 0.0, 1.0, 0.1, 1.0, 0.0
        ));
        connect!(
            duty_cycle@::widget::Signal::Changed(value),
            relm,
            Signal::DutyCycle(::redpitaya_scpi::generator::Source::OUT1, value as f32)
        );

        Widget {
            page: page,
            toggle: toggle,
            amplitude: amplitude,
            offset: offset,
            frequency: frequency,
            duty_cycle: duty_cycle,
        }
    }
}
