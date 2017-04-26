use gtk::{
    BoxExt,
    ButtonExt,
    ToggleButtonExt,
};

#[derive(Msg)]
pub enum Signal {
    Start,
    Stop,
}

#[derive(Clone)]
pub struct Widget {
    page: ::gtk::Box,
    toggle: ::gtk::ToggleButton,
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

    fn update(&mut self, event: Signal, _: &mut Self::Model) {
        match event {
            Signal::Start => self.toggle.set_label("Stop"),
            Signal::Stop => self.toggle.set_label("Run"),
        };
    }

    fn view(relm: ::relm::RemoteRelm<Signal>, _: &Self::Model) -> Self {
        let page = ::gtk::Box::new(::gtk::Orientation::Vertical, 0);

        let toggle = ::gtk::ToggleButton::new_with_label("Run");
        page.pack_start(&toggle, false, false, 0);

        let stream = relm.stream().clone();
        toggle.connect_toggled(move |w| {
            if w.get_active() {
                stream.emit(Signal::Start);
            } else {
                stream.emit(Signal::Stop);
            }
        });

        Widget {
            page: page,
            toggle: toggle,
        }
    }
}
