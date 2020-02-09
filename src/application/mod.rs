mod acquire;
mod generator;
mod graph;
mod trigger;

use acquire::Msg::{Rate, Start as AcquireStart, Stop as AcquireStop};
use acquire::Widget as AcquireWidget;
use generator::Msg::*;
use generator::Widget as GeneratorWidget;
use graph::Msg::*;
use graph::Widget as GraphWidget;
use gtk::prelude::*;
use trigger::Msg::*;
use trigger::Widget as TriggerWidget;

macro_rules! redraw {
    ($self:ident, $widget:ident, $image:ident) => {
        let context = cairo::Context::new(&$image);

        $self.transform(
            $self.model.scales,
            &context,
            $image.get_width() as f64,
            $image.get_height() as f64,
        );
        context.set_line_width(0.01);

        $self.$widget.emit($widget::Msg::Redraw(
            Box::new(context.clone()),
            Box::new($self.model.clone()),
        ));
    };
}

#[derive(relm_derive::Msg, Clone)]
pub enum Msg {
    AcquireRate(redpitaya_scpi::acquire::SamplingRate),
    GraphDraw,
    Level(String, i32),
    NeedDraw,
    Resize(i32, i32),
    TriggerAuto,
    TriggerNormal,
    TriggerSingle,
    Quit,
}

#[derive(Clone)]
pub struct Model {
    stream: relm::EventStream<Msg>,
    rate: redpitaya_scpi::acquire::SamplingRate,
    redpitaya: redpitaya_scpi::Redpitaya,
    scales: crate::Scales,
    levels: std::collections::HashMap<String, i32>,
}

