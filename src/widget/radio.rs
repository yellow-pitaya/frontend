use gtk::prelude::*;

#[derive(Debug)]
pub enum OutputMsg<T: std::fmt::Debug> {
    Change(T),
}

pub struct Options<T> {
    pub options: Vec<T>,
    pub current: Option<T>,
    pub label: &'static str,
}

pub struct Model<T> {
    options: Options<T>,
    radio: Vec<(gtk::CheckButton, T)>,
}

#[relm4::component(pub)]
impl<
        T: std::fmt::Debug + std::cmp::PartialEq + std::clone::Clone + std::fmt::Display + 'static,
    > relm4::SimpleComponent for Model<T>
{
    type Init = Options<T>;
    type Input = ();
    type Output = OutputMsg<T>;

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: relm4::ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let mut radio = Vec::new();
        let mut group_member = None;

        for option in init.options.iter() {
            let button = gtk::CheckButton::with_label(&format!("{option}"));
            button.set_group(group_member.as_ref());

            button.connect_toggled(gtk::glib::clone!(
                #[strong]
                sender,
                #[strong]
                option,
                move |f| if f.is_active() {
                    sender.output(OutputMsg::Change(option.clone())).ok();
                }
            ));

            if init.current == Some(option.clone()) {
                button.set_active(true);
            }

            if group_member.is_none() {
                group_member = Some(button.clone());
            }

            radio.push((button, option.clone()));
        }

        let model = Self {
            options: init,
            radio,
        };

        let widgets = view_output!();

        for (widget, _) in &model.radio {
            widgets.flow_box.append(widget);
        }

        relm4::ComponentParts { model, widgets }
    }

    view! {
        gtk::Frame {
            #[watch]
            set_label: Some(model.options.label),

            #[name = "flow_box"]
            gtk::FlowBox {
            }
        }
    }
}
