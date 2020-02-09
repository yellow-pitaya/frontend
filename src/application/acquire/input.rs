use crate::color::Colorable;
use relm::ContainerWidget;

#[derive(relm_derive::Msg, Clone)]
pub enum Signal {
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
    pub fn new(acquire: &redpitaya_scpi::acquire::Acquire) -> Self {
        Self {
            acquire: acquire.clone(),
            attenuation: 1,
            data: Vec::new(),
            started: false,
            source: redpitaya_scpi::acquire::Source::IN1,
        }
    }
}

#[derive(Clone)]
pub struct Widget {
    model: Model,
    stream: relm::EventStream<<Self as relm::Update>::Msg>,
    page: gtk::Box,
    pub palette: relm::Component<crate::widget::Palette>,
    attenuation: relm::Component<crate::widget::RadioGroup<u8>>,
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

impl relm::Update for Widget {
    type Model = Model;
    type Msg = Signal;
    type ModelParam = Model;

    fn model(_: &relm::Relm<Self>, model: Self::ModelParam) -> Self::Model {
        model
    }

    fn update(&mut self, event: Self::Msg) {
        match event {
            Signal::Attenuation(attenuation) => self.model.attenuation = attenuation,
            Signal::Gain(gain) => self.model.acquire.set_gain(self.model.source, gain),
            Signal::Redraw(context, model) => self.draw(&context, &model),
            Signal::SetData(data) => self.model.data = data,
            Signal::Start => self.model.started = true,
            Signal::Stop => self.model.started = false,
        };
    }
}

impl relm::Widget for Widget {
    type Root = gtk::Box;

    fn root(&self) -> Self::Root {
        self.page.clone()
    }

    fn view(relm: &relm::Relm<Self>, model: Self::Model) -> Self {
        let page = gtk::Box::new(gtk::Orientation::Vertical, 10);

        let palette = page.add_widget::<crate::widget::Palette>(());
        palette.emit(crate::widget::palette::Signal::SetLabel(format!(
            "{}",
            model.source
        )));
        palette.emit(crate::widget::palette::Signal::SetColor(
            model.source.into(),
        ));
        relm::connect!(palette@crate::widget::palette::Signal::Expand, relm, Signal::Start);
        relm::connect!(palette@crate::widget::palette::Signal::Fold, relm, Signal::Stop);

        use gtk::ContainerExt;
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 10);
        palette.widget().add(&vbox);

        let args = crate::widget::radio::Model {
            title: String::from("Gain"),
            options: vec![
                redpitaya_scpi::acquire::Gain::LV,
                redpitaya_scpi::acquire::Gain::HV,
            ],
            current: match model.acquire.get_gain(model.source) {
                Ok(gain) => Some(gain),
                Err(_) => None,
            },
        };
        let gain =
            vbox.add_widget::<crate::widget::RadioGroup<redpitaya_scpi::acquire::Gain>>(args);
        relm::connect!(
            gain@crate::widget::radio::Signal::Change(gain),
            relm,
            Signal::Gain(gain)
        );

        let args = crate::widget::radio::Model {
            title: String::from("Probe attenuation"),
            options: vec![1, 10, 100],
            current: Some(1),
        };
        let attenuation = vbox.add_widget::<crate::widget::RadioGroup<u8>>(args);
        relm::connect!(
            attenuation@crate::widget::radio::Signal::Change(attenuation),
            relm,
            Signal::Attenuation(attenuation)
        );

        Widget {
            model,
            page,
            palette,
            stream: relm.stream().clone(),
            attenuation,
        }
    }
}
