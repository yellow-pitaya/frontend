use color::Colorable;
use gtk::{
    BoxExt,
    ButtonExt,
    WidgetExt,
};
use relm::ContainerWidget;

#[derive(Copy, Clone, PartialEq)]
pub enum Mode {
    Auto,
    Normal,
    Single,
}

impl ::std::fmt::Display for Mode {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        let display = match self {
            &Mode::Auto => "Auto",
            &Mode::Normal => "Normal",
            &Mode::Single => "Single",
        };

        write!(f, "{}", display)
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum Channel {
    CH1,
    CH2,
    EXT,
}

impl ::std::fmt::Display for Channel {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        let display = match self {
            &Channel::CH1 => "CH1",
            &Channel::CH2 => "CH2",
            &Channel::EXT => "EXT",
        };

        write!(f, "{}", display)
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum Edge {
    Positive,
    Negative,
}

impl ::std::fmt::Display for Edge {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        let display = match self {
            &Edge::Positive => "Positive",
            &Edge::Negative => "Negative",
        };

        write!(f, "{}", display)
    }
}

#[derive(Msg)]
pub enum Signal {
    Auto,
    Normal,
    Single,
    Mode(Mode),
    Channel(Channel),
    Source(::redpitaya_scpi::trigger::Source),
    Edge(Edge),
    InternalTick,
    Delay(u8),
    Level(f32),
}

#[derive(Clone)]
pub struct Model {
    pub trigger: ::redpitaya_scpi::trigger::Trigger,
    pub mode: Mode,
}

#[derive(Clone)]
pub struct Widget {
    page: ::gtk::Box,
    pub level: ::relm::Component<::widget::PreciseScale>,
    pub single_button: ::gtk::Button,
    pub delay: ::relm::Component<::widget::PreciseScale>,
    stream: ::relm::EventStream<Signal>,
    mode: ::relm::Component<::widget::RadioGroup<Mode>>,
    channel: ::relm::Component<::widget::RadioGroup<Channel>>,
    edge: ::relm::Component<::widget::RadioGroup<Edge>>,
}

impl Widget {
    fn draw_level(&self, context: &::cairo::Context, scales: ::Scales) {
        context.move_to(scales.h.0, self.level.widget().get_value());
        context.line_to(scales.h.1, self.level.widget().get_value());

        context.move_to(self.delay.widget().get_value(), scales.v.0);
        context.line_to(self.delay.widget().get_value(), scales.v.1);

        context.stroke();
    }

    fn get_source(&self) -> Option<::redpitaya_scpi::trigger::Source> {
        let channel = self.channel.widget().get_current();
        let edge = self.edge.widget().get_current();

        if channel == Some(Channel::CH1) && edge == Some(Edge::Positive) {
            Some(::redpitaya_scpi::trigger::Source::CH1_PE)
        } else if channel == Some(Channel::CH1) && edge == Some(Edge::Negative) {
            Some(::redpitaya_scpi::trigger::Source::CH1_NE)
        } else if channel == Some(Channel::CH2) && edge == Some(Edge::Positive) {
            Some(::redpitaya_scpi::trigger::Source::CH2_PE)
        } else if channel == Some(Channel::CH2) && edge == Some(Edge::Negative) {
            Some(::redpitaya_scpi::trigger::Source::CH2_NE)
        } else if channel == Some(Channel::EXT) && edge == Some(Edge::Positive) {
            Some(::redpitaya_scpi::trigger::Source::EXT_PE)
        } else if channel == Some(Channel::EXT) && edge == Some(Edge::Negative) {
            Some(::redpitaya_scpi::trigger::Source::EXT_NE)
        } else {
            None
        }
    }
}

impl ::relm::Widget for Widget {
    type Model = Model;
    type Msg = Signal;
    type Root = ::gtk::Box;
    type ModelParam = ::redpitaya_scpi::trigger::Trigger;

    fn model(trigger: Self::ModelParam) -> Self::Model {
        Model {
            trigger: trigger,
            mode: Mode::Normal,
        }
    }

    fn root(&self) -> &Self::Root {
        &self.page
    }

