use crate::color::Colorable as _;
use gtk::prelude::*;

pub struct Model {
    handler: relm4::abstractions::DrawHandler,
}

pub struct Widgets {}

impl relm4::SimpleComponent for Model {
    type Init = ();
    type Input = ();
    type Output = ();
    type Root = gtk::DrawingArea;
    type Widgets = Widgets;

    fn init_root() -> Self::Root {
        gtk::DrawingArea::default()
    }

    fn init(
        _: Self::Init,
        root: Self::Root,
        _sender: relm4::ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let model = Self {
            handler: relm4::abstractions::DrawHandler::new_with_drawing_area(root),
        };

        let widgets = Widgets {};

        let drawing_area = model.handler.drawing_area();
        drawing_area.set_draw_func(|_, context, _, _| {
            context.set_color(crate::color::BACKGROUND);
            context.rectangle(0.0, 0.0, 20.0, 20.0);
            context.fill().unwrap();
        });
        drawing_area.set_size_request(20, 20);

        relm4::ComponentParts { model, widgets }
    }
}
