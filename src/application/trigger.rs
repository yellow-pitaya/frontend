use application::color::Colorable;
use gtk::{
    BoxExt,
    ButtonExt,
    ContainerExt,
    ComboBoxExt,
    RangeExt,
    WidgetExt,
};

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
    pub level_scale: ::gtk::Scale,
    single_button: ::gtk::Button,
    pub mode_combo: ::gtk::ComboBoxText,
    pub delay_scale: ::gtk::Scale,
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
                        self.level_scale.set_visible(false);
                        self.single_button.set_visible(false);
                    },
                    Mode::Normal => {
                        self.level_scale.set_visible(true);
                        self.single_button.set_visible(false);
                    },
                    Mode::Single => {
                        self.level_scale.set_visible(true);
                        self.single_button.set_visible(true);
                    },
                };
            },
            _ => (),
        }
    }

    fn view(relm: ::relm::RemoteRelm<Signal>, _: &Self::Model) -> Self {
        let page = ::gtk::Box::new(::gtk::Orientation::Vertical, 0);

        let frame = ::gtk::Frame::new("Level");
        page.pack_start(&frame, false, true, 0);

        let level_box = ::gtk::Box::new(::gtk::Orientation::Vertical, 10);
        frame.add(&level_box);

        let mode_combo = ::gtk::ComboBoxText::new();
        mode_combo.append_text("Auto");
        mode_combo.append_text("Normal");
        mode_combo.append_text("Single");
        level_box.pack_start(&mode_combo, false, false, 0);
        connect!(relm, mode_combo, connect_changed(_), Signal::Mode);

        let single_button = ::gtk::Button::new_with_label("Single");
        level_box.pack_start(&single_button, false, false, 0);
        connect!(relm, single_button, connect_clicked(_), Signal::Single);

        let level_scale = ::gtk::Scale::new_with_range(::gtk::Orientation::Horizontal, -10.0, 10.0, 0.1);
        level_scale.add_mark(0.0, ::gtk::PositionType::Top, None);

        level_scale.connect_format_value(move |_, value| {
            format!("{:.2} mV", value)
        });

        let stream = relm.stream().clone();
        level_scale.connect_change_value(move |_, _, value| {
            stream.emit(Signal::Level(value as f32));

            ::gtk::Inhibit(false)
        });
        level_box.pack_start(&level_scale, false, false, 0);

        let frame = ::gtk::Frame::new("Delay");
        page.pack_start(&frame, false, true, 0);

        let delay_scale = ::gtk::Scale::new_with_range(::gtk::Orientation::Horizontal, 0.0, 16384.0, 1.0);
        delay_scale.add_mark(0.0, ::gtk::PositionType::Top, None);

        delay_scale.connect_format_value(move |_, value| {
            format!("{:.0} Sample", value)
        });

        let stream = relm.stream().clone();
        delay_scale.connect_change_value(move |_, _, value| {
            stream.emit(Signal::Delay(value as u16));

            ::gtk::Inhibit(false)
        });
        frame.add(&delay_scale);

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
            delay_scale: delay_scale,
            level_scale: level_scale,
            mode_combo: mode_combo,
            stream: relm.stream().clone(),
        }
    }
}

impl ::application::Panel for Widget {
    fn draw(&self, context: &::cairo::Context, scales: ::application::Scales) {
        context.set_color(::application::color::TRIGGER);

        context.move_to(scales.h.0, self.level_scale.get_value());
        context.line_to(scales.h.1, self.level_scale.get_value());

        context.move_to(self.delay_scale.get_value(), scales.v.0);
        context.line_to(self.delay_scale.get_value(), scales.v.1);

        context.stroke();
    }
}

thread_local!(
    static GLOBAL: ::std::cell::RefCell<Option<::relm::EventStream<Signal>>> = ::std::cell::RefCell::new(None)
);
