use gtk::prelude::*;

#[derive(Debug)]
pub enum InputMsg {
    Expand,
    Fold,
}

#[derive(Debug)]
pub enum OutputMsg {
    Expand,
    Fold,
}

#[derive(Debug)]
pub struct Model {}

impl relm4::ContainerChild for ModelWidgets {
    type Child = gtk::Widget;
}

impl relm4::RelmContainerExt for ModelWidgets {
    fn container_add(&self, widget: &impl AsRef<Self::Child>) {
        self.parent.append(widget.as_ref());
    }
}

#[relm4::component(pub)]
impl relm4::Component for Model {
    type CommandOutput = ();
    type Init = (String, crate::Color);
    type Input = InputMsg;
    type Output = OutputMsg;

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: relm4::ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let model = Self {};

        let widgets = view_output!();
        widgets.parent.set_visible(false);

        let context = widgets.border.style_context();
        context.add_class(&init.1.to_string());

        relm4::ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut ModelWidgets,
        msg: Self::Input,
        sender: relm4::ComponentSender<Self>,
        _: &Self::Root,
    ) {
        match msg {
            InputMsg::Expand => {
                widgets.parent.set_visible(true);
                sender.output(OutputMsg::Expand).ok();
            }
            InputMsg::Fold => {
                widgets.parent.set_visible(false);
                sender.output(OutputMsg::Fold).ok();
            }
        }
    }

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,

            #[name = "border"]
            gtk::Box {
                set_hexpand: false,

                #[name = "toggle"]
                gtk::ToggleButton {
                    set_hexpand: true,
                    set_label: &init.0,
                    set_margin_bottom: 1,
                    set_margin_end: 1,
                    set_margin_start: 1,
                    set_margin_top: 1,

                    connect_toggled => move |this| if this.is_active() {
                        sender.input(InputMsg::Expand);
                    } else {
                        sender.input(InputMsg::Fold);
                    }
                },
            },
            #[name = "parent"]
            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
            },
        },
    }
}
