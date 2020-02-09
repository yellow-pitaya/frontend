mod input;

use crate::widget::radio::Msg::*;
use gtk::prelude::*;
use input::Msg::*;
use input::Widget as InputWidget;

type RateWidget = crate::widget::RadioGroup<redpitaya_scpi::acquire::SamplingRate>;

#[derive(relm_derive::Msg, Clone)]
pub enum Msg {
    Attenuation(redpitaya_scpi::acquire::Source, u8),
    Average(bool),
    Gain(
        redpitaya_scpi::acquire::Source,
        redpitaya_scpi::acquire::Gain,
    ),
    Rate(redpitaya_scpi::acquire::SamplingRate),
    SetData(redpitaya_scpi::acquire::Source, Vec<f64>),
    Start(redpitaya_scpi::acquire::Source),
    Stop(redpitaya_scpi::acquire::Source),
    Redraw(Box<cairo::Context>, Box<crate::application::Model>),
}

#[relm_derive::widget(clone)]
impl relm::Widget for Widget {
    fn model(
        _: &relm::Relm<Self>,
        model: redpitaya_scpi::acquire::Acquire,
    ) -> redpitaya_scpi::acquire::Acquire {
        model
    }

    fn update(&mut self, event: Msg) {
        match event {
            Msg::Average(enable) => {
                if enable {
                    self.model.enable_average();
                } else {
                    self.model.disable_average();
                }
            }
            Msg::Rate(rate) => self.model.set_decimation(rate.into()),
            Msg::SetData(source, data) => {
                self.get_input(source).emit(input::Msg::SetData(data))
            }
            Msg::Redraw(ref context, ref model) => self.draw(context, model),
            _ => (),
        };
    }

    view! {
        gtk::Box {
            orientation: gtk::Orientation::Vertical,
            spacing: 10,
            #[name="rate"]
            RateWidget(crate::widget::radio::Model {
                options: vec![
                    redpitaya_scpi::acquire::SamplingRate::RATE_1_9kHz,
                    redpitaya_scpi::acquire::SamplingRate::RATE_15_2kHz,
                    redpitaya_scpi::acquire::SamplingRate::RATE_103_8kHz,
                    redpitaya_scpi::acquire::SamplingRate::RATE_1_9MHz,
                    redpitaya_scpi::acquire::SamplingRate::RATE_15_6MHz,
                    redpitaya_scpi::acquire::SamplingRate::RATE_125MHz,
                ],
                current: match self.model.get_decimation() {
                    Ok(decimation) => Some(decimation.into()),
                    Err(_) => None,
                },
            }) {
                label: Some("Sampling Rate"),
                Change(rate) => Msg::Rate(rate),
            },
            #[name="average"]
            gtk::CheckButton {
                label: "Average",
                active: self.model.is_average_enabled(),
                toggled(w) => Msg::Average(w.get_active()),
            },
            #[name="in1"]
            InputWidget(input::Model::new(
                    self.model,
                    redpitaya_scpi::acquire::Source::IN1,
            )) {
                Attenuation(attenuation) => Msg::Attenuation(redpitaya_scpi::acquire::Source::IN1, attenuation),
                Gain(gain) => Msg::Gain(redpitaya_scpi::acquire::Source::IN1, gain),
                Start => Msg::Start(redpitaya_scpi::acquire::Source::IN1),
                Stop => Msg::Stop(redpitaya_scpi::acquire::Source::IN1),
            },
            #[name="in2"]
            InputWidget(input::Model::new(
                    self.model,
                    redpitaya_scpi::acquire::Source::IN2,
            )) {
                Attenuation(attenuation) => Msg::Attenuation(redpitaya_scpi::acquire::Source::IN2, attenuation),
                Gain(gain) => Msg::Gain(redpitaya_scpi::acquire::Source::IN2, gain),
                Start => Msg::Start(redpitaya_scpi::acquire::Source::IN2),
                Stop => Msg::Stop(redpitaya_scpi::acquire::Source::IN2),
            },
        }
    }
}

impl Widget {
    fn draw(&self, context: &Box<cairo::Context>, model: &Box<crate::application::Model>) {
        context.save();
        self.in1
            .emit(input::Msg::Redraw(context.clone(), model.clone()));
        context.restore();
        context.save();
        self.in2
            .emit(input::Msg::Redraw(context.clone(), model.clone()));
        context.restore();
    }

    fn get_input(
        &self,
        source: redpitaya_scpi::acquire::Source,
    ) -> &relm::Component<input::Widget> {
        match source {
            redpitaya_scpi::acquire::Source::IN1 => &self.in1,
            redpitaya_scpi::acquire::Source::IN2 => &self.in2,
        }
    }
}
