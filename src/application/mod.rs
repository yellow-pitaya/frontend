mod acquire;
mod generator;
mod graph;
mod trigger;

use gtk::prelude::*;
use relm4::ComponentController as _;

macro_rules! redraw {
    ($self:ident, $widget:ident, $image:ident) => {{
        let context = gtk::cairo::Context::new(&$image)?;

        if $image.width() > 0 && $image.height() > 0 {
            $self.transform(
                $self.data.scales,
                &context,
                $image.width() as f64,
                $image.height() as f64,
            );
            context.set_line_width(0.01);

            $self.$widget.emit($widget::InputMsg::Redraw(
                Box::new(context.clone()),
                Box::new($self.data.clone()),
            ));
        }
    }};
}

#[derive(Debug)]
pub enum Msg {
    Acquire(acquire::OutputMsg),
    Generator(generator::OutputMsg),
    Graph(graph::OutputMsg),
    Trigger(trigger::OutputMsg),
    Draw,
    Quit,
}

pub struct Model {
    data: Data,
    graph: relm4::Controller<graph::Model>,
    acquire: relm4::Controller<acquire::Model>,
    generator: relm4::Controller<generator::Model>,
    trigger: relm4::Controller<trigger::Model>,
}

#[derive(Clone)]
struct Data {
    rate: redpitaya_scpi::acquire::SamplingRate,
    redpitaya: redpitaya_scpi::Redpitaya,
    scales: crate::Scales,
    levels: std::collections::HashMap<String, i32>,
}

impl std::fmt::Debug for Data {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Data {
    fn offset<D>(&self, channel: D) -> f64
    where
        D: std::fmt::Display,
    {
        let channel = format!("{channel}");

        match self.levels.get(&channel) {
            Some(level) => {
                if channel == "DELAY" {
                    self.scales.x_to_offset(*level)
                } else {
                    self.scales.y_to_offset(*level)
                }
            }
            None => 0.0,
        }
    }
}

#[relm4::component(pub)]
impl relm4::Component for Model {
    type CommandOutput = ();
    type Init = redpitaya_scpi::Redpitaya;
    type Input = Msg;
    type Output = ();

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: relm4::ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        crate::Color::init();

        let mut scales = crate::Scales {
            h: (0.0, 0.0),
            v: (-5.0, 5.0),
            n_samples: init.data.buffer_size().unwrap(),
            window: crate::scales::Rect {
                width: 0,
                height: 0,
            },
        };

        let rate = init.acquire.get_decimation().unwrap().into();
        scales.with_sampling_rate(rate);

        let acquire = acquire::Model::builder()
            .launch(init.acquire.clone())
            .forward(sender.input_sender(), Msg::Acquire);

        let generator = generator::Model::builder()
            .launch(init.generator.clone())
            .forward(sender.input_sender(), Msg::Generator);

        let graph = graph::Model::builder()
            .launch(())
            .forward(sender.input_sender(), Msg::Graph);

        let trigger = trigger::Model::builder()
            .launch(init.trigger.clone())
            .forward(sender.input_sender(), Msg::Trigger);

        let mut model = Self {
            data: Data {
                rate,
                redpitaya: init,
                scales,
                levels: std::collections::HashMap::new(),
            },
            acquire,
            generator,
            graph,
            trigger,
        };

        let widgets = view_output!();

        model
            .data
            .redpitaya
            .data
            .set_units(redpitaya_scpi::data::Unit::VOLTS);

        model.data.redpitaya.acquire.start();

