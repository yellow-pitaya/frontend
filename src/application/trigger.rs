use ::gtk::{
    BoxExt,
    ContainerExt,
    RangeExt,
};

#[derive(Msg)]
pub enum Signal {
    Delay(u16),
    Level(f32),
}

#[derive(Clone)]
pub struct Widget {
    pub page: ::gtk::Box,
    pub delay_scale: ::gtk::Scale,
    pub level_scale: ::gtk::Scale,
}

impl ::relm::Widget for Widget {
    type Model = ();
    type Msg = Signal;
    type Root = ::gtk::Box;

    fn model() -> Self::Model {
    }

    fn root(&self) -> &Self::Root {
        &self.page
    }

    fn update(&mut self, _: Signal, _: &mut Self::Model) {
    }

    fn view(relm: ::relm::RemoteRelm<Signal>, _: &Self::Model) -> Self {
        let page = ::gtk::Box::new(::gtk::Orientation::Vertical, 0);

        let frame = ::gtk::Frame::new("Delay");
        page.pack_start(&frame, false, true, 0);

        let delay_scale = ::gtk::Scale::new_with_range(::gtk::Orientation::Horizontal, 0.0, 16384.0, 1.0);
        delay_scale.add_mark(0.0, ::gtk::PositionType::Top, None);

        delay_scale.connect_format_value(move |_, value| {
            format!("{:.0} Sample", value)
        });

        let stream = relm.stream().clone();
        delay_scale.connect_change_value(move |_, _, value| {
            stream.emit(Signal::Delay(value as u16));

            ::gtk::Inhibit(false)
        });
        frame.add(&delay_scale);

        let frame = ::gtk::Frame::new("Level");
        page.pack_start(&frame, false, true, 0);

        let level_scale = ::gtk::Scale::new_with_range(::gtk::Orientation::Horizontal, -10.0, 10.0, 0.1);
        level_scale.add_mark(0.0, ::gtk::PositionType::Top, None);

        level_scale.connect_format_value(move |_, value| {
            format!("{:.2} mV", value)
        });

        let stream = relm.stream().clone();
        level_scale.connect_change_value(move |_, _, value| {
            stream.emit(Signal::Level(value as f32));

            ::gtk::Inhibit(false)
        });
        frame.add(&level_scale);

        Widget {
            page: page,
            delay_scale: delay_scale,
            level_scale: level_scale,
        }
    }
}
