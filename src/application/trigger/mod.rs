pub mod channel;
pub mod edge;
pub mod mode;

pub use self::channel::Channel;
pub use self::edge::Edge;
pub use self::mode::Mode;

use crate::color::Colorable;
use gtk::prelude::*;
use relm::ContainerWidget;

#[derive(relm_derive::Msg, Clone)]
pub enum Signal {
    Auto,
    Normal,
    Single,
    Mode(Mode),
    Channel(Channel),
    Source(redpitaya_scpi::trigger::Source),
    Edge(Edge),
    InternalTick,
    Redraw(Box<cairo::Context>, Box<crate::application::Model>),
}

#[derive(Clone)]
pub struct Model {
    pub trigger: redpitaya_scpi::trigger::Trigger,
    pub channel: Option<Channel>,
    pub edge: Option<Edge>,
    pub mode: Mode,
}

#[derive(Clone)]
pub struct Widget {
    model: Model,
    page: gtk::Box,
    pub single_button: gtk::Button,
    stream: relm::EventStream<<Self as relm::Update>::Msg>,
    mode: relm::Component<crate::widget::RadioGroup<Mode>>,
    channel: relm::Component<crate::widget::RadioGroup<Channel>>,
    edge: relm::Component<crate::widget::RadioGroup<Edge>>,
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

    fn draw(&self, context: &cairo::Context, model: &crate::application::Model) {
        if self.model.mode == Mode::Normal || self.model.mode == Mode::Single {
            let width = model.scales.get_width();
            let height = model.scales.get_height();
            let delay = model.offset("DELAY");
            let trigger = model.offset("TRIG");

            context.set_color(crate::color::TRIGGER);

            context.set_line_width(width / 1000.0);
            context.move_to(delay, model.scales.v.0);
            context.line_to(delay, model.scales.v.1);
            context.stroke();

            context.set_line_width(height / 1000.0);
            context.move_to(model.scales.h.0, trigger);
            context.line_to(model.scales.h.1, trigger);
            context.stroke();
        }
    }
}

impl relm::Update for Widget {
    type Model = Model;
    type Msg = Signal;
    type ModelParam = redpitaya_scpi::trigger::Trigger;

    fn model(_: &relm::Relm<Self>, trigger: Self::ModelParam) -> Self::Model {
        Self::Model {
            trigger,
            mode: Mode::Normal,
            edge: None,
            channel: None,
        }
    }

    fn update(&mut self, event: Self::Msg) {
        match event {
            Signal::InternalTick => {
                match self.model.mode {
                    Mode::Auto => self.stream.emit(Signal::Auto),
                    Mode::Normal => self.stream.emit(Signal::Normal),
                    Mode::Single => (),
                };
            }
            Signal::Mode(mode) => {
                self.model.mode = mode;

                match mode {
                    Mode::Auto => self.single_button.set_visible(false),
                    Mode::Normal => self.single_button.set_visible(false),
                    Mode::Single => self.single_button.set_visible(true),
                };
            }
            Signal::Channel(channel) => {
                self.model.channel = Some(channel);
                if let Some(source) = self.get_source() {
                    self.stream.emit(Signal::Source(source));
                    self.model.trigger.enable(source);
                }
            }
            Signal::Edge(edge) => {
                self.model.edge = Some(edge);
                if let Some(source) = self.get_source() {
                    self.stream.emit(Signal::Source(source));
                    self.model.trigger.enable(source);
                }
            }
            Signal::Redraw(ref context, ref model) => self.draw(context, model),
            _ => (),
        }
    }
}

impl relm::Widget for Widget {
    type Root = gtk::Box;

    fn root(&self) -> Self::Root {
        self.page.clone()
    }

    fn view(relm: &relm::Relm<Self>, model: Self::Model) -> Self {
        let page = gtk::Box::new(gtk::Orientation::Vertical, 10);

        let args = crate::widget::radio::Model {
            title: String::from("Source"),
            options: vec![Channel::CH1, Channel::CH2, Channel::EXT],
            current: Some(Channel::CH1),
        };
        let channel = page.add_widget::<crate::widget::RadioGroup<Channel>>(args);
        relm::connect!(
            channel@crate::widget::radio::Signal::Change(channel),
            relm,
            Signal::Channel(channel)
        );

        let args = crate::widget::radio::Model {
            title: String::from("Edge"),
            options: vec![Edge::Positive, Edge::Negative],
            current: Some(Edge::Positive),
        };
        let edge = page.add_widget::<crate::widget::RadioGroup<Edge>>(args);
        relm::connect!(
            edge@crate::widget::radio::Signal::Change(edge),
            relm,
            Signal::Edge(edge)
        );

        let args = crate::widget::radio::Model {
            title: String::from("Mode"),
            options: vec![Mode::Auto, Mode::Normal, Mode::Single],
            current: Some(model.mode),
        };
        let mode = page.add_widget::<crate::widget::RadioGroup<Mode>>(args);
        relm::connect!(
            mode@crate::widget::radio::Signal::Change(mode),
            relm,
            Signal::Mode(mode)
        );

        let single_button = gtk::Button::new_with_label("Single");
        page.pack_start(&single_button, false, false, 0);
        relm::connect!(relm, single_button, connect_clicked(_), Signal::Single);

        let stream = relm.stream().clone();
        GLOBAL.with(move |global| *global.borrow_mut() = Some(stream));

        gtk::timeout_add(1_000, || {
            GLOBAL.with(|global| {
                if let Some(ref stream) = *global.borrow() {
                    stream.emit(Signal::InternalTick);
                }
            });

            glib::Continue(true)
        });

        let stream = relm.stream().clone();

        Widget {
            model,
            page,
            single_button,
            stream,
            mode,
            channel,
            edge,
        }
    }
}

thread_local!(
    static GLOBAL: std::cell::RefCell<Option<relm::EventStream<Signal>>> = std::cell::RefCell::new(None)
);
