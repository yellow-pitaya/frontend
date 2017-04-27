use relm::ContainerWidget;

#[derive(Msg)]
pub enum Signal {
    Start,
    Stop,
}

#[derive(Clone)]
pub struct Widget {
    page: ::gtk::Box,
    palette: ::relm::Component<::widget::Palette>,
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

        let palette = page.add_widget::<::widget::Palette, _>(&relm);
        palette.widget().set_label("IN 1");
        connect!(palette@::widget::Signal::Expand, relm, Signal::Start);
        connect!(palette@::widget::Signal::Fold, relm, Signal::Stop);

        Widget {
            page: page,
            palette: palette,
        }
    }
}
