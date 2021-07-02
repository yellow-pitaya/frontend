use gtk::prelude::*;

#[derive(relm_derive::Msg, Clone)]
pub enum Msg {
    Expand,
    Fold,
    SetColor(crate::color::Color),
    SetLabel(String),
}

#[relm_derive::widget(Clone)]
impl relm::Widget for Palette {
    fn model(_: ()) {}

    fn update(&mut self, event: Msg) {
        use crate::color::Colorable;

        match event {
            Msg::Expand => {
                self.widgets.parent.set_no_show_all(false);
                self.widgets.parent.show_all();
            }
            Msg::Fold => self.widgets.parent.hide(),
            Msg::SetColor(color) => self.set_color(color),
            Msg::SetLabel(label) => self.set_label(&label),
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
                    toggled(widget) => if widget.is_active() {
                        Msg::Expand
                    } else {
                        Msg::Fold
                    }
                },
            },
            #[name="parent"]
            #[container]
            gtk::Box {
                orientation: gtk::Orientation::Vertical,
                no_show_all: true,
            },
        },
    }
}

impl Palette {
    pub fn set_label(&self, label: &str) {
        self.widgets.toggle.set_label(label);
    }
}

impl crate::color::Colorable for Palette {
    fn set_color(&self, color: crate::color::Color) {
        let context = self.widgets.border.style_context();

        context.add_class(&format!("{}", color));
    }
}
