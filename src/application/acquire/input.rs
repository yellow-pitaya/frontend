use crate::color::Colorable;
use crate::widget::palette::Msg::*;
use crate::widget::radio::Msg::*;
use crate::widget::Palette;
use gtk::prelude::*;

type GainWidget = crate::widget::RadioGroup<redpitaya_scpi::acquire::Gain>;
type AttenuationWidget = crate::widget::RadioGroup<u8>;

#[derive(relm_derive::Msg, Clone)]
pub enum Msg {
    Attenuation(u8),
    Gain(redpitaya_scpi::acquire::Gain),
    SetData(Vec<f64>),
    Start,
    Stop,
    Redraw(Box<cairo::Context>, Box<crate::application::Model>),
}

#[derive(Clone)]
pub struct Model {
    acquire: redpitaya_scpi::acquire::Acquire,
    attenuation: u8,
    data: Vec<f64>,
    source: redpitaya_scpi::acquire::Source,
    started: bool,
}

impl Model {
    pub fn new(
        acquire: &redpitaya_scpi::acquire::Acquire,
        source: redpitaya_scpi::acquire::Source,
    ) -> Self {
        Self {
            acquire: acquire.clone(),
            attenuation: 1,
            data: Vec::new(),
            started: false,
            source,
        }
    }
}

#[relm_derive::widget(Clone)]
impl relm::Widget for Widget {
    fn model(_: &relm::Relm<Self>, model: Model) -> Model {
        model
    }

    fn update(&mut self, event: Msg) {
        match event {
            Msg::Attenuation(attenuation) => self.model.attenuation = attenuation,
            Msg::Gain(gain) => self.model.acquire.set_gain(self.model.source, gain),
            Msg::Redraw(context, model) => self.draw(&context, &model),
            Msg::SetData(data) => self.model.data = data,
            Msg::Start => self.model.started = true,
            Msg::Stop => self.model.started = false,
        };
    }

    fn init_view(&mut self) {
        self.components.palette
            .emit(crate::widget::palette::Msg::SetLabel(format!(
                "{}",
                self.model.source
            )));
        self.components.palette.emit(crate::widget::palette::Msg::SetColor(
            self.model.source.into(),
        ));
    }

    view! {
        #[name="page"]
        gtk::Box {
            orientation: gtk::Orientation::Vertical,
            spacing: 10,
            #[name="palette"]
            Palette {
                Expand => Msg::Start,
                Fold => Msg::Stop,

                gtk::Box {
                    orientation: gtk::Orientation::Vertical,
                    spacing: 10,

                    GainWidget(crate::widget::radio::Model {
                        options: vec![
                            redpitaya_scpi::acquire::Gain::LV,
                            redpitaya_scpi::acquire::Gain::HV,
                        ],
                        current: self.model.acquire.get_gain(self.model.source).ok(),
                    }) {
                        label: Some("Gain"),
                        Change(gain) => Msg::Gain(gain),
                    },
                    AttenuationWidget(crate::widget::radio::Model {
                        options: vec![1, 10, 100],
                        current: Some(1),
                    }) {
                        label: Some("Probe attenuation"),
                        Change(attenuation) => Msg::Attenuation(attenuation),
                    },
                },
            },
        },
    }
}

impl Widget {
    fn is_started(&self) -> bool {
        self.model.started
    }

    fn draw(&self, context: &cairo::Context, model: &crate::application::Model) {
        if !self.is_started() {
            return;
        }

        context.set_color(self.model.source.into());

        context.translate(0.0, model.offset(self.model.source));

        context.move_to(model.scales.h.0, 0.0);
        context.line_to(model.scales.h.1, 0.0);
        context.stroke();

        self.draw_data(&context, model.scales, self.model.attenuation);
    }

    fn draw_data(&self, context: &cairo::Context, scales: crate::Scales, attenuation: u8) {
        if self.model.data.is_empty() {
            return;
        }

        context.set_line_width(0.05);

        for sample in 0..scales.n_samples {
            let x = scales.sample_to_ms(sample);
            let y = self.model.data[sample as usize];

            context.line_to(x, y * attenuation as f64);
            context.move_to(x, y * attenuation as f64);
        }
        context.stroke();
    }
}
