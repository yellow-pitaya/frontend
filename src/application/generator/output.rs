use crate::color::Colorable;
use relm::ContainerWidget;

#[derive(relm_derive::Msg, Clone)]
pub enum Signal {
    Amplitude(f32),
    DutyCycle(f32),
    Frequency(u32),
    Offset(f32),
    Form(redpitaya_scpi::generator::Form),
    Redraw(Box<cairo::Context>, Box<crate::application::Model>),
    Start,
    Stop,
}

#[derive(Clone)]
pub struct Model {
    generator: redpitaya_scpi::generator::Generator,
    source: redpitaya_scpi::generator::Source,
}

impl Model {
    pub fn new(
        generator: &redpitaya_scpi::generator::Generator,
        source: redpitaya_scpi::generator::Source,
    ) -> Self {
        Self {
            generator: generator.clone(),
            source,
        }
    }
}

#[derive(Clone)]
pub struct Widget {
    model: Model,
    page: gtk::Box,
    palette: relm::Component<crate::widget::Palette>,
    amplitude: relm::Component<crate::widget::PreciseScale>,
    form: relm::Component<crate::widget::RadioGroup<redpitaya_scpi::generator::Form>>,
    offset: relm::Component<crate::widget::PreciseScale>,
    frequency: relm::Component<crate::widget::PreciseScale>,
    duty_cycle: relm::Component<crate::widget::PreciseScale>,
    source: redpitaya_scpi::generator::Source,
}

impl Widget {
    fn is_started(&self) -> bool {
        self.model.generator.is_started(self.model.source)
    }

    fn draw_data(&self, context: &cairo::Context, scales: crate::Scales) {
        context.set_line_width(0.05);

        if let Ok(form) = self.model.generator.get_form(self.model.source) {
            let amplitude = self
                .model
                .generator
                .get_amplitude(self.model.source)
                .unwrap_or_default();
            let frequency = self
                .model
                .generator
                .get_frequency(self.model.source)
                .unwrap_or_default() as f32;
            let duty_cycle = self
                .model
                .generator
                .get_duty_cycle(self.model.source)
                .unwrap_or_default();
            let offset = self
                .model
                .generator
                .get_offset(self.model.source)
                .unwrap_or_default();

            for sample in (scales.h.0 as i32)..(scales.h.1 as i32) {
                let x = scales.x_to_offset(sample) as f32;
                let mut y = match form {
                    redpitaya_scpi::generator::Form::SINE => self.sine(x, amplitude, frequency),
                    redpitaya_scpi::generator::Form::SQUARE => self.square(x, amplitude, frequency),
                    redpitaya_scpi::generator::Form::TRIANGLE => {
                        self.triangle(x, amplitude, frequency)
                    }
                    redpitaya_scpi::generator::Form::SAWU => self.sawu(x, amplitude, frequency),
                    redpitaya_scpi::generator::Form::SAWD => self.sawd(x, amplitude, frequency),
                    redpitaya_scpi::generator::Form::DC => self.dc(x, amplitude, frequency),
                    redpitaya_scpi::generator::Form::PWM => {
                        self.pwm(x, amplitude, frequency, duty_cycle)
                    }
                    _ => unimplemented!(),
                };

                y += offset;

                if y < -1.0 {
                    y = -1.0;
                }

                if y > 1.0 {
                    y = 1.0;
                }

                context.line_to(x.into(), y.into());
                context.move_to(x.into(), y.into());
            }

            context.stroke();
        }
    }

    fn sine(&self, x: f32, amplitude: f32, frequency: f32) -> f32 {
        amplitude * (x * frequency * 2.0 * std::f32::consts::PI).sin()
    }

    fn square(&self, x: f32, amplitude: f32, frequency: f32) -> f32 {
        amplitude * self.sine(x, amplitude, frequency).signum()
    }

