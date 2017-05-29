use color::Colorable;
use relm::ContainerWidget;
use super::Model;
use super::Signal;

#[derive(Clone)]
pub struct Widget {
    pub page: ::gtk::Box,
    pub palette: ::relm::Component<::widget::Palette>,
    pub amplitude: ::relm::Component<::widget::PreciseScale>,
    form: ::relm::Component<::widget::RadioGroup<::redpitaya_scpi::generator::Form>>,
    pub offset: ::relm::Component<::widget::PreciseScale>,
    pub frequency: ::relm::Component<::widget::PreciseScale>,
    pub duty_cycle: ::relm::Component<::widget::PreciseScale>,
    source: ::redpitaya_scpi::generator::Source,
}

impl Widget {
    fn is_started(&self) -> bool {
        self.palette.widget().get_active()
    }

    fn draw_data(&self, context: &::cairo::Context, scales: ::Scales) {
        context.set_line_width(0.05);

        if let Some(form) = self.form.widget().get_current() {
            let amplitude = self.amplitude.widget().get_value();
            let frequency = self.frequency.widget().get_value() / 1_000_000.0;
            let duty_cycle = self.duty_cycle.widget().get_value();
            let offset = self.offset.widget().get_value();

            for sample in (scales.h.0 as i32)..(scales.h.1 as i32) {
                let x = scales.x_to_offset(sample);
                let mut y = match form {
                    ::redpitaya_scpi::generator::Form::SINE => self.sine(x, amplitude, frequency),
                    ::redpitaya_scpi::generator::Form::SQUARE => self.square(x, amplitude, frequency),
                    ::redpitaya_scpi::generator::Form::TRIANGLE => self.triangle(x, amplitude, frequency),
                    ::redpitaya_scpi::generator::Form::SAWU => self.sawu(x, amplitude, frequency),
                    ::redpitaya_scpi::generator::Form::SAWD => self.sawd(x, amplitude, frequency),
                    ::redpitaya_scpi::generator::Form::DC => self.dc(x, amplitude, frequency),
                    ::redpitaya_scpi::generator::Form::PWM => self.pwm(x, amplitude, frequency, duty_cycle),
                    _ => unimplemented!(),
                };

                y += offset;

                if y < -1.0 {
                    y = -1.0;
                }

                if y > 1.0 {
                    y = 1.0;
                }

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

    fn dc(&self, _: f64, amplitude: f64, _: f64) -> f64 {
        amplitude
    }

    fn pwm(&self, x: f64, amplitude: f64, frequency: f64, duty_cycle: f64) -> f64 {
        let v = self.sawu(x, amplitude, frequency) - amplitude * duty_cycle;

        if v > 0.0 {
            -amplitude
        }
        else if v < 0.0 {
            amplitude
        }
        else {
            0.0
        }
    }
}

impl ::relm::Widget for Widget {
    type Model = Model;
    type Msg = Signal;
    type Root = ::gtk::Box;
    type ModelParam = Model;

    fn model(model: Self::Model) -> Self::Model {
        model
    }

    fn root(&self) -> &Self::Root {
        &self.page
    }

    fn update(&mut self, event: Signal, model: &mut Self::Model) {
        match event {
            Signal::Amplitude(value) => model.generator.set_amplitude(model.source, value),
            Signal::Offset(value) => model.generator.set_offset(model.source, value),
            Signal::Frequency(value) => model.generator.set_frequency(model.source, value),
            Signal::DutyCycle(value) => model.generator.set_duty_cycle(model.source, value),
            Signal::Start => model.generator.start(model.source),
            Signal::Stop => model.generator.stop(model.source),
            Signal::Form(form) => {
                let is_pwm = form == ::redpitaya_scpi::generator::Form::PWM;
                self.duty_cycle.widget().set_visible(is_pwm);
                model.generator.set_form(model.source, form);
            },
        };
    }

    fn view(relm: &::relm::RemoteRelm<Self>, model: &Self::Model) -> Self {
        let source = model.source;

        let page = ::gtk::Box::new(::gtk::Orientation::Vertical, 10);

        let palette = page.add_widget::<::widget::Palette, _>(&relm, ());
        palette.widget().set_label(format!("{}", model.source).as_str());
        palette.widget().set_color(model.source.into());
        connect!(palette@::widget::palette::Signal::Expand, relm, Signal::Start);
        connect!(palette@::widget::palette::Signal::Fold, relm, Signal::Stop);

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
            ::redpitaya_scpi::generator::Form::DC,
            // @TODO ::redpitaya_scpi::generator::Form::ARBITRARY,
            ],
            current: None,
        };
        let form = vbox.add_widget::<::widget::RadioGroup<::redpitaya_scpi::generator::Form>, _>(&relm, args);
        connect!(
            form@::widget::radio::Signal::Change(form),
            relm,
            Signal::Form(form)
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
            Signal::Amplitude(value as f32)
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
            Signal::Offset(value as f32)
        );

        let frequency = vbox.add_widget::<::widget::PreciseScale, _>(&relm, ());
        frequency.widget().set_label("Frequency (Hz)");
        frequency.widget().set_adjustment(::gtk::Adjustment::new(
            0.0, 0.0, 62_500_000.0, 1_000.0, 10_000.0, 0.0
        ));
        connect!(
            frequency@::widget::precise::Signal::Changed(value),
            relm,
            Signal::Frequency(value as u32)
        );

        let duty_cycle = vbox.add_widget::<::widget::PreciseScale, _>(&relm, ());
        duty_cycle.widget().set_no_show_all(true);
        duty_cycle.widget().set_visible(false);
        duty_cycle.widget().set_label("Duty cycle (%)");
        duty_cycle.widget().set_digits(2);
        duty_cycle.widget().set_adjustment(::gtk::Adjustment::new(
            0.0, 0.0, 1.0, 0.1, 1.0, 0.0
        ));
        connect!(
            duty_cycle@::widget::precise::Signal::Changed(value),
            relm,
            Signal::DutyCycle(value as f32)
        );

        Widget {
            page,
            palette,
            amplitude,
            form,
            offset,
            frequency,
            duty_cycle,
            source,
        }
    }

    fn init_view(&self, model: &mut Self::Model) {
        match model.generator.get_form(model.source) {
            Ok(form) => self.form.widget().set_current(form),
            Err(err) => error!("{}", err),
        };

        match model.generator.get_amplitude(model.source) {
            Ok(amplitude) => self.amplitude.widget().set_value(amplitude as f64),
            Err(err) => error!("{}", err),
        };

        match model.generator.get_offset(model.source) {
            Ok(offset) => self.offset.widget().set_value(offset as f64),
            Err(err) => error!("{}", err),
        };

        match model.generator.get_frequency(model.source) {
            Ok(frequency) => self.frequency.widget().set_value(frequency as f64),
            Err(err) => error!("{}", err),
        };

        match model.generator.get_duty_cycle(model.source) {
            Ok(duty_cycle) => self.duty_cycle.widget().set_value(duty_cycle as f64),
            Err(err) => error!("{}", err),
        };

        match model.generator.get_frequency(model.source) {
            Ok(frequency) => self.frequency.widget().set_value(frequency as f64),
            Err(err) => error!("{}", err),
        };
    }
}

impl ::application::Panel for Widget {
    fn draw(&self, context: &::cairo::Context, model: &::application::Model) {
        if !self.is_started() {
            return;
        }

        context.set_color(self.source.into());

        context.translate(0.0, model.offset(self.source));

        context.move_to(model.scales.h.0, 0.0);
        context.line_to(model.scales.h.1, 0.0);
        context.stroke();

        self.draw_data(&context, model.scales);
    }
}
