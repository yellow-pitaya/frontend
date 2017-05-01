use color::Colorable;
use relm::ContainerWidget;

#[derive(Clone)]
pub enum Signal {
    Start(::redpitaya_scpi::generator::Source),
    Stop(::redpitaya_scpi::generator::Source),
    Amplitude(::redpitaya_scpi::generator::Source, f32),
    Offset(::redpitaya_scpi::generator::Source, f32),
    Frequency(::redpitaya_scpi::generator::Source, u32),
    Level(::redpitaya_scpi::generator::Source, u32),
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
            Signal::Level(_, _) => "Signal::Level",
            Signal::Signal(_, _) => "Signal::Signal",
            Signal::Start(_) => "Signal::Start",
            Signal::Stop(_) => "Signal::Stop",
        }
    }
}

#[derive(Clone)]
pub struct Widget {
    pub page: ::gtk::Box,
    pub palette: ::relm::Component<::widget::Palette>,
    pub amplitude: ::relm::Component<::widget::PreciseScale>,
    form: ::relm::Component<::widget::RadioGroup<::redpitaya_scpi::generator::Form>>,
    pub offset: ::relm::Component<::widget::PreciseScale>,
    pub frequency: ::relm::Component<::widget::PreciseScale>,
    level: ::relm::Component<::widget::PreciseScale>,
    pub duty_cycle: ::relm::Component<::widget::PreciseScale>,
}

impl Widget {
    fn is_started(&self) -> bool {
        self.palette.widget().get_active()
    }

    fn draw_level(&self, context: &::cairo::Context, scales: ::Scales) {
        context.move_to(scales.h.0, 0.0);
        context.line_to(scales.h.1, 0.0);

        context.stroke();
    }

    fn draw_data(&self, context: &::cairo::Context, scales: ::Scales) {
        context.set_line_width(0.05);

        if let Some(form) = self.form.widget().get_current() {
            let amplitude = self.amplitude.widget().get_value();
            let frequency = self.frequency.widget().get_value() / 1_000_000.0;

            for sample in 0..scales.n_samples {
                let x = scales.sample_to_ms(sample);
                let y = match form {
                    ::redpitaya_scpi::generator::Form::SINE => self.sine(x, amplitude, frequency),
                    ::redpitaya_scpi::generator::Form::SQUARE => self.square(x, amplitude, frequency),
                    ::redpitaya_scpi::generator::Form::TRIANGLE => self.triangle(x, amplitude, frequency),
                    ::redpitaya_scpi::generator::Form::SAWU => self.sawu(x, amplitude, frequency),
                    ::redpitaya_scpi::generator::Form::SAWD => self.sawd(x, amplitude, frequency),
                    ::redpitaya_scpi::generator::Form::PWM => self.pwm(x, amplitude, frequency),
                    _ => unimplemented!(),
                };

                context.line_to(x, y);
                context.move_to(x, y);
            }

            context.stroke();
        }
    }

    fn sine(&self, x: f64, amplitude: f64, frequency: f64) -> f64 {
        amplitude * (x * frequency * 2.0 * ::std::f64::consts::PI).sin()
    }

    fn square(&self, x: f64, amplitude: f64, frequency: f64) -> f64 {
        amplitude * self.sine(x, amplitude, frequency).signum()
    }

    fn triangle(&self, x: f64, amplitude: f64, frequency: f64) -> f64 {
        amplitude * 2.0 / ::std::f64::consts::PI * (frequency * 2.0 * ::std::f64::consts::PI * x).sin().asin()
    }

    fn sawu(&self, x: f64, amplitude: f64, frequency: f64) -> f64 {
        amplitude * (x * frequency).fract()
    }

    fn sawd(&self, x: f64, amplitude: f64, frequency: f64) -> f64 {
        amplitude *  (1.0 - (x * frequency).fract())
    }

    fn pwm(&self, x: f64, amplitude: f64, frequency: f64) -> f64 {
        amplitude * (x * frequency * 2.0 * ::std::f64::consts::PI).sin()
    }
}

impl ::relm::Widget for Widget {
    type Model = ::redpitaya_scpi::generator::Generator;
    type Msg = Signal;
    type Root = ::gtk::Box;
    type ModelParam = ::redpitaya_scpi::generator::Generator;

    fn model(generator: Self::ModelParam) -> Self::Model {
        generator
    }

    fn root(&self) -> &Self::Root {
        &self.page
    }

    fn update(&mut self, event: Signal, generator: &mut Self::Model) {
        match event {
            Signal::Amplitude(source, value) => generator.set_amplitude(source, value),
            Signal::Offset(source, value) => generator.set_offset(source, value),
            Signal::Frequency(source, value) => generator.set_frequency(source, value),
            Signal::DutyCycle(source, value) => generator.set_duty_cycle(source, value),
            Signal::Start(source) => generator.start(source),
            Signal::Stop(source) => generator.stop(source),
            Signal::Signal(source, form) => {
                let is_pwm = form == ::redpitaya_scpi::generator::Form::PWM;
                self.duty_cycle.widget().set_visible(is_pwm);
                generator.set_form(source, form);
            },
            Signal::Level(_, _) => (),
        };
    }

