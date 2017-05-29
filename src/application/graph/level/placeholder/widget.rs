use gtk::{
    self,
    WidgetExt,
};
use color::Colorable;
use relm_attributes::widget;
use application::Panel;

impl ::application::Panel for Widget {
    fn draw(&self, _: &::cairo::Context, _: &::application::Model) {
    }
}

#[widget]
impl ::relm::Widget for Widget {
    fn model(_: ()) -> () {
    }

    fn update(&mut self, signal: super::Signal, _: &mut Self::Model) {
        match signal {
            super::Signal::Draw => {
                let context = self.create_context(&self.drawing_area);
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

    fn init_view(&self, _: &mut Self::Model) {
        self.drawing_area.set_size_request(20, 20);
    }
}
