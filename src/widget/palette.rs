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

#[derive(Msg)]
pub enum Signal {
    Expand,
    Fold,
}

#[widget]
impl ::relm::Widget for Palette {
    fn model(_: ()) -> () {
    }

    fn update(&mut self, event: Signal, _: &mut Self::Model) {
        match event {
            Signal::Expand => self.parent.show(),
            Signal::Fold => self.parent.hide(),
        };
    }

    fn init_view(&self, _: &mut Self::Model) {
        self.parent.hide();
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
            },
        },
    }
}

impl Palette {
    pub fn set_label(&self, label: &str) {
        self.toggle.set_label(label);
    }

    pub fn add<W>(&self, child: &W) where W: gtk::IsA<gtk::Widget> {
        self.parent.add(child);
    }

    pub fn get_active(&self) -> bool {
        self.toggle.get_active()
    }

    pub fn fold(&self) {
        self.parent.hide();
        self.toggle.set_active(false);
    }

    pub fn set_color(&self, color: ::color::Color) {
        let color = ::gdk_sys::GdkColor {
            pixel: 32,
            red: color.0 as u16 * ::std::u16::MAX,
            green: color.1 as u16 * ::std::u16::MAX,
            blue: color.2 as u16 * ::std::u16::MAX,
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
