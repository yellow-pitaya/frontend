mod acquire;
mod generator;
mod graph;
mod trigger;

use cairo::prelude::*;
use gtk::{
    BoxExt,
    ContainerExt,
    WidgetExt,
    WindowExt,
};
use relm::ContainerWidget;

trait Panel {
    fn draw(&self, context: &::cairo::Context, model: &Model);
}

#[derive(Clone)]
pub enum Signal {
    AcquireAttenuation(::redpitaya_scpi::acquire::Source, u8),
    AcquireRate(::redpitaya_scpi::acquire::SamplingRate),
    GraphDraw,
    TriggerAuto,
    TriggerNormal,
    TriggerSingle,
    Quit,
}

impl ::relm::DisplayVariant for Signal {
    fn display_variant(&self) -> &'static str {
        match *self {
            Signal::AcquireAttenuation(_, _) => "Signal::AcquireAttenuation",
            Signal::AcquireRate(_) => "Signal::AcquireRate",
            Signal::GraphDraw => "Signal::GraphDraw",
            Signal::TriggerAuto => "Signal::TriggerAuto",
            Signal::TriggerNormal => "Signal::TriggerNormal",
            Signal::TriggerSingle => "Signal::TriggerSingle",
            Signal::Quit => "Signal::Quit",
        }
    }
}

#[derive(Clone)]
pub struct Model {
    attenuation: u8,
    rate: ::redpitaya_scpi::acquire::SamplingRate,
    redpitaya: ::redpitaya_scpi::Redpitaya,
    scales: ::Scales,
}

#[derive(Clone)]
pub struct Application {
    window: ::gtk::Window,
    graph: ::relm::Component<graph::Widget>,
    status_bar: ::gtk::Statusbar,
    acquire: ::relm::Component<acquire::Widget>,
    generator: ::relm::Component<generator::Widget>,
    trigger: ::relm::Component<trigger::Widget>,
}

impl Application {
    fn update_status(&self, model: &Model) {
        let status = format!(
            "{} - {} V/div - {} µs/div",
            model.rate,
            model.scales.v_div(),
            model.scales.h_div()
        );

        self.status_bar.push(
            self.status_bar.get_context_id("sampling-rate"),
            status.as_str()
        );
    }

    pub fn draw(&self, model: &Model) {
        let graph = self.graph.widget();
        let width = graph.get_width();
        let height = graph.get_height();

        self.update_status(model);

        let image = ::cairo::ImageSurface::create(::cairo::Format::ARgb32, width as i32, height as i32);
        let context = ::cairo::Context::new(&image);

        self.transform(model.scales, &context, width, height);
        context.set_line_width(0.01);

        self.draw_panel(self.graph.widget(), &model, &context);
        self.draw_panel(self.trigger.widget(), &model, &context);
        self.draw_panel(self.generator.widget(), &model, &context);
        self.draw_panel(self.acquire.widget(), &model, &context);

        image.flush();
        graph.set_image(&image);
    }

    fn draw_panel(&self, panel: &Panel, model: &Model, context: &::cairo::Context) {
        context.save();
        panel.draw(&context, model);
        context.restore();
    }

    fn transform(&self, scales: ::Scales, context: &::cairo::Context, width: f64, height: f64) {
        context.set_matrix(::cairo::Matrix {
            xx: width / scales.get_width(),
            xy: 0.0,
            yy: -height / scales.get_height(),
            yx: 0.0,
            x0: scales.h.0 * width / scales.get_width(),
            y0: scales.v.1 * height / scales.get_height(),
        });
    }
}

impl ::relm::Widget for Application {
    type Model = Model;
    type ModelParam = ::redpitaya_scpi::Redpitaya;
    type Msg = Signal;
    type Root = ::gtk::Window;

    fn model(redpitaya: Self::ModelParam) -> Self::Model {
        let mut scales = ::Scales {
            h: (0.0, 0.0),
            v: (-5.0, 5.0),
            n_samples: redpitaya.data.buffer_size().unwrap(),
        };

        let rate = redpitaya.acquire.get_decimation()
            .unwrap()
            .into();
        scales.from_sampling_rate(rate);

        Model {
            attenuation: 1,
            rate: rate,
            redpitaya: redpitaya,
            scales: scales,
        }
    }

    fn root(&self) -> &Self::Root {
        &self.window
    }

    fn update(&mut self, event: Signal, model: &mut Self::Model) {
        match event {
            Signal::AcquireAttenuation(source, attenuation) => model.attenuation = attenuation,
            Signal::AcquireRate(rate) => {
                model.rate = rate;
                model.scales.from_sampling_rate(rate);
                self.draw(model);
            },

            Signal::GraphDraw => self.draw(model),

            Signal::TriggerAuto | Signal::TriggerSingle => self.acquire.widget().set_buffer(
                model.redpitaya.data.read_all(::redpitaya_scpi::acquire::Source::IN1)
            ),
            Signal::TriggerNormal => self.acquire.widget().set_buffer(
                model.redpitaya.data.read_oldest(::redpitaya_scpi::acquire::Source::IN1, 16_384)
            ),
            Signal::Quit => {
                model.redpitaya.acquire.stop();
                model.redpitaya.generator.stop(::redpitaya_scpi::generator::Source::OUT1);
                model.redpitaya.generator.stop(::redpitaya_scpi::generator::Source::OUT2);
                ::gtk::main_quit();
            },
        };
    }