impl Model {
    fn offset<D>(&self, channel: D) -> f64
    where
        D: std::fmt::Display,
    {
        let channel = format!("{}", channel);

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

#[relm_derive::widget(Clone)]
impl relm::Widget for Widget {
    fn model(relm: &relm::Relm<Self>, redpitaya: redpitaya_scpi::Redpitaya) -> Model {
        let mut scales = crate::Scales {
            h: (0.0, 0.0),
            v: (-5.0, 5.0),
            n_samples: redpitaya.data.buffer_size().unwrap(),
            window: crate::scales::Rect {
                width: 0,
                height: 0,
            },
        };

        let rate = redpitaya.acquire.get_decimation().unwrap().into();
        scales.from_sampling_rate(rate);

        Model {
            stream: relm.stream().clone(),
            rate,
            redpitaya,
            scales,
            levels: std::collections::HashMap::new(),
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            Msg::AcquireRate(rate) => {
                self.model.rate = rate;
                self.model.scales.from_sampling_rate(rate);
                self.graph.emit(graph::Msg::Invalidate);
            }

            Msg::NeedDraw => self.graph.emit(graph::Msg::Invalidate),
            Msg::GraphDraw => self.draw(),
            Msg::Level(channel, level) => {
                self.model.levels.insert(channel, level);
            }
            Msg::Resize(width, height) => {
                self.model.scales.window.width = width;
                self.model.scales.window.height = height;
                self.model.stream.emit(Msg::NeedDraw)
            }

            Msg::TriggerAuto | Msg::TriggerSingle => {
                self.acquire.emit(acquire::Msg::SetData(
                    redpitaya_scpi::acquire::Source::IN1,
                    self.model
                        .redpitaya
                        .data
                        .read_all(redpitaya_scpi::acquire::Source::IN1),
                ));
                self.acquire.emit(acquire::Msg::SetData(
                    redpitaya_scpi::acquire::Source::IN2,
                    self.model
                        .redpitaya
                        .data
                        .read_all(redpitaya_scpi::acquire::Source::IN2),
                ));
            }
            Msg::TriggerNormal => {
                self.acquire.emit(acquire::Msg::SetData(
                    redpitaya_scpi::acquire::Source::IN1,
                    self.model
                        .redpitaya
                        .data
                        .read_oldest(redpitaya_scpi::acquire::Source::IN1, 16_384),
                ));
                self.acquire.emit(acquire::Msg::SetData(
                    redpitaya_scpi::acquire::Source::IN2,
                    self.model
                        .redpitaya
                        .data
                        .read_oldest(redpitaya_scpi::acquire::Source::IN2, 16_384),
                ));
            }
            Msg::Quit => {
                self.model.redpitaya.acquire.stop();
                self.model
                    .redpitaya
                    .generator
                    .stop(redpitaya_scpi::generator::Source::OUT1);
                self.model
                    .redpitaya
                    .generator
                    .stop(redpitaya_scpi::generator::Source::OUT2);
                gtk::main_quit();
            }
        };
    }

    view! {
        #[name="window"]
        gtk::Window {
            //type: gtk::WindowType::Toplevel,
            title: "Yellow Pitaya",
            destroy(_) => Msg::Quit,

            #[name="main_box"]
            gtk::Box {
                orientation: gtk::Orientation::Horizontal,
                spacing: 0,

                gtk::EventBox {
                    child: {
                        pack_type: gtk::PackType::Start,
                        expand: true,
                        fill: true,
                        padding: 0,
                    },

                    #[name="graph"]
                    GraphWidget {
                        Draw => Msg::GraphDraw,
                        Level(ref channel, offset) => Msg::Level(channel.clone(), offset),
                        Resize(w, h) => Msg::Resize(w, h),
                    },
                },

                gtk::Box {
                    orientation: gtk::Orientation::Vertical,
                    spacing: 0,
                    child: {
                        pack_type: gtk::PackType::Start,
                        expand: false,
                        fill: false,
                        padding: 0,
                    },

                    gtk::Notebook {
                        child: {
                            pack_type: gtk::PackType::Start,
                            expand: true,
                            fill: true,
                            padding: 0,
                        },
                        scrollable: true,

                        gtk::Box {
                            orientation: gtk::Orientation::Vertical,
                            spacing: 0,
                            border_width: 10,
                            child: {
                                tab_label: Some("Acquire"),
                            },

                            #[name="acquire"]
                            AcquireWidget(self.model.redpitaya.acquire.clone()) {
                                Rate(rate) => Msg::AcquireRate(rate),
                                AcquireStart(source) => graph@graph::Msg::SourceStart(graph::level::Orientation::Left, format!("{}", source)),
                                AcquireStop(source) => graph@graph::Msg::SourceStop(graph::level::Orientation::Left, format!("{}", source)),
                            },
                        },

                        gtk::ScrolledWindow {
                            child: {
                                tab_label: Some("Generator"),
                            },

                            #[name="generator"]
                            GeneratorWidget(self.model.redpitaya.generator.clone()) {
                                Amplitude(_, _) => Msg::NeedDraw,
                                DutyCycle(_, _) => Msg::NeedDraw,
                                Frequency(_, _) => Msg::NeedDraw,
                                Offset(_, _) => Msg::NeedDraw,
                                Form(_, _) => Msg::NeedDraw,
                                Start(source) => graph@graph::Msg::SourceStart(graph::level::Orientation::Left, format!("{}", source)),
                                Stop(source) => graph@graph::Msg::SourceStop(graph::level::Orientation::Left, format!("{}", source)),
                            },
                        },

                        gtk::Box {
                            orientation: gtk::Orientation::Vertical,
                            spacing: 0,
                            border_width: 10,
                            child: {
                                tab_label: Some("Trigger"),
                            },

                            #[name="trigger"]
                            TriggerWidget(self.model.redpitaya.trigger.clone()) {
                                Auto => Msg::TriggerAuto,
                                Normal => Msg::TriggerNormal,
                                Single => Msg::TriggerSingle,

                                Auto => graph@graph::Msg::SourceStop(graph::level::Orientation::Right, "TRIG".to_string()),
                                Normal => graph@graph::Msg::SourceStart(graph::level::Orientation::Right, "TRIG".to_string()),
                                Single => graph@graph::Msg::SourceStart(graph::level::Orientation::Right, "TRIG".to_string()),

                                Auto => graph@graph::Msg::SourceStop(graph::level::Orientation::Top, "DELAY".to_string()),
                                Normal => graph@graph::Msg::SourceStart(graph::level::Orientation::Top, "DELAY".to_string()),
                                Single => graph@graph::Msg::SourceStart(graph::level::Orientation::Top, "DELAY".to_string()),
                            },
                        },
                    },
                    #[name="status_bar"]
                    gtk::Statusbar {
                        child: {
                            pack_type: gtk::PackType::Start,
                            expand: false,
                            fill: true,
                            padding: 0,
                        },
                    },
                },
            },
        },
    }

    fn init_view(&mut self) {
        crate::color::Color::init();

        self.model
            .redpitaya
            .data
            .set_units(redpitaya_scpi::data::Unit::VOLTS);

        self.window.show_all();

        self.model.redpitaya.acquire.start();

        // @FIXME
        // self.trigger.widget().single_button.set_visible(false);
    }
}

impl Widget {
    fn update_status(&self) {
        let status = format!(
            "{} - {} V/div - {} µs/div",
            self.model.rate,
            self.model.scales.v_div(),
            self.model.scales.h_div()
        );

        self.status_bar
            .push(self.status_bar.get_context_id("sampling-rate"), &status);
    }

    fn draw(&mut self) {
        self.update_status();

        let image = cairo::ImageSurface::create(
            cairo::Format::ARgb32,
            self.model.scales.window.width,
            self.model.scales.window.height,
        )
        .unwrap();

        redraw!(self, graph, image);
        redraw!(self, trigger, image);
        redraw!(self, generator, image);
        redraw!(self, acquire, image);

        self.graph.emit(graph::Msg::SetImage(image));
    }

    fn transform(&self, scales: crate::Scales, context: &cairo::Context, width: f64, height: f64) {
        context.set_matrix(cairo::Matrix {
            xx: width / scales.get_width(),
            xy: 0.0,
            yy: -height / scales.get_height(),
            yx: 0.0,
            x0: scales.h.1 * width / scales.get_width(),
            y0: scales.v.1 * height / scales.get_height(),
        });
    }
}
