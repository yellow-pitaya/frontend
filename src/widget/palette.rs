use gtk::{
    self,
    ButtonExt,
    ContainerExt,
    OrientableExt,
    ToggleButtonExt,
    WidgetExt,
};

#[derive(relm_derive::Msg, Clone)]
pub enum Signal {
    Expand,
    Fold,
    SetColor(crate::color::Color),
    SetLabel(String),
}

#[relm_derive::widget]
impl relm::Widget for Palette {
    fn model(_: ()) -> () {
    }

    fn update(&mut self, event: Signal) {
        match event {
            Signal::Expand => {
                self.parent.set_no_show_all(false);
                self.parent.show_all();
            },
            Signal::Fold => self.parent.hide(),
            Signal::SetColor(color) => self.set_color(color),
            Signal::SetLabel(label) => self.set_label(&label),
        };
    }

    view! {
        gtk::Box {
            orientation: gtk::Orientation::Vertical,
            #[name="border"]
            gtk::EventBox {
                #[name="toggle"]
                gtk::ToggleButton {
                    border_width: 1,
                    toggled(w) => if w.get_active() {
                        Signal::Expand
                    } else {
                        Signal::Fold
                    }
                },
            },
            #[name="parent"]
            gtk::Box {
                orientation: gtk::Orientation::Vertical,
                no_show_all: true,
            },
        },
    }
}

impl Palette {
    pub fn set_label(&self, label: &str) {
        self.toggle.set_label(label);
    }

    pub fn set_color(&self, color: crate::color::Color) {
        let color = gdk::RGBA {
            alpha: 1.,
            red: color.r,
            green: color.g,
            blue: color.b,
        };

        self.border.override_color(
            gtk::StateFlags::NORMAL,
            Some(&color)
        );
    }
}

impl Clone for Palette {
    fn clone(&self) -> Self {
        Self {
            border: self.border.clone(),
            gtkbox7: self.gtkbox7.clone(),
            model: self.model.clone(),
            parent: self.parent.clone(),
            toggle: self.toggle.clone(),
        }
    }
}
