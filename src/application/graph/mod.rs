pub mod level;

use crate::color::Colorable;
use gtk::prelude::*;
use level::placeholder::Widget as Placeholder;
use level::Msg::Level as LevelMsg;
use level::Widget as LevelWidget;

#[derive(relm_derive::Msg, Clone)]
pub enum Msg {
    Invalidate,
    Draw,
    Level(String, i32),
    Redraw(Box<cairo::Context>, Box<crate::application::Model>),
    Resize(i32, i32),
    SetImage(cairo::ImageSurface),
    SourceStart(level::Orientation, String),
    SourceStop(level::Orientation, String),
}

impl Widget {
    fn draw(&self, context: &cairo::Context, model: &crate::application::Model) {
        let width = model.scales.get_width();
        let height = model.scales.get_height();

        context.set_color(crate::color::BACKGROUND);
        context.rectangle(model.scales.h.0, model.scales.v.0, width, height);
        context.fill();

        context.set_color(crate::color::MAIN_SCALE);

        context.rectangle(model.scales.h.0, model.scales.v.0, width, height);
        context.stroke();

        for i in 0..11 {
            if i % 5 == 0 {
                context.set_color(crate::color::MAIN_SCALE);
            } else {
                context.set_color(crate::color::SECONDARY_SCALE);
            }

            let x = width / 10.0 * (i as f64);

            context.set_line_width(width / 1000.0);
            context.move_to(model.scales.h.0 + x, model.scales.v.0);
            context.line_to(model.scales.h.0 + x, model.scales.v.1);
            context.stroke();

            let y = height / 10.0 * (i as f64);

            context.set_line_width(height / 1000.0);
            context.move_to(model.scales.h.0, model.scales.v.0 + y);
            context.line_to(model.scales.h.1, model.scales.v.0 + y);
            context.stroke();
        }

        self.level_left.emit(level::Msg::Invalidate);
        self.level_right.emit(level::Msg::Invalidate);
    }

    fn get_width(&self) -> f64 {
        self.drawing_area.get_allocated_width() as f64
    }

    fn get_height(&self) -> f64 {
        self.drawing_area.get_allocated_height() as f64
    }

    fn set_image(&self, image: &cairo::ImageSurface) {
        let context = crate::create_context(&self.drawing_area);

        context.set_source_surface(image, 0.0, 0.0);
        context.paint();
    }

    fn invalidate(&self) {
        self.drawing_area
            .queue_draw_area(0, 0, self.get_width() as i32, self.get_height() as i32);

        self.level_left.emit(level::Msg::Invalidate);
        self.level_right.emit(level::Msg::Invalidate);
    }
}

#[relm_derive::widget(Clone)]
impl relm::Widget for Widget {
    fn model(_: ()) {}

    fn update(&mut self, event: Msg) {
        match event {
            Msg::Invalidate => self.invalidate(),
            Msg::SourceStart(orientation, source) => match orientation {
                level::Orientation::Left => self.level_left.emit(level::Msg::SourceStart(source)),
                level::Orientation::Right => self.level_right.emit(level::Msg::SourceStart(source)),
                level::Orientation::Top => self.level_top.emit(level::Msg::SourceStart(source)),
            },
            Msg::SourceStop(orientation, source) => match orientation {
                level::Orientation::Left => self.level_left.emit(level::Msg::SourceStop(source)),
                level::Orientation::Right => self.level_right.emit(level::Msg::SourceStop(source)),
                level::Orientation::Top => self.level_top.emit(level::Msg::SourceStop(source)),
            },
            Msg::Redraw(ref context, ref model) => self.draw(context, model),
            Msg::SetImage(ref image) => self.set_image(image),
            _ => (),
        }
    }

    view! {
        gtk::Box {
            orientation: gtk::Orientation::Horizontal,
            gtk::Box {
                orientation: gtk::Orientation::Vertical,
                child: {
                    expand: false,
                    fill: true,
                },
                Placeholder {
                    child: {
                        expand: false,
                        fill: true,
                    },
                },
                #[name="level_left"]
                LevelWidget(level::Orientation::Left) {
                    child: {
                        expand: true,
                        fill: true,
                    },
                    LevelMsg(ref name, offset) => Msg::Level(name.clone(), offset),
                },
            },
            gtk::Box {
                orientation: gtk::Orientation::Vertical,
                child: {
                    expand: true,
                    fill: true,
                },
                #[name="level_top"]
                LevelWidget(level::Orientation::Top) {
                    child: {
                        expand: false,
                        fill: true,
                    },
                    LevelMsg(ref name, offset) => Msg::Level(name.clone(), offset),
                },
                #[name="drawing_area"]
                gtk::DrawingArea {
                    child: {
                        expand: true,
                        fill: true,
                    },
                    draw(_, _) => (Msg::Draw, gtk::Inhibit(false)),
                },
            },
            gtk::Box {
                orientation: gtk::Orientation::Vertical,
                child: {
                    expand: false,
                    fill: true,
                },
                Placeholder {
                    child: {
                        expand: false,
                        fill: true,
                    },
                },
                #[name="level_right"]
                LevelWidget(level::Orientation::Right) {
                    child: {
                        expand: true,
                        fill: true,
                    },
                    LevelMsg(ref name, offset) => Msg::Level(name.clone(), offset),
                },
            },
            size_allocate(_, allocation) => Msg::Resize(allocation.width, allocation.height),
        },
    }
}
