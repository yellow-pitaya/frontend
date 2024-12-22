use crate::color::Colorable as _;
use gtk::prelude::*;
use relm4::ComponentController as _;

#[derive(Debug)]
pub enum InputMsg {
    Amplitude(f32),
    DutyCycle(f32),
    Frequency(u32),
    Offset(f32),
    Form(redpitaya_scpi::generator::Form),
    Redraw(Box<gtk::cairo::Context>, Box<crate::application::Data>),
    Start,
    Stop,
}

#[derive(Debug)]
pub enum OutputMsg {
    Start,
    Stop,
}

pub struct Model {
    amplitude: relm4::Controller<crate::widget::PreciseScale>,
    duty_cycle: relm4::Controller<crate::widget::PreciseScale>,
    form: relm4::Controller<crate::widget::RadioGroup<redpitaya_scpi::generator::Form>>,
    frequency: relm4::Controller<crate::widget::PreciseScale>,
    generator: redpitaya_scpi::generator::Generator,
    offset: relm4::Controller<crate::widget::PreciseScale>,
    palette: relm4::Controller<crate::widget::Palette>,
    source: redpitaya_scpi::generator::Source,
}

#[relm4::component(pub)]
impl relm4::SimpleComponent for Model {
    type Init = (
        redpitaya_scpi::generator::Generator,
        redpitaya_scpi::generator::Source,
    );
    type Input = InputMsg;
    type Output = OutputMsg;

    fn init(
        init: Self::Init,
        _root: Self::Root,
        sender: relm4::ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        use relm4::Component as _;

        let (generator, source) = init;

        let form = crate::widget::RadioGroup::builder()
            .launch(crate::widget::radio::Options {
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
                current: generator.get_form(source).ok(),
                label: "Form",
            })
            .forward(sender.input_sender(), |output| {
                let crate::widget::radio::OutputMsg::Change(form) = output;
                InputMsg::Form(form)
            });

        let palette = crate::widget::Palette::builder()
            .launch((source.to_string(), source.into()))
            .forward(sender.input_sender(), |output| match output {
                crate::widget::palette::OutputMsg::Expand => InputMsg::Start,
                crate::widget::palette::OutputMsg::Fold => InputMsg::Stop,
            });

        let amplitude = crate::widget::PreciseScale::builder()
            .launch(crate::widget::precise::Options {
                label: "Amplitude (V)",
                value: generator.get_amplitude(source).unwrap() as f64,
                digits: 2,
                adjustment: gtk::Adjustment::new(0.0, -1.0, 1.0, 0.1, 1.0, 0.0),
            })
            .forward(sender.input_sender(), |output| {
                let crate::widget::precise::OutputMsg::Change(amplitude) = output;
                InputMsg::Amplitude(amplitude as f32)
            });

        let offset = crate::widget::PreciseScale::builder()
            .launch(crate::widget::precise::Options {
                label: "Offest (V)",
                value: generator.get_offset(source).unwrap() as f64,
                digits: 2,
                adjustment: gtk::Adjustment::new(0.0, -1.0, 1.0, 0.1, 1.0, 0.0),
            })
            .forward(sender.input_sender(), |output| {
                let crate::widget::precise::OutputMsg::Change(offset) = output;
                InputMsg::Offset(offset as f32)
            });

        let frequency = crate::widget::PreciseScale::builder()
            .launch(crate::widget::precise::Options {
                label: "Frequency (Hz)",
                value: generator.get_frequency(source).unwrap() as f64,
                digits: 0,
                adjustment: gtk::Adjustment::new(0.0, 0.0, 62_500_000.0, 1_000.0, 10_000.0, 0.0),
            })
            .forward(sender.input_sender(), |output| {
                let crate::widget::precise::OutputMsg::Change(frequency) = output;
                InputMsg::Frequency(frequency as u32)
            });

        let duty_cycle = crate::widget::PreciseScale::builder()
            .launch(crate::widget::precise::Options {
                label: "Duty cycle (%)",
                value: generator.get_duty_cycle(source).unwrap() as f64,
                digits: 2,
                adjustment: gtk::Adjustment::new(0.0, 0.0, 1.0, 0.1, 1.0, 0.0),
            })
            .forward(sender.input_sender(), |output| {
                let crate::widget::precise::OutputMsg::Change(duty_cycle) = output;
                InputMsg::DutyCycle(duty_cycle as f32)
            });

        let model = Self {
            amplitude,
            duty_cycle,
            form,
            frequency,
            generator,
            offset,
            palette,
            source,
        };

        let widgets = view_output!();

        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 10);
        model.palette.widgets().container_add(&vbox);

        vbox.append(model.form.widget());
        vbox.append(model.amplitude.widget());
        vbox.append(model.offset.widget());
        vbox.append(model.frequency.widget());
        vbox.append(model.duty_cycle.widget());

        relm4::ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: relm4::ComponentSender<Self>) {
        use InputMsg::*;

        match msg {
            Amplitude(value) => self.generator.set_amplitude(self.source, value),
            Offset(value) => self.generator.set_offset(self.source, value),
            Frequency(value) => self.generator.set_frequency(self.source, value),
            DutyCycle(value) => self.generator.set_duty_cycle(self.source, value),
            Start => {
                self.generator.start(self.source);
                sender.output(OutputMsg::Start).ok();
            }
            Stop => {
                self.generator.stop(self.source);
                sender.output(OutputMsg::Stop).ok();
            }
            Redraw(ref context, ref model) => self.draw(context, model).unwrap(),
            Form(form) => {
                let is_pwm = form == redpitaya_scpi::generator::Form::PWM;
                self.duty_cycle.widget().set_visible(is_pwm);
                self.generator.set_form(self.source, form);
            }
        };
    }

    view! {
        #[name="page"]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 10,

            append: model.palette.widget(),
        },
    }
}

impl Model {
    fn is_started(&self) -> bool {
        self.generator.is_started(self.source)
    }

    fn draw_data(
        &self,
        context: &gtk::cairo::Context,
        scales: crate::Scales,
    ) -> Result<(), gtk::cairo::Error> {
        context.set_line_width(0.05);

        if let Ok(form) = self.generator.get_form(self.source) {
            let amplitude = self
                .generator
                .get_amplitude(self.source)
                .unwrap_or_default();
            let frequency = self
                .generator
                .get_frequency(self.source)
                .unwrap_or_default() as f32;
            let duty_cycle = self
                .generator
                .get_duty_cycle(self.source)
                .unwrap_or_default();
            let offset = self.generator.get_offset(self.source).unwrap_or_default();

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

                y = y.clamp(-1.0, 1.0);

                context.line_to(x.into(), y.into());
                context.move_to(x.into(), y.into());
            }

            context.stroke()?;
        }

        Ok(())
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

    fn draw(
        &self,
        context: &gtk::cairo::Context,
        data: &crate::application::Data,
    ) -> Result<(), gtk::cairo::Error> {
        if !self.is_started() {
            return Ok(());
        }

        context.set_color(self.source.into());

        context.translate(0.0, data.offset(self.source));

        context.move_to(data.scales.h.0, 0.0);
        context.line_to(data.scales.h.1, 0.0);
        context.stroke()?;

        self.draw_data(context, data.scales)
    }
}