        relm4::ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        msg: Self::Input,
        sender: relm4::ComponentSender<Self>,
        _: &Self::Root,
    ) {
        match msg {
            Msg::Draw => self.draw(widgets).unwrap(),
            Msg::Acquire(msg) => match msg {
                acquire::OutputMsg::Rate(rate) => {
                    self.data.rate = rate;
                    self.data.scales.with_sampling_rate(rate);
                    self.update_status(widgets);
                }
                acquire::OutputMsg::Start(source) => self.graph.emit(graph::InputMsg::SourceStart(
                    graph::level::Orientation::Left,
                    source.to_string(),
                )),
                acquire::OutputMsg::Stop(source) => self.graph.emit(graph::InputMsg::SourceStop(
                    graph::level::Orientation::Left,
                    source.to_string(),
                )),
            },
            Msg::Graph(msg) => match msg {
                graph::OutputMsg::Level(channel, level) => {
                    self.data.levels.insert(channel, level);
                }
                graph::OutputMsg::Resize(width, height) => {
                    self.data.scales.window.width = width;
                    self.data.scales.window.height = height;
                    sender.input(Msg::Draw);
                }
            },
            Msg::Generator(msg) => match msg {
                generator::OutputMsg::Start(source) => {
                    self.graph.emit(graph::InputMsg::SourceStart(
                        graph::level::Orientation::Left,
                        source.to_string(),
                    ))
                }
                generator::OutputMsg::Stop(source) => self.graph.emit(graph::InputMsg::SourceStop(
                    graph::level::Orientation::Left,
                    source.to_string(),
                )),
            },
            Msg::Trigger(msg) => match msg {
                trigger::OutputMsg::Auto => {
                    self.graph.emit(graph::InputMsg::SourceStop(
                        graph::level::Orientation::Right,
                        "TRIG".to_string(),
                    ));
                    self.graph.emit(graph::InputMsg::SourceStop(
                        graph::level::Orientation::Top,
                        "DELAY".to_string(),
                    ));

                    self.acquire.emit(acquire::InputMsg::SetData(
                        redpitaya_scpi::acquire::Source::IN1,
                        self.data
                            .redpitaya
                            .data
                            .read_all(redpitaya_scpi::acquire::Source::IN1),
                    ));
                    self.acquire.emit(acquire::InputMsg::SetData(
                        redpitaya_scpi::acquire::Source::IN2,
                        self.data
                            .redpitaya
                            .data
                            .read_all(redpitaya_scpi::acquire::Source::IN2),
                    ));
                }
                trigger::OutputMsg::Normal => {
                    self.graph.emit(graph::InputMsg::SourceStart(
                        graph::level::Orientation::Right,
                        "TRIG".to_string(),
                    ));
                    self.graph.emit(graph::InputMsg::SourceStart(
                        graph::level::Orientation::Top,
                        "DELAY".to_string(),
                    ));

                    self.acquire.emit(acquire::InputMsg::SetData(
                        redpitaya_scpi::acquire::Source::IN1,
                        self.data
                            .redpitaya
                            .data
                            .read_oldest(redpitaya_scpi::acquire::Source::IN1, 16_384),
                    ));
                    self.acquire.emit(acquire::InputMsg::SetData(
                        redpitaya_scpi::acquire::Source::IN2,
                        self.data
                            .redpitaya
                            .data
                            .read_oldest(redpitaya_scpi::acquire::Source::IN2, 16_384),
                    ));
                }
                trigger::OutputMsg::Single => {
                    self.graph.emit(graph::InputMsg::SourceStart(
                        graph::level::Orientation::Right,
                        "TRIG".to_string(),
                    ));
                    self.graph.emit(graph::InputMsg::SourceStart(
                        graph::level::Orientation::Top,
                        "DELAY".to_string(),
                    ));

                    self.acquire.emit(acquire::InputMsg::SetData(
                        redpitaya_scpi::acquire::Source::IN1,
                        self.data
                            .redpitaya
                            .data
                            .read_all(redpitaya_scpi::acquire::Source::IN1),
                    ));
                    self.acquire.emit(acquire::InputMsg::SetData(
                        redpitaya_scpi::acquire::Source::IN2,
                        self.data
                            .redpitaya
                            .data
                            .read_all(redpitaya_scpi::acquire::Source::IN2),
                    ));
                }
            },
            Msg::Quit => {
                self.data.redpitaya.acquire.stop();
                self.data
                    .redpitaya
                    .generator
                    .stop(redpitaya_scpi::generator::Source::OUT1);
                self.data
                    .redpitaya
                    .generator
                    .stop(redpitaya_scpi::generator::Source::OUT2);
                relm4::main_application().quit();
            }
        };
    }

    view! {
        #[name = "window"]
        gtk::Window {
            set_title: Some(env!("CARGO_PKG_NAME")),
            connect_close_request[sender] => move |_| {
                sender.input(Msg::Quit);
                gtk::glib::Propagation::Stop
            },

            #[name = "main_box"]
            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 0,

                append: model.graph.widget(),

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 0,

                    gtk::Notebook {
                        set_scrollable: true,
                        set_vexpand: true,

                        append_page: (model.acquire.widget(), Some(&gtk::Label::new(Some("Acquire")))),
                        append_page: (model.generator.widget(), Some(&gtk::Label::new(Some("Generator")))),
                        append_page: (model.trigger.widget(), Some(&gtk::Label::new(Some("Trigger")))),
                    },
                    #[name = "status_bar"]
                    gtk::Statusbar {
                    },
                },
            },
        },
    }
}

impl Model {
    fn update_status(&self, widgets: &ModelWidgets) {
        let status = format!(
            "{} - {} V/div - {} µs/div",
            self.data.rate,
            self.data.scales.v_div(),
            self.data.scales.h_div()
        );

        widgets
            .status_bar
            .push(widgets.status_bar.context_id("sampling-rate"), &status);
    }

    fn draw(&mut self, widgets: &ModelWidgets) -> Result<(), gtk::cairo::Error> {
        self.update_status(widgets);

        let image = gtk::cairo::ImageSurface::create(
            gtk::cairo::Format::ARgb32,
            self.data.scales.window.width,
            self.data.scales.window.height,
        )?;

        redraw!(self, graph, image);
        redraw!(self, acquire, image);
        redraw!(self, generator, image);
        redraw!(self, trigger, image);

        self.graph.emit(graph::InputMsg::SetImage(image));

        Ok(())
    }

    fn transform(
        &self,
        scales: crate::Scales,
        context: &gtk::cairo::Context,
        width: f64,
        height: f64,
    ) {
        context.set_matrix(gtk::cairo::Matrix::new(
            width / scales.width(),
            0.0,
            0.0,
            -height / scales.height(),
            scales.h.1 * width / scales.width(),
            scales.v.1 * height / scales.height(),
        ));
    }
}
