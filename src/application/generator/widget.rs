use gtk::{
    self,
    BoxExt,
    OrientableExt,
    WidgetExt,
};
use relm_attributes::widget;
use super::Signal;
use super::output::Widget as OutputWidget;
use super::output::Model as OutputModel;
use super::output::Signal::{
    Amplitude,
    DutyCycle,
    Frequency,
    Level,
    Offset,
    Form,
    Start,
    Stop,
};

#[widget]
impl ::relm::Widget for Widget {
    fn model(generator: ::redpitaya_scpi::generator::Generator) -> ::redpitaya_scpi::generator::Generator {
        generator
    }

    fn update(&mut self, _: Signal, _: &mut Self::Model) {
    }

    view! {
        gtk::Box {
            orientation: gtk::Orientation::Vertical,
            spacing: 10,
            #[name="out1"]
            OutputWidget(OutputModel {
                source: ::redpitaya_scpi::generator::Source::OUT1,
                generator: model.clone(),
            }) {
                Amplitude(amplitude) => Signal::Amplitude(::redpitaya_scpi::generator::Source::OUT1, amplitude),
                DutyCycle(duty_cycle) => Signal::DutyCycle(::redpitaya_scpi::generator::Source::OUT1, duty_cycle),
                Frequency(frequency) => Signal::Frequency(::redpitaya_scpi::generator::Source::OUT1, frequency),
                Level(level) => Signal::Level(::redpitaya_scpi::generator::Source::OUT1, level),
                Offset(offset) => Signal::Offset(::redpitaya_scpi::generator::Source::OUT1, offset),
                Form(form) => Signal::Form(::redpitaya_scpi::generator::Source::OUT1, form),
                Start => Signal::Start(::redpitaya_scpi::generator::Source::OUT1),
                Stop => Signal::Stop(::redpitaya_scpi::generator::Source::OUT1),
            },
            #[name="out2"]
            OutputWidget(OutputModel {
                source: ::redpitaya_scpi::generator::Source::OUT2,
                generator: model.clone(),
            }) {
                Amplitude(amplitude) => Signal::Amplitude(::redpitaya_scpi::generator::Source::OUT2, amplitude),
                DutyCycle(duty_cycle) => Signal::DutyCycle(::redpitaya_scpi::generator::Source::OUT2, duty_cycle),
                Frequency(frequency) => Signal::Frequency(::redpitaya_scpi::generator::Source::OUT2, frequency),
                Level(level) => Signal::Level(::redpitaya_scpi::generator::Source::OUT2, level),
                Offset(offset) => Signal::Offset(::redpitaya_scpi::generator::Source::OUT2, offset),
                Form(form) => Signal::Form(::redpitaya_scpi::generator::Source::OUT2, form),
                Start => Signal::Start(::redpitaya_scpi::generator::Source::OUT2),
                Stop => Signal::Stop(::redpitaya_scpi::generator::Source::OUT2),
            },
        },
    }
}

impl ::application::Panel for Widget {
    fn draw(&self, context: &::cairo::Context, model: &::application::Model) {
        context.save();
        self.out1.widget().draw(&context, &model);
        context.restore();
        context.save();
        self.out2.widget().draw(&context, &model);
        context.restore();
    }
}
