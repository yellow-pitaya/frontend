use gtk::{
    ContainerExt,
    ToggleButtonExt,
};

#[derive(Clone)]
pub enum Signal<T> {
    Change(T),
}

impl<T> ::relm::DisplayVariant for Signal<T> {
    fn display_variant(&self) -> &'static str {
        match *self {
            Signal::Change(_) => "Signal::Change",
        }
    }
}

#[derive(Clone)]
pub struct Model<T> {
    pub title: String,
    pub options: Vec<T>,
    pub current: Option<T>,
}

#[derive(Clone)]
pub struct RadioGroup<T> {
    frame: ::gtk::Frame,
    radio: Vec<(::gtk::RadioButton, T)>,
    phantom: ::std::marker::PhantomData<T>,
}

impl<T: ::std::clone::Clone> RadioGroup<T> {
    pub fn get_current(&self) -> Option<T> {
        for &(ref radio, ref signal) in self.radio.iter() {
            if radio.get_active() {
                return Some(signal.clone());
            }
        }

        None
    }
}

impl<T> ::relm::Widget for RadioGroup<T>
    where T: ::std::clone::Clone + ::std::fmt::Display + ::std::cmp::PartialEq + 'static
{
    type Model = Model<T>;
    type Msg = Signal<T>;
    type Root = ::gtk::Frame;
    type ModelParam = Model<T>;

    fn model(model: Self::ModelParam) -> Self::Model {
        model
    }

    fn root(&self) -> &Self::Root {
        &self.frame
    }

    fn update(&mut self, _: Signal<T>, _: &mut Self::Model) {
    }

    fn view(relm: &::relm::RemoteRelm<Self>, model: &Self::Model) -> Self {
        let frame = ::gtk::Frame::new(model.title.as_str());

        let flow_box = ::gtk::FlowBox::new();
        frame.add(&flow_box);

        let mut radio = Vec::new();
        let mut group_member = None;

        for option in model.options.iter() {
            let button = ::gtk::RadioButton::new_with_label_from_widget(
                group_member.as_ref(),
                format!("{}", option).as_str()
            );
            flow_box.add(&button);

            {
                let stream = relm.stream().clone();
                let option = option.clone();

                button.connect_toggled(move |f| {
                    if f.get_active() {
                        stream.emit(
                            Signal::Change(option.clone())
                        );
                    }
                });
            }

            if model.current == Some(option.clone()) {
                button.set_active(true);
            }

            if group_member == None {
                group_member = Some(button.clone());
            }

            radio.push((button, option.clone()));
        }

        RadioGroup {
            frame: frame,
            radio: radio,
            phantom: ::std::marker::PhantomData,
        }
    }
}
