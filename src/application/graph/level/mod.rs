pub mod placeholder;

use crate::color::Colorable;
use gtk::prelude::*;

#[derive(Debug)]
pub enum InputMsg {
    Click(f64, f64),
    Draw,
    Move(Option<(f64, f64)>, f64, f64),
    Release,
    SourceStart(String),
    SourceStop(String),
}

#[derive(Debug)]
pub enum OutputMsg {
    Level(String, i32),
}

#[derive(Clone, Debug)]
pub struct Level {
    enable: bool,
    offset: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Orientation {
    Left,
    Right,
    Top,
}

pub struct Model {
    current: Option<String>,
    orientation: Orientation,
    levels: std::collections::HashMap<String, Level>,
    handler: relm4::abstractions::DrawHandler,
}

#[relm4::component(pub)]
impl relm4::SimpleComponent for Model {
    type Init = Orientation;
    type Input = InputMsg;
    type Output = OutputMsg;

    fn init(
        init: Self::Init,
        _root: Self::Root,
        sender: relm4::ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let model = Self {
            current: None,
            orientation: init,
            levels: std::collections::HashMap::new(),
            handler: relm4::abstractions::DrawHandler::new(),
        };

        let drawing_area = model.handler.drawing_area();
        let widgets = view_output!();

        relm4::ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: relm4::ComponentSender<Self>) {
        match msg {
            InputMsg::Click(x, y) => self.on_click(x as i32, y as i32),
            InputMsg::Move(start_point, x, y) => {
                self.on_mouse_move(start_point, x as i32, y as i32).unwrap()
            }
            InputMsg::Draw => self.draw().unwrap(),
            InputMsg::SourceStart(source) => self.start(source).unwrap(),
            InputMsg::SourceStop(source) => self.stop(source).unwrap(),
            InputMsg::Release => self.on_release(&sender),
        }
    }

    view! {
        gtk::Box {
            #[local_ref]
            drawing_area -> gtk::DrawingArea {
                set_height_request: if model.orientation == Orientation::Top { 20 } else { -1 },
                set_hexpand: model.orientation == Orientation::Top,
                set_vexpand: model.orientation != Orientation::Top,
                set_width_request: if model.orientation == Orientation::Top { -1 } else { 20 },

                add_controller = gtk::GestureDrag {
                    connect_drag_begin[sender] => move |_, x, y| sender.input(InputMsg::Click(x, y)),
                    connect_drag_update[sender] => move |this, x, y| sender.input(InputMsg::Move(this.start_point(), x, y)),
                    connect_drag_end[sender] => move |_, _, _| sender.input(InputMsg::Release),
                },
            },
        }
    }
}

impl Model {
    fn start(&mut self, name: String) -> Result<(), gtk::cairo::Error> {
        let level = self.levels.entry(name).or_insert(Level {
            enable: false,
            offset: self.handler.drawing_area().height() / 2,
        });

        level.enable = true;

        self.draw()
    }

    fn stop(&mut self, name: String) -> Result<(), gtk::cairo::Error> {
        if let Some(level) = self.levels.get_mut(&name) {
            level.enable = false;
            self.draw()?;
        }

        Ok(())
    }

    fn set_level(&mut self, name: String, offset: i32) -> Result<(), gtk::cairo::Error> {
        if let Some(level) = self.levels.get_mut(&name) {
            level.offset = offset;
            self.draw()?;
        }

        Ok(())
    }

    fn draw(&mut self) -> Result<(), gtk::cairo::Error> {
        let width = self.handler.drawing_area().width();
        let height = self.handler.drawing_area().height();
        let context = self.handler.get_context();

        context.set_color(crate::color::BACKGROUND);
        context.rectangle(0.0, 0.0, width as f64, height as f64);
        context.fill()?;

        for (name, level) in &self.levels {
            if !level.enable {
                continue;
            }

            let (start, end) = match self.orientation {
                Orientation::Left => (0.0, width as f64),
                Orientation::Right => (width as f64, 0.0),
                Orientation::Top => (0.0, height as f64),
            };

            let middle = f64::max(start, end) / 2.0;
            let top = (level.offset + 7) as f64;
            let bottom = (level.offset - 7) as f64;

            context.set_color(name.clone().into());

            if self.orientation == Orientation::Top {
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
            context.fill()?;
        }

        Ok(())
    }

    fn on_click(&mut self, x: i32, y: i32) {
        let offset = match self.orientation {
            Orientation::Left | Orientation::Right => y,
            Orientation::Top => x,
        };

        self.current = self.find_level(offset);
    }

    fn find_level(&self, offset: i32) -> Option<String> {
        for (name, level) in &self.levels {
            if !level.enable {
                continue;
            }

            if offset + 7 >= level.offset && offset - 7 <= level.offset {
                return Some(name.clone());
            }
        }

        None
    }

    fn on_mouse_move(
        &mut self,
        start_point: Option<(f64, f64)>,
        x: i32,
        y: i32,
    ) -> Result<(), gtk::cairo::Error> {
        if let Some((start_x, start_y)) = start_point {
            let offset = match self.orientation {
                Orientation::Left | Orientation::Right => start_y as i32 + y,
                Orientation::Top => start_x as i32 + x,
            };

            if let Some(name) = self.current.clone() {
                self.set_level(name, offset)?;
            }
        }

        Ok(())
    }

    fn on_release(&mut self, sender: &relm4::ComponentSender<Self>) {
        let Some(name) = self.current.clone() else {
            sender.input(InputMsg::Draw);
            return;
        };

        let Some(level) = self.levels.get(&name) else {
            sender.input(InputMsg::Draw);
            return;
        };

        self.current = None;

        sender
            .output(OutputMsg::Level(name.clone(), level.offset))
            .ok();
    }
}