    fn view(relm: &::relm::RemoteRelm<Self>, model: &Self::Model) -> Self {
        let main_box = ::gtk::Box::new(::gtk::Orientation::Horizontal, 0);

        let graph_page = ::gtk::EventBox::new();
        main_box.pack_start(&graph_page, true, true, 0);

        let graph = graph_page.add_widget::<graph::Widget, _>(&relm, ());
        connect!(graph@graph::Signal::Draw, relm, Signal::GraphDraw);

        let vbox = ::gtk::Box::new(::gtk::Orientation::Vertical, 0);
        main_box.pack_start(&vbox, false, false, 0);

        let notebook = ::gtk::Notebook::new();
        vbox.pack_start(&notebook, true, true, 0);

        let acquire_page = ::gtk::Box::new(::gtk::Orientation::Vertical, 0);
        acquire_page.set_border_width(10);

        let acquire = acquire_page.add_widget::<acquire::Widget, _>(&relm, model.redpitaya.acquire.clone());
        connect!(acquire@acquire::Signal::Data, relm, Signal::GraphDraw);
        connect!(acquire@acquire::Signal::Rate(rate), relm, Signal::AcquireRate(rate));
        connect!(acquire@acquire::Signal::Level(_, _), relm, Signal::GraphDraw);
        connect!(acquire@acquire::Signal::Attenuation(source, attenuation), relm, Signal::AcquireAttenuation(source, attenuation));

        notebook.append_page(
            &acquire_page,
            Some(&::gtk::Label::new(Some("Acquire")))
        );

        let generator_page = ::gtk::Box::new(::gtk::Orientation::Vertical, 0);
        generator_page.set_border_width(10);
        let generator = generator_page.add_widget::<generator::Widget, _>(&relm, model.redpitaya.generator.clone());
        connect!(generator@generator::Signal::Level(_, _), relm, Signal::GraphDraw);

        notebook.append_page(
            &generator_page,
            Some(&::gtk::Label::new(Some("Generator")))
        );

        let status_bar = ::gtk::Statusbar::new();
        vbox.pack_start(&status_bar, false, true, 0);

        let window = ::gtk::Window::new(::gtk::WindowType::Toplevel);
        window.set_title("Yellow Pitaya");
        window.add(&main_box);
        connect!(relm, window, connect_destroy(_), Signal::Quit);

        let trigger_page = ::gtk::Box::new(::gtk::Orientation::Vertical, 0);
        trigger_page.set_border_width(10);
        let trigger = trigger_page.add_widget::<trigger::Widget, _>(&relm, model.redpitaya.trigger.clone());
        connect!(trigger@trigger::Signal::Auto, relm, Signal::TriggerAuto);
        connect!(trigger@trigger::Signal::Normal, relm, Signal::TriggerNormal);
        connect!(trigger@trigger::Signal::Single, relm, Signal::TriggerSingle);

        notebook.append_page(
            &trigger_page,
            Some(&::gtk::Label::new(Some("Trigger")))
        );

        let application = Application {
            window: window,
            graph: graph,
            status_bar: status_bar,
            acquire: acquire,
            generator: generator,
            trigger: trigger,
        };

        application
    }

    fn init_view(&self, model: &mut Model) {
        match model.redpitaya.generator.get_amplitude(::redpitaya_scpi::generator::Source::OUT1) {
            Ok(amplitude) => self.generator.widget().amplitude.widget().set_value(amplitude as f64),
            Err(err) => error!("{}", err),
        };

        match model.redpitaya.generator.get_offset(::redpitaya_scpi::generator::Source::OUT1) {
            Ok(offset) => self.generator.widget().offset.widget().set_value(offset as f64),
            Err(err) => error!("{}", err),
        };

        match model.redpitaya.generator.get_frequency(::redpitaya_scpi::generator::Source::OUT1) {
            Ok(frequency) => self.generator.widget().frequency.widget().set_value(frequency as f64),
            Err(err) => error!("{}", err),
        };

        match model.redpitaya.generator.get_duty_cycle(::redpitaya_scpi::generator::Source::OUT1) {
            Ok(duty_cycle) => self.generator.widget().duty_cycle.widget().set_value(duty_cycle as f64),
            Err(err) => error!("{}", err),
        };

        match model.redpitaya.trigger.get_delay() {
            Ok(delay) => self.trigger.widget().delay.widget().set_value(delay as f64),
            Err(err) => error!("{}", err),
        };

        match model.redpitaya.trigger.get_level() {
            Ok(level) => self.trigger.widget().level.widget().set_value(level as f64),
            Err(err) => error!("{}", err),
        };

        match model.redpitaya.generator.get_frequency(::redpitaya_scpi::generator::Source::OUT1) {
            Ok(frequency) => self.generator.widget().frequency.widget().set_value(frequency as f64),
            Err(err) => error!("{}", err),
        };

        self.window.show_all();

        // @FIXME
        self.acquire.widget().palette.widget().fold();
        self.generator.widget().duty_cycle.widget().set_visible(false);
        self.generator.widget().palette.widget().fold();
        self.trigger.widget().single_button.set_visible(false);
    }
}
