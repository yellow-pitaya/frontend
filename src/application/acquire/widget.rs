use gtk::{
    ContainerExt,
    ToggleButtonExt,
};
use relm::ContainerWidget;
use super::input::Model as InputModel;
use super::input::Widget as InputWidget;
use super::input::Signal as InputSignal;
use super::signal::Signal;

#[derive(Clone)]
pub struct Widget {
    vbox: ::gtk::Box,
    in1: ::relm::Component<InputWidget>,
    in2: ::relm::Component<InputWidget>,
    rate: ::relm::Component<::widget::RadioGroup<::redpitaya_scpi::acquire::SamplingRate>>,
    average: ::gtk::CheckButton,
}

impl ::relm::Widget for Widget {
    type Model = ::redpitaya_scpi::acquire::Acquire;
    type ModelParam = ::redpitaya_scpi::acquire::Acquire;
    type Msg = Signal;
    type Root = ::gtk::Box;

    fn model(model: Self::ModelParam) -> Self::Model {
        model
    }

    fn root(&self) -> &Self::Root {
        &self.vbox
    }

    fn update(&mut self, event: Self::Msg, acquire: &mut Self::Model) {
        match event {
            Signal::Average(enable) => if enable {
                acquire.enable_average();
            } else {
                acquire.disable_average();
            },
            Signal::Rate(rate) => acquire.set_decimation(rate.into()),
            _ => (),
        };
    }

    fn view(relm: &::relm::RemoteRelm<Self>, acquire: &Self::Model) -> Self {
        let vbox = ::gtk::Box::new(::gtk::Orientation::Vertical, 10);

        let args = ::widget::radio::Model {
            title: String::from("Sampling Rate"),
            options: vec![
                ::redpitaya_scpi::acquire::SamplingRate::RATE_1_9kHz,
                ::redpitaya_scpi::acquire::SamplingRate::RATE_15_2kHz,
                ::redpitaya_scpi::acquire::SamplingRate::RATE_103_8kHz,
                ::redpitaya_scpi::acquire::SamplingRate::RATE_1_9MHz,
                ::redpitaya_scpi::acquire::SamplingRate::RATE_15_6MHz,
                ::redpitaya_scpi::acquire::SamplingRate::RATE_125MHz,
            ],
            current: match acquire.get_decimation() {
                Ok(sampling_rate) => Some(sampling_rate.into()),
                Err(_) => None,
            },
        };
        let rate = vbox.add_widget::<::widget::RadioGroup<::redpitaya_scpi::acquire::SamplingRate>, _>(&relm, args);
        connect!(
            rate@::widget::radio::Signal::Change(rate),
            relm,
            Signal::Rate(rate)
        );

        let average = ::gtk::CheckButton::new_with_label("Average");
        average.set_active(acquire.is_average_enabled());
        vbox.add(&average);
        connect!(
            relm, average, connect_toggled(w), Signal::Average(w.get_active())
        );

        let in1 = vbox.add_widget::<InputWidget, _>(&relm, InputModel {
            source: ::redpitaya_scpi::acquire::Source::IN1,
            acquire: acquire.clone()
        });
        connect!(in1@InputSignal::Attenuation(attenuation), relm, Signal::Attenuation(::redpitaya_scpi::acquire::Source::IN1, attenuation));
        connect!(in1@InputSignal::Data, relm, Signal::Data(::redpitaya_scpi::acquire::Source::IN1));
        connect!(in1@InputSignal::Gain(gain), relm, Signal::Gain(::redpitaya_scpi::acquire::Source::IN1, gain));
        connect!(in1@InputSignal::Level(level), relm, Signal::Level(::redpitaya_scpi::acquire::Source::IN1, level));

        let in2 = vbox.add_widget::<InputWidget, _>(&relm, InputModel {
            source: ::redpitaya_scpi::acquire::Source::IN2,
            acquire: acquire.clone()
        });
        connect!(in2@InputSignal::Attenuation(attenuation), relm, Signal::Attenuation(::redpitaya_scpi::acquire::Source::IN2, attenuation));
        connect!(in2@InputSignal::Data, relm, Signal::Data(::redpitaya_scpi::acquire::Source::IN2));
        connect!(in2@InputSignal::Gain(gain), relm, Signal::Gain(::redpitaya_scpi::acquire::Source::IN2, gain));
        connect!(in2@InputSignal::Level(level), relm, Signal::Level(::redpitaya_scpi::acquire::Source::IN2, level));

        Widget {
            vbox,
            in1,
            in2,
            rate,
            average,
        }
    }
}

impl ::application::Panel for Widget {
    fn draw(&self, context: &::cairo::Context, model: &::application::Model) {
        context.save();
        self.in1.widget().draw(&context, &model);
        context.restore();
        context.save();
        self.in2.widget().draw(&context, &model);
        context.restore();
    }
}

impl Widget {
    pub fn set_buffer(&self, source: ::redpitaya_scpi::acquire::Source, buffer: String) {
        self.get_input(source)
            .set_buffer(buffer);
    }

    fn get_input(&self, source: ::redpitaya_scpi::acquire::Source) -> &InputWidget {
        match source {
            ::redpitaya_scpi::acquire::Source::IN1 => self.in1.widget(),
            ::redpitaya_scpi::acquire::Source::IN2 => self.in2.widget(),
        }
    }
}
