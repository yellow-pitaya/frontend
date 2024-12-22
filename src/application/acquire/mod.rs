mod input;

use gtk::prelude::*;
use relm4::ComponentController as _;

pub struct Model {
    in1: relm4::Controller<input::Model>,
    in2: relm4::Controller<input::Model>,
    rate: relm4::Controller<crate::widget::RadioGroup<redpitaya_scpi::acquire::SamplingRate>>,
    rp: redpitaya_scpi::acquire::Acquire,
}

#[derive(Debug)]
pub enum InputMsg {
    Average(bool),
    SetData(redpitaya_scpi::acquire::Source, Vec<f64>),
    Rate(redpitaya_scpi::acquire::SamplingRate),
    Redraw(Box<gtk::cairo::Context>, Box<crate::application::Data>),
}

#[derive(Debug)]
pub enum OutputMsg {
    Rate(redpitaya_scpi::acquire::SamplingRate),
    Start(redpitaya_scpi::acquire::Source),
    Stop(redpitaya_scpi::acquire::Source),
}

#[relm4::component(pub)]
impl relm4::SimpleComponent for Model {
    type Init = redpitaya_scpi::acquire::Acquire;
    type Input = InputMsg;
    type Output = OutputMsg;

    fn init(
        init: Self::Init,
        _root: Self::Root,
        sender: relm4::ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        use relm4::Component as _;

        let rate = crate::widget::RadioGroup::builder()
            .launch(crate::widget::radio::Options {
                options: vec![
                    redpitaya_scpi::acquire::SamplingRate::RATE_1_9kHz,
                    redpitaya_scpi::acquire::SamplingRate::RATE_15_2kHz,
                    redpitaya_scpi::acquire::SamplingRate::RATE_103_8kHz,
                    redpitaya_scpi::acquire::SamplingRate::RATE_1_9MHz,
                    redpitaya_scpi::acquire::SamplingRate::RATE_15_6MHz,
                    redpitaya_scpi::acquire::SamplingRate::RATE_125MHz,
                ],
                current: init.get_decimation().map(Into::into).ok(),
                label: "Samping Rate",
            })
            .forward(sender.input_sender(), |output| {
                let crate::widget::radio::OutputMsg::Change(rate) = output;
                InputMsg::Rate(rate)
            });

        let in1 = input::Model::builder()
            .launch((init.clone(), redpitaya_scpi::acquire::Source::IN1))
            .forward(sender.output_sender(), |output| match output {
                input::OutputMsg::Start => OutputMsg::Start(redpitaya_scpi::acquire::Source::IN1),
                input::OutputMsg::Stop => OutputMsg::Stop(redpitaya_scpi::acquire::Source::IN1),
            });

        let in2 = input::Model::builder()
            .launch((init.clone(), redpitaya_scpi::acquire::Source::IN2))
            .forward(sender.output_sender(), |output| match output {
                input::OutputMsg::Start => OutputMsg::Start(redpitaya_scpi::acquire::Source::IN2),
                input::OutputMsg::Stop => OutputMsg::Stop(redpitaya_scpi::acquire::Source::IN2),
            });

        let model = Self {
            rp: init,
            in1,
            in2,
            rate,
        };

        let widgets = view_output!();

        relm4::ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: relm4::ComponentSender<Self>) {
        match msg {
            InputMsg::Average(enable) => {
                if enable {
                    self.rp.enable_average();
                } else {
                    self.rp.disable_average();
                }
            }
            InputMsg::Rate(rate) => {
                self.rp.set_decimation(rate.into());
                sender.output(OutputMsg::Rate(rate)).ok();
            }
            InputMsg::SetData(source, data) => {
                self.input(source).emit(input::InputMsg::SetData(data))
            }
            InputMsg::Redraw(context, model) => self.draw(context, model).unwrap(),
        };
    }

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 10,

            append: model.rate.widget(),
            #[name = "average"]
            gtk::CheckButton {
                set_label: Some("Average"),
                #[watch]
                set_active: model.rp.is_average_enabled(),

                connect_toggled[sender] => move |this| {
                    sender.input(InputMsg::Average(this.is_active()));
                }
            },
            append: model.in1.widget(),
            append: model.in2.widget(),
        }
    }
}

impl Model {
    fn draw(
        &self,
        context: Box<gtk::cairo::Context>,
        model: Box<crate::application::Data>,
    ) -> Result<(), gtk::cairo::Error> {
        context.save()?;
        self.in1
            .emit(input::InputMsg::Redraw(context.clone(), model.clone()));
        context.restore()?;
        context.save()?;
        self.in2
            .emit(input::InputMsg::Redraw(context.clone(), model));
        context.restore()
    }

    fn input(&self, source: redpitaya_scpi::acquire::Source) -> &relm4::Controller<input::Model> {
        match source {
            redpitaya_scpi::acquire::Source::IN1 => &self.in1,
            redpitaya_scpi::acquire::Source::IN2 => &self.in2,
        }
    }
}
