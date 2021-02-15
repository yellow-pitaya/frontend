use gtk::prelude::*;

#[derive(relm_derive::Msg, Clone)]
pub enum Msg {
    Expand,
    Fold,
    Changed(f64),
    SetValue(f64),
    SetVisible(bool),
    SetDigits(u32),
    SetAdjustement(gtk::Adjustment),
    SetNoShowAll(bool),
}

#[relm_derive::widget(Clone)]
impl relm::Widget for PreciseScale {
    fn model(_: ()) {}

    fn update(&mut self, event: Msg) {
        match event {
            Msg::Expand => {
                self.widgets.scale.set_draw_value(false);
                self.widgets.spin.show();
            }
            Msg::Fold => {
                self.widgets.scale.set_draw_value(true);
                self.widgets.spin.hide();
            }
            Msg::SetValue(value) => self.set_value(value),
            Msg::SetVisible(visible) => self.set_no_show_all(visible),
            Msg::SetDigits(digits) => self.set_digits(digits),
            Msg::SetAdjustement(adjustment) => self.set_adjustment(adjustment),
            Msg::SetNoShowAll(no_show_all) => self.set_no_show_all(no_show_all),
            _ => (),
        };
    }

    fn init_view(&mut self) {
        self.widgets.spin.hide();
        self.widgets.scale.add_mark(0.0, gtk::PositionType::Top, None);
    }

    view! {
        #[name="frame"]
        gtk::Frame {
            gtk::Box {
                orientation: gtk::Orientation::Horizontal,
                border_width: 5,
                spacing: 5,
                #[name="toggle"]
                gtk::CheckButton {
                    toggled(w) => if w.get_active() {
                        Msg::Expand
                    } else {
                        Msg::Fold
                    }
                },
                gtk::Box {
                    child: {
                        expand: true,
                        fill: true,
                    },
                    orientation: gtk::Orientation::Vertical,
                    #[name="scale"]
                    gtk::Scale {
                        value_pos: gtk::PositionType::Bottom,
                        change_value(_, _, value) => (Msg::Changed(value), gtk::Inhibit(false)),
                    },
                    #[name="spin"]
                    gtk::SpinButton {
                        no_show_all: true,
                        value_changed(w) => Msg::Changed(w.get_value()),
                    },
                },
            },
        },
    }
}

impl PreciseScale {
    pub fn set_adjustment(&self, adjustment: gtk::Adjustment) {
        self.widgets.scale.set_adjustment(&adjustment);

        adjustment.set_step_increment(adjustment.get_step_increment() / 10.0);
        adjustment.set_page_increment(adjustment.get_page_increment() / 10.0);
        self.widgets.spin.set_adjustment(&adjustment);
    }

    fn set_value(&self, value: f64) {
        self.widgets.scale.set_value(value);
    }

    pub fn set_digits(&self, digits: u32) {
        self.widgets.spin.set_digits(digits);
    }

    pub fn set_no_show_all(&self, no_show_all: bool) {
        self.widgets.frame.set_no_show_all(no_show_all);
    }
}
