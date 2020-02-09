use gtk::prelude::*;

#[derive(relm_derive::Msg, Clone)]
pub enum Msg<T: std::clone::Clone + std::cmp::PartialEq> {
    Change(T),
    Set(T),
}

#[derive(Clone)]
pub struct Model<T> {
    pub options: Vec<T>,
    pub current: Option<T>,
}

#[derive(Clone)]
pub struct RadioGroup<T> {
    frame: gtk::Frame,
    radio: Vec<(gtk::RadioButton, T)>,
    phantom: std::marker::PhantomData<T>,
}

impl<T: std::clone::Clone + std::cmp::PartialEq> RadioGroup<T> {
    pub fn set_current(&self, current: T) {
        for &(ref radio, ref option) in self.radio.iter() {
            if current == *option {
                radio.set_active(true);
                break;
            }
        }
    }
}

impl<T: std::clone::Clone + std::cmp::PartialEq> relm::Update for RadioGroup<T> {
    type Model = Model<T>;
    type Msg = Msg<T>;
    type ModelParam = Model<T>;

    fn model(_: &relm::Relm<Self>, model: Self::ModelParam) -> Self::Model {
        model
    }

    fn update(&mut self, event: Self::Msg) {
        match event {
            Msg::Set(value) => self.set_current(value),
            _ => (),
        }
    }
}

impl<T> relm::Widget for RadioGroup<T>
where
    T: std::clone::Clone + std::fmt::Display + std::cmp::PartialEq + 'static,
{
    type Root = gtk::Frame;

    fn root(&self) -> Self::Root {
        self.frame.clone()
    }

    fn view(relm: &relm::Relm<Self>, model: Self::Model) -> Self {
        let frame = gtk::Frame::new(None);

        let flow_box = gtk::FlowBox::new();
        frame.add(&flow_box);

        let mut radio = Vec::new();
        let mut group_member = None;

        for option in model.options.iter() {
            let button = gtk::RadioButton::new_with_label(&format!("{}", option));
            button.join_group(group_member.as_ref());
            flow_box.add(&button);

            {
                let stream = relm.stream().clone();
                let option = option.clone();

                button.connect_toggled(move |f| {
                    if f.get_active() {
                        stream.emit(Msg::Change(option.clone()));
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
            frame,
            radio,
            phantom: std::marker::PhantomData,
        }
    }
}