    fn view(relm: &::relm::RemoteRelm<Self>, _: &Self::Model) -> Self {
        let page = ::gtk::Box::new(::gtk::Orientation::Vertical, 10);

        let palette = page.add_widget::<::widget::Palette, _>(&relm, ());
        palette.widget().set_label("OUT 1");
        palette.widget().set_color(::color::OUT1);
        connect!(palette@::widget::palette::Signal::Expand, relm, Signal::Start(::redpitaya_scpi::generator::Source::OUT1));
        connect!(palette@::widget::palette::Signal::Fold, relm, Signal::Stop(::redpitaya_scpi::generator::Source::OUT1));

        let vbox  = ::gtk::Box::new(::gtk::Orientation::Vertical, 10);
        palette.widget().add(&vbox);

        let args = ::widget::radio::Model {
            title: String::from("Form"),
            options: vec![
            ::redpitaya_scpi::generator::Form::SINE,
            ::redpitaya_scpi::generator::Form::SQUARE,
            ::redpitaya_scpi::generator::Form::TRIANGLE,
            ::redpitaya_scpi::generator::Form::SAWU,
            ::redpitaya_scpi::generator::Form::SAWD,
            ::redpitaya_scpi::generator::Form::PWM,
            // @TODO ::redpitaya_scpi::generator::Form::ARBITRARY,
            ],
            current: Some(::redpitaya_scpi::generator::Form::SINE),
        };
        let form = vbox.add_widget::<::widget::RadioGroup<::redpitaya_scpi::generator::Form>, _>(&relm, args);
        connect!(
            form@::widget::radio::Signal::Change(form),
            relm,
            Signal::Signal(::redpitaya_scpi::generator::Source::OUT1, form)
        );

        let amplitude = vbox.add_widget::<::widget::PreciseScale, _>(&relm, ());
        amplitude.widget().set_label("Amplitude (V)");
        amplitude.widget().set_digits(2);
        amplitude.widget().set_adjustment(::gtk::Adjustment::new(
            0.0, -1.0, 1.0, 0.1, 1.0, 0.0
        ));
        connect!(
            amplitude@::widget::precise::Signal::Changed(value),
            relm,
            Signal::Amplitude(::redpitaya_scpi::generator::Source::OUT1, value as f32)
        );

        let offset = vbox.add_widget::<::widget::PreciseScale, _>(&relm, ());
        offset.widget().set_label("Offset (V)");
        offset.widget().set_digits(2);
        offset.widget().set_adjustment(::gtk::Adjustment::new(
            0.0, -1.0, 1.0, 0.1, 1.0, 0.0
        ));
        connect!(
            offset@::widget::precise::Signal::Changed(value),
            relm,
            Signal::Offset(::redpitaya_scpi::generator::Source::OUT1, value as f32)
        );

        let frequency = vbox.add_widget::<::widget::PreciseScale, _>(&relm, ());
        frequency.widget().set_label("Frequency (Hz)");
        frequency.widget().set_adjustment(::gtk::Adjustment::new(
            0.0, 0.0, 62_500_000.0, 1_000.0, 10_000.0, 0.0
        ));
        connect!(
            frequency@::widget::precise::Signal::Changed(value),
            relm,
            Signal::Frequency(::redpitaya_scpi::generator::Source::OUT1, value as u32)
        );

        let level = vbox.add_widget::<::widget::PreciseScale, _>(&relm, ());
        level.widget().set_label("Level (V)");
        level.widget().set_adjustment(::gtk::Adjustment::new(
            0.0, -10.0, 10.0, 0.1, 1.0, 0.0
        ));
        connect!(
            level@::widget::precise::Signal::Changed(value),
            relm,
            Signal::Level(::redpitaya_scpi::generator::Source::OUT1, value as u32)
        );

        let duty_cycle = vbox.add_widget::<::widget::PreciseScale, _>(&relm, ());
        duty_cycle.widget().set_label("Duty cycle (%)");
        duty_cycle.widget().set_digits(2);
        duty_cycle.widget().set_adjustment(::gtk::Adjustment::new(
            0.0, 0.0, 1.0, 0.1, 1.0, 0.0
        ));
        connect!(
            duty_cycle@::widget::precise::Signal::Changed(value),
            relm,
            Signal::DutyCycle(::redpitaya_scpi::generator::Source::OUT1, value as f32)
        );

        Widget {
            page,
            palette,
            amplitude,
            form,
            offset,
            frequency,
            level,
            duty_cycle,
        }
    }
}

impl ::application::Panel for Widget {
    fn draw(&self, context: &::cairo::Context, scales: ::Scales) {
        if !self.is_started() {
            return;
        }

        context.set_color(::color::OUT1);

        let level = self.level.widget().get_value();
        context.translate(0.0, level);

        self.draw_level(&context, scales);
        self.draw_data(&context, scales);
    }
}
