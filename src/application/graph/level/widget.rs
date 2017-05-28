use gtk::{
    ContainerExt,
    WidgetExt,
};
use color::Colorable;
use super::Model;
use super::Signal;

#[derive(Clone, Copy, PartialEq)]
pub enum Orientation {
    Right,
    Left,
}

#[derive(Clone)]
pub struct Widget {
    event_box: ::gtk::EventBox,
    drawing_area: ::gtk::DrawingArea,
}

impl Widget {
    fn start(&self, model: &mut Model, name: String) {
        if model.levels.get(&name).is_none() {
            model.levels.insert(name.clone(), super::model::Level {
                enable: true,
                offset: self.get_height() / 2,
            });
        }

        model.levels.get_mut(&name).unwrap().enable = true;

        self.invalidate();
    }

    fn stop(&self, model: &mut Model, name: String) {
        if let Some(mut level) = model.levels.get_mut(&name) {
            level.enable = false;
            self.invalidate();
        }
    }

    fn set_level(&self, model: &mut Model, name: String, offset: i32) {
        if let Some(mut level) = model.levels.get_mut(&name) {
            level.offset = offset;
            self.invalidate();
        }
    }

    pub fn invalidate(&self) {
        self.drawing_area.queue_draw_area(
            0,
            0,
            self.get_width(),
            self.get_height(),
        );
    }

    fn get_width(&self) -> i32 {
        self.drawing_area.get_allocated_width()
    }

    fn get_height(&self) -> i32 {
        self.drawing_area.get_allocated_height()
    }

    fn set_image(&self, image: &::cairo::ImageSurface) {
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

    fn draw_levels(&self, model: &Model) {
        let width = self.get_width();
        let height = self.get_height();

        let image = ::cairo::ImageSurface::create(::cairo::Format::ARgb32, width, height);
        let context = ::cairo::Context::new(&image);

        context.set_color(::color::BACKGROUND);
        context.rectangle(0.0, 0.0, width as f64, height as f64);
        context.fill();

        for (name, level) in &model.levels {
            if level.enable == false {
                continue;
            }

            context.set_color(name.clone().into());

            context.move_to(0.0, (level.offset + 7) as f64);
            context.line_to((width / 2) as f64, (level.offset + 7) as f64);
            context.line_to(width as f64, level.offset as f64);
            context.line_to((width / 2) as f64, (level.offset - 7) as f64);
            context.line_to(0.0, (level.offset - 7) as f64);
            context.close_path();

            context.fill();
        }

        self.set_image(&image);
    }

    fn on_click(&self, model: &mut Model, _: i32, y: i32) {
        model.current = self.find_level(model, y);
    }

    fn find_level(&self, model: &Model, y: i32) -> Option<String> {
        for (name, level) in &model.levels {
            if level.enable == false {
                continue;
            }

            if y + 7 >= level.offset && y - 7 <= level.offset {
                return Some(name.clone());
            }
        }

        None
    }

    fn on_release(model: &mut Model) -> (Signal, ::gtk::Inhibit) {
        let mut result = (Signal::Draw, ::gtk::Inhibit(false));

        let name = match model.current.clone() {
            Some(name) => name,
            None => return result,
        };

        let level = match model.levels.get(&name) {
            Some(level) => level,
            None => return result,
        };

        model.current = None;

        result.0 = Signal::Level(name.clone(), level.offset);

        result
    }

    fn on_mouse_move(&self, model: &mut Model, _: i32, y: i32) {
        if let Some(name) = model.current.clone() {
            self.set_level(model, name, y);
        }
    }
}

impl ::application::Panel for Widget {
    fn draw(&self, _: &::cairo::Context, _: &::application::Model) {
        self.invalidate();
    }

    fn update_scales(&self, _: ::Scales) {
    }
}

impl ::relm::Widget for Widget {
    type Model = Model;
    type Msg = Signal;
    type Root = ::gtk::EventBox;
    type ModelParam = Orientation;

    fn model(orientation: Orientation) -> Self::Model {
        Model {
            current: None,
            orientation: orientation,
            levels: ::std::collections::HashMap::new(),
        }
    }

    fn root(&self) -> &Self::Root {
        &self.event_box
    }

    fn update(&mut self, signal: Signal, model: &mut Self::Model) {
        match signal {
            Signal::Click((x, y)) => self.on_click(model, x as i32, y as i32),
            Signal::Move((x, y)) => self.on_mouse_move(model, x as i32, y as i32),
            Signal::Draw => self.draw_levels(model),
            Signal::SourceStart(source) => self.start(model, source),
            Signal::SourceStop(source) => self.stop(model, source),
            Signal::Level(_, _) => (),
        }
    }

    fn view(relm: &::relm::RemoteRelm<Self>, _: &Self::Model) -> Self {
        let event_box = ::gtk::EventBox::new();

        connect!(relm, event_box, connect_button_press_event(_, event) (Signal::Click(event.get_position()), ::gtk::Inhibit(false)));
        connect!(relm, event_box, connect_button_release_event(_, _) with model Self::on_release(model));
        connect!(relm, event_box, connect_motion_notify_event(_, event) (Signal::Move(event.get_position()), ::gtk::Inhibit(false)));

        let drawing_area = ::gtk::DrawingArea::new();
        event_box.add(&drawing_area);

        connect!(relm, drawing_area, connect_draw(_, _) (Signal::Draw, ::gtk::Inhibit(false)));

        Widget {
            event_box: event_box,
            drawing_area: drawing_area,
        }
    }

    fn init_view(&self, _: &mut Self::Model) {
        self.drawing_area.set_size_request(20, -1);
    }
}
