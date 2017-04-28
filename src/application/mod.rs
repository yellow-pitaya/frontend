mod acquire;
mod color;
mod generator;
mod graph;
mod trigger;

use cairo::prelude::*;
use gtk::{
    BoxExt,
    ContainerExt,
    WidgetExt,
};
use relm::ContainerWidget;

trait Panel {
    fn draw(&self, context: &::cairo::Context, scales: ::Scales);
}

#[derive(Clone)]
pub enum Signal {
    AcquireStart,
    AcquireStop,
    GeneratorAmplitude(::redpitaya_scpi::generator::Source, f32),
    GeneratorOffset(::redpitaya_scpi::generator::Source, f32),
    GeneratorFrequency(::redpitaya_scpi::generator::Source, u32),
    GeneratorDutyCycle(::redpitaya_scpi::generator::Source, f32),
    GeneratorStart(::redpitaya_scpi::generator::Source),
    GeneratorStop(::redpitaya_scpi::generator::Source),
    GeneratorSignal(::redpitaya_scpi::generator::Source, ::redpitaya_scpi::generator::Form),
    GraphDraw,
    TriggerAuto,
    TriggerNormal,
    TriggerSingle,
    TriggerDelay(u16),
    TriggerLevel(f32),
    Quit,
}

impl ::relm::DisplayVariant for Signal {
    fn display_variant(&self) -> &'static str {
        match *self {
            Signal::AcquireStart => "Signal::AcquireStart",
            Signal::AcquireStop => "Signal::AcquireStop",
            Signal::GeneratorAmplitude(_, _) => "Signal::GeneratorAmplitude",
            Signal::GeneratorOffset(_, _) => "Signal::GeneratorOffset",
            Signal::GeneratorFrequency(_, _) => "Signal::GeneratorFrequency",
            Signal::GeneratorDutyCycle(_, _) => "Signal::GeneratorDutyCycle",
            Signal::GeneratorStart(_) => "Signal::GeneratorStart",
            Signal::GeneratorStop(_) => "Signal::GeneratorStop",
            Signal::GeneratorSignal(_, _) => "Signal::GeneratorSignal",
            Signal::TriggerAuto => "Signal::TriggerAuto",
            Signal::TriggerNormal => "Signal::TriggerNormal",
            Signal::TriggerSingle => "Signal::Single",
            Signal::TriggerDelay(_) => "Signal::TriggerDelay",
            Signal::TriggerLevel(_) => "Signal::TriggerLevel",
            Signal::GraphDraw => "Signal::GraphDraw",
            Signal::Quit => "Signal::Quit",
        }
    }
}

#[derive(Clone)]
pub struct Model {
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
    fn init(&self, model: &Model) {
        self.generator.widget().amplitude.widget().set_value(
            model.redpitaya.generator.get_amplitude(::redpitaya_scpi::generator::Source::OUT1) as f64
        );

        self.generator.widget().offset.widget().set_value(
            model.redpitaya.generator.get_offset(::redpitaya_scpi::generator::Source::OUT1) as f64
        );

        self.generator.widget().frequency.widget().set_value(
            model.redpitaya.generator.get_frequency(::redpitaya_scpi::generator::Source::OUT1) as f64
        );

        self.generator.widget().duty_cycle.widget().set_value(
            model.redpitaya.generator.get_duty_cycle(::redpitaya_scpi::generator::Source::OUT1) as f64
        );

        {
            let decimation = model.redpitaya.acquire.get_decimation();
            let status = format!(
                "{} - {} V/div - {} µs/div",
                decimation.get_sampling_rate(),
                model.scales.v_div(),
                model.scales.h_div()
            );

            self.status_bar.push(
                self.status_bar.get_context_id("sampling-rate"),
                status.as_str()
            );
        }

        self.trigger.widget().delay.widget().set_value(
            model.redpitaya.trigger.get_delay() as f64
        );

        self.trigger.widget().level.widget().set_value(
            model.redpitaya.trigger.get_level() as f64
        );

        self.trigger.widget().set_mode(trigger::Mode::Normal);

        self.window.show_all();

        // @FIXME
        self.acquire.widget().palette.widget().fold();
        self.generator.widget().duty_cycle.widget().set_visible(false);
        self.generator.widget().palette.widget().fold();
    }

    pub fn draw(&self, model: &Model) {
        let graph = self.graph.widget();
        let width = graph.get_width();
        let height = graph.get_height();

        let image = ::cairo::ImageSurface::create(::cairo::Format::ARgb32, width as i32, height as i32);
        let context = ::cairo::Context::new(&image);

        self.transform(model.scales, &context, width, height);
        context.set_line_width(0.01);

        self.draw_panel(self.graph.widget(), model.scales, &context);
        self.draw_panel(self.trigger.widget(), model.scales, &context);
        self.draw_panel(self.generator.widget(), model.scales, &context);
        self.draw_panel(self.acquire.widget(), model.scales, &context);

        image.flush();
        graph.set_image(&image);
    }

