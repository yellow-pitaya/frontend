pub mod level;

use crate::color::Colorable as _;
use gtk::prelude::*;
use relm4::ComponentController as _;

#[derive(Debug)]
pub enum InputMsg {
    Redraw(Box<gtk::cairo::Context>, Box<crate::application::Data>),
    SetImage(gtk::cairo::ImageSurface),
    SourceStart(level::Orientation, String),
    SourceStop(level::Orientation, String),
}

#[derive(Debug)]
pub enum OutputMsg {
    Level(String, i32),
    Resize(i32, i32),
}

pub struct Model {
    level_left: relm4::Controller<level::Model>,
    level_top: relm4::Controller<level::Model>,
    level_right: relm4::Controller<level::Model>,
    handler: relm4::abstractions::DrawHandler,
    p1: relm4::Controller<level::placeholder::Model>,
    p2: relm4::Controller<level::placeholder::Model>,
}

#[relm4::component(pub)]
impl relm4::SimpleComponent for Model {
    type Init = ();
    type Input = InputMsg;
    type Output = OutputMsg;

    fn init(
        _: Self::Init,
        _root: Self::Root,
        sender: relm4::ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        use relm4::Component as _;
        use relm4::ComponentController as _;

        let level_left = level::Model::builder()
            .launch(level::Orientation::Left)
            .forward(sender.output_sender(), |output| {
                let level::OutputMsg::Level(ref name, offset) = output;
                OutputMsg::Level(name.clone(), offset)
            });

        let level_top = level::Model::builder()
            .launch(level::Orientation::Top)
            .forward(sender.output_sender(), |output| {
                let level::OutputMsg::Level(ref name, offset) = output;
                OutputMsg::Level(name.clone(), offset)
            });

        let level_right = level::Model::builder()
            .launch(level::Orientation::Right)
            .forward(sender.output_sender(), |output| {
                let level::OutputMsg::Level(ref name, offset) = output;
                OutputMsg::Level(name.clone(), offset)
            });

        let p1 = level::placeholder::Model::builder().launch(()).detach();

        let p2 = level::placeholder::Model::builder().launch(()).detach();

        let model = Self {
            level_left,
            level_top,
            level_right,
            handler: relm4::abstractions::DrawHandler::new(),
            p1,
            p2,
        };

        let drawing_area = model.handler.drawing_area();
        let widgets = view_output!();

        relm4::ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _: relm4::ComponentSender<Self>) {
        match msg {
            InputMsg::SourceStart(orientation, source) => match orientation {
                level::Orientation::Left => {
                    self.level_left.emit(level::InputMsg::SourceStart(source))
                }
                level::Orientation::Right => {
                    self.level_right.emit(level::InputMsg::SourceStart(source))
                }
                level::Orientation::Top => {
                    self.level_top.emit(level::InputMsg::SourceStart(source))
                }
            },
            InputMsg::SourceStop(orientation, source) => match orientation {
                level::Orientation::Left => {
                    self.level_left.emit(level::InputMsg::SourceStop(source))
                }
                level::Orientation::Right => {
                    self.level_right.emit(level::InputMsg::SourceStop(source))
                }
                level::Orientation::Top => self.level_top.emit(level::InputMsg::SourceStop(source)),
            },
            InputMsg::Redraw(ref context, ref model) => self.draw(context, model).unwrap(),
            InputMsg::SetImage(image) => self.set_image(&image).unwrap(),
        }
    }

    view! {
        gtk::Box {
            set_hexpand: true,
            set_orientation: gtk::Orientation::Horizontal,
            set_vexpand: true,

            gtk::Box {
                set_halign: gtk::Align::Fill,
                set_orientation: gtk::Orientation::Vertical,

                append: model.p1.widget(),
                append: model.level_left.widget(),
            },
            gtk::Box {
                set_halign: gtk::Align::Fill,
                set_valign: gtk::Align::Fill,
                set_hexpand: true,
                set_orientation: gtk::Orientation::Vertical,
                set_vexpand: true,

                append: model.level_top.widget(),

                #[local_ref]
                drawing_area -> gtk::DrawingArea {
                    set_hexpand: true,
                    set_vexpand: true,

                    connect_resize[sender] => move |_, width, height| {
                        sender.output(OutputMsg::Resize(width, height)).ok();
                    },
                },
            },
            gtk::Box {
                set_halign: gtk::Align::Fill,
                set_orientation: gtk::Orientation::Vertical,

                append: model.p2.widget(),
                append: model.level_right.widget(),
            },
        }
    }
}

impl Model {
    fn draw(
        &self,
        context: &gtk::cairo::Context,
        data: &crate::application::Data,
    ) -> Result<(), gtk::cairo::Error> {
        let width = data.scales.width();
        let height = data.scales.height();

        context.set_color(crate::color::BACKGROUND);
        context.rectangle(data.scales.h.0, data.scales.v.0, width, height);
        context.fill()?;

        context.set_color(crate::color::MAIN_SCALE);

        context.rectangle(data.scales.h.0, data.scales.v.0, width, height);
        context.stroke()?;

        for i in 0..11 {
            if i % 5 == 0 {
                context.set_color(crate::color::MAIN_SCALE);
            } else {
                context.set_color(crate::color::SECONDARY_SCALE);
            }

            let x = width / 10.0 * (i as f64);

            context.set_line_width(width / 1000.0);
            context.move_to(data.scales.h.0 + x, data.scales.v.0);
            context.line_to(data.scales.h.0 + x, data.scales.v.1);
            context.stroke()?;

            let y = height / 10.0 * (i as f64);

            context.set_line_width(height / 1000.0);
            context.move_to(data.scales.h.0, data.scales.v.0 + y);
            context.line_to(data.scales.h.1, data.scales.v.0 + y);
            context.stroke()?;
        }

        Ok(())
    }

    fn set_image(&mut self, image: &gtk::cairo::ImageSurface) -> Result<(), gtk::cairo::Error> {
        let context = self.handler.get_context();

        context.set_source_surface(image, 0., 0.)?;
        context.paint()
    }
}
