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

    pub fn draw(&self) {
        let width = self.get_width();
        let height = self.get_height();
        let context = self.create_context();

        context.set_source_rgb(1.0, 1.0, 1.0);
        context.rectangle(0.0, 0.0, width, height);
        context.fill();

        self.draw_scales(&context, width, height);
    }

    fn draw_scales(&self, context: &::cairo::Context, width: f64, height: f64) {
        context.set_line_width(1.0);
        context.set_source_rgb(0.0, 0.0, 0.0);

        context.rectangle(0.0, 0.0, width, height);
        context.stroke();

        for i in 1..10 {
            let x = width / 10.0 * (i as f64);

            if i != 5 {
                context.set_source_rgba(0.0, 0.0, 0.0, 0.2);
            } else {
                context.set_source_rgb(0.0, 0.0, 0.0);
            }

            context.move_to(x, 0.0);
            context.line_to(x, height);

            let y = height / 10.0 * (i as f64);

            context.move_to(0.0, y);
            context.line_to(width, y);

            context.stroke();
        }
    }

    pub fn create_context(&self) -> ::cairo::Context {
        let window = self.drawing_area.get_window().unwrap();

        unsafe {
            use ::glib::translate::ToGlibPtr;

            let context = ::gdk_sys::gdk_cairo_create(window.to_glib_none().0);

            ::std::mem::transmute(context)
        }
    }
}

impl ::relm::Widget for Widget {
    type Model = ();
    type Msg = Signal;
    type Root = ::gtk::DrawingArea;

    fn model() -> Self::Model {
    }

    fn root(&self) -> &Self::Root {
        &self.drawing_area
    }

    fn update(&mut self, event: Signal, _: &mut Self::Model) {
        match event {
            Signal::Draw => self.draw(),
        }
    }

    fn view(relm: ::relm::RemoteRelm<Signal>, _: &Self::Model) -> Self {
        let drawing_area = ::gtk::DrawingArea::new();
        connect!(relm, drawing_area, connect_draw(_, _) (Signal::Draw, ::gtk::Inhibit(false)));

        let stream = relm.stream().clone();
        GLOBAL.with(move |global| {
            *global.borrow_mut() = Some(stream)
        });

        ::gtk::timeout_add(1_000, || {
            GLOBAL.with(|global| {
                if let Some(ref stream) = *global.borrow() {
                    stream.emit(Signal::Draw);
                }
            });

            ::glib::Continue(true)
        });

        Widget {
            drawing_area: drawing_area,
        }
    }
}

thread_local!(
    static GLOBAL: ::std::cell::RefCell<Option<::relm::EventStream<Signal>>> = ::std::cell::RefCell::new(None)
);
