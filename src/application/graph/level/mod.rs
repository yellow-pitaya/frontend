pub mod placeholder;

use crate::color::Colorable;
use gtk::prelude::*;

#[derive(relm_derive::Msg, Clone)]
pub enum Msg {
    Click(f64, f64),
    Draw,
    Invalidate,
    Move(f64, f64),
    Release,
    SourceStart(String),
    SourceStop(String),
    Level(String, i32),
}

#[derive(Clone, Debug)]
pub struct Level {
    enable: bool,
    offset: i32,
}

#[derive(Clone)]
pub struct Model {
    current: Option<String>,
    orientation: Orientation,
    levels: std::collections::HashMap<String, Level>,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Orientation {
    Left,
    Right,
    Top,
}

#[derive(Clone)]
pub struct Widget {
    stream: relm::EventStream<<Self as relm::Update>::Msg>,
    model: Model,
    gesture_drag: gtk::GestureDrag,
    drawing_area: gtk::DrawingArea,
}

// https://github.com/antoyo/relm/issues/42
impl Widget {
    fn start(&mut self, name: String) {
        if self.model.levels.get(&name).is_none() {
            self.model.levels.insert(
                name.clone(),
                Level {
                    enable: true,
                    offset: self.get_height() / 2,
                },
            );
        }

        self.model.levels.get_mut(&name).unwrap().enable = true;

        self.invalidate();
    }

    fn stop(&mut self, name: String) {
        if let Some(mut level) = self.model.levels.get_mut(&name) {
            level.enable = false;
            self.invalidate();
        }
    }

    fn set_level(&mut self, name: String, offset: i32) {
        if let Some(mut level) = self.model.levels.get_mut(&name) {
            level.offset = offset;
            self.invalidate();
        }
    }

    fn invalidate(&self) {
        self.drawing_area
            .queue_draw_area(0, 0, self.get_width(), self.get_height());
    }

    fn get_width(&self) -> i32 {
        self.drawing_area.get_allocated_width()
    }

    fn get_height(&self) -> i32 {
        self.drawing_area.get_allocated_height()
    }

    fn set_image(&self, image: &cairo::ImageSurface) {
        let context = crate::create_context(&self.drawing_area);

        context.set_source_surface(image, 0.0, 0.0);
        context.paint();
    }

    fn draw_levels(&self) {
        let width = self.get_width();
        let height = self.get_height();

        let image = cairo::ImageSurface::create(cairo::Format::ARgb32, width, height).unwrap();
        let context = cairo::Context::new(&image);

        context.set_color(crate::color::BACKGROUND);
        context.rectangle(0.0, 0.0, width as f64, height as f64);
        context.fill();

        for (name, level) in &self.model.levels {
            if !level.enable {
                continue;
            }

            let (start, end) = match self.model.orientation {
                Orientation::Left => (0.0, width as f64),
                Orientation::Right => (width as f64, 0.0),
                Orientation::Top => (0.0, height as f64),
            };

            let middle = f64::max(start, end) / 2.0;
            let top = (level.offset + 7) as f64;
            let bottom = (level.offset - 7) as f64;

            context.set_color(name.clone().into());

            if self.model.orientation == Orientation::Top {
                context.move_to(top, start);
                context.line_to(top, middle);
                context.line_to(level.offset as f64, end);
                context.line_to(bottom, middle);
                context.line_to(bottom, start);
            } else {
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

    fn on_click(&mut self, x: i32, y: i32) {
        let offset = match self.model.orientation {
            Orientation::Left | Orientation::Right => y,
            Orientation::Top => x,
        };

        self.model.current = self.find_level(offset);
    }

    fn find_level(&self, offset: i32) -> Option<String> {
        for (name, level) in &self.model.levels {
            if !level.enable {
                continue;
            }

            if offset + 7 >= level.offset && offset - 7 <= level.offset {
                return Some(name.clone());
            }
        }

        None
    }

    fn on_mouse_move(&mut self, x: i32, y: i32) {
        if let Some((start_x, start_y)) = self.gesture_drag.get_start_point() {
            let offset = match self.model.orientation {
                Orientation::Left | Orientation::Right => start_y as i32 + y,
                Orientation::Top => start_x as i32 + x,
            };

            if let Some(name) = self.model.current.clone() {
                self.set_level(name, offset);
            }
        }
    }

    fn on_release(&mut self) {
        let name = match self.model.current.clone() {
            Some(name) => name,
            None => {
                self.stream.emit(Msg::Draw);
                return;
            }
        };

        let level = match self.model.levels.get(&name) {
            Some(level) => level,
            None => {
                self.stream.emit(Msg::Draw);
                return;
            }
        };

        self.model.current = None;

        self.stream.emit(Msg::Level(name.clone(), level.offset));
    }
}

impl relm::Update for Widget {
    type Model = Model;
    type Msg = Msg;
    type ModelParam = Orientation;

    fn model(_: &relm::Relm<Self>, orientation: Orientation) -> Self::Model {
        Model {
            current: None,
            orientation,
            levels: std::collections::HashMap::new(),
        }
    }

    fn update(&mut self, event: Self::Msg) {
        match event {
            Msg::Click(x, y) => self.on_click(x as i32, y as i32),
            Msg::Move(x, y) => self.on_mouse_move(x as i32, y as i32),
            Msg::Draw => self.draw_levels(),
            Msg::Invalidate => self.invalidate(),
            Msg::SourceStart(source) => self.start(source),
            Msg::SourceStop(source) => self.stop(source),
            Msg::Release => self.on_release(),
            Msg::Level(_, _) => (),
        }
    }
}

impl relm::Widget for Widget {
    type Root = gtk::DrawingArea;

    fn root(&self) -> Self::Root {
        self.drawing_area.clone()
    }

    fn view(relm: &relm::Relm<Self>, model: Self::Model) -> Self {
        let drawing_area = gtk::DrawingArea::new();
        relm::connect!(
            relm,
            drawing_area,
            connect_draw(_, _),
            return (Msg::Draw, gtk::Inhibit(false))
        );

        let gesture_drag = gtk::GestureDrag::new(&drawing_area);
        relm::connect!(
            gesture_drag,
            connect_drag_begin(_, x, y),
            relm,
            Msg::Click(x, y)
        );
        relm::connect!(
            gesture_drag,
            connect_drag_update(_, x, y),
            relm,
            Msg::Move(x, y)
        );
        relm::connect!(gesture_drag, connect_drag_end(_, _, _), relm, Msg::Release);

        Widget {
            stream: relm.stream().clone(),
            model,
            gesture_drag,
            drawing_area,
        }
    }

    fn init_view(&mut self) {
        match self.model.orientation {
            Orientation::Left | Orientation::Right => self.drawing_area.set_size_request(20, -1),
            Orientation::Top => self.drawing_area.set_size_request(-1, 20),
        };
    }
}
