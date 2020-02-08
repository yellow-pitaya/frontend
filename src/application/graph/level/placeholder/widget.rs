use gtk::{
    self,
    WidgetExt,
};
use color::Colorable;
use relm_attributes::widget;

#[widget]
impl ::relm::Widget for Widget {
    fn model(_: ()) -> () {
    }

    fn update(&mut self, signal: super::Signal) {
        match signal {
            super::Signal::Draw => {
                let context = ::create_context(&self.drawing_area);
                context.set_color(::color::BACKGROUND);
                context.rectangle(0.0, 0.0, 20.0, 20.0);
                context.fill();
            },
        }
    }

    view! {
        #[name="drawing_area"]
        gtk::DrawingArea {
            draw(_, context) => (super::Signal::Draw, ::gtk::Inhibit(false)),
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
            model: self.model.clone(),
        }
    }
}
