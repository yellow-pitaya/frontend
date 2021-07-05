pub mod channel;
pub mod edge;
pub mod mode;

pub use self::channel::Channel;
pub use self::edge::Edge;
pub use self::mode::Mode;

use crate::color::Colorable;
use crate::widget::radio::Msg::*;
use gtk::prelude::*;

type ChannelWidget = crate::widget::RadioGroup<Channel>;
type EdgeWidget = crate::widget::RadioGroup<Edge>;
type ModeWidget = crate::widget::RadioGroup<Mode>;

#[derive(relm_derive::Msg, Clone)]
pub enum Msg {
    Auto,
    Normal,
    Single,
    Mode(Mode),
    Channel(Channel),
    Source(redpitaya_scpi::trigger::Source),
    Edge(Edge),
    InternalTick,
    Redraw(Box<gtk::cairo::Context>, Box<crate::application::Model>),
}

#[derive(Clone)]
pub struct Model {
    stream: relm::StreamHandle<Msg>,
    trigger: redpitaya_scpi::trigger::Trigger,
    channel: Option<Channel>,
    edge: Option<Edge>,
    mode: Mode,
}

#[relm_derive::widget(Clone)]
impl relm::Widget for Widget {
    fn model(relm: &relm::Relm<Self>, trigger: redpitaya_scpi::trigger::Trigger) -> Model {
        Self::Model {
            stream: relm.stream().clone(),
            trigger,
            mode: Mode::Normal,
            edge: None,
            channel: None,
        }
    }

    fn subscriptions(&mut self, relm: &relm::Relm<Self>) {
        relm::interval(relm.stream(), 1_000, || Msg::InternalTick);
    }

    fn update(&mut self, event: Msg) {
        match event {
            Msg::InternalTick => {
                match self.model.mode {
                    Mode::Auto => self.model.stream.emit(Msg::Auto),
                    Mode::Normal => self.model.stream.emit(Msg::Normal),
                    Mode::Single => (),
                };
            }
            Msg::Mode(mode) => {
                self.model.mode = mode;

                match mode {
                    Mode::Auto => self.widgets.single_button.set_visible(false),
                    Mode::Normal => self.widgets.single_button.set_visible(false),
                    Mode::Single => self.widgets.single_button.set_visible(true),
                };
            }
            Msg::Channel(channel) => {
                self.model.channel = Some(channel);
                if let Some(source) = self.get_source() {
                    self.model.stream.emit(Msg::Source(source));
                    self.model.trigger.enable(source);
                }
            }
            Msg::Edge(edge) => {
                self.model.edge = Some(edge);
                if let Some(source) = self.get_source() {
                    self.model.stream.emit(Msg::Source(source));
                    self.model.trigger.enable(source);
                }
            }
            Msg::Redraw(ref context, ref model) => self.draw(context, model).unwrap(),
            _ => (),
        }
    }

    view! {
        #[name="page"]
        gtk::Box {
            orientation: gtk::Orientation::Vertical,
            spacing: 10,

            ChannelWidget(crate::widget::radio::Model {
                options: vec![Channel::CH1, Channel::CH2, Channel::EXT],
                current: Some(Channel::CH1),
            }) {
                label: Some("Source"),
                Change(channel) => Msg::Channel(channel),
            },
            EdgeWidget(crate::widget::radio::Model {
                options: vec![Edge::Positive, Edge::Negative],
                current: Some(Edge::Positive),
            }) {
                label: Some("Edge"),
                Change(channel) => Msg::Edge(channel),
            },
            ModeWidget(crate::widget::radio::Model {
                options: vec![Mode::Auto, Mode::Normal, Mode::Single],
                current: Some(self.model.mode),
            }) {
                label: Some("Mode"),
                Change(channel) => Msg::Mode(channel),
            },
            #[name="single_button"]
            gtk::Button {
                child: {
                    pack_type: gtk::PackType::Start,
                    expand: false,
                    fill: false,
                    padding: 0,
                },
                label: "Single",

                clicked(_) => Msg::Single,
            }
        },
    }
}

impl Widget {
    fn get_source(&self) -> Option<redpitaya_scpi::trigger::Source> {
        if self.model.channel == Some(Channel::CH1) && self.model.edge == Some(Edge::Positive) {
            Some(redpitaya_scpi::trigger::Source::CH1_PE)
        } else if self.model.channel == Some(Channel::CH1)
            && self.model.edge == Some(Edge::Negative)
        {
            Some(redpitaya_scpi::trigger::Source::CH1_NE)
        } else if self.model.channel == Some(Channel::CH2)
            && self.model.edge == Some(Edge::Positive)
        {
            Some(redpitaya_scpi::trigger::Source::CH2_PE)
        } else if self.model.channel == Some(Channel::CH2)
            && self.model.edge == Some(Edge::Negative)
        {
            Some(redpitaya_scpi::trigger::Source::CH2_NE)
        } else if self.model.channel == Some(Channel::EXT)
            && self.model.edge == Some(Edge::Positive)
        {
            Some(redpitaya_scpi::trigger::Source::EXT_PE)
        } else if self.model.channel == Some(Channel::EXT)
            && self.model.edge == Some(Edge::Negative)
        {
            Some(redpitaya_scpi::trigger::Source::EXT_NE)
        } else {
            None
        }
    }

    fn draw(&self, context: &gtk::cairo::Context, model: &crate::application::Model) -> Result<(), gtk::cairo::Error> {
        if self.model.mode == Mode::Normal || self.model.mode == Mode::Single {
            let width = model.scales.get_width();
            let height = model.scales.get_height();
            let delay = model.offset("DELAY");
            let trigger = model.offset("TRIG");

            context.set_color(crate::color::TRIGGER);

            context.set_line_width(width / 1000.0);
            context.move_to(delay, model.scales.v.0);
            context.line_to(delay, model.scales.v.1);
            context.stroke()?;

            context.set_line_width(height / 1000.0);
            context.move_to(model.scales.h.0, trigger);
            context.line_to(model.scales.h.1, trigger);
            context.stroke()?;
        }

        Ok(())
    }
}
