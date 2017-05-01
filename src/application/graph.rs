use color::Colorable;
use gtk::{
    WidgetExt,
};

#[derive(Msg)]
pub enum Signal {
    Draw,
}

#[derive(Clone)]
pub struct Widget {
    drawing_area: ::gtk::DrawingArea,
}

impl Widget {
    pub fn get_width(&self) -> f64 {
        self.drawing_area.get_allocated_width() as f64
    }

    pub fn get_height(&self) -> f64 {
        self.drawing_area.get_allocated_height() as f64
    }

    pub fn set_image(&self, image: &::cairo::ImageSurface) {
        let context = self.create_context();

        context.set_source_surface(image, 0.0, 0.0);
        context.paint();
    }

    fn create_context(&self) -> ::cairo::Context {
        let window = self.drawing_area.get_window().unwrap();

        unsafe {
            use ::glib::translate::ToGlibPtr;

            let context = ::gdk_sys::gdk_cairo_create(window.to_glib_none().0);

            ::std::mem::transmute(context)
        }
    }
}

impl ::application::Panel for Widget {
    fn draw(&self, context: &::cairo::Context, model: &::application::Model) {
        let width = model.scales.get_width();
        let height = model.scales.get_height();

        context.set_color(::color::BACKGROUND);
        context.rectangle(model.scales.h.0, model.scales.v.0, width, height);
        context.fill();

        context.set_color(::color::MAIN_SCALE);

        context.rectangle(model.scales.h.0, model.scales.v.0, width, height);
        context.stroke();

        for i in 1..10 {
            if i == 5 {
                context.set_color(::color::MAIN_SCALE);
            } else {
                context.set_color(::color::SECONDARY_SCALE);
            }

            let x = width / 10.0 * (i as f64);

            context.move_to(model.scales.h.0 + x, model.scales.v.0);
            context.line_to(model.scales.h.0 + x, model.scales.v.1);
            context.stroke();

            let y = height / 10.0 * (i as f64);

            context.move_to(model.scales.h.0, model.scales.v.0 + y);
            context.line_to(model.scales.h.1, model.scales.v.0 + y);
            context.stroke();
        }
    }
}

impl ::relm::Widget for Widget {
    type Model = ();
    type Msg = Signal;
    type Root = ::gtk::DrawingArea;
    type ModelParam = ();

    fn model(_: Self::ModelParam) -> Self::Model {
    }

    fn root(&self) -> &Self::Root {
        &self.drawing_area
    }

    fn update(&mut self, _: Signal, _: &mut Self::Model) {
    }

    fn view(relm: &::relm::RemoteRelm<Self>, _: &Self::Model) -> Self {
        let drawing_area = ::gtk::DrawingArea::new();
        connect!(relm, drawing_area, connect_draw(_, _) (Signal::Draw, ::gtk::Inhibit(false)));

        Widget {
            drawing_area,
        }
    }
}
