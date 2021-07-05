use crate::color::Colorable;
use crate::widget::palette::Msg::*;
use crate::widget::precise::Msg::Changed;
use crate::widget::radio::Msg::Change;
use crate::widget::Palette;
use crate::widget::PreciseScale;
use gtk::prelude::*;

type FormWidget = crate::widget::RadioGroup<redpitaya_scpi::generator::Form>;

#[derive(relm_derive::Msg, Clone)]
pub enum Msg {
    Amplitude(f32),
    DutyCycle(f32),
    Frequency(u32),
    Offset(f32),
    Form(redpitaya_scpi::generator::Form),
    Redraw(Box<gtk::cairo::Context>, Box<crate::application::Model>),
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

#[relm_derive::widget(Clone)]
impl relm::Widget for Widget {
    fn model(_: &relm::Relm<Self>, model: Model) -> Model {
        model
    }

    fn update(&mut self, event: Msg) {
        match event {
            Msg::Amplitude(value) => self.model.generator.set_amplitude(self.model.source, value),
            Msg::Offset(value) => self.model.generator.set_offset(self.model.source, value),
            Msg::Frequency(value) => self.model.generator.set_frequency(self.model.source, value),
            Msg::DutyCycle(value) => self
                .model
                .generator
                .set_duty_cycle(self.model.source, value),
            Msg::Start => self.model.generator.start(self.model.source),
            Msg::Stop => self.model.generator.stop(self.model.source),
            Msg::Redraw(ref context, ref model) => self.draw(context, model).unwrap(),
            Msg::Form(form) => {
                let is_pwm = form == redpitaya_scpi::generator::Form::PWM;
                self.components.duty_cycle
                    .emit(crate::widget::precise::Msg::SetVisible(is_pwm));
                self.model.generator.set_form(self.model.source, form);
            }
        };
    }

    view! {
        #[name="page"]
        gtk::Box {
            orientation: gtk::Orientation::Vertical,
            spacing: 10,
            #[name="palette"]
            Palette {
                Expand => Msg::Start,
                Fold => Msg::Stop,

                gtk::Box {
                    orientation: gtk::Orientation::Vertical,
                    spacing: 10,

                    #[name="form"]
                    FormWidget(crate::widget::radio::Model {
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
                    }) {
                        label: Some("Form"),
                        Change(form) => Msg::Form(form),
                    },
                    #[name="amplitude"]
                    PreciseScale {
                        label: Some("Amplitude (V)"),
                        Changed(amplitude) => Msg::Amplitude(amplitude as f32),
                    },
                    #[name="offset"]
                    PreciseScale {
                        label: Some("Offset (V)"),
                        Changed(offset) => Msg::Offset(offset as f32),
                    },
                    #[name="frequency"]
                    PreciseScale {
                        label: Some("Frequency (Hz)"),
                        Changed(frequency) => Msg::Frequency(frequency as u32),
                    },
                    #[name="duty_cycle"]
                    PreciseScale {
                        label: Some("Duty cycle (%)"),
                        Changed(duty_cycle) => Msg::DutyCycle(duty_cycle as f32),
                    },
                },
            },
        },
    }

    fn init_view(&mut self) {
        // @FIXME
        self.components.palette.emit(crate::widget::palette::Msg::Fold);

        self.components.palette
            .emit(crate::widget::palette::Msg::SetLabel(format!(
                "{}",
                self.model.source
            )));
        self.components.palette.emit(crate::widget::palette::Msg::SetColor(
            self.model.source.into(),
        ));

        self.components.amplitude
            .emit(crate::widget::precise::Msg::SetDigits(2));
        self.components.amplitude
            .emit(crate::widget::precise::Msg::SetAdjustement(
                gtk::Adjustment::new(0.0, -1.0, 1.0, 0.1, 1.0, 0.0),
            ));

        self.components.offset.emit(crate::widget::precise::Msg::SetDigits(2));
        self.components.offset
            .emit(crate::widget::precise::Msg::SetAdjustement(
                gtk::Adjustment::new(0.0, -1.0, 1.0, 0.1, 1.0, 0.0),
            ));

        self.components.frequency
            .emit(crate::widget::precise::Msg::SetAdjustement(
                gtk::Adjustment::new(0.0, 0.0, 62_500_000.0, 1_000.0, 10_000.0, 0.0),
            ));

        self.components.duty_cycle
            .emit(crate::widget::precise::Msg::SetNoShowAll(true));
        self.components.duty_cycle
            .emit(crate::widget::precise::Msg::SetVisible(false));
        self.components.duty_cycle
            .emit(crate::widget::precise::Msg::SetDigits(2));
        self.components.duty_cycle
            .emit(crate::widget::precise::Msg::SetAdjustement(
                gtk::Adjustment::new(0.0, 0.0, 1.0, 0.1, 1.0, 0.0),
            ));

        match self.model.generator.get_form(self.model.source) {
            Ok(form) => self.components.form.emit(crate::widget::radio::Msg::Set(form)),
            Err(err) => log::error!("{}", err),
        };

        match self.model.generator.get_amplitude(self.model.source) {
            Ok(amplitude) => self.components.amplitude
                .emit(crate::widget::precise::Msg::SetValue(amplitude as f64)),
            Err(err) => log::error!("{}", err),
        };

        match self.model.generator.get_offset(self.model.source) {
            Ok(offset) => self.components.offset
                .emit(crate::widget::precise::Msg::SetValue(offset as f64)),
            Err(err) => log::error!("{}", err),
        };

        match self.model.generator.get_duty_cycle(self.model.source) {
            Ok(duty_cycle) => self.components.duty_cycle
                .emit(crate::widget::precise::Msg::SetValue(duty_cycle as f64)),
            Err(err) => log::error!("{}", err),
        };

        match self.model.generator.get_frequency(self.model.source) {
            Ok(frequency) => self.components.frequency
                .emit(crate::widget::precise::Msg::SetValue(frequency as f64)),
            Err(err) => log::error!("{}", err),
        };
    }
}

impl Widget {
    fn is_started(&self) -> bool {
        self.model.generator.is_started(self.model.source)
    }

    fn draw_data(&self, context: &gtk::cairo::Context, scales: crate::Scales) -> Result<(), gtk::cairo::Error> {
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

    fn draw(&self, context: &gtk::cairo::Context, model: &crate::application::Model) -> Result<(), gtk::cairo::Error> {
        if !self.is_started() {
            return Ok(());
        }

        context.set_color(self.model.source.into());

        context.translate(0.0, model.offset(self.model.source));

        context.move_to(model.scales.h.0, 0.0);
        context.line_to(model.scales.h.1, 0.0);
        context.stroke()?;

        self.draw_data(&context, model.scales)
    }
}
