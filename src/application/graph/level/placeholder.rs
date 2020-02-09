use crate::color::Colorable;
use gtk::{
    self,
    WidgetExt,
};

#[derive(relm_derive::Msg, Clone)]
pub enum Signal {
    Draw,
}

#[relm_derive::widget]
impl relm::Widget for Widget {
    fn model(_: ()) {
    }

    fn update(&mut self, signal: Signal) {
        match signal {
            Signal::Draw => {
                let context = crate::create_context(&self.drawing_area);
                context.set_color(crate::color::BACKGROUND);
                context.rectangle(0.0, 0.0, 20.0, 20.0);
                context.fill();
            },
        }
    }

    view! {
        #[name="drawing_area"]
        gtk::DrawingArea {
            draw(_, context) => (Signal::Draw, gtk::Inhibit(false)),
        },
    }

    fn init_view(&mut self) {
        self.drawing_area.set_size_request(20, 20);
    }
}

impl Clone for Widget {
    fn clone(&self) -> Self {
        Self {
            drawing_area: self.drawing_area.clone(),
            model: self.model,
        }
    }
}
