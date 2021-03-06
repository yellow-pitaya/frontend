mod output;

use gtk::prelude::*;
use output::Msg::*;
use output::Widget as OutputWidget;

#[derive(relm_derive::Msg, Clone)]
pub enum Msg {
    Amplitude(redpitaya_scpi::generator::Source, f32),
    DutyCycle(redpitaya_scpi::generator::Source, f32),
    Frequency(redpitaya_scpi::generator::Source, u32),
    Offset(redpitaya_scpi::generator::Source, f32),
    Form(
        redpitaya_scpi::generator::Source,
        redpitaya_scpi::generator::Form,
    ),
    Start(redpitaya_scpi::generator::Source),
    Stop(redpitaya_scpi::generator::Source),
    Redraw(Box<gtk::cairo::Context>, Box<crate::application::Model>),
}

#[relm_derive::widget(clone)]
impl relm::Widget for Widget {
    fn model(
        generator: redpitaya_scpi::generator::Generator,
    ) -> redpitaya_scpi::generator::Generator {
        generator
    }

    fn update(&mut self, event: Msg) {
        if let Msg::Redraw(context, model) = event {
            self.draw(context, model).unwrap();
        }
    }

    view! {
        gtk::Box {
            orientation: gtk::Orientation::Vertical,
            spacing: 10,
            #[name="out1"]
            OutputWidget(output::Model::new(self.model, redpitaya_scpi::generator::Source::OUT1)) {
                Amplitude(amplitude) => Msg::Amplitude(redpitaya_scpi::generator::Source::OUT1, amplitude),
                DutyCycle(duty_cycle) => Msg::DutyCycle(redpitaya_scpi::generator::Source::OUT1, duty_cycle),
                Frequency(frequency) => Msg::Frequency(redpitaya_scpi::generator::Source::OUT1, frequency),
                Offset(offset) => Msg::Offset(redpitaya_scpi::generator::Source::OUT1, offset),
                Form(form) => Msg::Form(redpitaya_scpi::generator::Source::OUT1, form),
                Start => Msg::Start(redpitaya_scpi::generator::Source::OUT1),
                Stop => Msg::Stop(redpitaya_scpi::generator::Source::OUT1),
            },
            #[name="out2"]
            OutputWidget(output::Model::new(self.model, redpitaya_scpi::generator::Source::OUT2)) {
                Amplitude(amplitude) => Msg::Amplitude(redpitaya_scpi::generator::Source::OUT2, amplitude),
                DutyCycle(duty_cycle) => Msg::DutyCycle(redpitaya_scpi::generator::Source::OUT2, duty_cycle),
                Frequency(frequency) => Msg::Frequency(redpitaya_scpi::generator::Source::OUT2, frequency),
                Offset(offset) => Msg::Offset(redpitaya_scpi::generator::Source::OUT2, offset),
                Form(form) => Msg::Form(redpitaya_scpi::generator::Source::OUT2, form),
                Start => Msg::Start(redpitaya_scpi::generator::Source::OUT2),
                Stop => Msg::Stop(redpitaya_scpi::generator::Source::OUT2),
            },
        },
    }
}

impl Widget {
    fn draw(
        &self,
        context: Box<gtk::cairo::Context>,
        model: Box<crate::application::Model>,
    ) -> Result<(), gtk::cairo::Error> {
        context.save()?;
        self.components
            .out1
            .emit(output::Msg::Redraw(context.clone(), model.clone()));
        context.restore()?;
        context.save()?;
        self.components
            .out2
            .emit(output::Msg::Redraw(context.clone(), model));
        context.restore()
    }
}
