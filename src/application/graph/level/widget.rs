use application::Panel;
use gtk::{
    GestureDragExt,
    WidgetExt,
};
use color::Colorable;
use super::Model;
use super::Signal;

#[derive(Clone, Copy, PartialEq)]
pub enum Orientation {
    Left,
    Right,
    Top,
}

#[derive(Clone)]
pub struct Widget {
    gesture_drag: ::gtk::GestureDrag,
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
        let context = self.create_context(&self.drawing_area);

        context.set_source_surface(image, 0.0, 0.0);
        context.paint();
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

            let (start, end) = match model.orientation {
                Orientation::Left => (0.0, width as f64),
                Orientation::Right => (width as f64, 0.0),
                Orientation::Top => (0.0, height as f64),
            };

            let middle = f64::max(start, end) / 2.0;
            let top = (level.offset + 7) as f64;
            let bottom = (level.offset - 7) as f64;

            context.set_color(name.clone().into());

            if model.orientation == Orientation::Top {
                context.move_to(top, start);
                context.line_to(top, middle);
                context.line_to(level.offset as f64, end);
                context.line_to(bottom, middle);
                context.line_to(bottom, start);
            }
            else {
                context.move_to(start, top);
                context.line_to(middle, top);
                context.line_to(end, level.offset as f64);
                context.line_to(middle, bottom);
                context.line_to(start, bottom);
            }

            context.close_path();
            context.fill();
        }

        self.set_image(&image);
    }

    fn on_click(&self, model: &mut Model, x: i32, y: i32) {
        let offset = match model.orientation {
            Orientation::Left | Orientation::Right => y,
            Orientation::Top => x,
        };

        model.current = self.find_level(model, offset);
    }

    fn find_level(&self, model: &Model, offset: i32) -> Option<String> {
        for (name, level) in &model.levels {
            if level.enable == false {
                continue;
            }

            if offset + 7 >= level.offset && offset - 7 <= level.offset {
                return Some(name.clone());
            }
        }

        None
    }

    fn on_mouse_move(&self, model: &mut Model, x: i32, y: i32) {
        if let Some((start_x, start_y)) = self.gesture_drag.get_start_point() {
            let offset = match model.orientation {
                Orientation::Left | Orientation::Right => start_y as i32 + y,
                Orientation::Top => start_x as i32 + x,
            };

            if let Some(name) = model.current.clone() {
                self.set_level(model, name, offset);
            }
        }
    }

    fn on_release(model: &mut Model) -> Signal {
        let name = match model.current.clone() {
            Some(name) => name,
            None => return Signal::Draw,
        };

        let level = match model.levels.get(&name) {
            Some(level) => level,
            None => return Signal::Draw,
        };

        model.current = None;

        Signal::Level(name.clone(), level.offset)
    }
}

impl ::application::Panel for Widget {
    fn draw(&self, _: &::cairo::Context, _: &::application::Model) {
        self.invalidate();
    }
}

impl ::relm::Widget for Widget {
    type Model = Model;
    type Msg = Signal;
    type Root = ::gtk::DrawingArea;
    type ModelParam = Orientation;

    fn model(orientation: Orientation) -> Self::Model {
        Model {
            current: None,
            orientation: orientation,
            levels: ::std::collections::HashMap::new(),
        }
    }

    fn root(&self) -> &Self::Root {
        &self.drawing_area
    }

    fn update(&mut self, signal: Signal, model: &mut Self::Model) {
        match signal {
            Signal::Click(x, y) => self.on_click(model, x as i32, y as i32),
            Signal::Move(x, y) => self.on_mouse_move(model, x as i32, y as i32),
            Signal::Draw => self.draw_levels(model),
            Signal::SourceStart(source) => self.start(model, source),
            Signal::SourceStop(source) => self.stop(model, source),
            Signal::Level(_, _) => (),
        }
    }

    fn view(relm: &::relm::RemoteRelm<Self>, _: &Self::Model) -> Self {
        let drawing_area = ::gtk::DrawingArea::new();
        connect!(relm, drawing_area, connect_draw(_, _) (Signal::Draw, ::gtk::Inhibit(false)));

        let gesture_drag = ::gtk::GestureDrag::new(&drawing_area);
        connect!(gesture_drag, connect_drag_begin(_, x, y), relm, Signal::Click(x, y));
        connect!(gesture_drag, connect_drag_update(_, x, y), relm, Signal::Move(x, y));
        connect!(relm, gesture_drag, connect_drag_end(_, _, _) with model (Self::on_release(model), ()));

        Widget {
            gesture_drag,
            drawing_area,
        }
    }

    fn init_view(&self, model: &mut Self::Model) {
        match model.orientation {
            Orientation::Left | Orientation::Right => self.drawing_area.set_size_request(20, -1),
            Orientation::Top => self.drawing_area.set_size_request(-1, 20),
        };
    }
}
