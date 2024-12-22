pub mod channel;
pub mod edge;
pub mod mode;

pub use channel::Channel;
pub use edge::Edge;
pub use mode::Mode;

use crate::color::Colorable;
use gtk::prelude::*;

#[derive(Debug)]
pub enum Command {
    InternalTick,
}

#[derive(Debug)]
pub enum InputMsg {
    Mode(Mode),
    Channel(Channel),
    Edge(Edge),
    Redraw(Box<gtk::cairo::Context>, Box<crate::application::Data>),
}

#[derive(Debug)]
pub enum OutputMsg {
    Auto,
    Normal,
    Single,
}

pub struct Model {
    channel: Option<Channel>,
    channel_widget: relm4::Controller<crate::widget::RadioGroup<Channel>>,
    edge: Option<Edge>,
    edge_widget: relm4::Controller<crate::widget::RadioGroup<Edge>>,
    mode: Mode,
    mode_widget: relm4::Controller<crate::widget::RadioGroup<Mode>>,
    trigger: redpitaya_scpi::trigger::Trigger,
}

#[relm4::component(pub)]
impl relm4::Component for Model {
    type CommandOutput = Command;
    type Init = redpitaya_scpi::trigger::Trigger;
    type Input = InputMsg;
    type Output = OutputMsg;

    fn init(
        init: Self::Init,
        _root: Self::Root,
        sender: relm4::ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        use relm4::ComponentController as _;

        let channel_widget = crate::widget::RadioGroup::builder()
            .launch(crate::widget::radio::Options {
                current: Some(Channel::CH1),
                label: "Source",
                options: vec![Channel::CH1, Channel::CH2, Channel::Ext],
            })
            .forward(sender.input_sender(), |output| {
                let crate::widget::radio::OutputMsg::Change(channel) = output;
                InputMsg::Channel(channel)
            });

        let edge_widget = crate::widget::RadioGroup::builder()
            .launch(crate::widget::radio::Options {
                current: Some(Edge::Positive),
                label: "Edge",
                options: vec![Edge::Positive, Edge::Negative],
            })
            .forward(sender.input_sender(), |output| {
                let crate::widget::radio::OutputMsg::Change(edge) = output;
                InputMsg::Edge(edge)
            });

        let mode_widget = crate::widget::RadioGroup::builder()
            .launch(crate::widget::radio::Options {
                current: Some(Mode::Normal),
                label: "Mode",
                options: vec![Mode::Auto, Mode::Normal, Mode::Single],
            })
            .forward(sender.input_sender(), |output| {
                let crate::widget::radio::OutputMsg::Change(mode) = output;
                InputMsg::Mode(mode)
            });

        let model = Self {
            channel: None,
            channel_widget,
            edge: None,
            edge_widget,
            mode: Mode::Normal,
            mode_widget,
            trigger: init,
        };

        let widgets = view_output!();

        sender.command(|out, shutdown| {
            let fut = shutdown
                .register(async move {
                    loop {
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                        out.send(Command::InternalTick).ok();
                    }
                })
                .drop_on_shutdown();

            Box::pin(fut)
        });

        relm4::ComponentParts { model, widgets }
    }

    fn update_cmd(
        &mut self,
        _: Self::CommandOutput,
        sender: relm4::ComponentSender<Self>,
        _: &Self::Root,
    ) {
        match self.mode {
            Mode::Auto => {
                sender.output(OutputMsg::Auto).ok();
            }
            Mode::Normal => {
                sender.output(OutputMsg::Normal).ok();
            }
            Mode::Single => (),
        }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        msg: Self::Input,
        _: relm4::ComponentSender<Self>,
        _: &Self::Root,
    ) {
        match msg {
            InputMsg::Mode(mode) => {
                self.mode = mode;

                match mode {
                    Mode::Auto => widgets.single_button.set_visible(false),
                    Mode::Normal => widgets.single_button.set_visible(false),
                    Mode::Single => widgets.single_button.set_visible(true),
                };
            }
            InputMsg::Channel(channel) => {
                self.channel = Some(channel);
                if let Some(source) = self.source() {
                    self.trigger.enable(source);
                }
            }
            InputMsg::Edge(edge) => {
                self.edge = Some(edge);
                if let Some(source) = self.source() {
                    self.trigger.enable(source);
                }
            }
            InputMsg::Redraw(ref context, ref model) => self.draw(context, model).unwrap(),
        }
    }

    view! {
        #[name = "page"]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 10,

            append: model.channel_widget.widget(),
            append: model.edge_widget.widget(),
            append: model.mode_widget.widget(),

            #[name = "single_button"]
            gtk::Button {
                set_label: "Single",

                connect_clicked[sender] => move |_| {
                    sender.output(OutputMsg::Single).ok();
                }
            }
        },
    }
}

impl Model {
    fn source(&self) -> Option<redpitaya_scpi::trigger::Source> {
        if self.channel == Some(Channel::CH1) && self.edge == Some(Edge::Positive) {
            Some(redpitaya_scpi::trigger::Source::CH1_PE)
        } else if self.channel == Some(Channel::CH1) && self.edge == Some(Edge::Negative) {
            Some(redpitaya_scpi::trigger::Source::CH1_NE)
        } else if self.channel == Some(Channel::CH2) && self.edge == Some(Edge::Positive) {
            Some(redpitaya_scpi::trigger::Source::CH2_PE)
        } else if self.channel == Some(Channel::CH2) && self.edge == Some(Edge::Negative) {
            Some(redpitaya_scpi::trigger::Source::CH2_NE)
        } else if self.channel == Some(Channel::Ext) && self.edge == Some(Edge::Positive) {
            Some(redpitaya_scpi::trigger::Source::EXT_PE)
        } else if self.channel == Some(Channel::Ext) && self.edge == Some(Edge::Negative) {
            Some(redpitaya_scpi::trigger::Source::EXT_NE)
        } else {
            None
        }
    }

    fn draw(
        &self,
        context: &gtk::cairo::Context,
        data: &crate::application::Data,
    ) -> Result<(), gtk::cairo::Error> {
        if self.mode == Mode::Normal || self.mode == Mode::Single {
            let width = data.scales.width();
            let height = data.scales.height();
            let delay = data.offset("DELAY");
            let trigger = data.offset("TRIG");

            context.set_color(crate::color::TRIGGER);

            context.set_line_width(width / 1000.0);
            context.move_to(delay, data.scales.v.0);
            context.line_to(delay, data.scales.v.1);
            context.stroke()?;

            context.set_line_width(height / 1000.0);
            context.move_to(data.scales.h.0, trigger);
            context.line_to(data.scales.h.1, trigger);
            context.stroke()?;
        }

        Ok(())
    }
}
