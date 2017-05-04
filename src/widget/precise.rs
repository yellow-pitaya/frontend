use relm_attributes::widget;
use gtk::{
    self,
    BoxExt,
    ContainerExt,
    FrameExt,
    OrientableExt,
    RangeExt,
    SpinButtonSignals,
    ToggleButtonExt,
    WidgetExt,
};
use relm::gtk_ext::BoxExtManual;

#[derive(Msg)]
pub enum Signal {
    Expand,
    Fold,
    Changed(f64),
}

#[widget]
impl ::relm::Widget for PreciseScale {
    fn model(_: ()) -> () {
    }

    fn update(&mut self, event: Signal, _: &mut Self::Model) {
        match event {
            Signal::Expand => {
                self.scale.set_draw_value(false);
                self.spin.show();
            },
            Signal::Fold => {
                self.scale.set_draw_value(true);
                self.spin.hide();
            },
            _ => (),
        };
    }

    fn init_view(&self, _: &mut Self::Model) {
        self.spin.hide();
        self.scale.add_mark(0.0, ::gtk::PositionType::Top, None);
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
                        Signal::Expand
                    } else {
                        Signal::Fold
                    }
                },
                gtk::Box {
                    packing: {
                        expand: true,
                        fill: true,
                    },
                    orientation: gtk::Orientation::Vertical,
                    #[name="scale"]
                    gtk::Scale {
                        value_pos: gtk::PositionType::Bottom,
                        change_value(_, _, value) => (Signal::Changed(value), ::gtk::Inhibit(false)),
                    },
                    #[name="spin"]
                    gtk::SpinButton {
                        no_show_all: true,
                        value_changed(w) => Signal::Changed(w.get_value()),
                    },
                },
            },
        },
    }
}

impl PreciseScale {
    pub fn set_adjustment(&self, adjustment: ::gtk::Adjustment) {
        self.scale.set_adjustment(&adjustment);

        adjustment.set_step_increment(
            adjustment.get_step_increment() / 10.0
        );
        adjustment.set_page_increment(
            adjustment.get_page_increment() / 10.0
        );
        self.spin.set_adjustment(&adjustment);
    }

    pub fn set_limit(&self, limit: (f64, f64)) {
        let adjustment = self.scale.get_adjustment();
        adjustment.set_lower(limit.0);
        adjustment.set_upper(limit.1);
        self.scale.set_adjustment(&adjustment);

        let adjustment = self.spin.get_adjustment();
        adjustment.set_lower(limit.0);
        adjustment.set_upper(limit.1);
        self.spin.set_adjustment(&adjustment);
    }

    pub fn set_value(&self, value: f64) {
        self.scale.set_value(value);
    }

    pub fn get_value(&self) -> f64 {
        self.scale.get_value()
    }

    pub fn set_label(&self, label: &str) {
        self.frame.set_label(label);
    }

    pub fn set_visible(&self, visible: bool) {
        self.frame.set_visible(visible);
    }

    pub fn set_digits(&self, digits: u32) {
        self.spin.set_digits(digits);
    }

    pub fn set_no_show_all(&self, no_show_all: bool) {
        self.frame.set_no_show_all(no_show_all);
    }
}
