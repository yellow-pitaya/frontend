use gtk::prelude::*;

#[derive(Debug)]
pub enum InputMsg {
    Expand,
    Fold,
}

#[derive(Debug)]
pub enum OutputMsg {
    Change(f64),
}

pub struct Options {
    pub label: &'static str,
    pub value: f64,
    pub digits: u32,
    pub adjustment: gtk::Adjustment,
}

pub struct Model {
    options: Options,
}

#[relm4::component(pub)]
impl relm4::Component for Model {
    type CommandOutput = ();
    type Init = Options;
    type Input = InputMsg;
    type Output = OutputMsg;

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: relm4::ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let model = Self { options: init };

        let widgets = view_output!();

        let adjustment = model.options.adjustment.clone();
        adjustment.set_step_increment(adjustment.step_increment() / 10.0);
        adjustment.set_page_increment(adjustment.page_increment() / 10.0);
        widgets.spin.set_adjustment(&adjustment);

        relm4::ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut ModelWidgets,
        msg: Self::Input,
        _: relm4::ComponentSender<Self>,
        _: &Self::Root,
    ) {
        match msg {
            InputMsg::Expand => {
                widgets.scale.set_draw_value(false);
                widgets.spin.show();
            }
            InputMsg::Fold => {
                widgets.scale.set_draw_value(true);
                widgets.spin.hide();
            }
        };
    }

    view! {
        #[name = "frame"]
        gtk::Frame {
            set_label: Some(model.options.label),
            set_hexpand: false,

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                //set_border_width: 5,
                set_spacing: 5,

                #[name = "toggle"]
                gtk::CheckButton {
                    connect_toggled[sender] => move |this| if this.is_active() {
                        sender.input(InputMsg::Expand);
                    } else {
                        sender.input(InputMsg::Fold);
                    }
                },
                gtk::Box {
                    set_hexpand: true,
                    set_orientation: gtk::Orientation::Vertical,

                    #[name = "scale"]
                    gtk::Scale {
                        add_mark: (0., gtk::PositionType::Top, None),
                        set_adjustment: &model.options.adjustment,
                        set_draw_value: true,
                        #[watch]
                        set_value: model.options.value,
                        set_value_pos: gtk::PositionType::Bottom,

                        connect_change_value[sender] => move |_, _, value| {
                            sender.output(OutputMsg::Change(value)).ok();
                            gtk::glib::Propagation::Proceed
                        },
                    },
                    #[name = "spin"]
                    gtk::SpinButton {
                        set_digits: model.options.digits,
                        #[watch]
                        set_value: model.options.value,
                        set_visible: false,

                        connect_value_changed[sender] => move |this| {
                            sender.output(OutputMsg::Change(this.value())).ok();
                        },
                    },
                },
            },
        },
    }
}
