use color::Colorable;
use relm::ContainerWidget;
use super::Model;
use super::Signal;

#[derive(Clone)]
pub struct Widget {
    data: ::std::cell::RefCell<Vec<f64>>,
    stream: ::relm::EventStream<Signal>,
    page: ::gtk::Box,
    pub palette: ::relm::Component<::widget::Palette>,
    source: ::redpitaya_scpi::acquire::Source,
    attenuation: ::relm::Component<::widget::RadioGroup<u8>>,
}

impl Widget {
    fn is_started(&self) -> bool {
        self.palette.widget().get_active()
    }

    pub fn set_data(&self, data: Vec<f64>) {
        *self.data.borrow_mut() = data;
        self.stream.emit(Signal::Data);
    }

    fn draw_data(&self, context: &::cairo::Context, scales: ::Scales, attenuation: u8) {
        let data = self.data.borrow();

        context.set_line_width(0.05);

        for sample in 0..scales.n_samples {
            let x = scales.sample_to_ms(sample);
            let y = data[sample as usize];

            context.line_to(x, y * attenuation as f64);
            context.move_to(x, y * attenuation as f64);
        }
        context.stroke();
    }
}

impl ::application::Panel for Widget {
    fn draw(&self, context: &::cairo::Context, model: &::application::Model) {
        if !self.is_started() {
            return;
        }

        context.set_color(self.source.into());

        context.translate(0.0, model.offset(self.source));

        context.move_to(model.scales.h.0, 0.0);
        context.line_to(model.scales.h.1, 0.0);
        context.stroke();

        let attenuation = match self.attenuation.widget().get_current() {
            Some(attenuation) => attenuation,
            None => 1,
        };
        self.draw_data(&context, model.scales, attenuation);
    }

    fn update_scales(&self, _: ::Scales) {
    }
}

impl ::relm::Widget for Widget {
    type Model = Model;
    type Msg = Signal;
    type Root = ::gtk::Box;
    type ModelParam = Model;

    fn model(model: Self::ModelParam) -> Self::Model {
        model
    }

    fn root(&self) -> &Self::Root {
        &self.page
    }

    fn update(&mut self, event: Signal, model: &mut Self::Model) {
        match event {
            Signal::Gain(gain) => model.acquire.set_gain(model.source, gain),
            _ => (),
        };
    }

    fn view(relm: &::relm::RemoteRelm<Self>, model: &Self::Model) -> Self {
        let page = ::gtk::Box::new(::gtk::Orientation::Vertical, 10);

        let palette = page.add_widget::<::widget::Palette, _>(&relm, ());
        palette.widget().set_label(format!("{}", model.source).as_str());
        palette.widget().set_color(model.source.into());
        connect!(palette@::widget::palette::Signal::Expand, relm, Signal::Start);
        connect!(palette@::widget::palette::Signal::Fold, relm, Signal::Stop);

        let vbox  = ::gtk::Box::new(::gtk::Orientation::Vertical, 10);
        palette.widget().add(&vbox);

        let args = ::widget::radio::Model {
            title: String::from("Gain"),
            options: vec![
                ::redpitaya_scpi::acquire::Gain::LV,
                ::redpitaya_scpi::acquire::Gain::HV,
            ],
            current: match model.acquire.get_gain(model.source) {
                Ok(gain) => Some(gain),
                Err(_) => None,
            },
        };
        let gain = vbox.add_widget::<::widget::RadioGroup<::redpitaya_scpi::acquire::Gain>, _>(&relm, args);
        connect!(
            gain@::widget::radio::Signal::Change(gain),
            relm,
            Signal::Gain(gain)
        );

        let args = ::widget::radio::Model {
            title: String::from("Probe attenuation"),
            options: vec![1, 10, 100],
            current: Some(1),
        };
        let attenuation = vbox.add_widget::<::widget::RadioGroup<u8>, _>(&relm, args);
        connect!(
            attenuation@::widget::radio::Signal::Change(attenuation),
            relm,
            Signal::Attenuation(attenuation)
        );

        let data = ::std::cell::RefCell::new(Vec::new());
        let stream = relm.stream().clone();
        let source = model.source;

        Widget {
            data,
            page,
            palette,
            stream,
            source,
            attenuation,
        }
    }
}
