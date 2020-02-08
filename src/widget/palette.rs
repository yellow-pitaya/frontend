use glib::translate::ToGlibPtr;
use gtk::{
    self,
    ButtonExt,
    ContainerExt,
    OrientableExt,
    ToggleButtonExt,
    WidgetExt,
};
use relm_attributes::widget;

#[derive(Msg, Clone)]
pub enum Signal {
    Expand,
    Fold,
    SetColor(::color::Color),
    SetLabel(String),
}

#[widget]
impl ::relm::Widget for Palette {
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

    pub fn set_color(&self, color: ::color::Color) {
        let color = ::gdk_sys::GdkColor {
            pixel: 32,
            red: color.r as u16 * ::std::u16::MAX,
            green: color.g as u16 * ::std::u16::MAX,
            blue: color.b as u16 * ::std::u16::MAX,
        };

        unsafe {
            ::gtk_sys::gtk_widget_modify_bg(
                self.border.to_glib_none().0,
                ::gtk_sys::GtkStateType::Normal,
                &color
            );
        }
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
