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

#[derive(Msg)]
pub enum Signal {
    Auto,
    Normal,
    Single,
    Mode(Mode),
    InternalTick,
    Delay(u16),
    Level(f32),
}

#[derive(Clone)]
pub struct Model {
    mode: Mode,
}

#[derive(Clone)]
pub struct Widget {
    page: ::gtk::Box,
    pub level: ::relm::Component<::widget::PreciseScale>,
    pub single_button: ::gtk::Button,
    pub delay: ::relm::Component<::widget::PreciseScale>,
    stream: ::relm::EventStream<Signal>,
    radio: ::relm::Component<::widget::RadioGroup<Mode>>,
}

impl ::relm::Widget for Widget {
    type Model = Model;
    type Msg = Signal;
    type Root = ::gtk::Box;
    type ModelParam = ();

    fn model(_: Self::ModelParam) -> Self::Model {
        Model {
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
            _ => (),
        }
    }

    fn view(relm: &::relm::RemoteRelm<Self>, model: &Self::Model) -> Self {
        let page = ::gtk::Box::new(::gtk::Orientation::Vertical, 10);

        let frame = ::gtk::Frame::new("Mode");
        page.pack_start(&frame, false, true, 0);

        let args = ::widget::radio::Model {
            options: vec![Mode::Auto, Mode::Normal, Mode::Single],
            current: Some(model.mode),
        };
        let radio = frame.add_widget::<::widget::RadioGroup<Mode>, _>(&relm, args);
        connect!(
            radio@::widget::radio::Signal::Change(mode),
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
        delay.widget().set_label("Delay (V)");
        delay.widget().set_digits(2);
        delay.widget().set_adjustment(::gtk::Adjustment::new(
            0.0, 0.0, 131_072.0, 1.1, 10.0, 0.0
        ));
        connect!(
            delay@::widget::precise::Signal::Changed(value),
            relm,
            Signal::Delay(value as u16)
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
            radio,
        }
    }
}

impl ::application::Panel for Widget {
    fn draw(&self, context: &::cairo::Context, scales: ::Scales) {
        context.set_color(::color::TRIGGER);

        context.move_to(scales.h.0, self.level.widget().get_value());
        context.line_to(scales.h.1, self.level.widget().get_value());

        context.move_to(self.delay.widget().get_value(), scales.v.0);
        context.line_to(self.delay.widget().get_value(), scales.v.1);

        context.stroke();
    }
}

thread_local!(
    static GLOBAL: ::std::cell::RefCell<Option<::relm::EventStream<Signal>>> = ::std::cell::RefCell::new(None)
);
