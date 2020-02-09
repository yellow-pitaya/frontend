mod input;

use gtk::{
    ContainerExt,
    ToggleButtonExt,
};
use relm::ContainerWidget;
use input::Model as InputModel;
use input::Widget as InputWidget;
use input::Signal as InputSignal;

#[derive(relm_derive::Msg, Clone)]
pub enum Signal {
    Attenuation(redpitaya_scpi::acquire::Source, u8),
    Average(bool),
    Gain(redpitaya_scpi::acquire::Source, redpitaya_scpi::acquire::Gain),
    Rate(redpitaya_scpi::acquire::SamplingRate),
    SetData(redpitaya_scpi::acquire::Source, Vec<f64>),
    Start(redpitaya_scpi::acquire::Source),
    Stop(redpitaya_scpi::acquire::Source),
    Redraw(Box<cairo::Context>, Box<crate::application::Model>),
}

#[derive(Clone)]
pub struct Widget {
    model: redpitaya_scpi::acquire::Acquire,
    vbox: gtk::Box,
    in1: relm::Component<InputWidget>,
    in2: relm::Component<InputWidget>,
    rate: relm::Component<crate::widget::RadioGroup<redpitaya_scpi::acquire::SamplingRate>>,
    average: gtk::CheckButton,
}

impl relm::Update for Widget {
    type Model = redpitaya_scpi::acquire::Acquire;
    type ModelParam = redpitaya_scpi::acquire::Acquire;
    type Msg = Signal;

    fn model(_: &relm::Relm<Self>, model: Self::ModelParam) -> Self::Model {
        model
    }

    fn update(&mut self, event: Self::Msg) {
        match event {
            Signal::Average(enable) => if enable {
                self.model.enable_average();
            } else {
                self.model.disable_average();
            },
            Signal::Rate(rate) => self.model.set_decimation(rate.into()),
            Signal::SetData(source, data) => self.set_data(source, data),
            Signal::Redraw(ref context, ref model) => self.draw(context, model),
            _ => (),
        };
    }
}

impl relm::Widget for Widget {
    type Root = gtk::Box;

    fn root(&self) -> Self::Root {
        self.vbox.clone()
    }

    fn view(relm: &relm::Relm<Self>, model: Self::Model) -> Self {
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 10);

        let args = crate::widget::radio::Model {
            title: String::from("Sampling Rate"),
            options: vec![
                redpitaya_scpi::acquire::SamplingRate::RATE_1_9kHz,
                redpitaya_scpi::acquire::SamplingRate::RATE_15_2kHz,
                redpitaya_scpi::acquire::SamplingRate::RATE_103_8kHz,
                redpitaya_scpi::acquire::SamplingRate::RATE_1_9MHz,
                redpitaya_scpi::acquire::SamplingRate::RATE_15_6MHz,
                redpitaya_scpi::acquire::SamplingRate::RATE_125MHz,
            ],
            current: match model.get_decimation() {
                Ok(sampling_rate) => Some(sampling_rate.into()),
                Err(_) => None,
            },
        };
        let rate = vbox.add_widget::<crate::widget::RadioGroup<redpitaya_scpi::acquire::SamplingRate>>(args);
        relm::connect!(
            rate@crate::widget::radio::Signal::Change(rate),
            relm,
            Signal::Rate(rate)
        );

        let average = gtk::CheckButton::new_with_label("Average");
        average.set_active(model.is_average_enabled());
        vbox.add(&average);
        relm::connect!(
            relm, average, connect_toggled(w), Signal::Average(w.get_active())
        );

        let in1 = vbox.add_widget::<InputWidget>(InputModel {
            attenuation: 1,
            started: false,
            source: redpitaya_scpi::acquire::Source::IN1,
            acquire: model.clone()
        });
        relm::connect!(in1@InputSignal::Attenuation(attenuation), relm, Signal::Attenuation(redpitaya_scpi::acquire::Source::IN1, attenuation));
        relm::connect!(in1@InputSignal::Gain(gain), relm, Signal::Gain(redpitaya_scpi::acquire::Source::IN1, gain));
        relm::connect!(in1@InputSignal::Start, relm, Signal::Start(redpitaya_scpi::acquire::Source::IN1));
        relm::connect!(in1@InputSignal::Stop, relm, Signal::Stop(redpitaya_scpi::acquire::Source::IN1));

        let in2 = vbox.add_widget::<InputWidget>(InputModel {
            attenuation: 1,
            started: false,
            source: redpitaya_scpi::acquire::Source::IN2,
            acquire: model.clone()
        });
        relm::connect!(in2@InputSignal::Attenuation(attenuation), relm, Signal::Attenuation(redpitaya_scpi::acquire::Source::IN2, attenuation));
        relm::connect!(in2@InputSignal::Gain(gain), relm, Signal::Gain(redpitaya_scpi::acquire::Source::IN2, gain));
        relm::connect!(in2@InputSignal::Start, relm, Signal::Start(redpitaya_scpi::acquire::Source::IN2));
        relm::connect!(in2@InputSignal::Stop, relm, Signal::Stop(redpitaya_scpi::acquire::Source::IN2));

        Widget {
            model,
            vbox,
            in1,
            in2,
            rate,
            average,
        }
    }
}

impl Widget {
    fn draw(&self, context: &Box<cairo::Context>, model: &Box<crate::application::Model>) {
        context.save();
        self.in1.emit(InputSignal::Redraw(context.clone(), model.clone()));
        context.restore();
        context.save();
        self.in2.emit(InputSignal::Redraw(context.clone(), model.clone()));
        context.restore();
    }

    fn set_data(&self, source: redpitaya_scpi::acquire::Source, data: Vec<f64>) {
        self.get_input(source)
            .emit(input::Signal::SetData(data));
    }

    fn get_input(&self, source: redpitaya_scpi::acquire::Source) -> &relm::Component<InputWidget> {
        match source {
            redpitaya_scpi::acquire::Source::IN1 => &self.in1,
            redpitaya_scpi::acquire::Source::IN2 => &self.in2,
        }
    }
}
