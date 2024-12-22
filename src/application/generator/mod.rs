mod output;

use gtk::prelude::*;
use relm4::ComponentController as _;

#[derive(Debug)]
pub enum InputMsg {
    Redraw(Box<gtk::cairo::Context>, Box<crate::application::Data>),
}

#[derive(Debug)]
pub enum OutputMsg {
    Start(redpitaya_scpi::generator::Source),
    Stop(redpitaya_scpi::generator::Source),
}

pub struct Model {
    out1: relm4::Controller<output::Model>,
    out2: relm4::Controller<output::Model>,
}

#[relm4::component(pub)]
impl relm4::SimpleComponent for Model {
    type Init = redpitaya_scpi::generator::Generator;
    type Input = InputMsg;
    type Output = OutputMsg;

    fn init(
        init: Self::Init,
        _root: Self::Root,
        sender: relm4::ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        use relm4::Component as _;

        let out1 = output::Model::builder()
            .launch((init.clone(), redpitaya_scpi::generator::Source::OUT1))
            .forward(sender.output_sender(), |output| match output {
                output::OutputMsg::Start => {
                    OutputMsg::Start(redpitaya_scpi::generator::Source::OUT1)
                }
                output::OutputMsg::Stop => OutputMsg::Stop(redpitaya_scpi::generator::Source::OUT1),
            });

        let out2 = output::Model::builder()
            .launch((init, redpitaya_scpi::generator::Source::OUT2))
            .forward(sender.output_sender(), |output| match output {
                output::OutputMsg::Start => {
                    OutputMsg::Start(redpitaya_scpi::generator::Source::OUT2)
                }
                output::OutputMsg::Stop => OutputMsg::Stop(redpitaya_scpi::generator::Source::OUT2),
            });

        let model = Self { out1, out2 };

        let widgets = view_output!();

        relm4::ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _: relm4::ComponentSender<Self>) {
        let InputMsg::Redraw(context, model) = msg;

        self.draw(context, model).unwrap();
    }

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 10,

            append: model.out1.widget(),
            append: model.out2.widget(),
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
        self.out1
            .emit(output::InputMsg::Redraw(context.clone(), model.clone()));
        context.restore()?;
        context.save()?;
        self.out2
            .emit(output::InputMsg::Redraw(context.clone(), model));
        context.restore()
    }
}