    fn draw_panel(&self, panel: &Panel, scales: ::Scales, context: &::cairo::Context) {
        context.save();
        panel.draw(&context, scales);
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
            n_samples: redpitaya.data.buffer_size(),
        };
        let decimation = redpitaya.acquire.get_decimation();
        scales.from_decimation(decimation);

        Model {
            redpitaya: redpitaya,
            scales: scales,
        }
    }

    fn root(&self) -> &Self::Root {
        &self.window
    }

    fn update(&mut self, event: Signal, model: &mut Self::Model) {
        match event {
            Signal::AcquireStart => model.redpitaya.acquire.start(),
            Signal::AcquireStop => model.redpitaya.acquire.stop(),

            Signal::GeneratorAmplitude(source, value) => model.redpitaya.generator.set_amplitude(source, value),
            Signal::GeneratorOffset(source, value) => model.redpitaya.generator.set_offset(source, value),
            Signal::GeneratorFrequency(source, value) => model.redpitaya.generator.set_frequency(source, value),
            Signal::GeneratorDutyCycle(source, value) => model.redpitaya.generator.set_duty_cycle(source, value),
            Signal::GeneratorStart(source) => model.redpitaya.generator.start(source),
            Signal::GeneratorStop(source) => model.redpitaya.generator.stop(source),
            Signal::GeneratorSignal(source, form) => model.redpitaya.generator.set_form(source, form),

            Signal::GraphDraw => self.draw(model),

            Signal::TriggerAuto | Signal::TriggerSingle => if model.redpitaya.acquire.is_started() {
                    self.acquire.widget().set_buffer(
                        model.redpitaya.data.read_all(::redpitaya_scpi::acquire::Source::IN1)
                    );
            },
            Signal::TriggerNormal => if model.redpitaya.acquire.is_started() {
                self.acquire.widget().set_buffer(
                    model.redpitaya.data.read_oldest(::redpitaya_scpi::acquire::Source::IN1, 16_384)
                );
            },
            Signal::TriggerDelay(value) => model.redpitaya.trigger.set_delay(value),
            Signal::TriggerLevel(value) => model.redpitaya.trigger.set_level(value),

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
        let acquire = acquire_page.add_widget::<acquire::Widget, _>(&relm, ());
        connect!(acquire@acquire::Signal::Data, relm, Signal::GraphDraw);
        connect!(acquire@acquire::Signal::Level(_, _), relm, Signal::GraphDraw);
        connect!(acquire@acquire::Signal::Start, relm, Signal::AcquireStart);
        connect!(acquire@acquire::Signal::Stop, relm, Signal::AcquireStop);

        notebook.append_page(
            &acquire_page,
            Some(&::gtk::Label::new(Some("Acquire")))
        );

        let generator_page = ::gtk::Box::new(::gtk::Orientation::Vertical, 0);
        generator_page.set_border_width(10);
        let generator = generator_page.add_widget::<generator::Widget, _>(&relm, ());
        connect!(generator@generator::Signal::Start(source), relm, Signal::GeneratorStart(source));
        connect!(generator@generator::Signal::Stop(source), relm, Signal::GeneratorStop(source));
        connect!(generator@generator::Signal::Amplitude(source, value), relm, Signal::GeneratorAmplitude(source, value));
        connect!(generator@generator::Signal::Offset(source, value), relm, Signal::GeneratorOffset(source, value));
        connect!(generator@generator::Signal::Frequency(source, value), relm, Signal::GeneratorFrequency(source, value));
        connect!(generator@generator::Signal::DutyCycle(source, value), relm, Signal::GeneratorDutyCycle(source, value));
        connect!(generator@generator::Signal::Signal(source, value), relm, Signal::GeneratorSignal(source, value));

        notebook.append_page(
            &generator_page,
            Some(&::gtk::Label::new(Some("Generator")))
        );

        let status_bar = ::gtk::Statusbar::new();
        vbox.pack_start(&status_bar, false, true, 0);

        let window = ::gtk::Window::new(::gtk::WindowType::Toplevel);
        window.add(&main_box);
        connect!(relm, window, connect_destroy(_), Signal::Quit);

        let trigger_page = ::gtk::Box::new(::gtk::Orientation::Vertical, 0);
        trigger_page.set_border_width(10);
        let trigger = trigger_page.add_widget::<trigger::Widget, _>(&relm, ());
        connect!(trigger@trigger::Signal::Auto, relm, Signal::TriggerAuto);
        connect!(trigger@trigger::Signal::Normal, relm, Signal::TriggerNormal);
        connect!(trigger@trigger::Signal::Single, relm, Signal::TriggerSingle);
        connect!(trigger@trigger::Signal::Delay(value), relm, Signal::TriggerDelay(value));
        connect!(trigger@trigger::Signal::Level(value), relm, Signal::TriggerLevel(value));

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

        application.init(model);

        application
    }

    fn init_view(&self) {
        // @FIXME
        //self.init();
    }
}
