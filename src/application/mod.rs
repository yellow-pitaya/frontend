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
        let context = gtk::cairo::Context::new(&$image)?;

        if $image.width() > 0 && $image.height() > 0 {
            $self.transform(
                $self.model.scales,
                &context,
                $image.width() as f64,
                $image.height() as f64,
            );
            context.set_line_width(0.01);

            $self.components.$widget.emit($widget::Msg::Redraw(
                Box::new(context.clone()),
                Box::new($self.model.clone()),
            ));
        }
    };
}

#[derive(relm_derive::Msg, Clone)]
pub enum Msg {
    AcquireRate(redpitaya_scpi::acquire::SamplingRate),
    Draw,
    Level(String, i32),
    Resize(i32, i32),
    TriggerAuto,
    TriggerNormal,
    TriggerSingle,
    Quit,
}

#[derive(Clone)]
pub struct Model {
    stream: relm::StreamHandle<Msg>,
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
            Msg::Draw => self.draw().unwrap(),
            Msg::AcquireRate(rate) => {
                self.model.rate = rate;
                self.model.scales.from_sampling_rate(rate);
                self.update_status();
            }

            Msg::Level(channel, level) => {
                self.model.levels.insert(channel, level);
            }
            Msg::Resize(width, height) => {
                self.model.scales.window.width = width;
                self.model.scales.window.height = height;
                self.draw().unwrap();
            }

            Msg::TriggerAuto | Msg::TriggerSingle => {
                self.components.acquire.emit(acquire::Msg::SetData(
                    redpitaya_scpi::acquire::Source::IN1,
                    self.model
                        .redpitaya
                        .data
                        .read_all(redpitaya_scpi::acquire::Source::IN1),
                ));
                self.components.acquire.emit(acquire::Msg::SetData(
                    redpitaya_scpi::acquire::Source::IN2,
                    self.model
                        .redpitaya
                        .data
                        .read_all(redpitaya_scpi::acquire::Source::IN2),
                ));
            }
            Msg::TriggerNormal => {
                self.components.acquire.emit(acquire::Msg::SetData(
                    redpitaya_scpi::acquire::Source::IN1,
                    self.model
                        .redpitaya
                        .data
                        .read_oldest(redpitaya_scpi::acquire::Source::IN1, 16_384),
                ));
                self.components.acquire.emit(acquire::Msg::SetData(
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
            title: env!("CARGO_PKG_NAME"),
            delete_event(_, _) => (Msg::Quit, gtk::Inhibit(false)),

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
                        Draw => Msg::Draw,
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

        self.widgets.window.show_all();

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

        self.widgets
            .status_bar
            .push(self.widgets.status_bar.context_id("sampling-rate"), &status);
    }

    fn draw(&mut self) -> Result<(), gtk::cairo::Error> {
        self.update_status();

        let image = gtk::cairo::ImageSurface::create(
            gtk::cairo::Format::ARgb32,
            self.model.scales.window.width,
            self.model.scales.window.height,
        )?;

        redraw!(self, graph, image);
        redraw!(self, trigger, image);
        redraw!(self, generator, image);
        redraw!(self, acquire, image);

        self.components.graph.emit(graph::Msg::SetImage(image));

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
            width / scales.get_width(),
            0.0,
            0.0,
            -height / scales.get_height(),
            scales.h.1 * width / scales.get_width(),
            scales.v.1 * height / scales.get_height(),
        ));
    }
}
