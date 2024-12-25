use gtk::prelude::*;

use crate::color::Colorable;

#[derive(Debug)]
pub enum InputMsg {
    Attenuation(u8),
    Gain(redpitaya_scpi::acquire::Gain),
    SetData(Vec<f64>),
    Redraw(Box<gtk::cairo::Context>, Box<crate::application::Data>),
    Start,
    Stop,
}

#[derive(Debug)]
pub enum OutputMsg {
    Start,
    Stop,
}

pub struct Model {
    acquire: redpitaya_scpi::acquire::Acquire,
    attenuation_radio: relm4::Controller<crate::widget::RadioGroup<u8>>,
    attenuation: u8,
    data: Vec<f64>,
    gain: relm4::Controller<crate::widget::RadioGroup<redpitaya_scpi::acquire::Gain>>,
    palette: relm4::Controller<crate::widget::Palette>,
    source: redpitaya_scpi::acquire::Source,
    started: bool,
}

#[relm4::component(pub)]
impl relm4::SimpleComponent for Model {
    type Init = (
        redpitaya_scpi::acquire::Acquire,
        redpitaya_scpi::acquire::Source,
    );
    type Input = InputMsg;
    type Output = OutputMsg;

    fn init(
        init: Self::Init,
        _root: Self::Root,
        sender: relm4::ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        use relm4::Component as _;
        use relm4::ComponentController as _;
        use relm4::RelmContainerExt as _;

        let palette = crate::widget::Palette::builder()
            .launch((init.1.to_string(), init.1.into()))
            .forward(sender.input_sender(), |output| match output {
                crate::widget::palette::OutputMsg::Expand => InputMsg::Start,
                crate::widget::palette::OutputMsg::Fold => InputMsg::Stop,
            });

        let gain = crate::widget::RadioGroup::builder()
            .launch(crate::widget::radio::Options {
                options: vec![
                    redpitaya_scpi::acquire::Gain::LV,
                    redpitaya_scpi::acquire::Gain::HV,
                ],
                current: init.0.gain(init.1).ok(),
                label: "Gain",
            })
            .forward(sender.input_sender(), |output| {
                let crate::widget::radio::OutputMsg::Change(gain) = output;
                InputMsg::Gain(gain)
            });

        let attenuation_radio = crate::widget::RadioGroup::builder()
            .launch(crate::widget::radio::Options {
                options: vec![1, 10, 100],
                current: Some(1),
                label: "Probe attenuation",
            })
            .forward(sender.input_sender(), |output| {
                let crate::widget::radio::OutputMsg::Change(attenuation) = output;
                InputMsg::Attenuation(attenuation)
            });

        let model = Self {
            acquire: init.0,
            attenuation: 1,
            attenuation_radio,
            data: Vec::new(),
            gain,
            started: false,
            source: init.1,
            palette,
        };

        let widgets = view_output!();

        model.palette.widgets().container_add(&widgets.child);

        relm4::ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: relm4::ComponentSender<Self>) {
        use InputMsg::*;

        match msg {
            Attenuation(attenuation) => self.attenuation = attenuation,
            Gain(gain) => self.acquire.set_gain(self.source, gain),
            Redraw(context, model) => self.draw(&context, &model).unwrap(),
            SetData(data) => self.data = data,
            Start => {
                self.started = true;
                sender.output(OutputMsg::Start).ok();
            }
            Stop => {
                self.started = false;
                sender.output(OutputMsg::Stop).ok();
            }
        };
    }

    view! {
        #[name = "page"]
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 10,

            append: model.palette.widget(),
        },
        #[name = "child"]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 10,

            append: model.gain.widget(),
            append: model.attenuation_radio.widget(),
        },
    }
}

impl Model {
    fn is_started(&self) -> bool {
        self.started
    }

    fn draw(
        &self,
        context: &gtk::cairo::Context,
        data: &crate::application::Data,
    ) -> Result<(), gtk::cairo::Error> {
        if !self.is_started() {
            return Ok(());
        }

        context.set_color(self.source.into());

        context.translate(0.0, data.offset(self.source));

        context.move_to(data.scales.h.0, 0.0);
        context.line_to(data.scales.h.1, 0.0);
        context.stroke()?;

        self.draw_data(context, data.scales, self.attenuation)
    }

    fn draw_data(
        &self,
        context: &gtk::cairo::Context,
        scales: crate::Scales,
        attenuation: u8,
    ) -> Result<(), gtk::cairo::Error> {
        if self.data.is_empty() {
            return Ok(());
        }

        context.set_line_width(0.05);

        for sample in 0..scales.n_samples {
            let x = scales.sample_to_ms(sample);
            let y = self.data[sample as usize];

            context.line_to(x, y * attenuation as f64);
            context.move_to(x, y * attenuation as f64);
        }
        context.stroke()
    }
}
