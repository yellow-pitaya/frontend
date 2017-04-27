use application::color::Colorable;
use gtk::{
    BoxExt,
    ButtonExt,
    ComboBoxExt,
    WidgetExt,
};
use relm::ContainerWidget;

pub enum Mode {
    Auto = 0,
    Normal = 1,
    Single = 2,
}

impl ::std::convert::From<i32> for Mode {
    fn from(x: i32) -> Self {
        match x {
            0 => Mode::Auto,
            1 => Mode::Normal,
            2 => Mode::Single,
            _ => Mode::Auto,
        }
    }
}

impl ::std::convert::From<Mode> for i32 {
    fn from(x: Mode) -> Self {
        match x {
            Mode::Auto => 0,
            Mode::Normal => 1,
            Mode::Single => 2,
        }
    }
}

#[derive(Msg)]
pub enum Signal {
    Auto,
    InternalTick,
    Normal,
    Single,
    Mode,
    Delay(u16),
    Level(f32),
}

#[derive(Clone)]
pub struct Widget {
    page: ::gtk::Box,
    pub level: ::relm::Component<::widget::PreciseScale>,
    single_button: ::gtk::Button,
    pub mode_combo: ::gtk::ComboBoxText,
    pub delay: ::relm::Component<::widget::PreciseScale>,
    stream: ::relm::EventStream<Signal>,
}

impl Widget {
    pub fn set_mode(&self, mode: Mode) {
        self.mode_combo.set_active(mode.into());
    }

    fn get_mode(&self) -> Mode {
        self.mode_combo.get_active()
            .into()
    }
}

impl ::relm::Widget for Widget {
    type Model = ();
    type Msg = Signal;
    type Root = ::gtk::Box;

    fn model() -> Self::Model {
    }

    fn root(&self) -> &Self::Root {
        &self.page
    }

    fn update(&mut self, event: Signal, _: &mut Self::Model) {
        match event {
            Signal::InternalTick => {
                match self.get_mode() {
                    Mode::Auto => self.stream.emit(Signal::Auto),
                    Mode::Normal => self.stream.emit(Signal::Normal),
                    Mode::Single => (),
                };
            },
            Signal::Mode => {
                match self.get_mode() {
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

    fn view(relm: ::relm::RemoteRelm<Signal>, _: &Self::Model) -> Self {
        let page = ::gtk::Box::new(::gtk::Orientation::Vertical, 10);

        let mode_combo = ::gtk::ComboBoxText::new();
        mode_combo.append_text("Auto");
        mode_combo.append_text("Normal");
        mode_combo.append_text("Single");
        page.pack_start(&mode_combo, false, false, 0);
        connect!(relm, mode_combo, connect_changed(_), Signal::Mode);

        let single_button = ::gtk::Button::new_with_label("Single");
        page.pack_start(&single_button, false, false, 0);
        connect!(relm, single_button, connect_clicked(_), Signal::Single);

        let level = page.add_widget::<::widget::PreciseScale, _>(&relm);
        level.widget().set_label("Level (V)");
        level.widget().set_digits(2);
        level.widget().set_adjustment(::gtk::Adjustment::new(
            0.0, -10.0, 10.0, 0.1, 1.0, 0.0
        ));
        connect!(
            level@::widget::Signal::Changed(value),
            relm,
            Signal::Level(value as f32)
        );

        let delay = page.add_widget::<::widget::PreciseScale, _>(&relm);
        delay.widget().set_label("Delay (V)");
        delay.widget().set_digits(2);
        delay.widget().set_adjustment(::gtk::Adjustment::new(
            0.0, 0.0, 131_072.0, 1.1, 10.0, 0.0
        ));
        connect!(
            delay@::widget::Signal::Changed(value),
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

        Widget {
            page: page,
            single_button: single_button,
            delay: delay,
            level: level,
            mode_combo: mode_combo,
            stream: relm.stream().clone(),
        }
    }
}

impl ::application::Panel for Widget {
    fn draw(&self, context: &::cairo::Context, scales: ::Scales) {
        context.set_color(::application::color::TRIGGER);

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