    fn update(&mut self, event: Signal, model: &mut Self::Model) {
        match event {
            Signal::InternalTick => {
                match model.mode {
                    Mode::Auto => self.stream.emit(Signal::Auto),
                    Mode::Normal => self.stream.emit(Signal::Normal),
                    Mode::Single => (),
                };
            },
            Signal::Mode(mode) => {
                model.mode = mode;

                match mode {
                    Mode::Auto => {
                        self.level.widget().set_visible(false);
                        self.single_button.set_visible(false);
                    },
                    Mode::Normal => {
                        self.level.widget().set_visible(true);
                        self.single_button.set_visible(false);
                    },
                    Mode::Single => {
                        self.level.widget().set_visible(true);
                        self.single_button.set_visible(true);
                    },
                };
            },
            Signal::Channel(_) | Signal::Edge(_) => {
                if let Some(source) = self.get_source() {
                    self.stream.emit(Signal::Source(source));
                    model.trigger.enable(source);
                }
            },
            Signal::Delay(delay) => model.trigger.set_delay_in_ns(delay),
            Signal::Level(level) => model.trigger.set_level(level),
            _ => (),
        }
    }

    fn view(relm: &::relm::RemoteRelm<Self>, model: &Self::Model) -> Self {
        let page = ::gtk::Box::new(::gtk::Orientation::Vertical, 10);

        let args = ::widget::radio::Model {
            title: String::from("Source"),
            options: vec![Channel::CH1, Channel::CH2, Channel::EXT],
            current: Some(Channel::CH1),
        };
        let channel = page.add_widget::<::widget::RadioGroup<Channel>, _>(&relm, args);
        connect!(
            channel@::widget::radio::Signal::Change(channel),
            relm,
            Signal::Channel(channel)
        );

        let args = ::widget::radio::Model {
            title: String::from("Edge"),
            options: vec![Edge::Positive, Edge::Negative],
            current: Some(Edge::Positive),
        };
        let edge = page.add_widget::<::widget::RadioGroup<Edge>, _>(&relm, args);
        connect!(
            edge@::widget::radio::Signal::Change(edge),
            relm,
            Signal::Edge(edge)
        );

        let args = ::widget::radio::Model {
            title: String::from("Mode"),
            options: vec![Mode::Auto, Mode::Normal, Mode::Single],
            current: Some(model.mode),
        };
        let mode = page.add_widget::<::widget::RadioGroup<Mode>, _>(&relm, args);
        connect!(
            mode@::widget::radio::Signal::Change(mode),
            relm,
            Signal::Mode(mode)
        );

        let single_button = ::gtk::Button::new_with_label("Single");
        page.pack_start(&single_button, false, false, 0);
        connect!(relm, single_button, connect_clicked(_), Signal::Single);

        let level = page.add_widget::<::widget::PreciseScale, _>(&relm, ());
        level.widget().set_label("Level (V)");
        level.widget().set_digits(2);
        level.widget().set_adjustment(::gtk::Adjustment::new(
            0.0, -10.0, 10.0, 0.1, 1.0, 0.0
        ));
        connect!(
            level@::widget::precise::Signal::Changed(value),
            relm,
            Signal::Level(value as f32)
        );

        let delay = page.add_widget::<::widget::PreciseScale, _>(&relm, ());
        delay.widget().set_label("Delay (ns)");
        delay.widget().set_digits(2);
        delay.widget().set_adjustment(::gtk::Adjustment::new(
            0.0, 0.0, 131_072.0, 1.1, 10.0, 0.0
        ));
        connect!(
            delay@::widget::precise::Signal::Changed(value),
            relm,
            Signal::Delay(value as u8)
        );

        let stream = relm.stream().clone();
        GLOBAL.with(move |global| {
            *global.borrow_mut() = Some(stream)
        });

        ::gtk::timeout_add(1_000, || {
            GLOBAL.with(|global| {
                if let Some(ref stream) = *global.borrow() {
                    stream.emit(Signal::InternalTick);
                }
            });

            ::glib::Continue(true)
        });

        let stream = relm.stream().clone();

        Widget {
            page,
            single_button,
            delay,
            level,
            stream,
            mode,
            channel,
            edge,
        }
    }
}

impl ::application::Panel for Widget {
    fn draw(&self, context: &::cairo::Context, model: &::application::Model) {
        context.set_color(::color::TRIGGER);

        self.draw_level(context, model.scales);
    }
}

thread_local!(
    static GLOBAL: ::std::cell::RefCell<Option<::relm::EventStream<Signal>>> = ::std::cell::RefCell::new(None)
);