    fn triangle(&self, x: f32, amplitude: f32, frequency: f32) -> f32 {
        amplitude * 2.0 / std::f32::consts::PI
            * (frequency * 2.0 * std::f32::consts::PI * x).sin().asin()
    }

    fn sawu(&self, x: f32, amplitude: f32, frequency: f32) -> f32 {
        amplitude * (x * frequency).fract()
    }

    fn sawd(&self, x: f32, amplitude: f32, frequency: f32) -> f32 {
        amplitude * (1.0 - (x * frequency).fract())
    }

    fn dc(&self, _: f32, amplitude: f32, _: f32) -> f32 {
        amplitude
    }

    fn pwm(&self, x: f32, amplitude: f32, frequency: f32, duty_cycle: f32) -> f32 {
        let v = self.sawu(x, amplitude, frequency) - amplitude * duty_cycle;

        if v > 0.0 {
            -amplitude
        } else if v < 0.0 {
            amplitude
        } else {
            0.0
        }
    }

    fn draw(&self, context: &cairo::Context, model: &crate::application::Model) {
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

impl relm::Update for Widget {
    type Model = Model;
    type Msg = Signal;
    type ModelParam = Model;

    fn model(_: &relm::Relm<Self>, model: Self::Model) -> Self::Model {
        model
    }

    fn update(&mut self, event: Self::Msg) {
        match event {
            Signal::Amplitude(value) => {
                self.model.generator.set_amplitude(self.model.source, value)
            }
            Signal::Offset(value) => self.model.generator.set_offset(self.model.source, value),
            Signal::Frequency(value) => {
                self.model.generator.set_frequency(self.model.source, value)
            }
            Signal::DutyCycle(value) => self
                .model
                .generator
                .set_duty_cycle(self.model.source, value),
            Signal::Start => self.model.generator.start(self.model.source),
            Signal::Stop => self.model.generator.stop(self.model.source),
            Signal::Redraw(ref context, ref model) => self.draw(context, model),
            Signal::Form(form) => {
                let is_pwm = form == redpitaya_scpi::generator::Form::PWM;
                self.duty_cycle
                    .emit(crate::widget::precise::Signal::SetVisible(is_pwm));
                self.model.generator.set_form(self.model.source, form);
            }
        };
    }
}

impl relm::Widget for Widget {
    type Root = gtk::Box;

    fn root(&self) -> Self::Root {
        self.page.clone()
    }

    fn view(relm: &relm::Relm<Self>, model: Self::Model) -> Self {
        let source = model.source;

        let page = gtk::Box::new(gtk::Orientation::Vertical, 10);

        let palette = page.add_widget::<crate::widget::Palette>(());
        palette.emit(crate::widget::palette::Signal::SetLabel(format!(
            "{}",
            model.source
        )));
        palette.emit(crate::widget::palette::Signal::SetColor(
            model.source.into(),
        ));
        relm::connect!(palette@crate::widget::palette::Signal::Expand, relm, Signal::Start);
        relm::connect!(palette@crate::widget::palette::Signal::Fold, relm, Signal::Stop);

        use gtk::ContainerExt;

        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 10);
        palette.widget().add(&vbox);

        let args = crate::widget::radio::Model {
            title: String::from("Form"),
            options: vec![
                redpitaya_scpi::generator::Form::SINE,
                redpitaya_scpi::generator::Form::SQUARE,
                redpitaya_scpi::generator::Form::TRIANGLE,
                redpitaya_scpi::generator::Form::SAWU,
                redpitaya_scpi::generator::Form::SAWD,
                redpitaya_scpi::generator::Form::PWM,
                redpitaya_scpi::generator::Form::DC,
                // @TODO redpitaya_scpi::generator::Form::ARBITRARY,
            ],
            current: None,
        };
        let form =
            vbox.add_widget::<crate::widget::RadioGroup<redpitaya_scpi::generator::Form>>(args);
        relm::connect!(
            form@crate::widget::radio::Signal::Change(form),
            relm,
            Signal::Form(form)
        );

        let amplitude = vbox.add_widget::<crate::widget::PreciseScale>(());
        amplitude.emit(crate::widget::precise::Signal::SetLabel(
            "Amplitude (V)".to_string(),
        ));
        amplitude.emit(crate::widget::precise::Signal::SetDigits(2));
        amplitude.emit(crate::widget::precise::Signal::SetAdjustement(
            gtk::Adjustment::new(0.0, -1.0, 1.0, 0.1, 1.0, 0.0),
        ));
        relm::connect!(
            amplitude@crate::widget::precise::Signal::Changed(value),
            relm,
            Signal::Amplitude(value as f32)
        );

        let offset = vbox.add_widget::<crate::widget::PreciseScale>(());
        offset.emit(crate::widget::precise::Signal::SetLabel(
            "Offset (V)".to_string(),
        ));
        offset.emit(crate::widget::precise::Signal::SetDigits(2));
        offset.emit(crate::widget::precise::Signal::SetAdjustement(
            gtk::Adjustment::new(0.0, -1.0, 1.0, 0.1, 1.0, 0.0),
        ));
        relm::connect!(
            offset@crate::widget::precise::Signal::Changed(value),
            relm,
            Signal::Offset(value as f32)
        );

        let frequency = vbox.add_widget::<crate::widget::PreciseScale>(());
        frequency.emit(crate::widget::precise::Signal::SetLabel(
            "Frequency (Hz)".to_string(),
        ));
        frequency.emit(crate::widget::precise::Signal::SetAdjustement(
            gtk::Adjustment::new(0.0, 0.0, 62_500_000.0, 1_000.0, 10_000.0, 0.0),
        ));
        relm::connect!(
            frequency@crate::widget::precise::Signal::Changed(value),
            relm,
            Signal::Frequency(value as u32)
        );

        let duty_cycle = vbox.add_widget::<crate::widget::PreciseScale>(());
        duty_cycle.emit(crate::widget::precise::Signal::SetNoShowAll(true));
        duty_cycle.emit(crate::widget::precise::Signal::SetVisible(false));
        duty_cycle.emit(crate::widget::precise::Signal::SetLabel(
            "Duty cycle (%)".to_string(),
        ));
        duty_cycle.emit(crate::widget::precise::Signal::SetDigits(2));
        duty_cycle.emit(crate::widget::precise::Signal::SetAdjustement(
            gtk::Adjustment::new(0.0, 0.0, 1.0, 0.1, 1.0, 0.0),
        ));
        relm::connect!(
            duty_cycle@crate::widget::precise::Signal::Changed(value),
            relm,
            Signal::DutyCycle(value as f32)
        );

        Widget {
            model,
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

    fn init_view(&mut self) {
        match self.model.generator.get_form(self.model.source) {
            Ok(form) => self.form.emit(crate::widget::radio::Signal::Set(form)),
            Err(err) => log::error!("{}", err),
        };

        match self.model.generator.get_amplitude(self.model.source) {
            Ok(amplitude) => self
                .amplitude
                .emit(crate::widget::precise::Signal::SetValue(amplitude as f64)),
            Err(err) => log::error!("{}", err),
        };

        match self.model.generator.get_offset(self.model.source) {
            Ok(offset) => self
                .offset
                .emit(crate::widget::precise::Signal::SetValue(offset as f64)),
            Err(err) => log::error!("{}", err),
        };

        match self.model.generator.get_duty_cycle(self.model.source) {
            Ok(duty_cycle) => self
                .duty_cycle
                .emit(crate::widget::precise::Signal::SetValue(duty_cycle as f64)),
            Err(err) => log::error!("{}", err),
        };

        match self.model.generator.get_frequency(self.model.source) {
            Ok(frequency) => self
                .frequency
                .emit(crate::widget::precise::Signal::SetValue(frequency as f64)),
            Err(err) => log::error!("{}", err),
        };
    }
}
