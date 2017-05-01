use gtk::{
    self,
    BoxExt,
    OrientableExt,
    WidgetExt,
};
use relm_attributes::widget;
use super::Output;
use super::Model;

#[widget]
impl ::relm::Widget for Widget {
    fn model(generator: ::redpitaya_scpi::generator::Generator) -> ::redpitaya_scpi::generator::Generator {
        generator
    }

    fn update(&mut self, _: (), _: &mut Self::Model) {
    }

    view! {
        gtk::Box {
            orientation: gtk::Orientation::Vertical,
            spacing: 10,
            #[name="out1"]
            Output(Model {
                source: ::redpitaya_scpi::generator::Source::OUT1,
                generator: model.clone()
            }),
            #[name="out2"]
            Output(Model {
                source: ::redpitaya_scpi::generator::Source::OUT2,
                generator: model.clone()
            }),
        },
    }
}

impl ::application::Panel for Widget {
    fn draw(&self, context: &::cairo::Context, model: &::application::Model) {
        context.save();
        self.out1.widget().draw(&context, &model);
        context.restore();
        context.save();
        self.out2.widget().draw(&context, &model);
        context.restore();
    }
}
